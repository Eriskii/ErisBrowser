//! New recognition charges are isolated after real lexer setup.
use super::*;
use std::mem::size_of;

fn setup(source: &str) -> Parser<'_> {
    let mut budget = regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    };
    Parser::start(source, false, false, &mut budget).unwrap()
}
fn reset(parser: &mut Parser<'_>, work: usize, heap: usize) {
    parser.compile_budget.steps = work;
    parser.compile_budget.allocated = MAX_HEAP - heap;
}
fn resource<T>(result: Result<T>) {
    match result {
        Err(error) => assert!(error.is_resource_limit(), "{error:?}"),
        Ok(_) => panic!("expected resource"),
    }
}

#[test]
fn argument_recognition_seven_eight_cut_precedes_token_consumption() {
    for source in ["...x", " ...x", "... /*after*/ x"] {
        for (work, ok) in [(7, false), (8, true)] {
            let mut parser = setup(source);
            let offset = parser.tokens[0].offset;
            reset(&mut parser, work, 0);
            let result = parser.argument_ellipsis();
            assert_eq!(result.is_ok(), ok);
            if ok {
                assert_eq!(result.unwrap(), offset);
                assert_eq!(parser.pos, 3);
                assert!(parser.is("x"));
            } else {
                resource(result);
                assert_eq!(parser.pos, 0);
            }
            assert_eq!(
                (parser.compile_budget.steps, parser.compile_budget.allocated),
                (0, MAX_HEAP)
            );
        }
    }
}

#[test]
fn contiguous_recognition_rejects_internal_gaps_but_preserves_numeric_dot_and_rest_fee() {
    for source in [". . .x", "./*inside*/..x", ".. /*inside*/.x", "....x"] {
        let mut parser = setup(source);
        reset(&mut parser, 8, 0);
        let error = parser.argument_ellipsis().unwrap_err();
        assert!(
            error.is_parse_error() && !error.is_unsupported(),
            "{source}: {error}"
        );
        assert_eq!(
            (parser.compile_budget.steps, parser.compile_budget.allocated),
            (0, MAX_HEAP)
        );
    }
    let mut numeric = setup("....5");
    reset(&mut numeric, 8, 0);
    assert_eq!(numeric.argument_ellipsis().unwrap(), 0);
    assert_eq!(numeric.pos, 3);
    assert!(matches!(numeric.tokens[numeric.pos].kind,TokenKind::Number(x) if x==0.5));
    assert_eq!(
        (
            numeric.compile_budget.steps,
            numeric.compile_budget.allocated
        ),
        (0, MAX_HEAP)
    );

    let mut recognition = setup("...tail)");
    reset(&mut recognition, 0, 0);
    assert_eq!(
        recognition
            .contiguous_ellipsis("invalid rest parameter ellipsis")
            .unwrap(),
        0
    );
    assert_eq!(
        (
            recognition.pos,
            recognition.compile_budget.steps,
            recognition.compile_budget.allocated
        ),
        (3, 0, MAX_HEAP)
    );
    // Existing rest tariff is solely copy_identifier1 / four-byte name+24.
    for (work, heap, ok) in [(0, 28, false), (1, 27, false), (1, 28, true)] {
        let mut parser = setup("...tail)");
        reset(&mut parser, work, heap);
        let result = parser.rest_parameter(false);
        assert_eq!(result.is_ok(), ok);
        if ok {
            let parameter = result.unwrap();
            assert!(parameter.rest);
            assert_eq!(parameter.name, "tail");
            assert!(parameter.initializer.is_none());
            assert!(parser.is(")"));
        } else {
            resource(result);
        }
        assert_eq!(parser.compile_budget.steps, 0);
        assert_eq!(
            parser.compile_budget.allocated,
            if work == 0 {
                MAX_HEAP - heap
            } else if heap == 27 {
                MAX_HEAP + 1
            } else {
                MAX_HEAP
            }
        );
    }
}

#[test]
fn argument_list_growth_and_marker_parent_selection_keep_existing_record_edges() {
    let bytes = 32 + size_of::<ExprId>();
    for (work, heap, ok) in [(1, bytes, false), (2, bytes - 1, false), (2, bytes, true)] {
        let mut parser = setup("x");
        let id = parser.emit_expr(Expr::Literal(Value::Undefined)).unwrap();
        let mut edges = Vec::new();
        reset(&mut parser, work, heap);
        let result = push(&mut edges, id, &mut parser.compile_budget);
        assert_eq!(result.is_ok(), ok);
        if ok {
            assert_eq!(edges, [id]);
        } else {
            resource(result);
            assert!(edges.is_empty());
        }
        assert_eq!(parser.compile_budget.steps, 0);
        assert_eq!(
            parser.compile_budget.allocated,
            if work == 1 {
                MAX_HEAP - heap
            } else if heap < bytes {
                MAX_HEAP + 1
            } else {
                MAX_HEAP
            }
        );
    }
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    assert_eq!(size_of::<ExprId>(), 8);
    for (source, spread, construct) in [
        ("f(1,)", false, false),
        ("new C(1,)", false, true),
        ("f(1,...x,)", true, false),
        ("new C(...x,)", true, true),
    ] {
        let unit = Parser::program(source).unwrap();
        let Stmt::Expr(id) = unit.stmt(unit.body[0]) else {
            panic!("expression")
        };
        let (items, is_spread, is_new) = match unit.expr(*id) {
            Expr::Call(_, items) => (items, false, false),
            Expr::New(_, items) => (items, false, true),
            Expr::CallSpread(_, items) => (items, true, false),
            Expr::NewSpread(_, items) => (items, true, true),
            _ => panic!("call/new parent"),
        };
        assert_eq!((is_spread, is_new), (spread, construct));
        assert_eq!(
            items
                .iter()
                .filter(|id| matches!(unit.expr(**id), Expr::SpreadArgument(_)))
                .count(),
            usize::from(spread)
        );
    }
}
