use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Git error: {0}")]
    Git(String),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Tunnel error: {0}")]
    Tunnel(String),
    #[error("Search error: {0}")]
    Search(String),
    #[error("Runtime error: {0}")]
    Runtime(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Rusqlite error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("{0}")]
    Internal(String),
    #[error("Path security error: {0}")]
    PathSecurity(String),
    #[error("Media allowlist error: {0}")]
    MediaAllowlist(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Json(_) => "SERIALIZATION_ERROR",
            AppError::Git(_) => "GIT_ERROR",
            AppError::Http(_) => "NETWORK_ERROR",
            AppError::Encryption(_) => "ENCRYPTION_ERROR",
            AppError::Process(_) => "PROCESS_ERROR",
            AppError::Tunnel(_) => "TUNNEL_ERROR",
            AppError::Search(_) => "SEARCH_ERROR",
            AppError::Runtime(_) => "RUNTIME_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::Rusqlite(_) => "DATABASE_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::PathSecurity(_) => "PATH_SECURITY_ERROR",
            AppError::MediaAllowlist(_) => "MEDIA_ALLOWLIST_ERROR",
        }
    }

    /// Returns a safe user-facing message that never leaks internal paths,
    /// database schema details, or system information.
    pub fn safe_message(&self) -> String {
        match self {
            // User-intentional messages — safe to pass through
            AppError::NotFound(m) | AppError::Validation(m) => m.clone(),
            // Generic messages for variants that may contain internal details
            AppError::Database(_) => "A database error occurred".into(),
            AppError::Io(_) => "A file system error occurred".into(),
            AppError::Json(_) => "Invalid data format".into(),
            AppError::Git(_) => "A git operation failed".into(),
            AppError::Http(_) => "A network error occurred".into(),
            AppError::Encryption(_) => "An encryption error occurred".into(),
            AppError::Process(_) => "A process operation failed".into(),
            AppError::Tunnel(_) => "A tunnel error occurred".into(),
            AppError::Search(_) => "A search error occurred".into(),
            AppError::Runtime(_) => "A runtime error occurred".into(),
            AppError::Rusqlite(_) => "A database error occurred".into(),
            AppError::Internal(_) => "An internal error occurred".into(),
            AppError::PathSecurity(_) => "Path security violation".into(),
            AppError::MediaAllowlist(_) => "Media access denied".into(),
        }
    }
}

#[derive(Serialize)]
struct AppErrorPayload {
    code: &'static str,
    message: String,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let payload = AppErrorPayload {
            code: self.code(),
            message: self.safe_message(),
        };
        payload.serialize(serializer)
    }
}

impl From<crate::utils::crypto::CryptoError> for AppError {
    fn from(e: crate::utils::crypto::CryptoError) -> Self {
        AppError::Encryption(e.to_string())
    }
}

impl From<git2::Error> for AppError {
    fn from(e: git2::Error) -> Self {
        AppError::Git(e.to_string())
    }
}

impl From<zip::result::ZipError> for AppError {
    fn from(e: zip::result::ZipError) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<crate::utils::path_security::PathSecurityError> for AppError {
    fn from(e: crate::utils::path_security::PathSecurityError) -> Self {
        AppError::PathSecurity(e.to_string())
    }
}

impl From<crate::utils::media_allowlist::MediaAllowlistError> for AppError {
    fn from(e: crate::utils::media_allowlist::MediaAllowlistError) -> Self {
        AppError::MediaAllowlist(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
