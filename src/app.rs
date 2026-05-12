use crate::args;
use crate::empty;
use crate::error::Result;
use crate::list;
use crate::put;
use crate::restore;
use crate::rm;

const APP_NAME: &str = "trash";

pub fn run() -> Result<()> {
    let cli = args::parse()?;

    if cli.help {
        println!("{}", args::usage());
        return Ok(());
    }

    if cli.version {
        println!("{APP_NAME} {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    match &cli.command {
        args::Command::Restore { .. } => restore::run(&cli),
        args::Command::List { .. } => list::run(&cli),
        args::Command::Put { .. } => put::run(&cli),
        args::Command::Empty { .. } => empty::run(&cli),
        args::Command::Rm { .. } => rm::run(&cli),
    }
}
