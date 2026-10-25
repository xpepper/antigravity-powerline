//! Small private files the status line keeps between invocations (PR lookups,
//! display mode, prompt-cache and idle snapshots). Every Refresh is a fresh
//! process, so any cross-Refresh memory has to live on disk.

use serde::{Serialize, de::DeserializeOwned};
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Kinds of per-Session or per-branch files that pruning may delete. Anything
/// else in the state directory (the `mode` override, the prune marker) is
/// never touched. `session_file` asserts its kind is listed here.
const PRUNABLE_KINDS: [&str; 3] = ["pr", "cache", "idle"];

/// A Session idle longer than this loses its snapshots; resuming it only
/// re-creates the baselines (idle time restarts from zero).
const MAX_AGE: Duration = Duration::from_secs(30 * 24 * 60 * 60);

/// Minimum time between two directory scans.
const PRUNE_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

const PRUNE_MARKER: &str = "last_prune";

/// Base directory for state files. Prefers the user-private cache directory
/// (e.g. `~/.cache` on Linux, `~/Library/Caches` on macOS) over the shared
/// system temp dir, since the temp dir's predictable, world-writable path
/// would let another local user pre-create or race the state files.
pub fn base_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("antigravity-powerline")
}

pub fn ensure_private_dir(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(dir, fs::Permissions::from_mode(0o700));
    }
    Ok(())
}

/// Writes `contents` to `path` with owner-only permissions, via a temp file
/// and rename so concurrent readers never see a partial write.
pub fn write_private_atomic(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        ensure_private_dir(parent)?;
    }

    let tmp_path = path.with_extension(format!("tmp.{}", std::process::id()));
    {
        let mut file = File::create(&tmp_path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.set_permissions(fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(contents)?;
        file.flush()?;
    }

    fs::rename(&tmp_path, path)
}

/// 16-hex-digit hash of a Session id, so it can name a file without exposing
/// the id. Stable across Refreshes of one build, not guaranteed across Rust
/// versions (state files are short-lived caches).
fn session_hash(session_id: &str) -> String {
    let mut hasher = DefaultHasher::new();
    session_id.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Path of a per-Session state file, e.g. `cache_<hash>.json`. The Session id
/// is hashed so arbitrary ids can never escape the state directory.
pub fn session_file(kind: &str, session_id: &str) -> PathBuf {
    debug_assert!(
        PRUNABLE_KINDS.contains(&kind),
        "unknown state kind {kind:?}: add it to PRUNABLE_KINDS or it is never pruned"
    );
    base_dir().join(format!("{kind}_{}.json", session_hash(session_id)))
}

/// Loads the snapshot at `path` (if readable), lets `assess` derive the next
/// one, and persists it only when it changed. Write failures are ignored: the
/// status line must render even when the cache dir is read-only.
pub fn update_snapshot<T>(path: &Path, assess: impl FnOnce(Option<T>) -> T) -> T
where
    T: Serialize + DeserializeOwned + PartialEq + Copy,
{
    let prev = fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok());
    let next = assess(prev);

    if prev != Some(next)
        && let Ok(json) = serde_json::to_string(&next)
    {
        let _ = write_private_atomic(path, json.as_bytes());
    }
    next
}

/// True only for names this tool generates: `<kind>_<16 lowercase hex>` plus
/// `.json`, `.lock` or a leftover `.tmp.<pid>`. A bare prefix is not enough,
/// so unrelated files that happen to share a kind's name are never deleted.
fn is_prunable_name(name: &str) -> bool {
    PRUNABLE_KINDS.iter().any(|kind| {
        name.strip_prefix(kind)
            .and_then(|rest| rest.strip_prefix('_'))
            .and_then(|rest| rest.split_at_checked(16))
            .is_some_and(|(hash, suffix)| {
                hash.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
                    && (matches!(suffix, ".json" | ".lock")
                        || suffix.strip_prefix(".tmp.").is_some_and(|pid| {
                            !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit())
                        }))
            })
    })
}

/// Deletes per-Session state files (and their `.lock` / leftover `.tmp.<pid>`
/// siblings) last modified more than `MAX_AGE` ago. Errors are ignored: a
/// file that cannot be removed is simply tried again on the next scan, and a
/// racing writer at worst loses one baseline.
fn prune_stale(dir: &Path, now: SystemTime) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.file_name().to_str().is_some_and(is_prunable_name) {
            continue;
        }
        let is_stale = entry
            .metadata()
            .ok()
            .filter(|metadata| metadata.is_file())
            .and_then(|metadata| metadata.modified().ok())
            .is_some_and(|modified| now.duration_since(modified).is_ok_and(|age| age > MAX_AGE));
        if is_stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Prunes `dir` about once per `PRUNE_INTERVAL`, tracked by the mtime of a
