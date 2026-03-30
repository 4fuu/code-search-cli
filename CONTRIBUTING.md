# Contributing

## Prerequisites

- Rust stable toolchain

## Build

```bash
cargo build
cargo build --release
```

## Test

```bash
cargo test --all-targets
```

Integration tests use fixture files under `tests/fixtures/` (one sample source file per supported language). When adding a new language, add a corresponding `tests/fixtures/sample.<ext>` that covers the symbol kinds your queries target.

## Lint

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Format your code before submitting:

```bash
cargo fmt
```

## Adding a Language

1. Add the `tree-sitter-<lang>` crate to `Cargo.toml`.
2. Create `src/lang/<lang>.rs` implementing the language support module.
3. Register the language in `src/core/language.rs`.
4. Add query files under `queries/<lang>/symbols.scm` and `queries/<lang>/references.scm`.

## Pull Requests

- Keep PRs focused — one feature or fix per PR.
- CI must pass (fmt, clippy, tests on Linux / macOS / Windows) before merge.

## License

By contributing, you agree your changes will be licensed under [MIT](./LICENSE).
