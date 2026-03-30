use crate::cli::OverviewArgs;
use crate::core::error::AppError;
use crate::core::language::Language;
use crate::core::output::print_overview;
use crate::core::parser::parse_file;
use anyhow::Result;
use std::fs;

pub fn run(args: OverviewArgs) -> Result<()> {
    let path = &args.file;
    if !path.exists() {
        return Err(AppError::FileNotFound(path.display().to_string()).into());
    }
    let language = Language::from_path(path)
        .ok_or_else(|| AppError::UnsupportedLanguage(path.display().to_string()))?;
    let source = fs::read_to_string(path)?;
    let file_label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let symbols = parse_file(path, &source, language)?;
    print_overview(&symbols, &file_label, &args.format)
}
