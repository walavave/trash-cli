use std::fs;

use crate::args::Cli;
use crate::error::Result;
use crate::trash::date::format_for_display;
use crate::trash::dirs::discover_roots;
use crate::trash::ds_store::read_entries;
use crate::trash::model::TrashedFile;

#[derive(Debug, Default)]
pub struct ScanReport {
    pub files: Vec<TrashedFile>,
    pub warnings: Vec<String>,
}

pub fn scan(cli: &Cli) -> Result<ScanReport> {
    let roots = discover_roots(cli)?;
    let mut report = ScanReport::default();

    for root in roots {
        scan_macos_root(&root, &mut report)?;
    }

    Ok(report)
}

fn scan_macos_root(root: &crate::trash::model::TrashRoot, report: &mut ScanReport) -> Result<()> {
    let entries = match read_entries(&root.native_metadata_path) {
        Ok(entries) => entries,
        Err(err) if matches!(&err, crate::error::Error::Io(io_err) if io_err.kind() == std::io::ErrorKind::NotFound) => {
            Vec::new()
        }
        Err(err) => {
            report.warnings.push(format!(
                "failed to read macOS trash metadata {}: {err}",
                root.native_metadata_path.display()
            ));
            Vec::new()
        }
    };

    let mut metadata_by_file = std::collections::BTreeMap::<String, MacTrashMetadata>::new();
    for entry in entries {
        let metadata = metadata_by_file.entry(entry.filename).or_default();
        match entry.structure_type.as_str() {
            "dscl" => {}
            "ptbL" => {
                if let Some(value) = entry.value.as_string() {
                    metadata.parent_path = Some(value.to_string());
                }
            }
            "ptbN" => {
                if let Some(value) = entry.value.as_string() {
                    metadata.file_name = Some(value.to_string());
                }
            }
            _ => {}
        }
    }

    match fs::read_dir(&root.trash_dir) {
        Ok(entries) => {
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        report.warnings.push(format!(
                            "failed to read an entry in {}: {err}",
                            root.trash_dir.display()
                        ));
                        continue;
                    }
                };

                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                if should_skip_native_entry(root, &file_name) {
                    continue;
                }

                let Some(metadata) = metadata_by_file.remove(&file_name) else {
                    report.warnings.push(format!(
                        "missing macOS trash metadata for file: {}",
                        path.display()
                    ));
                    continue;
                };

                let original_location = build_macos_original_location(
                    metadata.parent_path.as_deref(),
                    metadata.file_name.as_deref().or(Some(&file_name)),
                );
                let trashed_at = entry
                    .metadata()
                    .ok()
                    .and_then(|metadata| metadata.modified().ok());
                let modified_date = trashed_at.map(format_for_display);

                report.files.push(TrashedFile {
                    original_location,
                    modified_date,
                    trashed_at,
                    trash_path: path,
                    trash_root: root.clone(),
                });
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => report.warnings.push(format!(
            "failed to read trash dir {}: {err}",
            root.trash_dir.display()
        )),
    }

    Ok(())
}

fn should_skip_native_entry(root: &crate::trash::model::TrashRoot, file_name: &str) -> bool {
    let _ = root;
    file_name == ".DS_Store"
}

#[derive(Default)]
struct MacTrashMetadata {
    parent_path: Option<String>,
    file_name: Option<String>,
}

pub fn build_macos_original_location(
    parent_path: Option<&str>,
    file_name: Option<&str>,
) -> std::path::PathBuf {
    let mut path = match parent_path {
        Some(value) if value.starts_with('/') => std::path::PathBuf::from(value),
        Some(value) => std::path::PathBuf::from("/").join(value),
        None => std::path::PathBuf::from("/"),
    };

    if let Some(name) = file_name {
        let current_name = path.file_name().and_then(|value| value.to_str());
        if current_name != Some(name) || path.as_os_str() == "/" {
            path.push(name);
        }
    }

    crate::path::normalize(&path)
}
