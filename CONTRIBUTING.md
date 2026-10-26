# Contributing to antigravity-powerline

Thanks for improving `antigravity-powerline`. This project is a Rust status line for Google Antigravity CLI (`agy`), so changes should stay focused, fast, and compatible with the data the CLI provides.

## Getting started

Install the current stable [Rust toolchain](https://www.rust-lang.org/tools/install), then clone the repository and run the test suite:

```bash
git clone https://github.com/xpepper/antigravity-powerline.git
cd antigravity-powerline
cargo test --verbose
```

The project is a single Cargo package. Source code is in `src/`, status-line segments are in `src/segments/`, and user documentation is in `README.md`.

## Before opening a pull request

Run the same checks that CI runs:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --verbose
cargo build --release --verbose
scripts/mutants-diff
```

CI also checks shell scripts and enforces the mechanical repository rules (such as portable paths). See [the CI workflow](.github/workflows/ci.yml) for the complete checks and commands.

Documentation-only changes do not need Rust checks, but please confirm Markdown renders correctly and that commands and claims match the current implementation.

## Scope and design

Bug fixes, documentation improvements, and focused usability changes are welcome. For a new status-line segment, open an issue first and explain:

- the user problem it solves;
- which optional Antigravity CLI input fields provide the data;
- how it stays lightweight in a status-line refresh; and
- the configuration, tests, and README updates it needs.

Segments must handle missing input gracefully and avoid blocking the status line. See `AGENTS.md` for the project architecture and engineering constraints.

## Pull-request workflow

`main` is protected and changes merge through pull requests. Create a focused branch from current `main`, link the related issue when applicable, and use the pull-request template to describe motivation, behavior, verification, and documentation impact.

Keep changes small and avoid unrelated refactors. Required CI checks must pass, and review conversations must be resolved before merging.

### Optional: AI-assisted review

Maintainers review pull requests with the [gem-pr-review](https://github.com/xpepper/pr-review-gemini) plugin. [`.github/gem-pr-review.md`](.github/gem-pr-review.md) holds this repository's review notes: the hot-path rules, config compatibility contracts, and segment conventions.

You do not need the plugin to contribute. The file is plain Markdown, so read it as a short checklist before opening a pull request; the CI checks above are the merge bar either way.

## Releases

Maintainers should follow [RELEASING.md](RELEASING.md) when publishing a new version.

## Reporting issues

Use the bug-report template for reproducible problems and the feature-request template for proposals. Include only information needed to investigate; do not attach session data, credentials, or other sensitive information.
