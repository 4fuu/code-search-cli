use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("not inside a git repository")]
    NotInGitRepo,
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("file not found: {0}")]
    FileNotFound(String),
}

impl AppError {
    /// Error code for JSON output.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::NotInGitRepo => "NOT_IN_GIT_REPO",
            AppError::UnsupportedLanguage(_) => "UNSUPPORTED_LANGUAGE",
            AppError::FileNotFound(_) => "FILE_NOT_FOUND",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes() {
        assert_eq!(AppError::NotInGitRepo.code(), "NOT_IN_GIT_REPO");
        assert_eq!(
            AppError::UnsupportedLanguage("x".into()).code(),
            "UNSUPPORTED_LANGUAGE"
        );
        assert_eq!(AppError::FileNotFound("x".into()).code(), "FILE_NOT_FOUND");
    }

    #[test]
    fn error_display() {
        assert_eq!(
            AppError::NotInGitRepo.to_string(),
            "not inside a git repository"
        );
        assert_eq!(
            AppError::FileNotFound("foo.rs".into()).to_string(),
            "file not found: foo.rs"
        );
        assert_eq!(
            AppError::UnsupportedLanguage("java".into()).to_string(),
            "unsupported language: java"
        );
    }
}
