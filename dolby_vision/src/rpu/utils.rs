use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use anyhow::{bail, Result};

use super::dovi_rpu::DoviRpu;

pub fn parse_rpu_file<P: AsRef<Path>>(input: P) -> Result<Vec<DoviRpu>> {
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

    let offsets: Vec<usize> = data
        .windows(4)
        .enumerate()
        .filter_map(|(i, chunk)| {
            if matches!(chunk, &[0, 0, 0, 1]) {
                Some(i)
            } else {
                None
            }
        })
        .collect();

    if offsets.is_empty() {
        bail!("No NALU start codes found in the file. Maybe not a valid RPU?");
    }

    let count = offsets.len();
    let last = *offsets.last().unwrap();
    let mut warning_error = None;

    let mut rpus: Vec<DoviRpu> = Vec::with_capacity(count);
    let parsed_rpus_iter = offsets
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
                if warning_error.is_none() {
                    warning_error = Some(format!("Found invalid RPU: Index {}, error: {}", i, e,))
                }
            }

            res.ok()
        });
    rpus.extend(parsed_rpus_iter);

    if count > 0 && rpus.len() == count {
        Ok(rpus)
    } else if count == 0 {
        bail!("No RPU found");
    } else if let Some(error) = warning_error {
        bail!("{}", error);
    } else {
        bail!(
            "Number of valid RPUs different from total: expected {} got {}",
            count,
            rpus.len()
        );
    }
}
