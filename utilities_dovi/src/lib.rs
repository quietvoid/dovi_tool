use anyhow::{bail, Result};
use dolby_vision::rpu::dovi_rpu::DoviRpu;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use hevc_parser::{HevcParser, NALUStartCode};

pub fn parse_rpu_file(input: &Path) -> Result<Option<Vec<DoviRpu>>> {
    let rpu_file = File::open(input)?;
    let metadata = rpu_file.metadata()?;

    // Should never be this large, avoid mistakes
    if metadata.len() > 250_000_000 {
        bail!("Input file probably too large");
    }

    let mut reader = BufReader::new(rpu_file);

    // Should be small enough to fit in the memory
    let mut data = vec![0; metadata.len() as usize];
    reader.read_exact(&mut data)?;

    let mut offsets = Vec::with_capacity(200_000);
    let mut parser = HevcParser::with_nalu_start_code(NALUStartCode::Length4);

    parser.get_offsets(&data, &mut offsets);

    if offsets.is_empty() {
        bail!("No NALU start codes found in the file. Maybe not a valid RPU?");
    }

    let count = offsets.len();
    let last = *offsets.last().unwrap();
    let mut warned = false;

    let rpus: Vec<DoviRpu> = offsets
        .iter()
        .enumerate()
        .map(|(index, offset)| {
            let size = if offset == &last {
                data.len() - offset
            } else {
                offsets[index + 1] - offset
            };

            let start = *offset;
            let end = start + size;

            DoviRpu::parse_unspec62_nalu(&data[start..end])
        })
        .enumerate()
        .filter_map(|(i, res)| {
            if let Err(e) = &res {
                if !warned {
                    println!("Error parsing frame {}: {}", i, e);
                    warned = true;
                }
            }

            res.ok()
        })
        .collect();

    if count > 0 && rpus.len() == count {
        Ok(Some(rpus))
    } else if count == 0 {
        bail!("No RPU found");
    } else {
        bail!(
            "Number of valid RPUs different from total: expected {} got {}",
            count,
            rpus.len()
        );
    }
}
