//! Bounded JSON reporting after the benchmark's surface owner is released.
//!
//! The caller proves owner release and, in check mode, actual verified-frame
//! retirement before setting `checked`. This serializer has no renderer/owner
//! access and cannot manufacture that proof. Missing phases remain JSON null.
use super::native_benchmark_identity::MAX_IDENTITY_BYTES;
use crate::presenter::{
    Target, TimingOutcome, TimingReport, TimingRoute, TimingSample, TimingSubmission,
};
use std::io::{self, Write};

const MAX_SAMPLES: usize = 128;
const MAX_ERROR_JSON_BYTES: usize = 4096;
const MAX_REPORT_BYTES: usize = 256 * 1024;
const HEX: &[u8; 16] = b"0123456789abcdef";

pub(super) struct Context<'a> {
    pub check: bool,
    pub route: TimingRoute,
    pub requested: u8,
    pub completed: u8,
    pub scene: &'a [u8],
    pub target: Target,
    pub scale_factor: f64,
    pub source_load_ms: Option<f64>,
    pub edit_sequence: u64,
    pub diagnostics: usize,
    pub checked: bool,
    pub failure: Option<&'a str>,
}

/// Stream one schema-1 JSON object and newline. All input and output-size
/// admission runs before the first output write. Failure strings are retained
/// exactly or refused; the 4096-byte limit includes their quotes and escaping.
/// An I/O failure can leave a partial artifact, which must never count as a run.
pub(super) fn write(
    output: &mut impl Write,
    context: Context<'_>,
    timing: Option<&TimingReport>,
) -> Result<(), String> {
    validate(&context, timing)?;
    // A no-storage first pass also bounds numeric formatting and all syntax.
    // The second pass uses the same immutable inputs, with a 512-byte hex stack
    // buffer; neither pass builds a JSON String or clones the sample vector.
    render(&mut Count(0), &context, timing)
        .map_err(|_| "native benchmark report exceeds output budget")?;
    render(output, &context, timing).map_err(|_| "native benchmark report output failed".into())
}

fn validate(context: &Context<'_>, timing: Option<&TimingReport>) -> Result<(), String> {
    if context.scene.len() > MAX_IDENTITY_BYTES
        || usize::from(context.requested) > MAX_SAMPLES
        || context.completed > context.requested
    {
        return Err("native benchmark report input budget".into());
    }
    if !context.scale_factor.is_finite() || context.scale_factor <= 0.0 {
        return Err("native benchmark report invalid scale factor".into());
    }
    if context.check {
        if context.requested != 1 || timing.is_some() {
            return Err("native benchmark check cannot contain timing samples".into());
        }
        if context.checked && context.completed != 1 {
            return Err("native benchmark check lacks successful retirement".into());
        }
    } else if context.checked || !(2..=128).contains(&context.requested) {
        return Err("native benchmark measurement mode mismatch".into());
    }
    if let Some(timing) = timing {
        if timing.samples.len() > MAX_SAMPLES
            || timing.requested != context.requested
            || timing.accepted > timing.requested
            || timing.samples.len() > usize::from(timing.accepted)
        {
            return Err("native benchmark report sample budget".into());
        }
        error_length(timing.failure.as_deref())?;
    }
    error_length(context.failure)?;
    Ok(())
}

fn error_length(value: Option<&str>) -> Result<(), String> {
    let Some(value) = value else { return Ok(()) };
    if value.len() > MAX_ERROR_JSON_BYTES {
        return Err("native benchmark report failure text budget".into());
    }
    let mut length = 2usize;
    for ch in value.chars() {
        let added = match ch {
            '"' | '\\' | '\u{08}' | '\u{0c}' | '\n' | '\r' | '\t' => 2,
            '\u{00}'..='\u{1f}' => 6,
            _ => ch.len_utf8(),
        };
        length = length
            .checked_add(added)
            .filter(|length| *length <= MAX_ERROR_JSON_BYTES)
            .ok_or("native benchmark report failure text budget")?;
    }
    Ok(())
}

struct Count(usize);
impl Write for Count {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_add(bytes.len())
            .filter(|length| *length <= MAX_REPORT_BYTES)
            .ok_or_else(|| io::Error::other("report size"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn route(value: TimingRoute) -> &'static str {
    match value {
        TimingRoute::CpuUpload => "cpu-upload",
        TimingRoute::NativeRaster => "native-raster",
    }
}
fn outcome(value: TimingOutcome) -> &'static str {
    match value {
        TimingOutcome::Presented => "presented",
        TimingOutcome::NotPresented => "not-presented",
        TimingOutcome::Aborted => "aborted",
        TimingOutcome::Failed => "failed",
    }
}
fn submission(value: TimingSubmission) -> &'static str {
    match value {
        TimingSubmission::None => "none",
        TimingSubmission::Draws => "draws",
        TimingSubmission::Flush => "flush",
    }
}

