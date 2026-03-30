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
                total: None,
                offset: 0,
                limit: crate::cli::DEFAULT_LIMIT,
                error: None,
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    Ok(())
}

/// Print symbols for `symbols` / `definition` commands.
pub fn print_symbols(
    symbols: &[Symbol],
    command: &str,
    format: &OutputFormat,
    total: Option<usize>,
    offset: usize,
    limit: usize,
) -> Result<()> {
    match format {
        OutputFormat::Text => {
            print_count_line(symbols.len(), total, offset, limit, "match", "matches");
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
                total,
                offset,
                limit,
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
    total: Option<usize>,
    offset: usize,
    limit: usize,
) -> Result<()> {
    match format {
        OutputFormat::Text => {
            if let Some(msg) = warning {
                eprintln!("warning: {}", msg);
            }
            print_count_line(refs.len(), total, offset, limit, "reference", "references");
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
                total,
                offset,
                limit,
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

fn print_count_line(
    count: usize,
    total: Option<usize>,
    offset: usize,
    limit: usize,
    singular: &str,
    plural: &str,
) {
    let noun = if count == 1 { singular } else { plural };
    if let Some(total) = total {
        let paginated = offset > 0 || total > limit;
        if paginated {
            let start = if count == 0 { 0 } else { offset + 1 };
            let end = offset + count;
            println!(
                "{} {} (showing {}-{} of {})",
                count, noun, start, end, total
            );
        } else if total != count {
            println!("{} {} ({} total)", count, noun, total);
        } else {
            println!("{} {}", count, noun);
        }
    } else {
        println!("{} {}", count, noun);
    }
}

#[derive(Serialize)]
struct JsonOutput<'a> {
    command: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    repo_root: Option<String>,
    matches: Vec<SymbolMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<usize>,
    #[serde(skip_serializing_if = "is_zero")]
    offset: usize,
    #[serde(skip_serializing_if = "is_default_limit")]
    limit: usize,
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
    total: Option<usize>,
    #[serde(skip_serializing_if = "is_zero")]
    offset: usize,
    #[serde(skip_serializing_if = "is_default_limit")]
    limit: usize,
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

fn is_zero(value: &usize) -> bool {
    *value == 0
}

fn is_default_limit(value: &usize) -> bool {
    *value == crate::cli::DEFAULT_LIMIT
}
