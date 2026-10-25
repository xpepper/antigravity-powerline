# Roadmap & TODO: `antigravity-powerline`

> **Handoff & Reference Context for Agents**:
> - **Sibling Repo**: [`xpepper/copilot-powerline`](https://github.com/xpepper/copilot-powerline) (available locally at `../copilot-powerline` when present) is the reference implementation for Phase 1, Phase 2, Phase 4, and Phase 5. Read its corresponding source files and tests before implementing each phase.
> - **Workflow & Branch Protection**: `main` is protected (`CI ok` required status check, conversation resolution required, admins enforced). Always work on a focused feature branch, follow test-first development, and run `scripts/mutants-diff` + `bash scripts/check-machine-paths` before pushing a PR.

## Phase 1: Foundation & Security Hardening (Port from `copilot-powerline`)

- [x] **1. Private Cross-Refresh State & Auto-Pruning (`src/state.rs`)**
  - *Reference*: `../copilot-powerline/src/state.rs`, `../copilot-powerline/src/git.rs`, and `../copilot-powerline/tests/prune_wiring.rs`
  - [x] Move state/cache files out of shared `/tmp` into user-private `dirs::cache_dir()/antigravity-powerline` (`0o700` dir, `0o600` atomic writes).
  - [x] Implement daily stale-file pruning (`prune_if_due`, 30-day `MAX_AGE`, `PRUNABLE_KINDS`).
  - [x] Add `git::find_repo_root` (`src/git.rs`) so git/PR/diff lookups resolve consistently from any subdirectory.
  - [x] Add `tests/prune_wiring.rs` to cover `main()` pruning and nested-subdirectory PR cache resolution.

## Phase 2: Rich PR Segment (Port from `copilot-powerline`)

- [ ] **2. PR Review State, Unresolved Threads & Hardened Fetching (`src/github.rs`, `src/segments/pr.rs`)**
  - *Reference*: `../copilot-powerline/src/github.rs`, `../copilot-powerline/src/segments/pr.rs`, `../copilot-powerline/src/theme.rs`, `../copilot-powerline/src/config.rs`, `../copilot-powerline/tests/pr_wiring.rs`
  - [ ] Add `output_with_timeout` (`GH_TIMEOUT = 10s`) and atomic lock-file spawn claiming (`try_claim_spawn`) so background `gh` calls never hang or mask stale cache data.
  - [ ] Validate PR URLs with `is_safe_github_url` before splicing into OSC 8 terminal escape sequences.
  - [ ] Extend `gh pr view` query to fetch `number,url,state,isDraft,reviewDecision,latestReviews`.
  - [ ] Add `pr_draft`, `pr_approved`, and `pr_changes_requested` colors to `Palette` (`src/theme.rs`) across all themes.
  - [ ] Tint the `#<number>` hyperlink by `ReviewState` (`Approved`, `ChangesRequested`, `Draft`, or default `pr` color).
  - [ ] Support `show_review_state` (spells out `· approved`, `· changes requested`, `· draft`; always on in `plain` theme).
  - [ ] Support `show_unresolved` (fetches unresolved review threads via `gh api graphql` when `--fetch-unresolved` is passed to background worker, renders `· N unresolved` in alert color).
  - [ ] Extract `Sandbox` into `tests/common/mod.rs` and add `tests/pr_wiring.rs` to cover `main()` PR worker wiring.

## Phase 3: New Segments Inspired by Copilot CLI Native Footer

- [ ] **3. Dedicated `ci` Segment (Branch & PR Check Status)**
  - *Design Decision*: Must be a **separate, independent segment** (`"ci"` in `segments`, `[ci]` in `powerline.toml`) so users can toggle/position it independently of `pr` and see CI status even on branches without an open PR (e.g. `main`).
  - [ ] Share the background GitHub worker/cache architecture (`src/github.rs`) so it never blocks the `<15ms` status-line hot path (piggyback on `statusCheckRollup` when a PR exists, or query latest branch workflow run via `gh run list --branch <branch> --limit 1 --json status,conclusion,url`).
  - [ ] Render concise check status: passing (`✓`), failing (`✗` + optional failed count), or pending/running (`●`), with optional clickable OSC 8 link to the failing check or Actions run.

- [ ] **4. Git Diff Counter (`+added -removed`)**
  - [ ] Expose uncommitted line additions (`+N` in green/up color) and deletions (`-M` in red/down color), inspired by the Copilot CLI native footer.
  - [ ] Evaluate whether to expose as a standalone `diff` segment, an option on `[git]` (`show_diff = true`), or both.
  - [ ] Keep execution strictly under `<15ms` (e.g. fast `git diff --numstat HEAD` with bounded timeout or cached against `.git/index` mtime).
  - [ ] Auto-hide when working tree has `+0 -0`.

## Phase 4: Session Intelligence & Ergonomics (Port from `copilot-powerline`)

- [ ] **5. Prompt Cache Trend (`↑`/`↓`) & Recent Miss Alert (`· miss 133k`)**
  - *Reference*: `../copilot-powerline/src/state.rs`, `../copilot-powerline/src/cache_trend.rs`, `../copilot-powerline/src/segments/cache.rs`, `../copilot-powerline/tests/cache_wiring.rs`
  - [ ] Add per-Session file hashing (`session_hash`, `session_file(kind, session_id)`) and `update_snapshot` to `src/state.rs` alongside their first caller.
  - [ ] Port `src/cache_trend.rs` using `session_id` (falling back to `conversation_id`), `total_input_tokens`, and `cache_read_input_tokens`.
  - [ ] Show `↑` / `↓` on `cache` segment when the latest step's cache hit rate diverges from the session average.
  - [ ] Show `· miss <tokens>` for `miss_visible_seconds` (default 300s) after a step with `>= 50k` uncached input tokens.
  - [ ] Add `tests/cache_wiring.rs` to cover `main()` wiring for `cache` and `cache_expiry`.

- [ ] **6. Long-Break / Cold Cache Reminder (`cache_expiry` segment)**
  - *Reference*: `../copilot-powerline/src/idle.rs`, `../copilot-powerline/src/segments/cache_expiry.rs`
  - [ ] Port `src/idle.rs` and `src/segments/cache_expiry.rs` (`Idle: 30m · cache likely cold`).
  - [ ] Track idle time per session across refreshes; hide below `min_tokens` (50k) or `idle_seconds` (1800s).

- [ ] **7. Compact / Full Display Mode Toggle (`--toggle`, `src/mode.rs`)**
  - *Reference*: `../copilot-powerline/src/mode.rs`
  - [ ] Add `mode = "full" | "compact"` and `compact_segments` to `Config`.
  - [ ] Implement `antigravity-powerline --toggle` to flip between full and compact segment lists on the fly.

- [ ] **8. Token Alert Icon Placement Polish (`src/segments/tokens.rs`)**
  - *Reference*: `../copilot-powerline/src/segments/tokens.rs`
  - [ ] Note: Context window threshold alert (`alert_threshold = 100_000`) already exists in `antigravity-powerline`, but currently replaces the `Tokens:` label/icon with `⚠️ `. Update it to keep the segment label/icon visible (`Tokens: ⚠️ 108k/1.0M (10%)`), matching `copilot-powerline`.

## Phase 5: Quality, Documentation & Release Automation

- [ ] **9. Domain Docs, Examples & Wiring Tests**
  - *Reference*: `../copilot-powerline/GLOSSARY.md`, `../copilot-powerline/examples/powerline.toml`, `../copilot-powerline/tests/init_wiring.rs`
  - [ ] Complete `GLOSSARY.md` with any remaining Antigravity CLI domain concepts (`GLOSSARY.md`, `CODING_STANDARDS.md`, and `tests/repo_rules.rs` are in place).
  - [ ] Add annotated `examples/powerline.toml` with a sync test against `Config::default()`.
  - [ ] Add `tests/init_wiring.rs` integration test.