/// marker file, so the common case costs a single `stat`. Refreshes that
/// overlap at the moment the marker goes stale may each scan once; the scan
/// is idempotent, so that is cheaper than a lock needing stale-lock handling.
/// Only ever call it with the state directory.
pub fn prune_if_due(dir: &Path, now: SystemTime) {
    let marker = dir.join(PRUNE_MARKER);
    let recently_pruned = fs::metadata(&marker)
        .and_then(|metadata| metadata.modified())
        .is_ok_and(|modified| {
            now.duration_since(modified)
                .is_ok_and(|age| age < PRUNE_INTERVAL)
        });
    if recently_pruned {
        return;
    }

    // Stamp before scanning, and skip the scan when stamping fails, so an
    // unwritable directory is not rescanned on every Refresh.
    if File::create(&marker)
        .and_then(|file| file.set_modified(now))
        .is_ok()
    {
        prune_stale(dir, now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: Duration = Duration::from_secs(24 * 60 * 60);

    fn touch(dir: &Path, name: &str, age: Duration, now: SystemTime) -> PathBuf {
        let path = dir.join(name);
        let file = File::create(&path).unwrap();
        file.set_modified(now - age).unwrap();
        path
    }

    #[test]
    fn test_base_dir_ends_with_antigravity_powerline() {
        assert!(base_dir().ends_with("antigravity-powerline"));
    }

    #[test]
    fn test_session_file_is_private_and_per_session_and_kind() {
        let a = session_file("cache", "sess-a");
        assert!(a.starts_with(base_dir()));
        assert_eq!(a, session_file("cache", "sess-a"));
        assert_ne!(a, session_file("cache", "sess-b"));
        assert_ne!(a, session_file("idle", "sess-a"));
    }

    #[test]
    fn test_session_file_name_is_prunable() {
        let path = session_file("cache", "../../etc/passwd");
        let name = path.file_name().unwrap().to_str().unwrap();
        assert!(is_prunable_name(name), "{name} would never be pruned");
        assert_eq!(path.parent(), Some(base_dir().as_path()));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "unknown state kind")]
    fn test_session_file_rejects_a_kind_that_pruning_does_not_know() {
        session_file("brand_new", "sess-a");
    }

    #[test]
    fn test_update_snapshot_hands_the_persisted_value_to_the_next_refresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache_0123456789abcdef.json");

        let first = update_snapshot(&path, |prev: Option<u64>| {
            assert_eq!(prev, None);
            7
        });
        let second = update_snapshot(&path, |prev: Option<u64>| prev.unwrap() + 1);

        assert_eq!((first, second), (7, 8));
    }

    #[test]
    fn test_update_snapshot_treats_an_unreadable_snapshot_as_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache_0123456789abcdef.json");
        fs::write(&path, "not json").unwrap();

        let next = update_snapshot(&path, |prev: Option<u64>| prev.unwrap_or(42));

        assert_eq!(next, 42);
    }

    #[test]
    fn test_update_snapshot_still_returns_when_the_write_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir
            .path()
            .join("missing")
            .join("cache_0123456789abcdef.json");
        fs::write(dir.path().join("missing"), "a file, not a dir").unwrap();

        assert_eq!(update_snapshot(&path, |_: Option<u64>| 5), 5);
    }

    #[test]
    fn test_write_private_atomic_creates_private_dir_and_file() {
        let dir = tempfile::tempdir().unwrap();
        let nested_dir = dir.path().join("private_state");
        let file_path = nested_dir.join("entry.json");

        write_private_atomic(&file_path, b"{\"ok\":true}").unwrap();
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "{\"ok\":true}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir_mode = fs::metadata(&nested_dir).unwrap().permissions().mode() & 0o777;
            let file_mode = fs::metadata(&file_path).unwrap().permissions().mode() & 0o777;
            assert_eq!(dir_mode, 0o700);
            assert_eq!(file_mode, 0o600);
        }
    }

    #[test]
    fn test_prune_if_due_removes_only_old_per_session_and_pr_files() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        let old = 31 * DAY;

        let generated_pr = crate::github::get_cache_file_path(Path::new("repo"), "branch")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let removed: Vec<PathBuf> = [
            "pr_0123456789abcdef.json",
            "cache_0123456789abcdef.json",
            "idle_0123456789abcdef.json",
            "pr_0123456789abcdef.lock",
            "cache_0123456789abcdef.tmp.4242",
            generated_pr.as_str(),
        ]
        .iter()
        .map(|name| touch(dir.path(), name, old, now))
        .collect();
        let exact_max_age = touch(dir.path(), "pr_1111111111111111.json", 30 * DAY, now);
        let recent = touch(dir.path(), "cache_fedcba9876543210.json", 29 * DAY, now);
        let mode = touch(dir.path(), "mode", old, now);
        let unrelated = touch(dir.path(), "notes.txt", old, now);
        let unseparated = touch(dir.path(), "cachet.json", old, now);

        prune_if_due(dir.path(), now);

        assert!(removed.iter().all(|path| !path.exists()));
        assert!(
            exact_max_age.exists()
                && recent.exists()
                && mode.exists()
                && unrelated.exists()
                && unseparated.exists()
        );
    }

    #[test]
    fn test_prune_if_due_keeps_old_files_with_an_allowed_prefix_but_another_shape() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();

        let kept: Vec<PathBuf> = [
            "pr_notes.txt",
            "cache_backup",
            "idle_0123456789abcdef.json.bak",
            "pr_0123456789abcdef",
            "pr_0123456789abcdef.txt",
            "cache_0123.json",
            "cache_0123456789ABCDEF.json",
            "idle_0123456789abcdef0.json",
            "pr_0123456789abcdef.tmp.",
            "pr_0123456789abcdef.tmp.pid",
            "cache_0123456789abcdé.json",
        ]
        .iter()
        .map(|name| touch(dir.path(), name, 31 * DAY, now))
        .collect();

        prune_if_due(dir.path(), now);

        for path in &kept {
            assert!(path.exists(), "{} was pruned", path.display());
        }
    }

    #[test]
    fn test_prune_if_due_keeps_files_dated_in_the_future() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        let path = dir.path().join("pr_0123456789abcdef.json");
        File::create(&path)
            .unwrap()
            .set_modified(now + 40 * DAY)
            .unwrap();

        prune_if_due(dir.path(), now);

        assert!(path.exists());
    }

    #[test]
    fn test_prune_if_due_skips_directories_and_tolerates_a_missing_dir() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        let subdir = dir.path().join("pr_0123456789abcdef.json");
        fs::create_dir(&subdir).unwrap();
        File::open(&subdir)
            .unwrap()
            .set_modified(now - 40 * DAY)
            .unwrap();

        prune_if_due(dir.path(), now);
        prune_if_due(&dir.path().join("missing"), now);

        assert!(subdir.is_dir());
    }

    #[test]
    fn test_prune_if_due_prunes_once_then_waits_for_the_interval() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        let first = touch(dir.path(), "pr_aaaaaaaaaaaaaaaa.json", 31 * DAY, now);

        prune_if_due(dir.path(), now);
        assert!(!first.exists());
        assert!(dir.path().join("last_prune").exists());

        let second = touch(dir.path(), "pr_bbbbbbbbbbbbbbbb.json", 31 * DAY, now);
        prune_if_due(dir.path(), now + DAY / 2);
        assert!(second.exists());

        prune_if_due(dir.path(), now + DAY);
        assert!(!second.exists());
    }

    #[test]
    fn test_prune_if_due_treats_a_future_marker_as_due_and_resets_it() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        touch(dir.path(), "last_prune", Duration::ZERO, now + 10 * DAY);
        let old = touch(dir.path(), "cache_aaaaaaaaaaaaaaaa.json", 31 * DAY, now);

        prune_if_due(dir.path(), now);
        assert!(!old.exists());

        let next = touch(dir.path(), "cache_bbbbbbbbbbbbbbbb.json", 31 * DAY, now);
        prune_if_due(dir.path(), now + DAY / 2);
        assert!(next.exists());
    }

    #[test]
    fn test_prune_if_due_skips_the_scan_when_the_marker_cannot_be_written() {
        let dir = tempfile::tempdir().unwrap();
        let now = SystemTime::now();
        fs::create_dir(dir.path().join("last_prune")).unwrap();
        let old = touch(dir.path(), "pr_aaaaaaaaaaaaaaaa.json", 31 * DAY, now);

        prune_if_due(dir.path(), now);
        prune_if_due(&dir.path().join("missing"), now);

        assert!(old.exists());
    }
}
