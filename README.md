# codes — Fast Symbol Search for AI Agents

[中文说明](./README.zh-CN.md)

`codes` is a Tree-sitter based code navigation CLI built for AI agents. Instead of reading entire files with `grep` or `Read`, agents can query symbols directly — getting precise, low-token answers about definitions, references, and file structure.

Indexed rust-lang/rust (35k files, 305k symbols) in **9.5 seconds** on a cold cache (AMD Ryzen 5 7500F, 2 threads).

## Why codes

AI agents exploring a codebase waste tokens on broad grep scans and full file reads. `codes` changes the workflow:

| Task | Without codes | With codes |
|------|--------------|-----------|
| Find where `Parser` is defined | `rg -n "struct Parser"` across repo, read several files | `codes definition --name Parser` → one line |
| See what a file exports | Read the whole file | `codes overview src/parser.rs` → symbol list only |
| Find all call sites of a function | `rg -n "fn_name"` with false positives | `codes references --name fn_name` → AST-aware, kind-filtered |
| Explore what symbols exist | Guess file paths, open files | `codes symbols --name parse` → repo-wide instant results |

**Rule of thumb**: thinking about a *symbol name* → `codes`. Thinking about *raw text* (log strings, SQL, env vars, routes) → `rg`/`grep`.

## Features

- `codes overview <file>` — parse a single file without touching the global cache
- `codes symbols` — repo-wide symbol search with kind and language filters
- `codes definition` — exact-name definition lookup
- `codes references` — kind-aware AST reference search (fewer false positives than grep)
- `codes index` — prewarm the cache
- Incremental refresh before every query — modified files are visible immediately
- `codes skill install` — install an agent skill file for Claude Code or Codex
- Text and JSON output modes

## Installation

### Homebrew

```bash
brew tap 4fuu/code-search-cli https://github.com/4fuu/code-search-cli
brew install 4fuu/code-search-cli/codes
```

### Scoop

```powershell
scoop bucket add code-search-cli https://github.com/4fuu/code-search-cli
scoop install codes
```

### Shell installer

Installs `codes` into `~/.local/bin`.

```bash
curl -fsSL https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.sh | bash
```

Specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.sh | bash -s -- v0.1.0
```

<details>
<summary>PowerShell installer (fallback)</summary>

```powershell
iwr https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.ps1 -UseBasicParsing | iex
Install-Codes
```

Specific version:

```powershell
iwr https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.ps1 -UseBasicParsing | iex
Install-Codes -Version v0.1.0
```

</details>

## Usage

```bash
codes overview src/core/parser.rs
codes symbols --name parse
codes definition --name Parser
codes references --name Parser --include-def
codes index
codes clear-cache
codes skill install --target claude-code
```

## Command Reference

```text
codes [-j <n>] <subcommand>

codes overview <file> [--format text|json]
codes symbols [--name <substr>] [--kind <kind>] [--lang <lang>] [--path <glob>] [--limit <n>] [--offset <n>] [--format text|json]
codes definition --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--limit <n>] [--offset <n>] [--format text|json]
codes references --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--include-def] [--limit <n>] [--offset <n>] [--format text|json]
codes index
codes clear-cache
codes skill print
codes skill install --target <codex|claude-code> [--force]
```

### Options

| Flag | Commands | Description |
|------|----------|-------------|
| `--name` | symbols, definition, references | symbols: case-insensitive substring; definition/references: exact match |
| `--kind` | all search | Filter by symbol kind (tab-completable) |
| `--lang` | all search | Filter by language (aliases: `rs`, `ts`, `py`) |
| `--path` | all search | Filter by file path — substring or glob with `*` |
| `--limit` | symbols, definition, references | Cap number of results (default: 100) |
| `--offset` | symbols, definition, references | Skip the first N results |
| `--include-def` | references | Include the definition site in results |
| `--format` | all | `text` (default) or `json` |
| `-j` | all | Number of threads for indexing |

## Output Example

```text
$ codes symbols --name parse
3 matches
function  parse       src/parser.rs:24  rust        pub fn parse(input: &str) -> Result<Ast>
method    parse       src/lib.rs:41     rust        pub fn parse(&mut self) -> Result<Vec<Token>>
class     Parser      src/parser.ts:12  typescript  export class Parser
```

## License

MIT. See [LICENSE](./LICENSE).

---

Inspired by [cx](https://github.com/ind-igo/cx), which introduced the idea of tree-sitter based symbol queries as an agent-first tool. After studying it I found enough design differences in indexing strategy, reference queries, and output model that I chose to write `codes` from scratch rather than fork.
