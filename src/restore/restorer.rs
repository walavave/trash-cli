use crate::error::{Error, Result};
use crate::trash::ds_store::remove_trash_entry;
use crate::trash::fs_ops::{ensure_parent_dir, move_path, remove_path};
use crate::trash::model::TrashedFile;

#[derive(Debug, Default)]
pub struct RestoreOutcome {
    pub metadata_warning: Option<String>,
}

pub fn restore_file(file: &TrashedFile, overwrite: bool) -> Result<RestoreOutcome> {
    if file.original_location.exists() && !overwrite {
        return Err(Error::message(format!(
            "refusing to overwrite existing file \"{}\"",
            file.original_location
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
        )));
    }

    ensure_parent_dir(&file.original_location)?;

    if overwrite && file.original_location.exists() {
        remove_path(&file.original_location)?;
    }

    let file_name = file
        .trash_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string();
    move_path(&file.trash_path, &file.original_location)?;

    let metadata_warning =
        match remove_trash_entry(&file.trash_root.native_metadata_path, &file_name) {
            Ok(()) => None,
            Err(err) => Some(format!(
                "restored {}, but failed to update metadata {}: {err}",
                file.original_location.display(),
                file.trash_root.native_metadata_path.display()
            )),
        };

    Ok(RestoreOutcome { metadata_warning })
}
