//! Small private files the status line keeps between invocations (PR lookups,
//! display mode, prompt-cache and idle snapshots). Every Refresh is a fresh
//! process, so any cross-Refresh memory has to live on disk.

use serde::{Serialize, de::DeserializeOwned};
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Kinds of per-Session or per-branch files that pruning may delete. Anything
/// else in the state directory (the `mode` override, the prune marker) is
/// never touched. `session_file` asserts its kind is listed here.
const PRUNABLE_KINDS: [&str; 3] = ["pr", "cache", "idle"];

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

/// 16-hex-digit hash of a Session id, so it can name a file or appear in a
/// log without exposing the id. Stable across Refreshes of one build, not
/// guaranteed across Rust versions (state files are short-lived caches).
#[allow(dead_code)]
pub fn session_hash(session_id: &str) -> String {
    let mut hasher = DefaultHasher::new();
    session_id.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Path of a per-Session state file, e.g. `cache_<hash>.json`. The Session id
/// is hashed so arbitrary ids can never escape the state directory.
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_session_file_is_private_and_per_session_and_kind() {
        let a = session_file("cache", "sess-a");
        assert!(a.starts_with(base_dir()));
        assert_eq!(a, session_file("cache", "sess-a"));
        assert_ne!(a, session_file("cache", "sess-b"));
        assert_ne!(a, session_file("idle", "sess-a"));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "unknown state kind")]
    fn test_session_file_rejects_a_kind_that_pruning_does_not_know() {
        session_file("brand_new", "sess-a");
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
    fn test_update_snapshot_loads_derives_and_persists_only_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache_0123456789abcdef.json");

        let first = update_snapshot(&path, |prev: Option<u64>| {
            assert_eq!(prev, None);
            42
        });
        assert_eq!(first, 42);
        assert_eq!(fs::read_to_string(&path).unwrap(), "42");

        let sentinel_mtime = SystemTime::now() - Duration::from_secs(3600);
        File::open(&path)
            .unwrap()
            .set_modified(sentinel_mtime)
            .unwrap();

        let unchanged = update_snapshot(&path, |prev: Option<u64>| {
            assert_eq!(prev, Some(42));
            42
        });
        assert_eq!(unchanged, 42);
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            sentinel_mtime
        );

        let updated = update_snapshot(&path, |prev: Option<u64>| prev.unwrap_or(0) + 8);
        assert_eq!(updated, 50);
        assert_eq!(fs::read_to_string(&path).unwrap(), "50");
    }
}
