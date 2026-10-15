//! Computed Expression[+In] and conditional-middle Assignment[+In] controls.
//! Reviewed literals frozen before follow-up production; existing APIs only.
use super::*;

fn execute_mode(
    runtime: &mut Runtime,
    document: &mut Document,
    strict: bool,
    source: &str,
) -> Result<Value> {
    if strict {
        runtime.execute_strict(source, document)
    } else {
        runtime.execute(source, document)
    }
}

fn clean(runtime: &Runtime, case_id: &str, strict: bool) {
    assert_eq!(
        (
            runtime.frames.len(),
            runtime.eval_depth,
            runtime.stack_units,
            runtime.calls
        ),
        (0, 0, 0, 0),
        "{case_id} strict={strict}"
    );
    assert!(!runtime.microtasks_suspended(), "{case_id} strict={strict}");
    assert!(runtime.console.is_empty(), "{case_id} strict={strict}");
}

fn succeeds(case_id: &str, source: &str, expected: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let observed = execute_mode(&mut runtime, &mut document, strict, source)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} runtime: {error:?}"));
        assert_eq!(
            observed,
            Value::String(expected.into()),
            "{case_id} strict={strict}"
        );
        clean(&runtime, case_id, strict);

        let active = parser::Parser::program_context(source, false, strict)
            .map(|unit| code::canonical(&unit))
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} active: {error:?}"));
        let legacy = parser_legacy::Parser::program_context(source, false, strict)
            .and_then(code::compile)
            .map(|unit| code::canonical(&unit))
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} legacy: {error:?}"));
        assert_eq!(active, legacy, "{case_id} strict={strict}");
    }
}

fn syntax_error(case_id: &str, source: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let runtime_error = execute_mode(&mut runtime, &mut document, strict, source).unwrap_err();
        let active_error = parser::Parser::program_context(source, false, strict).unwrap_err();
        let legacy_error =
            parser_legacy::Parser::program_context(source, false, strict).unwrap_err();
        for (path, error) in [
            ("runtime", runtime_error),
            ("active", active_error),
            ("legacy", legacy_error),
        ] {
            assert_eq!(
                error.name(),
                "SyntaxError",
                "{case_id} strict={strict} {path}"
            );
            assert!(
                error.is_parse_error() && !error.is_unsupported(),
                "{case_id} strict={strict} {path}: {error:?}"
            );
        }
        clean(&runtime, case_id, strict);
    }
}

#[test]
fn expression_in_computed_member_classic_for() {
    succeeds(
        "computed_member_classic_for",
        r###"var o={x:1},table={true:'T'},v;for(v=table['x' in o];false;){}v;"###,
        "T",
    );
}

#[test]
fn expression_in_computed_call_result_classic_for() {
    succeeds(
        "computed_call_result_classic_for",
        r###"var o={x:1},v;function f(){return {true:'C'};}for(v=f()['x' in o];false;){}v;"###,
        "C",
    );
}

#[test]
fn expression_in_computed_constructor_classic_for() {
    succeeds(
        "computed_constructor_classic_for",
        r###"var o={x:1},table={true:function(){this.v='N';}};for(var v=new table['x' in o];false;){}v.v;"###,
        "N",
    );
}

#[test]
fn expression_in_nested_computed_members_restore_each_owner() {
    succeeds(
        "nested_computed_members_restore_each_owner",
        r###"var o={x:1},keys={true:'k'},values={k:'V'},v;for(v=values[keys['x' in o]];false;){}v;"###,
        "V",
    );
}

#[test]
fn expression_in_computed_inner_in_preserves_outer_for_in() {
    succeeds(
        "computed_inner_in_preserves_outer_for_in",
        r###"var o={x:1},target={},out='';for(target['x' in o] in {a:1})out+=target['true'];out;"###,
        "a",
    );
}

#[test]
fn expression_in_conditional_true_arm_classic_for() {
    succeeds(
        "conditional_true_arm_classic_for",
        r###"var o={x:1},v;for(v=true?'x' in o:false;false;){}v?'T':'F';"###,
        "T",
    );
}

