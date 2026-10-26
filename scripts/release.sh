#!/usr/bin/env bash
# Automates the deterministic steps of RELEASING.md.
#
#   scripts/release.sh [--dry-run] prepare X.Y.Z   bump the version, run the release checks,
#                                                  push a branch and open the bump PR
#   scripts/release.sh [--dry-run] tag             after the bump PR is merged: tag main,
#                                                  push the tag, watch the release run,
#                                                  verify crates.io, append generated notes
#
# Merging the PR stays manual on purpose: it is the human review point.
set -euo pipefail

REPO="xpepper/antigravity-powerline"
CRATE="antigravity-powerline"

case "${DRY_RUN:-false}" in
  1|true|yes) DRY_RUN=true ;;
  *) DRY_RUN=false ;;
esac

args=()
for arg in "$@"; do
  case "$arg" in
    --dry-run|-n) DRY_RUN=true ;;
    *) args+=("$arg") ;;
  esac
done
if [ ${#args[@]} -gt 0 ]; then
  set -- "${args[@]}"
else
  set --
fi

die() { echo "error: $*" >&2; exit 1; }

require_clean_tree() {
  if [ -n "$(git status --porcelain)" ]; then
    if [ "$DRY_RUN" = true ]; then
      echo "warning: working tree is not clean (dry-run)" >&2
    else
      die "working tree is not clean"
    fi
  fi
}

require_main_branch() {
  if [ "$(git branch --show-current)" != "main" ]; then
    if [ "$DRY_RUN" = true ]; then
      echo "warning: not on main branch (dry-run)" >&2
    else
      die "run from the main branch"
    fi
  fi
}

crate_version() {
  sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1
}

prepare() {
  local version="${1:-}"
  [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "usage: $0 [--dry-run] prepare X.Y.Z"
  require_clean_tree
  require_main_branch
  if [ "$(crate_version)" = "$version" ]; then
    if [ "$DRY_RUN" = true ]; then
      echo "warning: Cargo.toml is already at $version (dry-run)" >&2
    else
      die "Cargo.toml is already at $version"
    fi
  fi
  if git rev-parse -q --verify "refs/tags/v$version" >/dev/null 2>&1; then
    if [ "$DRY_RUN" = true ]; then
      echo "warning: tag v$version already exists (dry-run)" >&2
    else
      die "tag v$version already exists"
    fi
  fi

  if [ "$DRY_RUN" = true ]; then
    echo "git pull --ff-only"
    echo "git checkout -b chore/release-$version"
    echo "VERSION=\"$version\" perl -0pi -e 's/^version = \".*\"/version = \"\$ENV{VERSION}\"/m' Cargo.toml"
    echo "cargo build --quiet"
    echo "cargo fmt --all --check"
    echo "cargo clippy --all-targets -- -D warnings"
    echo "cargo test"
    echo "cargo build --release"
    echo "dist plan >/dev/null"
    echo "git add Cargo.toml Cargo.lock"
    echo "git commit -m \"chore(release): bump version to $version\""
    echo "cargo publish --dry-run"
    echo "git push -u origin chore/release-$version"
    cat <<EOF
gh pr create --base main \\
  --title "chore(release): bump version to $version" \\
  --body "TL;DR: bump to $version. Release checks passed locally (fmt, clippy, test, release build, publish dry-run, dist plan).

Merge, then run \\\`scripts/release.sh tag\\\`."
EOF
    return 0
  fi

  git pull --ff-only
  git checkout -b "chore/release-$version"
  VERSION="$version" perl -0pi -e 's/^version = ".*"/version = "$ENV{VERSION}"/m' Cargo.toml
  cargo build --quiet # refreshes Cargo.lock

  cargo fmt --all --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  cargo build --release
  dist plan >/dev/null

  git add Cargo.toml Cargo.lock
  git commit -m "chore(release): bump version to $version"
  cargo publish --dry-run # needs the clean tree the commit just created

  git push -u origin "chore/release-$version"
  gh pr create --base main \
    --title "chore(release): bump version to $version" \
    --body "TL;DR: bump to $version. Release checks passed locally (fmt, clippy, test, release build, publish dry-run, dist plan).

Merge, then run \`scripts/release.sh tag\`."
}

tag() {
  require_clean_tree
  require_main_branch
  local version tag_name
  version="$(crate_version)"
  tag_name="v$version"
  if git rev-parse -q --verify "refs/tags/$tag_name" >/dev/null 2>&1; then
    if [ "$DRY_RUN" = true ]; then
      echo "warning: tag $tag_name already exists (dry-run)" >&2
    else
      die "tag $tag_name already exists"
    fi
  fi
  if ! git log -1 --format='%s' | grep -q "bump version to $version"; then
    if [ "$DRY_RUN" = true ]; then
      echo "warning: HEAD is not the version bump commit for $version (dry-run)" >&2
    else
      die "HEAD is not the version bump commit for $version"
    fi
  fi

  if [ "$DRY_RUN" = true ]; then
    echo "git pull --ff-only"
    echo "git tag -a $tag_name -m $tag_name"
    echo "git push origin $tag_name"
    echo "gh run list --workflow=release.yml --branch $tag_name --limit 1 --json databaseId --jq '.[0].databaseId // empty'"
    echo "gh run watch <run_id> --exit-status"
    echo "curl -fsS -H \"User-Agent: $CRATE-release-script\" https://crates.io/api/v1/crates/$CRATE/$version"
    echo "gh api repos/$REPO/releases/generate-notes -f tag_name=$tag_name --jq .body"
    echo "gh release view $tag_name --json body --jq .body"
    echo "gh release edit $tag_name --notes \"<body + notes>\""
    return 0
  fi

  git pull --ff-only
  git tag -a "$tag_name" -m "$tag_name"
  git push origin "$tag_name"

  local run_id=""
  for _ in $(seq 1 20); do
    run_id="$(gh run list --workflow=release.yml --branch "$tag_name" --limit 1 --json databaseId --jq '.[0].databaseId // empty')"
    [ -n "$run_id" ] && break
    sleep 3
  done
  [ -n "$run_id" ] || die "release run for $tag_name did not start"
  gh run watch "$run_id" --exit-status || die "release run failed: re-run the failed jobs (safe to repeat)"

  curl -fsS -H "User-Agent: $CRATE-release-script" \
    "https://crates.io/api/v1/crates/$CRATE/$version" >/dev/null ||
    die "$version is not on crates.io"
  echo "crates.io has $version"

  local notes body
  notes="$(gh api "repos/$REPO/releases/generate-notes" -f tag_name="$tag_name" --jq .body)"
  body="$(gh release view "$tag_name" --json body --jq .body)"
  gh release edit "$tag_name" --notes "$body

$notes"
  echo "Released $tag_name"
}

case "${1:-}" in
  prepare) shift; prepare "$@" ;;
  tag) tag ;;
  *) die "usage: $0 [--dry-run] prepare X.Y.Z | tag" ;;
esac
