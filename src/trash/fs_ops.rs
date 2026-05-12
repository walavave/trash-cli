use std::fs;
use std::io;
use std::path::Path;

use crate::error::Result;

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn remove_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn move_path(src: &Path, dst: &Path) -> Result<()> {
    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(err) if is_cross_device(&err) => {
            copy_recursively(src, dst)?;
            remove_path(src)?;
            Ok(())
        }
        Err(err) => Err(err.into()),
    }
}

fn is_cross_device(err: &io::Error) -> bool {
    matches!(err.raw_os_error(), Some(18))
}

fn copy_recursively(src: &Path, dst: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(src)?;
    if metadata.file_type().is_symlink() {
        copy_symlink(src, dst)?;
        return Ok(());
    }

    if metadata.is_dir() {
        fs::create_dir_all(dst)?;
        fs::set_permissions(dst, metadata.permissions())?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let child_src = entry.path();
            let child_dst = dst.join(entry.file_name());
            copy_recursively(&child_src, &child_dst)?;
        }
        return Ok(());
    }

    fs::copy(src, dst)?;
    fs::set_permissions(dst, metadata.permissions())?;
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(src: &Path, dst: &Path) -> Result<()> {
    use std::os::unix::fs::symlink;

    let target = fs::read_link(src)?;
    symlink(target, dst)?;
    Ok(())
}

#[cfg(not(unix))]
fn copy_symlink(_src: &Path, _dst: &Path) -> Result<()> {
    Err(crate::error::Error::message(
        "symlink restore is not supported on this platform",
    ))
}
