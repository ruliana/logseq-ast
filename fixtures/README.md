# Fixtures

BDD-ish golden tests live here.

Each fixture is:
- `NAME.md` (input Logseq page)
- `NAME.json` (expected AST output)

Tests should parse the markdown and compare the produced JSON (stable ordering).
