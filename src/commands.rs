use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub enum Command {
    Demux {
        #[clap(
            name = "input",
            long,
            short = 'i',
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[clap(help = "Uses stdin as input data", parse(from_os_str))]
        stdin: Option<PathBuf>,

        #[clap(
            long,
            short = 'b',
            help = "BL output file location",
            parse(from_os_str)
        )]
        bl_out: Option<PathBuf>,

        #[clap(
            long,
            short = 'e',
            help = "EL output file location",
            parse(from_os_str)
        )]
        el_out: Option<PathBuf>,

        #[clap(long, help = "Output the EL file only")]
        el_only: bool,
    },

    ExtractRpu {
        #[clap(
            name = "input",
            long,
            short = 'i',
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[clap(help = "Uses stdin as input data", parse(from_os_str))]
        stdin: Option<PathBuf>,

        #[clap(
            long,
            short = 'o',
            help = "RPU output file location",
            parse(from_os_str)
        )]
        rpu_out: Option<PathBuf>,
    },

    Editor {
        #[clap(
            name = "input",
            long,
            short = 'i',
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[clap(
            name = "json",
            long,
            short = 'j',
            help = "Sets the edit JSON file to use",
            parse(from_os_str)
        )]
        json_file: PathBuf,

        #[clap(
            long,
            short = 'o',
            help = "Modified RPU output file location",
            parse(from_os_str)
        )]
        rpu_out: Option<PathBuf>,
    },

    Convert {
        #[clap(
            name = "input",
            long,
            short = 'i',
            help = "Sets the input file to use",
            conflicts_with = "stdin",
            parse(from_os_str)
        )]
        input: Option<PathBuf>,

        #[clap(help = "Uses stdin as input data", parse(from_os_str))]
        stdin: Option<PathBuf>,

        #[clap(
            long,
            short = 'o',
            help = "Converted single layer output file location",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,

        #[clap(short = 'd', long, help = "Discard the EL stream")]
        discard: bool,
    },

    InjectRpu {
        #[clap(
            name = "input",
            long,
            short = 'i',
            help = "Sets the input HEVC file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[clap(long, help = "Sets the input RPU file to use", parse(from_os_str))]
        rpu_in: PathBuf,

        #[clap(
            long,
            short = 'o',
            help = "Output HEVC file location",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },

    Info {
        #[clap(
            name = "input",
            long,
            short = 'i',
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[clap(
            name = "frame",
            long,
            short = 'f',
            help = "Frame number to show info for"
        )]
        frame: Option<usize>,
    },

    Generate {
        #[clap(
            name = "json",
            long,
            short = 'j',
            help = "Sets the generator config JSON file to use",
            conflicts_with = "xml",
            parse(from_os_str)
        )]
        json_file: Option<PathBuf>,

        #[clap(
            long,
            short = 'o',
            help = "Generated RPU output file location",
            parse(from_os_str)
        )]
        rpu_out: Option<PathBuf>,

        #[clap(
            name = "hdr10plus-json",
            long,
            help = "HDR10+ JSON file to generate from",
            conflicts_with = "madvr-file",
            parse(from_os_str)
        )]
        hdr10plus_json: Option<PathBuf>,

        #[clap(
            short = 'x',
            long,
            help = "XML metadata file to generate from",
            conflicts_with_all = &["json", "hdr10plus_json", "madvr-file"],
            parse(from_os_str)
        )]
        xml: Option<PathBuf>,

        #[clap(long, help = "Canvas width for L5 metadata generation")]
        canvas_width: Option<u16>,

        #[clap(long, help = "Canvas height for L5 metadata generation")]
        canvas_height: Option<u16>,

        #[clap(
            name = "madvr-file",
            long,
            help = "madVR measurement file to generate from",
            parse(from_os_str)
        )]
        madvr_file: Option<PathBuf>,

        #[clap(
            long,
            help = "madVR source: use custom per-frame target nits if available"
        )]
        use_custom_targets: bool,
    },

    Export {
        #[clap(
            name = "input",
            long,
            short = 'i',
            help = "Sets the input RPU file to use",
            parse(from_os_str)
        )]
        input: PathBuf,

        #[clap(
            name = "output",
            long,
            short = 'o',
            help = "Output JSON file name",
            parse(from_os_str)
        )]
        output: Option<PathBuf>,
    },
}
