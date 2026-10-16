//! Flat class syntax; each element or semicolon advances a paid driver step.
use super::*;
use code::{Class, ClassElement, ClassElementKind, ClassKind};
use expressions::Frame as Expression;

pub(super) enum Frame {
    Start { declaration: bool },
    Heritage(State),
    Elements(State),
    Key(State, Header),
    Member(State, PropertyName, Header),
}

pub(super) struct State {
    name: Option<String>,
    heritage: Option<ExprId>,
    constructor: Option<FunctionId>,
    elements: Vec<code::ClassElement>,
    saved: Context,
}

pub(super) struct Header {
    kind: code::ClassElementKind,
    is_static: bool,
    is_constructor: bool,
}

struct Context {
    strict: bool,
    allow_in: bool,
    in_class: bool,
}

impl Context {
    fn enter(p: &mut Parser<'_>) -> Self {
        let saved = Self {
            strict: p.strict,
            allow_in: p.allow_in,
            in_class: p.in_class,
        };
        p.strict = true;
        p.in_class = true;
        saved
    }
    fn restore(self, p: &mut Parser<'_>) {
        p.strict = self.strict;
        p.allow_in = self.allow_in;
        p.in_class = self.in_class;
    }
}

// Fixed ASCII grammar words require neither a new JsString nor a UTF-8 copy.
fn literal_is(key: &PropertyName, text: &str) -> bool {
    matches!(key, PropertyName::Literal(value, _) if value.len() == text.len()
        && value.units().iter().copied().eq(text.bytes().map(u16::from)))
}

fn deferred() -> ScriptError {
    // Preserve the complete historical class-field differential diagnostic.
    ScriptError::unsupported("'class' is not supported")
}

pub(super) fn step(p: &mut Parser<'_>, frame: Frame, output: Option<Output>) -> Result<Transition> {
    match frame {
        Frame::Start { declaration } => {
            p.expect("class")?;
            let saved = Context::enter(p);
            let name = if !p.is("extends") && matches!(p.tokens[p.pos].kind, TokenKind::Word(_)) {
                Some(p.binding_identifier()?)
            } else if declaration {
                return Err(p.error("class declaration requires a name"));
            } else {
                None
            };
            let state = State {
                name,
                heritage: None,
                constructor: None,
                elements: Vec::new(),
                saved,
            };
            if p.eat("extends") {
                p.allow_in = true;
                child(Frame::Heritage(state), Expression::LeftHandSide)
            } else {
                p.expect("{")?;
                Ok(Transition::Again(Frame::Elements(state).into()))
            }
        }
        Frame::Heritage(mut state) => {
            state.heritage = Some(expression(output));
            p.allow_in = state.saved.allow_in;
            p.expect("{")?;
            Ok(Transition::Again(Frame::Elements(state).into()))
        }
        Frame::Elements(state) => elements(p, state),
        Frame::Key(state, mut header) => {
            let name = key(output);
            if !p.is("(") {
                return Err(deferred());
            }
            if header.is_static && literal_is(&name, "prototype") {
                return Err(p.error("static class element cannot be named prototype"));
            }
            header.is_constructor = !header.is_static && literal_is(&name, "constructor");
            if header.is_constructor {
                if header.kind != ClassElementKind::Method {
                    return Err(p.error("class constructor cannot be an accessor"));
                }
                if state.constructor.is_some() {
                    return Err(p.error("duplicate class constructor"));
                }
            }
            let derived_constructor = header.is_constructor && state.heritage.is_some();
            child(
                Frame::Member(state, name, header),
                functions::Frame::ClassMethod(derived_constructor),
            )
        }
        Frame::Member(mut state, name, header) => {
            let mut function = function(output);
            match header.kind {
                ClassElementKind::Getter if !function.params.is_empty() => {
                    return Err(p.error("invalid getter parameter count"));
                }
                ClassElementKind::Setter
                    if function.params.len() != 1 || function.params[0].rest =>
                {
                    return Err(p.error("invalid setter parameter count"));
                }
                _ => {}
            }
            function.constructable = header.is_constructor;
            if header.is_constructor {
                function.class_kind = Some(if state.heritage.is_some() {
                    ClassKind::ExplicitDerived
                } else {
                    ClassKind::ExplicitBase
                });
            }
            let id = p.emit_function(function)?;
            if header.is_constructor {
                state.constructor = Some(id);
            } else {
                push(
                    &mut state.elements,
                    ClassElement {
                        name: name.finish(),
                        function: id,
                        kind: header.kind,
                        is_static: header.is_static,
                    },
                    &mut p.compile_budget,
                )?;
            }
            Ok(Transition::Again(Frame::Elements(state).into()))
        }
    }
}

fn elements(p: &mut Parser<'_>, state: State) -> Result<Transition> {
    if p.eat("}") {
        let constructor = if let Some(id) = state.constructor {
            id
        } else {
            p.emit_function(FunctionCode {
                params: Vec::new(),
                body: Vec::new(),
                name: None,
                arrow: false,
                self_name: false,
                constructable: true,
                strict: true,
                class_kind: Some(if state.heritage.is_some() {
                    ClassKind::DefaultDerived
                } else {
                    ClassKind::DefaultBase
                }),
            })?
        };
        state.saved.restore(p);
        let id = p.emit_class(Class {
            name: state.name,
            heritage: state.heritage,
            constructor,
            elements: state.elements,
        })?;
        return Ok(Transition::Done(Output::Class(id)));
    }
    if p.done() {
        return Err(p.error("unterminated class body"));
    }
    if p.eat(";") {
        return Ok(Transition::Again(Frame::Elements(state).into()));
    }
    if matches!(p.tokens[p.pos].kind, TokenKind::Invalid(_)) {
        return Err(p.error("invalid class element"));
    }
    let is_static = p.is("static") && !p.token_is(p.pos + 1, "(");
    if is_static {
        p.pos += 1;
        if p.is("{") || p.is(";") || p.is("=") || p.is("}") {
            return Err(deferred());
        }
    }
    if p.is("*") || p.is("async") && !p.token_is(p.pos + 1, "(") {
        return Err(deferred());
    }
    let kind = if (p.is("get") || p.is("set"))
        && !["(", ";", "=", "}"]
            .iter()
            .any(|next| p.token_is(p.pos + 1, next))
    {
        let setter = p.eat("set");
        if !setter {
            p.expect("get")?;
        }
        if setter {
            ClassElementKind::Setter
        } else {
            ClassElementKind::Getter
        }
    } else {
        ClassElementKind::Method
    };
    child(
        Frame::Key(
            state,
            Header {
                kind,
                is_static,
                is_constructor: false,
            },
        ),
        Expression::Key,
    )
}
