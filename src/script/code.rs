//! Flat executable ownership. Child edges never own other syntax records.
use super::{
    DeclarationKind, JsString, MAX_HEAP, MAX_TOKENS, RegExp, Result, ScriptError, Value,
    compile_allocate, regexp, regexp_error,
};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExprId(usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct StmtId(usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FunctionId(usize);

#[derive(Debug)]
pub(super) struct Parameter {
    pub name: String,
    pub initializer: Option<ExprId>,
    pub rest: bool,
}
impl Parameter {
    fn is_simple(&self) -> bool {
        !self.rest && self.initializer.is_none()
    }
}
#[derive(Debug)]
pub(super) struct Function {
    pub params: Vec<Parameter>,
    pub body: Vec<StmtId>,
    pub name: Option<String>,
    pub arrow: bool,
    pub self_name: bool,
    pub constructable: bool,
    pub strict: bool,
}
impl Function {
    pub fn has_parameter_expressions(&self) -> bool {
        self.params.iter().any(|p| p.initializer.is_some())
    }
    pub fn has_simple_parameters(&self) -> bool {
        self.params.iter().all(Parameter::is_simple)
    }
    pub fn length(&self) -> usize {
        self.params
            .iter()
            .position(|p| !p.is_simple())
            .unwrap_or(self.params.len())
    }
    fn empty() -> Self {
        Self {
            params: Vec::new(),
            body: Vec::new(),
            name: None,
            arrow: true,
            self_name: false,
            constructable: false,
            strict: true,
        }
    }
}

#[derive(Debug)]
pub(super) struct Unit {
    expressions: Vec<Expr>,
    statements: Vec<Stmt>,
    functions: Vec<Function>,
    pub body: Vec<StmtId>,
    pub strict: bool,
    pub compiled_storage: usize,
}
impl Unit {
    pub fn expr(&self, id: ExprId) -> &Expr {
        &self.expressions[id.0]
    }
    pub fn stmt(&self, id: StmtId) -> &Stmt {
        &self.statements[id.0]
    }
    pub fn function(&self, id: FunctionId) -> &Function {
        &self.functions[id.0]
    }
    pub fn anonymous(&self, id: ExprId) -> bool {
        matches!(self.expr(id), Expr::Function(id) if self.function(*id).name.is_none())
    }
    fn empty(strict: bool) -> Self {
        Self {
            expressions: Vec::new(),
            statements: Vec::new(),
            functions: Vec::new(),
            body: Vec::new(),
            strict,
            compiled_storage: 0,
        }
    }
}
#[derive(Clone, Debug)]
pub(super) struct FunctionRef {
    pub unit: Rc<Unit>,
    id: FunctionId,
}
impl FunctionRef {
    pub fn new(unit: &Rc<Unit>, id: FunctionId) -> Self {
        Self {
            unit: unit.clone(),
            id,
        }
    }
    pub fn empty() -> Result<Self> {
        let mut unit = Unit::empty(true);
        unit.functions
            .try_reserve_exact(1)
            .map_err(|_| ScriptError::resource("empty executable allocation failed"))?;
        unit.functions.push(Function::empty());
        Ok(Self {
            unit: Rc::new(unit),
            id: FunctionId(0),
        })
    }
}
impl std::ops::Deref for FunctionRef {
    type Target = Function;
    fn deref(&self) -> &Self::Target {
        self.unit.function(self.id)
    }
}

#[derive(Debug)]
pub(super) enum Expr {
    Literal(Value),
    RegExp(Rc<RegExp>),
    Ident(String),
    Array(Vec<Option<ExprId>>),
    Object(Vec<(PropertyName, ObjectEntry)>),
    Unary(String, ExprId),
    BinaryChain(ExprId, Vec<(String, ExprId)>),
    Conditional(ExprId, ExprId, ExprId),
    Assign(String, ExprId, ExprId),
    Update(ExprId, f64, bool),
    Member(ExprId, ExprId),
    Call(ExprId, Vec<ExprId>),
    New(ExprId, Vec<ExprId>),
    Function(FunctionId),
    Sequence(Vec<ExprId>),
    Template(JsString, Vec<(ExprId, JsString)>),
}
#[derive(Debug)]
pub(super) enum PropertyName {
    Literal(JsString),
    Computed(ExprId),
}
#[derive(Debug)]
pub(super) enum ObjectEntry {
    Data(ExprId),
    Method(FunctionId),
    Accessor(FunctionId, bool),
    Prototype(ExprId),
}
#[derive(Debug)]
pub(super) enum Stmt {
    Empty,
    Label(usize, StmtId),
    Expr(ExprId),
    Var(Vec<(String, Option<ExprId>)>, DeclarationKind),
    Block(Vec<StmtId>),
    If(ExprId, StmtId, Option<StmtId>),
    While(ExprId, StmtId),
    DoWhile(ExprId, StmtId),
    For(Option<StmtId>, Option<ExprId>, Option<ExprId>, StmtId),
    ForIn(ForBinding, ExprId, StmtId),
    Switch(ExprId, Vec<(Option<ExprId>, Vec<StmtId>)>),
    Function(String, FunctionId),
    Return(Option<ExprId>),
    Throw(ExprId),
    Try(StmtId, Option<CatchClause>, Option<StmtId>),
    Break(Option<usize>),
    Continue(Option<usize>),
}
#[derive(Debug)]
pub(super) enum ForBinding {
    Declaration(String, DeclarationKind),
    Target(ExprId),
}

#[derive(Debug)]
pub(super) struct CatchClause {
    pub binding: Option<String>,
    pub body: Vec<StmtId>,
}

// Every auxiliary vector is bounded and charged before growth, including the
// lowering worklist. Only successful lowering publishes a unit and its IDs.
fn reserve<T>(items: &mut Vec<T>, needed: usize, budget: &mut regexp::Budget) -> Result<()> {
    if needed > MAX_TOKENS {
        return Err(ScriptError::resource(
            "executable record count limit exceeded",
        ));
    }
    if needed > items.capacity() {
        let capacity = needed
            .max(items.capacity().saturating_mul(2))
            .clamp(4, MAX_TOKENS);
        budget.work(items.len() + 1).map_err(regexp_error)?;
        compile_allocate(budget, capacity * std::mem::size_of::<T>() + 32)?;
        items
            .try_reserve_exact(capacity - items.len())
            .map_err(|_| ScriptError::resource("executable allocation failed"))?;
    }
    Ok(())
}
fn push<T>(items: &mut Vec<T>, value: T, budget: &mut regexp::Budget) -> Result<()> {
    budget.work(1).map_err(regexp_error)?;
    reserve(items, items.len().saturating_add(1), budget)?;
    items.push(value);
    Ok(())
}
// A source list already supplies its exact length. Reserve once and precharge
// every output slot before visiting its input; no geometric copies or per-item
// capacity checks are needed while this list is filled.
fn list<T>(length: usize, budget: &mut regexp::Budget) -> Result<Vec<T>> {
    if length > MAX_TOKENS {
        return Err(ScriptError::resource(
            "executable edge count limit exceeded",
        ));
    }
    budget.work(length).map_err(regexp_error)?;
    let mut output = Vec::new();
    if length != 0 {
        compile_allocate(budget, length * std::mem::size_of::<T>() + 32)?;
        output
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("executable edge allocation failed"))?;
    }
    Ok(output)
}
fn text(value: &str, budget: &mut regexp::Budget) -> Result<String> {
    budget.work(1 + value.len() / 8).map_err(regexp_error)?;
    compile_allocate(budget, value.len() + 32)?;
    let mut output = String::new();
    output
        .try_reserve_exact(value.len())
        .map_err(|_| ScriptError::resource("executable text allocation failed"))?;
    output.push_str(value);
    Ok(output)
}

enum Task<'a> {
    Expr(&'a super::Expr, ExprId),
    Stmt(&'a super::Stmt, StmtId),
    Function(&'a super::FunctionCode, FunctionId),
}
struct Lower<'a> {
    unit: Unit,
    tasks: Vec<Task<'a>>,
    budget: regexp::Budget,
}
impl<'a> Lower<'a> {
    fn expr(&mut self, value: &'a super::Expr) -> Result<ExprId> {
        let id = ExprId(self.unit.expressions.len());
        push(
            &mut self.unit.expressions,
            Expr::Literal(Value::Undefined),
            &mut self.budget,
        )?;
        push(&mut self.tasks, Task::Expr(value, id), &mut self.budget)?;
        Ok(id)
    }
    fn stmt(&mut self, value: &'a super::Stmt) -> Result<StmtId> {
        let id = StmtId(self.unit.statements.len());
        push(&mut self.unit.statements, Stmt::Empty, &mut self.budget)?;
        // These variants cannot call stmt() while their fields are lowered.
        // Emit them directly instead of enqueueing and revisiting a leaf.
        if matches!(
            value,
            super::Stmt::Empty
                | super::Stmt::Expr(_)
                | super::Stmt::Var(..)
                | super::Stmt::Function(..)
                | super::Stmt::Return(_)
                | super::Stmt::Throw(_)
                | super::Stmt::Break(_)
                | super::Stmt::Continue(_)
        ) {
            self.unit.statements[id.0] = self.lower_stmt(value)?;
        } else {
            push(&mut self.tasks, Task::Stmt(value, id), &mut self.budget)?;
        }
        Ok(id)
    }
    fn function(&mut self, value: &'a super::FunctionCode) -> Result<FunctionId> {
        let id = FunctionId(self.unit.functions.len());
        push(
            &mut self.unit.functions,
            Function::empty(),
            &mut self.budget,
        )?;
        push(&mut self.tasks, Task::Function(value, id), &mut self.budget)?;
        Ok(id)
    }
    fn expressions(&mut self, input: &'a [super::Expr]) -> Result<Vec<ExprId>> {
        let mut output = list(input.len(), &mut self.budget)?;
        for value in input {
            let id = self.expr(value)?;
            output.push(id);
        }
        Ok(output)
    }
    fn statements(&mut self, input: &'a [super::Stmt]) -> Result<Vec<StmtId>> {
        let needed = self.unit.statements.len().saturating_add(input.len());
        reserve(&mut self.unit.statements, needed, &mut self.budget)?;
        let mut output = list(input.len(), &mut self.budget)?;
        for value in input {
            let id = self.stmt(value)?;
            output.push(id);
        }
        Ok(output)
    }
    fn optional_expr(&mut self, input: Option<&'a super::Expr>) -> Result<Option<ExprId>> {
        input.map(|v| self.expr(v)).transpose()
    }
    fn optional_stmt(&mut self, input: Option<&'a super::Stmt>) -> Result<Option<StmtId>> {
        input.map(|v| self.stmt(v)).transpose()
    }
    fn lower_expr(&mut self, value: &'a super::Expr) -> Result<Expr> {
        use super::Expr as E;
        Ok(match value {
            E::Literal(value) => Expr::Literal(value.clone()),
            E::RegExp(value) => Expr::RegExp(value.clone()),
            E::Ident(name) => Expr::Ident(text(name, &mut self.budget)?),
            E::Array(items) => {
                let mut output = list(items.len(), &mut self.budget)?;
                for item in items {
                    let item = self.optional_expr(item.as_ref())?;
                    output.push(item);
                }
                Expr::Array(output)
            }
            E::Object(items) => {
                let mut output = list(items.len(), &mut self.budget)?;
                for (key, entry) in items {
                    let key = match key {
                        super::PropertyName::Literal(key, _) => PropertyName::Literal(key.clone()),
                        super::PropertyName::Computed(value) => {
                            PropertyName::Computed(self.expr(value)?)
                        }
                    };
                    let entry = match entry {
                        super::ObjectEntry::Data(value) => ObjectEntry::Data(self.expr(value)?),
                        super::ObjectEntry::Prototype(value) => {
                            ObjectEntry::Prototype(self.expr(value)?)
                        }
                        super::ObjectEntry::Method(value) => {
                            ObjectEntry::Method(self.function(value)?)
                        }
                        super::ObjectEntry::Accessor(value, setter) => {
                            ObjectEntry::Accessor(self.function(value)?, *setter)
                        }
                    };
                    output.push((key, entry));
                }
                Expr::Object(output)
            }
            E::Unary(op, child) => Expr::Unary(text(op, &mut self.budget)?, self.expr(child)?),
            E::BinaryChain(head, tail) => {
                let head = self.expr(head)?;
                let mut output = list(tail.len(), &mut self.budget)?;
                for (op, child) in tail {
                    let op = text(op, &mut self.budget)?;
                    let child = self.expr(child)?;
                    output.push((op, child));
                }
                Expr::BinaryChain(head, output)
            }
            E::Conditional(a, b, c) => {
                Expr::Conditional(self.expr(a)?, self.expr(b)?, self.expr(c)?)
            }
            E::Assign(op, a, b) => {
                Expr::Assign(text(op, &mut self.budget)?, self.expr(a)?, self.expr(b)?)
            }
            E::Update(child, delta, prefix) => Expr::Update(self.expr(child)?, *delta, *prefix),
            E::Member(a, b) => Expr::Member(self.expr(a)?, self.expr(b)?),
            E::Call(head, args) => Expr::Call(self.expr(head)?, self.expressions(args)?),
            E::New(head, args) => Expr::New(self.expr(head)?, self.expressions(args)?),
            E::Function(value) => Expr::Function(self.function(value)?),
            E::Sequence(values) => Expr::Sequence(self.expressions(values)?),
            E::Template(head, tail) => {
                let mut output = list(tail.len(), &mut self.budget)?;
                for (child, tail) in tail {
                    let child = self.expr(child)?;
                    output.push((child, tail.clone()));
                }
                Expr::Template(head.clone(), output)
            }
        })
    }
    fn lower_stmt(&mut self, value: &'a super::Stmt) -> Result<Stmt> {
        use super::Stmt as S;
        Ok(match value {
            S::Empty => Stmt::Empty,
            S::Label(label, body) => Stmt::Label(*label, self.stmt(body)?),
            S::Expr(value) => Stmt::Expr(self.expr(value)?),
            S::Var(bindings, kind) => {
                let mut output = list(bindings.len(), &mut self.budget)?;
                for (name, initializer) in bindings {
                    let name = text(name, &mut self.budget)?;
                    let initializer = self.optional_expr(initializer.as_ref())?;
                    output.push((name, initializer));
                }
                Stmt::Var(output, *kind)
            }
            S::Block(body) => Stmt::Block(self.statements(body)?),
            S::If(condition, yes, no) => Stmt::If(
                self.expr(condition)?,
                self.stmt(yes)?,
                self.optional_stmt(no.as_deref())?,
            ),
            S::While(condition, body) => Stmt::While(self.expr(condition)?, self.stmt(body)?),
            S::DoWhile(condition, body) => Stmt::DoWhile(self.expr(condition)?, self.stmt(body)?),
            S::For(init, condition, update, body) => Stmt::For(
                self.optional_stmt(init.as_deref())?,
                self.optional_expr(condition.as_ref())?,
                self.optional_expr(update.as_ref())?,
                self.stmt(body)?,
            ),
            S::ForIn(binding, value, body) => {
                let binding = match binding {
                    super::ForBinding::Declaration(name, kind) => {
                        ForBinding::Declaration(text(name, &mut self.budget)?, *kind)
                    }
                    super::ForBinding::Target(value) => ForBinding::Target(self.expr(value)?),
                };
                Stmt::ForIn(binding, self.expr(value)?, self.stmt(body)?)
            }
            S::Switch(value, cases) => {
                let value = self.expr(value)?;
                let mut output = list(cases.len(), &mut self.budget)?;
                for (condition, body) in cases {
                    let condition = self.optional_expr(condition.as_ref())?;
                    let body = self.statements(body)?;
                    output.push((condition, body));
                }
                Stmt::Switch(value, output)
            }
            S::Function(name, code) => {
                Stmt::Function(text(name, &mut self.budget)?, self.function(code)?)
            }
            S::Return(value) => Stmt::Return(self.optional_expr(value.as_ref())?),
            S::Throw(value) => Stmt::Throw(self.expr(value)?),
            S::Try(body, handler, finalizer) => {
                let body = self.stmt(body)?;
                let handler = handler
                    .as_ref()
                    .map(|h| -> Result<CatchClause> {
                        Ok(CatchClause {
                            binding: h
                                .binding
                                .as_deref()
                                .map(|n| text(n, &mut self.budget))
                                .transpose()?,
                            body: self.statements(&h.body)?,
                        })
                    })
                    .transpose()?;
                Stmt::Try(body, handler, self.optional_stmt(finalizer.as_deref())?)
            }
            S::Break(target) => Stmt::Break(*target),
            S::Continue(target) => Stmt::Continue(*target),
        })
    }
    fn lower_function(&mut self, code: &'a super::FunctionCode) -> Result<Function> {
        let mut params = list(code.params.len(), &mut self.budget)?;
        for p in &code.params {
            let parameter = Parameter {
                name: text(&p.name, &mut self.budget)?,
                initializer: self.optional_expr(p.initializer.as_deref())?,
                rest: p.rest,
            };
            params.push(parameter);
        }
        Ok(Function {
            params,
            body: self.statements(&code.body)?,
            name: code
                .name
                .as_deref()
                .map(|n| text(n, &mut self.budget))
                .transpose()?,
            arrow: code.arrow,
            self_name: code.self_name,
            constructable: code.constructable,
            strict: code.strict,
        })
    }
    fn finish(mut self) -> Result<Rc<Unit>> {
        while !self.tasks.is_empty() {
            self.budget.work(1).map_err(regexp_error)?;
            match self.tasks.pop().unwrap() {
                Task::Expr(value, id) => self.unit.expressions[id.0] = self.lower_expr(value)?,
                Task::Stmt(value, id) => self.unit.statements[id.0] = self.lower_stmt(value)?,
                Task::Function(value, id) => {
                    self.unit.functions[id.0] = self.lower_function(value)?
                }
            }
        }
        compile_allocate(
            &mut self.budget,
            std::mem::size_of::<Unit>() + 2 * std::mem::size_of::<usize>(),
        )?;
        self.unit.compiled_storage = self.budget.allocated;
        Ok(Rc::new(self.unit))
    }
}

pub(super) fn compile(program: super::Program) -> Result<Rc<Unit>> {
    let mut lower = Lower {
        unit: Unit::empty(program.strict),
        tasks: Vec::new(),
        budget: regexp::Budget {
            steps: program.remaining_work,
            allocated: program.compiled_storage,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        },
    };
    lower.unit.body = lower.statements(&program.body)?;
    lower.finish()
}

pub(super) fn handler(program: super::Program, name: String) -> Result<FunctionRef> {
    let mut budget = regexp::Budget {
        steps: program.remaining_work,
        allocated: program.compiled_storage,
        heap_limit: MAX_HEAP,
        stack_limit: 16,
    };
    compile_allocate(&mut budget, 256 + name.len())?;
    budget.work(2).map_err(regexp_error)?;
    let function = super::FunctionCode {
        params: vec![super::Parameter::simple("event".into())],
        body: Rc::new(program.body),
        name: Some(name),
        arrow: false,
        self_name: false,
        constructable: false,
        strict: program.strict,
    };
    let mut lower = Lower {
        unit: Unit::empty(program.strict),
        tasks: Vec::new(),
        budget,
    };
    let id = lower.function(&function)?;
    let unit = lower.finish()?;
    Ok(FunctionRef { unit, id })
}

// Borrow statement children in source order without visiting expression or
// function bodies. One cursor represents one ancestor, independent of width.
#[derive(Clone, Copy)]
enum StatementChildren<'a> {
    Empty,
    Branches {
        first: Option<&'a StmtId>,
        middle: &'a [StmtId],
        last: Option<&'a StmtId>,
    },
    Cases {
        cases: &'a [(Option<ExprId>, Vec<StmtId>)],
        body: &'a [StmtId],
    },
}

impl<'a> StatementChildren<'a> {
    fn of(statement: &'a Stmt) -> Self {
        let (first, middle, last) = match statement {
            Stmt::Block(body) => (None, body.as_slice(), None),
            Stmt::If(_, yes, no) => (Some(yes), &[][..], no.as_ref()),
            Stmt::Label(_, body) | Stmt::While(_, body) | Stmt::DoWhile(_, body) => {
                (Some(body), &[][..], None)
            }
            Stmt::For(init, _, _, body) => (init.as_ref(), &[][..], Some(body)),
            Stmt::ForIn(_, _, body) => (Some(body), &[][..], None),
            Stmt::Try(body, handler, finalizer) => (
                Some(body),
                handler.as_ref().map_or(&[][..], |handler| &handler.body),
                finalizer.as_ref(),
            ),
            Stmt::Switch(_, cases) if !cases.is_empty() => {
                return Self::Cases { cases, body: &[] };
            }
            _ => return Self::Empty,
        };
        if first.is_none() && middle.is_empty() && last.is_none() {
            Self::Empty
        } else {
            Self::Branches {
                first,
                middle,
                last,
            }
        }
    }

    fn next(&mut self, work: &mut impl FnMut() -> Result<()>) -> Result<Option<&'a StmtId>> {
        work()?;
        match self {
            Self::Empty => Ok(None),
            Self::Branches {
                first,
                middle,
                last,
            } => {
                if let Some(first) = first.take() {
                    return Ok(Some(first));
                }
                if let Some((statement, rest)) = middle.split_first() {
                    *middle = rest;
                    return Ok(Some(statement));
                }
                Ok(last.take())
            }
            Self::Cases { cases, body } => loop {
                if let Some((statement, rest)) = body.split_first() {
                    *body = rest;
                    return Ok(Some(statement));
                }
                let Some(((_, statements), rest)) = cases.split_first() else {
                    return Ok(None);
                };
                // Empty case lists still consume work before advancing.
                work()?;
                *cases = rest;
                *body = statements;
            },
        }
    }
}

