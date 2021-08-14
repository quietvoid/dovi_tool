use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "dovi_tool", about = "Stuff about Dolby Vision")]
pub enum Command {
    Demux {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[structopt(
            help = "Uses stdin as input data",
            conflicts_with = "input",
            parse(from_os_str)
        )]
        stdin: Option<PathBuf>,

        #[structopt(
            long,
            short = "bl",
            help = "BL output file location",
            parse(from_os_str)
        )]
        bl_out: Option<PathBuf>,

        #[structopt(
            long,
            short = "el",
            help = "EL output file location",
            parse(from_os_str)
        )]
        el_out: Option<PathBuf>,
    },

    ExtractRpu {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[structopt(
            help = "Uses stdin as input data",
            conflicts_with = "input",
            parse(from_os_str)
        )]
        stdin: Option<PathBuf>,

        #[structopt(
            long,
            short = "o",
            help = "RPU output file location",
            parse(from_os_str)
        )]
        rpu_out: Option<PathBuf>,
    },

    Editor {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(
            name = "json",
            long,
            short = "j",
            help = "Sets the edit JSON file to use",
            parse(from_os_str)
        )]
        json_file: PathBuf,

        #[structopt(
            long,
            short = "o",
            help = "Modified RPU output file location",
            parse(from_os_str)
        )]
        rpu_out: Option<PathBuf>,
    },

    Convert {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[structopt(
            help = "Uses stdin as input data",
            conflicts_with = "input",
            parse(from_os_str)
        )]
        stdin: Option<PathBuf>,

        #[structopt(
            long,
            short = "o",
            help = "Converted single layer output file location",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,

        #[structopt(short = "d", long, help = "Discard the EL stream")]
        discard: bool,
    },

    InjectRpu {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input HEVC file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(long, help = "Sets the input RPU file to use", parse(from_os_str))]
        rpu_in: PathBuf,

        #[structopt(
            long,
            short = "o",
            help = "Output HEVC file location",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },

    Info {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(
            name = "frame",
            long,
            short = "f",
            help = "Frame number to show info for"
        )]
        frame: Option<usize>,
    },

    Generate {
        #[structopt(
            name = "json",
            long,
            short = "j",
            help = "Sets the generator config JSON file to use",
            parse(from_os_str)
        )]
        json_file: PathBuf,

        #[structopt(
            long,
            short = "o",
            help = "Generated RPU output file location",
            parse(from_os_str)
        )]
        rpu_out: Option<PathBuf>,
    },

    Export {
        #[structopt(
            name = "input",
            long,
            short = "i",
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(
            name = "output",
            long,
            short = "o",
            help = "Output JSON file name",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },
}
