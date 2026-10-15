//! Public call/new spread literals frozen before production implementation.
//! Only existing Runtime/parser interfaces; private quotas are held separately.
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

fn observe(case_id: &str, source: &str, expression: &str, expected: &str) {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        execute_mode(&mut runtime, &mut document, strict, source)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} source: {error:?}"));
        clean(&runtime);
        let observed = execute_mode(&mut runtime, &mut document, strict, expression)
            .unwrap_or_else(|error| panic!("{case_id} strict={strict} observation: {error:?}"));
        assert_eq!(
            observed,
            Value::String(expected.into()),
            "{case_id} strict={strict}"
        );
        clean(&runtime);
        assert!(!runtime.microtasks_suspended(), "{case_id} strict={strict}");
        assert!(runtime.console.is_empty(), "{case_id} strict={strict}");
    }
}

#[test]
fn call_spread_preserves_ordinary_call_constructor_and_rest_controls() {
    observe(
        "ordinary_control",
        r###"
        var obj={tag:'R',m:function(...v){return this.tag+':'+v.join(',');}};
        function C(...v){this.values=v;this.target=new.target;}
        var ordinary=obj.m(1,2),created=new C(3,4);
        var arrow=(...v)=>v.join(',');
        var dynamic=Function('...v','return v.join(",");');
        var arrowResult=arrow(5,6),dynamicResult=dynamic(7,8);
        "###,
        r###"[ordinary,created.values.join(','),created.target===C,Object.getPrototypeOf(created)===C.prototype,arrowResult,dynamicResult].join('|')"###,
        "R:1,2|3,4|true|true|5,6|7,8",
    );
}

#[test]
fn call_spread_mixed_multiple_empty_bound_and_native_calls() {
    observe(
        "mixed_calls",
        r###"
        function collect(...x){return x;}
        var marker={},a=collect(0,...[1,marker],...[],2,...[3],);
        var empty=collect(...[]),receiver={tag:'R'};
        function bound(...x){return this.tag+':'+x.join(',');}
        var b=bound.bind(receiver,7),got=b(...[8,9]);
        var max=Math.max(...[-3,2,1],...[]),nil=collect(...[],...[]);
        "###,
        r###"[a.length,a[0],a[1],a[2]===marker,a[3],a[4],empty.length,got,max,nil.length].join('|')"###,
        "5|0|1|true|2|3|0|R:7,8,9|2|0",
    );
}

#[test]
fn call_spread_new_preserves_new_target_prototype_and_bound_arguments() {
    observe(
        "mixed_constructors",
        r###"
        function C(...tail){this.values=tail;this.target=new.target;}
        var a=new C(1,...[2,3],...[],4,);
        var B=C.bind({unused:true},8),c=new B(...[9],);
        var n=new Array(...[5,6],),e=new C(...[]);
        "###,
        r###"[a.values.join(','),a.target===C,Object.getPrototypeOf(a)===C.prototype,c.values.join(','),c.target===C,n.join(','),e.values.length,e.target===C].join('|')"###,
        "1,2,3,4|true|true|8,9|true|5,6|0|true",
    );
}

#[test]
fn call_spread_caches_target_receiver_and_next_before_ordered_steps() {
    observe(
        "ordered_cached_handles",
        r###"
        var log='',bad=false,targetGets=0,nextGets=0,nextCalls=0,obj={},src={},it={};
        Object.defineProperty(obj,'m',{configurable:true,get:function(){targetGets++;log+='G';return function(a,b,c){'use strict';log+='F';bad=bad||this!==obj||arguments.length!==3;return [a,b,c].join(',');};}});
        Object.defineProperty(src,Symbol.iterator,{get:function(){log+='I';return function(){'use strict';log+='M';bad=bad||this!==src||arguments.length!==0;Object.defineProperty(obj,'m',{value:function(){bad=true;},configurable:true});return it;};}});
        Object.defineProperty(it,'next',{configurable:true,get:function(){nextGets++;log+='N';return function(){'use strict';bad=bad||this!==it||arguments.length!==0;log+='S';nextCalls++;if(nextCalls===1){Object.defineProperty(it,'next',{value:function(){bad=true;throw 'uncached next';},configurable:true});return {get done(){log+='D';return false;},get value(){log+='V';return 20;}};}return {get done(){log+='D';return true;},get value(){bad=true;throw 'done value';}};};}});
        function first(){log+='A';return 10;}function last(){log+='B';return 30;}
        var result=obj.m(first(),...src,last());
        "###,
        r###"[log,result,bad,targetGets,nextGets,nextCalls].join('|')"###,
        "GAIMNSDVSDBF|10,20,30|false|1|1|2",
    );
}

