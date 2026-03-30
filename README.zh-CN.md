# code-search-cli

[English README](./README.md)

`codes` 是一个基于 Tree-sitter 的本地代码搜索 CLI，面向 Git 仓库工作。

当前支持：

- Rust
- TypeScript / TSX
- Python
- Go

它专注于本地符号搜索、定义查询、仓库索引和启发式引用搜索，不试图替代完整语言服务或语义级 rename。

## 功能

- `codes overview <file>`：只解析单文件，不依赖全局缓存
- `codes symbols`：全仓库符号搜索
- `codes definition`：大小写不敏感的精确名字定义查询
- `codes references`：kind-aware 的 AST 引用搜索
- `codes index`：预热缓存和仓库级符号索引
- 查询前自动增量刷新，文件改动后可立即看到最新结果
- 仓库级符号索引可加速 `symbols --name` 与 `definition`
- 支持 `text` / `json` 两种输出

## 安装

### Scoop

直接把本仓库作为 bucket：

```powershell
scoop bucket add code-search-cli https://github.com/4fuu/code-search-cli
scoop install codes
```

### Homebrew

直接把本仓库作为 tap：

```bash
brew tap 4fuu/code-search-cli https://github.com/4fuu/code-search-cli
brew install 4fuu/code-search-cli/codes
```

### Shell 安装脚本

会把 `codes` 安装到 `~/.local/bin`。

```bash
curl -fsSL https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.sh | bash
```

安装指定版本：

```bash
curl -fsSL https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.sh | bash -s -- v0.1.0
```

<details>
<summary>PowerShell 安装脚本（备选）</summary>

```powershell
iwr https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.ps1 -UseBasicParsing | iex
Install-Codes
```

指定版本：

```powershell
iwr https://raw.githubusercontent.com/4fuu/code-search-cli/main/scripts/install.ps1 -UseBasicParsing | iex
Install-Codes -Version v0.1.0
```

</details>

## 用法

```bash
codes overview src/core/parser.rs
codes symbols --name parse
codes definition --name Parser
codes references --name Parser
codes index
codes clear-cache
codes skill install --target codex
```

## 命令一览

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

## 输出示例

```text
$ codes symbols --name parse
3 matches
function  parse       src/parser.rs:24  rust  pub fn parse(input: &str) -> Result<Ast>
method    parse       src/lib.rs:41     rust  pub fn parse(&mut self) -> Result<Vec<Token>>
class     Parser      src/parser.ts:12  typescript  export class Parser
```

## 协议

MIT，见 [LICENSE](./LICENSE)。
