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

- [x] **2. PR Review State, Unresolved Threads & Hardened Fetching (`src/github.rs`, `src/segments/pr.rs`)**
  - *Reference*: `../copilot-powerline/src/github.rs`, `../copilot-powerline/src/segments/pr.rs`, `../copilot-powerline/src/theme.rs`, `../copilot-powerline/src/config.rs`, `../copilot-powerline/tests/pr_wiring.rs`
  - [x] Add `output_with_timeout` (`GH_TIMEOUT = 10s`) and atomic lock-file spawn claiming (`try_claim_spawn`) so background `gh` calls never hang or mask stale cache data.
  - [x] Validate PR URLs with `is_safe_github_url` before splicing into OSC 8 terminal escape sequences.
  - [x] Extend `gh pr view` query to fetch `number,url,state,isDraft,reviewDecision,latestReviews`.
  - [x] Add `pr_draft`, `pr_approved`, and `pr_changes_requested` colors to `Palette` (`src/theme.rs`) across all themes.
  - [x] Tint the `#<number>` hyperlink by `ReviewState` (`Approved`, `ChangesRequested`, `Draft`, or default `pr` color).
  - [x] Support `show_review_state` (spells out `· approved`, `· changes requested`, `· draft`; always on in `plain` theme).
  - [x] Support `show_unresolved` (fetches unresolved review threads via `gh api graphql` when `--show-unresolved` is passed to background worker, renders `· N unresolved` in alert color).
  - [x] Extract `Sandbox` into `tests/common/mod.rs` and add `tests/pr_wiring.rs` to cover `main()` PR worker wiring.

- [ ] **2a. Decide How Merged / Closed PRs Render (`src/github.rs`, `src/segments/pr.rs`)**
  - A merged or closed PR still renders as `PR #29` in the default `pr` colour; only its Review State is dropped (`parse_pr_view`). Decide whether to hide it, mark it (e.g. `· merged`), or keep it as is. `copilot-powerline` behaves the same today. Raised in PR #5 review.

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

- [x] **5. Prompt Cache Trend (`↑`/`↓`) & Recent Miss Alert (`· miss 169k`)**
  - *Reference*: `../copilot-powerline/src/state.rs`, `../copilot-powerline/src/cache_trend.rs`, `../copilot-powerline/src/segments/cache.rs`, `../copilot-powerline/tests/cache_wiring.rs`
  - [x] Add per-Session file hashing (`session_hash`, `session_file(kind, session_id)`) and `update_snapshot` to `src/state.rs` alongside their first caller.
  - [x] Add `src/cache_trend.rs`, adapted rather than ported: Antigravity's `current_usage` is per-Step (confirmed against local conversation `gen_metadata`), so the snapshot sums Session totals itself instead of diffing cumulative counters. Keyed by `conversation_id` (falling back to `session_id`, its documented compatibility copy); no snapshot while the id is empty.
  - [x] Show `↑` / `↓` on `cache` segment when the latest Step's cache hit rate diverges from the Session average by more than 5 points.
  - [x] Show `· miss <tokens>` for `miss_visible_seconds` (default 300s) after a Step with `>= 50k` uncached input tokens, even when `hide_when_zero` would hide a fully cold Step.
  - [x] Add `tests/cache_wiring.rs` to cover `main()` wiring for `cache` (`cache_expiry` is covered with item 6).
  - [x] Follow-up: capture one live status line payload to confirm `current_usage` mirrors the per-Step `gen_metadata` counts. Confirmed on 2026-10-26 (agy 1.4.7): all 9 Steps of a captured conversation matched `gen_metadata` exactly (input, output, cache read). `total_input_tokens` is the context window fill (`gen_metadata` 9.10.1), not a sum of Step inputs.
  - [x] Follow-up: two consecutive Steps with identical input, cache-read and output counts are deduplicated as one repeated Refresh, so the second is not counted and its Cache Miss does not renew. The live capture showed `total_output_tokens` is the running sum of Step outputs and `total_input_tokens` changed on every Step, so add both to the deduplication key. Also drop "timer" from the `src/cache_trend.rs` comments and `test_timer_refresh_keeps_the_arrow` (there is no timer, see Refresh in `GLOSSARY.md`). Raised in PR #6 review. Done: `Step` carries both totals, built by `Step::latest` from the whole `context_window`; snapshots written before the change reset once to a fresh baseline.
  - [ ] Follow-up (review judgement call, not done): the clock comes from `github::current_timestamp` (house style shared with `copilot-powerline`).

