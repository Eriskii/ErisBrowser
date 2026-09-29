use super::*;
use expressions::Frame as Expression;

pub(super) enum Frame {
    Start(bool),
    Dynamic,
    Default(State, String),
    Body(State),
    Arrow(Vec<Parameter>),
    ArrowExpression(State),
}
pub(super) struct State {
    params: Vec<Parameter>,
    saved: Context,
    unique: bool,
    arrow: bool,
    dynamic: bool,
}
struct Context {
    loop_depth: usize,
    switch_depth: usize,
    allow_in: bool,
    strict: bool,
    labels: Vec<ActiveLabel>,
    new_target_allowed: bool,
}
impl Context {
    fn enter(p: &mut Parser<'_>, arrow: bool) -> Self {
        let saved = Self {
            new_target_allowed: p.new_target_allowed,
            loop_depth: p.loop_depth,
            switch_depth: p.switch_depth,
            allow_in: p.allow_in,
            strict: p.strict,
            labels: std::mem::take(&mut p.labels),
        };
        p.loop_depth = 0;
        p.switch_depth = 0;
        p.allow_in = true;
        p.function_depth += 1;
        p.new_target_allowed |= !arrow;
        saved
    }
    fn restore(self, p: &mut Parser<'_>) {
        p.loop_depth = self.loop_depth;
        p.switch_depth = self.switch_depth;
        p.allow_in = self.allow_in;
        p.strict = self.strict;
        p.labels = self.labels;
        p.new_target_allowed = self.new_target_allowed;
    }
}
pub(super) fn step(p: &mut Parser<'_>, frame: Frame, output: Option<Output>) -> Result<Transition> {
    match frame {
        Frame::Start(unique) => {
            p.expect("(")?;
            let saved = Context::enter(p, false);
            let state = State {
                params: Vec::new(),
                saved,
                unique,
                arrow: false,
                dynamic: false,
            };
            if p.eat(")") {
                function_body(p, state)
            } else {
                parameters(p, state)
            }
        }
        Frame::Dynamic => {
            let saved = Context::enter(p, false);
            let state = State {
                params: Vec::new(),
                saved,
                unique: false,
                arrow: false,
                dynamic: true,
            };
            if p.done() {
                function_body(p, state)
            } else {
                parameters(p, state)
            }
        }
        Frame::Default(mut state, name) => {
            let parameter = p.parameter(name, Some(expression(output)))?;
            push(&mut state.params, parameter, &mut p.compile_budget)?;
            if parameter_end(p, state.dynamic)? {
                function_body(p, state)
            } else {
                parameters(p, state)
            }
        }
        Frame::Body(state) => finish(p, state, body(output)),
        Frame::Arrow(params) => {
            let saved = Context::enter(p, true);
            let state = State {
                params,
                saved,
                unique: true,
                arrow: true,
                dynamic: false,
            };
            if p.eat("{") {
                child(Frame::Body(state), statements::BodyFrame::start(true, true))
            } else {
                child(Frame::ArrowExpression(state), Expression::Assignment)
            }
        }
        Frame::ArrowExpression(state) => {
            let id = p.emit_stmt(Stmt::Return(Some(expression(output))))?;
            let mut statements = Vec::new();
            push(&mut statements, id, &mut p.compile_budget)?;
            finish(
                p,
                state,
                Body {
                    statements,
                    own_strict: false,
                    has_lexical: false,
                },
            )
        }
    }
}
fn parameter_end(p: &mut Parser<'_>, dynamic: bool) -> Result<bool> {
    if if dynamic { p.done() } else { p.eat(")") } {
        return Ok(true);
    }
    p.expect(",")?;
    Ok(if dynamic { p.done() } else { p.eat(")") })
}
fn parameters(p: &mut Parser<'_>, mut state: State) -> Result<Transition> {
    loop {
        if p.is(".") {
            let parameter = p.rest_parameter(state.dynamic)?;
            push(&mut state.params, parameter, &mut p.compile_budget)?;
            if state.dynamic {
                if !p.done() {
                    return Err(p.error("unexpected token after rest parameter"));
                }
            } else {
                p.expect(")")?;
            }
            return function_body(p, state);
        }
        let name = p.binding_identifier()?;
        if p.eat("=") {
            return child(Frame::Default(state, name), Expression::Assignment);
        }
        let parameter = p.parameter(name, None)?;
        push(&mut state.params, parameter, &mut p.compile_budget)?;
        if parameter_end(p, state.dynamic)? {
            return function_body(p, state);
        }
    }
}
fn function_body(p: &mut Parser<'_>, state: State) -> Result<Transition> {
    if state.dynamic {
        return Ok(Transition::Done(Output::Parameters(state)));
    }
    p.expect("{")?;
    child(Frame::Body(state), statements::BodyFrame::start(true, true))
}
pub(super) fn finish(p: &mut Parser<'_>, state: State, body: Body) -> Result<Transition> {
    let strict = p.strict;
    let non_simple = state.params.iter().any(|p| !p.is_simple());
    if state.arrow {
        if body.own_strict && non_simple {
            return Err(p.error("use strict directive with non-simple parameters"));
        }
        if body.has_lexical {
            check_scope(
                &p.unit,
                &mut p.compile_budget,
                body.statements.iter(),
                false,
            )?;
        }
        check_parameter_lexicals(
            &p.unit,
            &mut p.compile_budget,
            &p.tokens[p.pos],
            state.params.iter().map(|p| p.name.as_str()),
            &body.statements,
            true,
        )?;
        for param in &state.params {
            p.validate_identifier(&param.name, true)?;
        }
        p.function_depth -= 1;
    } else {
        if body.has_lexical {
            check_scope(
                &p.unit,
                &mut p.compile_budget,
                body.statements.iter(),
                false,
            )?;
        }
        p.function_depth -= 1;
        if body.own_strict && non_simple {
            return Err(p.error("use strict directive with non-simple parameters"));
        }
        for param in &state.params {
            p.validate_identifier(&param.name, true)?;
        }
        check_parameter_lexicals(
            &p.unit,
            &mut p.compile_budget,
            &p.tokens[p.pos],
            state.params.iter().map(|p| p.name.as_str()),
            &body.statements,
            state.unique || strict || non_simple,
        )?;
    }
    state.saved.restore(p);
    let code = FunctionCode {
        params: state.params,
        body: body.statements,
        name: None,
        arrow: state.arrow,
        self_name: false,
        constructable: !state.arrow,
        strict,
    };
    if state.arrow {
        let id = p.emit_function(code)?;
        done_expr(p, Expr::Function(id))
    } else {
        Ok(Transition::Done(Output::Function(code)))
    }
}
