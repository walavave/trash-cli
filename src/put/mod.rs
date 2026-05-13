use std::env;
use std::fs;
use std::path::PathBuf;

use crate::args::Cli;
use crate::error::{Error, Result};
use crate::path;
use crate::trash::dirs::root_for_path;
use crate::trash::ds_store::upsert_trash_entry;
use crate::trash::fs_ops::move_path;

pub fn run(cli: &Cli) -> Result<()> {
    let cwd = env::current_dir()?;
    let paths = cli.put_paths();
    if paths.is_empty() {
        return Err(Error::message("put requires at least one path"));
    }

    for input in paths {
        let resolved = path::resolve(&cwd, input);
        trash_one(cli, resolved)?;
    }

    Ok(())
}

fn trash_one(cli: &Cli, original_location: PathBuf) -> Result<()> {
    if fs::symlink_metadata(&original_location).is_err() {
        return Err(Error::message(format!(
            "path does not exist: {}",
            original_location.display()
        )));
    }

    let root = root_for_path(cli, &original_location)?;
    fs::create_dir_all(&root.trash_dir)?;

    let basename = path::basename(&original_location);
    let file_name = unique_name(&root.trash_dir, &basename)?;
    let trash_path = root.trash_dir.join(&file_name);

    move_path(&original_location, &trash_path)?;

    if let Err(err) = upsert_trash_entry(&root.native_metadata_path, &file_name, &original_location)
    {
        let _ = move_path(&trash_path, &original_location);
        return Err(err.into());
    }

    Ok(())
}

fn unique_name(trash_dir: &std::path::Path, basename: &str) -> Result<String> {
    let mut candidate = basename.to_string();
    let mut index = 1usize;

    while trash_dir.join(&candidate).exists() {
        candidate = format!("{basename}_{index}");
        index += 1;
    }

    Ok(candidate)
}
