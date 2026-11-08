# Local CI & Pre-Push Hook Guide

## Why Local CI?

Running GitHub Actions for private repositories costs money. To keep development cost-free, all continuous integration checks are run locally before code is pushed to upstream branches.

## Setup

Run the following command once after cloning the repository:

```bash
just install-hooks
```

This sets `git config core.hooksPath .githooks` and ensures executable permissions on `.githooks/pre-push`.

## What Runs in Local CI?

Running `just ci` runs the entire validation suite:

1. **`just format`**: `cargo fmt --all -- --check`
2. **`just lint`**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **`just test`**: `cargo test --all-targets --all-features`

## Git Integration

When you run `git push`, Git automatically invokes `.githooks/pre-push`. If any check in `just ci` fails, the push is aborted immediately, preventing broken or non-compliant commits from reaching remote branches.
