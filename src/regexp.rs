//! Custom bounded ECMAScript non-Unicode regular expressions over UTF-16.
//! Parsing and matching share caller-owned work and cumulative heap budgets.
//! The backtracking VM uses an explicit stack; only nested lookahead assertions
//! recurse, under a separate guard supplied by the script runtime.

use crate::js_string::{JsString, is_js_whitespace};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const MAX_PATTERN: usize = 8192;
const MAX_NODES: usize = 4096;
const MAX_DEPTH: usize = 32;
const MAX_CAPTURES: usize = 128;
const MAX_REPEAT: usize = 1_000_000;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Error {
    Syntax(String),
    Unsupported(&'static str),
    Resource(&'static str),
}
type Result<T> = std::result::Result<T, Error>;

pub(crate) struct Budget {
    pub steps: usize,
    pub allocated: usize,
    pub heap_limit: usize,
    pub stack_limit: usize,
}
impl Budget {
    pub fn work(&mut self, count: usize) -> Result<()> {
        if count > self.steps {
            self.steps = 0;
            return Err(Error::Resource("regular expression work limit exceeded"));
        }
        self.steps -= count;
        Ok(())
    }
    fn allocate(&mut self, count: usize) -> Result<()> {
        self.allocated = self.allocated.saturating_add(count);
        if self.allocated > self.heap_limit {
            return Err(Error::Resource(
                "regular expression allocation limit exceeded",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Flags {
    pub indices: bool,
    pub global: bool,
    pub ignore_case: bool,
    pub multiline: bool,
    pub dot_all: bool,
    pub sticky: bool,
}
impl Flags {
    fn parse(text: &JsString) -> Result<Self> {
        let mut seen = 0u16;
        let mut result = Self::default();
        let mut unsupported = None;
        for unit in text.units() {
            let bit = match *unit {
                100 => {
                    result.indices = true;
                    1
                }
                103 => {
                    result.global = true;
                    2
                }
                105 => {
                    result.ignore_case = true;
                    4
                }
                109 => {
                    result.multiline = true;
                    8
                }
                115 => {
                    result.dot_all = true;
                    16
                }
                117 => {
                    unsupported = Some("Unicode RegExp flag u is not implemented");
                    32
                }
                118 => {
                    unsupported = Some("Unicode sets RegExp flag v is not implemented");
                    64
                }
                121 => {
                    result.sticky = true;
                    128
                }
                _ => return Err(Error::Syntax("invalid regular expression flag".into())),
            };
            if seen & bit != 0 {
                return Err(Error::Syntax("duplicate regular expression flag".into()));
            }
            seen |= bit;
        }
        if seen & 96 == 96 {
            return Err(Error::Syntax(
                "RegExp flags u and v are mutually exclusive".into(),
            ));
        }
        if let Some(message) = unsupported {
            return Err(Error::Unsupported(message));
        }
        Ok(result)
    }
    pub fn text(self) -> JsString {
        [
            (self.indices, b'd'),
            (self.global, b'g'),
            (self.ignore_case, b'i'),
            (self.multiline, b'm'),
            (self.dot_all, b's'),
            (self.sticky, b'y'),
        ]
        .into_iter()
        .filter_map(|(on, unit)| on.then_some(u16::from(unit)))
        .collect::<Vec<_>>()
        .into()
    }
}

#[derive(Clone, Debug)]
enum ClassItem {
    Range(u16, u16),
    Digit(bool),
    Space(bool),
    Word(bool),
}
#[derive(Clone, Debug)]
enum Node {
    Unit(u16),
    Dot,
    Class(Vec<ClassItem>, bool),
    Sequence(Vec<usize>),
    Alternative(Vec<usize>),
    Capture(usize, usize),
    Repeat {
        node: usize,
        min: usize,
        max: usize,
        lazy: bool,
        first: usize,
        end: usize,
    },
    Start,
    End,
    Boundary(bool),
    Backref(usize),
    NamedBackref(JsString),
    Lookahead(usize, bool),
}

#[derive(Clone, Debug)]
pub(crate) struct RegExp {
    pub source: JsString,
    pub flags: Flags,
    pub names: BTreeMap<JsString, usize>,
    nodes: Vec<Node>,
    root: usize,
    captures: usize,
    start_nodes: Option<Vec<Vec<usize>>>,
}

struct Parser<'a> {
    source: &'a [u16],
    at: usize,
    nodes: Vec<Node>,
    names: BTreeMap<JsString, usize>,
    captures: usize,
    depth: usize,
    identity_k: bool,
    budget: &'a mut Budget,
}

impl RegExp {
    pub fn compile(pattern: JsString, flags: &JsString, budget: &mut Budget) -> Result<Self> {
        budget.work(pattern.len().saturating_add(flags.len()))?;
        if pattern.len() > MAX_PATTERN {
            return Err(Error::Resource("regular expression pattern limit exceeded"));
        }
        let flags = Flags::parse(flags)?;
        budget.allocate(pattern.byte_len().saturating_mul(2) + 128)?;
        let mut parser = Parser {
            source: pattern.units(),
            at: 0,
            nodes: Vec::new(),
            names: BTreeMap::new(),
            captures: 0,
            depth: 0,
            identity_k: false,
            budget,
        };
        let root = parser.disjunction()?;
        if parser.at != parser.source.len() {
            return Err(parser.syntax("unexpected closing parenthesis"));
        }
        // NamedCaptureGroups applies to the whole pattern, including escapes
        // before the first named group and escapes inside character classes.
        if parser.identity_k && !parser.names.is_empty() {
            return Err(parser.syntax("invalid named capture escape"));
        }
        for node in &mut parser.nodes {
            match node {
                Node::Backref(index) if *index > parser.captures => {
                    return Err(Error::Unsupported(
                        "legacy decimal/octal RegExp escapes are not implemented",
                    ));
                }
                Node::NamedBackref(name) => {
                    let Some(index) = parser.names.get(name) else {
                        if parser.names.is_empty() {
                            return Err(Error::Unsupported(
                                "legacy named-reference identity escapes are not implemented",
                            ));
                        }
                        return Err(Error::Syntax("unknown named capture reference".into()));
                    };
                    *node = Node::Backref(*index);
                }
                _ => {}
            }
        }
        let mut result = Self {
            source: pattern.clone(),
            flags,
            names: parser.names,
            nodes: parser.nodes,
            root,
            captures: parser.captures,
            start_nodes: None,
        };
        result.start_nodes = result.first_nodes(root, parser.budget, 0)?;
        if result
            .start_nodes
            .as_ref()
            .is_some_and(|items| items.iter().any(Vec::is_empty))
        {
            result.start_nodes = None;
        }
        Ok(result)
    }

    pub fn find(
        &self,
        text: &[u16],
        start: usize,
        sticky: bool,
        budget: &mut Budget,
    ) -> Result<Option<Match>> {
        if start > text.len() {
            return Ok(None);
        }
        for position in start..=text.len() {
            budget.work(1)?;
            if let Some(nodes) = &self.start_nodes {
                let mut possible = false;
                for prefix in nodes {
                    let mut matched = true;
                    for (offset, id) in prefix.iter().enumerate() {
                        if let Some(unit) = text.get(position + offset)
                            && self.unit_matches(*id, *unit, budget)?
                        {
                            continue;
                        }
                        matched = false;
                        break;
                    }
                    if matched {
                        possible = true;
                        break;
                    }
                }
                if !possible {
                    if sticky {
                        break;
                    }
                    continue;
                }
            }
            budget.allocate((self.captures + 1) * 32)?;
            let state = State {
                position,
                captures: vec![None; self.captures + 1],
                tasks: vec![Task::Node(self.root)],
            };
            if let Some(mut state) = self.run(text, state, budget, 0)? {
                state.captures[0] = Some((position, state.position));
                return Ok(Some(Match {
                    captures: state.captures,
                }));
            }
            if sticky {
                break;
            }
        }
        Ok(None)
    }

    fn run(
        &self,
        text: &[u16],
        mut state: State,
        budget: &mut Budget,
        depth: usize,
    ) -> Result<Option<State>> {
        if depth > budget.stack_limit.min(16) {
            return Err(Error::Resource(
                "regular expression assertion nesting limit exceeded",
            ));
        }
        let mut alternatives = Vec::<State>::new();
        'next: loop {
            budget.work(1)?;
            let Some(task) = state.tasks.pop() else {
                return Ok(Some(state));
            };
            match task {
                Task::Node(id) => match &self.nodes[id] {
                    Node::Unit(unit) => {
                        if text
                            .get(state.position)
                            .is_some_and(|next| self.equal(*next, *unit))
                        {
                            state.position += 1;
                            continue;
                        }
                    }
                    Node::Dot => {
                        if text
                            .get(state.position)
                            .is_some_and(|unit| self.flags.dot_all || !line(*unit))
                        {
                            state.position += 1;
                            continue;
                        }
                    }
                    Node::Class(items, inverted) => {
                        if let Some(unit) = text.get(state.position) {
                            let mut matched = false;
                            for item in items {
                                budget.work(1)?;
                                if class_matches(item, *unit, self.flags.ignore_case) {
                                    matched = true;
                                    break;
                                }
                            }
                            if matched != *inverted {
                                state.position += 1;
                                continue;
                            }
                        }
                    }
                    Node::Sequence(nodes) => {
                        budget.work(nodes.len())?;
                        budget.allocate(nodes.len().saturating_mul(32))?;
                        state
                            .tasks
                            .extend(nodes.iter().rev().map(|node| Task::Node(*node)));
                        continue;
                    }
                    Node::Alternative(nodes) => {
                        for node in nodes.iter().skip(1).rev() {
                            let mut branch = state.copy(budget)?;
                            branch.push(Task::Node(*node), budget)?;
                            alternatives.push(branch);
                        }
                        state.push(Task::Node(nodes[0]), budget)?;
                        continue;
                    }
                    Node::Capture(index, node) => {
                        state.captures[*index] = Some((state.position, state.position));
                        state.push(Task::CaptureEnd(*index), budget)?;
                        state.push(Task::Node(*node), budget)?;
                        continue;
                    }
                    Node::Repeat { .. } => {
                        state.push(Task::Repeat(id, 0), budget)?;
                        continue;
                    }
                    Node::Start => {
                        if state.position == 0
                            || self.flags.multiline && line(text[state.position - 1])
                        {
                            continue;
                        }
                    }
                    Node::End => {
                        if state.position == text.len()
                            || self.flags.multiline
                                && text.get(state.position).is_some_and(|u| line(*u))
                        {
                            continue;
                        }
                    }
                    Node::Boundary(positive) => {
                        let left = state.position > 0 && word(text[state.position - 1]);
                        let right = text.get(state.position).is_some_and(|u| word(*u));
                        if (left != right) == *positive {
                            continue;
                        }
                    }
                    Node::Backref(index) => {
                        let Some((start, end)) = state.captures[*index] else {
                            continue;
                        };
                        let len = end - start;
                        budget.work(len)?;
                        if state.position + len <= text.len()
                            && text[start..end]
                                .iter()
                                .zip(&text[state.position..state.position + len])
                                .all(|(left, right)| self.equal(*left, *right))
                        {
                            state.position += len;
                            continue;
                        }
                    }
                    Node::NamedBackref(_) => unreachable!("compile resolves names"),
                    Node::Lookahead(node, negative) => {
                        let mut branch = state.copy(budget)?;
                        branch.tasks.clear();
                        branch.push(Task::Node(*node), budget)?;
                        let result = self.run(text, branch, budget, depth + 1)?;
                        match (result, negative) {
                            (Some(found), false) => {
                                state.captures = found.captures;
                                continue;
                            }
                            (None, true) => continue,
                            _ => {}
                        }
                    }
                },
                Task::CaptureEnd(index) => {
                    if let Some((start, _)) = state.captures[index] {
                        state.captures[index] = Some((start, state.position));
                    }
                    continue;
                }
                Task::Repeat(id, count) => {
                    let Node::Repeat {
                        node,
                        min,
                        max,
                        lazy,
                        first,
                        end,
                    } = self.nodes[id]
                    else {
                        unreachable!()
                    };
                    if count >= max {
                        continue;
                    }
                    let optional = count >= min;
                    if optional && !lazy {
                        alternatives.push(state.copy(budget)?);
                    }
                    budget.work(end - first)?;
                    if optional && lazy {
                        let mut repeat = state.copy(budget)?;
                        repeat.captures[first..end].fill(None);
                        repeat.push(Task::RepeatEnd(id, count, repeat.position), budget)?;
                        repeat.push(Task::Node(node), budget)?;
                        alternatives.push(repeat);
                    } else {
                        state.captures[first..end].fill(None);
                        state.push(Task::RepeatEnd(id, count, state.position), budget)?;
                        state.push(Task::Node(node), budget)?;
                    }
                    continue;
                }
                Task::RepeatEnd(id, count, previous) => {
                    let Node::Repeat { min, .. } = self.nodes[id] else {
                        unreachable!()
                    };
                    // An optional empty iteration fails so its captures cannot
                    // replace those from the previous consuming repetition.
                    if state.position != previous || count < min {
                        state.push(Task::Repeat(id, count + 1), budget)?;
                        continue;
                    }
                }
            }
            if let Some(previous) = alternatives.pop() {
                state = previous;
                continue 'next;
            }
            return Ok(None);
        }
    }

    fn equal(&self, a: u16, b: u16) -> bool {
        a == b || self.flags.ignore_case && canonical(a) == canonical(b)
    }
    // Conservative two-unit prefixes avoid constructing VM states at positions
    // that cannot start a match. Unknown or combinatorial prefixes disable this
    // optimization; it never changes the pattern's matching semantics.
    fn first_nodes(
        &self,
        id: usize,
        budget: &mut Budget,
        depth: usize,
    ) -> Result<Option<Vec<Vec<usize>>>> {
        budget.work(1)?;
        if depth >= budget.stack_limit.saturating_mul(4).min(MAX_DEPTH * 4) {
            return Err(Error::Resource(
                "regular expression prefix nesting limit exceeded",
            ));
        }
        match &self.nodes[id] {
            Node::Unit(_) | Node::Dot | Node::Class(..) => {
                budget.allocate(40)?;
                Ok(Some(vec![vec![id]]))
            }
            Node::Start | Node::End | Node::Boundary(_) | Node::Lookahead(..) => {
                Ok(Some(vec![Vec::new()]))
            }
            Node::Backref(_) | Node::NamedBackref(_) => Ok(None),
            Node::Capture(_, child) => self.first_nodes(*child, budget, depth + 1),
            Node::Repeat { node, min, max, .. } => {
                if *max == 0 {
                    return Ok(Some(vec![Vec::new()]));
                }
                let Some(first) = self.first_nodes(*node, budget, depth + 1)? else {
                    return Ok(None);
                };
                let mut all = if *min == 0 {
                    vec![Vec::new()]
                } else {
                    Vec::new()
                };
                if *min <= 1 {
                    all.extend(first.clone());
                }
                if *max >= 2 {
                    let Some(twice) = prefix_product(&first, &first, budget)? else {
                        return Ok(None);
                    };
                    all.extend(twice);
                }
                if all.len() > 64 {
                    return Ok(None);
                }
                budget.allocate(all.len() * 48)?;
                Ok(Some(all))
            }
            Node::Sequence(children) | Node::Alternative(children) => {
                let sequence = matches!(self.nodes[id], Node::Sequence(_));
                let mut all = if sequence {
                    vec![Vec::new()]
                } else {
                    Vec::new()
                };
                for child in children {
                    if sequence && all.iter().all(|prefix| prefix.len() == 2) {
                        break;
                    }
                    let Some(first) = self.first_nodes(*child, budget, depth + 1)? else {
                        return Ok(None);
                    };
                    if sequence {
                        let Some(combined) = prefix_product(&all, &first, budget)? else {
                            return Ok(None);
                        };
                        all = combined;
                    } else {
                        if all.len() + first.len() > 64 {
                            return Ok(None);
                        }
                        budget.allocate(first.len() * 48)?;
                        all.extend(first);
                    }
                }
                Ok(Some(all))
            }
        }
    }
    fn unit_matches(&self, id: usize, unit: u16, budget: &mut Budget) -> Result<bool> {
        budget.work(1)?;
        match &self.nodes[id] {
            Node::Unit(other) => Ok(self.equal(unit, *other)),
            Node::Dot => Ok(self.flags.dot_all || !line(unit)),
            Node::Class(items, inverted) => {
                for item in items {
                    budget.work(1)?;
                    if class_matches(item, unit, self.flags.ignore_case) {
                        return Ok(!inverted);
                    }
                }
                Ok(*inverted)
            }
            _ => unreachable!(),
        }
    }

    /// EscapeRegExpPattern for a non-Unicode pattern. Preserve existing escapes.
    pub fn escaped_source(&self) -> JsString {
        if self.source.is_empty() {
            return "(?:)".into();
        }
        let mut output = Vec::new();
        let mut escaped = false;
        for unit in self.source.units() {
            if escaped && line(*unit) {
                output.pop();
            }
            match *unit {
                10 => output.extend_from_slice(&[92, 110]),
                13 => output.extend_from_slice(&[92, 114]),
                0x2028 => output.extend("\\u2028".encode_utf16()),
                0x2029 => output.extend("\\u2029".encode_utf16()),
                47 if !escaped => output.extend_from_slice(&[92, 47]),
                unit => output.push(unit),
            }
            escaped = !escaped && *unit == 92;
        }
        output.into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Match {
    pub captures: Vec<Option<(usize, usize)>>,
}
impl Match {
    pub fn span(&self) -> (usize, usize) {
        self.captures[0].unwrap()
    }
}
#[derive(Clone)]
enum Task {
    Node(usize),
    CaptureEnd(usize),
    Repeat(usize, usize),
    RepeatEnd(usize, usize, usize),
}
struct State {
    position: usize,
    captures: Vec<Option<(usize, usize)>>,
    tasks: Vec<Task>,
}
impl State {
    fn copy(&self, budget: &mut Budget) -> Result<Self> {
        budget.work(self.captures.len() + self.tasks.len() + 1)?;
        budget.allocate(self.captures.len() * 32 + self.tasks.len() * 32 + 96)?;
        Ok(Self {
            position: self.position,
            captures: self.captures.clone(),
            tasks: self.tasks.clone(),
        })
    }
    fn push(&mut self, task: Task, budget: &mut Budget) -> Result<()> {
        budget.allocate(32)?;
        self.tasks.push(task);
        Ok(())
    }
}

impl Parser<'_> {
    fn syntax(&self, message: &str) -> Error {
        Error::Syntax(format!("{message} at pattern code unit {}", self.at))
    }
    fn peek(&self) -> Option<u16> {
        self.source.get(self.at).copied()
    }
    fn eat(&mut self, unit: u16) -> bool {
        if self.peek() == Some(unit) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn node(&mut self, node: Node) -> Result<usize> {
        self.budget.work(1)?;
        self.budget.allocate(128)?;
        if self.nodes.len() >= MAX_NODES {
            return Err(Error::Resource("regular expression node limit exceeded"));
        }
        let id = self.nodes.len();
        self.nodes.push(node);
        Ok(id)
    }
    fn disjunction(&mut self) -> Result<usize> {
        if self.depth >= MAX_DEPTH.min(self.budget.stack_limit) {
            return Err(Error::Resource("regular expression nesting limit exceeded"));
        }
        self.depth += 1;
        let mut alternatives = Vec::new();
        loop {
            let mut terms = Vec::new();
            while self.peek().is_some_and(|unit| !matches!(unit, 41 | 124)) {
                terms.push(self.term()?);
            }
            alternatives.push(self.node(Node::Sequence(terms))?);
            if !self.eat(124) {
                break;
            }
        }
        self.depth -= 1;
        if alternatives.len() == 1 {
            Ok(alternatives[0])
        } else {
            self.node(Node::Alternative(alternatives))
        }
    }
    fn term(&mut self) -> Result<usize> {
        self.budget.work(1)?;
        let first = self.captures + 1;
        if self.peek() == Some(123) {
            let saved = self.at;
            self.at += 1;
            if self.decimal()?.is_some() {
                if self.eat(44) {
                    self.decimal()?;
                }
                if self.eat(125) {
                    return Err(self.syntax("nothing to repeat"));
                }
            }
            self.at = saved;
        }
        let atom = match self.peek().unwrap() {
            42 | 43 | 63 => return Err(self.syntax("nothing to repeat")),
            40 => {
                self.at += 1;
                let mut capture = true;
                let mut look = None;
                let mut name = None;
                if self.eat(63) {
                    if self.eat(58) {
                        capture = false;
                    } else if self.eat(61) {
                        capture = false;
                        look = Some(false);
                    } else if self.eat(33) {
                        capture = false;
                        look = Some(true);
                    } else if self.eat(60) {
                        if matches!(self.peek(), Some(61 | 33)) {
                            return Err(Error::Unsupported(
                                "RegExp lookbehind assertions are not implemented",
                            ));
                        }
                        name = Some(self.name()?);
                    } else if matches!(self.peek(), Some(105 | 109 | 115 | 45)) {
                        return Err(Error::Unsupported(
                            "RegExp modifier groups are not implemented",
                        ));
                    } else {
                        return Err(self.syntax("invalid group prefix"));
                    }
                }
                let index = if capture {
                    self.captures += 1;
                    if self.captures > MAX_CAPTURES {
                        return Err(Error::Resource("regular expression capture limit exceeded"));
                    }
                    if let Some(name) = name
                        && self.names.insert(name, self.captures).is_some()
                    {
                        return Err(Error::Unsupported(
                            "duplicate named capture groups are not implemented",
                        ));
                    }
                    Some(self.captures)
                } else {
                    None
                };
                let body = self.disjunction()?;
                if !self.eat(41) {
                    return Err(self.syntax("unterminated capture group"));
                }
                if let Some(negative) = look {
                    self.node(Node::Lookahead(body, negative))?
                } else if let Some(index) = index {
                    self.node(Node::Capture(index, body))?
                } else {
                    body
                }
            }
            91 => self.class()?,
            46 => {
                self.at += 1;
                self.node(Node::Dot)?
            }
            94 => {
                self.at += 1;
                self.node(Node::Start)?
            }
            36 => {
                self.at += 1;
                self.node(Node::End)?
            }
            92 => {
                self.at += 1;
                self.escape(false)?
            }
            unit => {
                self.at += 1;
                self.node(Node::Unit(unit))?
            }
        };
        let quantifier_start = self.at;
        let quantifier = if self.eat(42) {
            Some((0, usize::MAX))
        } else if self.eat(43) {
            Some((1, usize::MAX))
        } else if self.eat(63) {
            Some((0, 1))
        } else if self.eat(123) {
            if let Some(min) = self.decimal()? {
                let max = if self.eat(44) {
                    self.decimal()?.unwrap_or(usize::MAX)
                } else {
                    min
                };
                if self.eat(125) {
                    Some((min, max))
                } else {
                    self.at = quantifier_start;
                    None
                }
            } else {
                self.at = quantifier_start;
                None
            }
        } else {
            None
        };
        if let Some((min, max)) = quantifier {
            if min > max {
                return Err(self.syntax("quantifier minimum exceeds maximum"));
            }
            if matches!(
                self.nodes[atom],
                Node::Start | Node::End | Node::Boundary(_)
            ) {
                return Err(self.syntax("assertion cannot be quantified"));
            }
            let lazy = self.eat(63);
            self.node(Node::Repeat {
                node: atom,
                min,
                max,
                lazy,
                first,
                end: self.captures + 1,
            })
        } else {
            Ok(atom)
        }
    }
    fn decimal(&mut self) -> Result<Option<usize>> {
        let mut value = 0usize;
        let start = self.at;
        while let Some(unit @ 48..=57) = self.peek() {
            self.at += 1;
            value = value
                .saturating_mul(10)
                .saturating_add((unit - 48) as usize);
            if value > MAX_REPEAT {
                return Err(Error::Resource(
                    "regular expression repetition limit exceeded",
                ));
            }
        }
        Ok((self.at != start).then_some(value))
    }
    fn name(&mut self) -> Result<JsString> {
        let start = self.at;
        while let Some(unit) = self.peek() {
            if unit == 62 {
                break;
            }
            if unit > 127 || unit == 92 {
                return Err(Error::Unsupported(
                    "non-ASCII RegExp capture names are not implemented",
                ));
            }
            if !(matches!(unit, 65..=90 | 97..=122 | 95 | 36)
                || self.at > start && matches!(unit, 48..=57))
            {
                return Err(self.syntax("invalid capture name"));
            }
            self.at += 1;
        }
        if self.at == start || !self.eat(62) {
            return Err(self.syntax("unterminated capture name"));
        }
        self.budget.allocate((self.at - start) * 4 + 64)?;
        Ok(self.source[start..self.at - 1].into())
    }
    fn escape(&mut self, class: bool) -> Result<usize> {
        let Some(unit) = self.peek() else {
            return Err(self.syntax("trailing escape"));
        };
        self.at += 1;
        let item = match unit {
            100 => Some(ClassItem::Digit(false)),
            68 => Some(ClassItem::Digit(true)),
            115 => Some(ClassItem::Space(false)),
            83 => Some(ClassItem::Space(true)),
            119 => Some(ClassItem::Word(false)),
            87 => Some(ClassItem::Word(true)),
            _ => None,
        };
        if let Some(item) = item {
            return self.node(Node::Class(vec![item], false));
        }
        let value = match unit {
            98 if !class => return self.node(Node::Boundary(true)),
            66 if !class => return self.node(Node::Boundary(false)),
            98 => 8,
            102 => 12,
            110 => 10,
            114 => 13,
            116 => 9,
            118 => 11,
            48 if !self.peek().is_some_and(|u| matches!(u, 48..=57)) => 0,
            48 => {
                return Err(Error::Unsupported(
                    "legacy octal RegExp escapes are not implemented",
                ));
            }
            49..=57 => {
                self.at -= 1;
                let index = self.decimal()?.unwrap();
                if class {
                    return Err(Error::Unsupported(
                        "legacy numeric RegExp class escapes are not implemented",
                    ));
                }
                return self.node(Node::Backref(index));
            }
            107 if !class && self.eat(60) => {
                let name = self.name()?;
                return self.node(Node::NamedBackref(name));
            }
            107 => {
                self.identity_k = true;
                107
            }
            99 => {
                let Some(letter @ (65..=90 | 97..=122)) = self.peek() else {
                    return Err(Error::Unsupported(
                        "legacy RegExp control escapes are not implemented",
                    ));
                };
                self.at += 1;
                letter % 32
            }
            117 | 120 => {
                let start = self.at;
                let mut value = 0u16;
                for _ in 0..if unit == 117 { 4 } else { 2 } {
                    let digit = self
                        .peek()
                        .and_then(|u| char::from_u32(u as u32))
                        .and_then(|c| c.to_digit(16));
                    let Some(digit) = digit else {
                        self.at = start;
                        return self.node(Node::Unit(unit));
                    };
                    value = value * 16 + digit as u16;
                    self.at += 1;
                }
                value
            }
            _ => unit,
        };
        self.node(Node::Unit(value))
    }
    fn class_item(&mut self) -> Result<ClassItem> {
        let Some(unit) = self.peek() else {
            return Err(self.syntax("unterminated character class"));
        };
        self.at += 1;
        if unit != 92 {
            return Ok(ClassItem::Range(unit, unit));
        }
        let node = self.escape(true)?;
        match &self.nodes[node] {
            Node::Unit(unit) => Ok(ClassItem::Range(*unit, *unit)),
            Node::Class(items, _) => Ok(items[0].clone()),
            _ => unreachable!(),
        }
    }
    fn class(&mut self) -> Result<usize> {
        self.at += 1;
        let inverted = self.eat(94);
        let mut items = Vec::new();
        while self.peek().is_some() && self.peek() != Some(93) {
            self.budget.work(1)?;
            self.budget.allocate(32)?;
            let start = self.class_item()?;
            if self.peek() == Some(45) && self.source.get(self.at + 1).is_some_and(|u| *u != 93) {
                self.at += 1;
                let end = self.class_item()?;
                match (start, end) {
                    (ClassItem::Range(a, _), ClassItem::Range(b, _)) => {
                        if a > b {
                            return Err(self.syntax("reversed character class range"));
                        }
                        items.push(ClassItem::Range(a, b));
                    }
                    _ => {
                        return Err(Error::Unsupported(
                            "legacy character-class escape ranges are not implemented",
                        ));
                    }
                }
            } else {
                items.push(start);
            }
        }
        if !self.eat(93) {
            return Err(self.syntax("unterminated character class"));
        }
        self.node(Node::Class(items, inverted))
    }
}

fn prefix_product(
    left: &[Vec<usize>],
    right: &[Vec<usize>],
    budget: &mut Budget,
) -> Result<Option<Vec<Vec<usize>>>> {
    let mut output = Vec::new();
    for a in left {
        for b in right {
            budget.work(1)?;
            if output.len() >= 64 {
                return Ok(None);
            }
            budget.allocate(48)?;
            output.push(a.iter().chain(b).take(2).copied().collect());
            if a.len() == 2 {
                break;
            }
        }
    }
    Ok(Some(output))
}
fn line(unit: u16) -> bool {
    matches!(unit, 10 | 13 | 0x2028 | 0x2029)
}
fn word(unit: u16) -> bool {
    matches!(unit, 48..=57 | 65..=90 | 97..=122 | 95)
}
fn canonical(unit: u16) -> u16 {
    let Some(ch) = char::from_u32(unit as u32) else {
        return unit;
    };
    let mut upper = ch.to_uppercase();
    let Some(first) = upper.next() else {
        return unit;
    };
    if upper.next().is_some() || first.len_utf16() != 1 || unit >= 128 && (first as u32) < 128 {
        return unit;
    }
    first as u16
}
fn class_matches(item: &ClassItem, unit: u16, ignore_case: bool) -> bool {
    match item {
        ClassItem::Digit(inverted) => matches!(unit, 48..=57) != *inverted,
        ClassItem::Space(inverted) => is_js_whitespace(unit) != *inverted,
        ClassItem::Word(inverted) => word(unit) != *inverted,
        ClassItem::Range(start, end) => {
            if (*start..=*end).contains(&unit) {
                return true;
            }
            if !ignore_case {
                return false;
            }
            let folded = canonical(unit);
            if (*start..=*end).contains(&folded) {
                return true;
            }
            // Fixed process-wide canonical equivalence table. Building this
            // scans the 65,536-code-unit alphabet once, independent of input.
            static INVERSE: OnceLock<BTreeMap<u16, Vec<u16>>> = OnceLock::new();
            INVERSE
                .get_or_init(|| {
                    let mut map = BTreeMap::<u16, Vec<u16>>::new();
                    for value in 0..=u16::MAX {
                        let folded = canonical(value);
                        if folded != value {
                            map.entry(folded).or_default().push(value);
                        }
                    }
                    map
                })
                .get(&folded)
                .is_some_and(|values| values.iter().any(|v| (*start..=*end).contains(v)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn budget() -> Budget {
        Budget {
            steps: 100_000,
            allocated: 0,
            heap_limit: 8 * 1024 * 1024,
            stack_limit: 16,
        }
    }
    fn matches(pattern: &str, flags: &str, text: &str) -> Option<Vec<Option<String>>> {
        let mut budget = budget();
        let pattern = RegExp::compile(pattern.into(), &flags.into(), &mut budget).unwrap();
        let text = JsString::from(text);
        pattern
            .find(text.units(), 0, false, &mut budget)
            .unwrap()
            .map(|found| {
                found
                    .captures
                    .into_iter()
                    .map(|span| span.map(|(a, b)| text.slice(a, b).to_string()))
                    .collect()
            })
    }
    #[test]
    fn alternatives_repeats_and_capture_clearing_follow_backtracking_order() {
        assert_eq!(
            matches("(a|ab)+?b", "", "aab"),
            Some(vec![Some("aab".into()), Some("a".into())])
        );
        assert_eq!(
            matches("(z)((a+)?(b+)?(c))*", "", "zaacbbbcac"),
            Some(vec![
                Some("zaacbbbcac".into()),
                Some("z".into()),
                Some("ac".into()),
                Some("a".into()),
                None,
                Some("c".into())
            ])
        );
        assert_eq!(matches("(a?)*", "", ""), Some(vec![Some("".into()), None]));
        assert_eq!(
            matches("(a?){2}", "", ""),
            Some(vec![Some("".into()), Some("".into())])
        );
        assert_eq!(
            matches("(a*)*", "", "aaa"),
            Some(vec![Some("aaa".into()), Some("aaa".into())])
        );
    }
    #[test]
    fn assertions_references_classes_and_utf16_are_real() {
        assert_eq!(
            matches(r"^(?<word>\w+)\s+\k<word>$", "i", "Hello HELLO").unwrap()[0],
            Some("Hello HELLO".into())
        );
        assert_eq!(
            matches(r"(?=(a+))a*b\1", "", "baaabac").unwrap()[0],
            Some("aba".into())
        );
        assert_eq!(
            matches(r"\b(?!bad)\w+\b", "", "bad good").unwrap()[0],
            Some("good".into())
        );
        assert!(matches("a$", "", "a\n").is_none());
        assert!(matches("^b$", "m", "a\nb\nc").is_some());
        assert!(matches("[^]", "", "\n").is_some());
        assert!(matches("[]", "", "x").is_none());
        assert!(matches("[a-z]", "i", "ſ").is_none());
        assert!(matches("[ς]", "i", "Σ").is_some());
        let mut budget = budget();
        let pattern = RegExp::compile(r"\uD800".into(), &"".into(), &mut budget).unwrap();
        assert!(
            pattern
                .find(&[0xd800], 0, false, &mut budget)
                .unwrap()
                .is_some()
        );
        assert_eq!(matches("..", "", "😀").unwrap()[0], Some("😀".into()));
    }
    #[test]
    fn invalid_unsupported_and_hostile_patterns_are_distinguished() {
        for source in ["(", "[", "*", "{2}", "a{3,2}", "[z-a]", "(?x)"] {
            assert!(
                matches!(
                    RegExp::compile(source.into(), &"".into(), &mut budget()),
                    Err(Error::Syntax(_))
                ),
                "{source}"
            );
        }
        for (source, flags) in [("x", "u"), ("x", "v"), ("(?<=a)b", ""), (r"\1", "")] {
            assert!(matches!(
                RegExp::compile(source.into(), &flags.into(), &mut budget()),
                Err(Error::Unsupported(_))
            ));
        }
        assert!(matches!(
            RegExp::compile("a".into(), &"gg".into(), &mut budget()),
            Err(Error::Syntax(_))
        ));
        let mut budget = budget();
        let pattern = RegExp::compile("(a+)+$".into(), &"".into(), &mut budget).unwrap();
        assert!(matches!(
            pattern.find(
                "aaaaaaaaaaaaaaaaaaaaaaaa!"
                    .encode_utf16()
                    .collect::<Vec<_>>()
                    .as_slice(),
                0,
                false,
                &mut budget
            ),
            Err(Error::Resource(_))
        ));
    }
    #[test]
    fn named_groups_disallow_identity_k_escapes_throughout_the_pattern() {
        for source in [
            r"(?<x>a)\k",
            r"(?<x>a)\kfoo",
            r"\k(?<x>a)",
            r"(?<x>a)[\k]",
            r"[\k](?<x>a)",
            r"(?<x>a)[\k<x>]",
        ] {
            assert!(
                matches!(
                    RegExp::compile(source.into(), &"".into(), &mut budget()),
                    Err(Error::Syntax(_))
                ),
                "{source}"
            );
        }
        for source in [r"\k", r"[\k]"] {
            assert_eq!(matches(source, "", "k").unwrap()[0], Some("k".into()));
        }
        assert_eq!(
            matches(r"(?<x>a)\k<x>", "", "aa").unwrap()[0],
            Some("aa".into())
        );
    }
    #[test]
    fn prefix_optimization_preserves_unoptimized_capture_results() {
        let atoms = [
            "a", "b?", "(a|b)", "(?:a|)", "[ab]", "a*", "(a)?", "(?=a)a", r"(a)\1",
        ];
        let endings = ["b", "(?:a|b)?", "a{2,3}", "$", "", "(?!b)", "(a?)*"];
        let inputs = [
            "", "a", "b", "ab", "ba", "aaa", "aab", "bbaa", "xabba", "\n",
        ];
        for atom in atoms {
            for ending in endings {
                let pattern = format!("{atom}{ending}");
                let compiled =
                    RegExp::compile(pattern.clone().into(), &"".into(), &mut budget()).unwrap();
                let mut unoptimized = compiled.clone();
                unoptimized.start_nodes = None;
                for input in inputs {
                    let text = JsString::from(input);
                    let expected = unoptimized
                        .find(text.units(), 0, false, &mut budget())
                        .unwrap();
                    let actual = compiled
                        .find(text.units(), 0, false, &mut budget())
                        .unwrap();
                    assert_eq!(actual, expected, "/{pattern}/ on {input:?}");
                }
            }
        }
    }
}
