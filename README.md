# rust-logseq-ast

Rust workspace for a CLI that transforms a markdown file written in a **Logseq**-style log format into an **AST**.

## Layout

- `crates/logseq-core/` — parsing + AST types (library)
- `crates/logseq-cli/` — command line interface (binary)

## Dev

Once Rust is installed:

```bash
cargo test
cargo clippy
cargo fmt
```
