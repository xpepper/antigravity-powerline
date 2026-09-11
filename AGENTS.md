# Agent Guidelines for `antigravity-powerline`

Welcome! This document provides architecture overviews, design constraints, and development guidelines for AI coding agents and contributors collaborating on `antigravity-powerline`.

---

## 1. Project Overview

`antigravity-powerline` is a fast, modular, and customizable status line tool for Google Antigravity CLI written in Rust.

### Data Flow
1. **Stdin Input**: Antigravity CLI passes a JSON payload via standard input on statusline refreshes (containing `model`, `cwd`, `context_window`, `quota`, `session_id`, `artifact_count`, etc.).
2. **Configuration**: The tool loads `~/.gemini/powerline.toml` (or a path provided via `--config`), falling back to built-in defaults.
3. **Segment Assembly**: Iterates through enabled segments (`model`, `git`, `pr`, `tokens`, `quota`, `cache`, `total_tokens`, `artifacts`), formatting each.
4. **Rendering**: The `renderer` applies the configured style (`minimal`, `powerline`, `capsule`, `plain`) and theme ANSI colors (`github`, `nord`, `tokyo-night`, `colorblind`, `plain`).
5. **Stdout Output**: Emits the single-line formatted status line to standard output.

---

## 2. Non-Negotiable Engineering Rules

1. **Sub-15ms Execution**:
   - This tool runs inside an interactive terminal status line loop.
   - Zero heavy dependencies, no network calls, fast filesystem checks (e.g. lightweight git branch detection).
2. **Defensive Stdin Parsing**:
   - Antigravity CLI payloads may vary or be empty. Never assume a field is always present.
   - All input deserialization in `src/input.rs` must use `Option<T>` or `#[serde(default)]` and fall back gracefully without panicking.
3. **Zero Compiler Warnings**:
   - All code must compile cleanly with `cargo check` and `cargo test` without warnings.
4. **No Machine-Specific Paths**:
   - Never hardcode or leak absolute paths from the developer's local machine in documentation, code comments, or tests.

---

## 3. Development & Verification Workflow

Always follow test-first development for behavioural changes:

```bash
# 1. Check code formatting
cargo fmt --check

# 2. Run all unit tests
cargo test

# 3. Run linter
cargo clippy --all-targets -- -D warnings
```

---

## 4. Commit Conventions

Use Conventional Commits:
- `feat(<scope>): <description>` — New features or segment additions
- `fix(<scope>): <description>` — Bug fixes or edge-case handling
- `perf(<scope>): <description>` — Performance optimizations
- `test(<scope>): <description>` — Adding or updating unit tests
- `style: <description>` — Code formatting or stylistic improvements
- `ci: <description>` — CI/CD workflow configuration
- `docs: <description>` — Documentation or README updates
- `chore: <description>` — Maintenance, dependency bumps, or metadata changes
