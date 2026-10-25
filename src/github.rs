use crate::config::PrConfig;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// The verdict a pull request's reviews add up to (see `GLOSSARY.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewState {
    Draft,
    Approved,
    ChangesRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub number: u64,
    pub url: String,
    /// Optional so cache files written before review support still parse.
    #[serde(default)]
    pub review_state: Option<ReviewState>,
    /// None when unknown: not fetched, the query failed, or the PR is not open.
    #[serde(default)]
    pub unresolved_threads: Option<UnresolvedThreads>,
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

    crate::state::base_dir().join(format!("pr_{:016x}.json", hash))
}

pub fn read_cache_entry(cache_path: &Path) -> Option<PrCacheEntry> {
    let content = fs::read_to_string(cache_path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn write_cache_entry(cache_path: &Path, entry: &PrCacheEntry) -> std::io::Result<()> {
    let json = serde_json::to_string(entry)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    crate::state::write_private_atomic(cache_path, json.as_bytes())
}

/// How long a background `gh` call may run. Shorter than the refresh
/// throttle, so a hung call is killed before the next worker can start.
pub const GH_TIMEOUT: Duration = Duration::from_secs(10);

/// Runs `command` and returns its stdout, or `None` when it cannot start or
/// is still running after `timeout` (it is killed). stderr is discarded.
///
/// Without a deadline, a `gh` call stuck on the network would hold the
/// refresh claim until it goes stale, and the next worker would pile on.
pub fn output_with_timeout(command: &mut Command, timeout: Duration) -> Option<Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut pipe = child.stdout.take()?;
    // Drained on its own thread so a large response cannot fill the pipe and
    // stall the child while this thread polls.
    let reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        buf
    });

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) if reader.is_finished() => break status,
            Ok(_) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(25));
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
        }
    };
    Some(Output {
        status,
        stdout: reader.join().ok()?,
        stderr: Vec::new(),
    })
}

pub fn is_cache_fresh(entry: &PrCacheEntry, ttl_seconds: u64, now: u64) -> bool {
    now.saturating_sub(entry.timestamp) < ttl_seconds
}

pub fn lock_file_path(cache_path: &Path) -> PathBuf {
    cache_path.with_extension("lock")
}

/// Returns true if a background refresh claim for `cache_path` was taken
/// recently (within `throttle_seconds`), based on the claim lock file's
/// mtime. The lock file is a separate marker from the cache entry itself, so
/// throttling never touches (or misrepresents the freshness of) cached data.
pub fn should_throttle_spawn(cache_path: &Path, now: u64, throttle_seconds: u64) -> bool {
    is_lock_fresh(&lock_file_path(cache_path), now, throttle_seconds)
}

/// Atomically claims the right to spawn a background refresh for
/// `cache_path`. Returns true if this call won the claim, false if another
/// process already holds a live claim (or won a concurrent race for a new
/// one). The claim never reads or writes the cache entry, so a losing (or
/// failing) refresh attempt can never mark stale cached data as fresh.
pub fn try_claim_spawn(cache_path: &Path, now: u64, throttle_seconds: u64) -> bool {
    if should_throttle_spawn(cache_path, now, throttle_seconds) {
        return false;
    }

    let lock_path = lock_file_path(cache_path);
    if let Some(parent) = lock_path.parent() {
        let _ = crate::state::ensure_private_dir(parent);
    }

    if create_lock(&lock_path) {
        return true;
    }

    // A lock exists and looked stale (e.g. left behind by a crashed worker).
    // Only the holder of a short-lived reclaim guard may delete it, and it
    // re-checks staleness under the guard, so a lock freshly created by a
    // concurrent claimer is never removed.
    let guard_path = lock_path.with_extension("lock.reclaim");
    if !create_lock(&guard_path) {
        if !is_lock_fresh(&guard_path, now, throttle_seconds) {
            // A crashed reclaimer left its guard behind; clear it so a
            // later refresh can reclaim.
            let _ = fs::remove_file(&guard_path);
        }
        return false;
    }
    let won = !is_lock_fresh(&lock_path, now, throttle_seconds) && {
        let _ = fs::remove_file(&lock_path);
        create_lock(&lock_path)
    };
    let _ = fs::remove_file(&guard_path);
    won
}

