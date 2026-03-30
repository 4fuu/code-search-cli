use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[value(name = "rust", alias = "rs")]
    Rust,
    #[value(name = "typescript", alias = "ts")]
    TypeScript,
    #[value(name = "python", alias = "py")]
    Python,
    #[value(name = "go", alias = "golang")]
    Go,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::TypeScript => write!(f, "typescript"),
            Language::Python => write!(f, "python"),
            Language::Go => write!(f, "go"),
        }
    }
}

impl Language {
    /// Detect language from file extension. Returns `None` for unsupported extensions.
    pub fn from_path(path: &Path) -> Option<Language> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" => Some(Language::TypeScript),
            "py" => Some(Language::Python),
            "go" => Some(Language::Go),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn from_path_rust() {
        assert_eq!(
            Language::from_path(Path::new("foo.rs")),
            Some(Language::Rust)
        );
    }

    #[test]
    fn from_path_typescript() {
        assert_eq!(
            Language::from_path(Path::new("foo.ts")),
            Some(Language::TypeScript)
        );
        assert_eq!(
            Language::from_path(Path::new("foo.tsx")),
            Some(Language::TypeScript)
        );
    }

    #[test]
    fn from_path_python() {
        assert_eq!(
            Language::from_path(Path::new("foo.py")),
            Some(Language::Python)
        );
    }

    #[test]
    fn from_path_go() {
        assert_eq!(Language::from_path(Path::new("foo.go")), Some(Language::Go));
    }

    #[test]
    fn from_path_unsupported() {
        assert_eq!(Language::from_path(Path::new("foo.java")), None);
        assert_eq!(Language::from_path(Path::new("foo.c")), None);
        assert_eq!(Language::from_path(Path::new("foo")), None);
    }

    #[test]
    fn display() {
        assert_eq!(Language::Rust.to_string(), "rust");
        assert_eq!(Language::TypeScript.to_string(), "typescript");
        assert_eq!(Language::Python.to_string(), "python");
        assert_eq!(Language::Go.to_string(), "go");
    }
}
