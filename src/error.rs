use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    Database(String),
    Serialization(String),
    NotFound(String),
    Camera(String),
    Export(String),
    Unknown(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Camera(msg) => write!(f, "Camera error: {}", msg),
            AppError::Export(msg) => write!(f, "Export error: {}", msg),
            AppError::Unknown(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl From<rexie::Error> for AppError {
    fn from(e: rexie::Error) -> Self {
        AppError::Database(format!("{:?}", e))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
