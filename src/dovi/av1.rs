//! Dolby Vision in AV1 bitstreams
//!
//! RPUs travel in ITU-T T.35 metadata OBUs, one per temporal unit. Reading,
//! placing and writing those OBUs is `av1_parser`'s job; this module only knows
//! which of them carry an RPU, and what to do with it.

use std::fs::File;
use std::io::{BufRead, BufWriter, Write, stdout};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use indicatif::ProgressBar;

use av1_parser::io::processor::{Av1Processor, Av1ProcessorOpts, ContainerProcessorOpts};
use av1_parser::io::writer::Av1Writer;
use av1_parser::io::{IoFormat as Av1Format, IoProcessor, StreamInfo};
use av1_parser::metadata::{ITU_T35_COUNTRY_CODE_US, Metadata, itu_t35_metadata_obu};
use av1_parser::{Obu, TemporalUnit};
use dolby_vision::av1::ITU_T35_DOVI_RPU_PAYLOAD_HEADER;
use dolby_vision::rpu::dovi_rpu::DoviRpu;
use dolby_vision::rpu::utils::parse_rpu_file;
use hevc_parser::io::IoFormat;

use super::general_read_write::DoviProcessorError;
use super::{CliOptions, apply_options, write_rpu_file};

/// HDR10+: provider code, provider oriented code, application identifier
const HDR10PLUS_T35_HEADER: &[u8] = &[0x00, 0x3C, 0x00, 0x01, 0x04];

/// An AV1 input, or stdin that turned out to be something else
pub enum Input {
    Av1(Av1Input),
    /// Not AV1. For stdin, the reader still holds everything that was read
    /// to find out.
    Other(Option<Box<dyn BufRead>>),
}

pub struct Av1Input {
    path: PathBuf,
    format: Av1Format,
    stdin: Option<Box<dyn BufRead>>,
}

impl Input {
    /// Look at `path` - a file, or `-` for stdin - and tell AV1 apart from HEVC
    pub fn open(path: &Path, track_number: Option<usize>) -> Result<Self> {
        if path == Path::new("-") {
            let probed = av1_parser::io::probe_stdin()?;

            return Ok(match probed.format {
                Some(_) => Self::Av1(Av1Input {
                    path: path.to_path_buf(),
                    format: Av1Format::RawStdin,
                    stdin: Some(probed.reader),
                }),
                None => Self::Other(Some(probed.reader)),
            });
        }

        if !av1_parser::io::is_av1_file(path, track_number) {
            return Ok(Self::Other(None));
        }

        Ok(Self::Av1(Av1Input {
            path: path.to_path_buf(),
            format: av1_parser::io::format_from_path(path, track_number)?,
            stdin: None,
        }))
    }
}

impl Av1Input {
    pub fn format(&self) -> Av1Format {
        self.format
    }

    fn progress_bar(&self) -> Result<ProgressBar> {
        let format = if self.stdin.is_some() {
            IoFormat::RawStdin
        } else {
            IoFormat::Raw
        };

        super::initialize_progress_bar(&format, &self.path)
    }

    fn process(&mut self, opts: Av1ProcessorOpts, processor: &mut dyn IoProcessor) -> Result<()> {
        let mut av1_processor = Av1Processor::new(self.format, opts);

        match self.stdin.as_mut() {
            Some(reader) => av1_processor.process_io(reader, processor),
            None => av1_processor.process_file(processor, Some(&self.path)),
        }
    }

    /// Output file name for a bitstream written from this input
    pub fn default_output(&self, stem: &str) -> PathBuf {
        let ext = if self.format == Av1Format::Ivf {
            "ivf"
        } else {
            "av1"
        };

        PathBuf::from(format!("{stem}.{ext}"))
    }