fn is_lock_fresh(lock_path: &Path, now: u64, throttle_seconds: u64) -> bool {
    fs::metadata(lock_path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .is_some_and(|dur| now.saturating_sub(dur.as_secs()) < throttle_seconds)
}

/// `create_new` fails if another process's creation won, which is what
/// arbitrates the race.
fn create_lock(lock_path: &Path) -> bool {
    match File::options().write(true).create_new(true).open(lock_path) {
        Ok(_file) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(lock_path, fs::Permissions::from_mode(0o600));
            }
            true
        }
        Err(_) => false,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhPrViewPayload {
    number: u64,
    url: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    is_draft: bool,
    // Empty when the repo requires no approving review, hence the fallback
    // to `latest_reviews`.
    #[serde(default)]
    review_decision: Option<String>,
    #[serde(default)]
    latest_reviews: Vec<GhReview>,
}

#[derive(Deserialize)]
struct GhReview {
    #[serde(default)]
    state: Option<String>,
}

/// What the worker learns from one `gh pr view` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrView {
    pub pr: PullRequestInfo,
    pub open: bool,
}

/// Parses the stdout of `gh pr view --json number,url,state,isDraft,
/// reviewDecision,latestReviews`.
pub fn parse_pr_view(stdout: &[u8]) -> Option<PrView> {
    let payload: GhPrViewPayload = serde_json::from_slice(stdout).ok()?;
    // Without a `state` (an older `gh`), the PR counts as not open: review
    // info is hidden rather than shown for a PR that may be merged.
    let open = payload.state.as_deref() == Some("OPEN");
    let review_state = if open { review_state(&payload) } else { None };

    Some(PrView {
        open,
        pr: PullRequestInfo {
            number: payload.number,
            url: payload.url,
            review_state,
            unresolved_threads: None,
        },
    })
}

/// An open PR's Review State, first match wins: draft, then GitHub's own
/// review decision, then the latest review of each reviewer.
fn review_state(payload: &GhPrViewPayload) -> Option<ReviewState> {
    if payload.is_draft {
        return Some(ReviewState::Draft);
    }
    match payload.review_decision.as_deref() {
        Some("APPROVED") => return Some(ReviewState::Approved),
        Some("CHANGES_REQUESTED") => return Some(ReviewState::ChangesRequested),
        Some("REVIEW_REQUIRED") => return None,
        _ => {}
    }
    let has_review = |wanted: &str| {
        payload
            .latest_reviews
            .iter()
            .any(|r| r.state.as_deref() == Some(wanted))
    };
    if has_review("CHANGES_REQUESTED") {
        Some(ReviewState::ChangesRequested)
    } else if has_review("APPROVED") {
        Some(ReviewState::Approved)
    } else {
        None
    }
}

/// How many review threads the GraphQL query fetches; past that the count
/// shows as a lower bound (`3+`) instead of paginating.
const REVIEW_THREADS_PAGE: u32 = 100;

const REVIEW_THREADS_QUERY: &str = r"
query($owner: String!, $repo: String!, $number: Int!, $first: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $number) {
      reviewThreads(first: $first) {
        pageInfo { hasNextPage }
        nodes { isResolved }
      }
    }
  }
}";

/// The pull request's Unresolved Threads. `more` is set when the first
/// page of threads did not hold them all, so `count` is a lower bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedThreads {
    pub count: u32,
    pub more: bool,
}

/// Parses the stdout of the `reviewThreads` GraphQL query. Outdated threads
/// count too: branch protection blocks the merge on them all the same.
pub fn parse_unresolved_threads(stdout: &[u8]) -> Option<UnresolvedThreads> {
    let json: serde_json::Value = serde_json::from_slice(stdout).ok()?;
    let threads = json.pointer("/data/repository/pullRequest/reviewThreads")?;
    let count = threads
        .get("nodes")?
        .as_array()?
        .iter()
        .filter(|t| t.get("isResolved").and_then(|r| r.as_bool()) == Some(false))
        .count();
    let more = threads
        .pointer("/pageInfo/hasNextPage")
        .and_then(|n| n.as_bool())
        .unwrap_or(false);

    Some(UnresolvedThreads {
        count: u32::try_from(count).ok()?,
        more,
    })
}

