use super::*;
use crate::script::{MAX_HEAP, MAX_STEPS, parser, parser_legacy};

fn clean(runtime: &Runtime) {
    assert_eq!(
        (
            runtime.frames.len(),
            runtime.eval_depth,
            runtime.stack_units,
            runtime.calls
        ),
        (0, 0, 0, 0)
    );
}
fn run(runtime: &mut Runtime, unit: &Rc<code::Unit>, doc: &mut Document) -> Result<Value> {
    match super::super::evaluate_statements(runtime, unit, ListOwner::Program, 1, doc)? {
        Flow::Normal(value) => Ok(value.unwrap_or(Value::Undefined)),
        _ => panic!("loop fixture left abrupt completion"),
    }
}
fn succeeds(source: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "{source}/{strict}");
        clean(&runtime);
    }
}

#[test]
fn for_of_flat_and_legacy_parsers_preserve_head_context_and_binding_rules() {
    for source in [
        "for(var x of []){}",
        "for(let x of []){}",
        "for(const x of []){}",
        "for(let of of []){}",
        "var async;for((async) of []){}",
        "for(var async of []){}",
        "for(let async of []){}",
        "for(o[k()] of (a,b)){}",
        "outer:for(const x of []){continue outer;}",
        "const x;",
        "for(const x;false;){}",
        "for(const x=1 of []){}",
        "for(var x,y of []){}",
        "for(let x of [],[]){}",
        "for(let x o\\u0066 []){}",
        "for(let of []){}",
        "for(async of []){}",
        "for(let x of []){var x;}",
    ] {
        for strict in [false, true] {
            let old = parser_legacy::Parser::program_context(source, false, strict)
                .and_then(code::compile)
                .map(|unit| code::canonical(&unit))
                .map_err(|error| (error.name().to_owned(), error.is_unsupported()));
            let new = parser::Parser::program_context(source, false, strict)
                .map(|unit| code::canonical(&unit))
                .map_err(|error| (error.name().to_owned(), error.is_unsupported()));
            assert_eq!(new, old, "{source}/{strict}");
        }
    }
    for source in ["for(const x;false;){}", "const x;", "for(async of []){}"] {
        assert_eq!(
            parser::Parser::program(source).unwrap_err().name(),
            "SyntaxError"
        );
    }
    for source in [
        "for([a,b] of []){}",
        "for({x:a} of []){}",
        "for(const [a] of []){}",
    ] {
        assert!(
            parser::Parser::program(source)
                .unwrap_err()
                .is_unsupported()
        );
    }
    assert!(
        parser::Parser::program("for await(var x of []){}")
            .unwrap_err()
            .is_unsupported()
    );
}

#[test]
fn for_of_lexical_rhs_tdz_closures_and_catch_var_resolution() {
    succeeds(
        r#"
        var escaped, caught=false;
        try { for(let item of (escaped=function(){return item;},[])){} } catch(e){caught=true;}
        var tdz=false;try{escaped();}catch(e){tdz=e instanceof ReferenceError;}
        var closures=[];for(const item of [2,4]){closures.push(function(){return item;});}
        var outer=10, inside;
        try{throw 20;}catch(outer){for(var outer of [30]){inside=outer;}}
        !caught && tdz && closures[0]()===2 && closures[1]()===4 && outer===10 && inside===30;
    "#,
    );
}

#[test]
fn for_of_step_failures_do_not_close_and_body_throw_keeps_identity() {
    succeeds(
        r#"
        var reason={},closed=0,reads=0,source={},it={};
        source[Symbol.iterator]=function(){return it;};
        Object.defineProperty(it,'return',{get:function(){reads++;return function(){closed++;throw 9;};}});
        it.next=function(){return {get done(){throw reason;}};};
        var first;try{for(var x of source){}}catch(e){first=e;}
        var quiet=closed===0&&reads===0;
        it.next=function(){return {done:false,value:2};};
        var second;try{for(var x of source){throw reason;}}catch(e){second=e;}
        first===reason&&quiet&&second===reason&&closed===1&&reads===1;
    "#,
    );
}

#[test]
fn for_of_close_is_live_after_assignment_failure_and_precedes_finally() {
    succeeds(
        r#"
        var trace='',reason={},source={},it={next:function(){return {value:1,done:false};}};
        source[Symbol.iterator]=function(){return it;};
        var target={set x(v){trace+='s';it.return=function(){trace+='r';return {};};throw reason;}};
        var caught;try{for(target.x of source){trace+='b';}}catch(e){caught=e;}finally{trace+='f';}
        caught===reason&&trace==='srf';
    "#,
    );
}

#[test]
fn for_of_labels_continue_without_close_and_outward_continue_closes() {
    succeeds(
        r#"
        var closes=0,steps=0,source={};source[Symbol.iterator]=function(){var n=0;return {
            next:function(){steps++;return {value:n++,done:n>3};},
            return:function(){closes++;return {};}};};
        outer:for(var x of source){if(x<2)continue outer;break;}
        var first=closes===1&&steps===3;
        for(var i=0;i<2;i++){inner:for(var x of source){continue inner;}}
        var second=closes===1&&steps===11;
        outer:for(var i=0;i<2;i++){for(var x of source){continue outer;}}
        first&&second&&closes===3&&steps===13;
    "#,
    );
}

