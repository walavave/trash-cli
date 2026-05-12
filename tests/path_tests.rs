use std::path::{Path, PathBuf};

use trash_cli_macos::path::{normalize, relative_to};

#[test]
fn normalizes_relative_segments() {
    let normalized = normalize(Path::new("/opt/example/../project/./src/"));
    assert_eq!(normalized, PathBuf::from("/opt/project/src"));
}

#[test]
fn computes_relative_paths() {
    let path = Path::new("/opt/project/src");
    let base = Path::new("/opt");
    assert_eq!(relative_to(path, base), Some(PathBuf::from("project/src")));
}
