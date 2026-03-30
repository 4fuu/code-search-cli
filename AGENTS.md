# code-search-cli

基于 Tree-sitter 的本地代码搜索 CLI，二进制名 `codes`。
覆盖 Rust / TypeScript(TSX) / Python / Go，输出 `text`（默认）和 `json` 两种格式。

**不做**：语义级 rename、完整类型推导、运行时下载 grammar、50+ 语言、服务端 daemon。

## CLI 命令

```
codes overview <file> [--format text|json]
codes symbols [--name <substr>] [--kind <kind>] [--lang <lang>] [--path <glob>] [--limit <n>] [--format text|json]
codes definition --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--format text|json]
codes references --name <name> [--kind <kind>] [--lang <lang>] [--path <glob>] [--include-def] [--format text|json]
codes index
codes clear-cache
codes skill print
```

- `--format` 默认 `text`
- `--kind` / `--lang` 均为枚举类型，支持 tab 补全，传错直接报错并列出合法值
- `--lang` 别名：`rs` / `ts` / `py` / `golang`
- `index` 可选预热；查询命令自动增量刷新并在需要时更新仓库级汇总索引

### 命令语义

- **overview**：单文件符号列表，不依赖全局缓存。
- **symbols**：全仓库符号搜索，`--name` 为大小写不敏感子串匹配；当查询串长度足够时，优先走仓库级名字子串索引缩小候选集，再做精确过滤。排序：名字精确 > 前缀 > 符号类型 > 路径字典序。`--limit` 在排序后截断。
- **definition**：按名字精确匹配（大小写不敏感）符号定义，同名多条正常返回。
- **references**：Kind-aware AST 名字级搜索（非完整语义引用）。流程：definition 找候选 → 推断 primary kind → 选择窄化 query → 过滤文件 → Tree-sitter 解析 → 启发式过滤。
  - 类型 kind（struct/enum/trait/…）：只抓 `type_identifier` 和 scoped path 前缀
  - 函数 kind（fn/method）：只抓 call expression 位置
  - 找不到 definition 时发出警告（结果退化为宽泛查询）
- **index**：输出 `Indexed N symbols across M files in Xs  (cached: C, updated: U)`
- **clear-cache**：删除 `.code-search/` 并打印路径；会与索引/查询命令共享同一把仓库级缓存锁，避免并发踩缓存。

### 警告机制

- `references`：definition 不在缓存时 → stderr 警告，结果可能不完整
- `symbols` / `definition`：同时指定了 `--kind` + `--lang` 且结果为空，而该语言本身不产生此 kind → stderr 警告

## 技术栈

- `tree-sitter` + 静态 grammar crate（不做动态下载）
- `ignore`：遍历仓库并尊重 `.gitignore`
- `serde` / `serde_json`：缓存与 JSON 输出
- `clap`（derive）：CLI
- `anyhow` / `thiserror`：错误处理

## 目录结构

```
src/
  main.rs
  cli.rs                  # clap 定义，--kind/--lang 均为 ValueEnum
  command/                # 各子命令入口
    overview.rs / symbols.rs / definition.rs / references.rs
    index.rs / skill.rs / clear_cache.rs
  core/
    repo.rs               # 找 git root，定位 .code-search/
    discover.rs           # 扫描候选文件，尊重 ignore 规则
    language.rs           # 后缀→语言映射，Language 枚举（含 ValueEnum）
    symbol.rs             # Symbol 结构和 SymbolKind 定义（含 ValueEnum）
    parser.rs             # 统一 parser 调用入口，缓存每语言 Query，复用线程本地 Parser
    query.rs              # find_references，接受 Option<SymbolKind> 选择 query
    cache.rs              # 单文件缓存 + 仓库级符号索引，manifest 读写，增量更新，并发锁与原子写
    output.rs             # text/json 输出，错误格式化
    error.rs
  lang/                   # 各语言 query 封装、节点解释、signature 提取
    mod.rs                # LanguageSupport trait，KindCategory，language_supports_kind()
    rust.rs / typescript.rs / python.rs / go.rs
queries/                  # Tree-sitter query 文件
  rust/                   symbols.scm  references.scm
  typescript/             symbols.scm  references.scm
  python/                 symbols.scm  references.scm
  go/                     symbols.scm  references.scm
tests/fixtures/
```

