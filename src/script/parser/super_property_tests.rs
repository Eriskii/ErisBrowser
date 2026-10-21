//! Parser-level flat-edge and grammar-context contracts; no JavaScript execution.
use super::*;

fn method_body(unit: &code::Unit) -> &[StmtId] {
    let Stmt::Class(id) = unit.stmt(unit.body[0]) else {
        panic!("class statement");
    };
    &unit.function(unit.class(*id).elements[0].function).body
}
fn statement_expr(unit: &code::Unit, id: StmtId) -> ExprId {
    let Stmt::Expr(expression) = unit.stmt(id) else {
        panic!("expression statement");
    };
    *expression
}
fn property_key(unit: &code::Unit, id: ExprId) -> ExprId {
    let Expr::SuperProperty(key) = unit.expr(id) else {
        panic!("super property edge");
    };
    *key
}

#[test]
fn super_property_parser_raw_keys_and_reference_consumers_keep_distinct_edges() {
    let unit = Parser::program(
        "class C {m(){super.x;super[key];super.m(a);new super.C(a);delete super[key];super[key]=rhs;super.x++;for(super.x in obj){}for(super[key] of values){}super.x.y;(super.m)(a);(0,super.m)(a);}}",
    ).unwrap();
    let body = method_body(&unit);
    assert_eq!(body.len(), 12);
    let key = property_key(&unit, statement_expr(&unit, body[0]));
    assert!(
        matches!(unit.expr(key), Expr::Literal(Value::String(name)) if name == &JsString::from("x"))
    );
    let key = property_key(&unit, statement_expr(&unit, body[1]));
    assert!(matches!(unit.expr(key), Expr::Ident(name) if name == "key"));
    for (index, construct) in [(2, false), (3, true), (10, false)] {
        let expression = unit.expr(statement_expr(&unit, body[index]));
        let (callee, args) = match (construct, expression) {
            (false, Expr::Call(callee, args)) | (true, Expr::New(callee, args)) => (callee, args),
            _ => panic!("call/construct must retain its own record"),
        };
        property_key(&unit, *callee);
        assert_eq!(args.len(), 1);
        assert!(matches!(unit.expr(args[0]), Expr::Ident(name) if name == "a"));
    }
    let Expr::Unary(op, target) = unit.expr(statement_expr(&unit, body[4])) else {
        panic!("delete record");
    };
    assert_eq!(op, "delete");
    property_key(&unit, *target);
    let Expr::Assign(op, target, value) = unit.expr(statement_expr(&unit, body[5])) else {
        panic!("assignment record");
    };
    assert_eq!(op, "=");
    property_key(&unit, *target);
    assert!(matches!(unit.expr(*value), Expr::Ident(name) if name == "rhs"));
    let Expr::Update(target, delta, prefix) = unit.expr(statement_expr(&unit, body[6])) else {
        panic!("update record");
    };
    assert_eq!((*delta, *prefix), (1.0, false));
    property_key(&unit, *target);
    for (index, of) in [(7, false), (8, true)] {
        let binding = match (of, unit.stmt(body[index])) {
            (false, Stmt::ForIn(binding, _, _)) | (true, Stmt::ForOf(binding, _, _)) => binding,
            _ => panic!("iteration record"),
        };
        let ForBinding::Target(target) = binding else {
            panic!("reference loop target");
        };
        property_key(&unit, *target);
    }
    let Expr::Member(object, _) = unit.expr(statement_expr(&unit, body[9])) else {
        panic!("ordinary member above super reference");
    };
    property_key(&unit, *object);
    let Expr::Call(callee, _) = unit.expr(statement_expr(&unit, body[11])) else {
        panic!("extracted call");
    };
    let Expr::Sequence(items) = unit.expr(*callee) else {
        panic!("comma keeps its value-producing boundary");
    };
    assert_eq!(items.len(), 2);
    property_key(&unit, items[1]);
}

