# Local Continuous Integration Specification

## Overview

Due to GitHub Actions billing on private repositories, CI must run locally on developer and agent workstations before pushing commits to upstream branches.

## Requirements

1. **Pre-push Hook**: Every local repository instance must configure Git to use `.githooks` (`git config core.hooksPath .githooks`).
2. **Hook Execution**: `.githooks/pre-push` must execute `just ci`.
3. **Check Matrix**: `just ci` must execute and pass:
   - Code formatting validation (`cargo fmt --all -- --check`)
   - Static analysis / linting (`cargo clippy --all-targets --all-features -- -D warnings`)
   - Test suite execution (`cargo test --all-targets --all-features`)
4. **Zero Bypass**: Agents and developers must never bypass the pre-push hook using `git push --no-verify`.
