use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum PathSecurityError {
    #[error("path must be inside a registered project root")]
    OutsideRoot,
    #[error("non-absolute path rejected: {0}")]
    NonAbsolute(String),
    #[error("path does not exist: {0}")]
    NotReadable(String),
    #[error("invalid media URL: {0}")]
    InvalidMediaUrl(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn normalize_for_comparison(target: &Path) -> PathBuf {
    let resolved = dunce::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(resolved.to_string_lossy().to_lowercase())
    }
    #[cfg(not(target_os = "windows"))]
    {
        resolved
    }
}

pub fn is_within_base(target_path: &Path, base_path: &Path) -> bool {
    let norm_target = normalize_for_comparison(target_path);
    let norm_base = normalize_for_comparison(base_path);
    if norm_target == norm_base {
        return true;
    }
    let base_str = norm_base.to_string_lossy();
    let base_with_sep = if base_str.ends_with(std::path::MAIN_SEPARATOR) {
        base_str.into_owned()
    } else {
        format!("{}{}", base_str, std::path::MAIN_SEPARATOR)
    };
    norm_target.to_string_lossy().starts_with(&base_with_sep)
}

pub fn is_within_any_base(target_path: &Path, allowed_bases: &[PathBuf]) -> bool {
    allowed_bases
        .iter()
        .any(|base| is_within_base(target_path, base))
}

pub fn resolve_and_validate(candidate: &Path, allowed_bases: &[PathBuf]) -> Option<PathBuf> {
    let resolved = dunce::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
    if is_within_any_base(&resolved, allowed_bases) {
        Some(resolved)
    } else {
        None
    }
}

pub fn validate_inside_roots(
    target_path: &Path,
    project_roots: &[PathBuf],
) -> Result<(), PathSecurityError> {
    let resolved = dunce::canonicalize(target_path).unwrap_or_else(|_| target_path.to_path_buf());
    if is_within_any_base(&resolved, project_roots) {
        Ok(())
    } else {
        Err(PathSecurityError::OutsideRoot)
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MediaUrl {
    pub hostname: String,
    pub file_path: PathBuf,
}

#[allow(dead_code)]
pub fn parse_media_url(
    request_url: &str,
    app_base: &Path,
    cwd_base: &Path,
) -> Result<MediaUrl, PathSecurityError> {
    let without_scheme = request_url
        .strip_prefix("media://")
        .or_else(|| request_url.strip_prefix("media:/"))
        .or_else(|| request_url.strip_prefix("media:"))
        .ok_or_else(|| PathSecurityError::InvalidMediaUrl("missing media:// scheme".into()))?;
    let (hostname, raw_path) = match without_scheme.find('/') {
        Some(pos) => {
            let (h, p) = without_scheme.split_at(pos);
            (h.to_string(), p.to_string())
        }
        None => (without_scheme.to_string(), String::new()),
    };
    let decoded = percent_decode(&raw_path);
    if hostname == "app" {
        let relative = decoded
            .trim_start_matches('/')
            .trim_start_matches('\\');
        let app_candidate = app_base.join(relative);
        let file_path = if app_candidate.exists() {
            app_candidate
        } else {
            cwd_base.join(relative)
        };
        Ok(MediaUrl {
            hostname,
            file_path,
        })
    } else {
        let file_path = if !hostname.is_empty()
            && hostname.len() == 1
            && hostname.chars().next().unwrap().is_ascii_alphabetic()
        {
            PathBuf::from(format!("{}:{}", hostname, decoded))
        } else {
            PathBuf::from(decoded)
        };
        Ok(MediaUrl {
            hostname,
            file_path,
        })
    }
}

pub fn sanitize_filename(value: &str) -> String {
    let cleaned: String = value
        .chars()
        .map(|c| {
            if matches!(
                c,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            ) || (c as u32) < 0x20
            {
                '_'
            } else if c.is_whitespace() {
                ' '
            } else {
                c
            }
        })
        .collect();
    let trimmed = cleaned
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if trimmed.is_empty() {
        "project".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

#[allow(dead_code)]
fn percent_decode(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    output.push(byte as char);
                    continue;
                }
            }
            output.push('%');
            output.push_str(&hex);
        } else {
            output.push(c);
        }
    }
    output
}
