//! Runs the built binary against a throwaway `HOME` to check that the
//! long-break reminder tracks idle time across Refreshes. The unit tests in
//! `src/idle.rs` and `src/segments/cache_expiry.rs` cover the rules; this
//! covers the snapshot wiring in `main`.

mod common;

use common::Sandbox;
use std::time::Duration;

/// A Refresh payload reporting the `conv` conversation after a Step whose
/// output brought the Session's output total to `total_output`.
fn payload(conv: &str, total_output: u64) -> String {
    format!(
        r#"{{"conversation_id":"{conv}","session_id":"","context_window":{{"total_input_tokens":200000,"total_output_tokens":{total_output},"context_window_size":1048576,"current_usage":{{"input_tokens":4000,"output_tokens":100,"cache_read_input_tokens":196000}}}}}}"#
    )
}

/// The reminder from one Refresh, empty when it is hidden.
fn refresh(sandbox: &Sandbox, conv: &str, total_output: u64) -> String {
    let args = [
        "--style",
        "plain",
        "--icon-set",
        "plain",
        "--theme",
        "plain",
    ];
    sandbox
        .refresh(&payload(conv, total_output), &args)
        .split("  |  ")
        .find(|segment| segment.starts_with("Idle:"))
        .unwrap_or("")
        .to_string()
}

/// A one-second threshold, so a test can wait out the break.
fn sandbox_with_short_break() -> Sandbox {
    let sandbox = Sandbox::new();
    sandbox.write_config("[cache_expiry]\nidle_seconds = 1\n");
    sandbox
}

#[test]
fn test_reminder_shows_after_a_break_and_clears_on_the_next_step() {
    let sandbox = sandbox_with_short_break();
    assert_eq!(refresh(&sandbox, "conv-a", 700), "");

    // Long enough for the whole-second clock to move at least once.
    std::thread::sleep(Duration::from_millis(1_100));

    let after_break = refresh(&sandbox, "conv-a", 700);
    assert!(
        [
            "Idle: 1s · cache likely cold",
            "Idle: 2s · cache likely cold"
        ]
        .contains(&after_break.as_str()),
        "unexpected reminder {after_break:?}"
    );
    assert_eq!(refresh(&sandbox, "conv-a", 800), "");
}

#[test]
fn test_each_conversation_keeps_its_own_snapshot() {
    let sandbox = sandbox_with_short_break();
    refresh(&sandbox, "conv-a", 700);
    refresh(&sandbox, "conv-b", 700);

    assert_eq!(sandbox.state_files("idle_").len(), 2);
}

#[test]
fn test_no_snapshot_without_a_conversation() {
    let sandbox = sandbox_with_short_break();

    assert_eq!(refresh(&sandbox, "", 700), "");
    assert!(sandbox.state_files("idle_").is_empty());
}

#[test]
fn test_no_snapshot_before_the_first_step() {
    let sandbox = sandbox_with_short_break();
    let before_first_step = r#"{"conversation_id":"conv-a","context_window":{"total_input_tokens":200000,"context_window_size":1048576}}"#;

    sandbox.refresh(before_first_step, &[]);

    assert!(sandbox.state_files("idle_").is_empty());
}

#[test]
fn test_no_snapshot_when_the_segment_is_not_listed() {
    let sandbox = Sandbox::new();
    sandbox.write_config("segments = [\"tokens\"]\n");

    refresh(&sandbox, "conv-a", 700);

    assert!(sandbox.state_files("idle_").is_empty());
}

#[test]
fn test_no_snapshot_when_disabled() {
    let sandbox = Sandbox::new();
    sandbox.write_config("[cache_expiry]\nenabled = false\n");

    refresh(&sandbox, "conv-a", 700);

    assert!(sandbox.state_files("idle_").is_empty());
}