fn error(output: &mut impl Write, value: Option<&str>) -> io::Result<()> {
    let Some(value) = value else {
        return output.write_all(b"null");
    };
    output.write_all(b"\"")?;
    for ch in value.chars() {
        match ch {
            '"' => output.write_all(b"\\\"")?,
            '\\' => output.write_all(b"\\\\")?,
            '\u{08}' => output.write_all(b"\\b")?,
            '\u{0c}' => output.write_all(b"\\f")?,
            '\n' => output.write_all(b"\\n")?,
            '\r' => output.write_all(b"\\r")?,
            '\t' => output.write_all(b"\\t")?,
            '\u{00}'..='\u{1f}' => {
                let byte = ch as u8;
                output.write_all(&[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX[usize::from(byte >> 4)],
                    HEX[usize::from(byte & 15)],
                ])?;
            }
            _ => output.write_all(ch.encode_utf8(&mut [0; 4]).as_bytes())?,
        }
    }
    output.write_all(b"\"")
}

fn hex(output: &mut impl Write, scene: &[u8]) -> io::Result<()> {
    let mut buffer = [0u8; 512];
    for chunk in scene.chunks(buffer.len() / 2) {
        for (index, byte) in chunk.iter().enumerate() {
            buffer[index * 2] = HEX[usize::from(byte >> 4)];
            buffer[index * 2 + 1] = HEX[usize::from(byte & 15)];
        }
        output.write_all(&buffer[..chunk.len() * 2])?;
    }
    Ok(())
}

fn optional_ns(output: &mut impl Write, value: Option<u64>) -> io::Result<()> {
    match value {
        Some(value) => write!(output, "{value}"),
        None => output.write_all(b"null"),
    }
}

fn sample(output: &mut impl Write, sample: &TimingSample) -> io::Result<()> {
    write!(
        output,
        "{{\"id\":{},\"generation\":{},\"viewport_revision\":{},\"serial\":{},\"width\":{},\"height\":{},\"route\":\"{}\",\"outcome\":\"{}\",\"submission\":\"{}\",\"configured\":{},\"ui_prepare_ns\":{},\"queue_ns\":{}",
        sample.id,
        sample.stamp.generation,
        sample.stamp.viewport_revision,
        sample.stamp.serial,
        sample.size.0,
        sample.size.1,
        route(sample.route),
        outcome(sample.outcome),
        submission(sample.submission),
        sample.configured,
        sample.ui_prepare_ns,
        sample.queue_ns,
    )?;
    for (name, value) in [
        ("acquire_ns", sample.acquire_ns),
        ("encode_upload_ns", sample.encode_upload_ns),
        ("submit_ns", sample.submit_ns),
        ("completion_wait_ns", sample.completion_wait_ns),
        ("cleanup_ns", sample.cleanup_ns),
        ("present_call_ns", sample.present_call_ns),
    ] {
        write!(output, ",\"{name}\":")?;
        optional_ns(output, value)?;
    }
    write!(
        output,
        ",\"owner_total_ns\":{},\"prepare_to_present_ns\":",
        sample.owner_total_ns
    )?;
    optional_ns(output, sample.prepare_to_present_ns)?;
    output.write_all(b"}")
}

