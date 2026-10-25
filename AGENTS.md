# Agent Guidelines for `antigravity-powerline`

Welcome! This document provides architecture overviews, design constraints, and development guidelines for AI coding agents and contributors collaborating on `antigravity-powerline`.

---

## 1. Project Overview

`antigravity-powerline` is a fast, modular, and customizable status line tool for Google Antigravity CLI (`agy`) written in Rust.

### Data Flow
1. **Stdin Input**: Antigravity CLI passes a JSON payload via standard input on statusline refreshes (`src/input.rs`).
2. **Configuration**: The tool loads `~/.gemini/powerline.toml` (or a path provided via `--config`), falling back to built-in defaults (`src/config.rs`).
3. **PR Lookup (optional)**: If the `pr` segment is enabled, reads a disk-cached PR reference for the current branch; a stale or missing cache triggers a throttled, detached background refresh via `gh pr view` (`--fetch-pr-cache`, `src/github.rs`) so the hot path never blocks on a network call.
4. **Segment Assembly**: Iterates through enabled segments and formats each through the segment dispatch `match` in [src/main.rs](src/main.rs).
5. **Rendering**: The `renderer` applies the configured style and theme ANSI colors (`src/theme.rs`).
6. **Stdout Output**: Emits the single-line formatted status line to standard output.

### Sibling Reference Repository (`copilot-powerline`)
`antigravity-powerline` tracks the architecture, engineering conventions, and applicable features of [`xpepper/copilot-powerline`](https://github.com/xpepper/copilot-powerline) (typically cloned alongside this repo in `../copilot-powerline`). When porting features or patterns (such as `src/state.rs`, `src/github.rs`, `src/segments/pr.rs`, `src/cache_trend.rs`, `src/idle.rs`, `src/mode.rs`), consult `copilot-powerline` for the reference implementation and test coverage, and see [`TODO.md`](TODO.md) for the prioritized 5-phase roadmap.

---

## 2. Non-Obvious Locations

List the tree for the layout; one file per segment lives in `src/segments/`. Notes it does not show:

- `src/git.rs`: reads `.git/HEAD` directly, so the hot path never spawns `git`.
- In [`.github/workflows/ci.yml`](.github/workflows/ci.yml), a pull request that only changes Markdown skips the Rust jobs (`scripts/docs-only-change` decides). `CI ok` is the only required check on `main`: a new required job must go in its `needs`.
- Each `scripts/<name>[.sh]` has a `tests/test-<name>.sh`, run by a step in the CI `ShellCheck` job; a new script needs both.
- [`tests/repo_rules.rs`](tests/repo_rules.rs): enforces mechanical repository invariants (such as isolated `tempfile::tempdir()` usage in tests so parallel mutation workers never collide).
- [`CODING_STANDARDS.md`](CODING_STANDARDS.md): the judgement calls a reviewer applies to a diff.

---

## 3. Non-Negotiable Engineering Rules

1. **Sub-15ms Execution**:
   - This tool runs inside an interactive terminal status line loop.
   - Zero heavy dependencies, no async runtimes, and no inline network calls during status line execution. Any segment that needs an external command or network call (such as `pr` or `ci`) must read a disk cache and refresh it via a detached, throttled background process.
2. **Defensive Stdin Parsing**:
   - Antigravity CLI payloads may vary or be empty. Never assume a field is always present.
   - All input deserialization in `src/input.rs` must use `Option<T>` or `#[serde(default)]` and fall back gracefully without panicking.
3. **Zero Compiler Warnings**:
   - All code must compile cleanly with `cargo check` and `cargo test` without warnings.
4. **No Machine-Specific Paths**:
   - [`scripts/check-machine-paths`](scripts/check-machine-paths) enforces portable paths in tracked files.

---

## 4. Development & Verification Workflow

Always follow test-first development for behavioural changes.

Run the checks defined in [the CI workflow](.github/workflows/ci.yml), the source of truth for formatting, linting (including compiler warnings), shell scripts, packaging, tests, and release builds.

Also run this local smoke check, which CI does not cover:

```bash
echo '{"model":{"display_name":"Gemini 3.8 Flash"},"context_window":{"total_input_tokens":47427,"context_window_size":1048576}}' | cargo run --
```

Before pushing a PR branch, run `scripts/mutants-diff`; kill every MISSED mutant with a test, or exclude it in [`.cargo/mutants.toml`](.cargo/mutants.toml) with the reason.

On pull requests, CI's `Mutants` job runs the same script. It is advisory, not a required check: survivors are listed in the job summary and the `mutants.out` artifact.

---

## 5. Commits

Commits follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat`, `fix`, `perf`, `test`, `refactor`, `style`, `build`, `ci`, `docs`, `chore`).

---

## Agent skills

### Issue tracker

Issues live in GitHub Issues on `xpepper/antigravity-powerline`, managed with the `gh` CLI. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default triage roles, each label named after its role (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root, created lazily. See `docs/agents/domain.md`.
