use trash_cli::rm::matches_original_location;

#[test]
fn matches_basename_patterns() {
    assert!(matches_original_location("*.o", "/tmp/file.o"));
    assert!(!matches_original_location("*.o", "/tmp/file.rs"));
}

#[test]
fn matches_absolute_patterns() {
    assert!(matches_original_location("/tmp/*", "/tmp/file.o"));
    assert!(!matches_original_location("/var/*", "/tmp/file.o"));
}