pub(super) struct StatementWalk<'a, I> {
    unit: &'a Unit,
    roots: I,
    ancestors: [StatementChildren<'a>; super::MAX_DEPTH],
    depth: usize,
    done: bool,
}

impl<'a, I: Iterator<Item = &'a StmtId>> StatementWalk<'a, I> {
    pub fn new(unit: &'a Unit, roots: I) -> Self {
        Self {
            unit,
            roots,
            ancestors: [StatementChildren::Empty; super::MAX_DEPTH],
            depth: 0,
            done: false,
        }
    }

    pub fn next(&mut self, mut work: impl FnMut() -> Result<()>) -> Result<Option<&'a StmtId>> {
        if self.done {
            return Ok(None);
        }
        let result = self.advance(&mut work);
        if !matches!(result, Ok(Some(_))) {
            self.done = true;
        }
        result
    }

    fn advance(&mut self, work: &mut impl FnMut() -> Result<()>) -> Result<Option<&'a StmtId>> {
        loop {
            let statement = if self.depth == 0 {
                work()?;
                let Some(root) = self.roots.next() else {
                    return Ok(None);
                };
                root
            } else if let Some(child) = self.ancestors[self.depth - 1].next(work)? {
                child
            } else {
                self.depth -= 1;
                continue;
            };
            let children = StatementChildren::of(self.unit.stmt(*statement));
            if !matches!(children, StatementChildren::Empty) {
                if self.depth == self.ancestors.len() {
                    return Err(ScriptError::resource("statement traversal depth exceeded"));
                }
                self.ancestors[self.depth] = children;
                self.depth += 1;
            }
            return Ok(Some(statement));
        }
    }
}

