use anyhow::{bail, ensure, format_err, Result};
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Write};
use std::path::Path;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};

mod utils;
use utils::nits_to_pq;

pub const MAGIC_CODE: &str = "mvr+";

#[derive(Debug, Default)]
pub struct MadVRMeasurements {
    pub header: MadVRHeader,
    pub scenes: Vec<MadVRScene>,
    pub frames: Vec<MadVRFrame>,
}

#[derive(Debug, Default)]
pub struct MadVRHeader {
    pub version: u32,
    pub header_size: u32,
    pub scene_count: u32,
    pub frame_count: u32,
    pub flags: u32,
    pub maxcll: u32,
    pub maxfall: u32,
    pub avgfall: u32,
    pub target_peak_nits: u32,
}

#[derive(Debug, Default)]
pub struct MadVRScene {
    pub start: u32,
    pub end: u32,
    pub peak_nits: u32,

    pub length: usize,
    pub max_pq: f64,
    pub avg_pq: f64,
}

#[derive(Debug, Default)]
pub struct MadVRFrame {
    pub peak_pq_2020: f64,
    pub peak_pq_dcip3: Option<f64>,
    pub peak_pq_709: Option<f64>,
    pub lum_histogram: Vec<f64>,
    pub hue_histogram: Option<Vec<f64>>,
    pub target_nits: Option<u16>,

    pub avg_pq: f64,
    pub target_pq: f64,
}

impl MadVRMeasurements {
    pub fn parse_file(path: &Path) -> Result<MadVRMeasurements> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;

        // Should never be this large, avoid mistakes
        if metadata.len() > 250_000_000 {
            bail!("madvr_parse: file probably too large");
        }

        let mut reader = BufReader::new(file);

        // Should be small enough to fit in the memory
        let mut data = vec![0; metadata.len() as usize];
        reader.read_exact(&mut data)?;

        Self::parse_measurements(&data)
    }

    pub fn parse_measurements(data: &[u8]) -> Result<MadVRMeasurements> {
        let mut reader = Cursor::new(&data[4..]);

        let magic = std::str::from_utf8(&data[..4])?;

        ensure!(
            magic == MAGIC_CODE,
            "invalid magic code {}, expected {}",
            magic,
            MAGIC_CODE
        );

        let mut measurements = MadVRMeasurements {
            header: MadVRHeader::parse(&mut reader)?,
            ..Default::default()
        };

        measurements.scenes = MadVRScene::parse_scenes(&measurements.header, &mut reader)?;
        measurements.frames = MadVRFrame::parse_frames(&measurements.header, &mut reader)?;

        if measurements.header.flags == 3 {
            let remaining = data[4..].len() - (reader.position() as usize);
            ensure!(
                remaining / 2 == measurements.frames.len(),
                "madvr_parse: invalid remaining bytes for custom per-frame target nits"
            );

            MadVRFrame::parse_custom_frame_target_nits(&mut measurements.frames, &mut reader)?;
        }

        measurements.compute_max_scene_avg()?;

        Ok(measurements)
    }

    fn compute_max_scene_avg(&mut self) -> Result<()> {
        let frame_count = self.frames.len();

        for s in self.scenes.iter_mut() {
            let frames = s.get_frames(frame_count, &self.frames)?;

            // Keep the max avg of all the frames in the scene
            s.avg_pq = frames
                .iter()
                .map(|f| f.avg_pq)
                .reduce(f64::max)
                .ok_or_else(|| format_err!("no frames for scene"))?;
        }

        Ok(())
    }

    pub fn write_measurements(&self) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(
            (10 * 4) + (self.scenes.len() * (3 * 4)) + (self.frames.len() * (32 * 2)),
        );

        let magic = MAGIC_CODE.as_bytes().read_u32::<LE>()?;
        out.write_u32::<LE>(magic)?;

        self.header.write(&mut out)?;
        MadVRScene::write_scenes(&self.scenes, &mut out)?;
        MadVRFrame::write_frames(&self.header, &self.frames, &mut out)?;

        if self.header.flags == 3 {
            MadVRFrame::write_custom_frame_target_nits(&self.frames, &mut out)?;
        }

        out.flush()?;

        Ok(out)
    }
}

