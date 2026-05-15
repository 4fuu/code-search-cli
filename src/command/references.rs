use crate::core::output::print_references;
use crate::{search_references, ReferenceSearchRequest, ReferencesArgs};
use anyhow::Result;

pub fn run(args: ReferencesArgs) -> Result<()> {
    let result = search_references(ReferenceSearchRequest {
        repo_path: std::env::current_dir()?,
        name: args.name.clone(),
        kind: args.kind,
        language: args.lang,
        path_pattern: args.path.clone(),
        include_definition: args.include_def,
        limit: args.limit,
        offset: args.offset,
    })?;
    let warning = (!result.warnings.is_empty()).then(|| result.warnings.join("; "));

    print_references(
        &result.items,
        &args.format,
        warning.as_deref(),
        Some(result.total),
        args.offset,
        args.limit,
    )
}