#[test]
fn expression_in_conditional_unselected_true_arm_still_parses() {
    succeeds(
        "conditional_unselected_true_arm_still_parses",
        r###"var o={x:1};for(var v=false?'x' in o:7;false;){}v+'';"###,
        "7",
    );
}

#[test]
fn expression_in_nested_middle_inherits_true_for_its_false_arm() {
    succeeds(
        "nested_middle_inherits_true_for_its_false_arm",
        r###"var o={x:1},out='';for(let v=true?false?0:'x' in o:false;!out;){out=v?'T':'F';break;}out;"###,
        "T",
    );
}

#[test]
fn expression_in_conditional_middle_contains_computed_member() {
    succeeds(
        "conditional_middle_contains_computed_member",
        r###"var o={x:1},table={true:'T'},v;for(v=true?table['x' in o]:'F';false;){}v;"###,
        "T",
    );
}

#[test]
fn expression_in_computed_close_must_not_enable_outer_in() {
    syntax_error(
        "computed_close_must_not_enable_outer_in",
        r###"for(obj[0] in other;false;){}"###,
    );
}

#[test]
fn expression_in_conditional_false_arm_must_keep_no_in() {
    syntax_error(
        "conditional_false_arm_must_keep_no_in",
        r###"for(true?0:'x' in obj;false;){}"###,
    );
}

#[test]
fn expression_in_conditional_test_must_keep_no_in() {
    syntax_error(
        "conditional_test_must_keep_no_in",
        r###"for('x' in {x:1}?1:0;false;){}"###,
    );
}

#[test]
fn expression_in_conditional_middle_is_assignment_not_sequence() {
    syntax_error(
        "conditional_middle_is_assignment_not_sequence",
        r###"for(true?1,2:3;false;){}"###,
    );
}

#[test]
fn expression_in_parenthesized_in_controls() {
    succeeds(
        "parenthesized_in_controls",
        r###"var o={x:1},table={true:'T'},v,out='';for(v=table[('x' in o)];false;){}out+=v;for(v=true?('x' in o):false;false;){}out+=v?'T':'F';for(v=false?0:('x' in o);false;){}out+=v?'T':'F';out;"###,
        "TTT",
    );
}

#[test]
fn expression_in_ordinary_true_context_false_arm_control() {
    succeeds(
        "ordinary_true_context_false_arm_control",
        r###"var o={x:1},v=false?0:'x' in o;v?'T':'F';"###,
        "T",
    );
}

#[test]
fn expression_in_computed_sequence_relational_key() {
    succeeds(
        "computed_sequence_relational_key",
        r###"var out='',o={x:1},table={true:'V'},v;function key(){out+='K';return 0;}for(v=table[key(),'x' in o];false;){}out+v;"###,
        "KV",
    );
}

#[test]
fn expression_in_computed_call_sequence_order() {
    succeeds(
        "computed_call_sequence_order",
        r###"var out='',table={k:'C'},v;function f(){out+='B';return table;}function k(){out+='K';return 'k';}for(v=f()[out+='L',k()];false;){}out+v;"###,
        "BLKC",
    );
}

#[test]
fn expression_in_computed_constructor_sequence_order() {
    succeeds(
        "computed_constructor_sequence_order",
        r###"var out='',ct={k:function(){out+='N';this.v='V';}},v;function key(){out+='K';return 'k';}for(v=new ct[out+='L',key()](out+='A');false;){}out+v.v;"###,
        "LKANV",
    );
}

#[test]
fn expression_in_computed_sequence_preserves_outer_for_in() {
    succeeds(
        "computed_sequence_preserves_outer_for_in",
        r###"var o={x:1},target={},out='';for(target[0,'x' in o] in {a:1})out+=target['true'];out;"###,
        "a",
    );
}

#[test]
fn expression_in_computed_sequence_rejects_trailing_comma() {
    syntax_error(
        "computed_sequence_rejects_trailing_comma",
        r###"obj[1,];"###,
    );
}

#[test]
fn expression_in_computed_sequence_outside_for() {
    succeeds(
        "computed_sequence_outside_for",
        r###"var o={k:'V'},n=0;var v=o[n=1,'k'];n+v;"###,
        "1V",
    );
}
