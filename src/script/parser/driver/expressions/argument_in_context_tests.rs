//! Frozen context/charge controls through existing parser entry points.
use super::super::{Kind, Output, Transition, functions, run, statements};
use super::*;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use std::mem::align_of;
use std::mem::size_of;

fn setup(source: &str) -> Parser<'_> {
    Parser::start(
        source,
        false,
        false,
        &mut regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        },
    )
    .unwrap()
}
fn reset(parser: &mut Parser<'_>, work: usize, heap: usize) {
    parser.compile_budget.steps = work;
    parser.compile_budget.allocated = MAX_HEAP - heap;
}
fn take_error<T>(result: Result<T>) -> ScriptError {
    match result {
        Err(error) => error,
        Ok(_) => panic!("expected error"),
    }
}
fn resource<T>(result: Result<T>) -> ScriptError {
    let error = take_error(result);
    assert!(error.is_resource_limit(), "{error:?}");
    error
}
fn paid_step(parser: &mut Parser<'_>, frame: Frame, output: Option<Output>) -> Result<Transition> {
    // This is the same dispatch payment made immediately before step by run.
    // It is a fixture boundary, not an alternate continuation driver.
    parser.compile_budget.work(1).map_err(regexp_error)?;
    step(parser, frame, output)
}
fn begin_owner(parser: &mut Parser<'_>, construct: bool) -> Result<Transition> {
    let callee = parser.emit_expr(Expr::Literal(Value::Undefined))?;
    paid_step(
        parser,
        if construct {
            Frame::NewHead
        } else {
            Frame::Base(false)
        },
        Some(Output::Expression(callee)),
    )
}
fn owner_child(transition: Transition, construct: bool) -> (Frame, Kind) {
    let Transition::Child(Kind::Expression(owner), child) = transition else {
        panic!("owner child")
    };
    assert_eq!(matches!(&owner, Frame::Arguments(..)), !construct);
    assert_eq!(matches!(&owner, Frame::NewArguments(..)), construct);
    assert!(matches!(&child, Kind::Expression(Frame::Assignment)));
    (owner, child)
}
fn expression_id(output: Output) -> ExprId {
    let Output::Expression(id) = output else {
        panic!("expression")
    };
    id
}
fn call_root(parser: &mut Parser<'_>) -> Kind {
    let callee = parser.emit_expr(Expr::Literal(Value::Undefined)).unwrap();
    Frame::PostfixResume(Postfix {
        value: callee,
        chain: 0,
        async_call: false,
    })
    .into()
}
fn replace_source<'s>(parser: &mut Parser<'s>, source: &'s str) {
    // Fixture transition between the actual two dynamic-function run calls.
    parser.tokens = lex(source, &mut parser.compile_budget).unwrap();
    parser.source = source;
    parser.pos = 0;
    parser.lex_work = source.len();
}

#[test]
fn argument_owner_entry_enables_in_without_changing_empty_lists() {
    for initial in [false, true] {
        for construct in [false, true] {
            for source in ["(1)", "(...x)"] {
                let mut parser = setup(source);
                parser.allow_in = initial;
                let transition = begin_owner(&mut parser, construct).unwrap();
                let _ = owner_child(transition, construct);
                assert!(
                    parser.allow_in,
                    "{source} new={construct} initial={initial}"
                );
                if source == "(1)" {
                    assert!(
                        matches!(&parser.tokens[parser.pos].kind, TokenKind::Number(value) if *value == 1.0)
                    );
                    assert_eq!(parser.tokens[parser.pos].offset, 1);
                } else {
                    assert!(parser.is("x"));
                    assert_eq!(parser.tokens[parser.pos].offset, 4);
                }
            }
            let mut parser = setup("()");
            parser.allow_in = initial;
            let transition = begin_owner(&mut parser, construct).unwrap();
            assert_eq!(parser.allow_in, initial);
            assert!(parser.done());
            assert_eq!(
                matches!(transition, Transition::Done(Output::Expression(_))),
                construct
            );
        }
        let mut parser = setup("");
        parser.allow_in = initial;
        assert!(matches!(
            begin_owner(&mut parser, true).unwrap(),
            Transition::Done(Output::Expression(_))
        ));
        assert_eq!(parser.allow_in, initial);
    }
}

