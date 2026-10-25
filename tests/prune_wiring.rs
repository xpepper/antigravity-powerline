//! Runs the built binary against a throwaway `HOME` to check that a status
//! line Refresh prunes stale state files and resolves PR cache entries from a
//! nested repository subdirectory.
#![cfg(unix)]

mod common;

use common::Sandbox;
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DAY: Duration = Duration::from_secs(24 * 60 * 60);

fn touch(dir: &Path, name: &str, age: Duration) -> PathBuf {
    let path = dir.join(name);
    File::create(&path)
        .unwrap()
        .set_modified(SystemTime::now() - age)
        .unwrap();
    path
}

#[test]
fn test_refresh_prunes_stale_state_files_and_keeps_the_rest() {
    let sandbox = Sandbox::new();
    let state = sandbox.state_dir();
    fs::create_dir_all(&state).unwrap();
    let stale = touch(&state, "pr_0123456789abcdef.json", 40 * DAY);
    let recent = touch(&state, "pr_fedcba9876543210.json", DAY);
    let mode = touch(&state, "mode", 40 * DAY);

    sandbox.refresh(r#"{"session_id":"wiring"}"#, &[]);

    assert!(!stale.exists());
    assert!(recent.exists() && mode.exists());
    assert!(state.join("last_prune").exists());
}

#[test]
fn test_refresh_scans_at_most_once_a_day() {
    let sandbox = Sandbox::new();
    let state = sandbox.state_dir();
    fs::create_dir_all(&state).unwrap();

    sandbox.refresh(r#"{"session_id":"wiring"}"#, &[]);
    assert!(state.join("last_prune").exists());
    let stale = touch(&state, "cache_0123456789abcdef.json", 40 * DAY);
    sandbox.refresh(r#"{"session_id":"wiring"}"#, &[]);

    assert!(stale.exists());
}

#[test]
fn test_refresh_resolves_pr_cache_by_repo_root_from_nested_subdirectory() {
    let sandbox = Sandbox::new();
    sandbox.write_config("theme = \"plain\"\nsegments = [\"git\", \"pr\"]\n");

    let repo_root = sandbox.home().join("repo");
    let nested = repo_root.join("crates").join("sub");
    fs::create_dir_all(repo_root.join(".git")).unwrap();
    fs::create_dir_all(&nested).unwrap();
    fs::write(repo_root.join(".git/HEAD"), "ref: refs/heads/feat-root\n").unwrap();

    let bin = sandbox.home().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let fake_gh = bin.join("gh");
    fs::write(&fake_gh, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&fake_gh, fs::Permissions::from_mode(0o755)).unwrap();

    let mut hasher = DefaultHasher::new();
    repo_root.to_string_lossy().hash(&mut hasher);
    "feat-root".hash(&mut hasher);
    let cache_name = format!("pr_{:016x}.json", hasher.finish());

    let state = sandbox.state_dir();
    fs::create_dir_all(&state).unwrap();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    fs::write(
        state.join(cache_name),
        format!(
            r#"{{"timestamp":{now},"pr":{{"number":42,"url":"https://github.com/xpepper/antigravity-powerline/pull/42"}}}}"#
        ),
    )
    .unwrap();

    let payload = format!(r#"{{"cwd":"{}"}}"#, nested.display());
    let line = sandbox.refresh_with_env(
        &payload,
        &["--style", "plain", "--icon-set", "plain"],
        &[("PATH", &bin)],
    );

    assert_eq!(line, "(feat-root)  |  PR #42");
}
