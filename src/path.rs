use std::path::{Component, Path, PathBuf};

pub fn normalize(path: &Path) -> PathBuf {
    if path.as_os_str().is_empty() {
        return PathBuf::new();
    }

    let mut out = PathBuf::new();
    let absolute = path.is_absolute();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
            Component::RootDir => {
                if out.as_os_str().is_empty() {
                    out.push(Path::new("/"));
                }
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() && !absolute {
                    out.push("..");
                }
            }
            Component::Normal(part) => out.push(part),
        }
    }

    if out.as_os_str().is_empty() && absolute {
        out.push(Path::new("/"));
    }

    out
}

pub fn resolve(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        normalize(path)
    } else {
        normalize(&cwd.join(path))
    }
}

pub fn join(base: &Path, path: &str) -> PathBuf {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        normalize(candidate)
    } else {
        normalize(&base.join(candidate))
    }
}

pub fn basename(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_string()
}

pub fn relative_to(path: &Path, base: &Path) -> Option<PathBuf> {
    let normalized_path = normalize(path);
    let normalized_base = normalize(base);

    if normalized_path == normalized_base {
        return Some(PathBuf::new());
    }

    normalized_path
        .strip_prefix(&normalized_base)
        .ok()
        .map(PathBuf::from)
}

pub fn is_same_or_inside(path: &Path, base: &Path) -> bool {
    relative_to(path, base).is_some()
}

pub fn first_component(path: &Path) -> Option<String> {
    path.components()
        .next()
        .and_then(|component| match component {
            Component::Normal(part) => part.to_str().map(|value| value.to_string()),
            _ => None,
        })
}
