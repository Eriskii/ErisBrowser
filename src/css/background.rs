//! Bounded single-layer background geometry and shorthand grammar.
use super::{CalculationParser, Length, PositionComponent, css_wide, gradient};
use crate::selectors::{self, Error, Kind, Token, spend};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackgroundSize {
    Explicit { width: Length, height: Length },
    Cover,
    Contain,
}
impl Default for BackgroundSize {
    fn default() -> Self {
        Self::Explicit {
            width: Length::Auto,
            height: Length::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundPosition {
    pub x: PositionComponent,
    pub y: PositionComponent,
}
impl Default for BackgroundPosition {
    fn default() -> Self {
        let start = PositionComponent {
            from_end: false,
            offset: Length::Percent(0.0),
        };
        Self { x: start, y: start }
    }
}

fn accepted<T>(value: Result<T, Error>) -> Result<Option<T>, Error> {
    match value {
        Ok(value) => Ok(Some(value)),
        Err(Error::Limit) => Err(Error::Limit),
        Err(_) => Ok(None),
    }
}

fn tokens<'s>(value: &'s str, work: &mut usize) -> Result<Vec<Token<'s>>, Error> {
    if value.len() > 4096 {
        return Err(Error::Limit);
    }
    selectors::tokens(value, work)
}

fn lexical_zero(raw: &str) -> bool {
    raw.trim_start_matches(['+', '-'])
        .split(['e', 'E'])
        .next()
        .is_some_and(|mantissa| mantissa.bytes().all(|b| b == b'0' || b == b'.'))
}

fn length(parser: &mut CalculationParser<'_, '_, '_>, size: bool) -> Result<Length, Error> {
    let token = *parser.tokens.get(parser.at).ok_or(Error::Invalid)?;
    if token.kind == Kind::Number {
        parser.take()?;
        spend(parser.work, token.raw.len().saturating_mul(2) + 4)?;
        return lexical_zero(token.raw)
            .then_some(Length::Px(0.0))
            .ok_or(Error::Invalid);
    }
    if !matches!(
        token.kind,
        Kind::Dimension | Kind::Percentage | Kind::Function
    ) {
        return Err(Error::Invalid);
    }
    // Reject negative literals before conversion can underflow them to zero.
    // Calculation ranges are handled after summation, not coefficient by coefficient.
    if size && token.kind != Kind::Function {
        spend(parser.work, token.raw.len().saturating_mul(3) + 4)?;
        let end = selectors::number_end(token.raw).ok_or(Error::Invalid)?;
        if token.raw.starts_with('-') && !lexical_zero(&token.raw[..end]) {
            return Err(Error::Invalid);
        }
    }
    let value = parser.operand(0)?;
    if !value.px.is_finite()
        || !value.percent.is_finite()
        || value.px.abs() > f64::from(f32::MAX)
        || value.percent.abs() > f64::from(f32::MAX)
    {
        return Err(Error::Invalid);
    }
    let value = match token.kind {
        Kind::Dimension => Length::Px(value.px as f32),
        Kind::Percentage => Length::Percent(value.percent as f32),
        _ => Length::Calc {
            px: value.px as f32,
            percent: value.percent as f32,
            has_percent: value.has_percent,
        },
    };
    if size {
        value.nonnegative().ok_or(Error::Invalid)
    } else {
        Ok(value)
    }
}

pub(super) fn parse_size(
    value: &str,
    units: [f32; 4],
    work: &mut usize,
) -> Result<Option<BackgroundSize>, Error> {
    let tokens = tokens(value, work)?;
    let mut parser = CalculationParser {
        tokens: &tokens,
        at: 0,
        terms: 0,
        units,
        work,
    };
    accepted((|| {
        parser.spaces()?;
        if let Some(token) = tokens.get(parser.at)
            && token.kind == Kind::Ident
        {
            spend(parser.work, token.raw.len().saturating_mul(3) + 4)?;
            let keyword = if token.is_name("cover") {
                Some(BackgroundSize::Cover)
            } else if token.is_name("contain") {
                Some(BackgroundSize::Contain)
            } else {
                None
            };
            if let Some(size) = keyword {
                parser.take()?;
                parser.spaces()?;
                return (parser.at == tokens.len())
                    .then_some(size)
                    .ok_or(Error::Invalid);
            }
        }
        let mut axes = [Length::Auto; 2];
        let mut count = 0;
        while parser.at < tokens.len() {
            if count == 2 {
                return Err(Error::Invalid);
            }
            let token = tokens[parser.at];
            axes[count] = if token.kind == Kind::Ident {
                parser.take()?;
                spend(parser.work, token.raw.len() + 2)?;
                if !token.is_name("auto") {
                    return Err(Error::Invalid);
                }
                Length::Auto
            } else {
                length(&mut parser, true)?
            };
            count += 1;
            parser.spaces()?;
        }
        spend(parser.work, 8)?;
        if count == 0 {
            return Err(Error::Invalid);
        }
        Ok(BackgroundSize::Explicit {
            width: axes[0],
            height: axes[1],
        })
    })())
}

pub(super) fn parse_position(
    value: &str,
    units: [f32; 4],
    work: &mut usize,
) -> Result<Option<BackgroundPosition>, Error> {
    use super::ObjectPositionToken::{Center, Horizontal, Offset, Vertical};
    let tokens = tokens(value, work)?;
    let mut parser = CalculationParser {
        tokens: &tokens,
        at: 0,
        terms: 0,
        units,
        work,
    };
    accepted((|| {
        let mut parts = [Center; 4];
        let mut count = 0;
        loop {
            parser.spaces()?;
            let Some(token) = tokens.get(parser.at) else {
                break;
            };
            if count == 4 {
                return Err(Error::Invalid);
            }
            parts[count] = if token.kind == Kind::Ident {
                let token = parser.take()?;
                spend(parser.work, token.raw.len().saturating_mul(6) + 8)?;
                if token.is_name("left") {
                    Horizontal(false)
                } else if token.is_name("right") {
                    Horizontal(true)
                } else if token.is_name("top") {
                    Vertical(false)
                } else if token.is_name("bottom") {
                    Vertical(true)
                } else if token.is_name("center") {
                    Center
                } else {
                    return Err(Error::Invalid);
                }
            } else {
                Offset(length(&mut parser, false)?)
            };
            count += 1;
        }
        spend(parser.work, 24)?;
        let center = PositionComponent::default();
        let edge = |end, offset| PositionComponent {
            from_end: end,
            offset,
        };
        let position =
            match &parts[..count] {
                [Vertical(end)] => BackgroundPosition {
                    x: center,
                    y: Vertical(*end).axis(false).unwrap(),
                },
                [one] => BackgroundPosition {
                    x: one.axis(true).ok_or(Error::Invalid)?,
                    y: center,
                },
                [first, second] => {
                    let swap = matches!(first, Vertical(_)) || matches!(second, Horizontal(_));
                    let (x, y) =
                        if swap && !matches!(first, Offset(_)) && !matches!(second, Offset(_)) {
                            (second.axis(true), first.axis(false))
                        } else {
                            (first.axis(true), second.axis(false))
                        };
                    BackgroundPosition {
                        x: x.ok_or(Error::Invalid)?,
                        y: y.ok_or(Error::Invalid)?,
                    }
                }
                [Horizontal(x), Offset(dx), Vertical(y)]
                | [Vertical(y), Horizontal(x), Offset(dx)] => BackgroundPosition {
                    x: edge(*x, *dx),
                    y: edge(*y, Length::Px(0.0)),
                },
                [Horizontal(x), Vertical(y), Offset(dy)]
                | [Vertical(y), Offset(dy), Horizontal(x)] => BackgroundPosition {
                    x: edge(*x, Length::Px(0.0)),
                    y: edge(*y, *dy),
                },
                [Horizontal(x), Offset(dx), Center] | [Center, Horizontal(x), Offset(dx)] => {
                    BackgroundPosition {
                        x: edge(*x, *dx),
                        y: center,
                    }
                }
                [Vertical(y), Offset(dy), Center] | [Center, Vertical(y), Offset(dy)] => {
                    BackgroundPosition {
                        x: center,
                        y: edge(*y, *dy),
                    }
                }
                [Horizontal(x), Offset(dx), Vertical(y), Offset(dy)]
                | [Vertical(y), Offset(dy), Horizontal(x), Offset(dx)] => BackgroundPosition {
                    x: edge(*x, *dx),
                    y: edge(*y, *dy),
                },
                _ => return Err(Error::Invalid),
            };
        Ok(position)
    })())
}

pub(super) fn parse_repeat(value: &str, work: &mut usize) -> Result<Option<[bool; 2]>, Error> {
    let tokens = tokens(value, work)?;
    accepted((|| {
        let mut axes = [true; 2];
        let mut count = 0;
        let mut alias = false;
        for token in tokens {
            spend(work, token.raw.len().saturating_mul(5) + 8)?;
            if token.kind == Kind::Whitespace {
                continue;
            }
            if token.kind != Kind::Ident || count == 2 || alias {
                return Err(Error::Invalid);
            }
            if token.is_name("repeat-x") || token.is_name("repeat-y") {
                if count != 0 {
                    return Err(Error::Invalid);
                }
                axes = [token.is_name("repeat-x"), token.is_name("repeat-y")];
                alias = true;
            } else {
                axes[count] = if token.is_name("repeat") {
                    true
                } else if token.is_name("no-repeat") {
                    false
                } else {
                    return Err(Error::Invalid);
                };
            }
            count += 1;
        }
        if count == 0 {
            return Err(Error::Invalid);
        }
        if count == 1 && !alias {
            axes[1] = axes[0];
        }
        Ok(axes)
    })())
}

#[derive(Clone, Copy)]
struct Component<'s> {
    first: Token<'s>,
    end: usize,
}

