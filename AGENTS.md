## ALWAYS RUN

Important: this list is separate from any others in the prompt. Follow this
order of operations.

1. study the README.md
2. grep `specs/` for any relevant keywords to the task you are implementing.
3. study the relevant specs.
4. grep `design/` for any relevant keywords to the task you are implementing.
5. study the relevant documents.
6. implement the change requested in the prompt.
7. run linting and formatting before committing.
8. Identify any remaining issues or features that need to be implemented.
  8a. include them in GAPS.md
9. summarize any major decisions or changes into DESIGN-DECISION-LOG.md. Include the date.
10. run local CI (`just ci`) and verify all checks pass.
11. commit and push the change.

## Branch cleanup

When starting work, the branch should be clean, and you should try pull the
latest changes from the primary upstream branch before continuing.

## Add and update documentation

Always add and update documentation as appropriate. Update at least the following:

- any relevant files in the `docs/` directory.
- any updated designs and considerations in the `specs/` directory.

## Issue Management

File issues liberally to help keep context minimal and focus on your current
task.

- Issues are tracked in GAPS.md.
  - Priority should be set to the following (P2 otherwise):
    - P0: Any required project fundamentals or initialization

## Committing code

- **unless** the prompt contains "don't commit", commit the code.
- **unless** the prompt contains "don't push", push the code.

- Use the conventional commit format for commit messages.
- The commit description must explain the problem first.
- The commit description must a summary of each area modified.

## Commit and PR descriptions

- When writing commit or PR descriptions, use the `/yft-pr-commit-description`
  skill if available.

## Linting

- Always run linting and formatting before committing.
- Formatting and lint fixing tools are available via `just fix`.
- Linting tools are available via `just lint`.

## Testing

- linting, formatting, and testing **must** pass before a commit.
- add unit tests for every change if possible.

## CI

GitHub Actions CI is disabled because CI in GitHub for private repositories costs money. For this reason, CI **must** be executed locally and pass before every push.

- Run local CI via `just ci` to validate formatting, linting, and tests.
- Agents **must** ensure a pre-push hook is installed and configured (run `just install-hooks` to set `git config core.hooksPath .githooks`). The hook executes `just ci` on every `git push`.
- Always test local CI before pushing. Never bypass the pre-push hook.

## Code Design

The following code tenants are followed:

- functional programming as much as possible.
- separate state from functional programming.
- re-use code as much as possible.
- leverage best-practice third party libraries.

## Examples

- example data is in the `examples/` directory.
