#[derive(clap::Parser, Debug)]
#[clap(name = "QVNT Interpreter", author, version, about, long_about = None)]
pub struct CliArgs {
    #[clap(short, long, help = "Specify QASM file path")]
    pub input: Option<String>,
    #[clap(long, help = "Set debug format for errors")]
    pub dbg: bool,
    #[clap(
        short,
        long,
        help = "Specify history path for interpreter commands",
        default_value = ".history"
    )]
    pub history: String,
}

impl CliArgs {
    pub fn new() -> Self {
        <Self as clap::StructOpt>::parse()
    }
}
