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
    let file_size_bytes = metadata.len() as usize;

    let mut reader = BufReader::new(rpu_file);

    let chunk_size = 100_000;
    let mut main_buf = vec![0; chunk_size];
    let mut chunk = Vec::with_capacity(chunk_size);
    let mut end = Vec::with_capacity(chunk_size);

    let mut offsets_count = 0;
    // Estimate RPU count from file size
    let mut rpus: Vec<DoviRpu> = Vec::with_capacity(chunk_size / 400);
    let mut warning_error = None;

    while let Ok(n) = reader.read(&mut main_buf) {
        let read_bytes = n;
        if read_bytes == 0 && end.is_empty() && chunk.is_empty() {
            break;
        }

        if read_bytes < chunk_size {
            chunk.extend_from_slice(&main_buf[..read_bytes]);
        } else {
            chunk.extend_from_slice(&main_buf);
        }

        let mut offsets: Vec<usize> = chunk
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
            bail!("No NALU start codes found in chunk. Maybe not a valid RPU?");
        }

        let last = if read_bytes < chunk_size {
            *offsets.last().unwrap()
        } else {
            let last = offsets.pop().unwrap();

            end.clear();
            end.extend_from_slice(&chunk[last..]);

            last
        };

        let count = offsets.len();
        let parsed_rpus_iter = offsets
            .iter()
            .enumerate()
            .map(|(index, offset)| {
                let size = if offset == &last {
                    chunk.len() - offset
                } else {
                    let size = if index == count - 1 {
                        last - offset
                    } else {
                        offsets[index + 1] - offset
                    };

                    match &chunk[offset + size - 1..offset + size + 3] {
                        [0, 0, 0, 1] => size - 1,
                        _ => size,
                    }
                };

                let start = *offset;
                let end = start + size;

                DoviRpu::parse_unspec62_nalu(&chunk[start..end])
            })
            .enumerate()
            .filter_map(|(i, res)| {
                if let Err(e) = &res {
                    if warning_error.is_none() {
                        warning_error = Some(format!("Found invalid RPU: Index {i}, error: {e}"))
                    }
                }

                res.ok()
            });
        rpus.extend(parsed_rpus_iter);

        if warning_error.is_some() {
            offsets_count += count;
            break;
        } else if rpus.is_empty() {
            bail!("No valid RPUs parsed for chunk, assuming invalid RPU file.");
        }

        if offsets_count == 0 && file_size_bytes > chunk_size {
            rpus.reserve((metadata.len() as usize - chunk_size) / 400);
        }
        offsets_count += count;

        chunk.clear();

        if !end.is_empty() {
            chunk.extend_from_slice(&end);
            end.clear()
        }
    }

    if offsets_count > 0 && rpus.len() == offsets_count {
        Ok(rpus)
    } else if offsets_count == 0 {
        bail!("No RPU found");
    } else if let Some(error) = warning_error {
        bail!("{}", error);
    } else {
        bail!(
            "Number of valid RPUs different from total: expected {} got {}",
            offsets_count,
            rpus.len()
        );
    }
}
