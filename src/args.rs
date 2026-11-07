use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None, name = "polder")]
pub(crate) struct Args {
    pub(crate) input: String,
    #[arg(short = 'd', long, required = false)]
    pub(crate) debug: bool,
    #[arg(short = 'D', long, required = false)]
    pub(crate) trace: bool,
    #[arg(long, required = false)]
    pub(crate) print_ast: bool,
}