#[test]
fn super_property_parser_permissions_follow_class_methods_and_lexical_arrows() {
    for source in [
        "class C {constructor(x=super.x){super.x;}}",
        "class C extends Object {constructor(x=()=>super.x){super();super.x;}m(x=super.x){return ()=>super[x];}get x(){return super.x;}set x(v){super.x=v;}static m(x=super.x){return ()=>super[x];}}",
        "class C {m(){return class D extends super.C {[super.k](){return super.x;}};}}",
        "class C {m(){return function(){return class D {m(){return super.x;}};};}}",
        "({m(){return class C {m(){return super.x;}};}})",
        "class C {m(){return {[super.key](){return 1;}};}}",
        "class C {m(){return (x=super.x)=>super[x];}}",
        "class C {m(){++super.x;--super[x];super.x+=1;super[x]&&=1;new super.C;new super.C(...args);super.m(...args);}}",
    ] {
        Parser::program(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
    for source in [
        "super.x;",
        "function f(){return super.x;}",
        "function f(x=super.x){}",
        "class C {m(){return function(){return super.x;};}}",
        "class C {m(){return function(x=super.x){};}}",
        "class C {m(){function f(){return ()=>super.x;}}}",
        "class C {m(){super();}}",
        "class C {m(){return super;}}",
        "class C extends Object {constructor(){new super();}}",
        "class C extends super.x {}",
        "class C {[super.x](){}}",
        "class C {m(){super.x;} } super.x;",
    ] {
        let error = Parser::program(source).unwrap_err();
        assert!(error.is_parse_error(), "{source}: {error:?}");
    }
}

#[test]
fn super_property_parser_object_methods_are_admitted_and_class_deferrals_remain() {
    for source in [
        "({m(){return super.x;}})",
        "({m(x=super.x){}})",
        "({get x(){return super.x;}})",
        "({set x(v){super.x=v;}})",
        "({m(){return ()=>super.x;}})",
        "class C {m(){return {m(){return super.x;}};}}",
    ] {
        Parser::program(source).unwrap_or_else(|error| panic!("{source}: {error:?}"));
    }
    for source in [
        "class C {field;}",
        "class C {#field;}",
        "class C {static {}}",
        "class C {*m(){return super.x;}}",
        "class C {async m(){return super.x;}}",
    ] {
        let error = Parser::program(source).unwrap_err();
        assert!(error.is_unsupported(), "{source}: {error:?}");
    }
    // A nested ordinary function does not inherit the method permission.
    let error = Parser::program("({m(){return function(){return super.x;};}})").unwrap_err();
    assert!(error.is_parse_error(), "{error:?}");
}

#[test]
fn super_property_parser_context_restores_on_syntax_unsupported_and_work_refusal() {
    for source in [
        "class C {m(){return super[;}}",
        "class C {m(){return class D {field;};}}",
        "class C {m(){return super[key];}} trailing = ;",
    ] {
        let mut budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        let mut parser = Parser::start(source, false, false, &mut budget).unwrap();
        parser.allow_in = false;
        let error = parser.parse_body().unwrap_err();
        assert!(error.is_parse_error() || error.is_unsupported());
        assert!(!parser.allow_in && !parser.strict && !parser.in_class);
        assert!(!parser.super_call_allowed);
        assert!(!parser.super_property_allowed && !parser.super_property_deferred);
        assert!(parser.compile_budget.steps < budget.steps);
    }
    let source = format!("class C {{m(){{{}return super[key];}}}}", ";".repeat(200));
    let mut budget = regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    };
    let mut parser = Parser::start(&source, false, false, &mut budget).unwrap();
    parser.compile_budget.steps = 64;
    let error = parser.parse_body().unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(parser.compile_budget.steps, 0);
    assert!(!parser.strict && !parser.in_class && !parser.super_call_allowed);
    assert!(!parser.super_property_allowed && !parser.super_property_deferred);
}

#[test]
fn object_super_parser_method_records_own_defaults_and_arrows_without_class_strictness() {
    let source = "({[key](x=super[key]){super.m(x);return ()=>super[key];},get x(){return super.x;},set x(v){super.x=v;},ordinary:function(){return 1;}});";
    for strict in [false, true] {
        let unit = Parser::program_context(source, false, strict).unwrap();
        let Expr::Object(entries) = unit.expr(statement_expr(&unit, unit.body[0])) else {
            panic!("object expression");
        };
        assert_eq!(entries.len(), 4);
        let code::PropertyName::Computed(key) = &entries[0].0 else {
            panic!("computed definition key");
        };
        assert!(matches!(unit.expr(*key), Expr::Ident(name) if name == "key"));
        let ObjectEntry::Method(id) = &entries[0].1 else {
            panic!("concise method");
        };
        let method = unit.function(*id);
        assert!(!method.arrow && !method.constructable && method.class_kind.is_none());
        assert_eq!(method.strict, strict);
        assert_eq!(method.params.len(), 1);
        property_key(&unit, method.params[0].initializer.unwrap());
        let Expr::Call(callee, arguments) = unit.expr(statement_expr(&unit, method.body[0])) else {
            panic!("super method call");
        };
        property_key(&unit, *callee);
        assert_eq!(arguments.len(), 1);
        assert!(matches!(unit.expr(arguments[0]), Expr::Ident(name) if name == "x"));
        let Stmt::Return(Some(arrow)) = unit.stmt(method.body[1]) else {
            panic!("returned lexical arrow");
        };
        let Expr::Function(arrow) = unit.expr(*arrow) else {
            panic!("arrow function record");
        };
        let arrow = unit.function(*arrow);
        assert!(arrow.arrow && !arrow.constructable && arrow.class_kind.is_none());
        assert_eq!(arrow.strict, strict);
        let Stmt::Return(Some(target)) = unit.stmt(arrow.body[0]) else {
            panic!("arrow return");
        };
        property_key(&unit, *target);
        for (index, setter) in [(1, false), (2, true)] {
            let ObjectEntry::Accessor(id, is_setter) = &entries[index].1 else {
                panic!("accessor function record");
            };
            assert_eq!(*is_setter, setter);
            let accessor = unit.function(*id);
            assert!(!accessor.arrow && !accessor.constructable && accessor.class_kind.is_none());
            assert_eq!(accessor.strict, strict);
            assert_eq!(accessor.params.len(), usize::from(setter));
            if setter {
                let Expr::Assign(op, target, _) =
                    unit.expr(statement_expr(&unit, accessor.body[0]))
                else {
                    panic!("setter assignment");
                };
                assert_eq!(op, "=");
                property_key(&unit, *target);
            } else {
                let Stmt::Return(Some(target)) = unit.stmt(accessor.body[0]) else {
                    panic!("getter return");
                };
                property_key(&unit, *target);
            }
        }
        let ObjectEntry::Data(value) = &entries[3].1 else {
            panic!("ordinary function-valued property");
        };
        let Expr::Function(id) = unit.expr(*value) else {
            panic!("ordinary function record");
        };
        let ordinary = unit.function(*id);
        assert!(ordinary.constructable && !ordinary.arrow && ordinary.class_kind.is_none());
        assert_eq!(ordinary.strict, strict);
    }
}

#[test]
fn object_super_parser_contexts_inherit_only_through_selected_syntax() {
    for strict in [false, true] {
        for source in [
            "({m(x=()=>super.x){return (y=super[y])=>super[x];}});",
            "({m(){return {[super.key](x=super.x){return super[x];},get [super.getter](){return super.x;}};}});",
            "({m(){return class D extends super.C {[super.key](){return super.x;}};}});",
            "({constructor(){return super.x;},__proto__(){return super.x;}});",
            "class C extends Object {constructor(){super();({[super.key](){return super.x;}});super();}}",
            "({m(){super.x=1;super[x]++;for(super.x in obj){}for(super[x] of list){}new super.C;delete super.x;}});",
        ] {
            Parser::program_context(source, false, strict)
                .unwrap_or_else(|error| panic!("strict={strict}, {source}: {error:?}"));
        }
        for source in [
            "({m(){super();}});",
            "({m(x=super()){}});",
            "({m(){return ()=>super();}});",
            "({m(){return super;}});",
            "({m(){return {f:function(){return super.x;}};}});",
            "({m(x=function(){return super.x;}){}});",
            "({[super.key](){return 1;}});",
            "({m(){return 1;},[super.key](){return 1;}});",
            "({m(){return 1;},x:super.x});",
            "({m(){return super.x;}});super.x;",
            "({m(){return class C extends Object {[super()](){}};}});",
            "({m(x,x){return super.x;}});",
            "({get x(v){return super.x;}});",
        ] {
            let error = Parser::program_context(source, false, strict).unwrap_err();
            assert!(
                error.is_parse_error(),
                "strict={strict}, {source}: {error:?}"
            );
        }
    }
}

#[test]
fn object_super_parser_entry_context_restores_after_errors_and_work_refusal() {
    for (source, unsupported) in [
        ("({m(){return super[;}});", false),
        ("({m(x=super[){}});", false),
        ("({m(){'use strict';return ()=>super[;}});", false),
        ("({m(){return class C {field;};}});", true),
    ] {
        let mut budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        let mut parser = Parser::start(source, false, false, &mut budget).unwrap();
        parser.allow_in = false;
        let error = parser.parse_body().unwrap_err();
        if unsupported {
            assert!(error.is_unsupported(), "{source}: {error:?}");
        } else {
            assert!(error.is_parse_error(), "{source}: {error:?}");
        }
        assert!(!parser.allow_in && !parser.strict && !parser.in_class);
        assert!(!parser.super_call_allowed);
        assert!(!parser.super_property_allowed && !parser.super_property_deferred);
        assert!(parser.compile_budget.steps < budget.steps);
    }
    let source = format!("({{m(){{{}return super[key];}}}});", ";".repeat(200));
    let mut budget = regexp::Budget {
        steps: MAX_STEPS,
        allocated: 0,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    };
    let mut parser = Parser::start(&source, false, false, &mut budget).unwrap();
    parser.allow_in = false;
    parser.compile_budget.steps = 64;
    let error = parser.parse_body().unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(parser.compile_budget.steps, 0);
    assert!(!parser.allow_in && !parser.strict && !parser.in_class);
    assert!(!parser.super_call_allowed);
    assert!(!parser.super_property_allowed && !parser.super_property_deferred);
}

#[test]
fn optional_chain_parser_refuses_valid_postfix_boundaries_without_implementing_them() {
    for strict in [false, true] {
        for source in [
            "value?.property;",
            "value?.[key];",
            "value?.(argument);",
            "value.method()?.property;",
            "value /* before */ ?. /* after */ property;",
            "value\n?.property;",
            "new Constructor()?.property;",
            "(new Constructor)?.property;",
            "new (value?.Constructor)();",
            "({m(){return super.x?.y;}});",
            "({m(x=super.x?.y){}});",
            "({m(){return ()=>super[key]?.();}});",
            "class C extends Object {constructor(){super()?.x;}}",
        ] {
            let error = Parser::program_context(source, false, strict).unwrap_err();
            assert!(
                error.is_unsupported(),
                "strict={strict}, {source}: {error:?}"
            );
            assert_eq!(error.message, "optional chaining is not implemented");
        }
    }
}

#[test]
fn optional_chain_parser_preserves_separated_tokens_super_context_and_new_restrictions() {
    for strict in [false, true] {
        for source in [
            "value ? .property : other;",
            "value?/* gap */.property:other;",
            "value?\n.property:other;",
            "value? .[key]:other;",
            "({m(){return super?.property;}});",
            "class C extends Object {constructor(){super?.property;}}",
            "({m(){return super?.[key];}});",
            "({m(){return super?.();}});",
            "super.x?.y;",
            "function f(){return super.x?.y;}",
            "new Constructor?.property;",
            "new Constructor?.(argument);",
            "({m(){return new super.x?.y;}});",
            "new new Constructor()?.property;",
            "({m(){return new super?.C;}});",
        ] {
            let error = Parser::program_context(source, false, strict).unwrap_err();
            assert!(
                error.is_parse_error(),
                "strict={strict}, {source}: {error:?}"
            );
        }
    }
}

#[test]
fn optional_chain_parser_keeps_decimal_question_dot_as_conditional_ast() {
    for strict in [false, true] {
        for source in [
            "flag?.3:0;",
            "flag ? .3 : 0;",
            "flag?/* gap */.3:0;",
            "new Constructor?.3:0;",
        ] {
            let unit = Parser::program_context(source, false, strict).unwrap();
            let Expr::Conditional(_, yes, no) = unit.expr(statement_expr(&unit, unit.body[0]))
            else {
                panic!("strict={strict}, {source}: conditional expression");
            };
            assert!(
                matches!(unit.expr(*yes), Expr::Literal(Value::Number(value)) if *value == 0.3)
            );
            assert!(matches!(unit.expr(*no), Expr::Literal(Value::Number(value)) if *value == 0.0));
        }
        let unit = Parser::program_context("class C {m(){super.x?.3:0;}}", false, strict).unwrap();
        let Expr::Conditional(condition, yes, no) =
            unit.expr(statement_expr(&unit, method_body(&unit)[0]))
        else {
            panic!("super property followed by a decimal conditional");
        };
        property_key(&unit, *condition);
        assert!(matches!(unit.expr(*yes), Expr::Literal(Value::Number(value)) if *value == 0.3));
        assert!(matches!(unit.expr(*no), Expr::Literal(Value::Number(value)) if *value == 0.0));
    }
}
