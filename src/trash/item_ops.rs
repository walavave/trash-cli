use crate::error::Result;
use crate::trash::ds_store::remove_trash_entry;
use crate::trash::fs_ops::remove_path;
use crate::trash::model::TrashedFile;

pub fn purge_trashed_file(file: &TrashedFile) -> Result<()> {
    remove_path(&file.trash_path)?;
    if let Some(file_name) = file.trash_path.file_name().and_then(|value| value.to_str()) {
        remove_trash_entry(&file.trash_root.native_metadata_path, file_name)?;
    }
    Ok(())
}
