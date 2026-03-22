# logseq-ast

A small, Unix-style CLI that parses a **single Logseq Markdown page** and prints an **Abstract Syntax Tree (AST)** as **JSON**.

This project is intentionally conservative and test-driven:
- it focuses on *parsing* (not rendering)
- it aims to preserve structure + ordering (including blank lines)
- it ships with golden (BDD-ish) fixtures and CLI integration tests

## Status

Working features:
- Bullet blocks (`- ` / `* `) + nested children via indentation
- Properties (`key:: value`) attach to the previous block
- Task markers: `TODO`, `DOING`, `DONE`, `NOW`, `LATER`
- Logseq inline constructs:
  - wiki page refs: `[[Page]]`
  - block refs: `((uuid))`
  - embeds: `{{embed [[Page]]}}`, `{{embed ((uuid))}}`
  - tags: `#tag`, `#[[multi word tag]]`
  - labeled refs / links: `[label]([[Page]])`, `[label](((uuid)))`
  - standard markdown links: `[label](https://example.com)`
  - code spans: `` `code` ``
- Fenced code blocks (```lang … ```) are parsed as **child blocks** when indented under a bullet (Option B)
- Property values also get a parsed representation (`value_ast`) including wiki refs/tags/block refs + **bare URLs**
- Blank lines are preserved as explicit nodes in the ordered top-level stream

## Install / Run

### Run from source

```bash
cd rust-logseq-ast

# read from a file
cargo run -p logseq-ast -- path/to/page.md > ast.json

# Unix filter style (STDIN is the default)
cat path/to/page.md | cargo run -p logseq-ast -- > ast.json

# explicit STDIN
cat path/to/page.md | cargo run -p logseq-ast -- - > ast.json
```

### Debug tokenization

`--debug-tokens` prints per-line inline tokenization hints to **STDERR** (STDOUT remains pure JSON):

```bash
echo '- Hello [[World]] #tag' | cargo run -p logseq-ast -- --debug-tokens > ast.json
```

## Output format (AST)

Top level:

- `version`: AST schema version
- `items`: ordered list of nodes
  - `{"type":"block", ...}`
  - `{"type":"blank_line","line": N}`

A `block` contains:
- `line`: original source line (1-indexed)
- `marker`: optional task marker
- `properties`: list of properties
  - each property contains:
    - `key`, `value` (raw string)
    - `value_ast` (parsed inline nodes; omitted if empty)
- `content`: inline nodes
- `children`: nested blocks

Inline nodes are tagged unions like:
- `text`, `page_ref`, `block_ref`, `tag`, `link`, `embed`, `code_span`, `code_block`

### Example

Input:

```md
- TODO Read [[Logseq]] docs #tag
  url:: https://example.com

  ```js
  console.log('hi')
  ```
```

Output (trimmed):

```json
{
  "version": 1,
  "items": [
    {
      "type": "block",
      "marker": "TODO",
      "properties": [
        {
          "key": "url",
          "value": "https://example.com",
          "value_ast": [
            {
              "type": "link",
              "label": [{ "type": "text", "value": "https://example.com" }],
              "url": "https://example.com"
            }
          ],
          "line": 2
        }
      ],
      "content": [
        { "type": "text", "value": "Read " },
        { "type": "page_ref", "title": "Logseq", "original": "Logseq" },
        { "type": "text", "value": " docs " },
        { "type": "tag", "title": "tag", "original": "tag" }
      ],
      "children": [
        {
          "type": "block",
          "properties": [],
          "content": [
            {
              "type": "code_block",
              "info": "js",
              "text": "  console.log('hi')"
            }
          ],
          "children": [],
          "line": 4
        }
      ],
      "line": 1
    },
    { "type": "blank_line", "line": 3 }
  ]
}
```

## Development

### One command to check everything

```bash
bash scripts/check.sh
```

This runs:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- optional: cyclomatic complexity via `km` (Kimün)
- optional: coverage via `cargo tarpaulin --engine llvm`

### Tests

- Golden fixtures live in `fixtures/` as `NAME.md` + `NAME.json`
- Golden tests: `crates/logseq-core/tests/golden.rs`
- CLI integration tests: `crates/logseq-cli/tests/cli.rs`

## Project layout

- `crates/logseq-core/` — AST types + parsing
- `crates/logseq-cli/` — CLI wrapper (binary name: `logseq-ast`)
- `fixtures/` — BDD-ish golden examples
- `docs/` — plan + notes
- `scripts/` — developer utilities

## License

MIT OR Apache-2.0