- [x] **6. Long-Break / Cold Cache Reminder (`cache_expiry` segment)**
  - *Reference*: `../copilot-powerline/src/idle.rs`, `../copilot-powerline/src/segments/cache_expiry.rs`
  - *Live capture (2026-10-26, agy 1.4.7)*: Antigravity CLI has no refresh timer (no Refresh during 327s idle), unlike Copilot CLI's `refreshInterval` that `idle.rs` relies on. One Refresh arrives when the user starts typing into an empty prompt (inferred from timing, not documented), so the reminder cannot count up on screen but shows as the user starts the next prompt. `session_id` was always empty, so key the idle snapshot by `conversation_id` like `cache_trend`; `total_output_tokens` grows on every Step and can serve as the "a Step happened" counter.
  - [x] Port `src/idle.rs` and `src/segments/cache_expiry.rs` (`Idle: 30m · cache likely cold`).
  - [x] Track idle time per session across refreshes; hide below `min_tokens` (50k) or `idle_seconds` (1800s).
  - *Adaptations*: the idle clock restarts when `cache_trend::Step::latest` changes (the same new-Step rule as `cache_trend`, both totals included) instead of on a cumulative token counter; keyed by `session_key()`, no snapshot without one or before the first Step; context size from `active_tokens()`; emoji `💤` since `⏳` is taken by `quota`; joins `default_segments()` after `cache`. Docs promise no live counter. Wiring covered by `tests/cache_expiry_wiring.rs`.
  - [ ] Follow-up (review judgement call, not done): `idle_seconds` names both the config threshold and the measured idle time (`idle::idle_seconds`, the `main` local, the render parameter), as in `copilot-powerline`; consider renaming the measured value (e.g. `idle_for`).
  - [ ] Follow-up (review judgement call, not done): the `cache_trend` and idle gates in `main` repeat the `session_key()` / `Step::latest` / `current_timestamp()` lookup; bind it once if a third caller appears.
  - [ ] Follow-up (review judgement call, not done): `Step` now has two users (`cache_trend`, `idle`); consider moving it out of `cache_trend`.

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

- [ ] **10. Build & Release Pipeline (multi-platform binaries, crates.io, Homebrew)**
  - *Reference*: `../copilot-powerline/dist-workspace.toml`, `../copilot-powerline/.github/workflows/release.yml`, `../copilot-powerline/.github/workflows/publish-crates.yml`, `[profile.dist]` in `../copilot-powerline/Cargo.toml`
  - [x] Set up `cargo-dist` (same version as `copilot-powerline`) with `shell` and `homebrew` installers, triggered by pushing a version tag.
  - [x] Build release binaries for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-gnu`, on non-deprecated `ubuntu-24.04` / `ubuntu-24.04-arm` runners, and attach them to a GitHub Release.
  - [x] Publish the Homebrew formula to the `xpepper/homebrew-tap` tap (needs a `HOMEBREW_TAP_TOKEN` repository secret).
  - [x] Publish to crates.io from a custom `publish-crates` job using trusted publishing (OIDC, `id-token: write`), not a long-lived token; `cargo publish --dry-run` already runs in CI.
  - [x] Document the release steps (tagging, secrets, crates.io trusted-publisher setup) for maintainers.
  - *Decisions (2026-10-26)*: first automated release is `0.3.0` (features since the hand-published `0.2.1`); versions stay chosen by hand, with a written SemVer policy in `RELEASING.md`; `v0.2.0` / `v0.2.1` tags backfilled on the commits published to crates.io. Two PRs: (1) dist pipeline and docs, (2) `scripts/release.sh` with `tests/test-release.sh`.
  - [x] Port `scripts/release.sh` (`prepare X.Y.Z`, `tag`, `--dry-run`) with its mocked test suite, run from the CI `ShellCheck` job.
  - [ ] Maintainer setup: `HOMEBREW_TAP_TOKEN` repository secret, crates.io trusted publisher (`xpepper` / `antigravity-powerline` / `release.yml`).
  - [ ] Cut `0.3.0` with the script and verify every channel (GitHub Release, installer, Homebrew, crates.io, `cargo binstall`).
