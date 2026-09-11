# Contributing

Thanks for your interest in improving `antigravity-powerline`!

## Development setup

You need a stable Rust toolchain (install via [rustup](https://rustup.rs) if you don't have one). Start by forking the repository, then clone **your fork**:

```bash
git clone https://github.com/<your-username>/antigravity-powerline.git
cd antigravity-powerline
cargo test
```

To keep your fork up to date, add the upstream repository:

```bash
git remote add upstream https://github.com/xpepper/antigravity-powerline.git
git fetch upstream
```

## Pull request workflow

1. Create a branch on your fork for your change.
2. Make your change, with tests for any new behavior.
3. Run the local checks below — CI runs the exact same commands on every PR.
4. Open a pull request against `main`. CI must pass before merging.

The `main` branch is protected: all changes land through pull requests, with `fmt`, `clippy`, and `test` as required status checks.

## Local checks

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Engineering constraints

See [AGENTS.md](AGENTS.md) for the full guidelines. The essentials:

- **Sub-15ms execution.** The binary runs inside an interactive status line loop: no heavy dependencies, no network calls, fast filesystem checks only.
- **Defensive stdin parsing.** Payloads from Antigravity CLI vary and may be empty: use `Option<T>` / `#[serde(default)]` and fall back gracefully, never panic.
- **Zero compiler warnings.** `cargo check` and `cargo test` must compile warning-free.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat(scope): ...`, `fix(scope): ...`, `perf(scope): ...`, `test(scope): ...`, `style: ...`, `ci: ...`, `docs: ...`, `chore: ...`.
