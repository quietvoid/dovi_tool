use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::{collections::HashMap, path::PathBuf};

pub struct Editor {
    input: PathBuf,
    json_path: PathBuf,
    rpu_out: PathBuf,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct EditConfig {
    #[serde(default)]
    mode: u8,

    #[serde(skip_serializing_if = "Option::is_none")]
    active_area: Option<ActiveArea>,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ActiveArea {
    #[serde(default)]
    crop: bool,

    edits: HashMap<String, u16>,
}

impl Editor {
    pub fn edit(input: PathBuf, json_path: PathBuf, rpu_out: Option<PathBuf>) {
        let out_path = if let Some(out_path) = rpu_out {
            out_path
        } else {
            PathBuf::from(format!(
                "{}{}",
                input.file_stem().unwrap().to_str().unwrap(),
                "_modified.bin"
            ))
        };

        let editor = Editor {
            input,
            json_path,
            rpu_out: out_path,
        };

        let json_file = File::open(editor.json_path).unwrap();
        let config: EditConfig = serde_json::from_reader(&json_file).unwrap();

        let rpu_file = File::open(editor.input).unwrap();
        let metadata = rpu_file.metadata().unwrap();

        // Should never be this large, avoid mistakes
        if metadata.len() > 250_000_000 {
            panic!("Input file probably too large");
        }

        let mut reader = BufReader::new(rpu_file);

        // Should be small enough to fit in the memory
        let mut data = vec![0; metadata.len() as usize];
        reader.read_exact(&mut data).unwrap();

        println!("{:?}", &data[..300]);
        println!("{:?}", config);
    }
}
