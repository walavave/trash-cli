use crate::args::Cli;
use crate::error::{Error, Result};
use crate::trash::item_ops::purge_trashed_file;
use crate::trash::scan::scan;

pub fn run(cli: &Cli) -> Result<()> {
    let pattern = cli
        .rm_pattern()
        .ok_or_else(|| Error::message("trash-rm requires a pattern"))?;
    let report = scan(cli)?;
    for warning in &report.warnings {
        eprintln!("WARN: {warning}");
    }

    for file in report.files {
        if matches_original_location(pattern, &file.original_location.to_string_lossy()) {
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

pub fn matches_original_location(pattern: &str, original_location: &str) -> bool {
    let subject = if pattern.starts_with('/') {
        original_location
    } else {
        original_location
            .rsplit('/')
            .next()
            .unwrap_or(original_location)
    };

    glob_match(pattern.as_bytes(), subject.as_bytes())
}

fn glob_match(pattern: &[u8], text: &[u8]) -> bool {
    let mut pattern_index = 0usize;
    let mut text_index = 0usize;
    let mut star_pattern = None;
    let mut star_text = 0usize;

    while text_index < text.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == text[text_index])
        {
            pattern_index += 1;
            text_index += 1;
            continue;
        }

        if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star_pattern = Some(pattern_index);
            pattern_index += 1;
            star_text = text_index;
            continue;
        }

        if let Some(star_index) = star_pattern {
            pattern_index = star_index + 1;
            star_text += 1;
            text_index = star_text;
            continue;
        }

        return false;
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}
