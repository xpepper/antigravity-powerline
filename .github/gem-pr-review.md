# Review guidelines for antigravity-powerline

A Rust status-line tool for Google Antigravity CLI (`agy`). Every refresh is a fresh process, so the hot path must stay fast and quiet.

## Global

- The hot path stays under 15 ms: no network, no heavy dependencies, no async runtime. Slow work (a `gh` call) runs in a detached background worker that only writes a disk cache.
- Stdin payloads can be empty or change shape. Deserialize with `Option<T>` or `#[serde(default)]` and never panic.
- Disk caches written across refreshes (such as the PR cache in `src/github.rs`) must use atomic temp-file-and-rename writes so concurrent readers never observe partial state.
- No absolute machine paths in code, docs, comments or tests.

## Contracts

- New config fields need serde defaults, so existing `powerline.toml` files keep loading.

## Performance

- Only the main refresh path is latency-sensitive. A background worker may block on `gh`, but it needs a bounded runtime.
- `gh` output is a small JSON document from a trusted local binary, so buffering it is fine.

## Conventions

- A new segment follows the steps in `AGENTS.md`: a file in `src/segments/`, registered in `mod.rs`, a config struct with defaults, wired into the `main.rs` match.
- Behaviour changes come with a test that failed first.
