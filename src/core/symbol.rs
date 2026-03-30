use crate::core::language::Language;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
#[value(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Interface,
    Class,
    TypeAlias,
    Const,
    Static,
    Variable,
    Module,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Struct => "struct",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Impl => "impl",
            SymbolKind::Interface => "interface",
            SymbolKind::Class => "class",
            SymbolKind::TypeAlias => "type_alias",
            SymbolKind::Const => "const",
            SymbolKind::Static => "static",
            SymbolKind::Variable => "variable",
            SymbolKind::Module => "module",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRange {
    pub line: usize,
    pub column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub language: Language,
    /// Path relative to repo root.
    pub path: String,
    pub container_name: Option<String>,
    pub signature: Option<String>,
    /// Entire definition range.
    pub range: TextRange,
    /// Symbol name range (for goto-definition).
    pub selection_range: TextRange,
    pub exported: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_kind_display() {
        assert_eq!(SymbolKind::Function.to_string(), "function");
        assert_eq!(SymbolKind::Method.to_string(), "method");
        assert_eq!(SymbolKind::Struct.to_string(), "struct");
        assert_eq!(SymbolKind::Enum.to_string(), "enum");
        assert_eq!(SymbolKind::Trait.to_string(), "trait");
        assert_eq!(SymbolKind::Impl.to_string(), "impl");
        assert_eq!(SymbolKind::Interface.to_string(), "interface");
        assert_eq!(SymbolKind::Class.to_string(), "class");
        assert_eq!(SymbolKind::TypeAlias.to_string(), "type_alias");
        assert_eq!(SymbolKind::Const.to_string(), "const");
        assert_eq!(SymbolKind::Static.to_string(), "static");
        assert_eq!(SymbolKind::Variable.to_string(), "variable");
        assert_eq!(SymbolKind::Module.to_string(), "module");
    }

    #[test]
    fn symbol_serde_roundtrip() {
        let sym = Symbol {
            name: "test_fn".to_string(),
            kind: SymbolKind::Function,
            language: Language::Rust,
            path: "src/lib.rs".to_string(),
            container_name: Some("MyStruct".to_string()),
            signature: Some("pub fn test_fn()".to_string()),
            range: TextRange {
                line: 10,
                column: 0,
                end_line: 15,
                end_column: 1,
            },
            selection_range: TextRange {
                line: 10,
                column: 7,
                end_line: 10,
                end_column: 14,
            },
            exported: true,
        };
        let json = serde_json::to_string(&sym).unwrap();
        let deser: Symbol = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.name, "test_fn");
        assert_eq!(deser.kind, SymbolKind::Function);
        assert_eq!(deser.language, Language::Rust);
        assert_eq!(deser.container_name, Some("MyStruct".to_string()));
        assert!(deser.exported);
    }
}
