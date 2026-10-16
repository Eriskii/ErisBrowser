//! Class grammar contracts inspected through flat records, without executing JS.
use super::*;
use crate::script::code::{ClassElement, ClassElementKind, ClassKind};
use std::mem::size_of;

fn budget() -> regexp::Budget {
    regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    }
}

#[test]
fn class_private_parser_constructor_tags_and_member_edges_are_distinct() {
    for (source, kind, derived, explicit) in [
        ("class C {}", ClassKind::DefaultBase, false, false),
        (
            "class C extends null {}",
            ClassKind::DefaultDerived,
            true,
            false,
        ),
        (
            "class C { constructor(x) {} }",
            ClassKind::ExplicitBase,
            false,
            true,
        ),
        (
            "class C extends Object { constructor(x) { super(x); } }",
            ClassKind::ExplicitDerived,
            true,
            true,
        ),
    ] {
        let unit = Parser::program(source).unwrap();
        let Stmt::Class(id) = unit.stmt(unit.body[0]) else {
            panic!("class statement");
        };
        let class = unit.class(*id);
        assert_eq!(class.name.as_deref(), Some("C"));
        assert_eq!(class.heritage.is_some(), derived);
        assert!(class.elements.is_empty());
        let constructor = unit.function(class.constructor);
        assert_eq!(constructor.class_kind, Some(kind));
        assert!(
            constructor.strict
                && constructor.constructable
                && !constructor.arrow
                && !constructor.self_name
        );
        assert!(constructor.name.is_none());
        assert_eq!(constructor.params.len(), usize::from(explicit));
        if !explicit {
            assert!(constructor.body.is_empty());
        }
    }
    let unit = Parser::program("class C extends Object { constructor(x,...rest){super(x,...rest);} m(a){} get x(){} set x(v){} static s(){} static [key](){} } function outside(){} (class {});").unwrap();
    let Stmt::Class(id) = unit.stmt(unit.body[0]) else {
        panic!("class");
    };
    let class = unit.class(*id);
    assert_eq!(class.elements.len(), 5);
    for (element, (kind, is_static)) in class.elements.iter().zip([
        (ClassElementKind::Method, false),
        (ClassElementKind::Getter, false),
        (ClassElementKind::Setter, false),
        (ClassElementKind::Method, true),
        (ClassElementKind::Method, true),
    ]) {
        assert_eq!((element.kind, element.is_static), (kind, is_static));
        let function = unit.function(element.function);
        assert!(
            function.strict && !function.arrow && !function.self_name && !function.constructable
        );
        assert!(function.class_kind.is_none());
    }
    for (element, name) in class.elements[..4].iter().zip(["m", "x", "x", "s"]) {
        assert!(
            matches!(&element.name, code::PropertyName::Literal(key) if key == &JsString::from(name))
        );
    }
    let code::PropertyName::Computed(key) = &class.elements[4].name else {
        panic!("computed key edge");
    };
    assert!(matches!(unit.expr(*key), Expr::Ident(name) if name == "key"));
    let constructor = unit.function(class.constructor);
    let Stmt::Expr(call) = unit.stmt(constructor.body[0]) else {
        panic!("super expression");
    };
    let Expr::SuperCallSpread(arguments) = unit.expr(*call) else {
        panic!("spread super record");
    };
    assert_eq!(arguments.len(), 2);
    assert!(matches!(unit.expr(arguments[0]), Expr::Ident(name) if name == "x"));
    let Expr::SpreadArgument(rest) = unit.expr(arguments[1]) else {
        panic!("spread marker");
    };
    assert!(matches!(unit.expr(*rest), Expr::Ident(name) if name == "rest"));
    let Stmt::Function(_, ordinary) = unit.stmt(unit.body[1]) else {
        panic!("ordinary function");
    };
    assert!(!unit.function(*ordinary).strict && unit.function(*ordinary).class_kind.is_none());
    let Stmt::Expr(anonymous) = unit.stmt(unit.body[2]) else {
        panic!("class expression");
    };
    assert!(unit.anonymous_class(*anonymous).is_some());
    assert!(!unit.anonymous(*anonymous));
}

#[test]
fn class_private_parser_context_restores_on_syntax_unsupported_and_terminal_refusal() {
    for source in [
        "class C { m(){ super(); } }",
        "class C { m( }",
        "class C { field; }",
    ] {
        let mut ledger = budget();
        let mut parser = Parser::start(source, false, false, &mut ledger).unwrap();
        parser.allow_in = false;
        let before = (
            parser.allow_in,
            parser.strict,
            parser.super_call_allowed,
            parser.in_class,
        );
        let error = parser.parse_body().unwrap_err();
        if source.contains("field") {
            assert!(error.is_unsupported());
        } else {
            assert!(error.is_parse_error(), "{error:?}");
        }
        assert_eq!(
            (
                parser.allow_in,
                parser.strict,
                parser.super_call_allowed,
                parser.in_class
            ),
            before
        );
        assert!(parser.compile_budget.steps < ledger.steps);
        assert!(parser.compile_budget.allocated >= ledger.allocated);
    }
    // Each semicolon needs its own charged transition. Sixty-four units cannot
    // scan 200 semicolons, independently of additional header/driver costs.
    let source = format!("class C {{ {} }}", ";".repeat(200));
    let mut ledger = budget();
    let mut parser = Parser::start(&source, false, false, &mut ledger).unwrap();
    parser.compile_budget.steps = 64;
    let before = (
        parser.allow_in,
        parser.strict,
        parser.super_call_allowed,
        parser.in_class,
    );
    assert!(parser.parse_body().unwrap_err().is_resource_limit());
    assert_eq!(parser.compile_budget.steps, 0);
    assert!(parser.pos > 2 && parser.pos < parser.tokens.len());
    assert_eq!(
        (
            parser.allow_in,
            parser.strict,
            parser.super_call_allowed,
            parser.in_class
        ),
        before
    );
}

#[test]
fn class_private_parser_element_vector_refuses_before_retaining_key_or_function_edge() {
    let unit = Parser::program("class C { m(){} }").unwrap();
    let Stmt::Class(id) = unit.stmt(unit.body[0]) else {
        panic!("class");
    };
    let function = unit.class(*id).elements[0].function;
    let function_address = unit.function(function) as *const code::Function;
    let bytes = 32 + size_of::<ClassElement>();
    for (work, heap, ok) in [(1, bytes, false), (2, bytes - 1, false), (2, bytes, true)] {
        let mut elements = Vec::new();
        let mut ledger = budget();
        ledger.steps = work;
        ledger.allocated = MAX_HEAP - heap;
        let result = push(
            &mut elements,
            ClassElement {
                name: code::PropertyName::Literal("m".into()),
                function,
                kind: ClassElementKind::Method,
                is_static: true,
            },
            &mut ledger,
        );
        if ok {
            result.unwrap();
        } else {
            assert!(result.unwrap_err().is_resource_limit());
        }
        assert_eq!(elements.len(), usize::from(ok));
        if ok {
            assert_eq!(elements[0].function, function);
            assert!(elements[0].is_static);
        }
        assert_eq!(
            unit.function(function) as *const code::Function,
            function_address
        );
        assert_eq!(ledger.steps, 0);
        assert_eq!(
            ledger.allocated,
            if work == 1 {
                MAX_HEAP - heap
            } else {
                MAX_HEAP - heap + bytes
            }
        );
    }
}
