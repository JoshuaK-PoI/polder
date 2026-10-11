use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "polder")]
pub(crate) struct Args {
    #[arg(short, long)]
    pub(crate) input: String,
    #[arg(short, long, required = false)]
    pub(crate) debug: bool,
}
