use std::fs;

fn workflow(name: &str) -> String {
    fs::read_to_string(format!(".github/workflows/{name}"))
        .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn production_workflow_preserves_the_marker_across_noop_runs() {
    let production = workflow("prod.yml");

    // The filtered `gh run list` goes through a search index that is
    // intermittently stale, so the previous run must be resolved through the
    // REST endpoint instead.
    assert!(
        !production.contains("gh run list"),
        "production workflow must not resolve the previous run via gh run list (stale index)"
    );
    assert!(
        production.contains(
            "repos/${{ github.repository }}/actions/workflows/prod.yml/runs?per_page=100"
        ),
        "production workflow is missing the REST lookup of the previous production run"
    );

    // The REST endpoint serves stale data too (it once named an April run as
    // "latest"), so a single lookup is not enough: the workflow must walk the
    // recent successful runs until one yields a usable marker, and a missing
    // marker must warn rather than abort — the chain is rebuilt on delivery.
    assert!(
        production.contains("select(.conclusion == \"success\")][0:10][]")
            && !production.contains("status=success")
            && !production.contains("branch=main"),
        "production workflow must walk several recent runs, not trust one lookup"
    );
    assert!(
        !production.contains("refusing to publish"),
        "a broken marker chain must warn and rebuild, never abort the run"
    );

    for required in [
        "--repo \"${{ github.repository }}\"",
        "for name in last-sent-prod last-sent; do",
        "last_sent=$(cat last_sent.txt 2>/dev/null || echo \"\")",
        "- name: Preserve last_sent marker when delivery is not needed",
        "if: steps.decide.outputs.should_send == 'false'",
        "- name: Upload preserved last_sent marker",
        "name: last-sent-prod",
        "path: last_sent.txt",
        "if-no-files-found: error",
    ] {
        assert!(
            production.contains(required),
            "production workflow is missing required delivery-state contract: {required}"
        );
    }
}

#[test]
fn delivery_workflow_writes_the_marker_only_after_a_delivery() {
    let delivery = workflow("common-delivery.yml");

    // The filtered `gh run list` goes through a search index that is
    // intermittently stale, so the previous run must be resolved through the
    // REST endpoint instead.
    assert!(
        !delivery.contains("gh run list"),
        "common delivery workflow must not resolve the previous run via gh run list (stale index)"
    );
    assert!(
        delivery.contains(
            "repos/${{ github.repository }}/actions/workflows/prod.yml/runs?per_page=100"
        ),
        "common delivery workflow is missing the REST lookup of the previous production run"
    );

    // Same as the production workflow: walk several recent runs, warn instead
    // of aborting when the chain is broken.
    assert!(
        delivery.contains("select(.conclusion == \"success\")][0:10][]")
            && !delivery.contains("status=success")
            && !delivery.contains("branch=main"),
        "common delivery workflow must walk several recent runs, not trust one lookup"
    );
    assert!(
        !delivery.contains("refusing to publish")
            && !delivery.contains("requires a valid last-sent marker"),
        "a broken marker chain must warn and rebuild, never abort the run"
    );
    assert!(
        !delivery.contains("if: always()\n        uses: actions/upload-artifact@v7\n        with:\n          name: last-sent-"),
        "the last_sent artifact must be uploaded only by successful runs"
    );

    for required in [
        "- name: Save last_sent marker",
        "steps.prepare.outputs.send == 'true' && (inputs.send_main == true || inputs.send_dev == true)",
        "printf '%s\\n' \"$LATEST_POST\" > last_sent.txt",
        "- name: Upload last_sent artifact",
        "name: last-sent-${{ env.ARTIFACT_SCOPE }}",
        "path: last_sent.txt",
    ] {
        assert!(
            delivery.contains(required),
            "common delivery workflow is missing required delivery-state contract: {required}"
        );
    }
}

#[test]
fn production_send_is_guarded_by_the_channel_pin_check() {
    let cli = fs::read_to_string("src/cli.rs")
        .unwrap_or_else(|error| panic!("failed to read src/cli.rs: {error}"));

    let guard_position = cli
        .find("pin_guard::enforce_pin_guard")
        .expect("production send must be preceded by the Telegram pin guard");
    let send_position = cli
        .find("Sending posts to production Telegram chat")
        .expect("production send must exist");
    assert!(
        guard_position < send_position,
        "the pin guard must run before the production Telegram send"
    );
}
