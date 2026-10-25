#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
script="$root/scripts/mutants-diff"
work="$(cd "$(mktemp -d)" && pwd -P)"
trap 'rm -rf "$work"' EXIT

# A repo whose `main` has two commits on its origin, while the local
# `origin/main` is stale at the first, and a branch off the second that
# changes one Rust file.
repo="$work/repo"
git init -q --bare "$work/origin.git"
git init -q -b main "$repo"
git -C "$repo" config user.email test@example.com
git -C "$repo" config user.name test
git -C "$repo" remote add origin "$work/origin.git"
echo 'fn a() {}' > "$repo/lib.rs"
echo '/target' > "$repo/.gitignore"
git -C "$repo" add lib.rs .gitignore
git -C "$repo" commit -q -m base
git -C "$repo" push -q origin main
stale="$(git -C "$repo" rev-parse HEAD)"
echo 'fn m() {}' > "$repo/main.rs"
git -C "$repo" add main.rs
git -C "$repo" commit -q -m "newer main"
git -C "$repo" push -q origin main
git -C "$repo" update-ref refs/remotes/origin/main "$stale"
git -C "$repo" switch -q -c feature
echo 'fn b() {}' >> "$repo/lib.rs"
git -C "$repo" commit -q -am change

# A fake cargo that records its arguments and environment, then exits
# $FAKE_EXIT.
fake_cargo="$work/cargo"
cat > "$fake_cargo" <<EOF
#!/usr/bin/env bash
echo "\$*" > "$work/args"
echo "\$HOME" > "$work/home"
echo "\${RUSTUP_HOME:-}" > "$work/rustup_home"
echo "\${CARGO_HOME:-}" > "$work/cargo_home"
exit "\${FAKE_EXIT:-0}"
EOF
chmod +x "$fake_cargo"

run() {
  (cd "$repo" && env -u RUSTUP_HOME -u CARGO_HOME CARGO="$fake_cargo" bash "$script" "$@")
}

fail() {
  echo "FAIL: $1" >&2
  exit 1
}

# A clean branch runs cargo mutants on its diff against the fetched base.
run || fail "expected success on a clean branch"
args="$(cat "$work/args")"
diff_file="$repo/target/mutants-diff/pr.diff"
[ "$args" = "mutants --in-diff $diff_file --test-tool nextest --no-shuffle -j 4" ] ||
  fail "unexpected cargo arguments: $args"
grep -q '^+fn b() {}$' "$diff_file" || fail "diff lacks the branch's change"
grep -q 'fn m() {}' "$diff_file" && fail "diff includes main's own commit: the stale base was not fetched"
grep -q '^+fn a() {}$' "$diff_file" && fail "diff includes the base's own lines"
[ "$(cat "$work/home")" = "$repo/target/mutants-home" ] ||
  fail "HOME is not the throwaway home: $(cat "$work/home")"
[ "$(cat "$work/rustup_home")" = "$HOME/.rustup" ] ||
  fail "RUSTUP_HOME does not point at the real toolchains"
[ "$(cat "$work/cargo_home")" = "$HOME/.cargo" ] ||
  fail "CARGO_HOME does not point at the real cargo home"

# A local base is diffed as is, and MUTANTS_JOBS sets the job count.
MUTANTS_JOBS=2 run main ||
  fail "expected success with a local base"
grep -q -- '-j 2$' "$work/args" || fail "MUTANTS_JOBS was not used: $(cat "$work/args")"

# A surviving mutant (cargo mutants exits 2) fails the script.
rm -f "$work/args"
if FAKE_EXIT=2 run; then
  fail "expected failure when cargo mutants reports a missed mutant"
fi
[ -f "$work/args" ] || fail "cargo mutants did not run"

# Uncommitted work would not be in the diff, so the script refuses, for an
# untracked file as much as for an edit.
assert_refused() {
  rm -f "$work/args"
  if run 2> "$work/stderr"; then
    fail "expected refusal: $1"
  fi
  grep -q 'commit' "$work/stderr" || fail "refusal does not say to commit: $1"
  [ ! -f "$work/args" ] || fail "cargo mutants ran: $1"
}

echo 'fn c() {}' > "$repo/new.rs"
assert_refused "untracked file"
rm "$repo/new.rs"

echo 'fn c() {}' >> "$repo/lib.rs"
assert_refused "edited file"

echo "PASS: mutants-diff"
