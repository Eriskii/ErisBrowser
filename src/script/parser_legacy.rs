//! Frozen pre-flat parser oracle, compiled only for private differential tests.
use super::*;

pub(super) struct ActiveLabel {
    pub(super) name: String,
    pub(super) target: usize,
    pub(super) iteration: bool,
}

pub(super) struct Parser<'source> {
    pub(super) tokens: tokens::Tokens,
    pub(super) source: &'source str,
    pub(super) lex_work: usize,
    pub(super) compile_budget: regexp::Budget,
    pub(super) pos: usize,
    pub(super) depth: usize,
    pub(super) function_depth: usize,
    pub(super) new_target_allowed: bool,
    pub(super) loop_depth: usize,
    pub(super) switch_depth: usize,
    pub(super) labels: Vec<ActiveLabel>,
    pub(super) next_label: usize,
    pub(super) allow_in: bool,
    pub(super) strict: bool,
}
impl<'source> Parser<'source> {
    pub(super) fn program(source: &'source str) -> Result<Program> {
        Self::program_context(source, false, false)
    }
    pub(super) fn program_context(
        source: &'source str,
        function: bool,
        strict: bool,
    ) -> Result<Program> {
        if source.len() > MAX_SOURCE {
            return Err(ScriptError::resource("script source limit exceeded"));
        }
        let mut compile_budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        let mut parser = Self {
            tokens: lex(source, &mut compile_budget)?,
            source,
            lex_work: source.len(),
            compile_budget,
            pos: 0,
            depth: 0,
            function_depth: usize::from(function),
            new_target_allowed: function,
            loop_depth: 0,
            switch_depth: 0,
            labels: Vec::new(),
            next_label: 0,
            allow_in: true,
            strict,
        };
        let (body, _, has_lexical) = parser.directive_body(false)?;
        if has_lexical {
            parser.check_scope(body.iter(), false)?;
        }
        Ok(Program {
            body,
            strict: parser.strict,
            compiled_storage: parser.compile_budget.allocated,
            remaining_work: parser.compile_budget.steps,
        })
    }
    fn directive_body(&mut self, block: bool) -> Result<(Vec<Stmt>, bool, bool)> {
        let mut own_strict = false;
        let mut has_lexical = false;
        let mut body = Vec::new();
        let mut prologue = true;
        let start = self.pos;
        while !(if block { self.eat("}") } else { self.done() }) {
            if self.done() {
                return Err(self.error("unterminated function body"));
            }
            let token = self.tokens[self.pos].clone();
            let before = self.pos;
            let statement = self.statement()?;
            let bare_string = token.string_literal
                && matches!(&statement, Stmt::Expr(Expr::Literal(Value::String(_))))
                && (self.pos == before + 1
                    || self.pos == before + 2
                        && matches!(&self.tokens[before + 1].kind, TokenKind::Symbol(s) if s == ";"));
            if prologue && bare_string {
                if token.use_strict {
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
            has_lexical |= statement.is_lexical_declaration(false);
            body.push(statement);
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
    fn assignment_target(&self, target: &Expr) -> Result<()> {
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
    fn statement(&mut self) -> Result<Stmt> {
        self.statement_context(true)
    }
    fn statement_context(&mut self, declarations: bool) -> Result<Stmt> {
        // A label retains labeled_statement and controlled_statement while its
        // body parses. Account for those two additional native frames before
        // entering statement_inner; flat aliases share this single charge.
        let weight = if self.label_start() { 3 } else { 1 };
        for _ in 0..weight {
            self.enter()?;
        }
        let result = self.statement_inner(declarations);
        self.depth -= weight;
        result
    }
    fn controlled_statement(&mut self) -> Result<Stmt> {
        let statement = self.statement_context(false)?;
        if matches!(
            statement,
            Stmt::Var(_, DeclarationKind::Let | DeclarationKind::Const)
        ) {
            return Err(self.error("lexical declarations require a statement list"));
        }
        if matches!(statement, Stmt::Function(..)) {
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
    pub(super) fn label_target(&mut self, name: &str) -> Result<Option<(usize, bool)>> {
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
    pub(super) fn labeled_statement(&mut self) -> Result<Stmt> {
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
            compile_allocate(
                &mut self.compile_budget,
                2 * std::mem::size_of::<ActiveLabel>(),
            )?;
            self.labels
                .try_reserve(1)
                .map_err(|_| ScriptError::resource("label allocation failed"))?;
            self.labels.push(ActiveLabel {
                name,
                target,
                iteration: false,
            });
        }
        let iteration = self.is("while") || self.is("do") || self.is("for");
        for label in &mut self.labels[start..] {
            label.iteration = iteration;
        }
        let body = self.controlled_statement();
        self.labels.truncate(start);
        Ok(Stmt::Label(target, Box::new(body?)))
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
        Ok(Stmt::Function(name, code))
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
                has_lexical |= statement.is_lexical_declaration(true);
                body.push(statement);
            }
            cases.push((condition, body));
        }
        self.switch_depth -= 1;
        if has_lexical {
            // The two flattened root passes can skip empty case lists without
            // yielding a statement. Charge those case visits before scanning.
            self.compile_budget
                .work(cases.len().saturating_mul(2))
                .map_err(regexp_error)?;
            self.check_scope(cases.iter().flat_map(|(_, body)| body.iter()), true)?;
        }
        Ok(Stmt::Switch(value, cases))
    }
    fn parse_if_statement(&mut self) -> Result<Stmt> {
        self.expect("(")?;
        let condition = self.sequence()?;
        self.expect(")")?;
        let yes = Box::new(self.controlled_statement()?);
        let no = if self.eat("else") {
            Some(Box::new(self.controlled_statement()?))
        } else {
            None
        };
        if self.strict
            && (matches!(&*yes, Stmt::Function(..))
                || no
                    .as_deref()
                    .is_some_and(|s| matches!(s, Stmt::Function(..))))
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
        Ok(Stmt::While(condition, Box::new(body)))
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
        Ok(Stmt::DoWhile(condition, Box::new(body)))
    }
    fn parse_for_statement(&mut self) -> Result<Stmt> {
        if self.is("await") {
            return Err(ScriptError::unsupported("for-await is not supported"));
        }
        self.expect("(")?;
        let saved_in = self.allow_in;
        self.allow_in = false;
        let forbidden = self.is("let") || (self.is("async") && self.token_is(self.pos + 1, "of"));
        let init = if self.is(";") {
            None
        } else if self.declaration_start() {
            Some(Box::new(self.declaration_context(true)?))
        } else {
            Some(Box::new(Stmt::Expr(self.sequence()?)))
        };
        self.allow_in = saved_in;
        let for_of = self.is("of");
        if self.eat("in") || self.eat("of") {
            if for_of
                && forbidden
                && !matches!(&init, Some(init) if matches!(&**init, Stmt::Var(..)))
            {
                return Err(self.error("invalid for-of assignment target"));
            }
            let binding = self.for_binding(init, for_of)?;
            let object = if for_of {
                self.expression()?
            } else {
                self.sequence()?
            };
            self.expect(")")?;
            self.loop_depth += 1;
            let body = self.controlled_statement()?;
            self.loop_depth -= 1;
            if let ForBinding::Declaration(name, kind) = &binding
                && *kind != DeclarationKind::Var
            {
                let vars = self
                    .var_names(std::iter::once(&body))?
                    .finish(&mut self.compile_budget)?;
                if vars.contains(name, &mut self.compile_budget)? {
                    return Err(self.error("for-in lexical binding conflicts with var"));
                }
            }
            let construct = if for_of { Stmt::ForOf } else { Stmt::ForIn };
            return Ok(construct(binding, object, Box::new(body)));
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
            && let Stmt::Var(bindings, kind) = &**init
            && *kind != DeclarationKind::Var
        {
            let vars = self
                .var_names(std::iter::once(&body))?
                .finish(&mut self.compile_budget)?;
            for (name, _) in bindings {
                if vars.contains(name, &mut self.compile_budget)? {
                    return Err(self.error("for lexical binding conflicts with var"));
                }
            }
            self.check_scope(std::iter::once(&**init), false)?;
        }
        Ok(Stmt::For(init, test, update, Box::new(body)))
    }
    // Finish head temporaries before descending into the body. This oracle is
    // recursive and must retain its existing native-stack bound in debug builds.
    fn for_binding(&mut self, init: Option<Box<Stmt>>, for_of: bool) -> Result<ForBinding> {
        match init.map(|init| *init) {
            Some(Stmt::Var(mut bindings, kind))
                if bindings.len() == 1 && bindings[0].1.is_none() =>
            {
                Ok(ForBinding::Declaration(bindings.remove(0).0, kind))
            }
            Some(Stmt::Expr(target @ (Expr::Ident(_) | Expr::Member(..)))) => {
                self.assignment_target(&target)?;
                Ok(ForBinding::Target(target))
            }
            Some(Stmt::Expr(Expr::Array(_) | Expr::Object(_))) if for_of => Err(
                ScriptError::unsupported("destructuring for-of targets are not supported"),
            ),
            _ => Err(self.error("invalid for-in binding")),
        }
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
        let body = Box::new(Stmt::Block(self.block()?));
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
                self.check_parameter_lexicals(std::iter::once(name.as_str()), &body, false)?;
            }
            Some(CatchClause { binding, body })
        } else {
            None
        };
        let finalizer = if self.eat("finally") {
            self.expect("{")?;
            Some(Box::new(Stmt::Block(self.block()?)))
        } else {
            None
        };
        if handler.is_none() && finalizer.is_none() {
            return Err(self.error("try requires catch or finally"));
        }
        Ok(Stmt::Try(body, handler, finalizer))
    }
    fn declaration(&mut self) -> Result<Stmt> {
        self.declaration_context(false)
    }
    fn declaration_context(&mut self, for_head: bool) -> Result<Stmt> {
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
            if kind == DeclarationKind::Const
                && value.is_none()
                && !(for_head && (self.is("in") || self.is("of")))
            {
                return Err(self.error("const declaration needs a value"));
            }
            bindings.push((name, value));
            if !self.eat(",") {
                break;
            }
        }
        Ok(Stmt::Var(bindings, kind))
    }
    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut body = Vec::new();
        let mut has_lexical = false;
        while !self.eat("}") {
            if self.done() {
                return Err(self.error("unterminated block"));
            }
            let statement = self.statement()?;
            has_lexical |= statement.is_lexical_declaration(true);
            body.push(statement);
        }
        if has_lexical {
            self.check_scope(body.iter(), true)?;
        }
        Ok(body)
    }
    fn check_scope<'a>(
        &mut self,
        body: impl Iterator<Item = &'a Stmt> + Clone,
        block_functions: bool,
    ) -> Result<()> {
        let mut lexical = names::Names::default();
        for statement in body.clone() {
            self.compile_budget.work(1).map_err(regexp_error)?;
            match statement {
                Stmt::Var(bindings, kind) if *kind != DeclarationKind::Var => {
                    for (name, _) in bindings {
                        lexical.push(name, false, &mut self.compile_budget)?;
                    }
                }
                Stmt::Function(name, _) if block_functions => {
                    lexical.push(name, true, &mut self.compile_budget)?;
                }
                _ => {}
            }
        }
        if lexical.is_empty() {
            return Ok(());
        }
        let lexical = lexical.finish(&mut self.compile_budget)?;
        if let Some(duplicate) = lexical.first_duplicate(&mut self.compile_budget)? {
            let prefix = if duplicate.block_function {
                "duplicate block binding '"
            } else {
                "duplicate lexical binding '"
            };
            return Err(names::named_error(
                prefix,
                duplicate.text,
                &mut self.compile_budget,
            )?);
        }
        let mut vars = self.var_names(body.clone())?;
        if !block_functions {
            for statement in body {
                self.compile_budget.work(1).map_err(regexp_error)?;
                if let Stmt::Function(name, _) = statement {
                    vars.push(name, false, &mut self.compile_budget)?;
                }
            }
        }
        let vars = vars.finish(&mut self.compile_budget)?;
        if let Some(name) = lexical.first_intersection(&vars, &mut self.compile_budget)? {
            return Err(names::named_error(
                "lexical and var declarations conflict for '",
                name,
                &mut self.compile_budget,
            )?);
        }
        Ok(())
    }
    fn var_names<'a>(&mut self, body: impl Iterator<Item = &'a Stmt>) -> Result<names::Names<'a>> {
        let mut names = names::Names::default();
        let mut walk = StatementWalk::new(body);
        while let Some(statement) =
            walk.next(|| self.compile_budget.work(1).map_err(regexp_error))?
        {
            match statement {
                Stmt::Var(bindings, DeclarationKind::Var) => {
                    for (name, _) in bindings {
                        names.push(name, false, &mut self.compile_budget)?;
                    }
                }
                Stmt::ForIn(ForBinding::Declaration(name, DeclarationKind::Var), _, _)
                | Stmt::ForOf(ForBinding::Declaration(name, DeclarationKind::Var), _, _) => {
                    names.push(name, false, &mut self.compile_budget)?;
                }
                _ => {}
            }
        }
        Ok(names)
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
            return Ok(PropertyName::Computed(Box::new(expression)));
        }
        if self.strict && self.tokens[self.pos].legacy_literal {
            return Err(self.error("legacy literals are forbidden in strict code"));
        }
        let (key, identifier) = match self.tokens[self.pos].kind.clone() {
            TokenKind::Word(word) => (self.identifier_key(&word.value)?, true),
            TokenKind::String(value) => (value, false),
            TokenKind::Number(value) => (
                number_format::format(value, |work, bytes| {
                    self.compile_budget.work(work).map_err(regexp_error)?;
                    compile_allocate(&mut self.compile_budget, bytes)
                })?
                .into(),
                false,
            ),
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
    fn parameter(&mut self, name: String, initializer: Option<Expr>) -> Result<Parameter> {
        self.compile_budget
            .work(1 + name.len() / 8)
            .map_err(regexp_error)?;
        self.compile_budget.allocated = self.compile_budget.allocated.saturating_add(
            2 * std::mem::size_of::<Parameter>()
                + name.len()
                + usize::from(initializer.is_some()) * (std::mem::size_of::<Expr>() + 32),
        );
        if self.compile_budget.allocated > MAX_HEAP {
            return Err(self.resource_error("parameter storage limit exceeded"));
        }
        Ok(Parameter {
            name,
            initializer: initializer.map(Rc::new),
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
        let saved_new_target = self.new_target_allowed;
        self.new_target_allowed = true;
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
                    params.push(self.rest_parameter()?);
                    self.expect(")")?;
                    break;
                }
                let name = self.binding_identifier()?;
                let initializer = if self.eat("=") {
                    Some(self.expression()?)
                } else {
                    None
                };
                params.push(self.parameter(name, initializer)?);
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
            self.check_scope(body.iter(), false)?;
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
        self.check_parameter_lexicals(
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
        self.new_target_allowed = saved_new_target;
        Ok(FunctionCode {
            params,
            body: Rc::new(body),
            name: None,
            arrow: false,
            self_name: false,
            constructable: true,
            strict,
        })
    }
    fn arrow(&mut self, params: Vec<Parameter>) -> Result<Expr> {
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
            (vec![Stmt::Return(Some(self.expression()?))], false, false)
        };
        let strict = self.strict;
        if own_strict && params.iter().any(|parameter| !parameter.is_simple()) {
            return Err(self.error("use strict directive with non-simple parameters"));
        }
        if has_lexical {
            self.check_scope(body.iter(), false)?;
        }
        self.check_parameter_lexicals(params.iter().map(|p| p.name.as_str()), &body, true)?;
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
        Ok(Expr::Function(FunctionCode {
            params,
            body: Rc::new(body),
            name: None,
            arrow: true,
            self_name: false,
            constructable: false,
            strict,
        }))
    }
    fn expression(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.assignment();
        self.depth -= 1;
        result
    }
    fn check_parameter_lexicals<'a>(
        &mut self,
        params: impl Iterator<Item = &'a str>,
        body: &[Stmt],
        unique: bool,
    ) -> Result<()> {
        let mut names = names::Names::default();
        for name in params {
            names.push(name, false, &mut self.compile_budget)?;
        }
        if names.is_empty() {
            return Ok(());
        }
        let params = names.finish(&mut self.compile_budget)?;
        if unique && params.first_duplicate(&mut self.compile_budget)?.is_some() {
            return Err(self.error("duplicate function parameter"));
        }
        for statement in body {
            self.compile_budget.work(1).map_err(regexp_error)?;
            if let Stmt::Var(bindings, kind) = statement
                && *kind != DeclarationKind::Var
            {
                for (name, _) in bindings {
                    if params.contains(name, &mut self.compile_budget)? {
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
    fn sequence(&mut self) -> Result<Expr> {
        let first = self.expression()?;
        if !self.eat(",") {
            return Ok(first);
        }
        let mut items = vec![first];
        loop {
            items.push(self.expression()?);
            if !self.eat(",") {
                break;
            }
        }
        Ok(Expr::Sequence(items))
    }
    fn assignment(&mut self) -> Result<Expr> {
        let mut left = self.binary(1)?;
        if self.eat("?") {
            let yes = self.expression()?;
            self.expect(":")?;
            let no = self.expression()?;
            left = Expr::Conditional(Box::new(left), Box::new(yes), Box::new(no));
        }
        for operator in [
            "=", "+=", "-=", "*=", "/=", "%=", "**=", "<<=", ">>=", ">>>=", "&=", "^=", "|=",
            "&&=", "||=", "??=",
        ] {
            if self.eat(operator) {
                self.assignment_target(&left)?;
                return Ok(Expr::Assign(
                    operator.to_owned(),
                    Box::new(left),
                    Box::new(self.expression()?),
                ));
            }
        }
        Ok(left)
    }
    fn binary(&mut self, min_precedence: u8) -> Result<Expr> {
        self.enter()?;
        let result = self.binary_inner(min_precedence);
        self.depth -= 1;
        result
    }
    fn binary_inner(&mut self, min_precedence: u8) -> Result<Expr> {
        let left = self.unary()?;
        let mut operations = Vec::new();
        let mut chain = 0;
        loop {
            let op = match &self.tokens[self.pos].kind {
                TokenKind::Symbol(symbol) => symbol.clone(),
                TokenKind::Word(word)
                    if !word.escaped && matches!(&*word.value, "in" | "instanceof") =>
                {
                    word.value.to_string()
                }
                _ => break,
            };
            if op == "in" && !self.allow_in {
                break;
            }
            let precedence = match op.as_str() {
                "||" | "??" => 1,
                "&&" => 2,
                "|" => 3,
                "^" => 4,
                "&" => 5,
                "==" | "!=" | "===" | "!==" => 6,
                "<" | ">" | "<=" | ">=" | "instanceof" | "in" => 7,
                "<<" | ">>" | ">>>" => 8,
                "+" | "-" => 9,
                "*" | "/" | "%" => 10,
                "**" => 11,
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
            operations.push((op, right));
        }
        // Folding an all-literal string addition chain preserves evaluation
        // order and avoids constructing every quadratic intermediate string.
        if let Expr::Literal(Value::String(first)) = &left
            && !operations.is_empty()
            && operations
                .iter()
                .all(|(op, value)| op == "+" && matches!(value, Expr::Literal(Value::String(_))))
        {
            let length = operations.iter().fold(first.len(), |length, (_, value)| {
                if let Expr::Literal(Value::String(text)) = value {
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
            self.compile_budget.allocated =
                self.compile_budget.allocated.saturating_add(length * 4);
            if self.compile_budget.allocated > MAX_HEAP {
                return Err(self.resource_error("script allocation limit exceeded"));
            }
            let mut units = Vec::with_capacity(length);
            units.extend_from_slice(first.units());
            for (_, value) in operations {
                let Expr::Literal(Value::String(text)) = value else {
                    unreachable!()
                };
                units.extend_from_slice(text.units());
            }
            return Ok(Expr::Literal(Value::String(units.into())));
        }
        Ok(if operations.is_empty() {
            left
        } else {
            Expr::BinaryChain(Box::new(left), operations)
        })
    }
    fn unary(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.unary_inner();
        self.depth -= 1;
        result
    }
    fn unary_inner(&mut self) -> Result<Expr> {
        for op in ["!", "-", "+", "~", "typeof", "void", "delete"] {
            if self.eat(op) {
                let value = self.unary()?;
                if self.strict
                    && op == "delete"
                    && matches!(&value, Expr::Ident(name) if name != "this")
                {
                    return Err(self.error("strict code cannot delete an identifier"));
                }
                return Ok(Expr::Unary(op.to_owned(), Box::new(value)));
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
            self.assignment_target(&value)?;
            return Ok(Expr::Update(Box::new(value), delta, true));
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
                value = Expr::Member(
                    Box::new(value),
                    Box::new(Expr::Literal(Value::String(property))),
                );
            } else if self.eat("[") {
                let property = self.expression()?;
                self.expect("]")?;
                value = Expr::Member(Box::new(value), Box::new(property));
            } else if self.eat("(") {
                let mut arguments = Vec::new();
                if !self.eat(")") {
                    loop {
                        arguments.push(self.expression()?);
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
                value = Expr::Call(Box::new(value), arguments);
            } else if matches!(self.tokens[self.pos].kind, TokenKind::TemplateStart) {
                return Err(ScriptError::unsupported(
                    "tagged template literals are not implemented",
                ));
            } else {
                break;
            }
        }
        if !self.tokens[self.pos].line_break_before && self.eat("++") {
            self.assignment_target(&value)?;
            value = Expr::Update(Box::new(value), 1.0, false);
        } else if !self.tokens[self.pos].line_break_before && self.eat("--") {
            self.assignment_target(&value)?;
            value = Expr::Update(Box::new(value), -1.0, false);
        }
        Ok(value)
    }
    fn primary(&mut self) -> Result<Expr> {
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
                    items.push((expression, binding_form));
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
                    if matches!(&expression, Expr::Array(_) | Expr::Object(_)) {
                        return Err(ScriptError::unsupported(
                            "destructuring parameters are not implemented",
                        ));
                    }
                    if !binding_form {
                        return Err(self.error("invalid arrow parameter"));
                    }
                    let (name, initializer) = match expression {
                        Expr::Ident(name) => (name, None),
                        Expr::Assign(operator, target, value) if operator == "=" => {
                            let Expr::Ident(name) = *target else {
                                return Err(self.error("invalid arrow parameter"));
                            };
                            (name, Some(*value))
                        }
                        _ => return Err(self.error("invalid arrow parameter")),
                    };
                    params.push(self.parameter(name, initializer)?);
                }
                if let Some(rest) = rest {
                    params.push(rest);
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
            return Ok(Expr::Sequence(
                items
                    .into_iter()
                    .map(|(expression, _)| expression)
                    .collect(),
            ));
        }
        if self.eat("[") {
            let saved = self.allow_in;
            self.allow_in = true;
            let mut items = Vec::new();
            while !self.eat("]") {
                if self.eat(",") {
                    items.push(None);
                    continue;
                }
                items.push(Some(self.expression()?));
                if self.eat("]") {
                    break;
                }
                self.expect(",")?;
            }
            self.allow_in = saved;
            return Ok(Expr::Array(items));
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
                self.compile_budget.work(1).map_err(regexp_error)?;
                self.compile_budget.allocated = self
                    .compile_budget
                    .allocated
                    .saturating_add(2 * std::mem::size_of::<(PropertyName, ObjectEntry)>());
                if self.compile_budget.allocated > MAX_HEAP {
                    return Err(self.resource_error("object literal allocation limit exceeded"));
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
                    ObjectEntry::Accessor(code, setter)
                } else if self.is("(") {
                    let mut code = self.function(true)?;
                    code.constructable = false;
                    ObjectEntry::Method(code)
                } else if self.eat(":") {
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
                    ObjectEntry::Data(Expr::Ident(name))
                };
                entries.push((key, value));
                if self.eat("}") {
                    break;
                }
                self.expect(",")?;
            }
            self.allow_in = saved;
            return Ok(Expr::Object(entries));
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
            return Ok(Expr::Function(code));
        }
        let token = self.tokens[self.pos].clone();
        if self.strict && token.legacy_literal {
            return Err(ScriptError::at(
                "legacy literals are forbidden in strict code",
                token.offset,
            ));
        }
        if !self.done() {
            self.pos += 1;
        }
        match token.kind {
            TokenKind::RegExp(pattern) => Ok(Expr::RegExp(pattern)),
            TokenKind::Invalid(error) => Err((*error).clone()),
            TokenKind::Number(n) => Ok(Expr::Literal(Value::Number(n))),
            TokenKind::String(s) => Ok(Expr::Literal(Value::String(s))),
            TokenKind::Word(s) if !s.escaped && matches!(&*s.value, "true" | "false") => {
                Ok(Expr::Literal(Value::Bool(&*s.value == "true")))
            }
            TokenKind::Word(s) if !s.escaped && &*s.value == "null" => {
                Ok(Expr::Literal(Value::Null))
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
                    self.arrow(vec![parameter])
                } else {
                    if word.escaped || s != "this" {
                        self.validate_identifier(&s, false)?;
                    }
                    Ok(Expr::Ident(s))
                }
            }
            _ => Err(ScriptError::at("expected expression", token.offset)),
        }
    }
    fn template_literal(&mut self) -> Result<Expr> {
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
            self.compile_budget.allocated = self
                .compile_budget
                .allocated
                .saturating_add(std::mem::size_of::<(Expr, JsString)>() * 2);
            if self.compile_budget.allocated > MAX_HEAP {
                return Err(self.resource_error("template storage limit exceeded"));
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
            tail.push((expression, text));
            at = end;
            interpolation = next;
            self.rescan_suffix(at)?;
        }
        Ok(if tail.is_empty() {
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
    fn regexp_literal(&mut self) -> Result<Expr> {
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
        Ok(Expr::RegExp(pattern))
    }
    fn new_expression(&mut self) -> Result<Expr> {
        self.enter()?;
        let result = self.new_expression_inner();
        self.depth -= 1;
        result
    }
    fn new_expression_inner(&mut self) -> Result<Expr> {
        if !self.eat("new") {
            return self.primary();
        }
        if self.eat(".") {
            self.expect("target")?;
            if !self.new_target_allowed {
                return Err(self.error("new.target outside a function"));
            }
            return Ok(Expr::NewTarget);
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
                constructor = Expr::Member(
                    Box::new(constructor),
                    Box::new(Expr::Literal(Value::String(key))),
                );
            } else if self.eat("[") {
                let key = self.expression()?;
                self.expect("]")?;
                constructor = Expr::Member(Box::new(constructor), Box::new(key));
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
                args.push(self.expression()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        Ok(Expr::New(Box::new(constructor), args))
    }
}
