use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct TrashRoot {
    pub trash_dir: PathBuf,
    pub native_metadata_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct TrashedFile {
    pub original_location: PathBuf,
    pub modified_date: Option<String>,
    pub trashed_at: Option<SystemTime>,
    pub trash_path: PathBuf,
    pub trash_root: TrashRoot,
}

impl TrashedFile {
    pub fn matches_target(&self, target: &Path) -> bool {
        crate::path::is_same_or_inside(&self.original_location, target)
    }
}
