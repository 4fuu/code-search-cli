use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[value(name = "rust", alias = "rs")]
    Rust,
    #[value(name = "javascript", alias = "js", alias = "jsx")]
    JavaScript,
    #[value(name = "typescript", alias = "ts")]
    TypeScript,
    #[value(name = "java")]
    Java,
    #[value(name = "python", alias = "py")]
    Python,
    #[value(name = "go", alias = "golang")]
    Go,
    #[value(name = "ruby", alias = "rb")]
    Ruby,
    #[value(name = "php")]
    Php,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "rust"),
            Language::JavaScript => write!(f, "javascript"),
            Language::TypeScript => write!(f, "typescript"),
            Language::Java => write!(f, "java"),
            Language::Python => write!(f, "python"),
            Language::Go => write!(f, "go"),
            Language::Ruby => write!(f, "ruby"),
            Language::Php => write!(f, "php"),
        }
    }
}

impl Language {
    /// Detect language from file extension. Returns `None` for unsupported extensions.
    pub fn from_path(path: &Path) -> Option<Language> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "rs" => Some(Language::Rust),
            "js" | "jsx" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "java" => Some(Language::Java),
            "py" => Some(Language::Python),
            "go" => Some(Language::Go),
            "rb" => Some(Language::Ruby),
            "php" => Some(Language::Php),
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
    fn from_path_javascript() {
        assert_eq!(
            Language::from_path(Path::new("foo.js")),
            Some(Language::JavaScript)
        );
        assert_eq!(
            Language::from_path(Path::new("foo.jsx")),
            Some(Language::JavaScript)
        );
    }

    #[test]
    fn from_path_java() {
        assert_eq!(
            Language::from_path(Path::new("Foo.java")),
            Some(Language::Java)
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
    fn from_path_ruby() {
        assert_eq!(
            Language::from_path(Path::new("foo.rb")),
            Some(Language::Ruby)
        );
    }

    #[test]
    fn from_path_php() {
        assert_eq!(
            Language::from_path(Path::new("foo.php")),
            Some(Language::Php)
        );
    }

    #[test]
    fn from_path_unsupported() {
        assert_eq!(Language::from_path(Path::new("foo.c")), None);
        assert_eq!(Language::from_path(Path::new("foo")), None);
    }

    #[test]
    fn display() {
        assert_eq!(Language::Rust.to_string(), "rust");
        assert_eq!(Language::JavaScript.to_string(), "javascript");
        assert_eq!(Language::TypeScript.to_string(), "typescript");
        assert_eq!(Language::Java.to_string(), "java");
        assert_eq!(Language::Python.to_string(), "python");
        assert_eq!(Language::Go.to_string(), "go");
        assert_eq!(Language::Ruby.to_string(), "ruby");
        assert_eq!(Language::Php.to_string(), "php");
    }
}
