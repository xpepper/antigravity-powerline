# antigravity-powerline

[![Crates.io](https://img.shields.io/crates/v/antigravity-powerline.svg)](https://crates.io/crates/antigravity-powerline)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A blazingly fast, modular, and customizable powerline status line for Google Antigravity CLI (`agy`), written in Rust. Inspired by [`copilot-powerline`](https://github.com/xpepper/copilot-powerline).

---

## Previews

```text
# Minimal Style (Plain Text)
[Gemini 3.8 Flash (High)]  │  (main)  │  PR #29  │  Tokens: 47k/1.0M (5%)  │  Quota: 5h: 96%  wk: 88%  │  Cache: 97%  │  Total: 52k

# Minimal Style (Nerd Icons)
󰚩 [Gemini 3.8 Flash (High)]  │  󰘬 (main)  │   #29  │  󰮚 47k/1.0M (5%)  │  󰔛 5h: 96%  wk: 88%  │  󰘸 97%  │  󰓅 52k

# Capsule Style (Nerd Icons)
󰚩 [Gemini 3.8 Flash (High)]  󰘬 (main)   #29  󰮚 47k/1.0M (5%)  󰔛 5h: 96%  wk: 88%  󰘸 97%  󰓅 52k 

# Capsule Style (Emoji Icons)
🤖 [Gemini 3.8 Flash (High)]  🌿 (main)  🔀 #29  🪙 47k/1.0M (5%)  ⏳ 5h: 96%  wk: 88%  ⚡ 97%  📊 52k 

# Context Alert Warning Threshold Triggered
[Gemini 3.8 Flash (High)]  │  (main)  │  ⚠️ 145k/1.0M (14%)  │  Quota: 5h: 96%  wk: 88%  │  Cache: 97%  │  Total: 195k
```

---

## Features

- ⚡️ **Sub-5ms Execution**: Ultra-lightweight compiled Rust binary; zero terminal lag or latency.
- 🤖 **Model Tracking**: Displays active model name and effort level (e.g., `Gemini 3.8 Flash (High)`).
- 🌿 **Instant Git Branch Detection**: Sub-millisecond direct repository `HEAD` resolution without process spawning overhead.
- 🔀 **GitHub Pull Request Detection**: Displays clickable terminal hyperlink to the active branch's pull request (e.g., `PR #29`) via `gh`, cached with background refresh. Stays off automatically when `gh` is unavailable.
- 🎯 **Context Window Monitoring**: Real-time context token usage vs model capacity, percentage used, and alert threshold warnings (`⚠️ `).
- ⏳ **Quota & Rate Limit Monitoring**: Displays remaining 5-hour and weekly quota fractions (e.g. `5h: 96%`, `wk: 88%`), color-coded by capacity.
- ⚡ **Prompt Cache Hit Rate**: Tracks prompt cache efficiency and cache hit percentage in real time.
- 📊 **Total Session Tokens**: Displays cumulative turn throughput across the entire interactive session.
- 📦 **Artifacts Counter**: Displays number of generated artifacts in the current workspace.
- 🎨 **Multiple Styles**: Choose from `minimal`, `powerline`, `capsule`, or `plain`.
- 🔣 **Icon Sets**: Full support for `nerd` font glyphs, `emoji`, or clean `plain` text labels.
- 🌈 **Themes**: Built-in support for `colorblind`, `github`, `nord`, `tokyo-night`, and `plain`.

---

## Quick Start

### 1. Build and Install

```bash
cargo install antigravity-powerline
```

Or install from source:

```bash
git clone https://github.com/xpepper/antigravity-powerline.git
cd antigravity-powerline
cargo install --path .
```

This installs `antigravity-powerline` to `~/.cargo/bin/antigravity-powerline`.

### 2. Generate Default Configuration

```bash
antigravity-powerline --init
```

This creates a default configuration file at `~/.gemini/powerline.toml`.

### 3. Connect to Google Antigravity CLI

Update `statusLine` in `~/.gemini/antigravity-cli/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "antigravity-powerline",
    "enabled": true
  }
}
```

*(If `~/.cargo/bin` is not in your default system `PATH`, specify `~/.cargo/bin/antigravity-powerline`)*.

---

## Status Line Legend & Segments

| Segment | Icon (`nerd`) | Icon (`emoji`) | Text (`plain`) | Example Value | Description |
|---|:---:|:---:|---|---|---|
| `model` | `󰚩` | `🤖` | `Model:` | `[Gemini 3.8 Flash (High)]` | Active model display name. |
| `git` | `󰘬` | `🌿` | `git:` | `(main)` | Active git branch name, resolved sub-millisecond. |
| `pr` | `` | `🔀` | `PR` | `PR #29` | Active GitHub pull request number with clickable OSC 8 hyperlink to PR. Hidden when no PR exists. |
| `tokens` | `󰮚` / `⚠️` | `🪙` / `⚠️` | `Tokens:` | `47k/1.0M (5%)` | Active context tokens vs model limit. Switches to `⚠️ ` when crossing alert threshold. |
| `quota` | `󰔛` | `⏳` | `Quota:` | `5h: 96%  wk: 88%` | Remaining 5-hour and weekly quotas, color-coded by capacity. |
| `cache` | `󰘸` | `⚡` | `Cache:` | `97%` | Prompt cache hit rate. Automatically hidden when zero. |
| `total_tokens` | `󰓅` | `📊` | `Total:` | `52k` | Total session token throughput. |
| `artifacts` | `󰏗` | `📦` | `Artifacts:` | `2` | Number of artifacts created in the current conversation. |

---

## Configuration (`~/.gemini/powerline.toml`)

`antigravity-powerline` is configured via `~/.gemini/powerline.toml`:

```toml
style = "minimal"      # Options: "minimal", "powerline", "capsule", "plain"
icon_set = "nerd"      # Options: "nerd", "emoji", "plain"
theme = "nord"         # Options: "nord", "colorblind", "github", "tokyo-night", "plain"
segments = [
    "model",
    "git",
    "pr",
    "tokens",
    "quota",
    "cache",
    "total_tokens",
    "artifacts",
]

[model]
enabled = true
brackets = true

[git]
enabled = true
parentheses = true

[pr]
enabled = true
hyperlinks = true
cache_ttl_seconds = 60

[tokens]
enabled = true
show_percentage = true
alert_threshold = 100000
alert_icon = "⚠️ "

[quota]
enabled = true
preferred_keys = [
    "gemini-5h",
    "gemini-weekly",
]

[cache]
enabled = true
hide_when_zero = true

[total_tokens]
enabled = true
hide_when_zero = true

[artifacts]
enabled = true
hide_when_zero = true
```

---

## CLI Options

```text
Usage: antigravity-powerline [OPTIONS]

Options:
  -c, --config <FILE>      Path to custom TOML configuration file
  -s, --style <STYLE>      Override display style (minimal, powerline, capsule, plain)
  -t, --theme <THEME>      Override color theme (colorblind, github, nord, tokyo-night, plain)
  -i, --icon-set <SET>     Override icon set (plain, nerd, emoji)
      --init               Generate a default ~/.gemini/powerline.toml configuration file
  -h, --help               Print help
  -V, --version            Print version
```

---

## License

MIT