#[test]
fn argument_close_restores_before_success_or_final_record_refusal() {
    let edge = 32 + size_of::<ExprId>();
    for initial in [false, true] {
        for construct in [false, true] {
            for (source, spread) in [
                ("(1)", false),
                ("(1,)", false),
                ("(...[1])", true),
                ("(...[1],)", true),
            ] {
                for refuse in [false, true] {
                    let mut parser = setup(source);
                    parser.allow_in = initial;
                    let transition = begin_owner(&mut parser, construct).unwrap();
                    let (owner, child) = owner_child(transition, construct);
                    assert!(parser.allow_in);
                    let output = run(&mut parser, child).unwrap();
                    assert!(parser.allow_in);
                    // dispatch1 + optional marker1 + cold edge2 + parent record1.
                    let work = 4 + usize::from(spread) - usize::from(refuse);
                    reset(&mut parser, work, edge);
                    let result = paid_step(&mut parser, owner, Some(output));
                    if refuse {
                        resource(result);
                    } else {
                        let (id, actual_construct) = match result.unwrap() {
                            Transition::Done(Output::Expression(id)) => (id, true),
                            Transition::Again(Kind::Expression(Frame::PostfixResume(state))) => {
                                (state.value, false)
                            }
                            _ => panic!("closed owner"),
                        };
                        assert_eq!(actual_construct, construct);
                        let (items, actual_spread) = match parser.unit.expr(id) {
                            Expr::Call(_, items) | Expr::New(_, items) => (items, false),
                            Expr::CallSpread(_, items) | Expr::NewSpread(_, items) => (items, true),
                            _ => panic!("argument parent"),
                        };
                        assert_eq!(actual_spread, spread);
                        assert_eq!(items.len(), 1);
                        assert_eq!(
                            matches!(parser.unit.expr(items[0]), Expr::SpreadArgument(_)),
                            spread
                        );
                    }
                    assert_eq!(
                        parser.allow_in, initial,
                        "{source} new={construct} refuse={refuse}"
                    );
                    assert!(parser.done());
                    assert_eq!(
                        (parser.compile_budget.steps, parser.compile_budget.allocated),
                        (0, MAX_HEAP)
                    );
                }
            }
        }
    }
}

#[test]
fn argument_context_nesting_restores_outer_in_separator() {
    for initial in [false, true] {
        for source in ["f('x' in o).slot in o", "new C('x' in o).slot in o"] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            let id = expression_id(run(&mut parser, Frame::Sequence.into()).unwrap());
            assert_eq!(parser.allow_in, initial);
            if initial {
                assert!(parser.done());
                assert!(matches!(parser.unit.expr(id), Expr::BinaryChain(..)));
            } else {
                assert!(parser.is("in"));
                assert!(matches!(parser.unit.expr(id), Expr::Member(..)));
            }
        }
        for source in [
            "f('x' in o,g('y' in o),)",
            "new C('x' in o,new D('y' in o),)",
            "f(...'x' in o?[1]:[],2)",
        ] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            run(&mut parser, Frame::Sequence.into()).unwrap();
            assert!(parser.done());
            assert_eq!(parser.allow_in, initial);
        }
        for source in ["f() in o", "new C in o", "new C() in o"] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            run(&mut parser, Frame::Sequence.into()).unwrap();
            assert_eq!(parser.done(), initial);
            assert_eq!(parser.allow_in, initial);
            if !initial {
                assert!(parser.is("in"));
            }
        }
    }
}

#[test]
fn run_errors_restore_entry_flag_without_rewriting_syntax_or_unsupported() {
    for initial in [false, true] {
        for source in ["f((1", "new C((1"] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            let error = take_error(run(&mut parser, Frame::Sequence.into()));
            assert!(error.is_parse_error() && !error.is_unsupported());
            assert_eq!(error.message, "expected ','");
            assert_eq!(error.offset, Some(source.len()));
            assert_eq!(parser.allow_in, initial);
            assert!(parser.done());
        }
        let mut parser = setup("f(1,. . .x)");
        parser.allow_in = initial;
        let error = take_error(run(&mut parser, Frame::Sequence.into()));
        assert!(error.is_parse_error() && !error.is_unsupported());
        assert_eq!(error.message, "invalid argument spread ellipsis");
        assert_eq!(error.offset, Some(6));
        assert_eq!(parser.allow_in, initial);
        let mut parser = setup("f((function*(){}))");
        parser.allow_in = initial;
        let error = take_error(run(&mut parser, Frame::Sequence.into()));
        assert!(error.is_unsupported());
        assert_eq!(error.message, "generator functions are not implemented");
        assert_eq!(error.offset, None);
        assert_eq!(parser.allow_in, initial);
    }
}

