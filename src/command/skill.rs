use crate::cli::{SkillCommand, SkillInstallArgs, SkillTarget};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run(cmd: SkillCommand) -> Result<()> {
    match cmd {
        SkillCommand::Print => print_skill(),
        SkillCommand::Install(args) => install_skill(args),
    }
}

fn print_skill() -> Result<()> {
    print!("{}", SKILL_MD);
    Ok(())
}

fn install_skill(args: SkillInstallArgs) -> Result<()> {
    let home_dir = detect_home_dir()?;
    let skill_path = install_path(args.target, &home_dir);

    if skill_path.exists() && !args.force {
        bail!(
            "{} already exists; re-run with --force to overwrite",
            skill_path.display()
        );
    }

    write_skill_file(&skill_path)?;
    println!("Installed {}", skill_path.display());
    Ok(())
}

fn detect_home_dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Some(profile) = std::env::var_os("USERPROFILE") {
        return Ok(PathBuf::from(profile));
    }
    bail!("could not determine home directory from HOME or USERPROFILE");
}

fn install_path(target: SkillTarget, home_dir: &Path) -> PathBuf {
    match target {
        SkillTarget::Codex => home_dir.join(".codex/skills/code-search-cli/SKILL.md"),
        SkillTarget::ClaudeCode => home_dir.join(".claude/skills/code-search-cli/SKILL.md"),
    }
}

fn write_skill_file(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .context("skill install path has no parent directory")?;
    fs::create_dir_all(parent)?;
    fs::write(path, SKILL_MD)?;
    Ok(())
}

const SKILL_MD: &str = r#"---
name: code-search-cli
description: Lightning-fast Tree-sitter symbol search for Rust/TypeScript/Python/Go. Use to find definitions, track usages, or explore code structure.
---

# code-search-cli

Binary: `codes`

## Speed

- Indexes 305k symbols across 35k files in ~10s (rust-lang/rust)
- Queries return in milliseconds using pre-built cache
- Incremental refresh: only re-parses changed files

## When to use codes vs rg/grep

**Quick Rule**:
- Looking for *code structures* (functions, classes, variables, etc.) → `codes`
- Looking for *text patterns* (log messages, comments, config values, etc.) → `rg`/`grep`

### Use `codes` when:
- You want to find **where a symbol is defined** → `codes definition --name <name>`
- You want to find **where a symbol is used/called** → `codes references --name <name>` (AST-aware, fewer false positives)
- You want to **browse all symbols** in a file → `codes overview <file>`
- You want to **search symbols by name** across the codebase → `codes symbols --name <substr>` (case-insensitive substring)

### Use `rg`/`grep` when:
- You're searching for **string literals**, log messages, or comments
- You need to search **configuration files**, SQL, or other non-code text
- You want **regex patterns** that span multiple symbols
- You need to search **outside indexed languages** (shell scripts, YAML, etc.)

### Typical workflow:
1. `codes symbols --name <substr>` — quickly find candidate symbols
2. `codes definition --name <name>` — jump to the definition
3. `codes references --name <name>` — find all usages (AST-aware)
4. `codes overview <file>` — get a file's symbol structure
5. If not found, fall back to `rg`/`grep` for literal text search

## Commands

```
codes overview <file> [--format text|json]
codes symbols [--name <substr>] [--kind <kind>] [--lang <lang>] [--path <glob>] [--limit <n>] [--offset <n>] [--format text|json]
codes definition --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--limit <n>] [--offset <n>] [--format text|json]
codes references --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--include-def] [--limit <n>] [--offset <n>] [--format text|json]
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
| `--limit` | symbols, definition, references | Cap number of results (default: 100) |
| `--offset` | symbols, definition, references | Skip the first N results |
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn install_path_for_codex() {
        let path = install_path(SkillTarget::Codex, Path::new("/tmp/home"));
        assert_eq!(
            path,
            PathBuf::from("/tmp/home/.codex/skills/code-search-cli/SKILL.md")
        );
    }

    #[test]
    fn install_path_for_claude_code() {
        let path = install_path(SkillTarget::ClaudeCode, Path::new("/tmp/home"));
        assert_eq!(
            path,
            PathBuf::from("/tmp/home/.claude/skills/code-search-cli/SKILL.md")
        );
    }

    #[test]
    fn write_skill_file_creates_parent_dirs_and_content() {
        let root = unique_temp_dir();
        let path = root.join(".codex/skills/code-search-cli/SKILL.md");

        write_skill_file(&path).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, SKILL_MD);

        fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("codes-skill-test-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