/// The owner, repo and number in a `https://github.com/<owner>/<repo>/pull/<n>`
/// URL. The URL names the PR's base repo, so a branch pushed to a fork still
/// queries the repo the PR lives in.
fn pr_coordinates(url: &str) -> Option<(&str, &str, u64)> {
    let path = url.strip_prefix("https://github.com/")?;
    match path.split('/').collect::<Vec<_>>()[..] {
        [owner, repo, "pull", number] if !owner.is_empty() && !repo.is_empty() => {
            Some((owner, repo, number.parse().ok()?))
        }
        _ => None,
    }
}

/// Runs `gh pr view` synchronously and writes the result to the cache file.
///
/// This blocks on a network call, so it must only ever run in the detached
/// background worker process spawned by `spawn_background_fetch`, never on
/// the status-line's hot path.
pub fn fetch_and_write_pr_cache(repo_dir: &Path, cache_path: &Path, fetch_unresolved: bool) {
    let now = current_timestamp();
    let started = Instant::now();

    let output = output_with_timeout(
        Command::new("gh")
            .args([
                "pr",
                "view",
                "--json",
                "number,url,state,isDraft,reviewDecision,latestReviews",
            ])
            .current_dir(repo_dir),
        GH_TIMEOUT,
    );

    let view = output
        .filter(|out| out.status.success())
        .and_then(|out| parse_pr_view(&out.stdout));
    let pr = view.map(|view| {
        let mut pr = view.pr;
        if view.open && fetch_unresolved {
            // Both calls share one GH_TIMEOUT, so the worker as a whole
            // still ends before the throttle lets the next one start.
            let remaining = GH_TIMEOUT.saturating_sub(started.elapsed());
            pr.unresolved_threads = fetch_unresolved_threads(repo_dir, &pr.url, remaining);
        }
        pr
    });
    let entry = PrCacheEntry { timestamp: now, pr };

    let _ = write_cache_entry(cache_path, &entry);
    let _ = fs::remove_file(lock_file_path(cache_path));
}

/// Counts the PR's Unresolved Threads with `gh api graphql`, since
/// `gh pr view --json` has no review threads field.
fn fetch_unresolved_threads(
    repo_dir: &Path,
    url: &str,
    timeout: Duration,
) -> Option<UnresolvedThreads> {
    let (owner, repo, number) = pr_coordinates(url)?;
    let output = output_with_timeout(
        Command::new("gh")
            .args(["api", "graphql", "-f"])
            .arg(format!("query={REVIEW_THREADS_QUERY}"))
            .arg("-f")
            .arg(format!("owner={owner}"))
            .arg("-f")
            .arg(format!("repo={repo}"))
            .arg("-F")
            .arg(format!("number={number}"))
            .arg("-F")
            .arg(format!("first={REVIEW_THREADS_PAGE}"))
            .current_dir(repo_dir),
        timeout,
    )?;
    output
        .status
        .success()
        .then(|| parse_unresolved_threads(&output.stdout))
        .flatten()
}

