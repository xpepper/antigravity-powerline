#!/usr/bin/env bash
# Tests for scripts/release.sh using mock binaries for git, gh, cargo, dist, curl.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
release_script="$root/scripts/release.sh"

test_dir="$(mktemp -d)"
bin_dir="$test_dir/bin"
work_dir="$test_dir/repo"
mkdir -p "$bin_dir" "$work_dir"

error_file="$test_dir/error.log"
output_file="$test_dir/output.log"
git_log="$test_dir/git.log"
gh_log="$test_dir/gh.log"
cargo_log="$test_dir/cargo.log"
dist_log="$test_dir/dist.log"
curl_log="$test_dir/curl.log"

trap 'rm -rf "$test_dir"' EXIT

# Mock git
cat >"$bin_dir/git" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${MOCK_GIT_LOG}"
case "${1:-}" in
  status)
    if [ "${MOCK_GIT_DIRTY:-0}" = "1" ]; then
      echo "M Cargo.toml"
    fi
    exit 0
    ;;
  branch)
    echo "${MOCK_GIT_BRANCH:-main}"
    exit 0
    ;;
  pull)
    exit "${MOCK_GIT_PULL_EXIT:-0}"
    ;;
  rev-parse)
    if [ "${MOCK_GIT_TAG_EXISTS:-0}" = "1" ]; then
      exit 0
    else
      exit 1
    fi
    ;;
  log)
    echo "${MOCK_GIT_HEAD_MSG:-chore(release): bump version to 0.8.0}"
    exit 0
    ;;
  checkout|add|commit|push|tag)
    exit "${MOCK_GIT_EXIT:-0}"
    ;;
  *)
    exit 0
    ;;
esac
EOF
chmod +x "$bin_dir/git"

# Mock gh
cat >"$bin_dir/gh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${MOCK_GH_LOG}"
case "${1:-}" in
  pr)
    exit "${MOCK_GH_PR_EXIT:-0}"
    ;;
  run)
    case "${2:-}" in
      list)
        if [ -n "${MOCK_GH_RUN_ID:-}" ]; then
          echo "$MOCK_GH_RUN_ID"
        fi
        exit 0
        ;;
      watch)
        exit "${MOCK_GH_WATCH_EXIT:-0}"
        ;;
    esac
    ;;
  api)
    echo "${MOCK_GH_API_NOTES:-Whats Changed notes}"
    exit 0
    ;;
  release)
    case "${2:-}" in
      view)
        echo "${MOCK_GH_RELEASE_BODY:-Existing release body}"
        exit 0
        ;;
      edit)
        exit "${MOCK_GH_RELEASE_EDIT_EXIT:-0}"
        ;;
    esac
    ;;
esac
exit 0
EOF
chmod +x "$bin_dir/gh"

# Mock cargo
cat >"$bin_dir/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${MOCK_CARGO_LOG}"
exit "${MOCK_CARGO_EXIT:-0}"
EOF
chmod +x "$bin_dir/cargo"

# Mock dist
cat >"$bin_dir/dist" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${MOCK_DIST_LOG}"
exit "${MOCK_DIST_EXIT:-0}"
EOF
chmod +x "$bin_dir/dist"

# Mock curl
cat >"$bin_dir/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"${MOCK_CURL_LOG}"
exit "${MOCK_CURL_EXIT:-0}"
EOF
chmod +x "$bin_dir/curl"

# Mock sleep (instant return to avoid waiting 60s in polling tests)
cat >"$bin_dir/sleep" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
chmod +x "$bin_dir/sleep"

