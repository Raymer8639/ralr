# Contributing to ralr

Thank you for your interest in contributing.

## Development workflow

1. Fork the repository and clone it locally.
2. Create a feature branch from `main`.
3. Make your changes, following the conventions below.
4. Run tests and ensure everything passes.
5. Commit with a descriptive message.
6. Push your branch and open a pull request.

## Conventions

### Every feature → test + docs

For every new feature or bug fix:
- Add or update tests under `tests/tests/`.
- Update relevant documentation (`CLAUDE.md`, docs under `docs/`, or README.md).
- If it changes user-facing behavior, update `CHANGELOG.md`.

### Every release → version bump

When publishing a release:
- Update the version in all `Cargo.toml` files.
- Update `CHANGELOG.md` with the release notes.
- Tag the release commit (`git tag vX.Y.Z`).

### Commit messages

Use imperative mood, short subject line, optional body for details:

```
Add bitwise NOT operator

Implements `~` unary operator for integer types.
Updates expression parser and eval_expr.
```

## Building

```bash
cargo build              # dev build
cargo build --release    # release build
cargo fmt                # format
cargo clippy             # lint
```

## Testing

```bash
cd tests && cargo test           # all tests
cd tests && cargo test <name>    # single test
```

Tests live in a separate Cargo workspace at `tests/`.

## Pull request checklist

- [ ] Code builds with `cargo build` and `cargo build --release`
- [ ] All tests pass (`cd tests && cargo test`)
- [ ] No clippy warnings
- [ ] `CHANGELOG.md` updated if applicable
- [ ] Relevant docs updated

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).
