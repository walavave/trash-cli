use trash_cli_macos::restore::parse_indexes;

#[test]
fn parses_ranges() {
    let indexes = parse_indexes("1,3-4", 6).expect("range should parse");
    assert_eq!(indexes, vec![1, 3, 4]);
}

#[test]
fn rejects_out_of_range_indexes() {
    assert!(parse_indexes("9", 2).is_err());
}
