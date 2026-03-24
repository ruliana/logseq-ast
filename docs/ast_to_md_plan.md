# logseq-ast-to-md — initial plan

## Goal

Create a second Unix-style CLI:

- Name: `logseq-ast-to-md`
- Input: JSON produced by `logseq-ast` (STDIN default, optional file path)
- Output: blog-style Markdown to STDOUT
- No frontmatter

## Required formatting rules (agreed)

### Inline conversion

- `[[Page]]` → plain text `Page`
- `((uuid))` → dropped
- `{{embed ...}}` → dropped
- `#tags` → dropped
- Markdown links `[label](url)` are preserved.

### Structure conversion (prose)

- Each top-level block becomes a paragraph.
- Child blocks become sentences appended after the parent paragraph.
- If a block’s rendered text is missing terminal punctuation, append `.`
  - terminal punctuation is `.`, `!`, `?`, `:`
- Exception: if a parent ends with `:`, children remain a markdown list, and **all descendants** remain nested list items.

### Blank lines

- Preserve blank line nodes as paragraph breaks.

### Code blocks

- Render fenced code blocks in Markdown.

## BDD plan

We’ll lock behavior with golden tests:

- Input fixture: `fixtures/ast_to_md/*.json` (AST inputs)
- Expected output: `fixtures/ast_to_md/*.md`

Scenarios to cover:
- Prose flattening (parent + children) + punctuation injection
- List mode triggered by parent ending with `:` (nested list retained)
- Inline transformations (page refs, dropped tags/embeds/blockrefs)
- Blank line handling (paragraph breaks)

## TODO (living)

- [ ] Define `logseq-ast-to-md` crate + wire into workspace
- [ ] Implement `render_blog_markdown(Document) -> String`
- [ ] Add golden fixtures + tests (AST JSON → Markdown)
- [ ] Add CLI tests (STDIN default, file input)
- [ ] Ensure `scripts/check.sh` passes
- [ ] Push repo updates
