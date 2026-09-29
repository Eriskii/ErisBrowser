use super::*;
use expressions::Frame as Expression;

pub(super) enum Frame {
    Start { declarations: bool },
    Controlled,
    ControlledDone,
    Expression,
    Declaration(bool),
    DeclarationDone,
    DeclarationDefault(Vec<(String, Option<ExprId>)>, DeclarationKind, String, bool),
    Function(String),
    Block,
    Label(usize, usize),
    SwitchExpression,
    SwitchCase(Switch),
    SwitchNext(Switch),
    SwitchStatement(Switch, Option<ExprId>, Vec<StmtId>),
    IfCondition,
    IfYes(ExprId),
    IfNo(ExprId, StmtId),
    WhileCondition,
    WhileBody(ExprId),
    DoBody,
    DoCondition(StmtId),
    ForDeclaration(bool),
    ForExpression(bool, bool),
    ForTest(Option<Stmt>),
    ForUpdate(Option<Stmt>, Option<ExprId>),
    ForBody(Option<Stmt>, Option<ExprId>, Option<ExprId>),
    ForInObject(ForBinding),
    ForInBody(ForBinding, ExprId),
    ForOfObject(ForBinding),
    ForOfBody(ForBinding, ExprId),
    Return,
    Throw,
    TryBody,
    CatchBody(StmtId, Option<String>),
    FinallyBody(StmtId, Option<CatchClause>),
}
pub(super) struct Switch {
    value: ExprId,
    cases: Vec<(Option<ExprId>, Vec<StmtId>)>,
    default: bool,
    lexical: bool,
}
pub(super) enum BodyFrame {
    Start { block: bool, directive: bool },
    Waiting(BodyState, Before),
}
pub(super) struct BodyState {
    block: bool,
    directive: bool,
    start: usize,
    prologue: bool,
    body: Body,
}
pub(super) struct Before {
    pos: usize,
    string: bool,
    strict: bool,
}
impl BodyFrame {
    pub fn start(block: bool, directive: bool) -> Self {
        Self::Start { block, directive }
    }
}
impl Before {
    fn new(p: &Parser<'_>) -> Self {
        Self {
            pos: p.pos,
            string: p.tokens[p.pos].string_literal,
            strict: p.tokens[p.pos].use_strict,
        }
    }
}
pub(super) fn body_step(
    p: &mut Parser<'_>,
    frame: BodyFrame,
    output: Option<Output>,
) -> Result<Transition> {
    let state = match frame {
        BodyFrame::Start { block, directive } => {
            let mut statements = Vec::new();
            if !block {
                let capacity = p.tokens.len().saturating_sub(1).min(MAX_TOKENS);
                if capacity != 0 {
                    p.compile_budget.work(1).map_err(regexp_error)?;
                    compile_allocate(
                        &mut p.compile_budget,
                        capacity * std::mem::size_of::<StmtId>() + 32,
                    )?;
                    statements
                        .try_reserve_exact(capacity)
                        .map_err(|_| ScriptError::resource("parser body allocation failed"))?;
                }
            }
            BodyState {
                block,
                directive,
                start: p.pos,
                prologue: true,
                body: Body {
                    statements,
                    own_strict: false,
                    has_lexical: false,
                },
            }
        }
        BodyFrame::Waiting(mut state, before) => {
            append_body(p, &mut state, before, statement(output))?;
            state
        }
    };
    body_next(p, state)
}
fn append_body(
    p: &mut Parser<'_>,
    state: &mut BodyState,
    before: Before,
    id: StmtId,
) -> Result<()> {
    if state.directive {
        let bare = before.string
            && matches!(p.unit.stmt(id),Stmt::Expr(id) if matches!(p.unit.expr(*id),Expr::Literal(Value::String(_))))
            && (p.pos == before.pos + 1
                || p.pos == before.pos + 2
                    && matches!(&p.tokens[before.pos+1].kind,TokenKind::Symbol(s) if s==";"));
        if state.prologue && bare {
            if before.strict {
                state.body.own_strict = true;
                p.strict = true;
                if p.tokens.range(state.start..p.pos).any(|t| t.legacy_literal) {
                    return Err(p.error("legacy escapes are forbidden in strict directives"));
                }
            }
        } else {
            state.prologue = false;
        }
    }
    state.body.has_lexical |= p.unit.stmt(id).is_lexical_declaration(!state.directive);
    push(&mut state.body.statements, id, &mut p.compile_budget)
}
fn body_next(p: &mut Parser<'_>, mut state: BodyState) -> Result<Transition> {
    loop {
        if if state.block { p.eat("}") } else { p.done() } {
            if !state.directive && state.body.has_lexical {
                check_scope(
                    &p.unit,
                    &mut p.compile_budget,
                    state.body.statements.iter(),
                    true,
                )?;
            }
            return Ok(Transition::Done(Output::Body(state.body)));
        }
        if p.done() {
            return Err(p.error(if state.directive {
                "unterminated function body"
            } else {
                "unterminated block"
            }));
        }
        let before = Before::new(p);
        if let Some(id) = list_leaf(p)? {
            append_body(p, &mut state, before, id)?;
        } else {
            return child(
                BodyFrame::Waiting(state, before),
                Frame::Start { declarations: true },
            );
        }
    }
}
// These bounded leaf forms complete in the containing list frame. Their existing
// name/list/emission charges cover the constant dispatch work; no additional
// continuation or speculative scan is needed for each bare declaration.
fn list_leaf(p: &mut Parser<'_>) -> Result<Option<StmtId>> {
    let leaf = p.is(";")
        || (p.declaration_start()
            && matches!(
                p.tokens.get(p.pos + 1).map(|t| &t.kind),
                Some(TokenKind::Word(_))
            )
            && p.token_is(p.pos + 2, ";"));
    if !leaf {
        return Ok(None);
    }
    let result = (|| {
        if p.eat(";") {
            return p.emit_stmt(Stmt::Empty);
        }
        let kind = declaration_kind(p)?;
        let name = p.binding_identifier()?;
        validate_declaration(p, &name, kind, false, false)?;
        let mut bindings = Vec::new();
        push(&mut bindings, (name, None), &mut p.compile_budget)?;
        p.semicolon()?;
        p.emit_stmt(Stmt::Var(bindings, kind))
    })();
    result.map(Some)
}

