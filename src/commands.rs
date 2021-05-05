use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "dovi_tool", about = "Stuff about Dolby Vision")]
pub enum Command {
    Demux {
        #[structopt(
            name = "input",
            short = "i",
            long,
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

        #[structopt(long, help = "BL output file location", parse(from_os_str))]
        bl_out: Option<PathBuf>,

        #[structopt(long, help = "EL output file location", parse(from_os_str))]
        el_out: Option<PathBuf>,
    },

    ExtractRpu {
        #[structopt(
            name = "input",
            short = "i",
            long,
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

        #[structopt(long, help = "RPU output file location", parse(from_os_str))]
        rpu_out: Option<PathBuf>,
    },

    Editor {
        #[structopt(
            name = "input",
            short = "i",
            long,
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(
            name = "json",
            short = "j",
            long,
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
            short = "i",
            long,
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
            short = "o",
            long,
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
            short = "i",
            long,
            help = "Sets the input HEVC file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(
            name = "rpu_in",
            long,
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        rpu_in: PathBuf,

        #[structopt(long, help = "Output HEVC file location", parse(from_os_str))]
        output: Option<PathBuf>,
    },

    Info {
        #[structopt(
            name = "input",
            short = "i",
            long,
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[structopt(
            name = "frame",
            short = "f",
            long,
            help = "Frame number to show info for"
        )]
        frame: Option<usize>,
    },
}