#[test]
fn run_root_and_argument_prefix_refusals_keep_original_numeric_charges() {
    let root_bytes = 32 + 4 * size_of::<Kind>();
    for initial in [false, true] {
        for (source, work, heap, pos, allocated) in [
            ("(1)", 0, root_bytes, 0, MAX_HEAP - root_bytes),
            ("(1)", 1, root_bytes, 0, MAX_HEAP),
            ("(1)", 2, root_bytes, 1, MAX_HEAP),
            ("(...x)", 9, root_bytes, 1, MAX_HEAP),
            ("(...x)", 10, root_bytes, 4, MAX_HEAP),
            ("(1)", 1, root_bytes - 1, 0, MAX_HEAP + 1),
        ] {
            let mut parser = setup(source);
            let root = call_root(&mut parser);
            parser.allow_in = initial;
            reset(&mut parser, work, heap);
            resource(run(&mut parser, root));
            assert_eq!(parser.allow_in, initial, "{source} work={work}");
            assert_eq!(parser.pos, pos);
            assert_eq!(
                (parser.compile_budget.steps, parser.compile_budget.allocated),
                (0, allocated)
            );
        }
    }
}

#[test]
fn run_refused_child_and_syntax_storage_restore_only_the_entry_flag() {
    let root_bytes = 32 + 4 * size_of::<Kind>();
    let grown_bytes = 32 + 8 * size_of::<Kind>();
    let edge = 32 + size_of::<ExprId>();
    let page = 32 + 128 * size_of::<Expr>();
    for initial in [false, true] {
        // Existing Cover enables +In inside the root run. The next dispatch
        // refuses before any expression record; zero-budget cleanup must work.
        let mut parser = setup("(");
        parser.allow_in = initial;
        reset(&mut parser, 2, root_bytes);
        resource(run(&mut parser, Frame::Primary.into()));
        assert_eq!((parser.allow_in, parser.pos), (initial, 1));
        assert_eq!(
            (parser.compile_budget.steps, parser.compile_budget.allocated),
            (0, MAX_HEAP)
        );

        // Primary->Assignment->Binary->Unary fills the four-slot vector.
        // Entering Unary's New child needs growth5 and the full eight-slot fee.
        let mut parser = setup("((");
        parser.allow_in = initial;
        reset(&mut parser, MAX_STEPS, root_bytes);
        resource(run(&mut parser, Frame::Primary.into()));
        assert_eq!((parser.allow_in, parser.pos), (initial, 1));
        assert_eq!(
            (parser.compile_budget.steps, parser.compile_budget.allocated),
            (MAX_STEPS - 10, MAX_HEAP + grown_bytes)
        );

        let mut parser = setup("(1)");
        let root = call_root(&mut parser);
        parser.allow_in = initial;
        reset(&mut parser, MAX_STEPS, root_bytes + edge - 1);
        resource(run(&mut parser, root));
        assert_eq!((parser.allow_in, parser.pos), (initial, 2));
        assert_eq!(
            (parser.compile_budget.steps, parser.compile_budget.allocated),
            (MAX_STEPS - 7, MAX_HEAP + 1)
        );

        let mut parser = setup("(1)");
        let root = call_root(&mut parser);
        for _ in 1..128 {
            parser.emit_expr(Expr::Literal(Value::Null)).unwrap();
        }
        parser.allow_in = initial;
        reset(&mut parser, MAX_STEPS, root_bytes + page - 1);
        resource(run(&mut parser, root));
        assert_eq!((parser.allow_in, parser.pos), (initial, 2));
        assert_eq!(
            (parser.compile_budget.steps, parser.compile_budget.allocated),
            (MAX_STEPS - 5, MAX_HEAP + 1)
        );
    }
}

#[test]
fn dynamic_parameters_body_and_finish_preserve_the_successful_handoff() {
    for initial in [false, true] {
        for (params, body, parameter_count) in
            [("", "return 1;", 0), ("a=f('x' in o)", "return a;", 1)]
        {
            let mut parser = setup(params);
            parser.allow_in = initial;
            let Output::Parameters(state) =
                run(&mut parser, functions::Frame::Dynamic.into()).unwrap()
            else {
                panic!("parameters")
            };
            assert!(parser.allow_in && parser.new_target_allowed);
            assert_eq!(parser.function_depth, 1);
            replace_source(&mut parser, body);
            let Output::Body(body) = run(
                &mut parser,
                statements::BodyFrame::start(false, true).into(),
            )
            .unwrap() else {
                panic!("body")
            };
            assert!(parser.allow_in && parser.new_target_allowed);
            assert_eq!(parser.function_depth, 1);
            let Transition::Done(Output::Function(function)) =
                functions::finish(&mut parser, state, body).unwrap()
            else {
                panic!("function")
            };
            assert_eq!(function.params.len(), parameter_count);
            assert_eq!(function.body.len(), 1);
            assert_eq!(parser.allow_in, initial);
            assert_eq!(parser.function_depth, 0);
            assert!(!parser.new_target_allowed);
        }
    }
}