## 符号模型

```rust
struct Symbol {
    name: String,
    kind: SymbolKind,
    language: Language,
    path: String,
    container_name: Option<String>,  // 类名、impl 名、模块名
    signature: Option<String>,
    range: TextRange,                // 整个定义范围
    selection_range: TextRange,      // 符号名范围
    exported: bool,
}
```

## 语言支持

| 语言 | 抽取的符号类型 |
|------|---------------|
| Rust | `struct` `enum` `trait` `impl` `fn` `type_alias` `const` `static` `module` |
| TypeScript/TSX | `function` `class` `interface` `type_alias` `enum` `method` `variable`（仅导出或顶层） |
| Python | `class` `function` `method` `variable`（module-level assignment） |
| Go | `func` `method` `struct` `interface` `type_alias` `const` `var` |

各语言支持的 kind 在 `LanguageSupport::supported_kinds()` 中声明，添加新语言时须实现此方法。

## 缓存设计

仓库内 `.code-search/` 目录，增量更新，不用 DB；仓库根目录额外使用 `.code-search.lock` 做进程级互斥。

```
.code-search/
  .gitignore          # 内容: *  (包括自身在内全部忽略)
  manifest.json       # 文件→缓存映射 + fingerprint + symbol_count
  symbols.bin         # 仓库级汇总符号索引（含 exact name / substring 辅助索引）
  files/
    ab/abcd1234.bin   # 单文件符号缓存
```

- fingerprint = `size + mtime_ms`
- 查询时自动增量刷新：遍历候选文件 → 检查 fingerprint → 新增/变更重新解析 → 删除文件移除缓存 → 更新仓库级 `symbols.bin`
- `manifest.json` 仍然保留，因为它负责记录每个源文件的 fingerprint、cache key 和 symbol_count；没有这层元数据就无法高效判断增量刷新范围
- `refresh_cache` 返回 `RefreshResult`，其中包含 `RefreshStats`、`total_symbols` 和按需加载的仓库级 `SymbolIndex`
- manifest、仓库级索引、单文件缓存都通过临时文件写入后替换，减少半写入损坏概率
- 索引/查询/清缓存命令共享仓库级锁，避免多个 `codes` 进程并发破坏缓存
- 额外忽略 `node_modules` / `dist` / `build` / `target` / `.venv` / `vendor`

## 输出格式

### text（默认）

紧凑单行格式，节省 token：

```
# overview
# src/parser.rs [rust]
struct Parser                    :12

# symbols
3 matches
function  parse       src/parser.rs:24    rust  pub fn parse(input: &str) -> Result<Ast>

# definition
1 match
struct  Parser  src/parser.rs:12  rust  pub struct Parser

# references
6 references
src/lib.rs:41           Parser::parse(input)

# index
Indexed 279 symbols across 29 files in 0.08s  (0 cached, 29 updated)
```

### json

```json
{
  "command": "symbols",
  "repo_root": "/path/to/repo",
  "matches": [{ "name": "...", "kind": "...", "language": "...", "path": "...", "line": 0, "column": 0, "end_line": 0, "end_column": 0, "container_name": null, "signature": "..." }]
}
```

references json 含可选 `"warning"` 字段。

### 错误

- text: `Error: not inside a git repository`
- json: `{ "error": { "code": "NOT_IN_GIT_REPO", "message": "..." } }`

## 编码规范

- 路径内部统一相对 repo root，使用 `PathBuf`
- JSON 输出中 Windows 也用 `/` 风格路径
- 支持 Unicode 路径
- 支持 Linux / macOS / Windows

## 测试策略

 - 单元测试：路径识别、语言识别、manifest 读写、名字索引过滤逻辑、输出结构
- fixture 测试：每种语言小型 fixture，验证 overview / symbols / definition / references
- CI：ubuntu-latest / macos-latest / windows-latest

## 二进制大小

- release（strip 后）：~6.5MB，主要体积来自静态链接的 tree-sitter grammar

## Skill 输出

`codes skill print` 输出标准 SKILL.md（含 frontmatter），可重定向到 `.claude/skills/code-search-cli/SKILL.md`。
