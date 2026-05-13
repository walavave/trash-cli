use std::path::PathBuf;

use trash_cli::trash::scan::build_macos_original_location;

#[test]
fn builds_macos_original_location_from_relative_parent() {
    let path = build_macos_original_location(Some("workspace/docs/"), Some("foo.txt"));
    assert_eq!(path, PathBuf::from("/workspace/docs/foo.txt"));
}

#[test]
fn keeps_absolute_parent_paths() {
    let path = build_macos_original_location(Some("/workspace/docs/"), Some("foo.txt"));
    assert_eq!(path, PathBuf::from("/workspace/docs/foo.txt"));
}
