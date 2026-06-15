use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

const IGNORED_DIR_NAMES: &[&str] = &[
    "node_modules",
    "venv",
    ".venv",
    "env",
    "__pycache__",
    ".pycache",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "vendor",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    "out",
    ".turbo",
    ".cache",
    "coverage",
    ".parcel-cache",
    ".vite",
    ".git",
    ".svn",
    ".hg",
    "bower_components",
    ".sass-cache",
    ".gradle",
    "bin",
    "obj",
];

static IGNORED_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| IGNORED_DIR_NAMES.iter().copied().collect());

#[allow(dead_code)]
pub fn is_ignored_dir_name(name: &str) -> bool {
    IGNORED_SET.contains(name.to_ascii_lowercase().as_str())
}

pub fn is_path_ignored(path: &Path) -> bool {
    for component in path.components() {
        if let std::path::Component::Normal(name) = component {
            if let Some(name_str) = name.to_str() {
                if name_str.starts_with('.') {
                    return true;
                }
                if is_ignored_dir_name(name_str) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn ripgrep_exclude_globs() -> Vec<String> {
    IGNORED_DIR_NAMES
        .iter()
        .map(|d| format!("!**/{d}/**"))
        .collect()
}