#[test]
fn call_spread_invalid_iterator_protocol_throws_without_target_or_close() {
    observe(
        "invalid_protocol",
        r###"
        var cases=[undefined,null,{},{},{},{},{}],caught=0,target=0,later=0,returns=0;
        cases[3][Symbol.iterator]=7;
        cases[4][Symbol.iterator]=function(){return 7;};
        var noncallable={next:7},nonobject={next:function(){return 7;}};
        function poison(it){Object.defineProperty(it,'return',{get:function(){returns++;throw 'close';}});}
        poison(noncallable);poison(nonobject);
        cases[5][Symbol.iterator]=function(){return noncallable;};
        cases[6][Symbol.iterator]=function(){return nonobject;};
        function sink(){target++;}function late(){later++;return 1;}
        for(var i=0;i<cases.length;i++){try{sink(...cases[i],late());}catch(e){if(e instanceof TypeError)caught++;}}
        "###,
        r###"[caught,target,later,returns].join('|')"###,
        "7|0|0|0",
    );
}

#[test]
fn call_spread_abrupt_iterator_stages_preserve_reason_and_never_close() {
    observe(
        "abrupt_stages_call_and_new",
        r###"
        var marker={},caught=0,target=0,later=0,returns=0;
        function sink(){target++;}function late(){later++;return 1;}
        for(var construct=0;construct<2;construct++){
          for(var stage=0;stage<6;stage++){
            var src={},it={};
            Object.defineProperty(it,'return',{get:function(){returns++;throw 'replacement';}});
            Object.defineProperty(it,'next',{get:function(){if(stage===2)throw marker;return function(){if(stage===3)throw marker;return {get done(){if(stage===4)throw marker;return false;},get value(){if(stage===5)throw marker;return 1;}};};}});
            Object.defineProperty(src,Symbol.iterator,{get:function(){if(stage===0)throw marker;return function(){if(stage===1)throw marker;return it;};}});
            try{if(construct)new sink(...src,late());else sink(...src,late());}catch(e){if(e===marker)caught++;}
          }
        }
        "###,
        r###"[caught,target,later,returns].join('|')"###,
        "12|0|0|0",
    );
}

#[test]
fn call_spread_argument_errors_and_callability_checks_follow_evaluation() {
    observe(
        "callability_and_later_errors",
        r###"
        var marker={},caught=0,exact=false,steps=0,later=0,returns=0,target=0,acquires=0;
        function source(){var n=0,it={next:function(){steps++;return n++===0?{done:false,value:1}:{done:true};}};Object.defineProperty(it,'return',{get:function(){returns++;throw 'close';}});var s={};s[Symbol.iterator]=function(){acquires++;return it;};return s;}
        function late(){later++;return 2;}function boom(){later++;throw marker;}function sink(){target++;}
        var bad={},badConstructor=Math.max;
        try{bad(...source(),late());}catch(e){if(e instanceof TypeError)caught++;}
        try{new badConstructor(...source(),late());}catch(e){if(e instanceof TypeError)caught++;}
        try{sink(...source(),boom());}catch(e){exact=e===marker;if(exact)caught++;}
        var getterTarget={};Object.defineProperty(getterTarget,'m',{get:function(){throw marker;}});
        var getterExact=false;try{getterTarget.m(...source(),late());}catch(e){getterExact=e===marker;}
        "###,
        r###"[caught,exact,steps,later,returns,target,acquires,getterExact].join('|')"###,
        "3|true|6|3|0|0|3|true",
    );
}

