//! Frozen owner, Sequence and restoration controls through existing parser APIs.
use super::super::{Kind, Output, Transition, enter, run};
use super::*;
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
fn resource<T>(result: Result<T>) {
    let error = take_error(result);
    assert!(error.is_resource_limit(), "{error:?}");
}
fn paid_step(parser: &mut Parser<'_>, frame: Frame, output: Option<Output>) -> Result<Transition> {
    // Exactly run's existing pre-dispatch debit; no alternate driver loop.
    parser.compile_budget.work(1).map_err(regexp_error)?;
    step(parser, frame, output)
}
fn expression_id(output: Output) -> ExprId {
    let Output::Expression(id) = output else {
        panic!("expression")
    };
    id
}
fn completed_id(transition: Transition) -> ExprId {
    let Transition::Done(output) = transition else {
        panic!("done")
    };
    expression_id(output)
}
fn computed_owner(parser: &mut Parser<'_>, construct: bool) -> (Frame, Kind) {
    let base = parser.emit_expr(Expr::Literal(Value::Undefined)).unwrap();
    let transition = paid_step(
        parser,
        if construct {
            Frame::NewHead
        } else {
            Frame::Base(false)
        },
        Some(Output::Expression(base)),
    )
    .unwrap();
    let Transition::Child(Kind::Expression(owner), child) = transition else {
        panic!("computed child")
    };
    assert_eq!(matches!(&owner, Frame::Computed(..)), !construct);
    assert_eq!(matches!(&owner, Frame::NewComputed(..)), construct);
    assert!(matches!(&child, Kind::Expression(Frame::Sequence)));
    (owner, child)
}
fn conditional_owner(parser: &mut Parser<'_>) -> (Frame, Kind) {
    let left = parser.emit_expr(Expr::Literal(Value::Bool(true))).unwrap();
    let transition = paid_step(
        parser,
        Frame::AssignmentHead,
        Some(Output::Expression(left)),
    )
    .unwrap();
    let Transition::Child(Kind::Expression(owner), child) = transition else {
        panic!("conditional child")
    };
    assert!(matches!(&owner, Frame::ConditionalYes(..)));
    assert!(matches!(&child, Kind::Expression(Frame::Assignment)));
    (owner, child)
}
fn call_root(parser: &mut Parser<'_>) -> Kind {
    let base = parser.emit_expr(Expr::Literal(Value::Undefined)).unwrap();
    Frame::PostfixResume(Postfix {
        value: base,
        chain: 0,
        async_call: false,
        allow_update: true,
    })
    .into()
}
fn assert_initial_four_slots() {
    // Exact whole-run cuts below apply to this observed physical capacity.
    // Spare physical capacity would change growth timing, not the fee rule.
    let mut parser = setup("");
    let mut frames = Vec::new();
    enter(&mut parser, &mut frames, Frame::Primary.into()).unwrap();
    assert_eq!((frames.len(), frames.capacity()), (1, 4));
}

#[test]
fn computed_entries_enable_in_and_schedule_existing_sequence() {
    for initial in [false, true] {
        for construct in [false, true] {
            for source in ["[1]", "[1,2]"] {
                let mut parser = setup(source);
                parser.allow_in = initial;
                let _ = computed_owner(&mut parser, construct);
                assert!(parser.allow_in);
                assert!(
                    matches!(&parser.tokens[parser.pos].kind, TokenKind::Number(value) if *value == 1.0)
                );
                assert_eq!(parser.tokens[parser.pos].offset, 1);
            }
        }
    }
}

