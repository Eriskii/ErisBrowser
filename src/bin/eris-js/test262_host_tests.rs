use super::*;

const PINNED_DONE: &str = include_str!(
    "../../../tests/upstream/test262-promise-methods-async/harness/doneprintHandle.js"
);

fn text(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

fn frame(mode: u8, parse: u8, policy: u8, harness: &[(&str, &str)], source: &str) -> Vec<u8> {
    let mut bytes = b"ERJS2".to_vec();
    bytes.extend_from_slice(&[mode, parse, policy]);
    bytes.extend_from_slice(&(harness.len() as u32).to_le_bytes());
    for (name, value) in harness {
        text(&mut bytes, name);
        text(&mut bytes, value);
    }
    text(&mut bytes, source);
    bytes
}

fn request(mode: u8, policy: u8, includes: &[(&str, &str)], source: &str) -> HostRequest {
    let mut harness = vec![("assert.js", ""), ("sta.js", "")];
    if policy == 1 {
        harness.push(("doneprintHandle.js", PINNED_DONE));
    }
    harness.extend_from_slice(includes);
    decode(&frame(mode, 0, policy, &harness, source)).expect("independent valid request")
}

fn evaluate(request: HostRequest) -> HostOutcome {
    evaluate_with_runtime(
        request,
        || Ok(eris::date_host::DateHost::unconfigured()),
        Runtime::try_with_date_host,
    )
    .expect("adapter setup succeeds")
}

fn counts(result: &HostOutcome) -> (u8, u8, u8) {
    (
        result.completion.done_calls,
        result.completion.success_prints,
        result.completion.failure_prints,
    )
}

#[test]
fn async_adapter_decodes_literal_header_and_rejects_all_truncations_and_bounds() {
    let literal =
        b"ERJS2\0\0\0\x02\0\0\0\x09\0\0\0assert.js\0\0\0\0\x06\0\0\0sta.js\0\0\0\0\0\0\0\0";
    let decoded = decode(literal).unwrap();
    assert_eq!(decoded.mode, 0);
    assert_eq!(decoded.policy as u8, 0);
    assert_eq!(
        decoded.harness,
        vec![
            ("assert.js".into(), "".into()),
            ("sta.js".into(), "".into())
        ]
    );
    assert!(decoded.source.is_empty());
    for cut in 0..literal.len() {
        assert!(decode(&literal[..cut]).is_err(), "cut {cut}");
    }
    for (at, value) in [
        (5, 2),
        (5, 3),
        (5, 255),
        (6, 1),
        (6, 2),
        (7, 2),
        (7, 255),
        (8, 33),
    ] {
        let mut bad = literal.to_vec();
        bad[at] = value;
        assert!(decode(&bad).is_err(), "header {at}={value}");
    }
    let mut trailing = literal.to_vec();
    trailing.push(0);
    assert!(decode(&trailing).is_err());
    let mut invalid_utf8 = literal.to_vec();
    invalid_utf8[16] = 255;
    assert!(decode(&invalid_utf8).is_err());
    let mut bad_length = literal.to_vec();
    bad_length[12..16].copy_from_slice(&257u32.to_le_bytes());
    assert!(decode(&bad_length).is_err());
    let mut oversized_source = literal[..literal.len() - 4].to_vec();
    oversized_source.extend_from_slice(&((super::super::MAX_SOURCE + 1) as u32).to_le_bytes());
    assert!(decode(&oversized_source).is_err());
    let oversized_request = vec![0; super::super::MAX_INPUT + 1];
    assert!(decode(&oversized_request).is_err());

    // Complete otherwise-valid exact/one-past frames isolate each cap from
    // truncation, invalid magic, helper authentication and other bounds.
    let name256 = "n".repeat(256);
    let name257 = "n".repeat(257);
    for (name, valid) in [(&name256, true), (&name257, false)] {
        let harness = [("assert.js", ""), ("sta.js", ""), (name.as_str(), "")];
        assert_eq!(decode(&frame(0, 0, 0, &harness, "")).is_ok(), valid);
    }
    let names: Vec<String> = (0..31).map(|i| format!("include{i}.js")).collect();
    let mut harness = vec![("assert.js", ""), ("sta.js", "")];
    for name in &names[..30] {
        harness.push((name.as_str(), ""));
    }
    assert_eq!(harness.len(), 32);
    assert!(decode(&frame(0, 0, 0, &harness, "")).is_ok());
    harness.push((names[30].as_str(), ""));
    assert_eq!(harness.len(), 33);
    assert!(decode(&frame(0, 0, 0, &harness, "")).is_err());

    let source_exact = " ".repeat(super::super::MAX_SOURCE);
    let source_over = " ".repeat(super::super::MAX_SOURCE + 1);
    let minimal = [("assert.js", ""), ("sta.js", "")];
    assert!(decode(&frame(0, 0, 0, &minimal, &source_exact)).is_ok());
    assert!(decode(&frame(0, 0, 0, &minimal, &source_over)).is_err());

    let overhead = frame(
        0,
        0,
        0,
        &[
            ("assert.js", ""),
            ("sta.js", ""),
            ("a.js", ""),
            ("b.js", ""),
        ],
        "",
    )
    .len();
    let second_len = super::super::MAX_INPUT - overhead - source_exact.len();
    assert!(second_len < super::super::MAX_SOURCE);
    for (extra, valid) in [(0, true), (1, false)] {
        let second = " ".repeat(second_len + extra);
        let harness = [
            ("assert.js", ""),
            ("sta.js", ""),
            ("a.js", source_exact.as_str()),
            ("b.js", second.as_str()),
        ];
        let complete = frame(0, 0, 0, &harness, "");
        assert_eq!(complete.len(), super::super::MAX_INPUT + extra);
        assert_eq!(decode(&complete).is_ok(), valid);
    }
}

#[test]
fn async_adapter_authenticates_unique_ordered_harness_and_exact_helper_bytes() {
    assert_eq!(PINNED_DONE.len(), 625);
    let good = [
        ("assert.js", ""),
        ("sta.js", ""),
        ("doneprintHandle.js", PINNED_DONE),
    ];
    assert!(decode(&frame(0, 0, 1, &good, "")).is_ok());
    for bad in [
        vec![
            ("sta.js", ""),
            ("assert.js", ""),
            ("doneprintHandle.js", PINNED_DONE),
        ],
        vec![
            ("assert.js", ""),
            ("doneprintHandle.js", PINNED_DONE),
            ("sta.js", ""),
        ],
        vec![("assert.js", ""), ("sta.js", "")],
        vec![
            ("assert.js", ""),
            ("sta.js", ""),
            ("doneprintHandle.js", PINNED_DONE),
            ("sta.js", ""),
        ],
        vec![
            ("assert.js", ""),
            ("sta.js", ""),
            ("doneprintHandle.js", PINNED_DONE),
            ("x.js", ""),
            ("x.js", ""),
        ],
    ] {
        assert!(decode(&frame(0, 0, 1, &bad, "")).is_err());
    }
    for altered in [
        PINNED_DONE.trim_end().to_owned(),
        format!("{PINNED_DONE}\n"),
        PINNED_DONE.replacen("2017", "2018", 1),
    ] {
        let bad = [
            ("assert.js", ""),
            ("sta.js", ""),
            ("doneprintHandle.js", altered.as_str()),
        ];
        assert!(decode(&frame(0, 0, 1, &bad, "")).is_err());
    }
    assert!(decode(&frame(0, 0, 0, &good, "")).is_err());
    let mut later = good.to_vec();
    later.push(("later.js", "var afterHelper = 1;"));
    assert_eq!(
        decode(&frame(1, 0, 1, &later, "")).unwrap().harness[3].0,
        "later.js"
    );
}

#[test]
fn async_adapter_parses_body_before_date_or_runtime_and_preserves_setup_failure() {
    for (mode, source) in [(0, "var ="), (1, "var eval;")] {
        let out = evaluate_with_runtime(
            request(
                mode,
                1,
                &[("bad.js", "throw new Error('must not execute');")],
                source,
            ),
            || panic!("early parse must precede Date capture"),
            |_| panic!("early parse must precede Runtime construction"),
        )
        .unwrap();
        assert_eq!(
            (out.outcome.status, out.outcome.phase),
            ("exception", "parse")
        );
        assert_eq!(out.outcome.error_identity, "SyntaxError");
        assert_eq!(counts(&out), (0, 0, 0));
        assert_eq!(out.completion.flags, 0);
        assert!(!out.completion.quiescent && !out.completion.audited);
        assert_eq!(out.completion.unhandled, 0);
    }
    let err = evaluate_with_runtime(
        request(0, 0, &[], "1;"),
        || Err("missing zone".into()),
        |_| panic!("capture failed"),
    );
    assert!(err.err().unwrap().contains("missing zone"));
    let err = evaluate_with_runtime(
        request(0, 0, &[], "throw 1;"),
        || Ok(eris::date_host::DateHost::unconfigured()),
        |_| Err(Runtime::parse_only("var =").unwrap_err()),
    );
    assert!(
        err.err()
            .unwrap()
            .contains("script runtime initialization:")
    );
}

#[test]
fn async_adapter_encodes_exact_eight_field_twelve_byte_tail() {
    let out = HostOutcome {
        outcome: super::super::Outcome {
            status: "complete",
            phase: "runtime",
            error_type: String::new(),
            error_identity: String::new(),
            message: String::new(),
            harness: String::new(),
        },
        completion: Test262Observation {
            policy: Test262Policy::AsyncDone,
            done_calls: 1,
            success_prints: 1,
            failure_prints: 0,
            flags: 0,
            quiescent: true,
            audited: true,
            unhandled: 0,
        },
    };
    let expected = b"ERJR3\x08\0\0\0complete\x07\0\0\0runtime\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x01\x01\x01\0\0\0\x01\x01\0\0\0\0";
    assert_eq!(encode(&out).unwrap(), expected);
    let mut out = out;
    out.outcome.message = "x".repeat(16385);
    assert!(encode(&out).is_err());
}

#[test]
fn async_adapter_completes_sync_and_async_only_after_fifo_and_final_audit() {
    for mode in [0, 1] {
        let sync = evaluate(request(
            mode,
            0,
            &[],
            "new Promise(function () {}); Promise.resolve(1).then(function () {});",
        ));
        assert_eq!(
            (sync.outcome.status, sync.outcome.phase),
            ("complete", "runtime")
        );
        assert_eq!(counts(&sync), (0, 0, 0));
        assert_eq!(sync.completion.flags, 0);
        assert!(sync.completion.quiescent && sync.completion.audited);
        assert_eq!(sync.completion.unhandled, 0);
        let out = evaluate(request(
            mode,
            1,
            &[],
            "Promise.resolve(1).then(function () { $DONE(); });",
        ));
        assert_eq!(
            (out.outcome.status, out.outcome.phase),
            ("complete", "runtime")
        );
        assert_eq!(counts(&out), (1, 1, 0));
        assert_eq!(out.completion.flags, 0);
        assert!(out.completion.quiescent && out.completion.audited);
        assert_eq!(out.completion.unhandled, 0);
    }
}

#[test]
fn async_adapter_binds_before_includes_and_accepts_saved_genuine_alias() {
    let includes = [(
        "save.js",
        "var savedDone = $DONE; $DONE = function () {}; var phase = 'include';",
    )];
    let out = evaluate(request(
        0,
        1,
        &includes,
        "if (phase !== 'include') throw 1; savedDone.call(null);",
    ));
    assert_eq!(out.outcome.status, "complete");
    assert_eq!(counts(&out), (1, 1, 0));
    assert_eq!(out.completion.flags, 0);
    let fake = evaluate(request(0, 1, &includes, "$DONE();"));
    assert_eq!(
        (
            fake.outcome.status,
            fake.outcome.phase,
            fake.outcome.message.as_str()
        ),
        ("host-failure", "host", "missing-completion")
    );
    assert_eq!(counts(&fake), (0, 0, 0));
}

#[test]
fn async_adapter_private_completion_flags_prevent_forged_or_incomplete_pass() {
    for (source, reason) in [
        ("1;", "missing-completion"),
        ("$DONE(); $DONE();", "duplicate-completion"),
        (
            "print('Test262:AsyncTestComplete');",
            "unauthenticated-completion",
        ),
        (
            "print = function () {}; $DONE();",
            "completion-protocol-error",
        ),
        ("$DONE(new Error('bad'));", "completion-error"),
        (
            "$DONE(); queueMicrotask(function () { throw new Error('late'); });",
            "queued-exception",
        ),
        ("$DONE(); Promise.reject(7);", "unhandled-rejection"),
    ] {
        let out = evaluate(request(0, 1, &[], source));
        assert_eq!(
            (
                out.outcome.status,
                out.outcome.phase,
                out.outcome.message.as_str()
            ),
            ("host-failure", "host", reason),
            "{source}"
        );
    }
    let handled = evaluate(request(
        0,
        1,
        &[],
        "var p = Promise.reject(7); queueMicrotask(function () { p.catch(function () {}); }); $DONE();",
    ));
    assert_eq!(handled.outcome.status, "complete");
    assert_eq!(handled.completion.unhandled, 0);
    let sync = evaluate(request(0, 0, &[], "Promise.reject(7);"));
    assert_eq!(
        (sync.outcome.status, sync.outcome.message.as_str()),
        ("host-failure", "unhandled-rejection")
    );
}

#[test]
fn async_adapter_keeps_actual_body_errors_resource_failures_and_harness_phase() {
    let body = evaluate(request(0, 1, &[], "throw new TypeError('body');"));
    assert_eq!(
        (
            body.outcome.status,
            body.outcome.phase,
            body.outcome.error_identity.as_str()
        ),
        ("exception", "runtime", "TypeError")
    );
    assert_ne!(body.completion.flags & 0x100, 0);
    let fatal = evaluate(request(0, 1, &[], "$DONE(); while (true) {}"));
    assert_eq!(
        (fatal.outcome.status, fatal.outcome.phase),
        ("resource", "runtime")
    );
    assert_ne!(fatal.completion.flags & 8, 0);
    assert!(!fatal.completion.audited);
    assert_eq!(counts(&fatal), (1, 1, 0));
    let harness = evaluate(request(
        0,
        1,
        &[("broken.js", "throw new TypeError('harness');")],
        "$DONE();",
    ));
    assert_eq!(
        (
            harness.outcome.status,
            harness.outcome.phase,
            harness.outcome.harness.as_str()
        ),
        ("harness-error", "harness", "broken.js")
    );
    assert_eq!(harness.outcome.error_identity, "TypeError");
}

#[test]
fn async_adapter_stops_on_harness_observations_before_later_entry_can_reset_work() {
    for (first, reason) in [
        ("$DONE();", "premature-completion"),
        (
            "queueMicrotask(function () { throw new Error('harness job'); });",
            "queued-exception",
        ),
    ] {
        let out = evaluate(request(
            0,
            1,
            &[("first.js", first), ("never.js", "while (true) {}")],
            "$DONE();",
        ));
        assert_eq!(
            (
                out.outcome.status,
                out.outcome.phase,
                out.outcome.harness.as_str(),
                out.outcome.message.as_str()
            ),
            ("host-failure", "host", "first.js", reason)
        );
        assert!(!out.completion.audited);
    }
}

#[test]
fn async_adapter_dynamic_function_syntax_error_is_runtime_after_successful_preparse() {
    for mode in [0, 1] {
        let out = evaluate(request(mode, 1, &[], "Function('return (');"));
        assert_eq!(
            (out.outcome.status, out.outcome.phase),
            ("exception", "runtime")
        );
        assert_eq!(out.outcome.error_type, "SyntaxError");
        assert_eq!(out.outcome.error_identity, "SyntaxError");
        assert!(out.outcome.harness.is_empty());
        assert_eq!(counts(&out), (0, 0, 0));
        assert_eq!(out.completion.flags, 0x100);
        assert!(out.completion.quiescent && out.completion.audited);
        assert_eq!(out.completion.unhandled, 0);
    }
}