impl MadVRHeader {
    fn parse(reader: &mut dyn Read) -> Result<MadVRHeader> {
        let mut header = MadVRHeader {
            version: reader.read_u32::<LE>()?,
            header_size: reader.read_u32::<LE>()?,
            scene_count: reader.read_u32::<LE>()?,
            frame_count: reader.read_u32::<LE>()?,
            flags: reader.read_u32::<LE>()?,
            maxcll: reader.read_u32::<LE>()?,
            ..Default::default()
        };

        ensure!(header.flags != 0, "incomplete measurement file");

        if header.version >= 5 {
            header.maxfall = reader.read_u32::<LE>()?;
            header.avgfall = reader.read_u32::<LE>()?;

            if header.version >= 6 {
                header.target_peak_nits = reader.read_u32::<LE>()?;
            }
        }

        Ok(header)
    }

    fn write(&self, writer: &mut dyn Write) -> Result<()> {
        ensure!(self.flags != 0, "can only write complete measurement files");

        writer.write_u32::<LE>(self.version)?;
        writer.write_u32::<LE>(self.header_size)?;
        writer.write_u32::<LE>(self.scene_count)?;
        writer.write_u32::<LE>(self.frame_count)?;
        writer.write_u32::<LE>(self.flags)?;
        writer.write_u32::<LE>(self.maxcll)?;

        if self.version >= 5 {
            writer.write_u32::<LE>(self.maxfall)?;
            writer.write_u32::<LE>(self.avgfall)?;

            if self.version >= 6 {
                writer.write_u32::<LE>(self.target_peak_nits)?;
            }
        }

        Ok(())
    }
}

impl MadVRScene {
    fn parse_scenes(header: &MadVRHeader, reader: &mut dyn Read) -> Result<Vec<MadVRScene>> {
        let mut scenes: Vec<MadVRScene> = Vec::new();

        for _ in 0..header.scene_count {
            let scene = MadVRScene {
                start: reader.read_u32::<LE>()?,
                ..Default::default()
            };

            scenes.push(scene);
        }

        for s in scenes.iter_mut() {
            s.end = reader.read_u32::<LE>()? - 1;

            s.length = (s.end - s.start + 1) as usize;
        }

        for s in scenes.iter_mut() {
            s.peak_nits = reader.read_u32::<LE>()?;

            s.max_pq = nits_to_pq(s.peak_nits);
        }

        Ok(scenes)
    }

    fn write_scenes(scenes: &[MadVRScene], writer: &mut dyn Write) -> Result<()> {
        for s in scenes {
            writer.write_u32::<LE>(s.start)?;
        }

        for s in scenes {
            writer.write_u32::<LE>(s.end + 1)?;
        }

        for s in scenes {
            writer.write_u32::<LE>(s.peak_nits)?;
        }

        Ok(())
    }

    pub fn get_frames<'a>(
        &self,
        frame_count: usize,
        frames: &'a [MadVRFrame],
    ) -> Result<&'a [MadVRFrame]> {
        let (start, end) = (self.start as usize, self.end as usize);

        ensure!(
            end < frame_count,
            "scene end higher than frame count: {} > {}",
            end,
            frame_count
        );

        Ok(&frames[start..=end])
    }
}

