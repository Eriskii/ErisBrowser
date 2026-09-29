//! Grammar continuations. Pending records own only flat IDs and bounded lists.
use super::*;

mod expressions;
mod functions;
mod statements;

enum Output {
    Expression(ExprId),
    Statement(StmtId),
    Body(Body),
    Function(FunctionCode),
    Parameters(functions::State),
    Key(PropertyName),
    Declaration(Stmt),
}
struct Body {
    statements: Vec<StmtId>,
    own_strict: bool,
    has_lexical: bool,
}
enum Kind {
    Expression(expressions::Frame),
    Statement(statements::Frame),
    Function(functions::Frame),
    Body(statements::BodyFrame),
}
enum Transition {
    Done(Output),
    Again(Kind),
    Child(Kind, Kind),
}
impl From<expressions::Frame> for Kind {
    fn from(frame: expressions::Frame) -> Self {
        Self::Expression(frame)
    }
}
impl From<statements::Frame> for Kind {
    fn from(frame: statements::Frame) -> Self {
        Self::Statement(frame)
    }
}
impl From<functions::Frame> for Kind {
    fn from(frame: functions::Frame) -> Self {
        Self::Function(frame)
    }
}
impl From<statements::BodyFrame> for Kind {
    fn from(frame: statements::BodyFrame) -> Self {
        Self::Body(frame)
    }
}
fn child(parent: impl Into<Kind>, child: impl Into<Kind>) -> Result<Transition> {
    Ok(Transition::Child(parent.into(), child.into()))
}
fn expression(output: Option<Output>) -> ExprId {
    let Some(Output::Expression(id)) = output else {
        unreachable!("expression grammar output")
    };
    id
}
fn statement(output: Option<Output>) -> StmtId {
    let Some(Output::Statement(id)) = output else {
        unreachable!("statement grammar output")
    };
    id
}
fn body(output: Option<Output>) -> Body {
    let Some(Output::Body(body)) = output else {
        unreachable!("body grammar output")
    };
    body
}
fn function(output: Option<Output>) -> FunctionCode {
    let Some(Output::Function(code)) = output else {
        unreachable!("function grammar output")
    };
    code
}
fn key(output: Option<Output>) -> PropertyName {
    let Some(Output::Key(key)) = output else {
        unreachable!("property grammar output")
    };
    key
}
fn declaration(output: Option<Output>) -> Stmt {
    let Some(Output::Declaration(stmt)) = output else {
        unreachable!("declaration grammar output")
    };
    stmt
}
fn done_expr(parser: &mut Parser<'_>, record: Expr) -> Result<Transition> {
    Ok(Transition::Done(Output::Expression(
        parser.emit_expr(record)?,
    )))
}
fn done_stmt(parser: &mut Parser<'_>, record: Stmt) -> Result<Transition> {
    Ok(Transition::Done(Output::Statement(
        parser.emit_stmt(record)?,
    )))
}

fn reserve(frames: &mut Vec<Kind>, budget: &mut regexp::Budget) -> Result<()> {
    let limit = MAX_HEAP / std::mem::size_of::<Kind>();
    if frames.len() >= limit {
        return Err(ScriptError::resource("parser continuation limit exceeded"));
    }
    if frames.len() == frames.capacity() {
        let capacity = (frames.len() + 1)
            .max(frames.capacity().saturating_mul(2))
            .clamp(4, limit);
        budget.work(frames.len() + 1).map_err(regexp_error)?;
        compile_allocate(budget, capacity * std::mem::size_of::<Kind>() + 32)?;
        frames
            .try_reserve_exact(capacity - frames.len())
            .map_err(|_| ScriptError::resource("parser continuation allocation failed"))?;
    }
    Ok(())
}
fn enter(parser: &mut Parser<'_>, frames: &mut Vec<Kind>, kind: Kind) -> Result<()> {
    reserve(frames, &mut parser.compile_budget)?;
    frames.push(kind);
    Ok(())
}
pub(super) fn parse(parser: &mut Parser<'_>) -> Result<()> {
    let body = body(Some(run(
        parser,
        statements::BodyFrame::start(false, true).into(),
    )?));
    if body.has_lexical {
        check_scope(
            &parser.unit,
            &mut parser.compile_budget,
            body.statements.iter(),
            false,
        )?;
    }
    parser.unit.body = body.statements;
    parser.unit.strict = parser.strict;
    Ok(())
}

