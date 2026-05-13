use std::fs;

#[path = "common/temp.rs"]
mod temp;

use temp::temp_dir;
use trash_cli::args::{Cli, Command};
use trash_cli::put;
use trash_cli::trash::ds_store::read_entries;

#[test]
fn put_appends_numeric_suffix_for_name_collisions() {
    let sandbox = temp_dir("trash-cli-macos-put");
    let source_dir = sandbox.join("source");
    let trash_dir = sandbox.join("trash");
    fs::create_dir_all(&source_dir).expect("failed to create source dir");
    fs::create_dir_all(&trash_dir).expect("failed to create trash dir");

    let original = source_dir.join("demo.txt");
    fs::write(&original, "payload").expect("failed to write original file");
    fs::write(trash_dir.join("demo.txt"), "existing")
        .expect("failed to write existing trashed file");

    let cli = Cli {
        command: Command::Put {
            paths: vec![original.clone()],
        },
        trash_dir: Some(trash_dir.clone()),
        version: false,
        help: false,
    };

    let original_cwd = std::env::current_dir().expect("failed to read cwd");
    std::env::set_current_dir(&source_dir).expect("failed to change cwd");
    let result = put::run(&cli);
    std::env::set_current_dir(original_cwd).expect("failed to restore cwd");
    result.expect("put should succeed");

    assert!(!original.exists());
    assert!(trash_dir.join("demo.txt_1").exists());

    let entries = read_entries(&trash_dir.join(".DS_Store")).expect("DS_Store should be readable");
    assert!(entries.iter().any(|entry| {
        entry.filename == "demo.txt_1"
            && entry.structure_type == "ptbN"
            && entry.value.as_string() == Some("demo.txt")
    }));
}