pub fn spawn_background_fetch(repo_dir: &Path, cache_path: &Path, fetch_unresolved: bool) {
    if let Ok(exe) = std::env::current_exe() {
        let mut command = Command::new(exe);
        command
            .arg("--fetch-pr-cache")
            .arg(cache_path)
            .arg("--repo-dir")
            .arg(repo_dir);
        if fetch_unresolved {
            command.arg("--fetch-unresolved");
        }
        let _ = command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}

/// Returns the current branch's PR info, if any is cached and fresh.
///
/// Never blocks on the network: a stale or missing cache triggers a
/// throttled, detached background refresh (via re-invoking this binary with
/// `--fetch-pr-cache`) and returns immediately with whatever was cached.
pub fn get_pr_info(
    repo_dir: &Path,
    branch: &str,
    config: &PrConfig,
    gh_available: bool,
) -> Option<PullRequestInfo> {
    if !gh_available {
        return None;
    }

    let cache_path = get_cache_file_path(repo_dir, branch);
    let now = current_timestamp();
    let throttle_seconds = 15;

    let entry = read_cache_entry(&cache_path);
    if let Some(entry) = &entry
        && is_cache_fresh(entry, config.cache_ttl_seconds, now)
    {
        return entry.pr.clone();
    }

    // Cache is stale or missing. Claim the right to refresh atomically; the
    // loser of a concurrent race does not spawn a duplicate worker, and any
    // stale entry on disk is left untouched (never marked fresh) until a
    // real refresh completes.
    if try_claim_spawn(&cache_path, now, throttle_seconds) {
        spawn_background_fetch(repo_dir, &cache_path, config.show_unresolved);
    }

    entry.and_then(|e| e.pr)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_cache_entry_roundtrip(entry: PrCacheEntry) {
        let dir = tempfile::tempdir().unwrap();
        let temp_file = dir.path().join("cache.json");

        write_cache_entry(&temp_file, &entry).unwrap();
        let loaded = read_cache_entry(&temp_file).unwrap();
        assert_eq!(loaded, entry);
    }

    #[test]
    fn test_cache_entry_roundtrip() {
        assert_cache_entry_roundtrip(PrCacheEntry {
            timestamp: 1000,
            pr: Some(PullRequestInfo {
                number: 29,
                url: "https://github.com/org/repo/pull/29".to_string(),
                review_state: None,
                unresolved_threads: None,
            }),
        });
    }

    #[test]
    fn test_cache_entry_none_roundtrip() {
        assert_cache_entry_roundtrip(PrCacheEntry {
            timestamp: 12345,
            pr: None,
        });
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
    fn test_get_cache_file_path_deterministic() {
        let path1 = get_cache_file_path(Path::new("/some/repo"), "feature-x");
        let path2 = get_cache_file_path(Path::new("/some/repo"), "feature-x");
        let path3 = get_cache_file_path(Path::new("/some/repo"), "feature-y");
        assert!(path1.starts_with(crate::state::base_dir()));
        assert_eq!(path1, path2);
        assert_ne!(path1, path3);
    }

    fn pr_view(fields: &str) -> Option<PrView> {
        let json = format!(
            r#"{{"number":29,"url":"https://github.com/org/repo/pull/29","state":"OPEN",{fields}}}"#
        );
        parse_pr_view(json.as_bytes())
    }

    fn review(state: &str) -> String {
        format!(r#"{{"author":{{"login":"someone"}},"state":"{state}"}}"#)
    }

    #[test]
    fn test_changes_requested_review_beats_an_approval() {
        let view = pr_view(&format!(
            r#""isDraft":false,"reviewDecision":"","latestReviews":[{},{}]"#,
            review("APPROVED"),
            review("CHANGES_REQUESTED")
        ))
        .unwrap();

        assert_eq!(view.pr.number, 29);
        assert_eq!(view.pr.url, "https://github.com/org/repo/pull/29");
        assert_eq!(view.pr.review_state, Some(ReviewState::ChangesRequested));
        assert!(view.open);
    }

    #[test]
    fn test_a_draft_shows_as_draft_whatever_its_reviews_say() {
        let view = pr_view(&format!(
            r#""isDraft":true,"reviewDecision":"CHANGES_REQUESTED","latestReviews":[{}]"#,
            review("CHANGES_REQUESTED")
        ))
        .unwrap();

        assert_eq!(view.pr.review_state, Some(ReviewState::Draft));
    }

    #[test]
    fn test_the_review_decision_wins_over_the_latest_reviews() {
        let view = pr_view(&format!(
            r#""isDraft":false,"reviewDecision":"APPROVED","latestReviews":[{}]"#,
            review("CHANGES_REQUESTED")
        ))
        .unwrap();

        assert_eq!(view.pr.review_state, Some(ReviewState::Approved));
    }

    #[test]
    fn test_a_changes_requested_decision_wins_over_an_approval() {
        let view = pr_view(&format!(
            r#""isDraft":false,"reviewDecision":"CHANGES_REQUESTED","latestReviews":[{}]"#,
            review("APPROVED")
        ))
        .unwrap();

        assert_eq!(view.pr.review_state, Some(ReviewState::ChangesRequested));
    }

    #[test]
    fn test_a_required_review_not_yet_given_sets_no_state() {
        let view = pr_view(&format!(
            r#""isDraft":false,"reviewDecision":"REVIEW_REQUIRED","latestReviews":[{}]"#,
            review("APPROVED")
        ))
        .unwrap();

        assert_eq!(view.pr.review_state, None);
    }

    #[test]
    fn test_a_merged_pr_has_no_review_state() {
        let json = format!(
            r#"{{"number":29,"url":"https://github.com/org/repo/pull/29","state":"MERGED","isDraft":false,"reviewDecision":"","latestReviews":[{}]}}"#,
            review("APPROVED")
        );
        let view = parse_pr_view(json.as_bytes()).unwrap();

        assert!(!view.open);
        assert_eq!(view.pr.review_state, None);
    }

    #[test]
    fn test_comments_and_dismissed_reviews_set_no_state() {
        let view = pr_view(&format!(
            r#""isDraft":false,"reviewDecision":"","latestReviews":[{},{}]"#,
            review("COMMENTED"),
            review("DISMISSED")
        ))
        .unwrap();

        assert_eq!(view.pr.review_state, None);
    }

    #[test]
    fn test_a_cache_entry_from_before_review_support_still_parses() {
        let dir = tempfile::tempdir().unwrap();
        let temp_file = dir.path().join("old_cache.json");
        fs::write(
            &temp_file,
            r#"{"timestamp":1000,"pr":{"number":29,"url":"https://github.com/org/repo/pull/29"}}"#,
        )
        .unwrap();

        let loaded = read_cache_entry(&temp_file).unwrap();

        assert_eq!(loaded.pr.unwrap().review_state, None);
    }

    fn review_threads(resolved: &[bool], has_next_page: bool) -> String {
        let nodes: Vec<String> = resolved
            .iter()
            .map(|r| format!(r#"{{"isResolved":{r},"isOutdated":true}}"#))
            .collect();
        format!(
            r#"{{"data":{{"repository":{{"pullRequest":{{"reviewThreads":{{"pageInfo":{{"hasNextPage":{has_next_page}}},"nodes":[{}]}}}}}}}}}}"#,
            nodes.join(",")
        )
    }

    #[test]
    fn test_counts_unresolved_threads_outdated_ones_included() {
        let json = review_threads(&[false, true, false, false], false);

        assert_eq!(
            parse_unresolved_threads(json.as_bytes()),
            Some(UnresolvedThreads {
                count: 3,
                more: false
            })
        );
    }

    #[test]
    fn test_threads_past_the_first_page_mark_the_count_as_a_lower_bound() {
        let json = review_threads(&[false, false], true);

        assert_eq!(
            parse_unresolved_threads(json.as_bytes()),
            Some(UnresolvedThreads {
                count: 2,
                more: true
            })
        );
    }

    #[test]
    fn test_a_graphql_error_gives_no_thread_count() {
        let json = r#"{"data":{"repository":null},"errors":[{"message":"Could not resolve"}]}"#;

        assert_eq!(parse_unresolved_threads(json.as_bytes()), None);
    }

    #[test]
    fn test_the_pr_url_names_the_repo_and_number_to_query() {
        assert_eq!(
            pr_coordinates("https://github.com/xpepper/antigravity-powerline/pull/29"),
            Some(("xpepper", "antigravity-powerline", 29))
        );
    }

    #[test]
    fn test_an_unexpected_pr_url_names_nothing_to_query() {
        for url in [
            "https://evil.example.com/o/r/pull/29",
            "https://github.com/o/r/issues/29",
            "https://github.com/o/r/pull/x",
            "https://github.com/o/pull/29",
            "https://github.com//r/pull/29",
            "https://github.com/o//pull/29",
        ] {
            assert_eq!(pr_coordinates(url), None, "{url}");
        }
    }

    #[test]
    fn test_get_pr_info_when_gh_not_available() {
        let res = get_pr_info(Path::new("/some/repo"), "main", &PrConfig::default(), false);
        assert!(res.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_output_with_timeout_returns_stdout_of_a_finished_command() {
        let mut command = Command::new("echo");
        command.arg("hello");
        let out = output_with_timeout(&mut command, std::time::Duration::from_secs(5)).unwrap();
        assert!(out.status.success());
        assert_eq!(out.stdout, b"hello\n");
    }

    #[cfg(unix)]
    #[test]
    fn test_output_with_timeout_kills_a_command_that_hangs() {
        let started = std::time::Instant::now();
        let mut command = Command::new("sleep");
        command.arg("2");
        assert!(output_with_timeout(&mut command, std::time::Duration::from_millis(100)).is_none());
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }

    #[cfg(unix)]
    #[test]
    fn test_output_with_timeout_times_out_when_a_descendant_holds_stdout() {
        let started = std::time::Instant::now();
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 2 &"]);
        assert!(output_with_timeout(&mut command, std::time::Duration::from_millis(100)).is_none());
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
    }

    #[test]
    fn test_output_with_timeout_is_none_when_the_command_is_missing() {
        let mut command = Command::new("antigravity-powerline-no-such-command");
        assert!(output_with_timeout(&mut command, std::time::Duration::from_secs(1)).is_none());
    }

    #[test]
    fn test_try_claim_spawn_only_one_winner_when_racing() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("pr_test.json");
        let now = current_timestamp();

        let first = try_claim_spawn(&cache_path, now, 15);
        let second = try_claim_spawn(&cache_path, now, 15);

        assert!(first, "first claim attempt should win");
        assert!(!second, "second concurrent claim attempt should lose");
    }

    #[test]
    fn test_try_claim_spawn_allows_reclaim_after_throttle_window() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("pr_test.json");
        let lock_path = lock_file_path(&cache_path);
        let now = 1_000_000;

        assert!(try_claim_spawn(&cache_path, now, 15));
        File::open(&lock_path)
            .unwrap()
            .set_modified(UNIX_EPOCH + Duration::from_secs(now))
            .unwrap();

        assert!(should_throttle_spawn(&cache_path, now + 14, 15));
        assert!(!should_throttle_spawn(&cache_path, now + 15, 15));
        assert!(try_claim_spawn(&cache_path, now + 15, 15));
    }

    #[test]
    fn test_try_claim_spawn_single_winner_when_reclaiming_stale_lock_concurrently() {
        use std::sync::{Arc, Barrier};
        use std::time::Duration;

        let now = current_timestamp();
        for round in 0..50 {
            let dir = tempfile::tempdir().unwrap();
            let cache_path = dir.path().join(format!("claim_stale_race_{round}.json"));
            let lock_path = lock_file_path(&cache_path);
            let stale = fs::File::create(&lock_path).unwrap();
            stale
                .set_modified(UNIX_EPOCH + Duration::from_secs(now - 100))
                .unwrap();
            drop(stale);

            let barrier = Arc::new(Barrier::new(8));
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let (barrier, cache_path) = (barrier.clone(), cache_path.clone());
                    std::thread::spawn(move || {
                        barrier.wait();
                        try_claim_spawn(&cache_path, now, 15)
                    })
                })
                .collect();
            let winners = handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .filter(|won| *won)
                .count();

            assert_eq!(winners, 1, "round {round}: exactly one claim must win");
        }
    }

    #[test]
    fn test_try_claim_spawn_does_not_touch_cache_entry() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("claim_no_touch.json");
        let original = PrCacheEntry {
            timestamp: 100,
            pr: Some(PullRequestInfo {
                number: 7,
                url: "https://github.com/org/repo/pull/7".to_string(),
                review_state: None,
                unresolved_threads: None,
            }),
        };
        write_cache_entry(&cache_path, &original).unwrap();

        let now = current_timestamp();
        assert!(try_claim_spawn(&cache_path, now, 15));

        let unchanged = read_cache_entry(&cache_path).unwrap();
        assert_eq!(unchanged, original);
    }
}
