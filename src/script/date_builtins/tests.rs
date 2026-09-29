use super::*;
use crate::date_host::{ZonePayload, ZonePayloadKind};
use std::sync::Arc;

fn zone(spec: &[u8]) -> Arc<crate::date_host::TimeZoneSnapshot> {
    let payload = ZonePayload::new(ZonePayloadKind::Posix2024, spec).unwrap();
    Arc::new(crate::date_host::TimeZoneSnapshot::parse(&payload, &mut Default::default()).unwrap())
}

fn prepared() -> (Runtime, Document) {
    (
        Runtime::with_date_host(DateHost::system(zone(b"UTC0"))),
        Document::parse(""),
    )
}

#[test]
fn date_unconfigured_embedding_has_explicit_local_failure_and_usable_utc() {
    let mut runtime = Runtime::new();
    let mut doc = Document::parse("");
    assert_eq!(
        runtime
            .execute("new Date(0).toISOString()", &mut doc)
            .unwrap(),
        Value::String("1970-01-01T00:00:00.000Z".into())
    );
    assert_eq!(
        runtime
            .execute("new Date(NaN).toString()", &mut doc)
            .unwrap(),
        Value::String("Invalid Date".into())
    );
    for source in [
        "new Date(0).getHours()",
        "new Date(1970,0,1)",
        "Date()",
        "Date.parse('1970-01-01T00:00:00')",
    ] {
        assert!(
            runtime
                .execute(source, &mut doc)
                .unwrap_err()
                .is_unsupported(),
            "{source}"
        );
    }
    assert!(
        matches!(runtime.execute("Date.now()", &mut doc).unwrap(), Value::Number(n) if n.is_finite())
    );
}

#[test]
fn date_host_clock_is_read_at_each_call_and_constructor_ignores_extra_call_values() {
    let host = DateHost::sequence_for_test(Some(zone(b"UTC0")), Arc::from([0, 1_234_000_000, -1]));
    let mut runtime = Runtime::with_date_host(host);
    let mut doc = Document::parse("");
    assert_eq!(
        runtime.execute("Date.now()", &mut doc).unwrap(),
        Value::Number(0.0)
    );
    assert_eq!(
        runtime.execute("new Date().getTime()", &mut doc).unwrap(),
        Value::Number(1234.0)
    );
    assert_eq!(
        runtime.execute("Date.now()", &mut doc).unwrap(),
        Value::Number(-1.0)
    );
    assert!(
        runtime
            .execute("Date.now()", &mut doc)
            .unwrap_err()
            .is_unsupported()
    );
}

#[test]
fn date_clock_and_zone_consume_the_shared_work_budget() {
    let (mut runtime, _) = prepared();
    runtime.steps = 0;
    assert!(runtime.date_now().unwrap_err().is_resource_limit());
    runtime.steps = 0;
    assert!(runtime.date_local(0.0).unwrap_err().is_resource_limit());
    runtime.steps = 0;
    assert!(runtime.date_utc(0.0).unwrap_err().is_resource_limit());
}

#[test]
fn date_parse_precharges_utf16_scan_and_wide_arithmetic_before_work() {
    let (mut runtime, _) = prepared();
    let text = JsString::from("1970-01-01T00:00:00.000Z");
    runtime.steps = text.len();
    assert!(runtime.date_parse(&text).unwrap_err().is_resource_limit());
    assert_eq!(runtime.steps, 0);
    runtime.steps = js_date::make_day_work(1e100, 0.0, 1.0) - 1;
    assert!(
        runtime
            .date_make_day(1e100, 0.0, 1.0)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.steps, 0);
}

#[test]
fn date_output_storage_failure_does_not_mutate_slots() {
    let (mut runtime, mut doc) = prepared();
    let date = runtime.execute("new Date(123)", &mut doc).unwrap();
    let (id, time) = runtime.date_slot(&date).unwrap();
    for name in ["toString", "toISOString", "toUTCString"] {
        runtime.steps = MAX_STEPS;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .date_native(name, date.clone(), &[], &mut doc)
                .unwrap_err()
                .is_resource_limit(),
            "{name}"
        );
        assert_eq!(runtime.objects[id].date_value, Some(time));
    }
}

#[test]
fn date_setter_work_refusal_preserves_the_coercion_callback_effect() {
    let (mut runtime, mut doc) = prepared();
    runtime
        .execute(
            "var d=new Date(0);var p={valueOf:function(){d.setTime(77);return 1;}};",
            &mut doc,
        )
        .unwrap();
    let date = runtime.execute("d", &mut doc).unwrap();
    let arg = runtime.execute("p", &mut doc).unwrap();
    let (id, _) = runtime.date_slot(&date).unwrap();
    runtime.steps = 159;
    assert!(
        runtime
            .date_native("setUTCDate", date, &[arg], &mut doc)
            .unwrap_err()
            .is_resource_limit()
    );
    assert_eq!(runtime.objects[id].date_value, Some(77.0));
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert!(runtime.frames.is_empty());
}

#[test]
fn date_limit_errors_bypass_author_catch_and_unwind_callbacks() {
    let (mut runtime, mut doc) = prepared();
    let error = runtime.execute("var caught=false;try{Date.UTC({valueOf:function(){while(true){};}});}catch(e){caught=true;}", &mut doc).unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(runtime.calls, 0);
    assert_eq!(runtime.stack_units, 0);
    assert!(runtime.frames.is_empty());
    assert_eq!(
        runtime.execute("caught", &mut doc).unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn date_historical_second_offset_roundtrips_and_clip_margin() {
    let mut runtime = Runtime::with_date_host(DateHost::system(zone(b"LMT4:56:02")));
    let mut doc = Document::parse("");
    assert_eq!(
        runtime
            .execute(
                "var d=new Date(-2208988800000);Date.parse(d.toString())",
                &mut doc
            )
            .unwrap(),
        Value::Number(-2208988800000.0)
    );
    assert!(
        runtime
            .date_utc(
                js_date::MAX_TIME
                    + f64::from(crate::date_host::MAX_ABS_OFFSET_SECONDS) * 1000.0
                    + 1.0
            )
            .unwrap()
            .is_nan()
    );
    assert!(runtime.date_utc(f64::MAX).unwrap().is_nan());
}

#[test]
fn own_date_parser_rejects_unicode_and_overflow_without_panicking() {
    for text in [
        "Thu Jan 01 1970 éé:éé:00 GMT+0000",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+9999999:00:00)",
        "Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+00:60:00)",
    ] {
        assert!(parse_own_date(text).is_none());
    }
}
