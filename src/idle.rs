//! Idle time since the Session's last Step, used by the long-break reminder
//! (`cache_expiry`). Typing, reading and long tool runs all count as idle.
//!
//! The payload carries no timestamps, so the time the latest Step was first
//! seen is kept in a per-Session snapshot.

use crate::cache_trend::Step;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The latest Step seen; a different one means a new Step happened.
    pub step: Step,
    /// Unix time (seconds) when `step` was first seen.
    pub changed_at: u64,
}

/// Restarts the idle clock when a new Step happened, returning the snapshot
/// to persist.
pub fn assess(prev: Option<Snapshot>, step: Step, now: u64) -> Snapshot {
    match prev {
        Some(prev) if prev.step == step => prev,
        _ => Snapshot {
            step,
            changed_at: now,
        },
    }
}

pub fn snapshot_path(session_id: &str) -> PathBuf {
    crate::state::session_file("idle", session_id)
}

/// Reads the Session snapshot at `path`, records `step` as the latest Step
/// and returns how many seconds have passed since it was first seen.
pub fn idle_seconds(path: &Path, step: Step, now: u64) -> u64 {
    let snapshot = crate::state::update_snapshot(path, |prev| assess(prev, step, now));
    now.saturating_sub(snapshot.changed_at)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(total_output_tokens: u64) -> Step {
        Step {
            input_tokens: 4_000,
            cache_read_tokens: 86_000,
            output_tokens: 100,
            total_input_tokens: 90_000,
            total_output_tokens,
        }
    }

    fn snap(step: Step, changed_at: u64) -> Snapshot {
        Snapshot { step, changed_at }
    }

    #[test]
    fn test_first_step_seen_starts_the_clock() {
        assert_eq!(assess(None, step(700), 1_000), snap(step(700), 1_000));
    }

    #[test]
    fn test_refresh_reporting_the_same_step_keeps_the_clock_running() {
        let prev = snap(step(700), 1_000);
        assert_eq!(assess(Some(prev), step(700), 1_400), prev);
    }

    #[test]
    fn test_new_step_restarts_the_clock() {
        // Only the output total tells this Step apart from the one before.
        let prev = snap(step(700), 1_000);
        assert_eq!(assess(Some(prev), step(800), 1_400), snap(step(800), 1_400));
    }

    #[test]
    fn test_idle_seconds_grow_across_refreshes_of_the_same_step() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("idle_0123456789abcdef.json");

        assert_eq!(idle_seconds(&path, step(700), 1_000), 0);
        assert_eq!(idle_seconds(&path, step(700), 1_030), 30);
        assert_eq!(idle_seconds(&path, step(700), 4_300), 3_300);
        assert_eq!(idle_seconds(&path, step(800), 4_310), 0);
    }

    #[test]
    fn test_clock_going_backwards_is_not_idle() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("idle_0123456789abcdef.json");

        assert_eq!(idle_seconds(&path, step(700), 1_000), 0);
        assert_eq!(idle_seconds(&path, step(700), 900), 0);
    }
}
