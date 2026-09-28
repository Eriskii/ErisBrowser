use super::*;

pub(super) enum Frame {
    Assignment,
    AssignmentHead,
    ConditionalYes(ExprId),
    ConditionalNo(ExprId, ExprId),
    AssignmentRight(String, ExprId),
    Sequence,
    SequenceHead,
    SequenceNext(Vec<ExprId>),
    Binary(u8),
    BinaryHead(u8),
    BinaryRight(Binary, &'static str),
    Unary,
    Prefix(&'static str),
    PrefixUpdate(f64),
    Base(bool),
    PostfixResume(Postfix),
    Computed(Postfix),
    Arguments(Postfix, Vec<ExprId>),
    New,
    NewHead,
    NewComputed(NewState),
    NewArguments(ExprId, Vec<ExprId>),
    Primary,
    CoverItem(Cover, bool),
    ArrayItem(Array),
    ObjectKey(Object, bool, bool),
    AccessorKey(Object, bool),
    ObjectFunction(Object, PropertyName, Option<bool>),
    ObjectValue(Object, PropertyName, bool),
    NamedFunction(Option<String>),
    Key,
    ComputedKey(bool),
    TemplatePart(JsString, Vec<(ExprId, JsString)>, bool),
    Pass,
}
pub(super) struct Binary {
    left: ExprId,
    operations: Vec<(String, ExprId)>,
    min: u8,
    chain: usize,
}
pub(super) struct Postfix {
    value: ExprId,
    chain: usize,
    async_call: bool,
}
pub(super) struct NewState {
    value: ExprId,
}
pub(super) struct Cover {
    items: Vec<(ExprId, bool)>,
    saved_in: bool,
}
pub(super) struct Array {
    items: Vec<Option<ExprId>>,
    saved_in: bool,
}
pub(super) struct Object {
    items: Vec<(code::PropertyName, ObjectEntry)>,
    saved_in: bool,
    prototype: bool,
}

pub(super) fn step(p: &mut Parser<'_>, frame: Frame, output: Option<Output>) -> Result<Transition> {
    match frame {
        Frame::Pass => Ok(Transition::Done(output.expect("grammar identity output"))),
        Frame::Assignment => {
            // A terminated literal has no postfix, binary, conditional or
            // assignment tail. Complete it in this charged dispatch instead of
            // suspending each of those empty layers. This bounded lookahead
            // does not scan or consume tokens; primary retains literal checks.
            let literal = matches!(
                p.tokens[p.pos].kind,
                TokenKind::Number(_) | TokenKind::String(_)
            );
            let terminated = [",", ";", ")", "]", "}", ":"]
                .iter()
                .any(|end| p.token_is(p.pos + 1, end))
                || matches!(
                    p.tokens.get(p.pos + 1).map(|t| &t.kind),
                    Some(TokenKind::End)
                );
            if literal && terminated {
                primary(p)
            } else {
                child(Frame::AssignmentHead, Frame::Binary(1))
            }
        }
        Frame::AssignmentHead => {
            let left = expression(output);
            if p.eat("?") {
                child(Frame::ConditionalYes(left), Frame::Assignment)
            } else {
                assignment_tail(p, left)
            }
        }
        Frame::ConditionalYes(left) => {
            let yes = expression(output);
            p.expect(":")?;
            child(Frame::ConditionalNo(left, yes), Frame::Assignment)
        }
        Frame::ConditionalNo(left, yes) => {
            let no = expression(output);
            let id = p.emit_expr(Expr::Conditional(left, yes, no))?;
            assignment_tail(p, id)
        }
        Frame::AssignmentRight(op, left) => {
            done_expr(p, Expr::Assign(op, left, expression(output)))
        }
        Frame::Sequence => child(Frame::SequenceHead, Frame::Assignment),
        Frame::SequenceHead => {
            let first = expression(output);
            if !p.eat(",") {
                return Ok(Transition::Done(Output::Expression(first)));
            }
            let mut items = Vec::new();
            push(&mut items, first, &mut p.compile_budget)?;
            child(Frame::SequenceNext(items), Frame::Assignment)
        }
        Frame::SequenceNext(mut items) => {
            push(&mut items, expression(output), &mut p.compile_budget)?;
            if p.eat(",") {
                child(Frame::SequenceNext(items), Frame::Assignment)
            } else {
                done_expr(p, Expr::Sequence(items))
            }
        }
        Frame::Binary(min) => child(Frame::BinaryHead(min), Frame::Unary),
        Frame::BinaryHead(min) => binary_next(
            p,
            Binary {
                left: expression(output),
                operations: Vec::new(),
                min,
                chain: 0,
            },
        ),
        Frame::BinaryRight(mut state, op) => {
            let right = expression(output);
            let op = p.copy_identifier(op)?;
            push(&mut state.operations, (op, right), &mut p.compile_budget)?;
            binary_next(p, state)
        }
        Frame::Unary => {
            for op in ["!", "-", "+", "~", "typeof", "void", "delete"] {
                if p.eat(op) {
                    return child(Frame::Prefix(op), Frame::Unary);
                }
            }
            if p.is("++") || p.is("--") {
                let delta = if p.eat("++") {
                    1.0
                } else {
                    p.pos += 1;
                    -1.0
                };
                return child(Frame::PrefixUpdate(delta), Frame::Unary);
            }
            let async_call = p.is("async")
                && p.token_is(p.pos + 1, "(")
                && !p.tokens[p.pos + 1].line_break_before;
            child(Frame::Base(async_call), Frame::New)
        }
        Frame::Prefix(op) => {
            let value = expression(output);
            if p.strict
                && op == "delete"
                && matches!(p.unit.expr(value),Expr::Ident(name) if name!="this")
            {
                return Err(p.error("strict code cannot delete an identifier"));
            }
            let op = p.copy_identifier(op)?;
            done_expr(p, Expr::Unary(op, value))
        }
        Frame::PrefixUpdate(delta) => {
            let value = expression(output);
            p.assignment_target(value)?;
            done_expr(p, Expr::Update(value, delta, true))
        }
        Frame::Base(async_call) => postfix(
            p,
            Postfix {
                value: expression(output),
                chain: 0,
                async_call,
            },
        ),
        Frame::PostfixResume(state) => postfix(p, state),
        Frame::Computed(mut state) => {
            let property = expression(output);
            p.expect("]")?;
            state.value = p.emit_expr(Expr::Member(state.value, property))?;
            postfix(p, state)
        }
        Frame::Arguments(state, mut arguments) => {
            push(&mut arguments, expression(output), &mut p.compile_budget)?;
            if p.eat(")") {
                call_done(p, state, arguments)
            } else {
                p.expect(",")?;
                child(Frame::Arguments(state, arguments), Frame::Assignment)
            }
        }
        Frame::New => {
            if p.eat("new") {
                child(Frame::NewHead, Frame::New)
            } else {
                child(Frame::Pass, Frame::Primary)
            }
        }
        Frame::NewHead => new_members(
            p,
            NewState {
                value: expression(output),
            },
        ),
        Frame::NewComputed(mut state) => {
            let key = expression(output);
            p.expect("]")?;
            state.value = p.emit_expr(Expr::Member(state.value, key))?;
            new_members(p, state)
        }
        Frame::NewArguments(constructor, mut arguments) => {
            push(&mut arguments, expression(output), &mut p.compile_budget)?;
            if p.eat(")") {
                done_expr(p, Expr::New(constructor, arguments))
            } else {
                p.expect(",")?;
                child(
                    Frame::NewArguments(constructor, arguments),
                    Frame::Assignment,
                )
            }
        }
        Frame::Primary => primary(p),
        Frame::CoverItem(mut state, binding) => {
            push(
                &mut state.items,
                (expression(output), binding),
                &mut p.compile_budget,
            )?;
            if p.eat(")") {
                cover_done(p, state, None, false)
            } else {
                p.expect(",")?;
                if p.eat(")") {
                    cover_done(p, state, None, true)
                } else {
                    cover_next(p, state)
                }
            }
        }
        Frame::ArrayItem(mut state) => {
            push(
                &mut state.items,
                Some(expression(output)),
                &mut p.compile_budget,
            )?;
            if p.eat("]") {
                array_done(p, state)
            } else {
                p.expect(",")?;
                array_next(p, state)
            }
        }
        Frame::ObjectKey(state, accessor, async_keyword) => {
            let key = key(output);
            if async_keyword
                && !p.is(":")
                && !p.is("(")
                && !p.is(",")
                && !p.is("}")
                && !p.tokens[p.pos].line_break_before
            {
                return Err(ScriptError::unsupported(
                    "async methods are not implemented",
                ));
            }
            if accessor && !p.is(":") && !p.is("(") && !p.is(",") && !p.is("}") {
                let setter =
                    matches!(&key,PropertyName::Literal(name,_) if name==&JsString::from("set"));
                child(Frame::AccessorKey(state, setter), Frame::Key)
            } else if p.is("(") {
                child(
                    Frame::ObjectFunction(state, key, None),
                    functions::Frame::Start(true),
                )
            } else if p.eat(":") {
                let prototype = matches!(&key,PropertyName::Literal(name,_) if name==&JsString::from("__proto__"));
                if prototype && state.prototype {
                    return Err(p.error("duplicate __proto__ prototype setter"));
                }
                child(Frame::ObjectValue(state, key, prototype), Frame::Assignment)
            } else {
                let PropertyName::Literal(name, true) = &key else {
                    return Err(p.error("object shorthand requires an identifier"));
                };
                p.compile_budget
                    .work(1 + name.len() / 8)
                    .map_err(regexp_error)?;
                compile_allocate(
                    &mut p.compile_budget,
                    name.len().saturating_mul(3).saturating_add(32),
                )?;
                let name = name
                    .to_utf8()
                    .map_err(|_| p.error("object shorthand requires an identifier"))?;
                p.validate_identifier(&name, false)?;
                let id = p.emit_expr(Expr::Ident(name))?;
                object_entry(p, state, key, ObjectEntry::Data(id))
            }
        }
        Frame::AccessorKey(state, setter) => child(
            Frame::ObjectFunction(state, key(output), Some(setter)),
            functions::Frame::Start(true),
        ),
        Frame::ObjectFunction(state, key, setter) => {
            let mut code = function(output);
            if let Some(setter) = setter
                && (code.params.len() != usize::from(setter) || code.params.iter().any(|p| p.rest))
            {
                return Err(p.error("invalid accessor parameter count"));
            }
            code.constructable = false;
            let id = p.emit_function(code)?;
            let entry = setter.map_or(ObjectEntry::Method(id), |setter| {
                ObjectEntry::Accessor(id, setter)
            });
            object_entry(p, state, key, entry)
        }
        Frame::ObjectValue(mut state, key, prototype) => {
            let id = expression(output);
            state.prototype |= prototype;
            let value = if prototype {
                ObjectEntry::Prototype(id)
            } else {
                ObjectEntry::Data(id)
            };
            object_entry(p, state, key, value)
        }
        Frame::NamedFunction(name) => {
            let mut code = function(output);
            if let Some(name) = &name {
                let saved = p.strict;
                p.strict = code.strict;
                p.validate_identifier(name, true)?;
                p.strict = saved;
            }
            code.name = name;
            code.self_name = code.name.is_some();
            let id = p.emit_function(code)?;
            done_expr(p, Expr::Function(id))
        }
        Frame::Key => {
            if p.eat("[") {
                let saved = p.allow_in;
                p.allow_in = true;
                child(Frame::ComputedKey(saved), Frame::Assignment)
            } else {
                Ok(Transition::Done(Output::Key(p.object_key()?)))
            }
        }
        Frame::ComputedKey(saved) => {
            let id = expression(output);
            p.expect("]")?;
            p.allow_in = saved;
            Ok(Transition::Done(Output::Key(PropertyName::Computed(id))))
        }
        Frame::TemplatePart(head, mut tail, saved) => {
            let id = expression(output);
            p.allow_in = saved;
            let start = p.tokens[p.pos].offset;
            p.expect("}")?;
            let (text, end, _, next) =
                quoted_text(p.source, start, '`', Some(&mut p.compile_budget))?;
            push(&mut tail, (id, text), &mut p.compile_budget)?;
            p.rescan_suffix(end)?;
            template_next(p, head, tail, next)
        }
    }
}
fn assignment_tail(p: &mut Parser<'_>, left: ExprId) -> Result<Transition> {
    for operator in [
        "=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "&=", "^=", "|=", "&&=",
        "||=", "??=",
    ] {
        if p.eat(operator) {
            p.assignment_target(left)?;
            let op = p.copy_identifier(operator)?;
            return child(Frame::AssignmentRight(op, left), Frame::Assignment);
        }
    }
    Ok(Transition::Done(Output::Expression(left)))
}
fn operator(p: &Parser<'_>) -> Option<(&'static str, u8)> {
    let op = match &p.tokens[p.pos].kind {
        TokenKind::Symbol(op) => op.as_str(),
        TokenKind::Word(word) if !word.escaped => word.value.as_ref(),
        _ => return None,
    };
    Some(match op {
        "||" => ("||", 1),
        "??" => ("??", 1),
        "&&" => ("&&", 2),
        "|" => ("|", 3),
        "^" => ("^", 4),
        "&" => ("&", 5),
        "==" => ("==", 6),
        "!=" => ("!=", 6),
        "===" => ("===", 6),
        "!==" => ("!==", 6),
        "<" => ("<", 7),
        ">" => (">", 7),
        "<=" => ("<=", 7),
        ">=" => (">=", 7),
        "instanceof" => ("instanceof", 7),
        "in" if p.allow_in => ("in", 7),
        "<<" => ("<<", 8),
        ">>" => (">>", 8),
        ">>>" => (">>>", 8),
        "+" => ("+", 9),
        "-" => ("-", 9),
        "*" => ("*", 10),
        "/" => ("/", 10),
        "%" => ("%", 10),
        "**" => ("**", 11),
        _ => return None,
    })
}
fn binary_next(p: &mut Parser<'_>, mut state: Binary) -> Result<Transition> {
    if let Some((op, precedence)) = operator(p)
        && precedence >= state.min
    {
        state.chain += 1;
        if state.chain > 4096 {
            return Err(p.resource_error("expression chain limit exceeded"));
        }
        p.pos += 1;
        return child(
            Frame::BinaryRight(state, op),
            Frame::Binary(if op == "**" {
                precedence
            } else {
                precedence + 1
            }),
        );
    }
    if let Expr::Literal(Value::String(first)) = p.unit.expr(state.left)
        && !state.operations.is_empty()
        && state.operations.iter().all(|(op, id)| {
            op == "+" && matches!(p.unit.expr(*id), Expr::Literal(Value::String(_)))
        })
    {
        let length = state
            .operations
            .iter()
            .fold(first.len(), |length, (_, id)| {
                let Expr::Literal(Value::String(text)) = p.unit.expr(*id) else {
                    unreachable!()
                };
                length.saturating_add(text.len())
            });
        if length > MAX_STRING {
            return Err(p.resource_error("script string limit exceeded"));
        }
        p.compile_budget
            .work(length / 8 + 1)
            .map_err(regexp_error)?;
        compile_allocate(&mut p.compile_budget, length * 4 + 32)?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("folded string allocation failed"))?;
        units.extend_from_slice(first.units());
        for (_, id) in state.operations {
            let Expr::Literal(Value::String(text)) = p.unit.expr(id) else {
                unreachable!()
            };
            units.extend_from_slice(text.units());
        }
        return done_expr(p, Expr::Literal(Value::String(units.into())));
    }
    if state.operations.is_empty() {
        Ok(Transition::Done(Output::Expression(state.left)))
    } else {
        done_expr(p, Expr::BinaryChain(state.left, state.operations))
    }
}
fn postfix(p: &mut Parser<'_>, mut state: Postfix) -> Result<Transition> {
    loop {
        state.chain += 1;
        if p.eat(".") {
            let name = p.identifier()?;
            let key = p.identifier_key(&name)?;
            let key = p.emit_expr(Expr::Literal(Value::String(key)))?;
            state.value = p.emit_expr(Expr::Member(state.value, key))?;
        } else if p.eat("[") {
            return child(Frame::Computed(state), Frame::Assignment);
        } else if p.eat("(") {
            if p.eat(")") {
                return call_done(p, state, Vec::new());
            }
            return child(Frame::Arguments(state, Vec::new()), Frame::Assignment);
        } else if matches!(p.tokens[p.pos].kind, TokenKind::TemplateStart) {
            return Err(ScriptError::unsupported(
                "tagged template literals are not implemented",
            ));
        } else {
            break;
        }
    }
    if !p.tokens[p.pos].line_break_before && p.eat("++") {
        p.assignment_target(state.value)?;
        state.value = p.emit_expr(Expr::Update(state.value, 1.0, false))?;
    } else if !p.tokens[p.pos].line_break_before && p.eat("--") {
        p.assignment_target(state.value)?;
        state.value = p.emit_expr(Expr::Update(state.value, -1.0, false))?;
    }
    Ok(Transition::Done(Output::Expression(state.value)))
}
fn call_done(p: &mut Parser<'_>, mut state: Postfix, arguments: Vec<ExprId>) -> Result<Transition> {
    if state.async_call && state.chain == 1 && p.is("=>") {
        if p.tokens[p.pos].line_break_before {
            return Err(p.error("line terminator before arrow"));
        }
        return Err(ScriptError::unsupported(
            "async arrow functions are not implemented",
        ));
    }
    state.value = p.emit_expr(Expr::Call(state.value, arguments))?;
    // Resume through a frame even for empty argument lists: postfix/call_done
    // must not recursively retain one native frame per call in a long chain.
    Ok(Transition::Again(Frame::PostfixResume(state).into()))
}
fn new_members(p: &mut Parser<'_>, mut state: NewState) -> Result<Transition> {
    loop {
        if p.eat(".") {
            let name = p.identifier()?;
            let key = p.identifier_key(&name)?;
            let key = p.emit_expr(Expr::Literal(Value::String(key)))?;
            state.value = p.emit_expr(Expr::Member(state.value, key))?;
        } else if p.eat("[") {
            return child(Frame::NewComputed(state), Frame::Assignment);
        } else if matches!(p.tokens[p.pos].kind, TokenKind::TemplateStart) {
            return Err(ScriptError::unsupported(
                "tagged template literals are not implemented",
            ));
        } else {
            break;
        }
    }
    if p.eat("(") && !p.eat(")") {
        child(
            Frame::NewArguments(state.value, Vec::new()),
            Frame::Assignment,
        )
    } else {
        done_expr(p, Expr::New(state.value, Vec::new()))
    }
}
fn primary(p: &mut Parser<'_>) -> Result<Transition> {
    if p.unsupported_async_start() {
        return Err(ScriptError::unsupported(
            "async functions are not implemented",
        ));
    }
    if matches!(p.tokens[p.pos].kind, TokenKind::TemplateStart) {
        let start = p.tokens[p.pos].offset;
        let (head, end, _, interpolation) =
            quoted_text(p.source, start, '`', Some(&mut p.compile_budget))?;
        p.pos += 1;
        p.rescan_suffix(end)?;
        return template_next(p, head, Vec::new(), interpolation);
    }
    if p.is("/") || p.is("/=") {
        return Ok(Transition::Done(Output::Expression(p.regexp_literal()?)));
    }
    if p.eat("(") {
        let saved_in = p.allow_in;
        p.allow_in = true;
        let state = Cover {
            items: Vec::new(),
            saved_in,
        };
        return if p.eat(")") {
            cover_done(p, state, None, false)
        } else {
            cover_next(p, state)
        };
    }
    if p.eat("[") {
        let saved_in = p.allow_in;
        p.allow_in = true;
        return array_next(
            p,
            Array {
                items: Vec::new(),
                saved_in,
            },
        );
    }
    if p.eat("{") {
        let saved_in = p.allow_in;
        p.allow_in = true;
        return object_next(
            p,
            Object {
                items: Vec::new(),
                saved_in,
                prototype: false,
            },
        );
    }
    if p.eat("function") {
        if p.is("*") {
            return Err(ScriptError::unsupported(
                "generator functions are not implemented",
            ));
        }
        let name = if matches!(p.tokens[p.pos].kind, TokenKind::Word(_)) {
            Some(p.binding_identifier()?)
        } else {
            None
        };
        return child(Frame::NamedFunction(name), functions::Frame::Start(false));
    }
    let offset = p.tokens[p.pos].offset;
    let legacy = p.tokens[p.pos].legacy_literal;
    let kind = match &p.tokens[p.pos].kind {
        TokenKind::Symbol(_) | TokenKind::End => {
            return Err(ScriptError::at("expected expression", offset));
        }
        kind => kind.clone(),
    };
    if p.strict && legacy {
        return Err(ScriptError::at(
            "legacy literals are forbidden in strict code",
            offset,
        ));
    }
    if !p.done() {
        p.pos += 1;
    }
    match kind {
        TokenKind::RegExp(pattern) => done_expr(p, Expr::RegExp(pattern)),
        TokenKind::Invalid(error) => Err((*error).clone()),
        TokenKind::Number(n) => done_expr(p, Expr::Literal(Value::Number(n))),
        TokenKind::String(s) => done_expr(p, Expr::Literal(Value::String(s))),
        TokenKind::Word(s) if !s.escaped && matches!(&*s.value, "true" | "false") => {
            done_expr(p, Expr::Literal(Value::Bool(&*s.value == "true")))
        }
        TokenKind::Word(s) if !s.escaped && &*s.value == "null" => {
            done_expr(p, Expr::Literal(Value::Null))
        }
        TokenKind::Word(s) if !s.escaped && &*s.value == "super" => Err(ScriptError::unsupported(
            "super property and constructor references are not implemented",
        )),
        TokenKind::Word(word) => {
            let name = p.copy_identifier(&word.value)?;
            if p.is("=>") {
                if p.tokens[p.pos].line_break_before {
                    return Err(p.error("line terminator before arrow"));
                }
                p.pos += 1;
                let parameter = p.parameter(name, None)?;
                let mut params = Vec::new();
                push(&mut params, parameter, &mut p.compile_budget)?;
                child(Frame::Pass, functions::Frame::Arrow(params))
            } else {
                if word.escaped || name != "this" {
                    p.validate_identifier(&name, false)?;
                }
                done_expr(p, Expr::Ident(name))
            }
        }
        _ => Err(ScriptError::at("expected expression", offset)),
    }
}
fn cover_next(p: &mut Parser<'_>, state: Cover) -> Result<Transition> {
    if p.is(".") {
        let rest = p.rest_parameter()?;
        p.expect(")")?;
        return cover_done(p, state, Some(rest), false);
    }
    let binding = matches!(p.tokens[p.pos].kind, TokenKind::Word(_));
    child(Frame::CoverItem(state, binding), Frame::Assignment)
}
fn cover_done(
    p: &mut Parser<'_>,
    mut state: Cover,
    rest: Option<Parameter>,
    trailing: bool,
) -> Result<Transition> {
    p.allow_in = state.saved_in;
    if p.is("=>") {
        if p.tokens[p.pos].line_break_before {
            return Err(p.error("line terminator before arrow"));
        }
        p.pos += 1;
        let mut params = Vec::new();
        for (id, binding) in state.items {
            if matches!(p.unit.expr(id), Expr::Array(_) | Expr::Object(_)) {
                return Err(ScriptError::unsupported(
                    "destructuring parameters are not implemented",
                ));
            }
            if !binding {
                return Err(p.error("invalid arrow parameter"));
            }
            let (name, initializer) = match p.unit.take_expr(id) {
                Expr::Ident(name) => (name, None),
                Expr::Assign(operator, target, value) if operator == "=" => {
                    let Expr::Ident(name) = p.unit.take_expr(target) else {
                        return Err(p.error("invalid arrow parameter"));
                    };
                    (name, Some(value))
                }
                _ => return Err(p.error("invalid arrow parameter")),
            };
            let parameter = p.parameter(name, initializer)?;
            push(&mut params, parameter, &mut p.compile_budget)?;
        }
        if let Some(rest) = rest {
            push(&mut params, rest, &mut p.compile_budget)?;
        }
        return child(Frame::Pass, functions::Frame::Arrow(params));
    }
    if rest.is_some() {
        return Err(p.error("rest parameter requires an arrow function"));
    }
    if state.items.is_empty() || trailing {
        return Err(p.error("invalid parenthesized expression"));
    }
    if state.items.len() == 1 {
        return Ok(Transition::Done(Output::Expression(
            state.items.pop().unwrap().0,
        )));
    }
    let mut items = Vec::new();
    for (id, _) in state.items {
        push(&mut items, id, &mut p.compile_budget)?;
    }
    done_expr(p, Expr::Sequence(items))
}
fn array_next(p: &mut Parser<'_>, mut state: Array) -> Result<Transition> {
    while !p.eat("]") {
        if p.eat(",") {
            push(&mut state.items, None, &mut p.compile_budget)?;
            continue;
        }
        return child(Frame::ArrayItem(state), Frame::Assignment);
    }
    array_done(p, state)
}
fn array_done(p: &mut Parser<'_>, state: Array) -> Result<Transition> {
    p.allow_in = state.saved_in;
    done_expr(p, Expr::Array(state.items))
}
fn object_next(p: &mut Parser<'_>, state: Object) -> Result<Transition> {
    if p.eat("}") {
        p.allow_in = state.saved_in;
        return done_expr(p, Expr::Object(state.items));
    }
    if p.is("*") || p.is(".") {
        return Err(ScriptError::unsupported(
            "generator methods and object spread are not implemented",
        ));
    }
    let accessor = p.is("get") || p.is("set");
    let async_keyword = p.is("async");
    child(Frame::ObjectKey(state, accessor, async_keyword), Frame::Key)
}
fn object_entry(
    p: &mut Parser<'_>,
    mut state: Object,
    key: PropertyName,
    value: ObjectEntry,
) -> Result<Transition> {
    push(
        &mut state.items,
        (key.finish(), value),
        &mut p.compile_budget,
    )?;
    if p.eat("}") {
        p.allow_in = state.saved_in;
        done_expr(p, Expr::Object(state.items))
    } else {
        p.expect(",")?;
        object_next(p, state)
    }
}
fn template_next(
    p: &mut Parser<'_>,
    head: JsString,
    tail: Vec<(ExprId, JsString)>,
    interpolation: bool,
) -> Result<Transition> {
    if !interpolation {
        return done_expr(
            p,
            if tail.is_empty() {
                Expr::Literal(Value::String(head))
            } else {
                Expr::Template(head, tail)
            },
        );
    }
    if tail.len() >= 4096 {
        return Err(p.resource_error("template substitution limit exceeded"));
    }
    let saved = p.allow_in;
    p.allow_in = true;
    child(Frame::TemplatePart(head, tail, saved), Frame::Sequence)
}
