# antigravity-powerline

A fast, modular, and customizable powerline status line for Google Antigravity CLI.

## Overview

`antigravity-powerline` intercepts the JSON statusline payload passed by Antigravity CLI and renders a beautiful, information-dense powerline or minimal status line.

### Segments Supported
- **Model**: Current model name (e.g., `Gemini 3.8 Flash (High)`).
- **Git**: Active branch name.
- **Tokens / Context**: Active tokens vs window limit, percentage used, and alert thresholds.
- **Quota**: 5h and weekly quota usage/remaining limits for Gemini and 3P models.
- **Cache**: Prompt cache hit rate or cached token count.
- **Total Tokens**: Cumulative session throughput.
- **Artifacts**: Count of generated artifacts.

## Installation

```bash
cargo install --path .
```

## Quick Start & Configuration

Add or edit in `~/.gemini/antigravity-cli/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "antigravity-powerline",
    "enabled": true
  }
}
```

To customize appearance and segments, generate the default config:

```bash
antigravity-powerline --init
```

This creates `~/.gemini/powerline.toml`.

## License

MIT
