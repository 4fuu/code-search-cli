use crate::core::output::print_symbols;
use crate::{search_definitions, DefinitionArgs, DefinitionSearchRequest};
use anyhow::Result;

pub fn run(args: DefinitionArgs) -> Result<()> {
    let result = search_definitions(DefinitionSearchRequest {
        repo_path: std::env::current_dir()?,
        name: args.name.clone(),
        kind: args.kind,
        language: args.lang,
        path_pattern: args.path.clone(),
        limit: args.limit,
        offset: args.offset,
    })?;
    for warning in &result.warnings {
        eprintln!("warning: {warning}");
    }

    print_symbols(
        &result.items,
        "definition",
        &args.format,
        Some(result.total),
        args.offset,
        args.limit,
    )
}
