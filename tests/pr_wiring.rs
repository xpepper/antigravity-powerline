//! Runs the built binary against a throwaway `HOME` that is also a git repo,
//! to check the `pr` segment wiring end to end: a Refresh with no cache must
//! spawn the `--fetch-pr-cache` worker, the worker must make the right `gh`
//! calls and cache what they return, and the next Refresh must show it. The
//! unit tests in `src/github.rs` and `src/segments/pr.rs` cover parsing and
//! rendering, but cannot see what `main` passes to the worker.
//!
//! A fake `gh` on `PATH` stands in for the network calls and logs each one.
#![cfg(unix)]

mod common;

use common::Sandbox;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const URL: &str = "https://github.com/org/repo/pull/29";

/// A config showing the pr segment with both review switches on, as they
/// are off by default.
const REVIEWS_ON: &str = "theme = \"plain\"\nsegments = [\"pr\"]\n\n[pr]\nshow_review_state = true\nshow_unresolved = true\n";

fn pr_view(state: &str, latest_review: &str) -> String {
    format!(
        r#"{{"number":29,"url":"{URL}","state":"{state}","isDraft":false,"reviewDecision":"","latestReviews":[{{"author":{{"login":"someone"}},"state":"{latest_review}"}}]}}"#
    )
}

/// Three unresolved threads, one of them outdated, and one resolved.
const THREADS: &str = r#"{"data":{"repository":{"pullRequest":{"reviewThreads":{"pageInfo":{"hasNextPage":false},"nodes":[{"isResolved":false,"isOutdated":false},{"isResolved":true,"isOutdated":false},{"isResolved":false,"isOutdated":true},{"isResolved":false,"isOutdated":false}]}}}}}"#;

struct FakeGh {
    bin: PathBuf,
    log: PathBuf,
}

impl FakeGh {
    /// The `gh` calls made so far, one line of arguments each.
    fn calls(&self) -> Vec<String> {
        fs::read_to_string(&self.log)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }
}

/// Makes `HOME` a repo on branch `feature` and puts a `gh` in a bin dir
/// under it that answers `gh pr view` with `pr_view` and `gh api graphql`
/// with `threads`, or fails that call when `threads` is `None`.
fn fake_gh(sandbox: &Sandbox, pr_view: &str, threads: Option<&str>) -> FakeGh {
    let home = sandbox.home();
    fs::create_dir_all(home.join(".git")).unwrap();
    fs::write(home.join(".git/HEAD"), "ref: refs/heads/feature\n").unwrap();

    let bin = home.join("bin");
    fs::create_dir_all(&bin).unwrap();
    let pr_view_path = home.join("gh-pr-view.json");
    fs::write(&pr_view_path, pr_view).unwrap();
    let threads_path = home.join("gh-threads.json");
    if let Some(threads) = threads {
        fs::write(&threads_path, threads).unwrap();
    }
    let log = home.join("gh-calls.log");
    let gh = bin.join("gh");
    fs::write(
        &gh,
        format!(
            "#!/bin/sh\n\
             echo \"$*\" | /usr/bin/tr '\\n' ' ' >> '{log}'\n\
             echo >> '{log}'\n\
             case \"$1\" in\n\
             pr) exec /bin/cat '{pr_view}' ;;\n\
             api) [ -f '{threads}' ] && exec /bin/cat '{threads}' ;;\n\
             esac\n\
             exit 1\n",
            log = log.display(),
            pr_view = pr_view_path.display(),
            threads = threads_path.display(),
        ),
    )
    .unwrap();
    fs::set_permissions(&gh, fs::Permissions::from_mode(0o755)).unwrap();
    FakeGh { bin, log }
}