    /// `extract-rpu`: every RPU, in presentation order, as an RPU file
    pub fn extract_rpu(
        &mut self,
        rpu_out: &Path,
        options: CliOptions,
        limit: Option<u64>,
        track_number: Option<usize>,
    ) -> Result<()> {
        let progress_bar = self.progress_bar()?;
        let mut processor = RpuExtractor {
            input: self.path.clone(),
            options,
            progress_bar,
            rpu_out: rpu_out.to_path_buf(),
            rpus: Vec::new(),
        };

        let opts = Av1ProcessorOpts {
            limit,
            container_opts: Some(ContainerProcessorOpts { track_number }),
        };

        self.process(opts, &mut processor)
    }

    /// `remove` and `convert`: the bitstream with its RPUs taken out, or
    /// converted according to the options
    pub fn rewrite(&mut self, output: &Path, action: RpuAction, options: CliOptions) -> Result<()> {
        let progress_bar = self.progress_bar()?;
        let mut processor = BitstreamRewriter {
            input: self.path.clone(),
            options,
            progress_bar,
            action,
            output: output.to_path_buf(),
            writer: None,
        };

        self.process(Av1ProcessorOpts::default(), &mut processor)
    }

    /// `inject-rpu`: one RPU per temporal unit, replacing any already there
    pub fn inject_rpu(&mut self, rpu_in: &Path, output: &Path, options: CliOptions) -> Result<()> {
        println!("Parsing RPU file...");
        stdout().flush().ok();

        let rpus = parse_rpu_file(rpu_in)?;

        println!("Processing input video for frame order info...");
        stdout().flush().ok();

        let mut counter = UnitCounter {
            input: self.path.clone(),
            units: 0,
        };
        self.process(Av1ProcessorOpts::default(), &mut counter)?;

        if counter.units as usize != rpus.len() {
            println!(
                "\nWarning: mismatched lengths. video {}, RPU {}",
                counter.units,
                rpus.len()
            );

            if rpus.len() < counter.units as usize {
                println!("Metadata will be duplicated at the end to match video length\n");
            } else {
                println!("Metadata will be skipped at the end to match video length\n");
            }
        }

        println!("Rewriting file with interleaved RPU OBUs..");
        stdout().flush().ok();

        let progress_bar = self.progress_bar()?;
        let mut processor = BitstreamRewriter {
            input: self.path.clone(),
            options,
            progress_bar,
            action: RpuAction::Inject(rpus),
            output: output.to_path_buf(),
            writer: None,
        };

        self.process(Av1ProcessorOpts::default(), &mut processor)
    }
}

/// The RPU in a metadata OBU: the whole T.35 message, starting at the country code
pub fn dovi_rpu_t35(obu: &Obu) -> Option<&[u8]> {
    let t35 = Metadata::from_obu(obu)?.itu_t35()?;

    (t35.country_code == ITU_T35_COUNTRY_CODE_US
        && t35.payload().starts_with(ITU_T35_DOVI_RPU_PAYLOAD_HEADER))
    .then_some(t35.bytes)
}

pub fn is_dovi_rpu_obu(obu: &Obu) -> bool {
    dovi_rpu_t35(obu).is_some()
}

pub fn is_hdr10plus_obu(obu: &Obu) -> bool {
    Metadata::from_obu(obu)
        .and_then(|meta| meta.itu_t35())
        .is_some_and(|t35| {
            t35.country_code == ITU_T35_COUNTRY_CODE_US
                && t35.payload().starts_with(HDR10PLUS_T35_HEADER)
        })
}

/// A metadata OBU carrying `rpu`
pub fn dovi_rpu_obu(rpu: &DoviRpu) -> Result<Obu> {
    Ok(itu_t35_metadata_obu(
        &rpu.write_av1_rpu_metadata_obu_t35_complete()?,
    ))
}

/// What happens to the RPUs of a bitstream that is written back
pub enum RpuAction {
    Remove,
    Convert,
    Inject(Vec<DoviRpu>),
}

struct RpuExtractor {
    input: PathBuf,
    options: CliOptions,
    progress_bar: ProgressBar,

