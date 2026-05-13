use trash_cli::args::{Command, parse_from};

#[test]
fn usage_uses_trash_binary_name() {
    let usage = trash_cli::args::usage();
    assert!(usage.starts_with("trash "));
    assert!(usage.contains("\n  trash put [OPTIONS] FILE...\n"));
    assert!(usage.contains("\n  trash rm PATTERN\n"));
    assert!(usage.contains("rm PATTERN:"));
    assert!(usage.contains("Supports * and ? wildcards."));
    assert!(!usage.contains("trash-cli-macos"));
    assert!(!usage.contains("trash-put"));
}

#[test]
fn no_args_defaults_to_help() {
    let cli = parse_from(std::iter::empty::<&str>()).expect("empty args should parse");
    assert!(cli.help);
    match cli.command {
        Command::Restore { target_path, .. } => assert!(target_path.is_none()),
        other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn bare_positional_is_not_treated_as_restore() {
    let err = parse_from(["foo"]).expect_err("bare positional should be rejected");
    assert_eq!(err.to_string(), "unknown command: foo");
}

#[test]
fn rejects_removed_alias_commands() {
    for alias in ["trash-list", "trash-put", "trash-restore", "trash-empty", "trash-rm"] {
        let err = parse_from([alias]).expect_err("removed alias should be rejected");
        assert_eq!(err.to_string(), format!("unknown command: {alias}"));
    }
}
