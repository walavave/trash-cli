#[path = "common/temp.rs"]
mod temp;

use std::path::Path;

use temp::temp_dir;
use trash_cli::trash::ds_store::{read_entries, remove_trash_entry, upsert_trash_entry};

#[test]
fn upsert_and_remove_native_trash_entries_round_trip() {
    let dir = temp_dir("trash-cli-macos-ds-store");
    let metadata_path = dir.join(".DS_Store");

    upsert_trash_entry(
        &metadata_path,
        "demo.txt",
        Path::new("/workspace/docs/demo.txt"),
    )
    .expect("should create metadata");
    upsert_trash_entry(&metadata_path, "note.txt", Path::new("/workspace/note.txt"))
        .expect("should append metadata");

    let entries = read_entries(&metadata_path).expect("metadata should parse");
    assert!(entries.iter().any(|entry| {
        entry.filename == "demo.txt"
            && entry.structure_type == "ptbL"
            && entry.value.as_string() == Some("/workspace/docs")
    }));
    assert!(entries.iter().any(|entry| {
        entry.filename == "note.txt"
            && entry.structure_type == "ptbN"
            && entry.value.as_string() == Some("note.txt")
    }));

    remove_trash_entry(&metadata_path, "demo.txt").expect("should remove metadata");
    let entries = read_entries(&metadata_path).expect("metadata should still parse");
    assert!(!entries.iter().any(|entry| entry.filename == "demo.txt"));
    assert!(entries.iter().any(|entry| entry.filename == "note.txt"));

    remove_trash_entry(&metadata_path, "note.txt").expect("should remove final metadata");
    assert!(!metadata_path.exists());
}
