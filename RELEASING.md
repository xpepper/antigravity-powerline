# Releasing antigravity-powerline

`antigravity-powerline` is released to three channels from the same version:

- crates.io, for `cargo install antigravity-powerline`
- GitHub Releases with prebuilt binaries, a shell installer, and checksums, built by [dist](https://github.com/axodotdev/cargo-dist) (`.github/workflows/release.yml`, configured in `dist-workspace.toml`)
- Homebrew, through the [xpepper/homebrew-tap](https://github.com/xpepper/homebrew-tap) tap

Prebuilt targets are macOS and Linux on arm64 and x86_64.

On each release the workflow pushes an updated formula to the tap. This needs a `HOMEBREW_TAP_TOKEN` repository secret: a token with write access to the tap repository's contents. Do not advertise a target as supported unless CI verifies it.

The workflow then publishes the crate to crates.io (`.github/workflows/publish-crates.yml`, a dist custom publish job). It runs only after the GitHub Release exists, because a crates.io version cannot be deleted or re-uploaded. It authenticates with [crates.io trusted publishing](https://crates.io/docs/trusted-publishing), so there is no token secret. The one-time setup is in the crate's settings on crates.io, under Trusted Publishing, with repository owner `xpepper`, repository `antigravity-powerline`, and workflow `release.yml`. crates.io checks the calling workflow, not `publish-crates.yml`.

## Choose the version

Versions follow [Semantic Versioning](https://semver.org/). The crate is a binary, so its public interface is what users script and configure against:

- the command-line flags shown by `--help` (the hidden background-worker flags are internal);
- the keys and accepted values of `powerline.toml`: segment names, styles, themes, icon sets, and per-segment options;
- the default configuration path (`~/.gemini/powerline.toml`) and the binary name;
- the prebuilt targets and install channels listed above.

The rendered status line is not an interface: its wording, colors, and layout may change in any release.

Pick the bump from the changes since the last tag:

| Change | Before 1.0 | From 1.0 |
|---|---|---|
| Breaking: removes or renames a flag, config key, value, or segment; changes what an existing key means; makes a config that used to load fail; drops a target or channel | minor | major |
| Feature: a new segment, option, flag, theme, style, or target | minor | minor |
| Fix, performance, documentation, or internal change | patch | patch |

Before 1.0 a breaking change shares the minor bump with features, so call it out under a **Breaking** heading in the release notes, with the migration step.

A version is never reused: the tag is `v` plus the `Cargo.toml` version, and crates.io refuses a version that was published before. Tags `v0.2.0` and `v0.2.1` mark the versions published by hand before this process existed.

## Prepare a release

1. Start from an up-to-date `main` branch with the required CI checks passing.
2. Update the package version in `Cargo.toml` and any user-facing version references.
3. Review the README installation instructions and compatibility notes.
4. Run the release checks:

   ```bash
   cargo fmt --all --check
   cargo clippy --all-targets -- -D warnings
   cargo test --verbose
   cargo build --release --verbose
   cargo publish --dry-run
   dist plan
   ```

   `dist plan` needs the dist version that `cargo-dist-version` in `dist-workspace.toml` pins. If you changed `dist-workspace.toml` or upgraded dist, run `dist generate` and commit the regenerated workflow; the release workflow fails when it is out of date.

5. Open and merge the version-bump pull request.

## Tag

From the merged release commit on `main`, replace the example version and push an annotated tag. Do not run `cargo publish` by hand: the workflow does it.

```bash
VERSION=0.3.0
git tag -a "v$VERSION" -m "v$VERSION"
git push origin "v$VERSION"
```

Pushing the tag starts the Release workflow, which builds every target, creates the GitHub Release with the binaries, `antigravity-powerline-installer.sh`, and checksums, then publishes the Homebrew formula and the crate. Do not create the release by hand with `gh release create`: the workflow creates it.

If the crates.io job fails, fix the cause and re-run the failed jobs. The job skips a version that is already on crates.io, so a re-run is safe.

When the workflow has finished, append GitHub's generated notes to the release body:

```bash
NOTES=$(gh api "repos/xpepper/antigravity-powerline/releases/generate-notes" -f tag_name="v$VERSION" --jq .body)
BODY=$(gh release view "v$VERSION" --json body --jq .body)
gh release edit "v$VERSION" --notes "$BODY

$NOTES"
```

## After publishing

Confirm that the crates.io version and GitHub tag agree, and that every channel installs it:

```bash
cargo search antigravity-powerline --limit 1
cargo binstall antigravity-powerline --no-confirm
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/xpepper/antigravity-powerline/releases/latest/download/antigravity-powerline-installer.sh | sh
antigravity-powerline --version
brew upgrade xpepper/tap/antigravity-powerline || brew install xpepper/tap/antigravity-powerline
```

Then update any launch or distribution material to use the published version and the installation commands in the README.
