use super::*;

const LITERALS: &str = include_str!("../../../tests/conformance/number-format-literals.tsv");
const CASES: &str = include_str!("../../../tests/conformance/number-format.js");
const CORRECT: &str = include_str!("../../../tests/conformance/number-format-correct-control.js");
const WRONG: &str = include_str!("../../../tests/conformance/number-format-wrong-control.js");

fn fresh() -> (Runtime, Document) {
    (
        Runtime::try_new().unwrap(),
        Document::parse(
            "<!doctype html><html><head><title>Before</title></head><body></body></html>",
        ),
    )
}

fn clean(runtime: &Runtime) {
    assert!(runtime.frames.is_empty());
    assert!(runtime.active_array_joins.is_empty());
    assert_eq!(runtime.json_depth, 0);
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
}

fn evaluate(
    runtime: &mut Runtime,
    doc: &mut Document,
    source: &str,
    strict: bool,
) -> Result<Value> {
    let result = if strict {
        runtime.execute_strict(source, doc)
    } else {
        runtime.execute(source, doc)
    };
    clean(runtime);
    result
}

fn literals() -> Vec<(&'static str, f64, &'static str)> {
    assert_eq!(LITERALS.lines().next(), Some("name\tbits_hex\texpected"));
    let rows: Vec<_> = LITERALS
        .lines()
        .skip(1)
        .map(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            assert_eq!(fields.len(), 3);
            (
                fields[0],
                f64::from_bits(u64::from_str_radix(fields[1], 16).unwrap()),
                fields[2],
            )
        })
        .collect();
    assert_eq!(rows.len(), 74);
    rows
}

fn limited_format(number: f64, work: usize, heap: usize) -> Result<String> {
    let (mut work_left, mut heap_left) = (work, heap);
    format(number, |steps, bytes| {
        if steps > work_left || bytes > heap_left {
            return Err(ScriptError::resource("independent formatter allowance"));
        }
        work_left -= steps;
        heap_left -= bytes;
        Ok(())
    })
}

#[test]
fn seventy_four_raw_binary64_literals_and_actual_format_admission() {
    for (name, number, expected) in literals() {
        let (mut work, mut heap) = (0, 0);
        let actual = format(number, |steps, bytes| {
            work += steps;
            heap += bytes;
            Ok(())
        })
        .unwrap();
        assert_eq!(actual, expected, "{name}");
        assert_eq!(
            limited_format(number, work, heap).unwrap(),
            expected,
            "{name}"
        );
        if work != 0 {
            assert!(
                limited_format(number, work - 1, heap)
                    .unwrap_err()
                    .is_resource_limit(),
                "{name}"
            );
        }
        if heap != 0 {
            assert!(
                limited_format(number, work, heap - 1)
                    .unwrap_err()
                    .is_resource_limit(),
                "{name}"
            );
        }
    }
}