#[cfg(test)]
pub(super) fn test_unit(body: &[super::Stmt]) -> Rc<Unit> {
    let mut lower = Lower {
        unit: Unit::empty(false),
        tasks: Vec::new(),
        budget: regexp::Budget {
            steps: super::MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        },
    };
    lower.unit.body = lower.statements(body).unwrap();
    lower.finish().unwrap()
}
#[cfg(test)]
pub(super) fn test_function(code: &super::FunctionCode) -> FunctionRef {
    let mut lower = Lower {
        unit: Unit::empty(code.strict),
        tasks: Vec::new(),
        budget: regexp::Budget {
            steps: super::MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        },
    };
    let id = lower.function(code).unwrap();
    FunctionRef {
        unit: lower.finish().unwrap(),
        id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::Document;
    use crate::script::{MAX_STEPS, Parser, Runtime};

    fn budget() -> regexp::Budget {
        regexp::Budget {
            steps: MAX_STEPS,
            allocated: 0,
            heap_limit: MAX_HEAP,
            stack_limit: 16,
        }
    }
    fn lower<'a>() -> Lower<'a> {
        Lower {
            unit: Unit::empty(false),
            tasks: Vec::new(),
            budget: budget(),
        }
    }

    // A test-only inverse on small, parser-bounded inputs checks every stored
    // edge and payload. Production execution never rebuilds owning syntax.
    fn expression(unit: &Unit, id: ExprId) -> super::super::Expr {
        use super::super::Expr as E;
        let child = |id| Box::new(expression(unit, id));
        let children = |ids: &[ExprId]| ids.iter().map(|&id| expression(unit, id)).collect();
        match unit.expr(id) {
            Expr::Literal(v) => E::Literal(v.clone()),
            Expr::RegExp(v) => E::RegExp(v.clone()),
            Expr::Ident(v) => E::Ident(v.clone()),
            Expr::Array(v) => E::Array(
                v.iter()
                    .map(|id| id.map(|id| expression(unit, id)))
                    .collect(),
            ),
            Expr::Object(v) => E::Object(
                v.iter()
                    .map(|(key, entry)| {
                        let key = match key {
                            PropertyName::Literal(v) => {
                                super::super::PropertyName::Literal(v.clone(), true)
                            }
                            PropertyName::Computed(id) => {
                                super::super::PropertyName::Computed(child(*id))
                            }
                        };
                        let entry = match entry {
                            ObjectEntry::Data(id) => {
                                super::super::ObjectEntry::Data(expression(unit, *id))
                            }
                            ObjectEntry::Prototype(id) => {
                                super::super::ObjectEntry::Prototype(expression(unit, *id))
                            }
                            ObjectEntry::Method(id) => {
                                super::super::ObjectEntry::Method(function(unit, *id))
                            }
                            ObjectEntry::Accessor(id, setter) => {
                                super::super::ObjectEntry::Accessor(function(unit, *id), *setter)
                            }
                        };
                        (key, entry)
                    })
                    .collect(),
            ),
            Expr::Unary(op, a) => E::Unary(op.clone(), child(*a)),
            Expr::BinaryChain(a, tail) => E::BinaryChain(
                child(*a),
                tail.iter()
                    .map(|(op, id)| (op.clone(), expression(unit, *id)))
                    .collect(),
            ),
            Expr::Conditional(a, b, c) => E::Conditional(child(*a), child(*b), child(*c)),
            Expr::Assign(op, a, b) => E::Assign(op.clone(), child(*a), child(*b)),
            Expr::Update(a, delta, prefix) => E::Update(child(*a), *delta, *prefix),
            Expr::Member(a, b) => E::Member(child(*a), child(*b)),
            Expr::Call(a, args) => E::Call(child(*a), children(args)),
            Expr::New(a, args) => E::New(child(*a), children(args)),
            Expr::Function(id) => E::Function(function(unit, *id)),
            Expr::Sequence(v) => E::Sequence(children(v)),
            Expr::Template(head, tail) => E::Template(
                head.clone(),
                tail.iter()
                    .map(|(id, s)| (expression(unit, *id), s.clone()))
                    .collect(),
            ),
        }
    }
    fn statements(unit: &Unit, ids: &[StmtId]) -> Vec<super::super::Stmt> {
        ids.iter().map(|&id| statement(unit, id)).collect()
    }
    fn statement(unit: &Unit, id: StmtId) -> super::super::Stmt {
        use super::super::Stmt as S;
        let child = |id| Box::new(statement(unit, id));
        let expr = |id| expression(unit, id);
        match unit.stmt(id) {
            Stmt::Empty => S::Empty,
            Stmt::Label(label, id) => S::Label(*label, child(*id)),
            Stmt::Expr(id) => S::Expr(expr(*id)),
            Stmt::Var(v, kind) => S::Var(
                v.iter().map(|(s, id)| (s.clone(), id.map(expr))).collect(),
                *kind,
            ),
            Stmt::Block(v) => S::Block(statements(unit, v)),
            Stmt::If(a, b, c) => S::If(expr(*a), child(*b), c.map(child)),
            Stmt::While(a, b) => S::While(expr(*a), child(*b)),
            Stmt::DoWhile(a, b) => S::DoWhile(expr(*a), child(*b)),
            Stmt::For(a, b, c, d) => S::For(a.map(child), b.map(expr), c.map(expr), child(*d)),
            Stmt::ForIn(binding, a, b) => S::ForIn(
                match binding {
                    ForBinding::Declaration(s, k) => {
                        super::super::ForBinding::Declaration(s.clone(), *k)
                    }
                    ForBinding::Target(id) => super::super::ForBinding::Target(expr(*id)),
                },
                expr(*a),
                child(*b),
            ),
            Stmt::Switch(a, cases) => S::Switch(
                expr(*a),
                cases
                    .iter()
                    .map(|(id, v)| (id.map(expr), statements(unit, v)))
                    .collect(),
            ),
            Stmt::Function(s, id) => S::Function(s.clone(), function(unit, *id)),
            Stmt::Return(id) => S::Return(id.map(expr)),
            Stmt::Throw(id) => S::Throw(expr(*id)),
            Stmt::Try(a, b, c) => S::Try(
                child(*a),
                b.as_ref().map(|h| super::super::CatchClause {
                    binding: h.binding.clone(),
                    body: statements(unit, &h.body),
                }),
                c.map(child),
            ),
            Stmt::Break(id) => S::Break(*id),
            Stmt::Continue(id) => S::Continue(*id),
        }
    }
    fn function(unit: &Unit, id: FunctionId) -> super::super::FunctionCode {
        let f = unit.function(id);
        super::super::FunctionCode {
            params: f
                .params
                .iter()
                .map(|p| super::super::Parameter {
                    name: p.name.clone(),
                    initializer: p.initializer.map(|id| Rc::new(expression(unit, id))),
                    rest: p.rest,
                })
                .collect(),
            body: Rc::new(statements(unit, &f.body)),
            name: f.name.clone(),
            arrow: f.arrow,
            self_name: f.self_name,
            constructable: f.constructable,
            strict: f.strict,
        }
    }

    #[test]
    fn every_syntax_edge_roundtrips_without_retaining_the_parser_tree() {
        let source = r#"
            function outer(a=()=>3,...rest) {
                var list=[,a,rest], regex=/a/g, seq=(a++,a?a:rest.length);
                const o={plain:1,m(x){return x;},get value(){return a;},set value(x){a=x;},__proto__:null,[a]:a};
                var text=`x${!a}:${seq}`, surrogate='\ud800';
                label:{if(a)break label;else ;}
                while(a){break;}do{continue;}while(false);
                for(let i=0;i<1;i++){if(i)continue;else break;}
                for(var k in o){o[k]=k;}for(o.value in o){}
                switch(a){case 1:a+=2;break;default:;}
                try{throw new Error('x');}catch(e){a=2;}finally{a=3;}
                return function named(b=a){return ()=>[regex,seq,text,b,outer()];};
            }
        "#;
        let tree = Parser::program(source).unwrap();
        let expected = format!("{:?}", tree.body);
        let unit = compile(tree).unwrap();
        assert_eq!(format!("{:?}", statements(&unit, &unit.body)), expected);
        assert!(unit.functions.len() >= 7);
        // One unit owns all nested functions, including default initializers.
        let weak = Rc::downgrade(&unit);
        let Stmt::Function(_, id) = unit.stmt(unit.body[0]) else {
            panic!("outer function");
        };
        let handle = FunctionRef::new(&unit, *id);
        let copy = handle.clone();
        assert!(Rc::ptr_eq(&handle.unit, &copy.unit));
        drop(unit);
        drop(handle);
        assert!(weak.upgrade().is_some());
        drop(copy);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn flat_scope_walk_preserves_order_boundaries_work_and_terminal_depth_errors() {
        let tree=Parser::program("var a;{var b;if(1)var c;else var d;}switch(0){case 0:case 1:var e;default:;}try{var f;}catch(e){var g;}finally{var h;}function hidden(){var excluded;}").unwrap();
        let unit = test_unit(&tree.body);
        let mut old = super::super::StatementWalk::new(tree.body.iter());
        let mut new = StatementWalk::new(&unit, unit.body.iter());
        let (mut old_work, mut new_work) = (0, 0);
        loop {
            let a = old
                .next(|| {
                    old_work += 1;
                    Ok(())
                })
                .unwrap();
            let b = new
                .next(|| {
                    new_work += 1;
                    Ok(())
                })
                .unwrap();
            assert_eq!(
                a.map(|s| format!("{s:?}")),
                b.map(|&id| format!("{:?}", statement(&unit, id)))
            );
            if a.is_none() {
                break;
            }
        }
        assert_eq!(old_work, new_work);
        let mut unit = Unit::empty(false);
        unit.statements.push(Stmt::Empty);
        for i in 0..97 {
            unit.statements.push(Stmt::Label(i, StmtId(i)));
        }
        let roots = [StmtId(96)];
        let mut walk = StatementWalk::new(&unit, roots.iter());
        let mut count = 0;
        while walk.next(|| Ok(())).unwrap().is_some() {
            count += 1;
        }
        assert_eq!(count, 97);
        let roots = [StmtId(97)];
        let mut walk = StatementWalk::new(&unit, roots.iter());
        for _ in 0..96 {
            assert!(walk.next(|| Ok(())).unwrap().is_some());
        }
        assert!(walk.next(|| Ok(())).unwrap_err().is_resource_limit());
        assert!(
            walk.next(|| panic!("terminal walker cannot advance"))
                .unwrap()
                .is_none()
        );
        let mut walk = StatementWalk::new(&unit, roots.iter());
        assert!(
            walk.next(|| Err(ScriptError::resource("no work")))
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            walk.next(|| panic!("terminal work failure"))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn flat_storage_and_copy_work_fail_before_allocation_or_content_scan() {
        let mut ledger = budget();
        ledger.steps = 0;
        let mut records = Vec::<usize>::new();
        assert!(
            push(&mut records, 1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(
            (records.len(), records.capacity(), ledger.allocated),
            (0, 0, 0)
        );
        let mut ledger = budget();
        ledger.heap_limit = 4 * std::mem::size_of::<usize>() + 31;
        assert!(
            push(&mut records, 1, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!((records.len(), records.capacity()), (0, 0));
        let mut ledger = budget();
        ledger.steps = 2;
        assert!(
            list::<usize>(3, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(ledger.allocated, 0);
        let mut ledger = budget();
        ledger.heap_limit = 3 * std::mem::size_of::<usize>() + 31;
        assert!(
            list::<usize>(3, &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(ledger.allocated, ledger.heap_limit + 1);
        let mut ledger = budget();
        ledger.steps = 128;
        assert!(
            text(&"a".repeat(1024), &mut ledger)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(ledger.allocated, 0);
        let mut ledger = budget();
        ledger.steps = 0;
        assert!(list::<usize>(0, &mut ledger).unwrap().is_empty());
        assert_eq!(ledger.allocated, 0);
        assert!(
            list::<usize>(MAX_TOKENS + 1, &mut budget())
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            reserve(&mut records, MAX_TOKENS + 1, &mut budget())
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn lowering_shares_the_parse_ledger_and_never_publishes_partial_code() {
        let mut program = Parser::program("function f(a=1){return a+1;} f();").unwrap();
        program.remaining_work = 0;
        assert!(compile(program).unwrap_err().is_resource_limit());
        let mut program = Parser::program("var a=1;").unwrap();
        program.compiled_storage = MAX_HEAP;
        assert!(compile(program).unwrap_err().is_resource_limit());
        let mut builder = lower();
        let input = super::super::Expr::Ident(String::new());
        // Node reservation succeeds; refusing the worklist cannot publish a unit.
        builder.budget.heap_limit = 4 * std::mem::size_of::<Expr>() + 32;
        assert!(builder.expr(&input).unwrap_err().is_resource_limit());
        assert_eq!(builder.unit.expressions.len(), 1);
        assert!(builder.tasks.is_empty());
        let program = Parser::program("var a=1;").unwrap();
        let prior = program.compiled_storage;
        assert!(compile(program).unwrap().compiled_storage > prior);
    }

    #[test]
    fn flat_units_release_deep_edges_on_a_small_native_stack() {
        // These records are constructed directly, not accepted deep source or
        // deep execution. This isolates the ownership/destruction property.
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let mut unit = Unit::empty(false);
                unit.expressions.push(Expr::Literal(Value::Undefined));
                for i in 1..24_000 {
                    unit.expressions
                        .push(Expr::Unary("!".into(), ExprId(i - 1)));
                }
                for i in 0..8_000 {
                    let expression = ExprId(unit.expressions.len());
                    unit.expressions.push(if i == 0 {
                        Expr::Literal(Value::Undefined)
                    } else {
                        Expr::Function(FunctionId(i - 1))
                    });
                    unit.statements.push(Stmt::Return(Some(expression)));
                    let mut function = Function::empty();
                    function.body.push(StmtId(i));
                    unit.functions.push(function);
                }
                let unit = Rc::new(unit);
                let weak = Rc::downgrade(&unit);
                let handle = FunctionRef::new(&unit, FunctionId(7_999));
                drop(unit);
                assert!(weak.upgrade().is_some());
                drop(handle);
                assert!(weak.upgrade().is_none());
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn closures_and_callbacks_keep_their_own_units_across_script_entries() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime
            .execute(
                "function make(x){return function(a=()=>x){return a();};}var old=make(7);",
                &mut document,
            )
            .unwrap();
        let Value::Function(id) = runtime.lookup(0, "old").unwrap().1 else {
            panic!("closure");
        };
        let weak = Rc::downgrade(&runtime.functions[id].code.unit);
        runtime
            .execute(
                "function newer(){return old()+5;}var host={valueOf(){return newer();}};",
                &mut document,
            )
            .unwrap();
        assert_eq!(
            runtime
                .execute("var answer=host+1;old()+':'+answer;", &mut document)
                .unwrap()
                .to_string(),
            "7:13"
        );
        runtime.execute("var marker={};function abrupt(){throw marker;}try{abrupt();}catch(e){if(e!==marker)throw 1;}",&mut document).unwrap();
        assert_eq!(
            (runtime.calls, runtime.eval_depth, runtime.stack_units),
            (0, 0, 0)
        );
        assert!(weak.upgrade().is_some());
        drop(runtime);
        assert!(weak.upgrade().is_none());
    }
}