#[test]
fn call_spread_sparse_unicode_and_overridden_iterators_supply_values() {
    observe(
        "iterator_values",
        r###"
        function collect(...x){return x;}
        var a=[,,3];Array.prototype[1]=7;var sparse=collect(...a);delete Array.prototype[1];
        var text=collect(...'A\uD834\uDD1E\uD800');
        a[Symbol.iterator]=function(){var n=0;return {next:function(){return n++===0?{done:false,value:9}:{done:true};}};};
        var custom=collect(...a);
        "###,
        r###"[sparse.length,sparse[0]===undefined,sparse[1],sparse[2],text.length,text[0],text[1].length,text[2].length,text[2].charCodeAt(0),custom.join(',')].join('|')"###,
        "3|true|7|3|3|A|2|1|55296|9",
    );
}

#[test]
fn call_spread_reentrant_iteration_keeps_each_accumulator_independent() {
    observe(
        "reentrant_spread",
        r###"
        var log='',nested,inner={},outer={};function collect(...v){return v;}
        inner[Symbol.iterator]=function(){log+='I';var n=0;return {next:function(){log+='i';return n++===0?{done:false,value:4}:{done:true};}};};
        outer[Symbol.iterator]=function(){log+='O';var n=0;return {next:function(){log+='o';if(n++===0){nested=collect(...inner);return {done:false,value:2};}return {done:true};}};};
        var result=collect(1,...outer,3);
        "###,
        r###"[log,result.join(','),nested.join(',')].join('|')"###,
        "OoIiio|1,2,3|4",
    );
}

#[test]
fn call_spread_author_entry_does_not_checkpoint_between_arguments() {
    observe(
        "checkpoint_order",
        r###"
        var log='',src={};src[Symbol.iterator]=function(){log+='I';var n=0;return {next:function(){if(n++===0){queueMicrotask(function(){log+='Q';});return {done:false,value:1};}return {done:true};}};};
        function late(){log+='A';return 2;}function sink(){log+='T';}
        sink(...src,late());log+='S';
        "###,
        "log",
        "IATSQ",
    );
}

#[test]
fn call_spread_dynamic_function_assignment_and_new_member_contexts() {
    observe(
        "dynamic_and_expression_context",
        r###"
        var dynamic=Function('a','return (function(...v){return v.join(",");})(...a);');
        var first=dynamic([1,2]),x=[3],y=[4,5];
        var second=(function(...v){return v.join(',');})(...(x=y),6);
        function C(...v){this.values=v;}var holders={C:C};
        var third=new holders['C'](...[7,8]);function factory(){return C;}
        var fourth=new (factory())(...[9]);
        var numberError=false;try{dynamic(....5);}catch(e){numberError=e instanceof TypeError;}
        "###,
        r###"[first,second,x===y,third.values.join(','),fourth.values.join(','),numberError].join('|')"###,
        "1,2|4,5,6|true|7,8|9|true",
    );
}

#[test]
fn call_spread_rest_forwarding_reaches_bound_promise_resolve() {
    observe(
        "rest_to_bound_resolve",
        r###"
        var original=Promise.resolve,bound=original.bind(Promise),count=0,values;
        Promise.resolve=function(...args){count++;return bound(...args);};
        Promise.all([1,2,3]).then(function(v){values=v;});Promise.resolve=original;
        "###,
        r###"count+'|'+values.join(',')"###,
        "3|1,2,3",
    );
}

