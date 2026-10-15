//! Opt-in authenticated completion framing for one bounded Test262 variant.
use super::{
    Document, MAX_INPUT, MAX_SOURCE, Outcome, Reader, Runtime, ScriptError, error_outcome,
};
use eris::script::{Test262Observation, Test262Policy};
use std::io::{self, Write};

const PINNED_DONE: &[u8] = include_bytes!(
    "../../../tests/upstream/test262-promise-methods-async/harness/doneprintHandle.js"
);

struct HostRequest {
    mode: u8,
    policy: Test262Policy,
    harness: Vec<(String, String)>,
    source: String,
}

struct HostOutcome {
    outcome: Outcome,
    completion: Test262Observation,
}

pub(super) fn run(input: &[u8]) -> Result<(), String> {
    let request = decode(input)?;
    let result = evaluate_with_runtime(
        request,
        || {
            let mut discovery = eris::date_host::ZoneDiscoverySupervisor::new();
            discovery
                .capture(
                    &std::env::current_exe().map_err(|error| error.to_string())?,
                    eris::date_host::ZoneDiscoveryConfig::from_environment()
                        .map_err(|error| error.to_string())?,
                    || false,
                )
                .map(|captured| captured.date_host())
                .map_err(|error| error.to_string())
        },
        Runtime::try_with_date_host,
    )?;
    io::stdout()
        .write_all(&encode(&result)?)
        .map_err(|error| error.to_string())
}

fn decode(input: &[u8]) -> Result<HostRequest, String> {
    if input.len() > MAX_INPUT {
        return Err("request exceeds adapter input budget".into());
    }
    let mut reader = Reader { input, offset: 0 };
    if reader.take(5)? != b"ERJS2" {
        return Err("invalid authenticated test request magic".into());
    }
    let mode = reader.take(1)?[0];
    if mode > 1 {
        return Err("authenticated request needs sloppy or strict mode".into());
    }
    if reader.take(1)?[0] != 0 {
        return Err("authenticated request cannot be parse-only".into());
    }
    let policy = match reader.take(1)?[0] {
        0 => Test262Policy::CheckedSync,
        1 => Test262Policy::AsyncDone,
        _ => return Err("invalid completion policy".into()),
    };
    let count = reader.u32()? as usize;
    if count > 32 {
        return Err("invalid authenticated harness count".into());
    }
    let mut harness = Vec::new();
    for _ in 0..count {
        harness.push((reader.string(256)?, reader.string(MAX_SOURCE)?));
    }
    let source = reader.string(MAX_SOURCE)?;
    if reader.offset != input.len() {
        return Err("trailing authenticated request data".into());
    }
    // At most32 names of at most256 bytes; no author callbacks or hash authority.
    for (index, (name, _)) in harness.iter().enumerate() {
        if harness[..index].iter().any(|(earlier, _)| earlier == name) {
            return Err("duplicate authenticated harness name".into());
        }
    }
    if harness.first().map(|item| item.0.as_str()) != Some("assert.js")
        || harness.get(1).map(|item| item.0.as_str()) != Some("sta.js")
    {
        return Err("authenticated harness prefix must be assert.js then sta.js".into());
    }
    match policy {
        Test262Policy::CheckedSync => {
            if harness.iter().any(|(name, _)| name == "doneprintHandle.js") {
                return Err("checked-sync request cannot contain doneprintHandle.js".into());
            }
        }
        Test262Policy::AsyncDone => {
            let Some((name, helper)) = harness.get(2) else {
                return Err("authenticated async request is missing its helper".into());
            };
            if name != "doneprintHandle.js" || helper.as_bytes() != PINNED_DONE {
                return Err("authenticated async helper bytes or position differ".into());
            }
        }
    }
    Ok(HostRequest {
        mode,
        policy,
        harness,
        source,
    })
}

fn initial_observation(policy: Test262Policy) -> Test262Observation {
    Test262Observation {
        policy,
        done_calls: 0,
        success_prints: 0,
        failure_prints: 0,
        flags: 0,
        quiescent: false,
        audited: false,
        unhandled: 0,
    }
}

fn observation(runtime: &Runtime) -> Result<Test262Observation, String> {
    runtime
        .test262_host_observation()
        .ok_or_else(|| "missing private Test262 session".into())
}

fn host_failure(completion: Test262Observation, harness: &str) -> HostOutcome {
    HostOutcome {
        outcome: Outcome {
            status: "host-failure",
            phase: "host",
            error_type: String::new(),
            error_identity: String::new(),
            message: host_reason(completion).into(),
            harness: harness.into(),
        },
        completion,
    }
}

fn passes(completion: Test262Observation) -> bool {
    let expected = match completion.policy {
        Test262Policy::CheckedSync => (0, 0, 0),
        Test262Policy::AsyncDone => (1, 1, 0),
    };
    completion.flags == 0
        && completion.quiescent
        && completion.audited
        && completion.unhandled == 0
        && (
            completion.done_calls,
            completion.success_prints,
            completion.failure_prints,
        ) == expected
}

