use std::path::PathBuf;

use trash_cli_macos::args::{Command, parse_from};

#[test]
fn parses_trash_list_alias() {
    let cli = parse_from(["trash-list", "/tmp"]).expect("trash-list should parse");
    match cli.command {
        Command::List {
            target_path: Some(path),
            ..
        } => assert_eq!(path, PathBuf::from("/tmp")),
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn parses_trash_put_alias() {
    let cli = parse_from(["trash-put", "foo.txt"]).expect("trash-put should parse");
    match cli.command {
        Command::Put { paths } => assert_eq!(paths, vec![PathBuf::from("foo.txt")]),
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn usage_uses_trash_binary_name() {
    let usage = trash_cli_macos::args::usage();
    assert!(usage.starts_with("trash "));
    assert!(usage.contains("\n  trash [put|trash-put] [OPTIONS] FILE...\n"));
    assert!(!usage.contains("trash-cli-macos"));
}
