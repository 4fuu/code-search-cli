use crate::cli::SkillCommand;
use anyhow::Result;

pub fn run(cmd: SkillCommand) -> Result<()> {
    match cmd {
        SkillCommand::Print => print_skill(),
    }
}

fn print_skill() -> Result<()> {
    print!("{}", SKILL_MD);
    Ok(())
}

const SKILL_MD: &str = r#"---
name: code-search-cli
description: Tree-sitter based local code search CLI (codes)
version: 0.1.0
languages: [rust, typescript, python, go]
---

# code-search-cli

Binary: `codes`

## Commands

```
codes overview <file> [--format text|json]
codes symbols [--name <substr>] [--kind <kind>] [--lang <lang>] [--path <glob>] [--limit <n>] [--format text|json]
codes definition --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--format text|json]
codes references --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--include-def] [--format text|json]
codes index
codes clear-cache
codes skill print
```

## Options

| Flag | Commands | Description |
|------|----------|-------------|
| `--name` | symbols, definition, references | symbols: case-insensitive substring; definition/references: exact match |
| `--kind` | all search | Filter by symbol kind (tab-completable) |
| `--lang` | all search | Filter by language (tab-completable, aliases: rs/ts/py) |
| `--path` | all search | Filter by file path — substring or glob with `*` |
| `--limit` | symbols | Cap number of results |
| `--include-def` | references | Include the definition site in results |
| `--format` | all | `text` (default, compact) or `json` |

## Supported Languages

| Language   | `--lang` values          | Extensions  |
|------------|--------------------------|-------------|
| Rust       | `rust`, `rs`             | .rs         |
| TypeScript | `typescript`, `ts`       | .ts, .tsx   |
| Python     | `python`, `py`           | .py         |
| Go         | `go`, `golang`           | .go         |

## Symbol Kinds

`function` `method` `struct` `enum` `trait` `impl` `interface` `class` `type_alias` `const` `static` `variable` `module`

## Output Formats

- `text` (default): Compact single-line, one symbol/reference per line
- `json`: Structured JSON with full metadata

## Behavior Notes

- `overview` works on a single file without a git repo or cache
- Cache lives in `.code-search/` and auto-refreshes incrementally on every query
- `references` is AST-aware (not grep): uses kind-specific Tree-sitter queries to reduce false positives
- When `references` finds no definition in cache, it warns and falls back to a broad query
- `index` prints: `Indexed N symbols across M files`
"#;