const SETUP: &str = r#"
    var seen=0,closed=0,caught=0,finalized=0,source={};
    source[Symbol.iterator]=function(){var n=0;return {
        next:function(){return {done:n===3,value:n++};},
        return:function(){closed++;return {};}};};
"#;

#[test]
fn for_of_every_work_cut_unwinds_without_running_author_error_cleanup() {
    let unit = parser::Parser::program(
        "try{for(let x of source){seen+=x;}}catch(e){caught++;}finally{finalized++;}seen;",
    )
    .unwrap();
    let setup = |runtime: &mut Runtime, doc: &mut Document| {
        runtime.execute(SETUP, doc).unwrap();
        runtime.steps = MAX_STEPS;
    };
    let mut measured = Runtime::new();
    let mut doc = Document::parse("");
    setup(&mut measured, &mut doc);
    assert_eq!(
        run(&mut measured, &unit, &mut doc).unwrap(),
        Value::Number(3.0)
    );
    let work = MAX_STEPS - measured.steps;
    assert!(work > 100 && work < 10_000);
    for allowance in 0..=work {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        setup(&mut runtime, &mut doc);
        runtime.steps = allowance;
        let result = run(&mut runtime, &unit, &mut doc);
        if allowance == work {
            assert_eq!(result.unwrap(), Value::Number(3.0));
        } else {
            assert!(
                result.unwrap_err().is_resource_limit(),
                "allowance {allowance}"
            );
        }
        assert_eq!(runtime.lookup(1, "closed").unwrap().1, Value::Number(0.0));
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        // Once all values ran, normal finally may legitimately precede a
        // later terminal stop. Before that point no error cleanup may run.
        if runtime.lookup(1, "seen").unwrap().1 != Value::Number(3.0) {
            assert_eq!(
                runtime.lookup(1, "finalized").unwrap().1,
                Value::Number(0.0)
            );
        }
        clean(&runtime);
    }
    let weak = Rc::downgrade(&unit);
    drop(unit);
    assert!(
        weak.upgrade().is_none(),
        "completed frames retained executable code"
    );
}

#[test]
fn for_of_heap_refusal_preserves_prior_effects_and_suppresses_close() {
    let unit =
        parser::Parser::program("try{for(const x of source){seen++;}}catch(e){caught++;}").unwrap();
    let mut measured = Runtime::new();
    let mut doc = Document::parse("");
    measured.execute(SETUP, &mut doc).unwrap();
    let base = measured.allocated;
    run(&mut measured, &unit, &mut doc).unwrap();
    let used = measured.allocated - base;
    assert!(used > 0);
    for available in [0, 1, 127, 128, used / 3, used / 2, used - 1, used] {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        runtime.execute(SETUP, &mut doc).unwrap();
        runtime.allocated = MAX_HEAP - available;
        let result = run(&mut runtime, &unit, &mut doc);
        if available == used {
            result.unwrap();
        } else {
            assert!(
                result.unwrap_err().is_resource_limit(),
                "heap {available}/{used}"
            );
        }
        assert_eq!(runtime.lookup(1, "closed").unwrap().1, Value::Number(0.0));
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
        clean(&runtime);
    }
}

#[test]
fn for_of_terminal_close_callback_failure_outranks_pending_author_throw() {
    let mut runtime = Runtime::new();
    let mut doc = Document::parse("");
    let error = runtime
        .execute(
            r#"
        var seen=0,caught=0,finalized=0,source={};source[Symbol.iterator]=function(){return {
            next:function(){return {value:1,done:false};},
            return:function(){seen++;while(true){};}};};
        try{for(var x of source){throw 7;}}catch(e){caught++;}finally{finalized++;}
    "#,
            &mut doc,
        )
        .unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(runtime.lookup(1, "seen").unwrap().1, Value::Number(1.0));
    assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Number(0.0));
    assert_eq!(
        runtime.lookup(1, "finalized").unwrap().1,
        Value::Number(0.0)
    );
    clean(&runtime);
}

#[test]
fn for_of_protocol_walk_is_lazy_charged_and_preserves_accessor_receiver() {
    let mut runtime = Runtime::new();
    let mut doc = Document::parse("");
    assert_eq!(
        runtime
            .execute(
                r#"
        var it={next:function(){return {done:true};}},source={};
        Object.setPrototypeOf(it,window);source[Symbol.iterator]=function(){return it;};
        for(var x of source){}true;
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
    let error = runtime
        .execute("delete it.next;for(var x of source){}", &mut doc)
        .unwrap_err();
    assert!(error.is_unsupported());
    clean(&runtime);
    succeeds(
        r#"
        var actual,source={},result={get done(){actual=this;return true;}};
        source[Symbol.iterator]=function(){return {next:function(){return result;}};};
        for(var x of source){}actual===result;
    "#,
    );
}

#[test]
fn for_of_nested_loops_use_small_native_stack_and_release_frames() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let source = format!(
                "var n=0;{}n++;{}n===1;",
                "for(let x of [1]){".repeat(40),
                "}".repeat(40)
            );
            succeeds(&source);
        })
        .unwrap()
        .join()
        .unwrap();
}
