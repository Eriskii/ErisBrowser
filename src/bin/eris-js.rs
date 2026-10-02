//! Bounded Test262 transport adapter for Eris's own interpreter.
//! Each invocation receives one test variant and unchanged harness sources.
use eris::{
    dom::Document,
    script::{Runtime, ScriptError},
};
use std::io::{self, Read, Write};

const MAX_INPUT: usize = 4 * 1024 * 1024;
const MAX_SOURCE: usize = 2 * 1024 * 1024;

struct Request {
    mode: u8,
    parse_only: bool,
    harness: Vec<(String, String)>,
    source: String,
}

#[derive(Debug, PartialEq)]
struct Outcome {
    status: &'static str,
    phase: &'static str,
    error_type: String,
    error_identity: String,
    message: String,
    harness: String,
}

fn main() {
    #[cfg(all(target_os = "linux", feature = "vulkan-presenter"))]
    if std::env::args().nth(1).as_deref() == Some("--clean-worker-launch") {
        let result = match (std::env::args().nth(2), std::env::args().nth(3)) {
            (Some(role), None) if role == "--timezone-discovery" => {
                eris::worker::launch_worker(&role)
            }
            _ => Err("invalid adapter helper role".into()),
        };
        match result {
            Ok(never) => match never {},
            Err(error) => {
                eprintln!("eris-js: {error}");
                std::process::exit(2);
            }
        }
    }
    if std::env::args().nth(1).as_deref() == Some("--timezone-discovery") {
        if std::env::args().len() != 2 || eris::date_host::serve_timezone_discovery().is_err() {
            std::process::exit(2);
        }
        return;
    }
    if let Err(error) = run() {
        eprintln!("eris-js: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    if std::env::args().len() != 1 {
        return Err("eris-js reads a single ERJS1 framed test request on stdin".into());
    }
    #[cfg(target_os = "linux")]
    for (resource, limit) in [
        (rustix::process::Resource::As, 512 * 1024 * 1024),
        (rustix::process::Resource::Core, 0),
    ] {
        let old = rustix::process::getrlimit(resource);
        let cap = old.maximum.unwrap_or(u64::MAX).min(limit);
        rustix::process::setrlimit(
            resource,
            rustix::process::Rlimit {
                current: Some(cap),
                maximum: Some(cap),
            },
        )
        .map_err(|error| error.to_string())?;
    }
    let mut input = Vec::new();
    io::stdin()
        .take(MAX_INPUT as u64 + 1)
        .read_to_end(&mut input)
        .map_err(|error| error.to_string())?;
    if input.len() > MAX_INPUT {
        return Err("request exceeds adapter input budget".into());
    }
    let request = decode(&input)?;
    let outcome = evaluate_with_host(request, || {
        let mut discovery = eris::date_host::ZoneDiscoverySupervisor::new();
        discovery
            .capture(
                &std::env::current_exe().map_err(|e| e.to_string())?,
                eris::date_host::ZoneDiscoveryConfig::from_environment()
                    .map_err(|e| e.to_string())?,
                || false,
            )
            .map(|captured| captured.date_host())
            .map_err(|e| e.to_string())
    });
    if outcome.status == "adapter-error" {
        return Err(outcome.message);
    }
    let mut output = b"ERJR2".to_vec();
    for value in [
        outcome.status,
        outcome.phase,
        &outcome.error_type,
        &outcome.error_identity,
        &outcome.message,
        &outcome.harness,
    ] {
        if value.len() > 16384 {
            return Err("response field exceeds adapter budget".into());
        }
        output.extend_from_slice(&(value.len() as u32).to_le_bytes());
        output.extend_from_slice(value.as_bytes());
    }
    io::stdout()
        .write_all(&output)
        .map_err(|error| error.to_string())
}

fn decode(input: &[u8]) -> Result<Request, String> {
    let mut reader = Reader { input, offset: 0 };
    if reader.take(5)? != b"ERJS1" {
        return Err("invalid test request magic".into());
    }
    let mode = reader.take(1)?[0];
    if mode > 3 {
        return Err("invalid test mode".into());
    }
    let parse_only = match reader.take(1)?[0] {
        0 => false,
        1 => true,
        _ => return Err("invalid parse-only flag".into()),
    };
    let count = reader.u32()? as usize;
    if count > 32 || mode == 2 && count != 0 {
        return Err("invalid harness count or harness supplied for raw mode".into());
    }
    let mut harness = Vec::new();
    for _ in 0..count {
        harness.push((reader.string(256)?, reader.string(MAX_SOURCE)?));
    }
    let source = reader.string(MAX_SOURCE)?;
    if reader.offset != input.len() {
        return Err("trailing test request data".into());
    }
    Ok(Request {
        mode,
        parse_only,
        harness,
        source,
    })
}

struct Reader<'a> {
    input: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], String> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or("request length overflow")?;
        let value = self
            .input
            .get(self.offset..end)
            .ok_or("truncated test request")?;
        self.offset = end;
        Ok(value)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| "invalid length")?,
        ))
    }
    fn string(&mut self, cap: usize) -> Result<String, String> {
        let length = self.u32()? as usize;
        if length > cap {
            return Err("request string exceeds limit".into());
        }
        String::from_utf8(self.take(length)?.to_vec())
            .map_err(|_| "request source is not UTF-8".into())
    }
}

