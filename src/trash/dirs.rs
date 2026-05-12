use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::args::Cli;
use crate::error::Result;
use crate::trash::model::TrashRoot;

#[cfg(target_family = "unix")]
unsafe extern "C" {
    fn getuid() -> u32;
}

pub fn discover_roots(cli: &Cli) -> Result<Vec<TrashRoot>> {
    if let Some(custom) = &cli.trash_dir {
        return Ok(vec![build_root_for_custom(custom.clone())]);
    }

    let mut roots = Vec::new();

    if let Some(home) = env::var_os("HOME") {
        roots.push(build_root(PathBuf::from(home).join(".Trash")));
    }

    roots.push(volume_root(Path::new("/")));

    if let Ok(entries) = fs::read_dir("/Volumes") {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    roots.push(volume_root(&entry.path()));
                }
            }
        }
    }

    dedupe_roots(&mut roots);
    Ok(roots)
}

pub fn root_for_path(cli: &Cli, path: &Path) -> Result<TrashRoot> {
    if let Some(custom) = &cli.trash_dir {
        return Ok(build_root_for_custom(custom.clone()));
    }

    if let Some(mount_root) = volume_mount_root(path) {
        return Ok(volume_root(&mount_root));
    }

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        if path.starts_with(&home) {
            return Ok(build_root(home.join(".Trash")));
        }
    }

    Ok(volume_root(Path::new("/")))
}

fn build_root(trash_dir: PathBuf) -> TrashRoot {
    TrashRoot {
        native_metadata_path: trash_dir.join(".DS_Store"),
        trash_dir,
    }
}

fn build_root_for_custom(trash_dir: PathBuf) -> TrashRoot {
    build_root(trash_dir)
}

fn volume_root(volume_root: &Path) -> TrashRoot {
    let uid = current_uid();
    let trash_dir = volume_root.join(".Trashes").join(uid.to_string());
    build_root(trash_dir)
}

fn volume_mount_root(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    let first = components.next()?;
    let second = components.next()?;
    let third = components.next()?;

    if first != std::path::Component::RootDir {
        return None;
    }

    if second.as_os_str() != "Volumes" {
        return None;
    }

    Some(PathBuf::from("/Volumes").join(third.as_os_str()))
}

fn dedupe_roots(roots: &mut Vec<TrashRoot>) {
    let mut seen = std::collections::BTreeSet::<OsString>::new();
    roots.retain(|root| seen.insert(root.trash_dir.clone().into_os_string()));
}

fn current_uid() -> u32 {
    #[cfg(target_family = "unix")]
    unsafe {
        // SAFETY: `getuid` has no preconditions and returns the current process UID.
        getuid()
    }

    #[cfg(not(target_family = "unix"))]
    {
        0
    }
}
