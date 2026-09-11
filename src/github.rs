use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub number: u64,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrCacheEntry {
    pub timestamp: u64,
    pub pr: Option<PullRequestInfo>,
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn is_gh_available() -> bool {
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            let candidate = path.join("gh");
            #[cfg(windows)]
            let candidate = path.join("gh.exe");

            if candidate.is_file() {
                return true;
            }
        }
    }
    false
}

pub fn get_cache_file_path(repo_dir: &Path, branch: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    repo_dir.to_string_lossy().hash(&mut hasher);
    branch.hash(&mut hasher);
    let hash = hasher.finish();

    std::env::temp_dir()
        .join("antigravity-powerline")
        .join(format!("pr_{:016x}.json", hash))
}

pub fn read_cache_entry(cache_path: &Path) -> Option<PrCacheEntry> {
    let content = fs::read_to_string(cache_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_cache_entry(cache_path: &Path, entry: &PrCacheEntry) -> std::io::Result<()> {
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string(entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let tmp_path = cache_path.with_extension(format!("tmp.{}", std::process::id()));
    {
        let mut file = File::create(&tmp_path)?;
        file.write_all(json.as_bytes())?;
        file.flush()?;
    }

    fs::rename(&tmp_path, cache_path)
}

pub fn is_cache_fresh(entry: &PrCacheEntry, ttl_seconds: u64, now: u64) -> bool {
    now.saturating_sub(entry.timestamp) < ttl_seconds
}

pub fn should_throttle_spawn(cache_path: &Path, now: u64, throttle_seconds: u64) -> bool {
    if let Ok(metadata) = fs::metadata(cache_path)
        && let Ok(modified) = metadata.modified()
        && let Ok(dur) = modified.duration_since(UNIX_EPOCH)
    {
        return now.saturating_sub(dur.as_secs()) < throttle_seconds;
    }
    false
}

#[derive(Deserialize)]
struct GhPrViewPayload {
    number: u64,
    url: String,
}

pub fn fetch_and_write_pr_cache(repo_dir: &Path, cache_path: &Path) {
    let now = current_timestamp();

    let output = Command::new("gh")
        .args(["pr", "view", "--json", "number,url"])
        .current_dir(repo_dir)
        .output();

    let entry = match output {
        Ok(out) if out.status.success() => {
            if let Ok(payload) = serde_json::from_slice::<GhPrViewPayload>(&out.stdout) {
                PrCacheEntry {
                    timestamp: now,
                    pr: Some(PullRequestInfo {
                        number: payload.number,
                        url: payload.url,
                    }),
                }
            } else {
                PrCacheEntry {
                    timestamp: now,
                    pr: None,
                }
            }
        }
        _ => PrCacheEntry {
            timestamp: now,
            pr: None,
        },
    };

    let _ = write_cache_entry(cache_path, &entry);
}

pub fn spawn_background_fetch(repo_dir: &Path, cache_path: &Path) {
    if let Ok(exe) = std::env::current_exe() {
        let _ = Command::new(exe)
            .arg("--fetch-pr-cache")
            .arg(cache_path)
            .arg("--repo-dir")
            .arg(repo_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}

pub fn get_pr_info(
    repo_dir: &Path,
    branch: &str,
    ttl_seconds: u64,
    gh_available: bool,
) -> Option<PullRequestInfo> {
    if !gh_available {
        return None;
    }

    let cache_path = get_cache_file_path(repo_dir, branch);
    let now = current_timestamp();
    let throttle_seconds = 15;

    if let Some(entry) = read_cache_entry(&cache_path) {
        if is_cache_fresh(&entry, ttl_seconds, now) {
            return entry.pr;
        }

        // Cache is stale. If not throttled, spawn a background refresh
        if !should_throttle_spawn(&cache_path, now, throttle_seconds) {
            // Touch/update placeholder timestamp to throttle
            let _ = write_cache_entry(
                &cache_path,
                &PrCacheEntry {
                    timestamp: now,
                    pr: entry.pr.clone(),
                },
            );
            spawn_background_fetch(repo_dir, &cache_path);
        }

        entry.pr
    } else {
        // No cache exists yet. If not throttled, mark and spawn
        if !should_throttle_spawn(&cache_path, now, throttle_seconds) {
            let _ = write_cache_entry(
                &cache_path,
                &PrCacheEntry {
                    timestamp: now,
                    pr: None,
                },
            );
            spawn_background_fetch(repo_dir, &cache_path);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_roundtrip() {
        let temp_file = std::env::temp_dir().join(format!(
            "test_cache_{}_{}.json",
            std::process::id(),
            current_timestamp()
        ));

        let entry = PrCacheEntry {
            timestamp: 1000,
            pr: Some(PullRequestInfo {
                number: 29,
                url: "https://github.com/org/repo/pull/29".to_string(),
            }),
        };

        write_cache_entry(&temp_file, &entry).unwrap();
        let loaded = read_cache_entry(&temp_file).unwrap();
        assert_eq!(loaded, entry);

        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_cache_entry_none_roundtrip() {
        let temp_file = std::env::temp_dir().join(format!(
            "test_cache_none_{}_{}.json",
            std::process::id(),
            current_timestamp()
        ));

        let entry = PrCacheEntry {
            timestamp: 12345,
            pr: None,
        };

        write_cache_entry(&temp_file, &entry).unwrap();
        let loaded = read_cache_entry(&temp_file).unwrap();
        assert_eq!(loaded, entry);

        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_cache_freshness() {
        let entry = PrCacheEntry {
            timestamp: 100,
            pr: None,
        };
        assert!(is_cache_fresh(&entry, 60, 150));
        assert!(!is_cache_fresh(&entry, 60, 160));
        assert!(!is_cache_fresh(&entry, 60, 200));
    }

    #[test]
    fn test_gh_available_check() {
        // In the current environment, gh was confirmed to exist
        assert!(is_gh_available());
    }

    #[test]
    fn test_get_cache_file_path_deterministic() {
        let path1 = get_cache_file_path(Path::new("/some/repo"), "feature-x");
        let path2 = get_cache_file_path(Path::new("/some/repo"), "feature-x");
        let path3 = get_cache_file_path(Path::new("/some/repo"), "feature-y");
        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
    }

    #[test]
    fn test_get_pr_info_when_gh_not_available() {
        let res = get_pr_info(Path::new("/some/repo"), "main", 60, false);
        assert!(res.is_none());
    }
}