fn host_reason(completion: Test262Observation) -> &'static str {
    for (flag, reason) in [
        (8, "fatal-observation"),
        (2, "premature-completion"),
        (4, "queued-exception"),
        (1, "completion-error"),
        (64, "delegate-abrupt"),
        (128, "session-violation"),
        (16, "unauthenticated-completion"),
        (32, "invalid-completion-print"),
    ] {
        if completion.flags & flag != 0 {
            return reason;
        }
    }
    if !completion.quiescent {
        return "nonquiescent-runtime";
    }
    if !completion.audited {
        return "unaudited-runtime";
    }
    if completion.unhandled != 0 {
        return "unhandled-rejection";
    }
    if completion.policy == Test262Policy::AsyncDone {
        if completion.done_calls > 1 {
            return "duplicate-completion";
        }
        if completion.done_calls == 0 {
            return "missing-completion";
        }
    }
    "completion-protocol-error"
}

fn evaluate_with_runtime(
    request: HostRequest,
    capture: impl FnOnce() -> Result<eris::date_host::DateHost, String>,
    initialize: impl FnOnce(eris::date_host::DateHost) -> Result<Runtime, ScriptError>,
) -> Result<HostOutcome, String> {
    // decode has authenticated the helper before this function is reachable.
    // Body syntax errors precede Date capture, realm construction and harnesses.
    let parse = if request.mode == 1 {
        Runtime::parse_only_strict(&request.source)
    } else {
        Runtime::parse_only(&request.source)
    };
    if let Err(error) = parse {
        return Ok(HostOutcome {
            outcome: error_outcome(error, "parse", ""),
            completion: initial_observation(request.policy),
        });
    }
    let date_host = capture().map_err(|error| format!("Date host initialization: {error}"))?;
    let mut runtime =
        initialize(date_host).map_err(|error| format!("script runtime initialization: {error}"))?;
    runtime
        .enable_test262_host(request.policy)
        .map_err(|error| format!("Test262 host initialization: {error}"))?;
    let mut document = Document::parse("");
    for (index, (name, source)) in request.harness.into_iter().enumerate() {
        if let Err(error) = runtime.execute(&source, &mut document) {
            return Ok(HostOutcome {
                outcome: error_outcome(error, "harness", &name),
                completion: observation(&runtime)?,
            });
        }
        // Stop before any later execute can reset an ordinary entry's counters.
        let current = observation(&runtime)?;
        if current.flags != 0 {
            return Ok(host_failure(current, &name));
        }
        if request.policy == Test262Policy::AsyncDone && index == 2 {
            if let Err(error) = runtime.bind_test262_done_from_harness() {
                return Ok(HostOutcome {
                    outcome: error_outcome(error, "harness", &name),
                    completion: observation(&runtime)?,
                });
            }
            let current = observation(&runtime)?;
            if current.flags != 0 {
                return Ok(host_failure(current, &name));
            }
        }
    }
    if let Err(error) = runtime.arm_test262_body() {
        return Ok(HostOutcome {
            outcome: error_outcome(error, "host", ""),
            completion: observation(&runtime)?,
        });
    }
    let body = if request.mode == 1 {
        runtime.execute_strict(&request.source, &mut document)
    } else {
        runtime.execute(&request.source, &mut document)
    };
    // Exactly one final audit, using the body's remaining work/heap. Even when
    // admission refuses, the constant-size snapshot retains failure evidence.
    let finished = runtime.finish_test262_body(&body);
    let completion = match &finished {
        Ok(audited) => *audited,
        Err(_) => observation(&runtime)?,
    };
    if let Err(error) = body {
        return Ok(HostOutcome {
            outcome: error_outcome(error, "runtime", ""),
            completion,
        });
    }
    if let Err(error) = finished {
        return Ok(HostOutcome {
            outcome: error_outcome(error, "runtime", ""),
            completion,
        });
    }
    if !passes(completion) {
        return Ok(host_failure(completion, ""));
    }
    Ok(HostOutcome {
        outcome: Outcome {
            status: "complete",
            phase: "runtime",
            error_type: String::new(),
            error_identity: String::new(),
            message: String::new(),
            harness: String::new(),
        },
        completion,
    })
}

fn encode(result: &HostOutcome) -> Result<Vec<u8>, String> {
    let outcome = &result.outcome;
    let mut output = b"ERJR3".to_vec();
    for field in [
        outcome.status,
        outcome.phase,
        &outcome.error_type,
        &outcome.error_identity,
        &outcome.message,
        &outcome.harness,
    ] {
        if field.len() > 16384 {
            return Err("response field exceeds adapter budget".into());
        }
        output.extend_from_slice(&(field.len() as u32).to_le_bytes());
        output.extend_from_slice(field.as_bytes());
    }
    let e = result.completion;
    output.extend_from_slice(&[
        e.policy as u8,
        e.done_calls,
        e.success_prints,
        e.failure_prints,
    ]);
    output.extend_from_slice(&e.flags.to_le_bytes());
    output.extend_from_slice(&[u8::from(e.quiescent), u8::from(e.audited)]);
    output.extend_from_slice(&e.unhandled.to_le_bytes());
    Ok(output)
}

#[cfg(test)]
#[path = "test262_host_tests.rs"]
mod tests;
