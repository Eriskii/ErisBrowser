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
fn super_property_parser_object_methods_and_deferred_class_syntax_stay_unsupported() {
    for source in [
        "({m(){return super.x;}})",
        "({m(x=super.x){}})",
        "({get x(){return super.x;}})",
        "({set x(v){super.x=v;}})",
        "({m(){return ()=>super.x;}})",
        "class C {m(){return {m(){return super.x;}};}}",
        "class C {field;}",
        "class C {#field;}",
        "class C {static {}}",
        "class C {*m(){return super.x;}}",
        "class C {async m(){return super.x;}}",
    ] {
        let error = Parser::program(source).unwrap_err();
        assert!(error.is_unsupported(), "{source}: {error:?}");
    }
    // A nested ordinary function does not inherit even the deferred method permission.
    let error = Parser::program("({m(){return function(){return super.x;};}})").unwrap_err();
    assert!(error.is_parse_error(), "{error:?}");
}

#[test]
fn super_property_parser_context_restores_on_syntax_unsupported_and_work_refusal() {
    for source in [
        "class C {m(){return super[;}}",
        "class C {m(){return {m(){return super.x;}};}}",
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
