//! Cache Trend: compares the Cache Hit Rate of the latest Step with the
//! Session average before it. Also remembers the latest large Cache Miss,
//! which the per-Step rate and the arrow both hide once the next Step lands.
//!
//! Antigravity's `current_usage` holds the latest Step only, and the payload
//! has no Session cache totals, so the Session average is summed here, in a
//! per-Session snapshot, one Step at a time.

use crate::config::CacheConfig;
use crate::input::CurrentUsageInfo;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Percentage points the latest Step's hit rate must differ from the Session
/// average before a trend is shown, so small fluctuations don't flicker.
const TREND_TOLERANCE_PCT: f64 = 5.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Up,
    Down,
}

/// A Step that sent many input tokens without reading them from the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Miss {
    /// Input tokens of the Step not served from the cache.
    pub tokens: u64,
    /// Unix time (seconds) when the Step was seen.
    pub at: u64,
}

/// Token counts of one Step, as `current_usage` reports them. Antigravity's
/// `input_tokens` excludes the tokens read from the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    pub input_tokens: u64,
    pub cache_read_tokens: u64,
    pub output_tokens: u64,
}

impl Step {
    fn prompt_tokens(&self) -> u64 {
        self.input_tokens + self.cache_read_tokens
    }

    /// Cache Hit Rate in percent, `None` when the Step sent no prompt tokens.
    pub fn hit_rate(&self) -> Option<f64> {
        hit_rate(self.cache_read_tokens, self.prompt_tokens())
    }
}

