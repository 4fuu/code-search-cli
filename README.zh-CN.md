# codes — 面向 AI Agent 的快速符号搜索

[English README](./README.md)

`codes` 是一个基于 Tree-sitter 的代码导航 CLI，专为 AI Agent 设计。Agent 不再需要用 `grep` 或逐行读取文件来定位代码——直接查询符号，获得精准、低 token 消耗的结果。

冷缓存下索引 rust-lang/rust（35k 文件、305k 符号）仅需 **9.5 秒**（AMD Ryzen 5 7500F，双线程）。

## 为什么用 codes

AI Agent 探索代码库时，大量 token 浪费在宽泛的 grep 扫描和整文件读取上。`codes` 改变这个流程：

| 任务 | 不用 codes | 用 codes |
|------|-----------|---------|
| 找 `Parser` 定义在哪 | 全仓库 `rg "struct Parser"`，再读几个文件 | `codes definition --name Parser` → 一行结果 |
| 看一个文件导出了什么 | 读整个文件 | `codes overview src/parser.rs` → 仅符号列表 |
| 找函数的所有调用方 | `rg "fn_name"` 有大量误报 | `codes references --name fn_name` → AST 级别，按 kind 过滤 |
| 探索仓库有哪些符号 | 猜文件路径、打开文件 | `codes symbols --name parse` → 全仓库瞬时结果 |

**判断规则**：脑子里想的是*符号名* → 用 `codes`。想的是*原始文本*（日志字符串、SQL、环境变量、路由字面量）→ 用 `rg`/`grep`。

## 功能

- `codes overview <file>` — 解析单文件，无需全局缓存
- `codes symbols` — 全仓库符号搜索，支持 kind 和语言过滤
- `codes definition` — 精确名字定义查询
- `codes references` — kind-aware 的 AST 引用搜索，误报少于 grep
- `codes index` — 预热缓存
- 查询前自动增量刷新，文件改动后立即可见
- `codes skill install` — 为 Claude Code 或 Codex 安装 agent skill 文件
- 支持 `text` / `json` 两种输出

## 安装

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

### Shell 安装脚本

安装到 `~/.local/bin`：

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
codes references --name Parser --include-def
codes index
codes clear-cache
codes skill install --target claude-code
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

### 选项

| 参数 | 适用命令 | 说明 |
|------|----------|------|
| `--name` | symbols, definition, references | symbols：大小写不敏感子串；definition/references：精确匹配 |
| `--kind` | 所有搜索命令 | 按符号类型过滤（支持 tab 补全） |
| `--lang` | 所有搜索命令 | 按语言过滤（别名：`rs`、`ts`、`py`） |
| `--path` | 所有搜索命令 | 按文件路径过滤——子串或含 `*` 的 glob |
| `--limit` | symbols, definition, references | 限制结果数量（默认 100） |
| `--offset` | symbols, definition, references | 跳过前 N 条结果 |
| `--include-def` | references | 在结果中包含定义位置 |
| `--format` | 所有命令 | `text`（默认）或 `json` |
| `-j` | 所有命令 | 索引线程数 |

## 输出示例

```text
$ codes symbols --name parse
3 matches
function  parse       src/parser.rs:24  rust        pub fn parse(input: &str) -> Result<Ast>
method    parse       src/lib.rs:41     rust        pub fn parse(&mut self) -> Result<Vec<Token>>
class     Parser      src/parser.ts:12  typescript  export class Parser
```

## 协议

MIT，见 [LICENSE](./LICENSE)。

---

灵感来自 [cx](https://github.com/ind-igo/cx)，它最早提出了将 tree-sitter 符号查询作为 agent 优先工具的思路。研究之后发现两者在索引策略、引用查询和输出模型上有足够多的设计差异，因此选择从头重写，而非在其基础上 fork。
