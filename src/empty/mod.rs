use std::time::{Duration, SystemTime};

use crate::args::Cli;
use crate::error::Result;
use crate::trash::item_ops::purge_trashed_file;
use crate::trash::scan::scan;

pub fn run(cli: &Cli) -> Result<()> {
    let report = scan(cli)?;
    for warning in &report.warnings {
        eprintln!("WARN: {warning}");
    }

    let cutoff = cli.empty_days().and_then(|days| {
        let seconds = days.saturating_mul(24 * 60 * 60);
        SystemTime::now().checked_sub(Duration::from_secs(seconds))
    });

    for file in report.files {
        if should_delete(&file.trashed_at, cutoff) {
            if let Err(err) = purge_trashed_file(&file) {
                eprintln!(
                    "WARN: failed to remove {}: {err}",
                    file.original_location.display()
                );
            }
        }
    }

    Ok(())
}

pub fn should_delete(trashed_at: &Option<SystemTime>, cutoff: Option<SystemTime>) -> bool {
    match cutoff {
        Some(cutoff) => trashed_at.map(|time| time <= cutoff).unwrap_or(false),
        None => true,
    }
}