#[test]
fn correction_branch_fees_are_admitted_before_rewrite() {
    // The last four rows are algebra inputs to the correction helper, not
    // claims that Rust's shortest formatter emits these significands.
    let rows = [
        (
            f64::from_bits(0x430c_6bf5_2634_0002),
            "10000000000000003",
            16,
            Some(10000000000000002),
            174,
            32,
        ),
        (
            f64::from_bits(0x430c_6bf5_2634_0006),
            "10000000000000008",
            16,
            None,
            7,
            0,
        ),
        (1.25, "125", 1, None, 23, 0),
        (f64::from_bits(1), "5", -323, None, 4, 0),
        (1.25, "13", 1, Some(12), 69, 32),
        (15.0, "1", 2, Some(2), 70, 32),
        (9.5, "9", 1, Some(10), 62, 32),
        (1.75, "13", 1, None, 57, 0),
    ];
    for (number, digits, position, expected, work, heap) in rows {
        let (mut paid_work, mut paid_heap) = (0, 0);
        assert_eq!(
            even_significand(number, digits, position, &mut |steps, bytes| {
                paid_work += steps;
                paid_heap += bytes;
                Ok(())
            })
            .unwrap(),
            expected
        );
        assert_eq!((paid_work, paid_heap), (work, heap), "{digits}");
        for (work_limit, heap_limit, succeeds) in [(work, heap, true), (work - 1, heap, false)]
            .into_iter()
            .chain((heap != 0).then_some((work, heap.saturating_sub(1), false)))
        {
            let (mut remaining_work, mut remaining_heap) = (work_limit, heap_limit);
            let result = even_significand(number, digits, position, &mut |steps, bytes| {
                if steps > remaining_work || bytes > remaining_heap {
                    return Err(ScriptError::resource("independent correction allowance"));
                }
                remaining_work -= steps;
                remaining_heap -= bytes;
                Ok(())
            });
            if succeeds {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
        }
    }
}

#[test]
fn special_values_do_not_enter_correction() {
    for (number, expected) in [
        (0.0, "0"),
        (-0.0, "0"),
        (f64::NAN, "NaN"),
        (f64::INFINITY, "Infinity"),
        (f64::NEG_INFINITY, "-Infinity"),
    ] {
        assert_eq!(
            format(number, |_, _| panic!("special value correction")).unwrap(),
            expected
        );
    }
}

#[test]
fn runtime_formatter_exact_and_one_short_preserve_vm_state() {
    for number in [
        f64::from_bits(0x430c_6bf5_2634_0002),
        f64::from_bits(0xc30c_6bf5_2634_0002),
        1.25,
        f64::from_bits(1),
    ] {
        let (mut reference, _) = fresh();
        let (start_work, start_heap) = (reference.steps, reference.allocated);
        let expected = reference.number_text(number).unwrap();
        let (work, heap) = (
            start_work - reference.steps,
            reference.allocated - start_heap,
        );
        for (work_limit, heap_limit, succeeds) in [(work, heap, true), (work - 1, heap, false)]
            .into_iter()
            .chain((heap != 0).then_some((work, heap.saturating_sub(1), false)))
        {
            let (mut runtime, _) = fresh();
            runtime.steps = work_limit;
            runtime.allocated = MAX_HEAP - heap_limit;
            let result = runtime.number_text(number);
            if succeeds {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            clean(&runtime);
        }
    }
}

fn public_rows(start: usize, end: usize) {
    for strict in [false, true] {
        for index in start..end {
            let (mut runtime, mut doc) = fresh();
            let source = format!("{CASES}\nnumberFormatCases.public_row({index});");
            assert_eq!(
                evaluate(&mut runtime, &mut doc, &source, strict).unwrap(),
                Value::Bool(true),
                "row {index}, strict={strict}"
            );
        }
    }
}

#[test]
fn public_special_values_in_both_modes() {
    public_rows(0, 6);
}
#[test]
fn public_small_values_in_both_modes() {
    public_rows(6, 16);
}
#[test]
fn public_midpoint_neighbors_in_both_modes() {
    public_rows(16, 34);
}
#[test]
fn public_notation_boundaries_in_both_modes() {
    public_rows(34, 50);
}
#[test]
fn public_extreme_exponents_in_both_modes() {
    public_rows(50, 62);
}
#[test]
fn public_integer_boundaries_in_both_modes() {
    public_rows(62, 74);
}

#[test]
fn public_literal_keys_and_callback_order_in_both_modes() {
    for strict in [false, true] {
        for name in ["literal_keys", "callback_order_and_abrupt_identity"] {
            let (mut runtime, mut doc) = fresh();
            assert_eq!(
                evaluate(
                    &mut runtime,
                    &mut doc,
                    &format!("{CASES}\nnumberFormatCases.{name}();"),
                    strict
                )
                .unwrap(),
                Value::Bool(true)
            );
        }
    }
}

#[test]
fn frozen_positive_and_semantically_wrong_controls_in_both_modes() {
    for strict in [false, true] {
        let (mut runtime, mut doc) = fresh();
        assert_eq!(
            evaluate(&mut runtime, &mut doc, CORRECT, strict).unwrap(),
            Value::Bool(true)
        );
        let (mut runtime, mut doc) = fresh();
        let error = evaluate(&mut runtime, &mut doc, WRONG, strict).unwrap_err();
        assert_eq!(error.name(), "Error");
        assert_eq!(error.intrinsic_error_name(), Some("Error"));
        assert!(!error.is_resource_limit());
        let ErrorKind::Thrown(value) = error.kind else {
            panic!("wrong partner must throw");
        };
        assert_eq!(
            runtime.get(value, "message", &mut doc).unwrap(),
            Value::String("wrong shortest tie choice".into())
        );
        clean(&runtime);
    }
}

fn callback_input(strict: bool) -> (Runtime, Document, Value) {
    let (mut runtime, mut doc) = fresh();
    let value = evaluate(
        &mut runtime,
        &mut doc,
        r#"
        let formatTrace = '';
        let formatInput = {[Symbol.toPrimitive]: function(hint) {
            formatTrace += 'p'; document.title = 'prefix'; return 1000000000000000.25;
        }};
        formatInput;
    "#,
        strict,
    )
    .unwrap();
    (runtime, doc, value)
}

fn prefix_retained(runtime: &Runtime, doc: &Document) {
    assert_eq!(doc.title(), "prefix");
    assert_eq!(
        runtime.lookup(1, "formatTrace").unwrap().1,
        Value::String("p".into())
    );
    clean(runtime);
}

#[test]
fn runtime_denials_retain_preceding_authored_conversion_effects() {
    for strict in [false, true] {
        let (mut measured, mut doc, value) = callback_input(strict);
        let (start_work, start_heap) = (measured.steps, measured.allocated);
        assert_eq!(
            measured.string_hint(value, &mut doc).unwrap(),
            JsString::from("1000000000000000.2")
        );
        let (work, heap) = (start_work - measured.steps, measured.allocated - start_heap);
        prefix_retained(&measured, &doc);
        assert!(work > 0 && heap > 0);
        for (work_limit, heap_limit, succeeds) in [
            (work, heap, true),
            (work - 1, heap, false),
            (work, heap - 1, false),
        ] {
            let (mut runtime, mut doc, value) = callback_input(strict);
            runtime.steps = work_limit;
            runtime.allocated = MAX_HEAP - heap_limit;
            let result = runtime.string_hint(value, &mut doc);
            if succeeds {
                assert_eq!(result.unwrap(), JsString::from("1000000000000000.2"));
            } else {
                assert!(result.unwrap_err().is_resource_limit());
            }
            prefix_retained(&runtime, &doc);
        }
    }
}

fn compile_budget(work: usize, heap: usize) -> regexp::Budget {
    regexp::Budget {
        steps: work,
        allocated: 0,
        heap_limit: heap,
        stack_limit: MAX_STACK_UNITS / 4,
    }
}

#[test]
fn compiled_literal_key_exact_and_one_short_admission() {
    // Dynamic Function owns its statements in FunctionRef::body, whereas
    // code::canonical only visits Unit::body. Inspect the actual stored key.
    let check_key = |compiled: &code::FunctionRef| {
        let unit = &compiled.unit;
        assert_eq!(compiled.body.len(), 1);
        let code::Stmt::Return(Some(member)) = unit.stmt(compiled.body[0]) else {
            panic!("expected the authored return");
        };
        let code::Expr::Member(call, _) = unit.expr(*member) else {
            panic!("expected the first returned key");
        };
        let code::Expr::Call(_, arguments) = unit.expr(*call) else {
            panic!("expected Object.keys call");
        };
        assert_eq!(arguments.len(), 1);
        let code::Expr::Object(entries) = unit.expr(arguments[0]) else {
            panic!("expected the authored object literal");
        };
        assert_eq!(entries.len(), 1);
        let code::PropertyName::Literal(key) = &entries[0].0 else {
            panic!("expected a compiled literal key");
        };
        assert_eq!(key, &JsString::from("1000000000000000.2"));
    };
    let body = "return Object.keys({1000000000000000.25:7})[0];";
    let mut measured = compile_budget(MAX_STEPS, MAX_HEAP);
    let compiled = parser::Parser::dynamic_function("", body, &mut measured).unwrap();
    let work = MAX_STEPS - measured.steps;
    let heap = measured.allocated;
    check_key(&compiled);
    for (work_limit, heap_limit, succeeds) in [
        (work, heap, true),
        (work - 1, heap, false),
        (work, heap - 1, false),
    ] {
        let mut budget = compile_budget(work_limit, heap_limit);
        let result = parser::Parser::dynamic_function("", body, &mut budget);
        if succeeds {
            let compiled = result.unwrap();
            check_key(&compiled);
            let (mut runtime, mut doc) = fresh();
            let function = runtime.function_value(&compiled, 1).unwrap();
            assert_eq!(
                runtime
                    .call(function, vec![], Value::Undefined, &mut doc)
                    .unwrap(),
                Value::String("1000000000000000.2".into())
            );
            clean(&runtime);
        } else {
            assert!(result.err().unwrap().is_resource_limit());
        }
    }
}

#[test]
fn dynamic_compile_denial_retains_conversion_without_publishing_function() {
    let body = format!("{}return 7;", "({1000000000000000.25:7});".repeat(24));
    let mut budget = compile_budget(MAX_STEPS, MAX_HEAP);
    parser::Parser::dynamic_function("", &format!("\n{body}\n"), &mut budget).unwrap();
    let compile_work = MAX_STEPS - budget.steps;
    let compile_heap = budget.allocated;
    // Several literal-key compilations make the compiler interval larger than
    // the later function-publication tail. The error below verifies the cut
    // actually lands inside compilation, rather than assuming an opcode fee.
    assert!(compile_work > 0 && compile_heap > 0);
    for strict in [false, true] {
        let setup = format!(
            r#"
            let formatTrace = '';
            let formatInput = {{[Symbol.toPrimitive]: function() {{
                formatTrace += 'p'; document.title = 'prefix'; return {body:?};
            }}}}; formatInput;
        "#
        );
        let (mut measured, mut doc) = fresh();
        let value = evaluate(&mut measured, &mut doc, &setup, strict).unwrap();
        let (start_work, start_heap) = (measured.steps, measured.allocated);
        let function = measured
            .dynamic_function(
                vec![value],
                Runtime::native("Function", Value::Window),
                &mut doc,
            )
            .unwrap();
        let total_work = start_work - measured.steps;
        let total_heap = measured.allocated - start_heap;
        assert_eq!(
            measured
                .call(function, vec![], Value::Undefined, &mut doc)
                .unwrap(),
            Value::Number(7.0)
        );
        prefix_retained(&measured, &doc);
        assert!(total_work > compile_work);
        assert!(total_heap > compile_heap);
        for heap_denial in [false, true] {
            let (mut runtime, mut doc) = fresh();
            let value = evaluate(&mut runtime, &mut doc, &setup, strict).unwrap();
            let before_functions = runtime.functions.len();
            if heap_denial {
                runtime.allocated = MAX_HEAP - (total_heap - compile_heap / 2);
            } else {
                runtime.steps = total_work - compile_work / 2;
            }
            let error = runtime
                .dynamic_function(
                    vec![value],
                    Runtime::native("Function", Value::Window),
                    &mut doc,
                )
                .unwrap_err();
            assert!(error.is_resource_limit());
            let message = if heap_denial {
                "script compile allocation limit exceeded"
            } else {
                "regular expression work limit exceeded"
            };
            assert!(error.message.contains(message), "{error:?}");
            assert_eq!(runtime.functions.len(), before_functions);
            prefix_retained(&runtime, &doc);
        }
    }
}