fn position_start(token: Token<'_>) -> bool {
    matches!(
        token.kind,
        Kind::Number | Kind::Dimension | Kind::Percentage
    ) || token.kind == Kind::Function && token.is_name("calc")
        || token.kind == Kind::Ident
            && ["left", "right", "top", "bottom", "center"]
                .iter()
                .any(|s| token.is_name(s))
}
fn size_start(token: Token<'_>) -> bool {
    matches!(
        token.kind,
        Kind::Number | Kind::Dimension | Kind::Percentage
    ) || token.kind == Kind::Function && token.is_name("calc")
        || token.kind == Kind::Ident
            && ["auto", "cover", "contain"]
                .iter()
                .any(|s| token.is_name(s))
}
fn repeat_start(token: Token<'_>) -> bool {
    token.kind == Kind::Ident
        && ["repeat", "no-repeat", "repeat-x", "repeat-y"]
            .iter()
            .any(|s| token.is_name(s))
}

/// Borrowed authored slices, in the stable shorthand target order. Defaults
/// are static strings; no computed style or gradient allocation occurs here.
pub(super) fn parts<'s>(value: &'s str, work: &mut usize) -> Result<Option<[&'s str; 5]>, Error> {
    if value.len() > 4096 {
        return Err(Error::Limit);
    }
    // Covers the bounded span pass, name classification and repeated group
    // probes. Recursive calculation/image parsers additionally debit work.
    spend(work, value.len().saturating_mul(32) + 64)?;
    let tokens = selectors::tokens(value, work)?;
    accepted((|| {
        let mut components = [None; 11];
        let (mut at, mut count) = (0, 0);
        while at < tokens.len() {
            if tokens[at].kind == Kind::Whitespace {
                at += 1;
                continue;
            }
            if count == components.len() {
                return Err(Error::Invalid);
            }
            let first = tokens[at];
            at += 1;
            if first.kind == Kind::Function {
                let mut closes = [')'; super::MAX_CALC_DEPTH + 3];
                let mut depth = 1;
                while depth != 0 {
                    let token = *tokens.get(at).ok_or(Error::Invalid)?;
                    at += 1;
                    if let Some(open) = token.open() {
                        if depth == closes.len() {
                            return Err(Error::Limit);
                        }
                        closes[depth] = match open {
                            '(' => ')',
                            '[' => ']',
                            '{' => '}',
                            _ => return Err(Error::Invalid),
                        };
                        depth += 1;
                    } else if let Kind::Close(close) = token.kind {
                        if closes[depth - 1] != close {
                            return Err(Error::Invalid);
                        }
                        depth -= 1;
                    }
                }
            }
            components[count] = Some(Component {
                first,
                end: tokens[at - 1].end,
            });
            count += 1;
        }
        let component = |index: usize| components[index].unwrap();
        let slice =
            |start: usize, end: usize| &value[component(start).first.start..component(end - 1).end];
        let mut out = ["transparent", "none", "auto auto", "0% 0%", "repeat"];
        let mut seen = [false; 5];
        let mut index = 0;
        while index < count {
            let first = component(index).first;
            let start = index;
            index += 1;
            if first.kind == Kind::Function && first.is_name("linear-gradient")
                || first.kind == Kind::Ident && first.is_name("none")
            {
                if seen[1]
                    || gradient::parse_image(slice(start, index), [16.0, 16.0, 800.0, 600.0], work)?
                        .is_none()
                {
                    return Err(Error::Invalid);
                }
                seen[1] = true;
                out[1] = slice(start, index);
            } else if repeat_start(first) {
                while index < count && repeat_start(component(index).first) {
                    index += 1;
                }
                if seen[4] || parse_repeat(slice(start, index), work)?.is_none() {
                    return Err(Error::Invalid);
                }
                seen[4] = true;
                out[4] = slice(start, index);
            } else if position_start(first) {
                while index < count && position_start(component(index).first) {
                    index += 1;
                }
                if seen[3]
                    || parse_position(slice(start, index), [16.0, 16.0, 800.0, 600.0], work)?
                        .is_none()
                {
                    return Err(Error::Invalid);
                }
                seen[3] = true;
                out[3] = slice(start, index);
                if index < count && component(index).first.kind == Kind::Delim('/') {
                    index += 1;
                    let start = index;
                    while index < count && size_start(component(index).first) {
                        index += 1;
                    }
                    if start == index
                        || parse_size(slice(start, index), [16.0, 16.0, 800.0, 600.0], work)?
                            .is_none()
                    {
                        return Err(Error::Invalid);
                    }
                    seen[2] = true;
                    out[2] = slice(start, index);
                }
            } else {
                let raw = slice(start, index);
                if seen[0] || !super::supports_color(&super::strip_comments(raw)) {
                    return Err(Error::Invalid);
                }
                seen[0] = true;
                out[0] = raw;
            }
        }
        if !seen.iter().any(|v| *v) {
            return Err(Error::Invalid);
        }
        Ok(out)
    })())
}

pub(super) fn valid(name: &str, value: &str, work: &mut usize) -> Result<bool, Error> {
    if css_wide(value) {
        return Ok(true);
    }
    let units = [16.0, 16.0, 800.0, 600.0];
    match name {
        "background" => Ok(parts(value, work)?.is_some()),
        "background-image" => gradient::valid(name, value, work),
        "background-size" => Ok(parse_size(value, units, work)?.is_some()),
        "background-position" => Ok(parse_position(value, units, work)?.is_some()),
        "background-repeat" => Ok(parse_repeat(value, work)?.is_some()),
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests;