fn error_outcome(error: ScriptError, phase: &'static str, harness: &str) -> Outcome {
    let status = if error.is_resource_limit() {
        "resource"
    } else if !harness.is_empty() {
        "harness-error"
    } else if error.is_unsupported() {
        "unsupported"
    } else {
        "exception"
    };
    Outcome {
        status,
        phase,
        error_type: error.name().into(),
        error_identity: error.intrinsic_error_name().unwrap_or("").into(),
        message: error.message.chars().take(4096).collect(),
        harness: harness.into(),
    }
}

#[cfg(test)]
fn evaluate(request: Request) -> Outcome {
    evaluate_with_host(request, || Ok(eris::date_host::DateHost::unconfigured()))
}

fn evaluate_with_host(
    request: Request,
    capture: impl FnOnce() -> Result<eris::date_host::DateHost, String>,
) -> Outcome {
    evaluate_with_runtime(request, capture, Runtime::try_with_date_host)
}

fn evaluate_with_runtime(
    request: Request,
    capture: impl FnOnce() -> Result<eris::date_host::DateHost, String>,
    initialize: impl FnOnce(eris::date_host::DateHost) -> Result<Runtime, ScriptError>,
) -> Outcome {
    if request.mode == 3 {
        return Outcome {
            status: "unsupported",
            phase: "mode",
            error_type: String::new(),
            error_identity: String::new(),
            message: "module execution is not implemented".into(),
            harness: String::new(),
        };
    }
    // Parse source before evaluating anything. A runtime SyntaxError cannot be
    // confused with an early error, and parse-negative tests are never run.
    let parse = if request.mode == 1 {
        Runtime::parse_only_strict(&request.source)
    } else {
        Runtime::parse_only(&request.source)
    };
    if let Err(error) = parse {
        return error_outcome(error, "parse", "");
    }
    if request.parse_only {
        return Outcome {
            status: "complete",
            phase: "parse",
            error_type: String::new(),
            error_identity: String::new(),
            message: String::new(),
            harness: String::new(),
        };
    }
    let date_host = match capture() {
        Ok(host) => host,
        Err(error) => {
            return Outcome {
                status: "adapter-error",
                phase: "initialization",
                error_type: String::new(),
                error_identity: String::new(),
                message: format!("Date host initialization: {error}"),
                harness: String::new(),
            };
        }
    };
    let mut runtime = match initialize(date_host) {
        Ok(runtime) => runtime,
        Err(error) => {
            return Outcome {
                status: "adapter-error",
                phase: "initialization",
                error_type: String::new(),
                error_identity: String::new(),
                message: format!("script runtime initialization: {error}"),
                harness: String::new(),
            };
        }
    };
    let mut document = Document::parse("");
    for (name, source) in request.harness {
        if let Err(error) = runtime.execute(&source, &mut document) {
            return error_outcome(error, "harness", &name);
        }
    }
    let completion = if request.mode == 1 {
        runtime.execute_strict(&request.source, &mut document)
    } else {
        runtime.execute(&request.source, &mut document)
    };
    match completion {
        Ok(_) => Outcome {
            status: "complete",
            phase: "runtime",
            error_type: String::new(),
            error_identity: String::new(),
            message: String::new(),
            harness: String::new(),
        },
        Err(error) => {
            let phase = if error.is_parse_error() {
                "parse"
            } else {
                "runtime"
            };
            error_outcome(error, phase, "")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_initialization_failure_is_not_an_author_or_harness_exception() {
        let mut test = request("throw new SyntaxError('author');");
        test.harness
            .push(("setup.js".into(), "throw 'harness';".into()));
        let outcome = evaluate_with_runtime(
            test,
            || Ok(eris::date_host::DateHost::unconfigured()),
            |_| Err(Runtime::parse_only("var =").unwrap_err()),
        );
        assert_eq!(outcome.status, "adapter-error");
        assert_eq!(outcome.phase, "initialization");
        assert!(outcome.error_type.is_empty());
        assert!(outcome.error_identity.is_empty());
        assert!(outcome.harness.is_empty());
        assert!(
            outcome
                .message
                .starts_with("script runtime initialization:")
        );
    }

    #[test]
    fn adapter_parse_only_and_early_errors_never_construct_a_runtime() {
        let mut parse_only = request("throw 'never execute';");
        parse_only.parse_only = true;
        for (test, status) in [(parse_only, "complete"), (request("var ="), "exception")] {
            let outcome = evaluate_with_runtime(
                test,
                || panic!("parse-only and early errors precede host capture"),
                |_| panic!("parse-only and early errors precede realm construction"),
            );
            assert_eq!(outcome.status, status);
            assert_eq!(outcome.phase, "parse");
        }
        let outcome = evaluate_with_runtime(
            request("1;"),
            || Err("missing host".into()),
            |_| panic!("a failed host capture must not construct a realm"),
        );
        assert_eq!(outcome.status, "adapter-error");
        assert_eq!(outcome.phase, "initialization");
        assert!(outcome.message.contains("missing host"));
    }

    #[test]
    fn adapter_parses_before_host_capture_and_reports_initialization_failure() {
        let mut parse = request("Date.now()");
        parse.parse_only = true;
        assert_eq!(
            evaluate_with_host(parse, || panic!("parse-only must not capture a zone")).status,
            "complete"
        );
        assert_eq!(
            evaluate_with_host(request("var ="), || panic!(
                "early errors must precede capture"
            ))
            .phase,
            "parse"
        );
        let outcome = evaluate_with_host(request("throw 'author code must not run'"), || {
            Err("missing rules".into())
        });
        assert_eq!(outcome.status, "adapter-error");
        assert_eq!(outcome.phase, "initialization");
        assert!(outcome.message.contains("missing rules"));
    }

    fn request(source: &str) -> Request {
        Request {
            mode: 0,
            parse_only: false,
            harness: Vec::new(),
            source: source.into(),
        }
    }

    #[test]
    fn parse_negatives_never_execute_and_harness_errors_have_their_own_phase() {
        let mut negative = request("throw 1;");
        negative.parse_only = true;
        assert_eq!(evaluate(negative).status, "complete");
        let mut test = request("throw 1;");
        test.harness.push(("broken.js".into(), "throw 2;".into()));
        let outcome = evaluate(test);
        assert_eq!(outcome.status, "harness-error");
        assert_eq!(outcome.phase, "harness");
        assert_eq!(outcome.harness, "broken.js");
    }

    #[test]
    fn harness_shares_the_test_realm_and_cases_use_fresh_realms() {
        let mut test = request("if (prepared !== 42) throw 1;");
        test.harness
            .push(("setup.js".into(), "var prepared = 42;".into()));
        assert_eq!(evaluate(test).status, "complete");
        assert_eq!(evaluate(request("prepared;")).error_type, "ReferenceError");
    }

    #[test]
    fn strict_mode_changes_early_errors_and_runtime_receivers_without_changing_harness_mode() {
        let mut test = request(
            "function f(){return this;}if(f()!==undefined)throw 1; if(loose()!==globalThis)throw 2;",
        );
        test.mode = 1;
        test.harness
            .push(("setup.js".into(), "function loose(){return this;}".into()));
        assert_eq!(evaluate(test).status, "complete");
        let mut early = request("var eval;");
        early.mode = 1;
        let early = evaluate(early);
        assert_eq!(
            (early.status, early.phase, early.error_identity.as_str()),
            ("exception", "parse", "SyntaxError")
        );
        let mut write = request("missing=1;");
        write.mode = 1;
        let write = evaluate(write);
        assert_eq!(
            (write.status, write.phase, write.error_identity.as_str()),
            ("exception", "runtime", "ReferenceError")
        );
        let mut module = request("");
        module.mode = 3;
        assert_eq!(evaluate(module).status, "unsupported");
    }

    #[test]
    fn runtime_syntax_errors_are_distinct_from_early_errors_and_resource_limits() {
        let early = evaluate(request("var = ;"));
        assert_eq!(early.status, "exception");
        assert_eq!(early.phase, "parse");
        assert_eq!(early.error_type, "SyntaxError");
        let runtime = evaluate(request("throw new SyntaxError('runtime');"));
        assert_eq!(runtime.status, "exception");
        assert_eq!(runtime.phase, "runtime");
        assert_eq!(runtime.error_type, "SyntaxError");
        assert_eq!(runtime.error_identity, "SyntaxError");
        let resource = evaluate(request("while (true) {}"));
        assert_eq!(resource.status, "resource");
        assert_eq!(resource.phase, "runtime");
    }

    #[test]
    fn diagnostic_names_cannot_spoof_intrinsic_error_identity() {
        let fake = evaluate(request(
            "function Fake() {} Object.defineProperty(Fake, 'name', {value: 'TypeError'}); throw new Fake();",
        ));
        assert_eq!(fake.error_type, "TypeError");
        assert_eq!(fake.error_identity, "");
        let shaped = evaluate(request("throw {constructor: TypeError};"));
        assert_eq!(shaped.error_identity, "");
        let renamed = evaluate(request(
            "Object.defineProperty(TypeError, 'name', {value: 'Renamed'}); throw new TypeError();",
        ));
        assert_eq!(renamed.error_type, "Renamed");
        assert_eq!(renamed.error_identity, "TypeError");
    }

    #[test]
    fn malformed_lengths_raw_harness_and_trailing_input_are_rejected() {
        assert!(decode(b"ERJS1\0\0\0\0\0\0\xff\xff\xff\xff").is_err());
        assert!(decode(b"ERJS1\x02\0\x01\0\0\0").is_err());
        assert!(decode(b"ERJS1\0\0\0\0\0\0\0\0\0\0x").is_err());
    }
}
