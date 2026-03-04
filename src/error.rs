use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    Database(String),
    Serialization(String),
    Export(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(msg) => write!(f, "Database error: {}", msg),
            AppError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            AppError::Export(msg) => write!(f, "Export error: {}", msg),
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

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    // ── Display ───────────────────────────────────────────────────────────────

    /// Each variant has a distinct prefix so log lines clearly identify the
    /// error source without needing to inspect the type.
    #[wasm_bindgen_test]
    fn database_error_display() {
        let e = AppError::Database("connection lost".to_string());
        assert_eq!(e.to_string(), "Database error: connection lost");
    }

    #[wasm_bindgen_test]
    fn serialization_error_display() {
        let e = AppError::Serialization("bad JSON".to_string());
        assert_eq!(e.to_string(), "Serialization error: bad JSON");
    }

    #[wasm_bindgen_test]
    fn export_error_display() {
        let e = AppError::Export("jsPDF missing".to_string());
        assert_eq!(e.to_string(), "Export error: jsPDF missing");
    }

    /// An empty payload still produces the prefix — callers always get a
    /// non-empty, recognisable string from `.to_string()`.
    #[wasm_bindgen_test]
    fn empty_payload_still_has_prefix() {
        assert_eq!(
            AppError::Database(String::new()).to_string(),
            "Database error: "
        );
    }

    // ── PartialEq ─────────────────────────────────────────────────────────────

    /// Same variant and message → equal.
    #[wasm_bindgen_test]
    fn same_variant_and_message_are_equal() {
        assert_eq!(
            AppError::Export("x".to_string()),
            AppError::Export("x".to_string())
        );
    }

    /// Different messages in the same variant → not equal.
    #[wasm_bindgen_test]
    fn different_messages_are_not_equal() {
        assert_ne!(
            AppError::Database("a".to_string()),
            AppError::Database("b".to_string())
        );
    }

    /// Different variants with the same message → not equal. Prevents
    /// accidentally treating a DB error as a serialization error.
    #[wasm_bindgen_test]
    fn different_variants_are_not_equal() {
        assert_ne!(
            AppError::Database("x".to_string()),
            AppError::Serialization("x".to_string())
        );
    }

    // ── From<serde_json::Error> ───────────────────────────────────────────────

    /// JSON parse failures become `Serialization` so callers can match on the
    /// variant to distinguish storage errors from format errors.
    #[wasm_bindgen_test]
    fn from_serde_json_error_becomes_serialization() {
        let e = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let app_err = AppError::from(e);
        assert!(matches!(app_err, AppError::Serialization(_)));
        assert!(app_err.to_string().starts_with("Serialization error:"));
    }
}
