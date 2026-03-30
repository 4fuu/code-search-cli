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
description: Fast Tree-sitter symbol search for Rust/TypeScript/Python/Go. Indexes all 305k symbols across 35k files in rust-lang/rust in ~10s. Use before rg/grep to locate symbols cheaply; fall back to rg/grep only for raw text.
---

# code-search-cli

Binary: `codes`

## When to use codes vs rg/grep

**Rule**: thinking about a *symbol name* → `codes`; thinking about *raw text* → `rg`/`grep`.

Use `codes` first to locate symbols with minimal token cost. Only open files with `rg`/`grep`/Read
after you know exactly where to look.

Typical workflow:
1. `codes symbols --name <substr>` — locate candidates across the whole repo instantly
2. `codes definition --name <name>` — jump straight to the definition
3. `codes references --name <name>` — AST-aware call-site search, fewer false positives than rg/grep
4. `codes overview <file>` — get the symbol skeleton before reading a file in full
5. Once narrowed to 1–3 files, switch to `rg`/`grep` or Read for literals, log strings, SQL, env vars, routes

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