fn render(
    output: &mut impl Write,
    context: &Context<'_>,
    timing: Option<&TimingReport>,
) -> io::Result<()> {
    write!(
        output,
        "{{\"schema\":1,\"kind\":\"native-benchmark\",\"mode\":\"{}\",\"route\":\"{}\",\"requested_frames\":{},\"completed_frames\":{},\"accepted_frames\":{},\"scene_hex\":\"",
        if context.check { "check" } else { "measure" },
        route(context.route),
        context.requested,
        context.completed,
        timing.map_or(0, |report| report.accepted),
    )?;
    hex(output, context.scene)?;
    write!(
        output,
        "\",\"width\":{},\"height\":{},\"generation\":{},\"viewport_revision\":{},\"scale_factor\":{},\"source_load_ms\":",
        context.target.size.0,
        context.target.size.1,
        context.target.generation,
        context.target.viewport_revision,
        context.scale_factor,
    )?;
    match context.source_load_ms.filter(|value| value.is_finite()) {
        Some(value) => write!(output, "{value}")?,
        None => output.write_all(b"null")?,
    }
    write!(
        output,
        ",\"edit_sequence\":{},\"diagnostics\":{},\"checked\":{},\"failure\":",
        context.edit_sequence, context.diagnostics, context.checked
    )?;
    error(output, context.failure)?;
    output.write_all(b",\"owner_failure\":")?;
    error(output, timing.and_then(|report| report.failure.as_deref()))?;
    output.write_all(b",\"owner_init_ns\":")?;
    optional_ns(
        output,
        timing.and_then(|report| report.initialization.map(|init| init.owner_init_ns)),
    )?;
    output.write_all(b",\"samples\":[")?;
    if let Some(timing) = timing {
        for (index, record) in timing.samples.iter().enumerate() {
            if index != 0 {
                output.write_all(b",")?;
            }
            sample(output, record)?;
        }
    }
    output.write_all(b"]}\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presenter::{FrameStamp, TimingInit};

    fn context(scene: &[u8]) -> Context<'_> {
        Context {
            check: true,
            route: TimingRoute::NativeRaster,
            requested: 1,
            completed: 1,
            scene,
            target: Target {
                generation: 2,
                viewport_revision: 3,
                size: (1280, 880),
                occluded: false,
            },
            scale_factor: 1.0,
            source_load_ms: Some(12.5),
            edit_sequence: 4,
            diagnostics: 2,
            checked: true,
            failure: None,
        }
    }
    fn record() -> TimingSample {
        TimingSample {
            id: 1,
            stamp: FrameStamp {
                generation: 2,
                viewport_revision: 3,
                serial: 9,
            },
            size: (1280, 880),
            route: TimingRoute::CpuUpload,
            outcome: TimingOutcome::Aborted,
            submission: TimingSubmission::Flush,
            configured: true,
            ui_prepare_ns: 10,
            queue_ns: 11,
            acquire_ns: Some(0),
            encode_upload_ns: Some(12),
            submit_ns: Some(13),
            completion_wait_ns: Some(14),
            cleanup_ns: Some(15),
            present_call_ns: None,
            owner_total_ns: 16,
            prepare_to_present_ns: None,
        }
    }
    fn report() -> TimingReport {
        TimingReport {
            requested: 2,
            accepted: 1,
            initialization: Some(TimingInit { owner_init_ns: 17 }),
            samples: vec![record()],
            failure: Some("owner failed".into()),
        }
    }
    fn measure(scene: &[u8]) -> Context<'_> {
        Context {
            check: false,
            route: TimingRoute::CpuUpload,
            requested: 2,
            completed: 0,
            checked: false,
            ..context(scene)
        }
    }
    fn encode(context: Context<'_>, timing: Option<&TimingReport>) -> String {
        let mut out = Vec::new();
        write(&mut out, context, timing).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn check_report_has_exact_schema_full_identity_and_no_invented_timing() {
        assert_eq!(
            encode(context(&[0, 10, 128, 255]), None),
            concat!(
                "{\"schema\":1,\"kind\":\"native-benchmark\",\"mode\":\"check\",\"route\":\"native-raster\",",
                "\"requested_frames\":1,\"completed_frames\":1,\"accepted_frames\":0,\"scene_hex\":\"000a80ff\",",
                "\"width\":1280,\"height\":880,\"generation\":2,\"viewport_revision\":3,\"scale_factor\":1,\"source_load_ms\":12.5,",
                "\"edit_sequence\":4,\"diagnostics\":2,\"checked\":true,\"failure\":null,\"owner_failure\":null,\"owner_init_ns\":null,\"samples\":[]}\n"
            )
        );
    }

    #[test]
    fn partial_owner_sample_preserves_zero_null_and_separate_failures() {
        let ctx = Context {
            failure: Some("quote\" slash\\\n\t\0 é"),
            ..measure(&[])
        };
        let out = encode(ctx, Some(&report()));
        assert!(out.contains(
            "\"failure\":\"quote\\\" slash\\\\\\n\\t\\u0000 é\",\"owner_failure\":\"owner failed\""
        ));
        assert!(out.contains("\"owner_init_ns\":17"));
        assert!(out.contains(concat!(
            "\"samples\":[{\"id\":1,\"generation\":2,\"viewport_revision\":3,\"serial\":9,\"width\":1280,\"height\":880,",
            "\"route\":\"cpu-upload\",\"outcome\":\"aborted\",\"submission\":\"flush\",\"configured\":true,",
            "\"ui_prepare_ns\":10,\"queue_ns\":11,\"acquire_ns\":0,\"encode_upload_ns\":12,\"submit_ns\":13,",
            "\"completion_wait_ns\":14,\"cleanup_ns\":15,\"present_call_ns\":null,\"owner_total_ns\":16,\"prepare_to_present_ns\":null}]"
        )));
        let checked_then_failed = encode(
            Context {
                failure: Some("worker cleanup failed after verification"),
                ..context(&[])
            },
            None,
        );
        assert!(checked_then_failed.contains(concat!(
            "\"checked\":true,\"failure\":",
            "\"worker cleanup failed after verification\""
        )));
    }

    #[test]
    fn all_control_characters_and_unicode_have_bounded_valid_escaping() {
        let mut text: String = (0u8..=31).map(char::from).collect();
        text.push_str("\"\\é𝄞");
        let mut out = Vec::new();
        error(&mut out, Some(&text)).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert_eq!(
            out,
            concat!(
                "\"\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\b\\t\\n\\u000b\\f\\r",
                "\\u000e\\u000f\\u0010\\u0011\\u0012\\u0013\\u0014\\u0015\\u0016\\u0017\\u0018\\u0019",
                "\\u001a\\u001b\\u001c\\u001d\\u001e\\u001f\\\"\\\\é𝄞\""
            )
        );
    }

    #[test]
    fn full_64k_identity_is_streamed_in_full_with_all_128_samples() {
        let scene: Vec<u8> = (0..MAX_IDENTITY_BYTES).map(|index| index as u8).collect();
        let mut report = report();
        report.requested = 128;
        report.accepted = 128;
        report.samples = (1..=128)
            .map(|id| TimingSample { id, ..record() })
            .collect();
        let out = encode(
            Context {
                requested: 128,
                ..measure(&scene)
            },
            Some(&report),
        );
        let encoded = out
            .split("\"scene_hex\":\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert_eq!(encoded.len(), MAX_IDENTITY_BYTES * 2);
        for (index, pair) in encoded.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            assert_eq!(
                u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap(),
                scene[index]
            );
        }
        assert_eq!(out.matches("\"id\":").count(), 128);
        assert!(out.len() <= MAX_REPORT_BYTES);
    }

    #[test]
    fn oversized_inputs_and_escaped_errors_refuse_before_output() {
        let oversized = vec![0; MAX_IDENTITY_BYTES + 1];
        let mut out = Vec::new();
        assert!(write(&mut out, context(&oversized), None).is_err());
        assert!(out.is_empty());
        let mut report = report();
        report.samples = vec![record(); 129];
        assert!(write(&mut out, measure(&[]), Some(&report)).is_err());
        assert!(out.is_empty());
        let long = "\0".repeat(683);
        assert!(
            write(
                &mut out,
                Context {
                    checked: false,
                    completed: 0,
                    failure: Some(&long),
                    ..context(&[])
                },
                None
            )
            .is_err()
        );
        assert!(out.is_empty());
        report.samples.clear();
        report.failure = Some(long);
        assert!(write(&mut out, measure(&[]), Some(&report)).is_err());
        assert!(out.is_empty());
        let exact = "x".repeat(MAX_ERROR_JSON_BYTES - 2);
        assert!(
            write(
                &mut out,
                Context {
                    checked: false,
                    completed: 0,
                    failure: Some(&exact),
                    ..context(&[])
                },
                None
            )
            .is_ok()
        );
    }

    #[test]
    fn modes_nullable_load_time_and_terminal_enum_strings_remain_distinct() {
        for load in [
            None,
            Some(f64::NAN),
            Some(f64::INFINITY),
            Some(f64::NEG_INFINITY),
        ] {
            let out = encode(
                Context {
                    source_load_ms: load,
                    ..context(&[])
                },
                None,
            );
            assert!(out.contains("\"source_load_ms\":null"));
        }
        for (value, expected) in [
            (TimingOutcome::Presented, "presented"),
            (TimingOutcome::NotPresented, "not-presented"),
            (TimingOutcome::Aborted, "aborted"),
            (TimingOutcome::Failed, "failed"),
        ] {
            assert_eq!(outcome(value), expected);
        }
        for (value, expected) in [
            (TimingSubmission::None, "none"),
            (TimingSubmission::Draws, "draws"),
            (TimingSubmission::Flush, "flush"),
        ] {
            assert_eq!(submission(value), expected);
        }
        let mut out = Vec::new();
        assert!(
            write(
                &mut out,
                Context {
                    checked: true,
                    ..measure(&[])
                },
                None
            )
            .is_err()
        );
        assert!(write(&mut out, context(&[]), Some(&report())).is_err());
        assert!(
            write(
                &mut out,
                Context {
                    completed: 0,
                    ..context(&[])
                },
                None
            )
            .is_err()
        );
        assert!(
            write(
                &mut out,
                Context {
                    scale_factor: f64::NAN,
                    ..context(&[])
                },
                None
            )
            .is_err()
        );
        assert!(out.is_empty());
    }

    #[test]
    fn writer_failure_propagates_without_claiming_a_complete_report() {
        struct Fails;
        impl Write for Fails {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("closed output"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        assert_eq!(
            write(&mut Fails, context(&[]), None).unwrap_err(),
            "native benchmark report output failed"
        );
    }
}
