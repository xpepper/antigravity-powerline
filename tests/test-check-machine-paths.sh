#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
check_script="$root/scripts/check-machine-paths"
test_dir="$(mktemp -d)"
trap 'rm -rf "$test_dir"' EXIT

mkdir "$test_dir/repo"
git -C "$test_dir/repo" init --quiet
cd "$test_dir/repo"

assert_rejected() {
  if bash "$check_script" >output.log 2>&1; then
    echo "FAIL: expected a machine-path violation in $1" >&2
    exit 1
  fi
  if ! grep -Fq "$1" output.log; then
    echo "FAIL: missing diagnostic: $1" >&2
    cat output.log >&2
    exit 1
  fi
}

assert_accepted() {
  if ! bash "$check_script" >output.log 2>&1; then
    echo "FAIL: expected no machine-path violations" >&2
    cat output.log >&2
    exit 1
  fi
}

assert_accepted

for directory in Users home; do
  printf '# Example: /%s/developer/project\n' "$directory" >tracked.md
  git add tracked.md
  assert_rejected "tracked.md:1:"
done

printf 'Use relative paths or the home-directory helper.\n' >tracked.md
assert_accepted

printf '/%s/developer/project\n' home >untracked.md
assert_accepted

printf '/%s/developer/project\n' Users >AGENTS.md
git add AGENTS.md
assert_rejected "AGENTS.md:1:"
printf 'No machine paths here.\n' >AGENTS.md
assert_accepted

mkdir -p nested
printf '/%s/developer/project\n' Users >nested/AGENTS.md
git add nested/AGENTS.md
assert_rejected "nested/AGENTS.md:1:"
printf 'No machine paths here.\n' >nested/AGENTS.md

cd nested
assert_accepted
printf '/%s/developer/project\n' Users >../tracked.md
assert_rejected "tracked.md:1:"

mkdir "$test_dir/outside"
cd "$test_dir/outside"
assert_rejected "not a git repository"

echo "PASS: machine-path checks across all tracked files"
