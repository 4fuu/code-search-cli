use crate::cli::OutputFormat;
use crate::core::query::Reference;
use crate::core::symbol::Symbol;
use anyhow::Result;
use serde::Serialize;

/// Print symbols in the requested format for the `overview` command.
pub fn print_overview(symbols: &[Symbol], file_label: &str, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!(
                "# {} [{}]",
                file_label,
                symbols
                    .first()
                    .map(|s| s.language.to_string())
                    .unwrap_or_default()
            );
            for sym in symbols {
                let sig = sym.signature.as_deref().unwrap_or("");
                if sig.is_empty() {
                    println!("{} {}    :{}", sym.kind, sym.name, sym.range.line);
                } else {
                    println!(
                        "{} {}    :{}    {}",
                        sym.kind, sym.name, sym.range.line, sig
                    );
                }
            }
        }
        OutputFormat::Json => {
            let output = JsonOutput {
                command: "overview",
                repo_root: None,
                matches: symbols.iter().map(SymbolMatch::from).collect(),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

/// Print symbols for `symbols` / `definition` commands.
pub fn print_symbols(symbols: &[Symbol], command: &str, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Text => {
            println!(
                "{} {}",
                symbols.len(),
                if symbols.len() == 1 {
                    "match"
                } else {
                    "matches"
                }
            );
            for sym in symbols {
                let sig = sym.signature.as_deref().unwrap_or("");
                println!(
                    "{}  {}  {}:{}  {}  {}",
                    sym.kind, sym.name, sym.path, sym.range.line, sym.language, sig
                );
            }
        }
        OutputFormat::Json => {
            let output = JsonOutput {
                command,
                repo_root: None,
                matches: symbols.iter().map(SymbolMatch::from).collect(),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

/// Print references.
pub fn print_references(
    refs: &[Reference],
    format: &OutputFormat,
    warning: Option<&str>,
) -> Result<()> {
    match format {
        OutputFormat::Text => {
            if let Some(msg) = warning {
                eprintln!("warning: {}", msg);
            }
            println!(
                "{} {}",
                refs.len(),
                if refs.len() == 1 {
                    "reference"
                } else {
                    "references"
                }
            );
            for r in refs {
                println!("{}:{}    {}", r.path, r.line, r.context);
            }
        }
        OutputFormat::Json => {
            let output = JsonRefOutput {
                command: "references",
                references: refs
                    .iter()
                    .map(|r| RefMatch {
                        path: r.path.clone(),
                        line: r.line,
                        column: r.column,
                        context: r.context.clone(),
                    })
                    .collect(),
                warning: warning.map(|s| s.to_string()),
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

/// Print an error in the requested format.
pub fn print_error(err: &crate::core::error::AppError, format: &OutputFormat) {
    match format {
        OutputFormat::Text => {
            eprintln!("Error: {}", err);
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "error": {
                    "code": err.code(),
                    "message": err.to_string(),
                }
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
    }
}

#[derive(Serialize)]
struct JsonOutput<'a> {
    command: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_root: Option<String>,
    matches: Vec<SymbolMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct SymbolMatch {
    name: String,
    kind: String,
    language: String,
    path: String,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
    container_name: Option<String>,
    signature: Option<String>,
}

impl From<&Symbol> for SymbolMatch {
    fn from(s: &Symbol) -> Self {
        SymbolMatch {
            name: s.name.clone(),
            kind: s.kind.to_string(),
            language: s.language.to_string(),
            path: s.path.clone(),
            line: s.range.line,
            column: s.range.column,
            end_line: s.range.end_line,
            end_column: s.range.end_column,
            container_name: s.container_name.clone(),
            signature: s.signature.clone(),
        }
    }
}

#[derive(Serialize)]
struct JsonRefOutput<'a> {
    command: &'a str,
    references: Vec<RefMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct RefMatch {
    path: String,
    line: usize,
    column: usize,
    context: String,
}
