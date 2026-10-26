//! Runs the built binary against a throwaway `HOME` to check that the cache
//! segment keeps the Cache Trend and a large Cache Miss across Refreshes. The
//! unit tests in `src/cache_trend.rs` cover the rules; this covers the
//! snapshot wiring in `main`.

mod common;

use common::Sandbox;

/// A Refresh payload reporting one Step of the `conv` conversation, after
/// `total_output_tokens` output tokens in the Session.
fn payload(
    conv: &str,
    input_tokens: u64,
    cache_read_tokens: u64,
    total_output_tokens: u64,
) -> String {
    format!(
        r#"{{"conversation_id":"{conv}","session_id":"{conv}","context_window":{{"total_input_tokens":200000,"total_output_tokens":{total_output_tokens},"context_window_size":1048576,"current_usage":{{"input_tokens":{input_tokens},"output_tokens":100,"cache_read_input_tokens":{cache_read_tokens}}}}}}}"#
    )
}

fn refresh(sandbox: &Sandbox, conv: &str, input_tokens: u64, cache_read_tokens: u64) -> String {
    refresh_after(sandbox, conv, input_tokens, cache_read_tokens, 0)
}

fn refresh_after(
    sandbox: &Sandbox,
    conv: &str,
    input_tokens: u64,
    cache_read_tokens: u64,
    total_output_tokens: u64,
) -> String {
    let args = [
        "--style",
        "plain",
        "--icon-set",
        "plain",
        "--theme",
        "plain",
    ];
    let payload = payload(conv, input_tokens, cache_read_tokens, total_output_tokens);
    let output = sandbox.refresh(&payload, &args);
    output
        .split("  |  ")
        .find(|segment| segment.starts_with("Cache:"))
        .unwrap_or("")
        .to_string()
}

/// A warm Session, then a cold Step (168.8k sent, nothing read from the
/// cache), then a well-cached Step.
fn replay_cold_step(sandbox: &Sandbox) -> (String, String) {
    refresh(sandbox, "conv-a", 9_000, 40_000);
    let after_miss = refresh(sandbox, "conv-a", 168_800, 0);
    let next_step = refresh(sandbox, "conv-a", 4_000, 170_000);
    (after_miss, next_step)
}

#[test]
fn test_cache_miss_stays_visible_after_the_next_step() {
    let sandbox = Sandbox::new();

    let (after_miss, next_step) = replay_cold_step(&sandbox);

    assert_eq!(after_miss, "Cache: 0% ↓ · miss 169k");
    assert_eq!(next_step, "Cache: 98% ↑ · miss 169k");
}

#[test]
fn test_refresh_reporting_the_same_step_keeps_the_arrow() {
    let sandbox = Sandbox::new();
    refresh(&sandbox, "conv-a", 9_000, 40_000);
    refresh(&sandbox, "conv-a", 9_000, 1_000);

    assert_eq!(refresh(&sandbox, "conv-a", 9_000, 1_000), "Cache: 10% ↓");
}

#[test]
fn test_identical_steps_both_count_when_the_totals_advance() {
    let sandbox = Sandbox::new();
    refresh_after(&sandbox, "conv-a", 0, 100_000, 100);
    // Two cold Steps with identical counts take the Session average from
    // 100% to 33%, rather than to 50% if the second were taken for a repeat.
    refresh_after(&sandbox, "conv-a", 100_000, 0, 200);
    refresh_after(&sandbox, "conv-a", 100_000, 0, 300);

    assert_eq!(
        refresh_after(&sandbox, "conv-a", 6_000, 4_000, 400),
        "Cache: 40% ↑ · miss 100k"
    );
}

#[test]
fn test_each_conversation_keeps_its_own_snapshot() {
    let sandbox = Sandbox::new();
    replay_cold_step(&sandbox);

    assert_eq!(refresh(&sandbox, "conv-b", 4_000, 170_000), "Cache: 98%");
    assert_eq!(sandbox.state_files("cache_").len(), 2);
}

#[test]
fn test_cache_miss_can_be_turned_off() {
    let sandbox = Sandbox::new();
    sandbox.write_config("[cache]\nshow_last_miss = false\n");

    let (_, next_step) = replay_cold_step(&sandbox);

    assert_eq!(next_step, "Cache: 98% ↑");
}

#[test]
fn test_no_snapshot_without_a_conversation() {
    let sandbox = Sandbox::new();

    assert_eq!(refresh(&sandbox, "", 4_000, 170_000), "Cache: 98%");
    assert!(sandbox.state_files("cache_").is_empty());
}

#[test]
fn test_no_snapshot_when_no_signal_is_shown() {
    let sandbox = Sandbox::new();
    sandbox.write_config("[cache]\nshow_trend = false\nshow_last_miss = false\n");

    replay_cold_step(&sandbox);

    assert!(sandbox.state_files("cache_").is_empty());
}