    rpu_out: PathBuf,
    /// Encoded as `UNSPEC62` NAL units, the RPU file format
    rpus: Vec<Vec<u8>>,
}

impl IoProcessor for RpuExtractor {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    fn process_temporal_unit(&mut self, tu: TemporalUnit) -> Result<()> {
        let mut found = false;

        for t35 in tu.obus.iter().filter_map(dovi_rpu_t35) {
            if found {
                println!(
                    "Warning: Unexpected RPU OBU found for frame {}. Discarding.",
                    tu.index
                );
                continue;
            }

            found = true;

            let mut rpu = DoviRpu::parse_itu_t35_dovi_metadata_obu(t35)?;

            // As for HEVC: the options only apply with a mode or an edit config
            if self.options.mode.is_some() || self.options.edit_config.is_some() {
                apply_options(&self.options, &mut rpu)?;
            }

            self.rpus.push(rpu.write_hevc_unspec62_nalu()?);
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        self.progress_bar.finish_and_clear();

        if self.rpus.is_empty() {
            bail!(DoviProcessorError::NoRpuFound);
        }

        write_rpu_file(&self.rpu_out, std::mem::take(&mut self.rpus))
    }
}

struct BitstreamRewriter {
    input: PathBuf,
    options: CliOptions,
    progress_bar: ProgressBar,

    action: RpuAction,
    output: PathBuf,
    writer: Option<Av1Writer<BufWriter<File>>>,
}

impl BitstreamRewriter {
    fn converted(&self, obu: Obu) -> Result<Obu> {
        if self.options.mode.is_none() && self.options.edit_config.is_none() {
            return Ok(obu);
        }

        let t35 = dovi_rpu_t35(&obu).expect("only called for RPU OBUs");
        let mut rpu = DoviRpu::parse_itu_t35_dovi_metadata_obu(t35)?;
        apply_options(&self.options, &mut rpu)?;

        dovi_rpu_obu(&rpu)
    }
}

impl IoProcessor for BitstreamRewriter {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, delta: u64) {
        self.progress_bar.inc(delta);
    }

    fn process_stream_info(&mut self, info: &StreamInfo) -> Result<()> {
        let file = File::create(&self.output).expect("Can't create file");
        self.writer = Some(Av1Writer::new(
            BufWriter::with_capacity(100_000, file),
            info,
        )?);

        Ok(())
    }

    fn process_temporal_unit(&mut self, mut tu: TemporalUnit) -> Result<()> {
        let obus = std::mem::take(&mut tu.obus);

        for obu in obus {
            if self.options.drop_hdr10plus && is_hdr10plus_obu(&obu) {
                continue;
            }

            if is_dovi_rpu_obu(&obu) {
                match self.action {
                    RpuAction::Convert => tu.obus.push(self.converted(obu)?),
                    RpuAction::Remove | RpuAction::Inject(_) => {}
                }
            } else {
                tu.obus.push(obu);
            }
        }

        if let RpuAction::Inject(rpus) = &self.action {
            // Past the end of the list, the last RPU is repeated
            if let Some(rpu) = rpus.get(tu.index as usize).or(rpus.last()) {
                tu.insert_metadata(dovi_rpu_obu(rpu)?);
            }
        }

        self.writer
            .as_mut()
            .expect("stream info comes first")
            .write_temporal_unit(&tu)
    }

    fn finalize(&mut self) -> Result<()> {
        self.progress_bar.finish_and_clear();

        if let Some(writer) = self.writer.as_mut() {
            writer.flush()?;
        }

        Ok(())
    }
}

struct UnitCounter {
    input: PathBuf,
    units: u64,
}

impl IoProcessor for UnitCounter {
    fn input(&self) -> &PathBuf {
        &self.input
    }

    fn update_progress(&mut self, _delta: u64) {}

    fn process_temporal_unit(&mut self, _tu: TemporalUnit) -> Result<()> {
        self.units += 1;
        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        Ok(())
    }
}
