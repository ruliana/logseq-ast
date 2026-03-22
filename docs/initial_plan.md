# Initial plan — Logseq Markdown → AST (JSON)

## Goal

Build a small Unix-style CLI that:

- takes **one Logseq page markdown file** as input
- parses it into a well-defined **AST**
- prints the AST as **JSON to STDOUT**
- uses STDERR + exit codes for errors

Non-goals (initially):
- Rendering, graph-wide resolution, or evaluating embeds/queries
- Reading an entire graph directory
- Mutating files

## Spec / syntax sources we will follow

Primary reference (Logseq docs):
- Logseq docs page: `Markdown.md` (mentions that Logseq’s markdown is parsed by `mldoc`)
  - Page reference: `[[page name]]`
  - Block reference: `((block-uuid))`
  - Embed page: `{{embed [[page name]]}}`
  - Embed block: `{{embed ((block-uuid))}}`
  - Properties: `property:: value`
  - Page ref with label: `[display text]([[page name]])`
  - Block ref with label: `[display text](((block-uuid)))`
Source: https://github.com/logseq/docs/blob/master/pages/Markdown.md

We will also preserve standard Markdown constructs that appear in Logseq pages:
- headings, paragraphs
- emphasis/strong/inline code
- links/images
- fenced code blocks

## “Pay attention to” items (first-class in our AST)

### 1) Page references
- `[[Page Name]]`
- label form: `[label]([[Page Name]])`

### 2) Block references
- `((uuid))`
- label form: `[label](((uuid)))`

### 3) Links
- Standard markdown links: `[label](https://example.com)`
- We must not confuse `([label]([[page]]))` with `(url)` links.

### 4) Embeds
- `{{embed [[page]]}}`
- `{{embed ((uuid))}}`

We will represent embeds as nodes, but we will **not expand** them (no graph access).

### 5) Code blocks
- Preserve fenced code blocks as-is (info string + raw content).
- Inline code should be preserved.

### 6) Properties
- `property:: value` lines.
- In Logseq, properties can appear at block level. We should model them in the AST without losing ordering.

### 7) Tasks / markers (optional but likely important)
The docs mention task markers like `TODO`, `DOING`, `DONE`, `LATER`, `NOW`.
We should parse these if they appear at the beginning of a block.

### 8) Wiki links / page references ("wiki" in Logseq)
By “wiki” you meant Logseq’s wiki-style page references.

We will treat these as first-class syntactic nodes:
- `[[Page Name]]`
- labeled form: `[label]([[Page Name]])`
- tags as page-ish refs:
  - `#tag`
  - `#[[Multi Word Tag]]`

Since we are parsing **a single page file**, we will not validate whether a referenced page exists in the graph; we only normalize + capture the reference.

## CLI (Unix philosophy)

Binary name: `logseq`

Proposed interface:

```bash
logseq <path/to/page.md> > ast.json

# later (optional)
logseq - < page.md > ast.json
```

- Input: file path, or `-` for STDIN (we can add `-` support early)
- Output: always the AST JSON on STDOUT
- Errors: human-readable parse error on STDERR
- Exit codes: `0` success, `2` parse error, `1` other I/O errors

## AST design (v1)

### Top-level

```jsonc
{
  "type": "Document",
  "version": 1,
  "items": [ ... ]
}
```

### Block structure
Logseq pages are block-oriented. Even when written in Markdown, content is often represented as a tree of blocks.

We will model **block nodes** with:
- `id` (optional; might be present in properties as `id:: <uuid>`)
- `indent` or `level` (derived from markdown list nesting / indentation)
- `properties` map (ordered list in addition to a map to preserve duplicates if needed)
- `marker` (TODO/DOING/etc, optional)
- `content` as inline nodes
- `children` blocks

### Inline nodes
We’ll represent the “inner reference” constructs as explicit inline nodes:
- `Text { value }`
- `PageRef { title, original }`
- `BlockRef { uuid }`
- `Link { label: [Inline], url }`
- `Embed { target: PageRef|BlockRef }`
- `CodeSpan { code }`
- `Emphasis/Strong/Strikethrough/Highlight` (as needed)

### Code blocks
- `CodeBlock { info: Option<String>, text: String }`

### Headings
Logseq markdown pages can contain headings; we’ll include:
- `Heading { level, content: [Inline] }`

## Parsing strategy

We need to parse **Logseq-specific tokens** and **Markdown structure**.

### Approach A (recommended for correctness): 2-stage parse
1) **Block/line parser** for Logseq page structure:
   - read file line-by-line
   - detect list bullets / indentation
   - detect property lines (`key:: value`)
   - build a preliminary block tree with raw text for each block
2) **Inline parser** applied to each block’s raw text:
   - scan for Logseq tokens (`[[...]]`, `((...))`, `{{embed ...}}`, `[...]([[...]])`, `[...](((...)))`)
   - for remaining spans, parse standard Markdown inline elements (or keep as plain text initially)

