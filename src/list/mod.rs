use std::cmp::Ordering;
use std::env;
use std::path::Path;

use crate::args::{Cli, SortMode};
use crate::error::Result;
use crate::path;
use crate::trash::model::TrashedFile;
use crate::trash::scan::scan;

pub fn run(cli: &Cli) -> Result<()> {
    let cwd = env::current_dir()?;
    let target_path = cli.target_path().map(|path| path::resolve(&cwd, path));
    let items = candidates(cli, target_path.as_deref())?;

    for file in &items {
        println!("{}", format_item(file));
    }

    Ok(())
}

pub fn candidates(cli: &Cli, target_path: Option<&Path>) -> Result<Vec<TrashedFile>> {
    let report = scan(cli)?;
    for warning in &report.warnings {
        eprintln!("WARN: {warning}");
    }

    let mut items: Vec<_> = report
        .files
        .into_iter()
        .filter(|file| match target_path {
            Some(target) => file.matches_target(target),
            None => true,
        })
        .collect();
    sort_items(&mut items, cli.sort());
    Ok(items)
}

pub fn sort_items(items: &mut [TrashedFile], sort: SortMode) {
    match sort {
        SortMode::None => {}
        SortMode::Date => items.sort_by(|left, right| {
            let left_key = left.modified_date.as_deref().unwrap_or("");
            let right_key = right.modified_date.as_deref().unwrap_or("");
            left_key
                .cmp(right_key)
                .then_with(|| left.original_location.cmp(&right.original_location))
        }),
        SortMode::Path => items.sort_by(|left, right| {
            let left_key = left.original_location.to_string_lossy();
            let right_key = right.original_location.to_string_lossy();
            match left_key.cmp(&right_key) {
                Ordering::Equal => left
                    .modified_date
                    .as_deref()
                    .unwrap_or("")
                    .cmp(right.modified_date.as_deref().unwrap_or("")),
                other => other,
            }
        }),
    }
}

pub fn format_item(file: &TrashedFile) -> String {
    let date = file.modified_date.as_deref().unwrap_or("-");
    format!("{date} {}", file.original_location.display())
}

pub fn format_indexed_item(index: usize, file: &TrashedFile) -> String {
    let date = file.modified_date.as_deref().unwrap_or("-");
    format!("{index:4} {date} {}", file.original_location.display())
}