#[test]
fn call_spread_active_and_legacy_parsers_accept_selected_argument_contexts() {
    for source in [
        "f(...xs);",
        "f(1,...xs,2,...ys,);",
        "f(...[],...[]);",
        "new F(...xs);",
        "new F(1,...xs,2,...ys,);",
        "new (factory())(...xs).value;",
        "f(1,);new F(1,);",
        "f(.../*after ellipsis*/xs);",
        "f(...\nxs);",
        "f(....5);",
        "f(...(xs=ys));",
        "function f(...rest){return g(...rest);}",
        "var f=(...rest)=>g(...rest);",
    ] {
        for strict in [false, true] {
            let legacy = parser_legacy::Parser::program_context(source, false, strict)
                .and_then(code::compile)
                .map(|unit| code::canonical(&unit))
                .unwrap_or_else(|error| panic!("legacy {source} strict={strict}: {error:?}"));
            let active = parser::Parser::program_context(source, false, strict)
                .map(|unit| code::canonical(&unit))
                .unwrap_or_else(|error| panic!("active {source} strict={strict}: {error:?}"));
            assert_eq!(active, legacy, "{source} strict={strict}");
        }
    }
}

#[test]
fn call_spread_malformed_ellipsis_and_argument_holes_remain_rejected() {
    for source in [
        "f(...);",
        "f(...,);",
        "f(,1);",
        "f(1,,2);",
        "f(. . .xs);",
        "f(.. .xs);",
        "f(. ..xs);",
        "f(./*split*/..xs);",
        "f(.. /*split*/ .xs);",
        "f(....xs);",
        "new F(. . .xs);",
        "new F(,1);",
        "new F(...);",
    ] {
        for strict in [false, true] {
            let legacy = parser_legacy::Parser::program_context(source, false, strict).unwrap_err();
            let active = parser::Parser::program_context(source, false, strict).unwrap_err();
            assert_eq!(
                legacy.name(),
                "SyntaxError",
                "legacy {source} strict={strict}"
            );
            assert_eq!(
                active.name(),
                "SyntaxError",
                "active {source} strict={strict}"
            );
        }
    }
    // Argument markers must not broaden unrelated array/object/expression syntax.
    for source in ["var x=...xs;", "var x=[...xs];", "var x=({...xs});"] {
        for strict in [false, true] {
            assert!(
                parser_legacy::Parser::program_context(source, false, strict).is_err(),
                "legacy {source} strict={strict}"
            );
            assert!(
                parser::Parser::program_context(source, false, strict).is_err(),
                "active {source} strict={strict}"
            );
        }
    }
}

#[test]
fn call_spread_rest_parameter_grammar_remains_separate_and_unchanged() {
    for source in [
        "function f(...xs){return xs.length;}",
        "function f(a,...xs){return xs.length;}",
        "var f=(...xs)=>xs.length;",
        "var f={m(...xs){return xs.length;}};",
    ] {
        for strict in [false, true] {
            let legacy = parser_legacy::Parser::program_context(source, false, strict)
                .and_then(code::compile)
                .map(|unit| code::canonical(&unit))
                .unwrap();
            let active = parser::Parser::program_context(source, false, strict)
                .map(|unit| code::canonical(&unit))
                .unwrap();
            assert_eq!(active, legacy, "{source} strict={strict}");
        }
    }
    for source in [
        "function f(. . .xs){}",
        "function f(.. .xs){}",
        "function f(....xs){}",
        "function f(...xs,){}",
        "function f(...xs,y){}",
        "function f(...xs=[]){}",
        "var f=(...xs,)=>xs;",
        "var f=(...xs=[])=>xs;",
    ] {
        for strict in [false, true] {
            assert_eq!(
                parser_legacy::Parser::program_context(source, false, strict)
                    .unwrap_err()
                    .name(),
                "SyntaxError",
                "legacy {source} strict={strict}"
            );
            assert_eq!(
                parser::Parser::program_context(source, false, strict)
                    .unwrap_err()
                    .name(),
                "SyntaxError",
                "active {source} strict={strict}"
            );
        }
    }
}