pub(super) fn step(p: &mut Parser<'_>, frame: Frame, output: Option<Output>) -> Result<Transition> {
    match frame {
        Frame::Start { declarations } => start(p, declarations),
        Frame::Controlled => child(
            Frame::ControlledDone,
            Frame::Start {
                declarations: false,
            },
        ),
        Frame::ControlledDone => {
            let id = statement(output);
            if matches!(
                p.unit.stmt(id),
                Stmt::Var(_, DeclarationKind::Let | DeclarationKind::Const)
            ) {
                return Err(p.error("lexical declarations require a statement list"));
            }
            if matches!(p.unit.stmt(id), Stmt::Function(..)) {
                if p.strict {
                    return Err(p.error("strict function declarations require a statement list"));
                }
                return Err(ScriptError::unsupported(
                    "legacy conditional function declarations are not implemented",
                ));
            }
            Ok(Transition::Done(Output::Statement(id)))
        }
        Frame::Expression => {
            let id = expression(output);
            p.semicolon()?;
            done_stmt(p, Stmt::Expr(id))
        }
        Frame::Declaration(for_head) => {
            let kind = declaration_kind(p)?;
            declaration_next(p, Vec::new(), kind, for_head)
        }
        Frame::DeclarationDone => {
            let record = declaration(output);
            p.semicolon()?;
            done_stmt(p, record)
        }
        Frame::DeclarationDefault(mut bindings, kind, name, for_head) => {
            let id = expression(output);
            push(&mut bindings, (name, Some(id)), &mut p.compile_budget)?;
            if p.eat(",") {
                declaration_next(p, bindings, kind, for_head)
            } else {
                Ok(Transition::Done(Output::Declaration(Stmt::Var(
                    bindings, kind,
                ))))
            }
        }
        Frame::Function(name) => {
            let mut code = function(output);
            let saved = p.strict;
            p.strict = code.strict;
            p.validate_identifier(&name, true)?;
            p.strict = saved;
            code.name = Some(p.copy_identifier(&name)?);
            let id = p.emit_function(code)?;
            done_stmt(p, Stmt::Function(name, id))
        }
        Frame::Block => done_stmt(p, Stmt::Block(body(output).statements)),
        Frame::Label(start, target) => {
            let id = statement(output);
            p.labels.truncate(start);
            done_stmt(p, Stmt::Label(target, id))
        }
        Frame::SwitchExpression => {
            let value = expression(output);
            p.expect(")")?;
            p.expect("{")?;
            p.switch_depth += 1;
            switch_next(
                p,
                Switch {
                    value,
                    cases: Vec::new(),
                    default: false,
                    lexical: false,
                },
            )
        }
        Frame::SwitchNext(state) => switch_next(p, state),
        Frame::SwitchCase(state) => {
            let condition = expression(output);
            p.expect(":")?;
            switch_body(p, state, Some(condition), Vec::new())
        }
        Frame::SwitchStatement(mut state, condition, mut statements) => {
            let id = statement(output);
            state.lexical |= p.unit.stmt(id).is_lexical_declaration(true);
            push(&mut statements, id, &mut p.compile_budget)?;
            switch_body(p, state, condition, statements)
        }
        Frame::IfCondition => {
            let id = expression(output);
            p.expect(")")?;
            child(Frame::IfYes(id), Frame::Controlled)
        }
        Frame::IfYes(condition) => {
            let yes = statement(output);
            if p.eat("else") {
                child(Frame::IfNo(condition, yes), Frame::Controlled)
            } else {
                done_stmt(p, Stmt::If(condition, yes, None))
            }
        }
        Frame::IfNo(condition, yes) => {
            done_stmt(p, Stmt::If(condition, yes, Some(statement(output))))
        }
        Frame::WhileCondition => {
            let condition = expression(output);
            p.expect(")")?;
            p.loop_depth += 1;
            child(Frame::WhileBody(condition), Frame::Controlled)
        }
        Frame::WhileBody(condition) => {
            let body = statement(output);
            p.loop_depth -= 1;
            done_stmt(p, Stmt::While(condition, body))
        }
        Frame::DoBody => {
            let body = statement(output);
            p.loop_depth -= 1;
            p.expect("while")?;
            p.expect("(")?;
            child(Frame::DoCondition(body), Expression::Sequence)
        }
        Frame::DoCondition(body) => {
            let condition = expression(output);
            p.expect(")")?;
            p.eat(";");
            done_stmt(p, Stmt::DoWhile(condition, body))
        }
        Frame::ForDeclaration(saved) => for_head(p, Some(declaration(output)), saved, false),
        Frame::ForExpression(saved, forbidden) => {
            for_head(p, Some(Stmt::Expr(expression(output))), saved, forbidden)
        }
        Frame::ForTest(init) => {
            let test = expression(output);
            p.expect(";")?;
            for_update(p, init, Some(test))
        }
        Frame::ForUpdate(init, test) => {
            let update = expression(output);
            p.expect(")")?;
            p.loop_depth += 1;
            child(Frame::ForBody(init, test, Some(update)), Frame::Controlled)
        }
        Frame::ForBody(init, test, update) => {
            let body = statement(output);
            p.loop_depth -= 1;
            if let Some(Stmt::Var(bindings, kind)) = &init
                && *kind != DeclarationKind::Var
            {
                let vars = var_names(&p.unit, &mut p.compile_budget, std::iter::once(&body))?
                    .finish(&mut p.compile_budget)?;
                for (name, _) in bindings {
                    if vars.contains(name, &mut p.compile_budget)? {
                        return Err(p.error("for lexical binding conflicts with var"));
                    }
                }
                check_declaration(bindings, &mut p.compile_budget)?;
            }
            let init = init.map(|record| p.emit_stmt(record)).transpose()?;
            done_stmt(p, Stmt::For(init, test, update, body))
        }
        Frame::ForInObject(binding) => {
            let object = expression(output);
            p.expect(")")?;
            p.loop_depth += 1;
            child(Frame::ForInBody(binding, object), Frame::Controlled)
        }
        Frame::ForInBody(binding, object) => {
            let body = statement(output);
            p.loop_depth -= 1;
            if let ForBinding::Declaration(name, kind) = &binding
                && *kind != DeclarationKind::Var
            {
                let vars = var_names(&p.unit, &mut p.compile_budget, std::iter::once(&body))?
                    .finish(&mut p.compile_budget)?;
                if vars.contains(name, &mut p.compile_budget)? {
                    return Err(p.error("for-in lexical binding conflicts with var"));
                }
            }
            done_stmt(p, Stmt::ForIn(binding, object, body))
        }
        Frame::ForOfObject(binding) => {
            let object = expression(output);
            p.expect(")")?;
            p.loop_depth += 1;
            child(Frame::ForOfBody(binding, object), Frame::Controlled)
        }
        Frame::ForOfBody(binding, object) => {
            let body = statement(output);
            p.loop_depth -= 1;
            if let ForBinding::Declaration(name, kind) = &binding
                && *kind != DeclarationKind::Var
            {
                let vars = var_names(&p.unit, &mut p.compile_budget, std::iter::once(&body))?
                    .finish(&mut p.compile_budget)?;
                if vars.contains(name, &mut p.compile_budget)? {
                    return Err(p.error("for-of lexical binding conflicts with var"));
                }
            }
            done_stmt(p, Stmt::ForOf(binding, object, body))
        }
        Frame::Return => {
            let id = expression(output);
            p.semicolon()?;
            done_stmt(p, Stmt::Return(Some(id)))
        }
        Frame::Throw => {
            let id = expression(output);
            p.semicolon()?;
            done_stmt(p, Stmt::Throw(id))
        }
        Frame::TryBody => {
            let body = p.emit_stmt(Stmt::Block(body(output).statements))?;
            if p.eat("catch") {
                let binding = if p.eat("(") {
                    let name = p.binding_identifier()?;
                    p.expect(")")?;
                    Some(name)
                } else {
                    None
                };
                p.expect("{")?;
                child(
                    Frame::CatchBody(body, binding),
                    BodyFrame::start(true, false),
                )
            } else {
                try_tail(p, body, None)
            }
        }
        Frame::CatchBody(try_body, binding) => {
            let body = body(output).statements;
            if let Some(name) = &binding {
                check_parameter_lexicals(
                    &p.unit,
                    &mut p.compile_budget,
                    &p.tokens[p.pos],
                    std::iter::once(name.as_str()),
                    &body,
                    false,
                )?;
            }
            try_tail(p, try_body, Some(CatchClause { binding, body }))
        }
        Frame::FinallyBody(try_body, handler) => {
            let finalizer = p.emit_stmt(Stmt::Block(body(output).statements))?;
            done_stmt(p, Stmt::Try(try_body, handler, Some(finalizer)))
        }
    }
}
fn start(p: &mut Parser<'_>, declarations: bool) -> Result<Transition> {
    if p.label_start() {
        return labeled(p);
    }
    if p.eat(";") {
        return done_stmt(p, Stmt::Empty);
    }
    if p.eat("{") {
        return child(Frame::Block, BodyFrame::start(true, false));
    }
    if !declarations
        && (p.is("const") || p.is("class") || p.is("let") && p.token_is(p.pos + 1, "["))
    {
        return Err(p.error("declaration is not allowed in statement position"));
    }
    if p.declaration_start() && (declarations || !p.is("let")) {
        return child(Frame::DeclarationDone, Frame::Declaration(false));
    }
    if p.eat("function") {
        if p.is("*") {
            return Err(ScriptError::unsupported(
                "generator functions are not implemented",
            ));
        }
        let name = p.binding_identifier()?;
        return child(Frame::Function(name), functions::Frame::Start(false));
    }
    if p.eat("switch") {
        p.expect("(")?;
        return child(Frame::SwitchExpression, Expression::Assignment);
    }
    if p.eat("if") {
        p.expect("(")?;
        return child(Frame::IfCondition, Expression::Sequence);
    }
    if p.eat("while") {
        p.expect("(")?;
        return child(Frame::WhileCondition, Expression::Sequence);
    }
    if p.eat("do") {
        p.loop_depth += 1;
        return child(Frame::DoBody, Frame::Controlled);
    }
    if p.eat("for") {
        if p.is("await") {
            return Err(ScriptError::unsupported("for-await is not supported"));
        }
        p.expect("(")?;
        let saved = p.allow_in;
        p.allow_in = false;
        if p.is(";") {
            return for_head(p, None, saved, false);
        }
        if p.declaration_start() {
            return child(Frame::ForDeclaration(saved), Frame::Declaration(true));
        }
        let forbidden = p.is("let") || (p.is("async") && p.token_is(p.pos + 1, "of"));
        return child(Frame::ForExpression(saved, forbidden), Expression::Sequence);
    }
    if p.eat("return") {
        if p.function_depth == 0 {
            return Err(p.error("return outside function"));
        }
        if p.is(";") || p.is("}") || p.done() || p.tokens[p.pos].line_break_before {
            p.semicolon()?;
            return done_stmt(p, Stmt::Return(None));
        }
        return child(Frame::Return, Expression::Sequence);
    }
    if p.eat("throw") {
        if p.tokens[p.pos].line_break_before {
            return Err(p.error("line break is not allowed after throw"));
        }
        return child(Frame::Throw, Expression::Sequence);
    }
    if p.eat("try") {
        p.expect("{")?;
        return child(Frame::TryBody, BodyFrame::start(true, false));
    }
    if p.eat("break") {
        let target = p.control_target(false)?;
        p.semicolon()?;
        return done_stmt(p, Stmt::Break(target));
    }
    if p.eat("continue") {
        let target = p.control_target(true)?;
        p.semicolon()?;
        return done_stmt(p, Stmt::Continue(target));
    }
    for unsupported in [
        "class", "import", "export", "catch", "finally", "do", "with",
    ] {
        if p.is(unsupported) {
            if p.strict && matches!(unsupported, "with" | "yield") {
                return Err(p.error(format!("'{unsupported}' is forbidden in strict code")));
            }
            return Err(ScriptError::unsupported(format!(
                "'{unsupported}' is not supported"
            )));
        }
    }
    child(Frame::Expression, Expression::Sequence)
}
fn labeled(p: &mut Parser<'_>) -> Result<Transition> {
    let start = p.labels.len();
    let target = p.next_label;
    p.next_label = p
        .next_label
        .checked_add(1)
        .ok_or_else(|| p.resource_error("label target limit exceeded"))?;
    while p.label_start() {
        let name = p.identifier()?;
        p.validate_identifier(&name, false)?;
        p.expect(":")?;
        if p.label_target(&name)?.is_some() {
            return Err(p.error("duplicate active label"));
        }
        if p.labels.len() >= MAX_DEPTH {
            return Err(p.resource_error("active label limit exceeded"));
        }
        push(
            &mut p.labels,
            ActiveLabel {
                name,
                target,
                iteration: false,
            },
            &mut p.compile_budget,
        )?;
    }
    let iteration = p.is("while") || p.is("do") || p.is("for");
    for label in &mut p.labels[start..] {
        label.iteration = iteration;
    }
    child(Frame::Label(start, target), Frame::Controlled)
}
fn declaration_kind(p: &mut Parser<'_>) -> Result<DeclarationKind> {
    if p.eat("const") {
        Ok(DeclarationKind::Const)
    } else if p.eat("var") {
        Ok(DeclarationKind::Var)
    } else {
        p.expect("let")?;
        Ok(DeclarationKind::Let)
    }
}
fn validate_declaration(
    p: &Parser<'_>,
    name: &str,
    kind: DeclarationKind,
    initializer: bool,
    for_head: bool,
) -> Result<()> {
    if kind != DeclarationKind::Var && name == "let" {
        return Err(p.error("lexical declaration cannot bind let"));
    }
    if kind == DeclarationKind::Const && !initializer && !(for_head && (p.is("in") || p.is("of"))) {
        return Err(p.error("const declaration needs a value"));
    }
    Ok(())
}
fn declaration_next(
    p: &mut Parser<'_>,
    mut bindings: Vec<(String, Option<ExprId>)>,
    kind: DeclarationKind,
    for_head: bool,
) -> Result<Transition> {
    loop {
        let name = p.binding_identifier()?;
        if kind != DeclarationKind::Var && name == "let" {
            return Err(p.error("lexical declaration cannot bind let"));
        }
        if p.eat("=") {
            return child(
                Frame::DeclarationDefault(bindings, kind, name, for_head),
                Expression::Assignment,
            );
        }
        validate_declaration(p, &name, kind, false, for_head)?;
        push(&mut bindings, (name, None), &mut p.compile_budget)?;
        if !p.eat(",") {
            return Ok(Transition::Done(Output::Declaration(Stmt::Var(
                bindings, kind,
            ))));
        }
    }
}
fn switch_next(p: &mut Parser<'_>, mut state: Switch) -> Result<Transition> {
    if p.eat("}") {
        p.switch_depth -= 1;
        if state.lexical {
            p.compile_budget
                .work(state.cases.len().saturating_mul(2))
                .map_err(regexp_error)?;
            check_scope(
                &p.unit,
                &mut p.compile_budget,
                state.cases.iter().flat_map(|(_, body)| body.iter()),
                true,
            )?;
        }
        return done_stmt(p, Stmt::Switch(state.value, state.cases));
    }
    if p.eat("case") {
        return child(Frame::SwitchCase(state), Expression::Assignment);
    }
    if p.eat("default") {
        if state.default {
            return Err(p.error("duplicate switch default"));
        }
        state.default = true;
        p.expect(":")?;
        return switch_body(p, state, None, Vec::new());
    }
    Err(p.error("expected case or default"))
}
fn switch_body(
    p: &mut Parser<'_>,
    mut state: Switch,
    condition: Option<ExprId>,
    mut statements: Vec<StmtId>,
) -> Result<Transition> {
    while !p.is("case") && !p.is("default") && !p.is("}") {
        if p.done() {
            return Err(p.error("unterminated switch"));
        }
        if let Some(id) = list_leaf(p)? {
            state.lexical |= p.unit.stmt(id).is_lexical_declaration(true);
            push(&mut statements, id, &mut p.compile_budget)?;
        } else {
            return child(
                Frame::SwitchStatement(state, condition, statements),
                Frame::Start { declarations: true },
            );
        }
    }
    push(
        &mut state.cases,
        (condition, statements),
        &mut p.compile_budget,
    )?;
    Ok(Transition::Again(Frame::SwitchNext(state).into()))
}
fn for_head(
    p: &mut Parser<'_>,
    init: Option<Stmt>,
    saved: bool,
    forbidden: bool,
) -> Result<Transition> {
    p.allow_in = saved;
    let for_of = p.is("of");
    if p.eat("in") || p.eat("of") {
        if for_of && forbidden {
            return Err(p.error("invalid for-of assignment head"));
        }
        let binding = match init {
            Some(Stmt::Var(mut bindings, kind))
                if bindings.len() == 1 && bindings[0].1.is_none() =>
            {
                ForBinding::Declaration(bindings.remove(0).0, kind)
            }
            Some(Stmt::Expr(target))
                if matches!(p.unit.expr(target), Expr::Ident(_) | Expr::Member(..)) =>
            {
                p.assignment_target(target)?;
                ForBinding::Target(target)
            }
            Some(Stmt::Expr(target))
                if for_of && matches!(p.unit.expr(target), Expr::Array(_) | Expr::Object(_)) =>
            {
                return Err(ScriptError::unsupported(
                    "destructuring for-of targets are not supported",
                ));
            }
            _ => return Err(p.error("invalid for-in binding")),
        };
        return if for_of {
            child(Frame::ForOfObject(binding), Expression::Assignment)
        } else {
            child(Frame::ForInObject(binding), Expression::Sequence)
        };
    }
    p.expect(";")?;
    if p.eat(";") {
        for_update(p, init, None)
    } else {
        child(Frame::ForTest(init), Expression::Sequence)
    }
}
fn for_update(p: &mut Parser<'_>, init: Option<Stmt>, test: Option<ExprId>) -> Result<Transition> {
    if p.eat(")") {
        p.loop_depth += 1;
        child(Frame::ForBody(init, test, None), Frame::Controlled)
    } else {
        child(Frame::ForUpdate(init, test), Expression::Sequence)
    }
}
fn try_tail(p: &mut Parser<'_>, body: StmtId, handler: Option<CatchClause>) -> Result<Transition> {
    if p.eat("finally") {
        p.expect("{")?;
        child(
            Frame::FinallyBody(body, handler),
            BodyFrame::start(true, false),
        )
    } else if handler.is_none() {
        Err(p.error("try requires catch or finally"))
    } else {
        done_stmt(p, Stmt::Try(body, handler, None))
    }
}