fn refresh(sandbox: &Sandbox, gh: &FakeGh) -> String {
    let payload = format!(r#"{{"cwd":"{}"}}"#, sandbox.home().display());
    sandbox.refresh_with_env(
        &payload,
        &["--style", "plain", "--icon-set", "plain"],
        &[("PATH", &gh.bin)],
    )
}

/// Waits for the detached worker the first Refresh spawned: done once it
/// has written the cache (atomically, so the file appears complete).
fn wait_for_worker(sandbox: &Sandbox) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let cached = sandbox
            .state_files("pr_")
            .iter()
            .any(|p| p.extension().is_some_and(|e| e == "json"));
        if cached {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the PR cache worker never finished"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn shown_after_fetch(sandbox: &Sandbox, gh: &FakeGh) -> String {
    refresh(sandbox, gh);
    wait_for_worker(sandbox);
    refresh(sandbox, gh)
}

#[test]
fn test_shows_review_state_and_unresolved_threads_after_the_fetch() {
    let sandbox = Sandbox::new();
    sandbox.write_config(REVIEWS_ON);
    let gh = fake_gh(
        &sandbox,
        &pr_view("OPEN", "CHANGES_REQUESTED"),
        Some(THREADS),
    );

    let line = shown_after_fetch(&sandbox, &gh);

    assert_eq!(line, "PR #29 · changes requested · 3 unresolved");
    let calls = gh.calls();
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert!(
        calls[0]
            .starts_with("pr view --json number,url,state,isDraft,reviewDecision,latestReviews"),
        "{calls:?}"
    );
    assert!(calls[1].starts_with("api graphql"), "{calls:?}");
    assert!(calls[1].contains("owner=org"), "{calls:?}");
    assert!(calls[1].contains("repo=repo"), "{calls:?}");
    assert!(calls[1].contains("number=29"), "{calls:?}");
}

#[test]
fn test_a_refresh_right_after_the_fetch_does_not_fetch_again() {
    let sandbox = Sandbox::new();
    // A zero TTL makes every cache entry stale, so only the spawn throttle
    // stands between each Refresh and a new worker.
    sandbox.write_config("theme = \"plain\"\nsegments = [\"pr\"]\n\n[pr]\ncache_ttl_seconds = 0\n");
    let gh = fake_gh(&sandbox, &pr_view("OPEN", "APPROVED"), None);

    shown_after_fetch(&sandbox, &gh);
    // Give a wrongly spawned second worker time to call `gh`.
    std::thread::sleep(Duration::from_millis(500));

    assert_eq!(gh.calls().len(), 1, "{:?}", gh.calls());
}

#[test]
fn test_a_failed_thread_query_still_shows_the_review_state() {
    let sandbox = Sandbox::new();
    sandbox.write_config(REVIEWS_ON);
    let gh = fake_gh(&sandbox, &pr_view("OPEN", "CHANGES_REQUESTED"), None);

    let line = shown_after_fetch(&sandbox, &gh);

    assert_eq!(line, "PR #29 · changes requested");
}

#[test]
fn test_a_merged_pr_skips_the_thread_query() {
    let sandbox = Sandbox::new();
    sandbox.write_config(REVIEWS_ON);
    let gh = fake_gh(&sandbox, &pr_view("MERGED", "APPROVED"), Some(THREADS));

    let line = shown_after_fetch(&sandbox, &gh);

    assert_eq!(line, "PR #29");
    assert_eq!(gh.calls().len(), 1, "{:?}", gh.calls());
}

#[test]
fn test_the_default_config_skips_the_thread_query() {
    let sandbox = Sandbox::new();
    sandbox.write_config("theme = \"plain\"\nsegments = [\"pr\"]\n");
    let gh = fake_gh(
        &sandbox,
        &pr_view("OPEN", "CHANGES_REQUESTED"),
        Some(THREADS),
    );

    let line = shown_after_fetch(&sandbox, &gh);

    // The plain theme has no colours, so the review state is spelled out.
    assert_eq!(line, "PR #29 · changes requested");
    assert_eq!(gh.calls().len(), 1, "{:?}", gh.calls());
}

#[test]
fn test_the_review_state_alone_skips_the_thread_query() {
    let sandbox = Sandbox::new();
    sandbox
        .write_config("theme = \"plain\"\nsegments = [\"pr\"]\n\n[pr]\nshow_review_state = true\n");
    let gh = fake_gh(&sandbox, &pr_view("OPEN", "APPROVED"), Some(THREADS));

    let line = shown_after_fetch(&sandbox, &gh);

    assert_eq!(line, "PR #29 · approved");
    assert_eq!(gh.calls().len(), 1, "{:?}", gh.calls());
}
