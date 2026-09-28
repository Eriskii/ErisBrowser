//! Direct flat syntax construction. Grammar recursion retains its existing guards.
//! Child IDs never own syntax, including partially parsed functions/defaults.
use super::*;
macro_rules! emit {
    ($parser:expr, $record:expr) => {{
        let record = $record;
        $parser.emit_expr(record)
    }};
}
use code::{
    CatchClause, Expr, ExprId, ForBinding, Function as FunctionCode, FunctionId, ObjectEntry,
    Parameter, Stmt, StmtId,
};

// IdentifierName syntax is needed only until an object entry is complete.
enum PropertyName {
    Literal(JsString, bool),
    Computed(ExprId),
}
impl PropertyName {
    fn finish(self) -> code::PropertyName {
        match self {
            Self::Literal(value, _) => code::PropertyName::Literal(value),
            Self::Computed(value) => code::PropertyName::Computed(value),
        }
    }
}

struct ActiveLabel {
    name: String,
    target: usize,
    iteration: bool,
}

pub(super) struct Parser<'source> {
    tokens: tokens::Tokens,
    source: &'source str,
    lex_work: usize,
    compile_budget: regexp::Budget,
    unit: code::Unit,
    pos: usize,
    depth: usize,
    function_depth: usize,
    loop_depth: usize,
    switch_depth: usize,
    labels: Vec<ActiveLabel>,
    next_label: usize,
    allow_in: bool,
    strict: bool,
}
impl<'source> Parser<'source> {
    pub(super) fn program(source: &'source str) -> Result<Rc<code::Unit>> {
        Self::program_context(source, false, false)
    }
    pub(super) fn program_context(
        source: &'source str,
        function: bool,
        strict: bool,
    ) -> Result<Rc<code::Unit>> {
        let mut parser = Self::parse_context(source, function, strict)?;
        parser.unit.finish(&mut parser.compile_budget)
    }
    fn parse_context(source: &'source str, function: bool, strict: bool) -> Result<Self> {
        let budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        let mut parser = Self::start(source, function, strict, budget)?;
        parser.parse_body()?;
        Ok(parser)
    }
    fn start(
        source: &'source str,
        function: bool,
        strict: bool,
        mut budget: regexp::Budget,
    ) -> Result<Self> {
        if source.len() > MAX_SOURCE {
            return Err(ScriptError::resource("script source limit exceeded"));
        }
        Ok(Self {
            tokens: lex(source, &mut budget)?,
            source,
            lex_work: source.len(),
            compile_budget: budget,
            unit: code::Unit::empty(strict),
            pos: 0,
            depth: 0,
            function_depth: usize::from(function),
            loop_depth: 0,
            switch_depth: 0,
            labels: Vec::new(),
            next_label: 0,
            allow_in: true,
            strict,
        })
    }
    fn parse_body(&mut self) -> Result<()> {
        let (body, _, has_lexical) = self.directive_body(false)?;
        if has_lexical {
            check_scope(&self.unit, &mut self.compile_budget, body.iter(), false)?;
        }
        self.unit.body = body;
        self.unit.strict = self.strict;
        Ok(())
    }
    pub(super) fn handler(source: &'source str, name: String) -> Result<code::FunctionRef> {
        let mut parser = Self::parse_context(source, true, false)?;
        compile_allocate(&mut parser.compile_budget, 256 + name.len())?;
        let event = parser.copy_identifier("event")?;
        let mut params = Vec::new();
        push(
            &mut params,
            Parameter {
                name: event,
                initializer: None,
                rest: false,
            },
            &mut parser.compile_budget,
        )?;
        let function = FunctionCode {
            params,
            body: std::mem::take(&mut parser.unit.body),
            name: Some(name),
            arrow: false,
            self_name: false,
            constructable: false,
            strict: parser.strict,
        };
        let id = parser.emit_function(function)?;
        let unit = parser.unit.finish(&mut parser.compile_budget)?;
        Ok(code::FunctionRef::new(&unit, id))
    }
    fn emit_expr(&mut self, value: Expr) -> Result<ExprId> {
        self.unit.add_expr(value, &mut self.compile_budget)
    }
    fn emit_stmt(&mut self, value: Stmt) -> Result<StmtId> {
        self.unit.add_stmt(value, &mut self.compile_budget)
    }
    fn emit_function(&mut self, value: FunctionCode) -> Result<FunctionId> {
        self.unit.add_function(value, &mut self.compile_budget)
    }
    fn directive_body(&mut self, block: bool) -> Result<(Vec<StmtId>, bool, bool)> {
        let mut own_strict = false;
        let mut has_lexical = false;
        let mut body = Vec::new();
        if !block {
            // The already lexed prefix bounds its number of root statements.
            // Reserve that many ID slots once (at most 256 KiB), avoiding
            // repeated copies of a long top-level body. RegExp/template rescans
            // can extend this prefix; subsequent growth uses the normal ledger.
            let capacity = self.tokens.len().saturating_sub(1).min(MAX_TOKENS);
            if capacity != 0 {
                self.compile_budget.work(1).map_err(regexp_error)?;
                compile_allocate(
                    &mut self.compile_budget,
                    capacity * std::mem::size_of::<StmtId>() + 32,
                )?;
                body.try_reserve_exact(capacity)
                    .map_err(|_| ScriptError::resource("parser body allocation failed"))?;
            }
        }
        let mut prologue = true;
        let start = self.pos;
        while !(if block { self.eat("}") } else { self.done() }) {
            if self.done() {
                return Err(self.error("unterminated function body"));
            }
            let string_literal = self.tokens[self.pos].string_literal;
            let use_strict = self.tokens[self.pos].use_strict;
            let before = self.pos;
            let statement = self.statement()?;
            let bare_string = string_literal
                && matches!(self.unit.stmt(statement), Stmt::Expr(id) if matches!(self.unit.expr(*id), Expr::Literal(Value::String(_))))
                && (self.pos == before + 1
                    || self.pos == before + 2
                        && matches!(&self.tokens[before + 1].kind, TokenKind::Symbol(s) if s == ";"));
            if prologue && bare_string {
                if use_strict {
                    own_strict = true;
                    self.strict = true;
                    if self
                        .tokens
                        .range(start..self.pos)
                        .any(|token| token.legacy_literal)
                    {
                        return Err(self.error("legacy escapes are forbidden in strict directives"));
                    }
                }
            } else {
                prologue = false;
            }
            has_lexical |= self.unit.stmt(statement).is_lexical_declaration(false);
            push(&mut body, statement, &mut self.compile_budget)?;
        }
        Ok((body, own_strict, has_lexical))
    }
    fn done(&self) -> bool {
        matches!(self.tokens[self.pos].kind, TokenKind::End)
    }
    fn is(&self, text: &str) -> bool {
        self.token_is(self.pos, text)
    }
    fn token_is(&self, pos: usize, text: &str) -> bool {
        match self.tokens.get(pos).map(|token| &token.kind) {
            Some(TokenKind::Word(word)) => !word.escaped && &*word.value == text,
            Some(TokenKind::Symbol(symbol)) => symbol == text,
            _ => false,
        }
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.is(text) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) -> Result<()> {
        if self.eat(text) {
            Ok(())
        } else {
            Err(self.error(format!("expected '{text}'")))
        }
    }
    fn error(&self, message: impl Into<String>) -> ScriptError {
        if let TokenKind::Invalid(error) = &self.tokens[self.pos].kind {
            return (**error).clone();
        }
        ScriptError::at(message, self.tokens[self.pos].offset)
    }
    fn resource_error(&self, message: impl Into<String>) -> ScriptError {
        let mut error = ScriptError::resource(message);
        error.offset = Some(self.tokens[self.pos].offset);
        error
    }
    fn identifier(&mut self) -> Result<String> {
        if let TokenKind::Word(s) = &self.tokens[self.pos].kind {
            let value = s.value.clone();
            let name = self.copy_identifier(&value)?;
            self.pos += 1;
            Ok(name)
        } else {
            Err(self.error("expected identifier"))
        }
    }
    fn copy_identifier(&mut self, name: &str) -> Result<String> {
        self.compile_budget
            .work(1 + name.len() / 8)
            .map_err(regexp_error)?;
        compile_allocate(&mut self.compile_budget, name.len() + 24)?;
        let mut copy = String::new();
        copy.try_reserve_exact(name.len())
            .map_err(|_| ScriptError::resource("identifier copy allocation failed"))?;
        copy.push_str(name);
        Ok(copy)
    }
    fn binding_identifier(&mut self) -> Result<String> {
        if self.is("{") || self.is("[") || self.is(".") {
            return Err(ScriptError::unsupported(
                "destructuring and rest bindings are not implemented",
            ));
        }
        let name = self.identifier()?;
        self.validate_identifier(&name, true)?;
        Ok(name)
    }
    fn validate_identifier(&self, name: &str, binding: bool) -> Result<()> {
        if matches!(
            name,
            "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "enum"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
        ) || self.strict
            && (matches!(
                name,
                "implements"
                    | "interface"
                    | "let"
                    | "package"
                    | "private"
                    | "protected"
                    | "public"
                    | "static"
                    | "yield"
            ) || binding && matches!(name, "eval" | "arguments"))
        {
            return Err(self.error(format!(
                "invalid {} '{name}'",
                if binding { "binding" } else { "identifier" }
            )));
        }
        Ok(())
    }
    fn assignment_target(&self, target: ExprId) -> Result<()> {
        let target = self.unit.expr(target);
        if !matches!(target, Expr::Ident(name) if name != "this")
            && !matches!(target, Expr::Member(..))
        {
            return Err(self.error("invalid assignment target"));
        }
        if self.strict
            && matches!(target, Expr::Ident(name) if matches!(name.as_str(), "eval"|"arguments"))
        {
            return Err(self.error("strict assignment to eval or arguments"));
        }
        Ok(())
    }
    fn semicolon(&mut self) -> Result<()> {
        if self.eat(";") || self.done() || self.is("}") || self.tokens[self.pos].line_break_before {
            Ok(())
        } else {
            Err(self.error("expected ';'"))
        }
    }
    fn enter(&mut self) -> Result<()> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(self.resource_error("parser nesting limit exceeded"));
        }
        Ok(())
    }
    fn statement(&mut self) -> Result<StmtId> {
        self.statement_context(true)
    }
    fn statement_context(&mut self, declarations: bool) -> Result<StmtId> {
        // A label retains labeled_statement and controlled_statement while its
        // body parses. Account for those two additional native frames before
        // entering statement_inner; flat aliases share this single charge.
        let weight = if self.label_start() { 3 } else { 1 };
        for _ in 0..weight {
            self.enter()?;
        }
        let result = self.statement_inner(declarations);
        self.depth -= weight;
        self.emit_stmt(result?)
    }
    fn controlled_statement(&mut self) -> Result<StmtId> {
        let statement = self.statement_context(false)?;
        if matches!(
            self.unit.stmt(statement),
            Stmt::Var(_, DeclarationKind::Let | DeclarationKind::Const)
        ) {
            return Err(self.error("lexical declarations require a statement list"));
        }
        if matches!(self.unit.stmt(statement), Stmt::Function(..)) {
            if self.strict {
                return Err(self.error("strict function declarations require a statement list"));
            }
            return Err(ScriptError::unsupported(
                "legacy conditional function declarations are not implemented",
            ));
        }
        Ok(statement)
    }
    fn declaration_start(&self) -> bool {
        self.is("var")
            || self.is("const")
            || self.is("let")
                && (self.token_is(self.pos + 1, "[")
                    || self.token_is(self.pos + 1, "{")
                    || matches!(
                        self.tokens.get(self.pos + 1).map(|token| &token.kind),
                        Some(TokenKind::Word(_))
                    ) && !self.token_is(self.pos + 1, "in")
                        && !self.token_is(self.pos + 1, "instanceof"))
    }
    fn unsupported_async_start(&self) -> bool {
        self.is("async")
            && self
                .tokens
                .get(self.pos + 1)
                .is_some_and(|token| !token.line_break_before)
            && (self.token_is(self.pos + 1, "function")
                || matches!(
                    self.tokens.get(self.pos + 1).map(|token| &token.kind),
                    Some(TokenKind::Word(_))
                ) && self.token_is(self.pos + 2, "=>")
                    && !self.tokens[self.pos + 2].line_break_before)
    }
    fn label_start(&self) -> bool {
        matches!(self.tokens[self.pos].kind, TokenKind::Word(_)) && self.token_is(self.pos + 1, ":")
    }
    fn label_target(&mut self, name: &str) -> Result<Option<(usize, bool)>> {
        let work = self.labels.iter().fold(1usize, |work, label| {
            work.saturating_add(1 + name.len().min(label.name.len()) / 8)
        });
        self.compile_budget.work(work).map_err(regexp_error)?;
        Ok(self
            .labels
            .iter()
            .rev()
            .find(|label| label.name == name)
            .map(|label| (label.target, label.iteration)))
    }
    fn labeled_statement(&mut self) -> Result<Stmt> {
        let start = self.labels.len();
        let target = self.next_label;
        self.next_label = self
            .next_label
            .checked_add(1)
            .ok_or_else(|| self.resource_error("label target limit exceeded"))?;
        // Consecutive labels all name the same statement. Flatten their aliases
        // into one target without building a recursive chain of AST wrappers.
        while self.label_start() {
            let name = self.identifier()?;
            self.validate_identifier(&name, false)?;
            self.expect(":")?;
            if self.label_target(&name)?.is_some() {
                return Err(self.error("duplicate active label"));
            }
            if self.labels.len() >= MAX_DEPTH {
                return Err(self.resource_error("active label limit exceeded"));
            }
            push(
                &mut self.labels,
                ActiveLabel {
                    name,
                    target,
                    iteration: false,
                },
                &mut self.compile_budget,
            )?;
        }
        let iteration = self.is("while") || self.is("do") || self.is("for");
        for label in &mut self.labels[start..] {
            label.iteration = iteration;
        }
        let body = self.controlled_statement();
        self.labels.truncate(start);
        Ok(Stmt::Label(target, body?))
    }
    fn control_target(&mut self, continuing: bool) -> Result<Option<usize>> {
        if !self.tokens[self.pos].line_break_before
            && matches!(self.tokens[self.pos].kind, TokenKind::Word(_))
        {
            let name = self.identifier()?;
            self.validate_identifier(&name, false)?;
            let Some((target, iteration)) = self.label_target(&name)? else {
                return Err(self.error("unknown control-flow label"));
            };
            if continuing && !iteration {
                return Err(self.error("continue label does not name an iteration statement"));
            }
            return Ok(Some(target));
        }
        if self.loop_depth == 0 && (continuing || self.switch_depth == 0) {
            return Err(self.error(if continuing {
                "continue outside loop"
            } else {
                "break outside loop or switch"
            }));
        }
        Ok(None)
    }
    fn statement_inner(&mut self, declarations: bool) -> Result<Stmt> {
        if self.label_start() {
            return self.labeled_statement();
        }
        if self.eat(";") {
            return Ok(Stmt::Empty);
        }
        if self.eat("{") {
            return Ok(Stmt::Block(self.block()?));
        }
        if !declarations
            && (self.is("const")
                || self.is("class")
                || self.is("let") && self.token_is(self.pos + 1, "["))
        {
            return Err(self.error("declaration is not allowed in statement position"));
        }
        // In Statement position, sloppy `let` is an IdentifierReference except
        // for the forbidden `let [` lookahead. A newline can terminate that
        // expression; it must not be consumed as a lexical declaration.
        if self.declaration_start() && (declarations || !self.is("let")) {
            let declaration = self.declaration()?;
            self.semicolon()?;
            return Ok(declaration);
        }
        if self.eat("function") {
            return self.parse_function_statement();
        }
        if self.eat("switch") {
            return self.parse_switch_statement();
        }
        if self.eat("if") {
            return self.parse_if_statement();
        }
        if self.eat("while") {
            return self.parse_while_statement();
        }
        if self.eat("do") {
            return self.parse_do_statement();
        }
        if self.eat("for") {
            return self.parse_for_statement();
        }
        if self.eat("return") {
            return self.parse_return_statement();
        }
        if self.eat("throw") {
            return self.parse_throw_statement();
        }
        if self.eat("try") {
            return self.parse_try_statement();
        }
        if self.eat("break") {
            let target = self.control_target(false)?;
            self.semicolon()?;
            return Ok(Stmt::Break(target));
        }
        if self.eat("continue") {
            let target = self.control_target(true)?;
            self.semicolon()?;
            return Ok(Stmt::Continue(target));
        }
        for unsupported in [
            "class", "import", "export", "catch", "finally", "do", "with",
        ] {
            if self.is(unsupported) {
                if self.strict && matches!(unsupported, "with" | "yield") {
                    return Err(self.error(format!("'{unsupported}' is forbidden in strict code")));
                }
                return Err(ScriptError::unsupported(format!(
                    "'{unsupported}' is not supported"
                )));
            }
        }
        let expression = self.sequence()?;
        self.semicolon()?;
        Ok(Stmt::Expr(expression))
    }
    fn parse_function_statement(&mut self) -> Result<Stmt> {
        if self.is("*") {
            return Err(ScriptError::unsupported(
                "generator functions are not implemented",
            ));
        }
        let name = self.binding_identifier()?;
        let mut code = self.function(false)?;
        let saved = self.strict;
        self.strict = code.strict;
        self.validate_identifier(&name, true)?;
        self.strict = saved;
        code.name = Some(self.copy_identifier(&name)?);
        Ok(Stmt::Function(name, self.emit_function(code)?))
    }
    fn parse_switch_statement(&mut self) -> Result<Stmt> {
        self.expect("(")?;
        let value = self.expression()?;
        self.expect(")")?;
        self.expect("{")?;
        let mut cases = Vec::new();
        let mut has_default = false;
        let mut has_lexical = false;
        self.switch_depth += 1;
        while !self.eat("}") {
            let condition = if self.eat("case") {
                Some(self.expression()?)
            } else if self.eat("default") {
                if has_default {
                    return Err(self.error("duplicate switch default"));
                }
                has_default = true;
                None
            } else {
                return Err(self.error("expected case or default"));
            };
            self.expect(":")?;
            let mut body = Vec::new();
            while !self.is("case") && !self.is("default") && !self.is("}") {
                if self.done() {
                    return Err(self.error("unterminated switch"));
                }
                let statement = self.statement()?;
                has_lexical |= self.unit.stmt(statement).is_lexical_declaration(true);
                push(&mut body, statement, &mut self.compile_budget)?;
            }
            push(&mut cases, (condition, body), &mut self.compile_budget)?;
        }
        self.switch_depth -= 1;
        if has_lexical {
            // The two flattened root passes can skip empty case lists without
            // yielding a statement. Charge those case visits before scanning.
            self.compile_budget
                .work(cases.len().saturating_mul(2))
                .map_err(regexp_error)?;
            check_scope(
                &self.unit,
                &mut self.compile_budget,
                cases.iter().flat_map(|(_, body)| body.iter()),
                true,
            )?;
        }
        Ok(Stmt::Switch(value, cases))
    }
    fn parse_if_statement(&mut self) -> Result<Stmt> {
        self.expect("(")?;
        let condition = self.sequence()?;
        self.expect(")")?;
        let yes = self.controlled_statement()?;
        let no = if self.eat("else") {
            Some(self.controlled_statement()?)
        } else {
            None
        };
        if self.strict
            && (matches!(self.unit.stmt(yes), Stmt::Function(..))
                || no.is_some_and(|s| matches!(self.unit.stmt(s), Stmt::Function(..))))
        {
            return Err(self.error("strict function declarations require a statement list"));
        }
        Ok(Stmt::If(condition, yes, no))
    }
    fn parse_while_statement(&mut self) -> Result<Stmt> {
        self.expect("(")?;
        let condition = self.sequence()?;
        self.expect(")")?;
        self.loop_depth += 1;
        let body = self.controlled_statement()?;
        self.loop_depth -= 1;
        Ok(Stmt::While(condition, body))
    }
    fn parse_do_statement(&mut self) -> Result<Stmt> {
        self.loop_depth += 1;
        let body = self.controlled_statement()?;
        self.loop_depth -= 1;
        self.expect("while")?;
        self.expect("(")?;
        let condition = self.sequence()?;
        self.expect(")")?;
        self.eat(";");
        Ok(Stmt::DoWhile(condition, body))
    }
    fn parse_for_statement(&mut self) -> Result<Stmt> {
        self.expect("(")?;
        let saved_in = self.allow_in;
        self.allow_in = false;
        let init = if self.is(";") {
            None
        } else if self.declaration_start() {
            Some(self.declaration()?)
        } else {
            Some(Stmt::Expr(self.sequence()?))
        };
        self.allow_in = saved_in;
        if self.eat("in") {
            let binding = match init {
                Some(Stmt::Var(mut bindings, kind))
                    if bindings.len() == 1 && bindings[0].1.is_none() =>
                {
                    ForBinding::Declaration(bindings.remove(0).0, kind)
                }
                Some(Stmt::Expr(target))
                    if matches!(self.unit.expr(target), Expr::Ident(_) | Expr::Member(..)) =>
                {
                    self.assignment_target(target)?;
                    ForBinding::Target(target)
                }
                _ => return Err(self.error("invalid for-in binding")),
            };
            let object = self.sequence()?;
            self.expect(")")?;
            self.loop_depth += 1;
            let body = self.controlled_statement()?;
            self.loop_depth -= 1;
            if let ForBinding::Declaration(name, kind) = &binding
                && *kind != DeclarationKind::Var
            {
                let vars = var_names(&self.unit, &mut self.compile_budget, std::iter::once(&body))?
                    .finish(&mut self.compile_budget)?;
                if vars.contains(name, &mut self.compile_budget)? {
                    return Err(self.error("for-in lexical binding conflicts with var"));
                }
            }
            return Ok(Stmt::ForIn(binding, object, body));
        }
        self.expect(";")?;
        let test = if self.is(";") {
            None
        } else {
            Some(self.sequence()?)
        };
        self.expect(";")?;
        let update = if self.is(")") {
            None
        } else {
            Some(self.sequence()?)
        };
        self.expect(")")?;
        self.loop_depth += 1;
        let body = self.controlled_statement()?;
        self.loop_depth -= 1;
        if let Some(init) = &init
            && let Stmt::Var(bindings, kind) = init
            && *kind != DeclarationKind::Var
        {
            let vars = var_names(&self.unit, &mut self.compile_budget, std::iter::once(&body))?
                .finish(&mut self.compile_budget)?;
            for (name, _) in bindings {
                if vars.contains(name, &mut self.compile_budget)? {
                    return Err(self.error("for lexical binding conflicts with var"));
                }
            }
            check_declaration(bindings, &mut self.compile_budget)?;
        }
        let init = init.map(|record| self.emit_stmt(record)).transpose()?;
        Ok(Stmt::For(init, test, update, body))
    }
    fn parse_return_statement(&mut self) -> Result<Stmt> {
        if self.function_depth == 0 {
            return Err(self.error("return outside function"));
        }
        let result = if self.is(";")
            || self.is("}")
            || self.done()
            || self.tokens[self.pos].line_break_before
        {
            None
        } else {
            Some(self.sequence()?)
        };
        self.semicolon()?;
        Ok(Stmt::Return(result))
    }
    fn parse_throw_statement(&mut self) -> Result<Stmt> {
        if self.tokens[self.pos].line_break_before {
            return Err(self.error("line break is not allowed after throw"));
        }
        let value = self.sequence()?;
        self.semicolon()?;
        Ok(Stmt::Throw(value))
    }
    fn parse_try_statement(&mut self) -> Result<Stmt> {
        self.expect("{")?;
        let statements = self.block()?;
        let body = self.emit_stmt(Stmt::Block(statements))?;
        let handler = if self.eat("catch") {
            let binding = if self.eat("(") {
                let name = self.binding_identifier()?;
                self.expect(")")?;
                Some(name)
            } else {
                None
            };
            self.expect("{")?;
            let body = self.block()?;
            if let Some(name) = &binding {
                check_parameter_lexicals(
                    &self.unit,
                    &mut self.compile_budget,
                    &self.tokens[self.pos],
                    std::iter::once(name.as_str()),
                    &body,
                    false,
                )?;
            }
            Some(CatchClause { binding, body })
        } else {
            None
        };
        let finalizer = if self.eat("finally") {
            self.expect("{")?;
            let statements = self.block()?;
            Some(self.emit_stmt(Stmt::Block(statements))?)
        } else {
            None
        };
        if handler.is_none() && finalizer.is_none() {
            return Err(self.error("try requires catch or finally"));
        }
        Ok(Stmt::Try(body, handler, finalizer))
    }
    fn declaration(&mut self) -> Result<Stmt> {
        let kind = if self.eat("const") {
            DeclarationKind::Const
        } else if self.eat("var") {
            DeclarationKind::Var
        } else {
            self.expect("let")?;
            DeclarationKind::Let
        };
        let mut bindings = Vec::new();
        loop {
            let name = self.binding_identifier()?;
            if kind != DeclarationKind::Var && name == "let" {
                return Err(self.error("lexical declaration cannot bind let"));
            }
            let value = if self.eat("=") {
                Some(self.expression()?)
            } else {
                None
            };
            if kind == DeclarationKind::Const && value.is_none() && !self.is("in") {
                return Err(self.error("const declaration needs a value"));
            }
            push(&mut bindings, (name, value), &mut self.compile_budget)?;
            if !self.eat(",") {
                break;
            }
        }
        Ok(Stmt::Var(bindings, kind))
    }
    fn block(&mut self) -> Result<Vec<StmtId>> {
        let mut body = Vec::new();
        let mut has_lexical = false;
        while !self.eat("}") {
            if self.done() {
                return Err(self.error("unterminated block"));
            }
            let statement = self.statement()?;
            has_lexical |= self.unit.stmt(statement).is_lexical_declaration(true);
            push(&mut body, statement, &mut self.compile_budget)?;
        }
        if has_lexical {
            check_scope(&self.unit, &mut self.compile_budget, body.iter(), true)?;
        }
        Ok(body)
    }
    fn object_key(&mut self) -> Result<PropertyName> {
        if self.eat("[") {
            // ComputedPropertyName uses AssignmentExpression[+In], rather
            // than Expression: commas need an explicit pair of parentheses.
            let saved = self.allow_in;
            self.allow_in = true;
            let expression = self.expression()?;
            self.expect("]")?;
            self.allow_in = saved;
            return Ok(PropertyName::Computed(expression));
        }
        if self.strict && self.tokens[self.pos].legacy_literal {
            return Err(self.error("legacy literals are forbidden in strict code"));
        }
        let (key, identifier) = match &self.tokens[self.pos].kind {
            TokenKind::Word(word) => {
                let value = word.value.clone();
                (self.identifier_key(&value)?, true)
            }
            TokenKind::String(value) => (value.clone(), false),
            TokenKind::Number(value) => {
                let value = *value;
                compile_allocate(&mut self.compile_budget, 160)?;
                self.compile_budget.work(1).map_err(regexp_error)?;
                (json_number(value).into(), false)
            }
            _ => return Err(self.error("expected object property")),
        };
        self.pos += 1;
        Ok(PropertyName::Literal(key, identifier))
    }
    fn identifier_key(&mut self, name: &str) -> Result<JsString> {
        self.compile_budget
            .work(1 + name.len() / 8)
            .map_err(regexp_error)?;
        compile_allocate(
            &mut self.compile_budget,
            name.len().saturating_mul(4).saturating_add(32),
        )?;
        Ok(JsString::from(name))
    }
    fn parameter(&mut self, name: String, initializer: Option<ExprId>) -> Result<Parameter> {
        Ok(Parameter {
            name,
            initializer,
            rest: false,
        })
    }
    fn rest_parameter(&mut self) -> Result<Parameter> {
        // Identifier rest must be last and cannot have an initializer. Keep
        // malformed ellipses distinct from unsupported binding patterns.
        let start = self.tokens[self.pos].offset;
        for byte in 0..3 {
            if self.tokens[self.pos].offset != start + byte || !self.eat(".") {
                return Err(self.error("invalid rest parameter ellipsis"));
            }
        }
        if self.is(".") {
            return Err(self.error("invalid rest parameter ellipsis"));
        }
        if self.is("{") || self.is("[") {
            let mut error =
                ScriptError::unsupported("destructuring rest parameters are not implemented");
            error.offset = Some(start);
            return Err(error);
        }
        let name = self.binding_identifier()?;
        if !self.is(")") {
            return Err(self.error("rest parameter must be last and cannot have an initializer"));
        }
        let mut parameter = self.parameter(name, None)?;
        parameter.rest = true;
        Ok(parameter)
    }
    fn function(&mut self, unique_parameters: bool) -> Result<FunctionCode> {
        self.expect("(")?;
        let saved = (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
            std::mem::take(&mut self.labels),
        );
        self.loop_depth = 0;
        self.switch_depth = 0;
        self.allow_in = true;
        self.function_depth += 1;
        let mut params = Vec::new();
        if !self.eat(")") {
            loop {
                if self.is(".") {
                    push(
                        &mut params,
                        self.rest_parameter()?,
                        &mut self.compile_budget,
                    )?;
                    self.expect(")")?;
                    break;
                }
                let name = self.binding_identifier()?;
                let initializer = if self.eat("=") {
                    Some(self.expression()?)
                } else {
                    None
                };
                push(
                    &mut params,
                    self.parameter(name, initializer)?,
                    &mut self.compile_budget,
                )?;
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
                if self.eat(")") {
                    break;
                }
            }
        }
        self.expect("{")?;
        let (body, own_strict, has_lexical) = self.directive_body(true)?;
        if has_lexical {
            check_scope(&self.unit, &mut self.compile_budget, body.iter(), false)?;
        }
        self.function_depth -= 1;
        let strict = self.strict;
        let non_simple = params.iter().any(|parameter| !parameter.is_simple());
        if own_strict && non_simple {
            return Err(self.error("use strict directive with non-simple parameters"));
        }
        for param in &params {
            self.validate_identifier(&param.name, true)?;
        }
        check_parameter_lexicals(
            &self.unit,
            &mut self.compile_budget,
            &self.tokens[self.pos],
            params.iter().map(|p| p.name.as_str()),
            &body,
            unique_parameters || strict || non_simple,
        )?;
        (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
            self.labels,
        ) = saved;
        Ok(FunctionCode {
            params,
            body,
            name: None,
            arrow: false,
            self_name: false,
            constructable: true,
            strict,
        })
    }
    fn arrow(&mut self, params: Vec<Parameter>) -> Result<ExprId> {
        let saved = (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
            std::mem::take(&mut self.labels),
        );
        self.loop_depth = 0;
        self.switch_depth = 0;
        self.allow_in = true;
        self.function_depth += 1;
        let (body, own_strict, has_lexical) = if self.eat("{") {
            self.directive_body(true)?
        } else {
            let value = self.expression()?;
            let record = self.emit_stmt(Stmt::Return(Some(value)))?;
            let mut body = Vec::new();
            push(&mut body, record, &mut self.compile_budget)?;
            (body, false, false)
        };
        let strict = self.strict;
        if own_strict && params.iter().any(|parameter| !parameter.is_simple()) {
            return Err(self.error("use strict directive with non-simple parameters"));
        }
        if has_lexical {
            check_scope(&self.unit, &mut self.compile_budget, body.iter(), false)?;
        }
        check_parameter_lexicals(
            &self.unit,
            &mut self.compile_budget,
            &self.tokens[self.pos],
            params.iter().map(|p| p.name.as_str()),
            &body,
            true,
        )?;
        for param in &params {
            self.validate_identifier(&param.name, true)?;
        }
        self.function_depth -= 1;
        (
            self.loop_depth,
            self.switch_depth,
            self.allow_in,
            self.strict,
            self.labels,
        ) = saved;
        let function = FunctionCode {
            params,
            body,
            name: None,
            arrow: true,
            self_name: false,
            constructable: false,
            strict,
        };
        let id = self.emit_function(function)?;
        emit!(self, Expr::Function(id))
    }
    fn expression(&mut self) -> Result<ExprId> {
        self.enter()?;
        let result = self.assignment();
        self.depth -= 1;
        result
    }
    fn sequence(&mut self) -> Result<ExprId> {
        let first = self.expression()?;
        if !self.eat(",") {
            return Ok(first);
        }
        let mut items = Vec::new();
        push(&mut items, first, &mut self.compile_budget)?;
        loop {
            push(&mut items, self.expression()?, &mut self.compile_budget)?;
            if !self.eat(",") {
                break;
            }
        }
        emit!(self, Expr::Sequence(items))
    }
    fn assignment(&mut self) -> Result<ExprId> {
        let mut left = self.binary(1)?;
        if self.eat("?") {
            let yes = self.expression()?;
            self.expect(":")?;
            let no = self.expression()?;
            left = emit!(self, Expr::Conditional(left, yes, no))?;
        }
        for operator in [
            "=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "&=", "^=", "|=",
            "&&=", "||=", "??=",
        ] {
            if self.eat(operator) {
                self.assignment_target(left)?;
                return emit!(
                    self,
                    Expr::Assign(self.copy_identifier(operator)?, left, self.expression()?,)
                );
            }
        }
        Ok(left)
    }
    fn binary(&mut self, min_precedence: u8) -> Result<ExprId> {
        self.enter()?;
        let result = self.binary_inner(min_precedence);
        self.depth -= 1;
        result
    }
    fn binary_inner(&mut self, min_precedence: u8) -> Result<ExprId> {
        let left = self.unary()?;
        let mut operations = Vec::new();
        let mut chain = 0;
        loop {
            let op = match &self.tokens[self.pos].kind {
                TokenKind::Symbol(symbol) => symbol.as_str(),
                TokenKind::Word(word)
                    if !word.escaped && matches!(&*word.value, "in" | "instanceof") =>
                {
                    word.value.as_ref()
                }
                _ => break,
            };
            if op == "in" && !self.allow_in {
                break;
            }
            let (op, precedence) = match op {
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
                "in" => ("in", 7),
                "<<" => ("<<", 8),
                ">>" => (">>", 8),
                ">>>" => (">>>", 8),
                "+" => ("+", 9),
                "-" => ("-", 9),
                "*" => ("*", 10),
                "/" => ("/", 10),
                "%" => ("%", 10),
                "**" => ("**", 11),
                _ => break,
            };
            if precedence < min_precedence {
                break;
            }
            chain += 1;
            if chain > 4096 {
                return Err(self.resource_error("expression chain limit exceeded"));
            }
            self.pos += 1;
            let right = self.binary(if op == "**" {
                precedence
            } else {
                precedence + 1
            })?;
            let op = self.copy_identifier(op)?;
            push(&mut operations, (op, right), &mut self.compile_budget)?;
        }
        // Folding an all-literal string addition chain preserves evaluation
        // order and avoids constructing every quadratic intermediate string.
        if let Expr::Literal(Value::String(first)) = self.unit.expr(left)
            && !operations.is_empty()
            && operations.iter().all(|(op, value)| {
                op == "+" && matches!(self.unit.expr(*value), Expr::Literal(Value::String(_)))
            })
        {
            let length = operations.iter().fold(first.len(), |length, (_, value)| {
                if let Expr::Literal(Value::String(text)) = self.unit.expr(*value) {
                    length.saturating_add(text.len())
                } else {
                    unreachable!()
                }
            });
            if length > MAX_STRING {
                return Err(self.resource_error("script string limit exceeded"));
            }
            self.compile_budget
                .work(length / 8 + 1)
                .map_err(regexp_error)?;
            compile_allocate(&mut self.compile_budget, length * 4 + 32)?;
            let mut units = Vec::new();
            units
                .try_reserve_exact(length)
                .map_err(|_| ScriptError::resource("folded string allocation failed"))?;
            units.extend_from_slice(first.units());
            for (_, value) in operations {
                let Expr::Literal(Value::String(text)) = self.unit.expr(value) else {
                    unreachable!()
                };
                units.extend_from_slice(text.units());
            }
            return emit!(self, Expr::Literal(Value::String(units.into())));
        }
        if operations.is_empty() {
            Ok(left)
        } else {
            emit!(self, Expr::BinaryChain(left, operations))
        }
    }
    fn unary(&mut self) -> Result<ExprId> {
        self.enter()?;
        let result = self.unary_inner();
        self.depth -= 1;
        result
    }
    fn unary_inner(&mut self) -> Result<ExprId> {
        for op in ["!", "-", "+", "~", "typeof", "void", "delete"] {
            if self.eat(op) {
                let value = self.unary()?;
                if self.strict
                    && op == "delete"
                    && matches!(self.unit.expr(value), Expr::Ident(name) if name != "this")
                {
                    return Err(self.error("strict code cannot delete an identifier"));
                }
                return emit!(self, Expr::Unary(self.copy_identifier(op)?, value));
            }
        }
        if self.is("++") || self.is("--") {
            let delta = if self.eat("++") {
                1.0
            } else {
                self.pos += 1;
                -1.0
            };
            let value = self.unary()?;
            self.assignment_target(value)?;
            return emit!(self, Expr::Update(value, delta, true));
        }
        let async_call = self.is("async")
            && self.token_is(self.pos + 1, "(")
            && !self.tokens[self.pos + 1].line_break_before;
        let mut value = self.new_expression()?;
        let mut chain = 0;
        loop {
            chain += 1;
            if chain > MAX_DEPTH / 2 {
                return Err(self.resource_error("expression chain limit exceeded"));
            }
            if self.eat(".") {
                let property = self.identifier()?;
                let property = self.identifier_key(&property)?;
                let property = emit!(self, Expr::Literal(Value::String(property)))?;
                value = emit!(self, Expr::Member(value, property))?;
            } else if self.eat("[") {
                let property = self.expression()?;
                self.expect("]")?;
                value = emit!(self, Expr::Member(value, property))?;
            } else if self.eat("(") {
                let mut arguments = Vec::new();
                if !self.eat(")") {
                    loop {
                        push(&mut arguments, self.expression()?, &mut self.compile_budget)?;
                        if self.eat(")") {
                            break;
                        }
                        self.expect(",")?;
                    }
                }
                if async_call && chain == 1 && self.is("=>") {
                    if self.tokens[self.pos].line_break_before {
                        return Err(self.error("line terminator before arrow"));
                    }
                    return Err(ScriptError::unsupported(
                        "async arrow functions are not implemented",
                    ));
                }
                value = emit!(self, Expr::Call(value, arguments))?;
            } else if matches!(self.tokens[self.pos].kind, TokenKind::TemplateStart) {
                return Err(ScriptError::unsupported(
                    "tagged template literals are not implemented",
                ));
            } else {
                break;
            }
        }
        if !self.tokens[self.pos].line_break_before && self.eat("++") {
            self.assignment_target(value)?;
            value = emit!(self, Expr::Update(value, 1.0, false))?;
        } else if !self.tokens[self.pos].line_break_before && self.eat("--") {
            self.assignment_target(value)?;
            value = emit!(self, Expr::Update(value, -1.0, false))?;
        }
        Ok(value)
    }
    fn primary(&mut self) -> Result<ExprId> {
        if self.unsupported_async_start() {
            return Err(ScriptError::unsupported(
                "async functions are not implemented",
            ));
        }
        if matches!(self.tokens[self.pos].kind, TokenKind::TemplateStart) {
            return self.template_literal();
        }
        if self.is("/") || self.is("/=") {
            return self.regexp_literal();
        }
        if self.eat("(") {
            // Parse the cover grammar once: template and RegExp rescanning
            // changes the token stream, so speculative parsing/rewinding is
            // not safe. Reinterpret only syntactically valid binding forms.
            let saved_in = self.allow_in;
            self.allow_in = true;
            let mut items = Vec::new();
            let mut rest = None;
            let mut trailing_comma = false;
            if !self.eat(")") {
                loop {
                    if self.is(".") {
                        rest = Some(self.rest_parameter()?);
                        self.expect(")")?;
                        break;
                    }
                    let binding_form = matches!(self.tokens[self.pos].kind, TokenKind::Word(_));
                    let expression = self.expression()?;
                    push(
                        &mut items,
                        (expression, binding_form),
                        &mut self.compile_budget,
                    )?;
                    if self.eat(")") {
                        break;
                    }
                    self.expect(",")?;
                    if self.eat(")") {
                        trailing_comma = true;
                        break;
                    }
                }
            }
            self.allow_in = saved_in;
            if self.is("=>") {
                if self.tokens[self.pos].line_break_before {
                    return Err(self.error("line terminator before arrow"));
                }
                self.pos += 1;
                let mut params = Vec::new();
                for (expression, binding_form) in items {
                    if matches!(self.unit.expr(expression), Expr::Array(_) | Expr::Object(_)) {
                        return Err(ScriptError::unsupported(
                            "destructuring parameters are not implemented",
                        ));
                    }
                    if !binding_form {
                        return Err(self.error("invalid arrow parameter"));
                    }
                    let (name, initializer) = match self.unit.take_expr(expression) {
                        Expr::Ident(name) => (name, None),
                        Expr::Assign(operator, target, value) if operator == "=" => {
                            let Expr::Ident(name) = self.unit.take_expr(target) else {
                                return Err(self.error("invalid arrow parameter"));
                            };
                            (name, Some(value))
                        }
                        _ => return Err(self.error("invalid arrow parameter")),
                    };
                    push(
                        &mut params,
                        self.parameter(name, initializer)?,
                        &mut self.compile_budget,
                    )?;
                }
                if let Some(rest) = rest {
                    push(&mut params, rest, &mut self.compile_budget)?;
                }
                return self.arrow(params);
            }
            if rest.is_some() {
                return Err(self.error("rest parameter requires an arrow function"));
            }
            if items.is_empty() || trailing_comma {
                return Err(self.error("invalid parenthesized expression"));
            }
            if items.len() == 1 {
                return Ok(items.pop().unwrap().0);
            }
            let mut values = Vec::new();
            for (value, _) in items {
                push(&mut values, value, &mut self.compile_budget)?;
            }
            return emit!(self, Expr::Sequence(values));
        }
        if self.eat("[") {
            let saved = self.allow_in;
            self.allow_in = true;
            let mut items = Vec::new();
            while !self.eat("]") {
                if self.eat(",") {
                    push(&mut items, None, &mut self.compile_budget)?;
                    continue;
                }
                push(
                    &mut items,
                    Some(self.expression()?),
                    &mut self.compile_budget,
                )?;
                if self.eat("]") {
                    break;
                }
                self.expect(",")?;
            }
            self.allow_in = saved;
            return emit!(self, Expr::Array(items));
        }
        if self.eat("{") {
            let saved = self.allow_in;
            self.allow_in = true;
            let mut entries = Vec::new();
            let mut prototype_setter = false;
            while !self.eat("}") {
                if self.is("*") || self.is(".") {
                    return Err(ScriptError::unsupported(
                        "generator methods and object spread are not implemented",
                    ));
                }
                let accessor = self.is("get") || self.is("set");
                let async_keyword = self.is("async");
                let mut key = self.object_key()?;
                if async_keyword
                    && !self.is(":")
                    && !self.is("(")
                    && !self.is(",")
                    && !self.is("}")
                    && !self.tokens[self.pos].line_break_before
                {
                    return Err(ScriptError::unsupported(
                        "async methods are not implemented",
                    ));
                }
                let value = if accessor
                    && !self.is(":")
                    && !self.is("(")
                    && !self.is(",")
                    && !self.is("}")
                {
                    let setter = matches!(&key, PropertyName::Literal(name, _) if name == &JsString::from("set"));
                    key = self.object_key()?;
                    let mut code = self.function(true)?;
                    if code.params.len() != usize::from(setter)
                        || code.params.iter().any(|parameter| parameter.rest)
                    {
                        return Err(self.error("invalid accessor parameter count"));
                    }
                    code.constructable = false;
                    ObjectEntry::Accessor(self.emit_function(code)?, setter)
                } else if self.is("(") {
                    let mut code = self.function(true)?;
                    code.constructable = false;
                    ObjectEntry::Method(self.emit_function(code)?)
                } else {
                    if self.eat(":") {
                        let is_prototype = matches!(&key, PropertyName::Literal(name, _) if name == &JsString::from("__proto__"));
                        if is_prototype && prototype_setter {
                            return Err(self.error("duplicate __proto__ prototype setter"));
                        }
                        let expression = self.expression()?;
                        if is_prototype {
                            prototype_setter = true;
                            ObjectEntry::Prototype(expression)
                        } else {
                            ObjectEntry::Data(expression)
                        }
                    } else {
                        let PropertyName::Literal(name, true) = &key else {
                            return Err(self.error("object shorthand requires an identifier"));
                        };
                        self.compile_budget
                            .work(1 + name.len() / 8)
                            .map_err(regexp_error)?;
                        compile_allocate(
                            &mut self.compile_budget,
                            name.len().saturating_mul(3).saturating_add(32),
                        )?;
                        let name = name
                            .to_utf8()
                            .map_err(|_| self.error("object shorthand requires an identifier"))?;
                        self.validate_identifier(&name, false)?;
                        ObjectEntry::Data(emit!(self, Expr::Ident(name))?)
                    }
                };
                push(
                    &mut entries,
                    (key.finish(), value),
                    &mut self.compile_budget,
                )?;
                if self.eat("}") {
                    break;
                }
                self.expect(",")?;
            }
            self.allow_in = saved;
            return emit!(self, Expr::Object(entries));
        }
        if self.eat("function") {
            if self.is("*") {
                return Err(ScriptError::unsupported(
                    "generator functions are not implemented",
                ));
            }
            let name = if matches!(self.tokens[self.pos].kind, TokenKind::Word(_)) {
                Some(self.binding_identifier()?)
            } else {
                None
            };
            let mut code = self.function(false)?;
            if let Some(name) = &name {
                let saved = self.strict;
                self.strict = code.strict;
                self.validate_identifier(name, true)?;
                self.strict = saved;
            }
            code.name = name;
            code.self_name = code.name.is_some();
            let code = self.emit_function(code)?;
            return emit!(self, Expr::Function(code));
        }
        let offset = self.tokens[self.pos].offset;
        let legacy_literal = self.tokens[self.pos].legacy_literal;
        let kind = match &self.tokens[self.pos].kind {
            TokenKind::Symbol(_) | TokenKind::End => {
                return Err(ScriptError::at("expected expression", offset));
            }
            kind => kind.clone(), // Rc or scalar payloads; no string allocation.
        };
        if self.strict && legacy_literal {
            return Err(ScriptError::at(
                "legacy literals are forbidden in strict code",
                offset,
            ));
        }
        if !self.done() {
            self.pos += 1;
        }
        match kind {
            TokenKind::RegExp(pattern) => emit!(self, Expr::RegExp(pattern)),
            TokenKind::Invalid(error) => Err((*error).clone()),
            TokenKind::Number(n) => emit!(self, Expr::Literal(Value::Number(n))),
            TokenKind::String(s) => emit!(self, Expr::Literal(Value::String(s))),
            TokenKind::Word(s) if !s.escaped && matches!(&*s.value, "true" | "false") => {
                emit!(self, Expr::Literal(Value::Bool(&*s.value == "true")))
            }
            TokenKind::Word(s) if !s.escaped && &*s.value == "null" => {
                emit!(self, Expr::Literal(Value::Null))
            }
            TokenKind::Word(s) if !s.escaped && &*s.value == "super" => {
                Err(ScriptError::unsupported(
                    "super property and constructor references are not implemented",
                ))
            }
            TokenKind::Word(word) => {
                let s = self.copy_identifier(&word.value)?;
                if self.is("=>") {
                    if self.tokens[self.pos].line_break_before {
                        return Err(self.error("line terminator before arrow"));
                    }
                    self.pos += 1;
                    let parameter = self.parameter(s, None)?;
                    let mut params = Vec::new();
                    push(&mut params, parameter, &mut self.compile_budget)?;
                    self.arrow(params)
                } else {
                    if word.escaped || s != "this" {
                        self.validate_identifier(&s, false)?;
                    }
                    emit!(self, Expr::Ident(s))
                }
            }
            _ => Err(ScriptError::at("expected expression", offset)),
        }
    }
    fn template_literal(&mut self) -> Result<ExprId> {
        let start = self.tokens[self.pos].offset;
        let (head, mut at, _, mut interpolation) =
            quoted_text(self.source, start, '`', Some(&mut self.compile_budget))?;
        self.pos += 1;
        self.rescan_suffix(at)?;
        let mut tail = Vec::new();
        while interpolation {
            if tail.len() >= 4096 {
                return Err(self.resource_error("template substitution limit exceeded"));
            }
            let saved = self.allow_in;
            self.allow_in = true;
            let expression = self.sequence();
            self.allow_in = saved;
            let expression = expression?;
            let start = self.tokens[self.pos].offset;
            self.expect("}")?;
            let (text, end, _, next) =
                quoted_text(self.source, start, '`', Some(&mut self.compile_budget))?;
            push(&mut tail, (expression, text), &mut self.compile_budget)?;
            at = end;
            interpolation = next;
            self.rescan_suffix(at)?;
        }
        self.emit_expr(if tail.is_empty() {
            Expr::Literal(Value::String(head))
        } else {
            Expr::Template(head, tail)
        })
    }
    fn rescan_suffix(&mut self, at: usize) -> Result<()> {
        self.tokens.truncate(self.pos);
        self.lex_work = self.lex_work.saturating_add(self.source.len() - at);
        if self.lex_work > MAX_SOURCE * 32 {
            return Err(ScriptError::resource(
                "script lexical rescan limit exceeded",
            ));
        }
        let suffix = lex(&self.source[at..], &mut self.compile_budget)?;
        if self.tokens.len().saturating_add(suffix.len()) > MAX_TOKENS {
            return Err(ScriptError::resource("script token limit exceeded"));
        }
        reserve_tokens(&mut self.tokens, suffix.len(), &mut self.compile_budget)?;
        self.compile_budget
            .work(suffix.len())
            .map_err(regexp_error)?;
        for mut token in suffix {
            token.offset += at;
            if let TokenKind::Invalid(error) = &mut token.kind {
                // lex just allocated this diagnostic; avoid a potentially
                // allocating clone-on-write path when adjusting its offset.
                let error = Rc::get_mut(error).ok_or_else(|| {
                    ScriptError::resource("shared diagnostic during lexical rescan")
                })?;
                if let Some(offset) = &mut error.offset {
                    *offset += at;
                }
            }
            self.tokens.push(token);
        }
        Ok(())
    }
    fn regexp_literal(&mut self) -> Result<ExprId> {
        let start = self.tokens[self.pos].offset;
        let mut at = start + 1;
        let mut class = false;
        let mut escaped = false;
        let mut end = None;
        while at < self.source.len() {
            let ch = self.source[at..].chars().next().unwrap();
            if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                return Err(ScriptError::at("line terminator in RegExp literal", at));
            }
            if !escaped {
                if ch == '/' && !class {
                    end = Some(at);
                    break;
                }
                if ch == '[' {
                    class = true;
                }
                if ch == ']' {
                    class = false;
                }
            }
            escaped = !escaped && ch == '\\';
            at += ch.len_utf8();
        }
        let end = end.ok_or_else(|| ScriptError::at("unterminated RegExp literal", start))?;
        at = end + 1;
        while at < self.source.len() {
            let ch = self.source[at..].chars().next().unwrap();
            self.compile_budget
                .work(if ch.is_ascii() {
                    1
                } else {
                    IDENTIFIER_LOOKUP_WORK
                })
                .map_err(regexp_error)?;
            if !is_identifier_part(ch) {
                break;
            }
            at += ch.len_utf8();
        }
        let pattern = RegExp::compile(
            self.source[start + 1..end].into(),
            &self.source[end + 1..at].into(),
            &mut self.compile_budget,
        )
        .map_err(|error| {
            let mut error = regexp_error(error);
            if !error.is_resource_limit() && !error.is_unsupported() {
                error.offset = Some(start);
            }
            error
        })?;
        let pattern = Rc::new(pattern);
        self.tokens[self.pos].kind = TokenKind::RegExp(pattern.clone());
        self.pos += 1;
        self.rescan_suffix(at)?;
        emit!(self, Expr::RegExp(pattern))
    }
    fn new_expression(&mut self) -> Result<ExprId> {
        self.enter()?;
        let result = self.new_expression_inner();
        self.depth -= 1;
        result
    }
    fn new_expression_inner(&mut self) -> Result<ExprId> {
        if !self.eat("new") {
            return self.primary();
        }
        let mut constructor = self.new_expression()?;
        let mut count = 0;
        loop {
            count += 1;
            if count > MAX_DEPTH / 2 {
                return Err(self.resource_error("constructor chain limit exceeded"));
            }
            if self.eat(".") {
                let key = self.identifier()?;
                let key = self.identifier_key(&key)?;
                let key = emit!(self, Expr::Literal(Value::String(key)))?;
                constructor = emit!(self, Expr::Member(constructor, key))?;
            } else if self.eat("[") {
                let key = self.expression()?;
                self.expect("]")?;
                constructor = emit!(self, Expr::Member(constructor, key))?;
            } else if matches!(self.tokens[self.pos].kind, TokenKind::TemplateStart) {
                return Err(ScriptError::unsupported(
                    "tagged template literals are not implemented",
                ));
            } else {
                break;
            }
        }
        let mut args = Vec::new();
        if self.eat("(") && !self.eat(")") {
            loop {
                push(&mut args, self.expression()?, &mut self.compile_budget)?;
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        emit!(self, Expr::New(constructor, args))
    }
}

fn check_scope<'a>(
    unit: &'a code::Unit,
    budget: &mut regexp::Budget,
    body: impl Iterator<Item = &'a StmtId> + Clone,
    block_functions: bool,
) -> Result<()> {
    let mut lexical = names::Names::default();
    for statement in body.clone() {
        budget.work(1).map_err(regexp_error)?;
        match unit.stmt(*statement) {
            Stmt::Var(bindings, kind) if *kind != DeclarationKind::Var => {
                for (name, _) in bindings {
                    lexical.push(name, false, budget)?;
                }
            }
            Stmt::Function(name, _) if block_functions => {
                lexical.push(name, true, budget)?;
            }
            _ => {}
        }
    }
    if lexical.is_empty() {
        return Ok(());
    }
    let lexical = lexical.finish(budget)?;
    if let Some(duplicate) = lexical.first_duplicate(budget)? {
        let prefix = if duplicate.block_function {
            "duplicate block binding '"
        } else {
            "duplicate lexical binding '"
        };
        return Err(names::named_error(prefix, duplicate.text, budget)?);
    }
    let mut vars = var_names(unit, budget, body.clone())?;
    if !block_functions {
        for statement in body {
            budget.work(1).map_err(regexp_error)?;
            if let Stmt::Function(name, _) = unit.stmt(*statement) {
                vars.push(name, false, budget)?;
            }
        }
    }
    let vars = vars.finish(budget)?;
    if let Some(name) = lexical.first_intersection(&vars, budget)? {
        return Err(names::named_error(
            "lexical and var declarations conflict for '",
            name,
            budget,
        )?);
    }
    Ok(())
}
fn var_names<'a>(
    unit: &'a code::Unit,
    budget: &mut regexp::Budget,
    body: impl Iterator<Item = &'a StmtId>,
) -> Result<names::Names<'a>> {
    let mut names = names::Names::default();
    let mut walk = code::StatementWalk::new(unit, body);
    while let Some(statement) = walk.next(|| budget.work(1).map_err(regexp_error))? {
        match unit.stmt(*statement) {
            Stmt::Var(bindings, DeclarationKind::Var) => {
                for (name, _) in bindings {
                    names.push(name, false, budget)?;
                }
            }
            Stmt::ForIn(ForBinding::Declaration(name, DeclarationKind::Var), _, _) => {
                names.push(name, false, budget)?;
            }
            _ => {}
        }
    }
    Ok(names)
}
fn check_parameter_lexicals<'a>(
    unit: &code::Unit,
    budget: &mut regexp::Budget,
    token: &Token,
    params: impl Iterator<Item = &'a str>,
    body: &[StmtId],
    unique: bool,
) -> Result<()> {
    let mut names = names::Names::default();
    for name in params {
        names.push(name, false, budget)?;
    }
    if names.is_empty() {
        return Ok(());
    }
    let params = names.finish(budget)?;
    if unique && params.first_duplicate(budget)?.is_some() {
        return Err(match &token.kind {
            TokenKind::Invalid(error) => (**error).clone(),
            _ => ScriptError::at("duplicate function parameter", token.offset),
        });
    }
    for statement in body {
        budget.work(1).map_err(regexp_error)?;
        if let Stmt::Var(bindings, kind) = unit.stmt(*statement)
            && *kind != DeclarationKind::Var
        {
            for (name, _) in bindings {
                if params.contains(name, budget)? {
                    return Err(ScriptError::at(
                        "parameter conflicts with lexical declaration",
                        0,
                    ));
                }
            }
        }
    }
    Ok(())
}

