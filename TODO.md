# Roadmap & TODO: `antigravity-powerline`

## Phase 1: Foundation & Security Hardening (Port from `copilot-powerline`)

- [ ] **1. Private Cross-Refresh State & Auto-Pruning (`src/state.rs`)**
  - [ ] Move state/cache files out of shared `/tmp` into user-private `dirs::cache_dir()/antigravity-powerline` (`0o700` dir, `0o600` atomic writes).
  - [ ] Implement per-session file hashing (`session_file(kind, session_id)`) and `update_snapshot`.
  - [ ] Implement daily stale-file pruning (`prune_if_due`, 30-day `MAX_AGE`, `PRUNABLE_KINDS`).
  - [ ] Add `git::find_repo_root` (`src/git.rs`) so git/PR/diff lookups resolve consistently from any subdirectory.

## Phase 2: Rich PR Segment (Port from `copilot-powerline`)

- [ ] **2. PR Review State, Unresolved Threads & Hardened Fetching (`src/github.rs`, `src/segments/pr.rs`)**
  - [ ] Add `output_with_timeout` (`GH_TIMEOUT = 10s`) and atomic lock-file spawn claiming (`try_claim_spawn`) so background `gh` calls never hang or mask stale cache data.
  - [ ] Validate PR URLs with `is_safe_github_url` before splicing into OSC 8 terminal escape sequences.
  - [ ] Extend `gh pr view` query to fetch `number,url,state,isDraft,reviewDecision,latestReviews`.
  - [ ] Add `pr_draft`, `pr_approved`, and `pr_changes_requested` colors to `Palette` (`src/theme.rs`) across all themes.
  - [ ] Tint the `#<number>` hyperlink by `ReviewState` (`Approved`, `ChangesRequested`, `Draft`, or default `pr` color).
  - [ ] Support `show_review_state` (spells out `· approved`, `· changes requested`, `· draft`; always on in `plain` theme).
  - [ ] Support `show_unresolved` (fetches unresolved review threads via `gh api graphql`, renders `· N unresolved` in alert color).

## Phase 3: New Segments Inspired by Copilot CLI Native Footer

- [ ] **3. Dedicated `ci` Segment (Branch & PR Check Status)**
  - [ ] Independent `ci` segment (`[ci]` in `powerline.toml`) that works both **with and without an open PR** (e.g., on `main` or a newly pushed branch).
  - [ ] Share the background GitHub worker/cache architecture so it never blocks the `<15ms` status-line hot path (piggyback on `statusCheckRollup` when a PR exists, or query latest branch workflow run via `gh run list --branch <branch> --limit 1`).
  - [ ] Render concise check status: passing (`✓`), failing (`✗` + optional failed count), or pending/running (`●`), with optional clickable OSC 8 link to the failing check or Actions run.

- [ ] **4. Git Diff Counter (`+added -removed`)**
  - [ ] Evaluate whether to expose as a standalone `git_diff` (or `diff`) segment, an option on `[git]` (`show_diff = true`), or both.
  - [ ] Compute uncommitted line additions (`+N` in green/up color) and deletions (`-M` in red/down color) via `git diff --numstat HEAD` (with a fast timeout or lightweight index stat check so clean working trees take `<1ms`).
  - [ ] Auto-hide when working tree has `+0 -0`.

## Phase 4: Session Intelligence & Ergonomics (Port from `copilot-powerline`)

- [ ] **5. Prompt Cache Trend (`↑`/`↓`) & Recent Miss Alert (`· miss 133k`)**
  - [ ] Port `src/cache_trend.rs` using `session_id` (falling back to `conversation_id`), `total_input_tokens`, and `cache_read_input_tokens`.
  - [ ] Show `↑` / `↓` on `cache` segment when the latest step's cache hit rate diverges from the session average.
  - [ ] Show `· miss <tokens>` for `miss_visible_seconds` (default 300s) after a step with `>= 50k` uncached input tokens.

- [ ] **6. Long-Break / Cold Cache Reminder (`cache_expiry` segment)**
  - [ ] Port `src/idle.rs` and `src/segments/cache_expiry.rs` (`Idle: 30m · cache likely cold`).
  - [ ] Track idle time per session across refreshes; hide below `min_tokens` (50k) or `idle_seconds` (1800s).

- [ ] **7. Compact / Full Display Mode Toggle (`--toggle`, `src/mode.rs`)**
  - [ ] Add `mode = "full" | "compact"` and `compact_segments` to `Config`.
  - [ ] Implement `antigravity-powerline --toggle` to flip between full and compact segment lists on the fly.

- [ ] **8. Token Alert Icon Placement Polish (`src/segments/tokens.rs`)**
  - [ ] Keep segment label/icon visible when `alert_threshold` is crossed (`Tokens: ⚠️ 108k/1.0M (10%)` instead of dropping `Tokens:`), matching `copilot-powerline`.

## Phase 5: Quality, Documentation & Release Automation

- [ ] **9. Domain Docs, Examples & Wiring Tests**
  - [ ] Add `GLOSSARY.md` and `CODING_STANDARDS.md` adapted for Antigravity CLI domain concepts.
  - [ ] Add annotated `examples/powerline.toml` with a sync test against `Config::default()`.
  - [ ] Add `tests/*_wiring.rs` integration tests and `scripts/check-machine-paths`.