This keeps the “Logseq-ness” explicit and avoids fighting a general Markdown parser.

### Approach B (fallback): leverage a Markdown parser + post-process
Use `pulldown-cmark` to parse markdown into events, then detect Logseq constructs inside text events.

This is simpler, but can be tricky because Logseq’s block semantics do not map 1:1 to Markdown AST events.

**Plan**: start with Approach A for blocks + a small inline tokenizer.

## Development approach: BDD + unit tests + coverage

- We’ll develop **behavior-first**: for each feature, write a human-readable scenario and make it pass.
- Unit tests live in `logseq-core` (fast, deterministic).
- We’ll use **golden tests** (fixture markdown → expected JSON) to lock behavior.
- Coverage: use `cargo-tarpaulin`.

Commands:

```bash
# unit + golden tests
cargo test

# coverage (local)
# Note: this environment requires the LLVM engine (ptrace/ASLR disable is not permitted).
cargo tarpaulin --workspace --engine llvm --out Html --output-dir target/coverage
```

Test organization:
- `crates/logseq-core/tests/*.rs` for integration-style golden tests
- `crates/logseq-core/src/*` `#[cfg(test)]` for unit tests
- `fixtures/` directory containing:
  - `*.md` input pages
  - `*.json` expected AST output

Definition of Done for a feature:
- [ ] scenario(s) written
- [ ] unit tests passing
- [ ] golden test(s) added
- [ ] coverage doesn’t decrease materially (we’ll enforce a threshold once we have enough tests)

## To-do (living checklist)

- [ ] Confirm scope: what exactly counts as a “block” in your Logseq page files (bullet-only vs any non-empty line).
- [ ] Decide how to represent *ordering*: keep raw line order + keep properties as both (a) list and (b) map.
- [ ] Define AST v1 schema in `logseq-core` (Document / Block / Inline / CodeBlock / Heading / Property / Marker).
- [ ] Add fixtures (`fixtures/*.md`) + golden JSON outputs (`fixtures/*.json`).
- [ ] Implement block tree parser (indentation + bullets + children).
- [ ] Implement property parsing (`key:: value`) + capture `id:: <uuid>`.
- [ ] Implement inline tokenizer for:
  - [ ] wiki links `[[...]]`
  - [ ] block refs `((uuid))`
  - [ ] embeds `{{embed ...}}`
  - [ ] labeled page ref `[label]([[page]])`
  - [ ] labeled block ref `[label](((uuid)))`
  - [ ] tags `#tag` and `#[[multi word]]`
  - [ ] standard links `[label](url)`
  - [ ] code spans `` `code` ``
- [ ] Handle fenced code blocks (```lang ... ```), preserving info string + content.
- [ ] Parse task markers at start of block (TODO/DOING/DONE/NOW/LATER) + priorities [#A]/[#B]/[#C] if desired.
- [ ] Implement CLI:
  - [ ] file path input
  - [ ] `-` for STDIN
  - [ ] JSON output to STDOUT
  - [ ] good errors to STDERR + exit codes
- [ ] Add `--debug-tokens` (optional) to print inline tokens for troubleshooting.
- [ ] Run `cargo fmt`, `cargo clippy`, `cargo test` in CI-like loop.

## Detailed milestones

### Milestone 0 — Fixtures and golden tests
- Collect 10–20 sample Logseq page snippets covering:
  - page refs, block refs, labeled refs
  - embeds
  - links (HTTP + file)
  - fenced code blocks
  - properties and `id::`
  - nested bullet blocks
  - mixed inline constructs
- Create golden JSON outputs for these fixtures.

### Milestone 1 — Define AST schema in `logseq-core`
- Replace placeholder `Unknown` node with real structs/enums.
- Ensure AST is stable + serializable with `serde`.

### Milestone 2 — Block tree parser
- Parse indentation + bullet markers into a tree.
- Attach property lines to the right block.
- Preserve original line numbers for good error messages.

### Milestone 3 — Inline tokenizer for Logseq constructs
- Implement a small scanner that emits inline tokens:
  - `[[...]]`
  - `((uuid))` with UUID validation (accept canonical UUID format; keep raw if invalid)
  - `{{embed ...}}`
  - label forms `[text]([[page]])` and `[text](((uuid)))`
- Add unit tests for ambiguous cases.

### Milestone 4 — CLI polish
- Support `-` for STDIN.
- `--format json` stays default; keep room for future formats.

### Milestone 5 — Error handling + reporting
- `ParseError` includes location: line/column if possible.
- Avoid panics; return actionable messages.

## Open questions (to confirm early)

1) **Exact Logseq “log format”**: are you referring to standard Logseq page markdown with bullets, or a specific convention (e.g., a custom “log page” template)?
2) Should we treat every non-empty line as a block, or only bullet-prefixed lines (e.g., `- `)?
3) For properties: do you want them preserved as raw strings, or parsed into typed values (numbers/bools/arrays)?

---

Once you confirm the answers, we can lock the AST schema and start implementing Milestone 1.