reset_env() {
  export PATH="$bin_dir:$PATH"
  export MOCK_GIT_LOG="$git_log"
  export MOCK_GH_LOG="$gh_log"
  export MOCK_CARGO_LOG="$cargo_log"
  export MOCK_DIST_LOG="$dist_log"
  export MOCK_CURL_LOG="$curl_log"

  export MOCK_GIT_DIRTY=0
  export MOCK_GIT_BRANCH="main"
  export MOCK_GIT_PULL_EXIT=0
  export MOCK_GIT_TAG_EXISTS=0
  export MOCK_GIT_HEAD_MSG="chore(release): bump version to 0.8.0"
  export MOCK_GIT_EXIT=0

  export MOCK_GH_PR_EXIT=0
  export MOCK_GH_RUN_ID="12345"
  export MOCK_GH_WATCH_EXIT=0
  export MOCK_GH_API_NOTES="What's Changed notes"
  export MOCK_GH_RELEASE_BODY="Existing release body"
  export MOCK_GH_RELEASE_EDIT_EXIT=0

  export MOCK_CARGO_EXIT=0
  export MOCK_DIST_EXIT=0
  export MOCK_CURL_EXIT=0

  : >"$git_log"
  : >"$gh_log"
  : >"$cargo_log"
  : >"$dist_log"
  : >"$curl_log"
  : >"$error_file"
  : >"$output_file"

  cat >"$work_dir/Cargo.toml" <<'EOF'
[package]
name = "antigravity-powerline"
version = "0.7.0"
EOF
  : >"$work_dir/Cargo.lock"
}

# --- Test: Invalid subcommand ---
reset_env
if (cd "$work_dir" && "$release_script" unknown_subcmd >"$output_file" 2>"$error_file"); then
  echo "Expected unknown subcommand to fail" >&2
  exit 1
fi
grep -Fq "usage: " "$error_file"

# --- Test prepare: usage / invalid version ---
reset_env
if (cd "$work_dir" && "$release_script" prepare not-a-version >"$output_file" 2>"$error_file"); then
  echo "Expected invalid version format to fail" >&2
  exit 1
fi
grep -Fq "usage: " "$error_file"

# --- Test prepare: dirty working tree ---
reset_env
export MOCK_GIT_DIRTY=1
if (cd "$work_dir" && "$release_script" prepare 0.8.0 >"$output_file" 2>"$error_file"); then
  echo "Expected dirty working tree in prepare to fail" >&2
  exit 1
fi
grep -Fq "working tree is not clean" "$error_file"

# --- Test prepare: wrong branch ---
reset_env
export MOCK_GIT_BRANCH="feat/something"
if (cd "$work_dir" && "$release_script" prepare 0.8.0 >"$output_file" 2>"$error_file"); then
  echo "Expected non-main branch in prepare to fail" >&2
  exit 1
fi
grep -Fq "run from the main branch" "$error_file"

# --- Test prepare: version already at target ---
reset_env
if (cd "$work_dir" && "$release_script" prepare 0.7.0 >"$output_file" 2>"$error_file"); then
  echo "Expected already-at-version in prepare to fail" >&2
  exit 1
fi
grep -Fq "Cargo.toml is already at 0.7.0" "$error_file"

# --- Test prepare: tag already exists ---
reset_env
export MOCK_GIT_TAG_EXISTS=1
if (cd "$work_dir" && "$release_script" prepare 0.8.0 >"$output_file" 2>"$error_file"); then
  echo "Expected existing tag in prepare to fail" >&2
  exit 1
fi
grep -Fq "tag v0.8.0 already exists" "$error_file"

# --- Test prepare: happy path ---
reset_env
(cd "$work_dir" && "$release_script" prepare 0.8.0 >"$output_file" 2>"$error_file")

grep -Fq 'version = "0.8.0"' "$work_dir/Cargo.toml"
grep -Fq "pull --ff-only" "$git_log"
grep -Fq "checkout -b chore/release-0.8.0" "$git_log"
grep -Fq "add Cargo.toml Cargo.lock" "$git_log"
grep -Fq "commit -m chore(release): bump version to 0.8.0" "$git_log"
grep -Fq "push -u origin chore/release-0.8.0" "$git_log"
grep -Fq "build --quiet" "$cargo_log"
grep -Fq "fmt --all --check" "$cargo_log"
grep -Fq "clippy --all-targets -- -D warnings" "$cargo_log"
grep -Fq "test" "$cargo_log"
grep -Fq "build --release" "$cargo_log"
grep -Fq "publish --dry-run" "$cargo_log"
grep -Fq "plan" "$dist_log"
grep -Fq "pr create --base main" "$gh_log"

