# Releasing logseq-ast

This repo publishes prebuilt binaries to the GitHub Releases page.

## Create a release

1. Bump version(s) if desired:
   - `crates/logseq-cli/Cargo.toml` (package `logseq-ast`)
   - optionally `crates/logseq-core/Cargo.toml`

2. Commit the version bump.

3. Tag and push:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Pushing a tag `v*` triggers the GitHub Actions **Release** workflow, which:
- builds `logseq-ast` for Linux (x86_64), macOS (x86_64 + Apple Silicon), and Raspberry Pi targets (armv7 + aarch64)
- uploads `.tar.gz` archives to the GitHub Release for that tag

## CI on main

Every push/PR runs:
- fmt
- clippy
- tests
