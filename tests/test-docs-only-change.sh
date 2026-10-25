#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check_script="$root/scripts/docs-only-change"

assert_docs_only() {
  if ! printf '%s' "$1" | bash "$check_script"; then
    echo "FAIL: expected docs-only for: $1" >&2
    exit 1
  fi
}

assert_not_docs_only() {
  if printf '%s' "$1" | bash "$check_script"; then
    echo "FAIL: expected a non-docs change for: $1" >&2
    exit 1
  fi
}

assert_docs_only $'README.md\n'
assert_docs_only $'AGENTS.md\ndocs/agents/domain.md\n'
assert_docs_only 'CONTRIBUTING.md'

assert_not_docs_only $'src/main.rs\n'
assert_not_docs_only $'README.md\nsrc/main.rs\n'
assert_not_docs_only $'.github/workflows/ci.yml\n'
assert_not_docs_only $'notes.md.rs\n'
assert_not_docs_only $'examples/powerline.toml\n'

# An empty list means the diff could not be computed, so run every check.
assert_not_docs_only ''
assert_not_docs_only $'\n'

echo "PASS: docs-only change detection"
