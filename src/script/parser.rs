//! Direct flat syntax construction with bounded grammar continuations.
//! Child IDs never own syntax, including partially parsed functions/defaults.
use super::*;
mod driver;
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
    function_depth: usize,
    new_target_allowed: bool,
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
        let mut budget = regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        };
        let mut parser = Self::start(source, function, strict, &mut budget)?;
        parser.parse_body()?;
        Ok(parser)
    }
    fn start(
        source: &'source str,
        function: bool,
        strict: bool,
        budget: &mut regexp::Budget,
    ) -> Result<Self> {
        if source.len() > MAX_SOURCE {
            return Err(ScriptError::resource("script source limit exceeded"));
        }
        Ok(Self {
            tokens: lex(source, budget)?,
            source,
            lex_work: source.len(),
            compile_budget: regexp::Budget {
                steps: budget.steps,
                allocated: budget.allocated,
                heap_limit: budget.heap_limit,
                stack_limit: budget.stack_limit,
            },
            unit: code::Unit::empty(strict),
            pos: 0,
            function_depth: usize::from(function),
            new_target_allowed: function,
            loop_depth: 0,
            switch_depth: 0,
            labels: Vec::new(),
            next_label: 0,
            allow_in: true,
            strict,
        })
    }
    fn parse_body(&mut self) -> Result<()> {
        driver::parse(self)
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
    pub(super) fn dynamic_function(
        params: &'source str,
        body: &'source str,
        budget: &mut regexp::Budget,
    ) -> Result<code::FunctionRef> {
        let mut parser = Self::start(params, false, false, budget)?;
        let result = (|| {
            let mut function = driver::dynamic_function(&mut parser, body)?;
            function.name = Some(parser.copy_identifier("anonymous")?);
            let id = parser.emit_function(function)?;
            let unit = std::mem::replace(&mut parser.unit, code::Unit::empty(false))
                .finish(&mut parser.compile_budget)?;
            Ok(code::FunctionRef::new(&unit, id))
        })();
        // Failed parses consume the same work and cumulative allocation budget.
        *budget = parser.compile_budget;
        result
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
    fn object_key(&mut self) -> Result<PropertyName> {
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
                (
                    number_format::format(value, |work, bytes| {
                        self.compile_budget.work(work).map_err(regexp_error)?;
                        compile_allocate(&mut self.compile_budget, bytes)
                    })?
                    .into(),
                    false,
                )
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
    fn rest_parameter(&mut self, dynamic: bool) -> Result<Parameter> {
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
        if !(self.is(")") || dynamic && self.done()) {
            return Err(self.error("rest parameter must be last and cannot have an initializer"));
        }
        let mut parameter = self.parameter(name, None)?;
        parameter.rest = true;
        Ok(parameter)
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
            Stmt::ForIn(ForBinding::Declaration(name, DeclarationKind::Var), _, _)
            | Stmt::ForOf(ForBinding::Declaration(name, DeclarationKind::Var), _, _) => {
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
    fn prior_accepted_nesting_matches_and_obsolete_grammar_stops_are_removed() {
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
                    let old = parser_legacy::Parser::program_context(&source, false, strict);
                    if old.as_ref().is_err_and(|error| error.is_resource_limit()) {
                        Parser::program_context(&source, false, strict).unwrap();
                    } else {
                        compare(&source, false, strict);
                    }
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
        let mut parser = Parser::start(source, false, false, &mut ledger)?;
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
            let mut parser = Parser::start(source, false, false, &mut budget()).unwrap();
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
        // Direct construction isolates destruction. Source parsing is checked
        // separately by the grammar-continuation tests.
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                for refuse in [false, true] {
                    let mut parser = Parser::start("", false, false, &mut budget()).unwrap();
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
