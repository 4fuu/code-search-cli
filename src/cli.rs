use crate::core::language::Language;
use crate::core::symbol::SymbolKind;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "codes", about = "Tree-sitter based code search CLI")]
pub struct Cli {
    #[arg(
        short = 'j',
        long,
        global = true,
        help = "Number of threads for parallel operations (default: half logical CPUs)"
    )]
    pub threads: Option<usize>,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Show symbols in a single file
    Overview(OverviewArgs),
    /// Search symbols across the repository
    Symbols(SymbolsArgs),
    /// Find symbol definitions by name
    Definition(DefinitionArgs),
    /// Find references to a symbol
    References(ReferencesArgs),
    /// Pre-build the symbol cache
    Index,
    /// Remove the .code-search cache directory
    ClearCache,
    /// Skill management
    #[command(subcommand)]
    Skill(SkillCommand),
}

#[derive(clap::Args)]
pub struct OverviewArgs {
    /// File to analyze
    pub file: PathBuf,
    #[arg(long, default_value = "text", help = "Output format")]
    pub format: OutputFormat,
}

#[derive(clap::Args)]
pub struct SymbolsArgs {
    #[arg(long, help = "Filter by name (case-insensitive substring)")]
    pub name: Option<String>,
    #[arg(long, value_enum, help = "Filter by symbol kind")]
    pub kind: Option<SymbolKind>,
    #[arg(long, value_enum, help = "Filter by language")]
    pub lang: Option<Language>,
    #[arg(long, help = "Filter by file path (substring or glob with *)")]
    pub path: Option<String>,
    #[arg(long, help = "Limit the number of results")]
    pub limit: Option<usize>,
    #[arg(long, default_value = "text", help = "Output format")]
    pub format: OutputFormat,
}

#[derive(clap::Args)]
pub struct DefinitionArgs {
    #[arg(long, help = "Symbol name to find (case-insensitive exact match)")]
    pub name: String,
    #[arg(long, value_enum, help = "Filter by symbol kind")]
    pub kind: Option<SymbolKind>,
    #[arg(long, value_enum, help = "Filter by language")]
    pub lang: Option<Language>,
    #[arg(long, help = "Filter by file path (substring or glob with *)")]
    pub path: Option<String>,
    #[arg(long, default_value = "text", help = "Output format")]
    pub format: OutputFormat,
}

#[derive(clap::Args)]
pub struct ReferencesArgs {
    #[arg(
        long,
        help = "Symbol name to find references for (case-insensitive exact match)"
    )]
    pub name: String,
    #[arg(long, value_enum, help = "Narrow search to this symbol kind")]
    pub kind: Option<SymbolKind>,
    #[arg(long, value_enum, help = "Search only files in this language")]
    pub lang: Option<Language>,
    #[arg(
        long,
        help = "Search only files matching this path (substring or glob with *)"
    )]
    pub path: Option<String>,
    #[arg(long, help = "Include the definition site in results")]
    pub include_def: bool,
    #[arg(long, default_value = "text", help = "Output format")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum SkillCommand {
    /// Print the SKILL.md content to stdout
    Print,
}

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}