impl MadVRFrame {
    fn parse_frames(header: &MadVRHeader, reader: &mut dyn Read) -> Result<Vec<MadVRFrame>> {
        let mut frames = Vec::new();

        let sdr_peak_pq = nits_to_pq(100);
        let hdr_peak_pq = 1.0;

        for _ in 0..header.frame_count {
            let mut frame = if header.version >= 6 {
                MadVRFrame {
                    peak_pq_2020: (reader.read_u16::<LE>()? as f64) / 64000.0,
                    peak_pq_dcip3: Some((reader.read_u16::<LE>()? as f64) / 64000.0),
                    peak_pq_709: Some((reader.read_u16::<LE>()? as f64) / 64000.0),
                    ..Default::default()
                }
            } else {
                MadVRFrame {
                    peak_pq_2020: (reader.read_u16::<LE>()? as f64) / 64000.0,
                    ..Default::default()
                }
            };

            if header.version >= 5 {
                let sdr_step: f64 = sdr_peak_pq / 64.0;
                let hdr_step: f64 = (hdr_peak_pq - sdr_peak_pq) / 192.0;

                // Value is in the middle of the histogram bin
                let sdr_step = sdr_step + (sdr_step / 2.0);
                let hdr_step = hdr_step + (hdr_step / 2.0);

                frame.lum_histogram = MadVRFrame::parse_histogram(256, reader)?;
                frame.hue_histogram = Some(MadVRFrame::parse_histogram(31, reader)?);

                frame.avg_pq = frame
                    .lum_histogram
                    .iter()
                    .enumerate()
                    .filter(|(i, p)| !(*i == 0 && **p > 2.0 && **p < 30.0)) // Filter out black bars
                    .map(|(i, percent)| {
                        let pq_value = if i <= 64 {
                            (i as f64) * sdr_step
                        } else {
                            sdr_peak_pq + (((i - 63) as f64) * hdr_step)
                        };

                        pq_value * (percent / 100.0)
                    })
                    .sum();
            } else {
                let step = hdr_peak_pq / 31.0;
                let step = step + (step / 2.0);

                frame.lum_histogram = MadVRFrame::parse_histogram(31, reader)?;

                frame.avg_pq = frame
                    .lum_histogram
                    .iter()
                    .enumerate()
                    .map(|(i, percent)| ((i as f64) * step) * (percent / 100.0))
                    .sum();
            }

            // Adjust depending on the sum of histogram bars
            let percent_sum: f64 = frame.lum_histogram.iter().sum();
            frame.avg_pq = (frame.avg_pq * (100.0 / percent_sum)).min(1.0);

            frames.push(frame);
        }

        Ok(frames)
    }

    fn parse_histogram(length: usize, reader: &mut dyn Read) -> Result<Vec<f64>> {
        let mut histogram: Vec<f64> = Vec::new();

        for _ in 0..length {
            let v = (reader.read_u16::<LE>()? as f64) / 640.0;
            histogram.push(v);
        }

        Ok(histogram)
    }

    fn write_frames(
        header: &MadVRHeader,
        frames: &[MadVRFrame],
        writer: &mut dyn Write,
    ) -> Result<()> {
        for f in frames {
            writer.write_u16::<LE>((f.peak_pq_2020 * 64000.0).round() as u16)?;

            if header.version >= 6 {
                ensure!(
                    f.peak_pq_dcip3.is_some() && f.peak_pq_709.is_some(),
                    "missing different gamut frame peaks for v6"
                );

                writer.write_u16::<LE>((f.peak_pq_dcip3.unwrap() * 64000.0).round() as u16)?;
                writer.write_u16::<LE>((f.peak_pq_709.unwrap() * 64000.0).round() as u16)?;
            }

            if header.version >= 5 {
                ensure!(
                    f.lum_histogram.len() == 256,
                    "lum histogram has to be size 256 for v5+"
                );
                ensure!(f.hue_histogram.is_some(), "missing hue histogram for v6");

                let hue_histogram = f.hue_histogram.as_ref().unwrap();
                ensure!(hue_histogram.len() == 31, "hue histogram has to be size 31");

                MadVRFrame::write_histogram(&f.lum_histogram, writer)?;
                MadVRFrame::write_histogram(hue_histogram, writer)?;
            } else {
                ensure!(
                    f.lum_histogram.len() == 31,
                    "lum histogram has to be size 31 for versions below 5"
                );
                MadVRFrame::write_histogram(&f.lum_histogram, writer)?;
            }
        }

        Ok(())
    }

    fn write_histogram(histogram: &[f64], writer: &mut dyn Write) -> Result<()> {
        for v in histogram {
            writer.write_u16::<LE>((v * 640.0).round() as u16)?;
        }

        Ok(())
    }

    fn parse_custom_frame_target_nits(
        frames: &mut [MadVRFrame],
        reader: &mut dyn Read,
    ) -> Result<()> {
        for f in frames.iter_mut() {
            let target_nits = reader.read_u16::<LE>()?;

            f.target_nits = Some(target_nits);
            f.target_pq = nits_to_pq(target_nits as u32);
        }

        Ok(())
    }

    fn write_custom_frame_target_nits(frames: &[MadVRFrame], writer: &mut dyn Write) -> Result<()> {
        for (i, f) in frames.iter().enumerate() {
            ensure!(
                f.target_nits.is_some(),
                "madvr_parse: missing target nits for frame {}",
                i
            );

            writer.write_u16::<LE>(f.target_nits.unwrap())?;
        }

        Ok(())
    }
}
