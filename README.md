# code-search-cli

[中文说明](./README.zh-CN.md)

`codes` is a Tree-sitter based local code search CLI for Git repositories.

It supports:

- Rust
- TypeScript / TSX
- Python
- Go

It focuses on fast local symbol search, definition lookup, repository indexing, and heuristic references. It does not try to be a full language server or semantic rename engine.

## Features

- `codes overview <file>`: parse a single file without touching the global cache
- `codes symbols`: search symbols across the repository
- `codes definition`: find case-insensitive exact-name definitions
- `codes references`: run kind-aware AST reference search
- `codes index`: prewarm the cache and repository symbol index
- Incremental refresh before queries, so modified files are visible immediately
- Repository-level symbol index for faster `symbols --name` and definition lookup
- Text and JSON output modes

## Installation

### Scoop

Use this repository directly as a bucket:

```powershell
scoop bucket add code-search-cli https://github.com/4fuu/code-search-cli
scoop install codes
```

### Homebrew

Use this repository directly as a tap:

```bash
brew tap 4fuu/code-search-cli https://github.com/4fuu/code-search-cli
brew install 4fuu/code-search-cli/codes
```

### Shell installer

This installs `codes` into `~/.local/bin`.

```bash
curl -fsSL https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.sh | bash -s -- v0.1.0
```

<details>
<summary>PowerShell install script (fallback)</summary>

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
codes references --name Parser
codes index
codes clear-cache
codes skill install --target codex
```

## Command Reference

```text
codes [-j <n>] <subcommand>

codes overview <file> [--format text|json]
codes symbols [--name <substr>] [--kind <kind>] [--lang <lang>] [--path <glob>] [--limit <n>] [--format text|json]
codes definition --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--format text|json]
codes references --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--include-def] [--format text|json]
codes index
codes clear-cache
codes skill print
codes skill install --target <codex|claude-code> [--force]
```

## Output Example

```text
$ codes symbols --name parse
3 matches
function  parse       src/parser.rs:24  rust  pub fn parse(input: &str) -> Result<Ast>
method    parse       src/lib.rs:41     rust  pub fn parse(&mut self) -> Result<Vec<Token>>
class     Parser      src/parser.ts:12  typescript  export class Parser
```

## License

MIT. See [LICENSE](./LICENSE).
