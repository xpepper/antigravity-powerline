//! Runs the built binary against a throwaway `HOME` to check that `main`
//! renders the quota segment it lists. The unit tests in
//! `src/segments/quota.rs` cover the formatting.

mod common;

use common::Sandbox;

#[test]
fn test_listed_quota_segment_is_rendered() {
    let sandbox = Sandbox::new();
    let payload =
        r#"{"quota":{"gemini-5h":{"remaining_fraction":0.961,"reset_in_seconds":17276}}}"#;

    let output = sandbox.refresh(
        payload,
        &[
            "--style",
            "plain",
            "--icon-set",
            "plain",
            "--theme",
            "plain",
        ],
    );

    assert!(
        output.contains("Quota: 5h: 96% (4h 47m)"),
        "no quota in {output:?}"
    );
}
