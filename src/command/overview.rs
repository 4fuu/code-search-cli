use crate::core::output::print_overview;
use crate::{overview, OverviewArgs, OverviewRequest};
use anyhow::Result;

pub fn run(args: OverviewArgs) -> Result<()> {
    let path = &args.file;
    let file_label = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());

    let symbols = overview(OverviewRequest { path: path.clone() })?;
    print_overview(&symbols, &file_label, &args.format)
}