impl From<&CurrentUsageInfo> for Step {
    fn from(usage: &CurrentUsageInfo) -> Self {
        Self {
            input_tokens: usage.input_tokens.unwrap_or(0),
            cache_read_tokens: usage.cache_read_input_tokens.unwrap_or(0),
            output_tokens: usage.output_tokens.unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    /// The Step this snapshot last counted. A Refresh reporting the same
    /// Step again (timer-driven) changes nothing.
    pub last_step: Step,
    /// Prompt tokens (input plus cache reads) summed over the Steps seen. A
    /// Step that lands between two Refreshes is never seen, so the Session
    /// average covers the Steps the status line saw.
    pub prompt_tokens: u64,
    /// Cache reads summed over the Steps seen.
    pub cache_read_tokens: u64,
    /// Verdict for the last Step.
    pub trend: Option<Trend>,
    /// Latest Step with at least `miss_min_tokens` (and at least one)
    /// uncached input tokens.
    pub last_miss: Option<Miss>,
}

/// Counts `step` into the Session snapshot `prev`, deciding its Cache Trend
/// against the Steps before it and whether it was a Cache Miss of at least
/// `miss_min_tokens` (and at least one) uncached tokens.
pub fn assess(prev: Option<Snapshot>, step: Step, now: u64, miss_min_tokens: u64) -> Snapshot {
    // The first Step seen is the baseline: before it, nothing could have
    // been read from the cache.
    let Some(prev) = prev else {
        return Snapshot {
            last_step: step,
            prompt_tokens: step.prompt_tokens(),
            cache_read_tokens: step.cache_read_tokens,
            trend: None,
            last_miss: None,
        };
    };
    if step == prev.last_step {
        return prev;
    }

    let delta = step
        .hit_rate()
        .zip(hit_rate(prev.cache_read_tokens, prev.prompt_tokens))
        .map(|(latest, average)| latest - average);
    let trend = match delta {
        Some(d) if d > TREND_TOLERANCE_PCT => Some(Trend::Up),
        Some(d) if d < -TREND_TOLERANCE_PCT => Some(Trend::Down),
        _ => None,
    };

    let miss = (step.input_tokens > 0 && step.input_tokens >= miss_min_tokens).then_some(Miss {
        tokens: step.input_tokens,
        at: now,
    });

    Snapshot {
        last_step: step,
        prompt_tokens: prev.prompt_tokens + step.prompt_tokens(),
        cache_read_tokens: prev.cache_read_tokens + step.cache_read_tokens,
        trend,
        last_miss: miss.or(prev.last_miss),
    }
}

fn hit_rate(cache_read: u64, prompt: u64) -> Option<f64> {
    // Multiplying first keeps whole percentages exact at the tolerance.
    (prompt > 0).then(|| cache_read as f64 * 100.0 / prompt as f64)
}

impl Snapshot {
    /// Size of the last miss while it is at most `miss_visible_seconds` old.
    pub fn recent_miss(&self, now: u64, miss_visible_seconds: u64) -> Option<u64> {
        self.last_miss
            .filter(|miss| now.saturating_sub(miss.at) <= miss_visible_seconds)
            .map(|miss| miss.tokens)
    }
}

pub fn snapshot_path(session_id: &str) -> PathBuf {
    crate::state::session_file("cache", session_id)
}

/// What the cache segment shows from the Session snapshot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Signals {
    pub trend: Option<Trend>,
    /// Size of a miss at most `miss_visible_seconds` old.
    pub recent_miss: Option<u64>,
}

/// Reads the Session snapshot at `path`, counts `step` into it and persists
/// the result. Returns what to display.
pub fn check(path: &Path, step: Step, now: u64, config: &CacheConfig) -> Signals {
    let snapshot =
        crate::state::update_snapshot(path, |prev| assess(prev, step, now, config.miss_min_tokens));
    Signals {
        trend: snapshot.trend,
        recent_miss: snapshot.recent_miss(now, config.miss_visible_seconds),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: u64 = 1_000_000;
    const MISS_MIN: u64 = 50_000;

    fn step(input_tokens: u64, cache_read_tokens: u64) -> Step {
        Step {
            input_tokens,
            cache_read_tokens,
            output_tokens: 100,
        }
    }

    fn assess_at_now(prev: Option<Snapshot>, step: Step) -> Snapshot {
        super::assess(prev, step, NOW, MISS_MIN)
    }

    #[test]
    fn test_first_step_seen_is_the_baseline() {
        let next = assess_at_now(None, step(4_000, 36_000));
        assert_eq!(next.trend, None);
        assert_eq!(next.last_miss, None);
        assert_eq!(
            (next.prompt_tokens, next.cache_read_tokens),
            (40_000, 36_000)
        );
    }

    /// A Session at 60% over 100k prompt tokens, last Step `last_step`.
    fn at_60_pct(last_step: Step) -> Snapshot {
        Snapshot {
            last_step,
            prompt_tokens: 100_000,
            cache_read_tokens: 60_000,
            trend: None,
            last_miss: None,
        }
    }

    #[test]
    fn test_step_hitting_cache_more_than_average_trends_up_and_is_counted() {
        // The Step sends 10k prompt tokens, 9k from cache (90%).
        let next = assess_at_now(Some(at_60_pct(step(5_000, 1_000))), step(1_000, 9_000));
        assert_eq!(next.trend, Some(Trend::Up));
        assert_eq!(
            (next.prompt_tokens, next.cache_read_tokens),
            (110_000, 69_000)
        );
        assert_eq!(next.last_step, step(1_000, 9_000));
    }

    #[test]
    fn test_step_missing_cache_trends_down() {
        // 10k prompt tokens, 1k from cache (10%).
        let next = assess_at_now(Some(at_60_pct(step(5_000, 1_000))), step(9_000, 1_000));
        assert_eq!(next.trend, Some(Trend::Down));
    }

    #[test]
    fn test_step_close_to_average_has_no_trend() {
        // 62% against 60%.
        let prev = Snapshot {
            trend: Some(Trend::Down),
            ..at_60_pct(step(5_000, 1_000))
        };
        assert_eq!(assess_at_now(Some(prev), step(3_800, 6_200)).trend, None);
    }

    #[test]
    fn test_step_exactly_at_the_tolerance_has_no_trend() {
        let prev = at_60_pct(step(5_000, 1_000));
        // 65% and 55% against 60%.
        assert_eq!(assess_at_now(Some(prev), step(3_500, 6_500)).trend, None);
        assert_eq!(assess_at_now(Some(prev), step(4_500, 5_500)).trend, None);
    }

    #[test]
    fn test_no_trend_without_prompt_tokens_to_compare() {
        let empty = Snapshot {
            prompt_tokens: 0,
            cache_read_tokens: 0,
            ..at_60_pct(step(0, 0))
        };
        assert_eq!(assess_at_now(Some(empty), step(1_000, 9_000)).trend, None);
        let prev = at_60_pct(step(5_000, 1_000));
        assert_eq!(assess_at_now(Some(prev), step(0, 0)).trend, None);
    }

    #[test]
    fn test_refresh_reporting_the_same_step_changes_nothing() {
        let prev = Snapshot {
            trend: Some(Trend::Down),
            ..at_60_pct(step(9_000, 1_000))
        };
        assert_eq!(assess_at_now(Some(prev), step(9_000, 1_000)), prev);
    }

    fn miss(tokens: u64, at: u64) -> Option<Miss> {
        Some(Miss { tokens, at })
    }

    #[test]
    fn test_large_uncached_step_is_recorded_as_miss() {
        // A cold Step: 168.8k sent, nothing read from the cache.
        let next = assess_at_now(Some(at_60_pct(step(5_000, 1_000))), step(168_800, 0));
        assert_eq!(next.last_miss, miss(168_800, NOW));
    }

    #[test]
    fn test_step_below_the_floor_keeps_the_last_miss() {
        let prev = Snapshot {
            last_miss: miss(168_800, NOW - 60),
            ..at_60_pct(step(168_800, 0))
        };
        let next = assess_at_now(Some(prev), step(4_000, 170_000));
        assert_eq!(next.last_miss, miss(168_800, NOW - 60));
    }

    #[test]
    fn test_newer_miss_replaces_the_last_one() {
        let prev = Snapshot {
            last_miss: miss(168_800, NOW - 60),
            ..at_60_pct(step(168_800, 0))
        };
        let next = assess_at_now(Some(prev), step(60_000, 8_000));
        assert_eq!(next.last_miss, miss(60_000, NOW));
    }

    #[test]
    fn test_fully_cached_step_is_not_a_miss_even_with_a_zero_floor() {
        let next = super::assess(Some(at_60_pct(step(5_000, 1_000))), step(0, 10_000), NOW, 0);
        assert_eq!(next.last_miss, None);
    }

    #[test]
    fn test_first_step_seen_is_not_a_miss() {
        // Nothing can be read from the cache before a Session's first Step,
        // such as one that opens with a large paste.
        let next = assess_at_now(None, step(155_135, 0));
        assert_eq!(next.last_miss, None);
        assert_eq!(next.trend, None);
    }

    #[test]
    fn test_miss_is_recent_within_the_visible_window() {
        let s = Snapshot {
            last_miss: miss(168_800, NOW),
            ..at_60_pct(step(168_800, 0))
        };
        assert_eq!(s.recent_miss(NOW, 300), Some(168_800));
        assert_eq!(s.recent_miss(NOW + 300, 300), Some(168_800));
        assert_eq!(s.recent_miss(NOW + 301, 300), None);
    }

    #[test]
    fn test_no_recent_miss_without_one() {
        assert_eq!(at_60_pct(step(5_000, 1_000)).recent_miss(NOW, 300), None);
    }

    #[test]
    fn test_check_carries_the_session_across_refreshes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cache_0123456789abcdef.json");
        let config = CacheConfig {
            miss_min_tokens: MISS_MIN,
            miss_visible_seconds: 300,
            ..CacheConfig::default()
        };
        let check = |step, now| super::check(&path, step, now, &config);

        assert_eq!(check(step(9_000, 40_000), NOW), Signals::default());
        let after_miss = check(step(168_800, 0), NOW + 10);
        assert_eq!(
            after_miss,
            Signals {
                trend: Some(Trend::Down),
                recent_miss: Some(168_800),
            }
        );
        // Timer Refresh, nothing new: both stay.
        assert_eq!(check(step(168_800, 0), NOW + 20), after_miss);
        // A well-cached Step: the arrow flips, the miss stays visible.
        assert_eq!(
            check(step(4_000, 170_000), NOW + 30),
            Signals {
                trend: Some(Trend::Up),
                recent_miss: Some(168_800),
            }
        );
        assert_eq!(check(step(4_000, 170_000), NOW + 400).recent_miss, None);
    }
}
