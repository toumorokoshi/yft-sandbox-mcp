# Linting

- linting commands are found in the `justfile`.

## Linting

- linting is available via `just lint`, which usually routes to the linter of the particular language of the repository, such as `npm` or `cargo`.

## Formatting

If there is no best practice formatter for the language of choice, then
`prettier` is used. It is available via `just format`.

## Fix

Auto-fixing is available via `just fix`.