fn push<T>(items: &mut Vec<T>, value: T, budget: &mut regexp::Budget) -> Result<()> {
    budget.work(1).map_err(regexp_error)?;
    if items.len() >= MAX_TOKENS {
        return Err(ScriptError::resource("parser list limit exceeded"));
    }
    if items.len() == items.capacity() {
        let capacity = (items.len() + 1)
            .max(items.capacity().saturating_mul(2))
            .min(MAX_TOKENS);
        budget.work(items.len() + 1).map_err(regexp_error)?;
        compile_allocate(budget, capacity * std::mem::size_of::<T>() + 32)?;
        items
            .try_reserve_exact(capacity - items.len())
            .map_err(|_| ScriptError::resource("parser list allocation failed"))?;
    }
    items.push(value);
    Ok(())
}
fn check_declaration(
    bindings: &[(String, Option<ExprId>)],
    budget: &mut regexp::Budget,
) -> Result<()> {
    let mut names = names::Names::default();
    budget.work(1).map_err(regexp_error)?;
    for (name, _) in bindings {
        names.push(name, false, budget)?;
    }
    let names = names.finish(budget)?;
    if let Some(duplicate) = names.first_duplicate(budget)? {
        return Err(names::named_error(
            "duplicate lexical binding '",
            duplicate.text,
            budget,
        )?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRAMMAR: &[&str] = &[
        "",
        ";;;",
        "'use strict';let a=1; a;",
        "'use\\x20strict';var eval;",
        "'\\1';'use strict';",
        "'use strict'+'';var eval;",
        "function f(a,a){}",
        "function f(a,a){'use strict';}",
        "function f(a,a){} @",
        "function f(a=1){'use strict';}",
        "function eval(){'use strict';}",
        "(function arguments(){'use strict';})",
        "function f(a=()=>later,later=1,...r){let body=1;return [a,r,body];}",
        "(a=()=>b,b=/x/g,c=`${b}`,...r)=>({a,b,c,r})",
        "(a,b,a)=>a",
        "((a))=>a",
        "(a.b=1)=>a",
        "(a,)=>a",
        "(a,)",
        "(a=(b,c))=>a",
        "(...x)=>x",
        "(...x)",
        "([x])=>x",
        "({x})=>x",
        "(a=/a/,b=`${/b/.test('b')}`)=>a",
        "(a=/a/,b=`${a}`)",
        "(a=/a/)=> {return `x${a}y`;}",
        "(a=1)\n=>a",
        "async(x)=>x",
        "({a:1,'quoted':2,0:3,[a]:4,plain,m(x){return x},get v(){return 1},set v(x){}})",
        "({__proto__:null,['__proto__']:1,m(){return this},get:1,set:2})",
        "({__proto__:1,__proto__:2})",
        "({'quoted'})",
        "({0})",
        "({get v(x){}})",
        "({set v(...x){}})",
        "({m(a,a){}})",
        "({async\nx(){}})",
        "[,,a,,b,]",
        "a.b[c](1,2).d++",
        "++a[b]",
        "new new A(1).b[2](3)",
        "a?b:c=d",
        "a=b=c",
        "a**b**c",
        "a+b*c/d%2-3",
        "'a'+'b'+'c'",
        "a??b||c&&d|e^f&g",
        "a<b<=c>d>=e==f!=g===h!==i",
        "a in b instanceof c",
        "typeof a;void b;delete c.x;!d;~e;--f;g--;",
        "delete eval;",
        "this=1;",
        "label:other:for(let i=0;i<2;i++){if(i)break label;else continue other;}",
        "label:{function f(){break label;}}",
        "label:{(()=>{break label;})();}",
        "for(let a,a;;){}",
        "for(let a;;){var a;}",
        "for(let a in b){var a;}",
        "for(a.b in c){}",
        "for(var a=1 in b){}",
        "for((a,b) in c){}",
        "switch(a){case 1:;case 2:let b;break;default:;}",
        "switch(a){case 1:let b;case 2:let b;}",
        "try{throw 1}catch(e){let x}finally{while(false){break;}}",
        "try{}catch(e){let e;}",
        "try{}",
        "throw\n1;",
        "do;while(false);",
        "if(a)let b;",
        "if(a)function f(){}",
        "if(a)let\nb;",
        "let a;{var a;}",
        "function f(a){let a;}",
        "a:/unterminated",
        "`x${({a:/}/}).a}y`",
        "`\\8`",
        "({[a in b]: c})",
        "var \\u0061=1; \\u0061;",
        "var \\u0069f=1;",
        "return 1;",
        "with(a){}",
        "class A{}",
        "function* f(){}",
        "({*m(){}})",
    ];

    fn budget() -> regexp::Budget {
        regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        }
    }
    fn compare(source: &str, function: bool, strict: bool) {
        let old = parser_legacy::Parser::program_context(source, function, strict)
            .and_then(code::compile);
        let new = Parser::program_context(source, function, strict);
        match (old, new) {
            (Ok(old), Ok(new)) => assert_eq!(
                code::canonical(&new),
                code::canonical(&old),
                "{strict}/{function}: {source}"
            ),
            (Err(old), Err(new)) => assert_eq!(
                format!("{new:?}"),
                format!("{old:?}"),
                "{strict}/{function}: {source}"
            ),
            (old, new) => panic!("{strict}/{function}: {source}\nold={old:?}\nnew={new:?}"),
        }
    }
    #[test]
    fn direct_parser_matches_frozen_grammar_payloads_and_early_errors() {
        for source in GRAMMAR {
            for strict in [false, true] {
                for function in [false, true] {
                    compare(source, function, strict);
                }
            }
        }
        for line in include_str!("../../tests/fixtures/scope-names.tsv")
            .lines()
            .filter(|s| !s.starts_with('#'))
        {
            let columns: Vec<_> = line.splitn(4, '\t').collect();
            compare(columns[3], false, columns[0].ends_with("/strict"));
        }
        for line in include_str!("../../tests/fixtures/script-completion.tsv")
            .lines()
            .filter(|s| !s.starts_with('#'))
        {
            let source = line.splitn(3, '\t').nth(2).unwrap();
            for strict in [false, true] {
                compare(source, false, strict);
            }
        }
        assert_eq!(
            std::mem::size_of::<ParameterStorage>(),
            std::mem::size_of::<super::super::Parameter>()
        );
    }

    #[test]
    fn grammar_nesting_guards_still_match_the_frozen_parser() {
        for (open, close, leaf) in [
            ("{", "}", ";"),
            ("if(1)", "", ";"),
            ("(function(){", "})()", ";"),
            ("(", ")", "1"),
            ("!", "", "1"),
            ("new ", "", "A"),
        ] {
            for depth in [1, 10, 11, 16, 31, 32, 48, 94, 95, 96, 97, 128] {
                let source = format!("{}{}{};", open.repeat(depth), leaf, close.repeat(depth));
                for strict in [false, true] {
                    compare(&source, false, strict);
                }
            }
        }
    }

    #[test]
    fn cover_reinterpretation_preserves_shared_rhs_and_rescan_semantics() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("<body></body>");
        let source = r#"
            var outside=7;
            var f=(a=/a/, b=`x${outside}`, c=()=>a, ...r)=>[a,b,c(),r];
            var result=f(undefined,undefined,undefined,9,10);
            result[0]===result[2] && result[0].test('a') && result[1]==='x7' && result[3].join(',')==='9,10';
        "#;
        assert_eq!(
            runtime.execute(source, &mut document).unwrap(),
            Value::Bool(true)
        );
        let unit = Parser::program("(a=()=>3,b=(1,2),...rest)=>[a,b,rest]").unwrap();
        let weak = Rc::downgrade(&unit);
        let Stmt::Expr(id) = unit.stmt(unit.body[0]) else {
            panic!("expression");
        };
        let Expr::Function(id) = unit.expr(*id) else {
            panic!("arrow");
        };
        let function = code::FunctionRef::new(&unit, *id);
        assert_eq!(
            function
                .params
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            ["a", "b", "rest"]
        );
        assert!(function.params[0].initializer.is_some());
        assert!(function.params[2].rest);
        drop(unit);
        assert!(weak.upgrade().is_some());
        drop(function);
        assert!(weak.upgrade().is_none());
    }

    fn compile_limited(source: &str, steps: usize, heap_limit: usize) -> Result<Rc<code::Unit>> {
        let mut ledger = budget();
        ledger.steps = steps;
        ledger.heap_limit = heap_limit;
        let mut parser = Parser::start(source, false, false, ledger)?;
        parser.parse_body()?;
        parser.unit.finish(&mut parser.compile_budget)
    }
    #[test]
    fn every_work_cut_and_heap_refusal_shares_one_compile_ledger() {
        for source in [
            "let a=1;function f(x=a,...rest){return [x,rest]}f();",
            "(a=/a/,b=`x${a}`,c=()=>b)=>({a,b,c})",
            "({get x(){return 1},set x(a){},m(a=()=>3){return a()}})",
            "for(let a=0;a<2;a++){try{a;}catch(e){}finally{a;}}",
            "label:switch(0){case 0:let a;break label;default:;}",
        ] {
            let parser = Parser::parse_context(source, false, false).unwrap();
            let steps = MAX_STEPS - parser.compile_budget.steps + 1; // publication
            let mut ledger = parser.compile_budget;
            let unit = parser.unit.finish(&mut ledger).unwrap();
            let storage = ledger.allocated;
            let canonical = code::canonical(&unit);
            for cut in 0..steps {
                assert!(
                    compile_limited(source, cut, MAX_HEAP)
                        .unwrap_err()
                        .is_resource_limit(),
                    "work {cut}: {source}"
                );
            }
            assert_eq!(
                code::canonical(&compile_limited(source, steps, storage).unwrap()),
                canonical
            );
            for cut in (0..storage).step_by(127).chain([storage - 1]) {
                assert!(
                    compile_limited(source, MAX_STEPS, cut)
                        .unwrap_err()
                        .is_resource_limit(),
                    "heap {cut}: {source}"
                );
            }
        }
    }

    #[test]
    fn partial_flat_units_release_payloads_after_grammar_and_quota_errors() {
        for (source, quota) in [
            ("function f(a=/a/){return ()=>", false),
            ("(a=/a/,b=`x${1}`)=>{let duplicate;let duplicate;}", false),
            ("({get x(a){}})", false),
            ("var a=1;", true),
        ] {
            let mut parser = Parser::start(source, false, false, budget()).unwrap();
            let marker = Rc::new(
                RegExp::compile("marker".into(), &"".into(), &mut parser.compile_budget).unwrap(),
            );
            let weak = Rc::downgrade(&marker);
            parser.emit_expr(Expr::RegExp(marker)).unwrap();
            if quota {
                parser.compile_budget.steps = 0;
            }
            let error = parser.parse_body().unwrap_err();
            assert_eq!(error.is_resource_limit(), quota);
            assert!(weak.upgrade().is_some());
            drop(parser);
            assert!(weak.upgrade().is_none());
        }
        let mut parser =
            Parser::parse_context("function f(){return /retained/}", false, false).unwrap();
        parser.compile_budget.steps = 0;
        assert!(
            parser
                .unit
                .finish(&mut parser.compile_budget)
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn handler_wrapper_uses_flat_parameters_and_the_same_body_strictness() {
        let function =
            Parser::handler("'use strict';return event.type;", "onclick".into()).unwrap();
        assert!(
            function.strict && !function.constructable && !function.arrow && !function.self_name
        );
        assert_eq!(function.name.as_deref(), Some("onclick"));
        assert_eq!(function.params.len(), 1);
        assert_eq!(function.params[0].name, "event");
        assert!(function.params[0].is_simple());
        assert!(function.unit.body.is_empty());
        assert_eq!(function.body.len(), 2);
        assert!(Parser::handler("let event;", "onclick".into()).is_ok()); // Existing handler grammar policy.
        assert!(
            Parser::handler("return (", "onclick".into())
                .unwrap_err()
                .is_parse_error()
        );
    }

    #[test]
    fn unpublished_parser_records_drop_on_a_small_stack_without_syntax_recursion() {
        // Direct construction isolates destruction. Grammar recursion and its
        // depth guard remain unchanged; this is not deep-source acceptance.
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                for refuse in [false, true] {
                    let mut parser = Parser::start("", false, false, budget()).unwrap();
                    let mut expression = parser.emit_expr(Expr::Literal(Value::Undefined)).unwrap();
                    for _ in 0..16_000 {
                        let operator = parser.copy_identifier("!").unwrap();
                        expression = parser.emit_expr(Expr::Unary(operator, expression)).unwrap();
                    }
                    let mut statement = parser.emit_stmt(Stmt::Expr(expression)).unwrap();
                    for i in 0..8_000 {
                        statement = parser.emit_stmt(Stmt::Label(i, statement)).unwrap();
                    }
                    push(&mut parser.unit.body, statement, &mut parser.compile_budget).unwrap();
                    if refuse {
                        parser.compile_budget.steps = 0;
                    }
                    let result = parser.unit.finish(&mut parser.compile_budget);
                    if refuse {
                        assert!(result.unwrap_err().is_resource_limit());
                    } else {
                        drop(result.unwrap());
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