#[test]
fn computed_close_restores_before_member_and_constructor_work_refusals() {
    for initial in [false, true] {
        for construct in [false, true] {
            for (source, sequence) in [("[1]", false), ("[1,2]", true)] {
                // 1 refuses Member; new-only 2 refuses New after Member;
                // final 2/3 admits the complete warm resume.
                let cuts: &[usize] = if construct { &[1, 2, 3] } else { &[1, 2] };
                for &work in cuts {
                    let mut parser = setup(source);
                    parser.allow_in = initial;
                    let (owner, child) = computed_owner(&mut parser, construct);
                    assert!(parser.allow_in);
                    let output = run(&mut parser, child).unwrap();
                    let key = match &output {
                        Output::Expression(id) => *id,
                        _ => panic!("key"),
                    };
                    assert_eq!(matches!(parser.unit.expr(key), Expr::Sequence(_)), sequence);
                    reset(&mut parser, work, 0);
                    let result = paid_step(&mut parser, owner, Some(output));
                    if work < 2 + usize::from(construct) {
                        resource(result);
                    } else {
                        let id = completed_id(result.unwrap());
                        let member = if construct {
                            let Expr::New(member, arguments) = parser.unit.expr(id) else {
                                panic!("new")
                            };
                            assert!(arguments.is_empty());
                            *member
                        } else {
                            id
                        };
                        let Expr::Member(_, actual_key) = parser.unit.expr(member) else {
                            panic!("member")
                        };
                        assert_eq!(*actual_key, key);
                    }
                    assert_eq!(parser.allow_in, initial);
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
fn computed_close_restores_before_exact_or_one_short_member_page() {
    let page = 32 + 128 * size_of::<Expr>();
    for initial in [false, true] {
        for construct in [false, true] {
            for refuse in [false, true] {
                let mut parser = setup("[1]");
                parser.allow_in = initial;
                let (owner, child) = computed_owner(&mut parser, construct);
                let output = run(&mut parser, child).unwrap();
                // Paid base+literal are exactly two records. Complete the
                // original 128-record page without reading sibling-private storage.
                for _ in 2..128 {
                    parser.emit_expr(Expr::Literal(Value::Null)).unwrap();
                }
                // dispatch1 + Member append1 + new page1; New append1 only
                // after successful Member publication. Directory already has spare.
                let work = 3 + usize::from(construct && !refuse);
                reset(&mut parser, work, page - usize::from(refuse));
                let result = paid_step(&mut parser, owner, Some(output));
                if refuse {
                    resource(result);
                } else {
                    completed_id(result.unwrap());
                }
                assert_eq!(parser.allow_in, initial);
                assert!(parser.done());
                assert_eq!(parser.compile_budget.steps, 0);
                assert_eq!(
                    parser.compile_budget.allocated,
                    MAX_HEAP + usize::from(refuse)
                );
            }
        }
    }
}

#[test]
fn conditional_middle_enables_in_and_colon_restores_before_false_child() {
    let root_bytes = 32 + 4 * size_of::<Kind>();
    for initial in [false, true] {
        for refuse in [false, true] {
            let mut parser = setup("?1:0");
            parser.allow_in = initial;
            let (owner, child) = conditional_owner(&mut parser);
            assert!(parser.allow_in);
            assert_eq!(parser.tokens[parser.pos].offset, 1);
            let yes = run(&mut parser, child).unwrap();
            assert!(parser.is(":"));
            reset(&mut parser, 1, 0);
            let Transition::Child(Kind::Expression(no_owner), no_child) =
                paid_step(&mut parser, owner, Some(yes)).unwrap()
            else {
                panic!("false child")
            };
            assert!(matches!(&no_owner, Frame::ConditionalNo(..)));
            assert!(matches!(&no_child, Kind::Expression(Frame::Assignment)));
            assert_eq!(parser.allow_in, initial);
            assert_eq!(parser.tokens[parser.pos].offset, 3);
            assert_eq!(
                (parser.compile_budget.steps, parser.compile_budget.allocated),
                (0, MAX_HEAP)
            );
            if refuse {
                reset(&mut parser, 0, root_bytes);
                resource(run(&mut parser, no_child));
                assert_eq!(parser.tokens[parser.pos].offset, 3);
                assert_eq!(
                    (parser.compile_budget.steps, parser.compile_budget.allocated),
                    (0, MAX_HEAP - root_bytes)
                );
            } else {
                reset(&mut parser, 3, root_bytes);
                let no = run(&mut parser, no_child).unwrap();
                assert_eq!(
                    (parser.compile_budget.steps, parser.compile_budget.allocated),
                    (0, MAX_HEAP)
                );
                reset(&mut parser, 2, 0);
                let id = completed_id(paid_step(&mut parser, no_owner, Some(no)).unwrap());
                assert!(matches!(parser.unit.expr(id), Expr::Conditional(..)));
                assert!(parser.done());
                assert_eq!(
                    (parser.compile_budget.steps, parser.compile_budget.allocated),
                    (0, MAX_HEAP)
                );
            }
            assert_eq!(parser.allow_in, initial);
        }
    }
}

#[test]
fn nested_expression_owners_restore_outer_in_separator_and_middle_context() {
    for initial in [false, true] {
        for source in [
            "obj['x' in o].slot in o",
            "new obj['x' in o].slot in o",
            "true?1:false?0:1 in o",
        ] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            run(&mut parser, Frame::Sequence.into()).unwrap();
            assert_eq!(parser.allow_in, initial);
            assert_eq!(parser.done(), initial);
            if !initial {
                assert!(parser.is("in"));
            }
        }
        for source in [
            "obj[1,'x' in o]",
            "new obj[1,'x' in o]",
            "true?true?1:'x' in o:0",
        ] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            run(&mut parser, Frame::Sequence.into()).unwrap();
            assert!(parser.done());
            assert_eq!(parser.allow_in, initial);
        }
    }
}

#[test]
fn computed_and_middle_run_errors_restore_entry_without_rewriting_error() {
    for initial in [false, true] {
        for (source, message) in [
            ("obj[1", "expected ']'"),
            ("new obj[1", "expected ']'"),
            ("true?1", "expected ':'"),
        ] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            let error = take_error(run(&mut parser, Frame::Sequence.into()));
            assert!(error.is_parse_error() && !error.is_unsupported());
            assert_eq!(error.message, message);
            assert_eq!(error.offset, Some(source.len()));
            assert!(parser.done());
            assert_eq!(parser.allow_in, initial);
        }
        for source in [
            "obj[function*(){}]",
            "new obj[function*(){}]",
            "true?function*(){}:0",
            "false?0:function*(){}",
        ] {
            let mut parser = setup(source);
            parser.allow_in = initial;
            let error = take_error(run(&mut parser, Frame::Sequence.into()));
            assert!(error.is_unsupported());
            assert_eq!(error.message, "generator functions are not implemented");
            assert_eq!(error.offset, None);
            assert_eq!(parser.allow_in, initial);
        }
    }
}

#[test]
fn existing_sequence_stages_preserve_cold_spare_and_final_record_cuts() {
    let one = 32 + size_of::<ExprId>();
    let two = 32 + 2 * size_of::<ExprId>();
    for initial in [false, true] {
        let mut parser = setup("");
        parser.allow_in = initial;
        let first = parser.emit_expr(Expr::Literal(Value::Null)).unwrap();
        reset(&mut parser, 1, 0);
        assert_eq!(
            completed_id(
                paid_step(
                    &mut parser,
                    Frame::SequenceHead,
                    Some(Output::Expression(first))
                )
                .unwrap()
            ),
            first
        );
        assert_eq!(
            (parser.compile_budget.steps, parser.compile_budget.allocated),
            (0, MAX_HEAP)
        );
        assert_eq!(parser.allow_in, initial);

        for (work, heap, succeeds, allocated) in [
            (2, one, false, MAX_HEAP - one),
            (3, one - 1, false, MAX_HEAP + 1),
            (3, one, true, MAX_HEAP),
        ] {
            let mut parser = setup(",");
            parser.allow_in = initial;
            let first = parser.emit_expr(Expr::Literal(Value::Null)).unwrap();
            reset(&mut parser, work, heap);
            let result = paid_step(
                &mut parser,
                Frame::SequenceHead,
                Some(Output::Expression(first)),
            );
            if succeeds {
                let Transition::Child(
                    Kind::Expression(Frame::SequenceNext(items)),
                    Kind::Expression(Frame::Assignment),
                ) = result.unwrap()
                else {
                    panic!("sequence next")
                };
                assert_eq!(items, [first]);
                assert!(items.capacity() >= 1);
            } else {
                resource(result);
            }
            assert!(parser.done());
            assert_eq!(parser.allow_in, initial);
            assert_eq!(
                (parser.compile_budget.steps, parser.compile_budget.allocated),
                (0, allocated)
            );
        }

        for spare in [false, true] {
            for refuse in [false, true] {
                let mut parser = setup(",");
                parser.allow_in = initial;
                let first = parser.emit_expr(Expr::Literal(Value::Null)).unwrap();
                let second = parser.emit_expr(Expr::Literal(Value::Bool(true))).unwrap();
                let Transition::Child(Kind::Expression(Frame::SequenceNext(mut items)), _) =
                    paid_step(
                        &mut parser,
                        Frame::SequenceHead,
                        Some(Output::Expression(first)),
                    )
                    .unwrap()
                else {
                    panic!("natural sequence owner")
                };
                if spare {
                    // Actual paid list growth precedes the isolated cut. Pop
                    // leaves physical spare; no logical-capacity fiction.
                    push(&mut items, second, &mut parser.compile_budget).unwrap();
                    assert_eq!(items.pop(), Some(second));
                    assert!(items.capacity() > items.len());
                }
                assert_eq!(items.len(), 1);
                let grows = items.len() == items.capacity();
                let work = 3 + 2 * usize::from(grows); // dispatch+push+[growth2]+Sequence record
                let heap = if grows { two } else { 0 };
                reset(&mut parser, work - usize::from(refuse), heap);
                let result = paid_step(
                    &mut parser,
                    Frame::SequenceNext(items),
                    Some(Output::Expression(second)),
                );
                if refuse {
                    resource(result);
                } else {
                    let id = completed_id(result.unwrap());
                    let Expr::Sequence(items) = parser.unit.expr(id) else {
                        panic!("sequence record")
                    };
                    assert_eq!(items, &[first, second]);
                }
                assert_eq!(parser.allow_in, initial);
                assert_eq!(
                    (parser.compile_budget.steps, parser.compile_budget.allocated),
                    (0, MAX_HEAP)
                );
            }
        }
    }
}

#[test]
fn computed_run_pays_sequence_heads_and_refuses_growth_before_unary() {
    assert_initial_four_slots();
    let root_bytes = 32 + 4 * size_of::<Kind>();
    let grown_bytes = 32 + 8 * size_of::<Kind>();
    for initial in [false, true] {
        for (work, heap, pos, allocated) in [
            (0, root_bytes, 0, MAX_HEAP - root_bytes),
            (1, root_bytes, 0, MAX_HEAP),
            (2, root_bytes, 1, MAX_HEAP),
            (1, root_bytes - 1, 0, MAX_HEAP + 1),
            (7, root_bytes, 3, MAX_HEAP),
        ] {
            let mut parser = setup("[0]");
            let root = call_root(&mut parser);
            parser.allow_in = initial;
            reset(&mut parser, work, heap);
            resource(run(&mut parser, root));
            assert_eq!((parser.allow_in, parser.pos), (initial, pos));
            assert_eq!(
                (parser.compile_budget.steps, parser.compile_budget.allocated),
                (0, allocated)
            );
        }
        let mut parser = setup("[0]");
        let root = call_root(&mut parser);
        parser.allow_in = initial;
        // Initial1+Postfix1+Sequence1+Assignment1+literal1+
        // SequenceHead1+Computed1+Member1. No vector growth after initial4.
        reset(&mut parser, 8, root_bytes);
        let id = expression_id(run(&mut parser, root).unwrap());
        assert!(matches!(parser.unit.expr(id), Expr::Member(..)));
        assert!(parser.done());
        assert_eq!(parser.allow_in, initial);
        assert_eq!(
            (parser.compile_budget.steps, parser.compile_budget.allocated),
            (0, MAX_HEAP)
        );

        for (work, heap, steps, allocated) in [
            (9, root_bytes + grown_bytes, 0, MAX_HEAP - grown_bytes),
            (
                MAX_STEPS,
                root_bytes,
                MAX_STEPS - 10,
                MAX_HEAP + grown_bytes,
            ),
            (
                MAX_STEPS,
                root_bytes + grown_bytes - 1,
                MAX_STEPS - 10,
                MAX_HEAP + 1,
            ),
        ] {
            let mut parser = setup("[!k]");
            let root = call_root(&mut parser);
            parser.allow_in = initial;
            reset(&mut parser, work, heap);
            resource(run(&mut parser, root));
            assert_eq!(parser.allow_in, initial);
            assert!(parser.is("!"));
            assert_eq!(parser.tokens[parser.pos].offset, 1);
            assert_eq!(
                (parser.compile_budget.steps, parser.compile_budget.allocated),
                (steps, allocated)
            );
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn actual_expression_in_layout_matches_both_selected_mirrors() {
    use std::mem::align_of;
    assert_eq!((size_of::<Frame>(), align_of::<Frame>()), (72, 8));
    assert_eq!((size_of::<Kind>(), align_of::<Kind>()), (120, 8));
    assert_eq!(
        (size_of::<Transition>(), align_of::<Transition>()),
        (240, 8)
    );
    assert_eq!((size_of::<Output>(), align_of::<Output>()), (88, 8));
    assert_eq!(32 + 4 * size_of::<Kind>(), 512);
    assert_eq!(MAX_HEAP / size_of::<Kind>(), 69905);
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[test]
fn conditional_full_compile_tariffs_preserve_observed_parent_totals() {
    // Frozen parent measurements through actual Unit::finish; neither a
    // computed-member total nor a universal two-dispatch growth assumption.
    for (source, work, heap) in [
        ("true?1:2", 43, 23844),
        ("f(true?1:2)", 69, 25982),
        ("for(var x=true?1:2;false;){}", 76, 24393),
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