#[test]
fn dynamic_errors_restore_each_run_entry_not_the_whole_function_context() {
    let root_bytes = 32 + 4 * size_of::<Kind>();
    for initial in [false, true] {
        for work in [1, 2] {
            let mut parser = setup("");
            parser.allow_in = initial;
            reset(&mut parser, work, root_bytes);
            let result = run(&mut parser, functions::Frame::Dynamic.into());
            if work == 1 {
                resource(result);
                assert_eq!((parser.allow_in, parser.function_depth), (initial, 0));
            } else {
                assert!(matches!(result.unwrap(), Output::Parameters(_)));
                assert!(parser.allow_in);
                assert_eq!(parser.function_depth, 1);
            }
            assert_eq!(
                (parser.compile_budget.steps, parser.compile_budget.allocated),
                (0, MAX_HEAP)
            );
        }
        for (params, unsupported) in [("a=(", false), ("a=function*(){}", true)] {
            let mut parser = setup(params);
            parser.allow_in = initial;
            let error = take_error(run(&mut parser, functions::Frame::Dynamic.into()));
            assert_eq!(error.is_unsupported(), unsupported);
            assert_eq!(error.is_parse_error(), !unsupported);
            assert_eq!(parser.allow_in, initial);
            // No full Context rollback or failed-parser reuse is promised.
            assert_eq!(parser.function_depth, 1);
            assert!(parser.new_target_allowed);
        }
        for (body, unsupported) in [("for(f(;", false), ("for(function*(){};", true)] {
            let mut parser = setup("");
            parser.allow_in = initial;
            let Output::Parameters(state) =
                run(&mut parser, functions::Frame::Dynamic.into()).unwrap()
            else {
                panic!("parameters")
            };
            assert!(parser.allow_in);
            replace_source(&mut parser, body);
            let error = take_error(run(
                &mut parser,
                statements::BodyFrame::start(false, true).into(),
            ));
            assert_eq!(error.is_unsupported(), unsupported);
            assert_eq!(error.is_parse_error(), !unsupported);
            assert!(parser.allow_in); // This run began true, even when State saved false.
            assert_eq!(parser.function_depth, 1);
            assert!(parser.new_target_allowed);
            drop(state);
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn actual_argument_in_layout_matches_both_selected_mirrors() {
    assert_eq!((size_of::<Frame>(), align_of::<Frame>()), (72, 8));
    assert_eq!((size_of::<Kind>(), align_of::<Kind>()), (112, 8));
    assert_eq!(
        (size_of::<Transition>(), align_of::<Transition>()),
        (224, 8)
    );
    assert_eq!((size_of::<Output>(), align_of::<Output>()), (88, 8));
    assert_eq!(32 + 4 * size_of::<Kind>(), 480);
    assert_eq!(MAX_HEAP / size_of::<Kind>(), 74898);
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn full_compile_tariffs_match_both_preimplementation_parent_observations() {
    // Frozen before candidate code: both 1.88/default and 1.99/host+Vulkan.
    // Includes real lexing, parse_context and final Unit publication.
    for (source, work, heap) in [
        ("f(1)", 43, 23770),
        ("f(1,2,)", 49, 23906),
        ("new C(1)", 55, 25637),
        ("new C(1,2,)", 61, 25773),
        ("f(...x)", 71, 25772),
        ("new C(...x,)", 75, 25855),
        ("for(f(0).slot in {a:1}){}", 93, 26268),
    ] {
        for strict in [false, true] {
            let parser = Parser::parse_context(source, false, strict).unwrap();
            let mut ledger = parser.compile_budget;
            let unit = parser.unit.finish(&mut ledger).unwrap();
            assert!(!unit.body.is_empty());
            assert_eq!(
                (MAX_STEPS - ledger.steps, ledger.allocated),
                (work, heap),
                "{source} strict={strict}"
            );
        }
    }
}