# --- Test tag: dirty working tree ---
reset_env
export MOCK_GIT_DIRTY=1
if (cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file"); then
  echo "Expected dirty working tree in tag to fail" >&2
  exit 1
fi
grep -Fq "working tree is not clean" "$error_file"

# --- Test tag: wrong branch ---
reset_env
export MOCK_GIT_BRANCH="feat/something"
if (cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file"); then
  echo "Expected non-main branch in tag to fail" >&2
  exit 1
fi
grep -Fq "run from the main branch" "$error_file"

# --- Test tag: tag already exists ---
reset_env
export MOCK_GIT_TAG_EXISTS=1
if (cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file"); then
  echo "Expected existing tag in tag to fail" >&2
  exit 1
fi
grep -Fq "tag v0.7.0 already exists" "$error_file"

# --- Test tag: wrong commit message ---
reset_env
export MOCK_GIT_HEAD_MSG="feat: unrelated change"
if (cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file"); then
  echo "Expected wrong commit message in tag to fail" >&2
  exit 1
fi
grep -Fq "HEAD is not the version bump commit for 0.7.0" "$error_file"

# --- Test tag: workflow run timeout ---
reset_env
export MOCK_GIT_HEAD_MSG="chore(release): bump version to 0.7.0"
export MOCK_GH_RUN_ID=""
if (cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file"); then
  echo "Expected missing run id to fail" >&2
  exit 1
fi
grep -Fq "release run for v0.7.0 did not start" "$error_file"

# --- Test tag: workflow watch failure ---
reset_env
export MOCK_GIT_HEAD_MSG="chore(release): bump version to 0.7.0"
export MOCK_GH_WATCH_EXIT=1
if (cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file"); then
  echo "Expected watch failure to fail" >&2
  exit 1
fi
grep -Fq "release run failed" "$error_file"

# --- Test tag: crates.io verification failure ---
reset_env
export MOCK_GIT_HEAD_MSG="chore(release): bump version to 0.7.0"
export MOCK_CURL_EXIT=1
if (cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file"); then
  echo "Expected curl crates.io failure to fail" >&2
  exit 1
fi
grep -Fq "0.7.0 is not on crates.io" "$error_file"

# --- Test tag: happy path ---
reset_env
export MOCK_GIT_HEAD_MSG="chore(release): bump version to 0.7.0"
(cd "$work_dir" && "$release_script" tag >"$output_file" 2>"$error_file")

grep -Fq "pull --ff-only" "$git_log"
grep -Fq "tag -a v0.7.0 -m v0.7.0" "$git_log"
grep -Fq "push origin v0.7.0" "$git_log"
grep -Fq "run list --workflow=release.yml --branch v0.7.0" "$gh_log"
grep -Fq "run watch 12345 --exit-status" "$gh_log"
grep -Fq "https://crates.io/api/v1/crates/antigravity-powerline/0.7.0" "$curl_log"
grep -Fq "api repos/xpepper/antigravity-powerline/releases/generate-notes -f tag_name=v0.7.0" "$gh_log"
grep -Fq "release view v0.7.0" "$gh_log"
grep -Fq "release edit v0.7.0 --notes Existing release body" "$gh_log"
grep -Fq "Released v0.7.0" "$output_file"

# --- Test prepare: --dry-run prints planned commands and leaves state untouched ---
reset_env
(cd "$work_dir" && "$release_script" --dry-run prepare 0.8.0 >"$output_file" 2>"$error_file")
grep -Fq "git pull --ff-only" "$output_file"
grep -Fq "git checkout -b chore/release-0.8.0" "$output_file"
grep -Fq 'VERSION="0.8.0" perl -0pi -e' "$output_file"
grep -Fq "cargo build --quiet" "$output_file"
grep -Fq "cargo fmt --all --check" "$output_file"
grep -Fq "cargo clippy --all-targets -- -D warnings" "$output_file"
grep -Fq "cargo test" "$output_file"
grep -Fq "cargo build --release" "$output_file"
grep -Fq "dist plan >/dev/null" "$output_file"
grep -Fq "git add Cargo.toml Cargo.lock" "$output_file"
grep -Fq 'git commit -m "chore(release): bump version to 0.8.0"' "$output_file"
grep -Fq "cargo publish --dry-run" "$output_file"
grep -Fq "git push -u origin chore/release-0.8.0" "$output_file"
grep -Fq "gh pr create --base main" "$output_file"
grep -Fq 'version = "0.7.0"' "$work_dir/Cargo.toml"
if grep -Fq "checkout" "$git_log"; then echo "Unexpected checkout in git log" >&2; exit 1; fi
if grep -Fq "commit" "$git_log"; then echo "Unexpected commit in git log" >&2; exit 1; fi
if grep -Fq "push" "$git_log"; then echo "Unexpected push in git log" >&2; exit 1; fi
if grep -Fq "pr" "$gh_log"; then echo "Unexpected pr in gh log" >&2; exit 1; fi

# --- Test prepare: --dry-run flag variations (-n, trailing, etc.) ---
reset_env
(cd "$work_dir" && "$release_script" prepare 0.8.0 --dry-run >"$output_file" 2>"$error_file")
grep -Fq "git checkout -b chore/release-0.8.0" "$output_file"

reset_env
(cd "$work_dir" && "$release_script" -n prepare 0.8.0 >"$output_file" 2>"$error_file")
grep -Fq "git checkout -b chore/release-0.8.0" "$output_file"

# --- Test prepare: DRY_RUN=1 environment variable ---
reset_env
(cd "$work_dir" && DRY_RUN=1 "$release_script" prepare 0.8.0 >"$output_file" 2>"$error_file")
grep -Fq "git checkout -b chore/release-0.8.0" "$output_file"
if grep -Fq "checkout" "$git_log"; then echo "Unexpected checkout in git log" >&2; exit 1; fi

# --- Test prepare: --dry-run warns on dirty tree and non-main branch without failing ---
reset_env
export MOCK_GIT_DIRTY=1
export MOCK_GIT_BRANCH="feat/something"
(cd "$work_dir" && "$release_script" --dry-run prepare 0.8.0 >"$output_file" 2>"$error_file")
grep -Fq "warning: working tree is not clean (dry-run)" "$error_file"
grep -Fq "warning: not on main branch (dry-run)" "$error_file"
grep -Fq "git checkout -b chore/release-0.8.0" "$output_file"

# --- Test tag: --dry-run prints planned commands and does not tag or push ---
reset_env
export MOCK_GIT_HEAD_MSG="chore(release): bump version to 0.7.0"
(cd "$work_dir" && "$release_script" --dry-run tag >"$output_file" 2>"$error_file")
grep -Fq "git pull --ff-only" "$output_file"
grep -Fq "git tag -a v0.7.0 -m v0.7.0" "$output_file"
grep -Fq "git push origin v0.7.0" "$output_file"
grep -Fq "gh run list" "$output_file"
grep -Fq "gh run watch" "$output_file"
grep -Fq "https://crates.io/api/v1/crates/antigravity-powerline/0.7.0" "$output_file"
grep -Fq "gh api repos/xpepper/antigravity-powerline/releases/generate-notes" "$output_file"
grep -Fq "gh release view v0.7.0" "$output_file"
grep -Fq "gh release edit v0.7.0" "$output_file"
if grep -Fq "tag -a" "$git_log"; then echo "Unexpected tag in git log" >&2; exit 1; fi
if grep -Fq "push origin" "$git_log"; then echo "Unexpected push in git log" >&2; exit 1; fi
if grep -Fq "release edit" "$gh_log"; then echo "Unexpected release edit in gh log" >&2; exit 1; fi
if grep -Fq "antigravity-powerline" "$curl_log"; then echo "Unexpected curl in curl log" >&2; exit 1; fi

# --- Test tag: --dry-run warns when HEAD is not bump commit without failing ---
reset_env
export MOCK_GIT_HEAD_MSG="feat: unrelated"
(cd "$work_dir" && "$release_script" --dry-run tag >"$output_file" 2>"$error_file")
grep -Fq "warning: HEAD is not the version bump commit for 0.7.0 (dry-run)" "$error_file"
grep -Fq "git tag -a v0.7.0 -m v0.7.0" "$output_file"

echo "All release script tests passed!"
