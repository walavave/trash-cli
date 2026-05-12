mod index;
mod prompt;
mod restorer;

use std::env;

use crate::args::Cli;
use crate::error::Result;
use crate::list;
use crate::path;
use crate::restore::prompt::prompt_line;
use crate::restore::restorer::restore_file;

pub use index::parse_indexes;

pub fn run(cli: &Cli) -> Result<()> {
    let cwd = env::current_dir()?;
    let target_path = cli.target_path().map(|path| path::resolve(&cwd, path));
    let candidates = list::candidates(cli, target_path.as_deref())?;

    if candidates.is_empty() {
        if let Some(target_path) = target_path {
            println!(
                "No matching trashed files found for '{}'",
                target_path.display()
            );
        } else {
            println!("No matching trashed files found");
        }
        return Ok(());
    }

    for (index, file) in candidates.iter().enumerate() {
        println!("{}", list::format_indexed_item(index, file));
    }

    let input = match prompt_line(&format!(
        "What file to restore [0..{}]: ",
        candidates.len() - 1
    ))? {
        Some(line) if !line.trim().is_empty() => line,
        _ => {
            println!("No files were restored");
            return Ok(());
        }
    };

    let indexes = parse_indexes(&input, candidates.len())?;
    for index in indexes {
        let outcome = restore_file(&candidates[index], cli.overwrite())?;
        if let Some(warning) = outcome.metadata_warning {
            eprintln!("WARN: {warning}");
        }
    }

    Ok(())
}
