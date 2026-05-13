use std::env;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

const APP_NAME: &str = "trash";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Date,
    Path,
    None,
}

#[derive(Debug, Clone)]
pub enum Command {
    Restore {
        target_path: Option<PathBuf>,
        sort: SortMode,
        overwrite: bool,
    },
    List {
        target_path: Option<PathBuf>,
        sort: SortMode,
    },
    Put {
        paths: Vec<PathBuf>,
    },
    Empty {
        days: Option<u64>,
    },
    Rm {
        pattern: String,
    },
}

#[derive(Debug, Clone)]
pub struct Cli {
    pub command: Command,
    pub trash_dir: Option<PathBuf>,
    pub version: bool,
    pub help: bool,
}

impl Cli {
    pub fn sort(&self) -> SortMode {
        match &self.command {
            Command::Restore { sort, .. } | Command::List { sort, .. } => *sort,
            Command::Put { .. } | Command::Empty { .. } | Command::Rm { .. } => SortMode::Date,
        }
    }

    pub fn target_path(&self) -> Option<&Path> {
        match &self.command {
            Command::Restore { target_path, .. } | Command::List { target_path, .. } => {
                target_path.as_deref()
            }
            Command::Put { .. } | Command::Empty { .. } | Command::Rm { .. } => None,
        }
    }

    pub fn overwrite(&self) -> bool {
        match &self.command {
            Command::Restore { overwrite, .. } => *overwrite,
            Command::List { .. }
            | Command::Put { .. }
            | Command::Empty { .. }
            | Command::Rm { .. } => false,
        }
    }

    pub fn put_paths(&self) -> &[PathBuf] {
        match &self.command {
            Command::Put { paths } => paths.as_slice(),
            Command::Restore { .. }
            | Command::List { .. }
            | Command::Empty { .. }
            | Command::Rm { .. } => &[],
        }
    }

    pub fn empty_days(&self) -> Option<u64> {
        match &self.command {
            Command::Empty { days } => *days,
            Command::Restore { .. }
            | Command::List { .. }
            | Command::Put { .. }
            | Command::Rm { .. } => None,
        }
    }

    pub fn rm_pattern(&self) -> Option<&str> {
        match &self.command {
            Command::Rm { pattern } => Some(pattern.as_str()),
            Command::Restore { .. }
            | Command::List { .. }
            | Command::Put { .. }
            | Command::Empty { .. } => None,
        }
    }
}

pub fn parse() -> Result<Cli> {
    parse_from(env::args().skip(1))
}

pub fn parse_from<I, S>(args: I) -> Result<Cli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut args = args.into_iter().map(Into::into).peekable();
    let mut trash_dir = None;
    let mut version = false;
    let mut help = false;

    let mut command = None;
    let mut sort = SortMode::Date;
    let mut overwrite = false;
    let mut positional: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                help = true;
                return Ok(Cli {
                    command: Command::Restore {
                        target_path: None,
                        sort,
                        overwrite,
                    },
                    trash_dir,
                    version,
                    help,
                });
            }
            "--version" => {
                version = true;
                return Ok(Cli {
                    command: Command::Restore {
                        target_path: None,
                        sort,
                        overwrite,
                    },
                    trash_dir,
                    version,
                    help,
                });
            }
            "--trash-dir" => {
                let value = args
                    .next()
                    .ok_or_else(|| Error::message("--trash-dir requires a value"))?;
                trash_dir = Some(PathBuf::from(value));
            }
            "--sort" => {
                let value = args
                    .next()
                    .ok_or_else(|| Error::message("--sort requires a value"))?;
                sort = parse_sort(&value)?;
            }
            "--overwrite" => overwrite = true,
            "restore" | "list" | "put" | "empty" | "rm" if command.is_none() => {
                command = Some(arg);
            }
            value if value.starts_with('-') => {
                return Err(Error::message(format!("unknown flag: {value}")));
            }
            value if command.is_none() => {
                return Err(Error::message(format!("unknown command: {value}")));
            }
            value => positional.push(value.to_string()),
        }
    }

    let command = match command.as_deref() {
        Some("restore") => {
            if positional.len() > 1 {
                return Err(Error::message("only one positional path is supported"));
            }
            Command::Restore {
                target_path: positional.into_iter().next().map(PathBuf::from),
                sort,
                overwrite,
            }
        }
        Some("list") => {
            if positional.len() > 1 {
                return Err(Error::message("only one positional path is supported"));
            }
            Command::List {
                target_path: positional.into_iter().next().map(PathBuf::from),
                sort,
            }
        }
        Some("put") => Command::Put {
            paths: positional.into_iter().map(PathBuf::from).collect(),
        },
        Some("empty") => Command::Empty {
            days: parse_days(positional)?,
        },
        Some("rm") => Command::Rm {
            pattern: parse_pattern(positional)?,
        },
        None => {
            help = true;
            Command::Restore {
                target_path: None,
                sort,
                overwrite,
            }
        }
        Some(other) => return Err(Error::message(format!("unknown command: {other}"))),
    };

    Ok(Cli {
        command,
        trash_dir,
        version,
        help,
    })
}

pub fn usage() -> String {
    format!(
        r#"{APP_NAME} {}

Usage:
  {APP_NAME} list [--sort date|path|none] [--trash-dir DIR] [PATH]
  {APP_NAME} restore [--sort date|path|none] [--trash-dir DIR] [--overwrite] [PATH]
  {APP_NAME} put [--trash-dir DIR] FILE...
  {APP_NAME} empty [--trash-dir DIR] [DAYS]
  {APP_NAME} rm [--trash-dir DIR] PATTERN

Commands:
  list                    List files in trash
  restore                 Restore files from trash
  put                     Move files or directories to trash
  empty                   Empty trash permanently
  rm                      Remove trashed files matching basename or full path glob

Global options:
  -h, --help              Show this help message
  --version               Show version information

rm PATTERN:
  If PATTERN starts with /, it matches the full original path.
  Otherwise, it matches only the basename. Supports * and ? wildcards.

Examples:
  {APP_NAME} put file.txt documents/
  {APP_NAME} list --sort path
  {APP_NAME} restore --overwrite
  {APP_NAME} empty 30
  {APP_NAME} rm "*.log"
"#,
        env!("CARGO_PKG_VERSION")
    )
}

fn parse_sort(value: &str) -> Result<SortMode> {
    match value {
        "date" => Ok(SortMode::Date),
        "path" => Ok(SortMode::Path),
        "none" => Ok(SortMode::None),
        other => Err(Error::message(format!("invalid sort mode: {other}"))),
    }
}

fn parse_days(positional: Vec<String>) -> Result<Option<u64>> {
    match positional.as_slice() {
        [] => Ok(None),
        [value] => value
            .parse()
            .map(Some)
            .map_err(|_| Error::message(format!("invalid days value: {value}"))),
        _ => Err(Error::message(
            "only one positional days value is supported",
        )),
    }
}

fn parse_pattern(positional: Vec<String>) -> Result<String> {
    match positional.as_slice() {
        [value] => Ok(value.clone()),
        [] => Err(Error::message("rm requires a pattern")),
        _ => Err(Error::message("only one pattern is supported")),
    }
}
