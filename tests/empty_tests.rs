use std::time::{Duration, SystemTime};

use trash_cli_macos::empty::should_delete;

#[test]
fn deletes_everything_without_cutoff() {
    assert!(should_delete(&None, None));
    assert!(should_delete(&Some(SystemTime::now()), None));
}

#[test]
fn keeps_recent_items_when_cutoff_is_set() {
    let now = SystemTime::now();
    let cutoff = now
        .checked_sub(Duration::from_secs(60))
        .expect("cutoff should be valid");
    let recent = now.checked_sub(Duration::from_secs(10));
    let old = now.checked_sub(Duration::from_secs(120));

    assert!(!should_delete(&recent, Some(cutoff)));
    assert!(should_delete(&old, Some(cutoff)));
    assert!(!should_delete(&None, Some(cutoff)));
}