pub(super) fn dynamic_function<'s>(
    parser: &mut Parser<'s>,
    source: &'s str,
) -> Result<FunctionCode> {
    let Output::Parameters(state) = run(parser, functions::Frame::Dynamic.into())? else {
        unreachable!("dynamic parameters output")
    };
    // Separate token streams prevent comments or delimiters in one fragment
    // from changing the grammar or lexical goal of the other fragment.
    parser.tokens = lex(source, &mut parser.compile_budget)?;
    parser.source = source;
    parser.pos = 0;
    parser.lex_work = source.len();
    let body = body(Some(run(
        parser,
        statements::BodyFrame::start(false, true).into(),
    )?));
    let Transition::Done(Output::Function(function)) = functions::finish(parser, state, body)?
    else {
        unreachable!("dynamic function output")
    };
    Ok(function)
}

fn run(parser: &mut Parser<'_>, root: Kind) -> Result<Output> {
    let mut frames = Vec::new();
    enter(parser, &mut frames, root)?;
    let mut output = None;
    while let Some(frame) = frames.pop() {
        parser.compile_budget.work(1).map_err(regexp_error)?;
        let transition = match frame {
            Kind::Expression(state) => expressions::step(parser, state, output.take()),
            Kind::Statement(state) => statements::step(parser, state, output.take()),
            Kind::Function(state) => functions::step(parser, state, output.take()),
            Kind::Body(state) => statements::body_step(parser, state, output.take()),
        }?;
        match transition {
            Transition::Done(value) => {
                output = Some(value);
            }
            Transition::Again(kind) => frames.push(kind),
            Transition::Child(parent, next) => {
                frames.push(parent);
                enter(parser, &mut frames, next)?;
            }
        }
    }
    Ok(output.expect("root grammar output"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminated_literal_dispatch_preserves_grammar_and_wide_upstream_parsing() {
        for source in [
            "1",
            "'x'",
            "1;",
            "1,2",
            "[1,2]",
            "({x:1,y:'v'})",
            "f(1, 'v')",
            "true?1:2",
            "1+2",
            "1?2:3",
            "1..x",
            "1['x']",
            "1()",
            "'x'.length",
            "'x'\n+1",
            "1=2",
            "1=>2",
            "1++",
            "++1",
            "1 x",
            "1:2",
            "[1 2]",
            "({x:1 2})",
            "f(1 2)",
            "010;",
            "'\\1';",
            "0x10;",
            "'use strict';010;",
        ] {
            for strict in [false, true] {
                for function in [false, true] {
                    let old = parser_legacy::Parser::program_context(source, function, strict)
                        .and_then(code::compile)
                        .map(|unit| code::canonical(&unit))
                        .map_err(|error| format!("{error:?}"));
                    let new = Parser::program_context(source, function, strict)
                        .map(|unit| code::canonical(&unit))
                        .map_err(|error| format!("{error:?}"));
                    assert_eq!(new, old, "{source:?}/{strict}/{function}");
                }
            }
        }
        let source = include_str!(
            "../../../tests/upstream/test262-array-sort/test/built-ins/Array/prototype/sort/stability-2048-elements.js"
        );
        for strict in [false, true] {
            Parser::program_context(source, false, strict).unwrap();
        }
    }

    #[test]
    fn grammar_driver_uses_a_small_native_stack_across_supported_nesting_shapes() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                for source in [
                    format!("{};{}", "(function(){".repeat(256), "})()".repeat(256)),
                    format!("{};{}", "{".repeat(512), "}".repeat(512)),
                    format!("{}0{}", "[".repeat(128), "]".repeat(128)),
                    format!("function f(a={}){{}}", "()=>".repeat(128) + "1"),
                    format!("{};{}", "function f(){".repeat(512), "}".repeat(512)),
                ] {
                    for strict in [false, true] {
                        let unit = Parser::program_context(&source, false, strict).unwrap();
                        drop(unit);
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
    #[test]
    fn original_upstream_32_iife_source_parses_executes_and_releases_on_a_small_stack() {
        let source = include_str!(
            "../../../tests/upstream/test262-functions/test/language/statements/function/S13.2.1_A1_T1.js"
        );
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(move || {
                for strict in [false, true] {
                    let unit = Parser::program_context(source, false, strict).unwrap();
                    let weak = Rc::downgrade(&unit);
                    drop(unit);
                    assert!(weak.upgrade().is_none());
                    let mut runtime = Runtime::new();
                    let mut document = Document::parse("");
                    let value = if strict {
                        runtime.execute_strict(source, &mut document)
                    } else {
                        runtime.execute(source, &mut document)
                    }
                    .unwrap();
                    assert_eq!(value, Value::Undefined);
                    assert_eq!(
                        (
                            runtime.frames.len(),
                            runtime.calls,
                            runtime.stack_units,
                            runtime.eval_depth
                        ),
                        (0, 0, 0, 0)
                    );
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn member_call_and_constructor_chains_continue_beyond_the_former_chain_guard() {
        for strict in [false, true] {
            let source = format!(
                "var x={{}};x.x=x;function f(){{return f;}}function C(){{return C;}} (x{})===x && (f{})===f && ({}C)===C;",
                ".x".repeat(128),
                "()".repeat(128),
                "new ".repeat(128)
            );
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let value = if strict {
                runtime.execute_strict(&source, &mut document)
            } else {
                runtime.execute(&source, &mut document)
            }
            .unwrap();
            assert_eq!(value, Value::Bool(true));
            assert_eq!(
                (
                    runtime.frames.len(),
                    runtime.calls,
                    runtime.stack_units,
                    runtime.eval_depth
                ),
                (0, 0, 0, 0)
            );
        }
    }

    #[test]
    fn deep_syntax_errors_drop_pending_contexts_without_publishing_a_body() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                for source in [
                    "(function(){".repeat(256) + "var x=/unfinished/;",
                    "[".repeat(256) + "/unfinished/",
                    "({[".repeat(128) + "0",
                    "function f(a=".repeat(128) + "1",
                    "`${".repeat(128) + "1",
                ] {
                    for strict in [false, true] {
                        let mut parser = Parser::start(
                            &source,
                            false,
                            strict,
                            &mut regexp::Budget {
                                steps: MAX_STEPS,
                                allocated: 0,
                                heap_limit: MAX_HEAP,
                                stack_limit: 16,
                            },
                        )
                        .unwrap();
                        let pattern = Rc::new(
                            RegExp::compile(
                                "retained".into(),
                                &"".into(),
                                &mut parser.compile_budget,
                            )
                            .unwrap(),
                        );
                        let weak = Rc::downgrade(&pattern);
                        parser.emit_expr(Expr::RegExp(pattern)).unwrap();
                        let error = parser.parse_body().unwrap_err();
                        assert!(error.is_parse_error(), "{error}");
                        assert!(parser.unit.body.is_empty());
                        assert_eq!(weak.strong_count(), 1);
                        drop(parser);
                        assert!(weak.upgrade().is_none());
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn continuation_growth_refuses_before_retaining_a_child_and_keeps_prior_charges() {
        let mut parser = Parser::start(
            "",
            false,
            false,
            &mut regexp::Budget {
                steps: MAX_STEPS,
                allocated: 0,
                heap_limit: MAX_HEAP,
                stack_limit: 16,
            },
        )
        .unwrap();
        let name = parser.copy_identifier("retained").unwrap();
        let mut frames = Vec::new();
        enter(
            &mut parser,
            &mut frames,
            expressions::Frame::NamedFunction(Some(name)).into(),
        )
        .unwrap();
        let mut accepted = parser.compile_budget.allocated;
        loop {
            let length = frames.len();
            let capacity = frames.capacity();
            match enter(
                &mut parser,
                &mut frames,
                expressions::Frame::NamedFunction(None).into(),
            ) {
                Ok(()) => accepted = parser.compile_budget.allocated,
                Err(error) => {
                    assert!(error.is_resource_limit());
                    assert_eq!(frames.len(), length);
                    assert_eq!(frames.capacity(), capacity);
                    assert!(accepted <= MAX_HEAP);
                    assert!(parser.compile_budget.allocated > MAX_HEAP);
                    assert!(capacity > 96);
                    assert!(capacity < MAX_HEAP / std::mem::size_of::<Kind>());
                    break;
                }
            }
        }
        assert!(
            matches!(&frames[0],Kind::Expression(expressions::Frame::NamedFunction(Some(name))) if name=="retained")
        );
        drop(frames);
        let mut frames = Vec::new();
        parser.compile_budget.steps = 0;
        let allocated = parser.compile_budget.allocated;
        assert!(
            enter(&mut parser, &mut frames, expressions::Frame::Primary.into())
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(frames.capacity(), 0);
        assert!(frames.is_empty());
        assert_eq!(parser.compile_budget.allocated, allocated);
    }
}
