//! One bounded legacy-sRGB linear-gradient layer. Parsing keeps stop storage on
//! the stack; only the per-computation pool may retain a computed image.
use super::{CalculationParser, css_wide, parse_color, supports_color};
use crate::graphics::gradient::{
    GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient,
    MAX_GRADIENT_STOPS,
};
use crate::selectors::{self, Error, Kind, spend};
use std::collections::BTreeMap;
use std::sync::Arc;

const MAX_AUTHORED_STOPS: usize = 16;
const EMPTY_STOP: GradientStop = GradientStop {
    color: GradientColor::CurrentColor,
    position: None,
};
pub(super) const MAX_RETAINED_GRADIENT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug)]
pub(super) struct ParsedGradient {
    direction: GradientDirection,
    stops: [GradientStop; MAX_GRADIENT_STOPS],
    len: usize,
}
impl ParsedGradient {
    fn stops(&self) -> &[GradientStop] {
        &self.stops[..self.len]
    }
    fn push(
        &mut self,
        color: GradientColor,
        position: Option<GradientPosition>,
    ) -> Result<(), Error> {
        if self.len == self.stops.len() {
            return Err(Error::Invalid);
        }
        self.stops[self.len] = GradientStop { color, position };
        self.len += 1;
        Ok(())
    }
}
// Deliberately stack-backed until pool admission; boxing would allocate before
// the retained-storage check for every validation-only parse.
#[allow(clippy::large_enum_variant)]
pub(super) enum ParsedImage {
    None,
    Gradient(ParsedGradient),
}

struct Parser<'a, 's, 'w> {
    source: &'s str,
    cursor: CalculationParser<'a, 's, 'w>,
}
impl Parser<'_, '_, '_> {
    fn kind(&self) -> Option<Kind> {
        self.cursor.tokens.get(self.cursor.at).map(|t| t.kind)
    }
    fn name(&self, name: &str) -> bool {
        self.cursor
            .tokens
            .get(self.cursor.at)
            .is_some_and(|t| t.kind == Kind::Ident && t.is_name(name))
    }
    fn zero(&mut self) -> Result<(), Error> {
        let token = self.cursor.take()?;
        // Lexical zero excludes underflowed nonzero numbers. This scan and the
        // bounded identifier/color scans are prepaid by parse_image's envelope.
        let mantissa = token
            .raw
            .trim_start_matches(['+', '-'])
            .split(['e', 'E'])
            .next()
            .ok_or(Error::Invalid)?;
        if token.kind != Kind::Number || !mantissa.bytes().all(|b| b == b'0' || b == b'.') {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    fn angle(&mut self) -> Result<GradientDirection, Error> {
        if self.kind() == Some(Kind::Number) {
            self.zero()?;
            return Ok(GradientDirection::Angle(0.0));
        }
        let token = self.cursor.take()?;
        if token.kind != Kind::Dimension {
            return Err(Error::Invalid);
        }
        let end = selectors::number_end(token.raw).ok_or(Error::Invalid)?;
        let value: f64 = token.raw[..end].parse().map_err(|_| Error::Invalid)?;
        let (unit, consumed) = super::css_identifier(&token.raw[end..]).ok_or(Error::Invalid)?;
        if consumed != token.raw.len() - end || !value.is_finite() {
            return Err(Error::Invalid);
        }
        let degrees = match unit.to_ascii_lowercase().as_str() {
            "deg" => value,
            "grad" => value * 0.9,
            "rad" => value * (180.0 / std::f64::consts::PI),
            "turn" => value * 360.0,
            _ => return Err(Error::Invalid),
        };
        if !degrees.is_finite() {
            return Err(Error::Invalid);
        }
        Ok(GradientDirection::Angle(degrees.rem_euclid(360.0)))
    }
    fn direction(&mut self) -> Result<GradientDirection, Error> {
        if !self.name("to") {
            return self.angle();
        }
        self.cursor.take()?;
        self.cursor.spaces()?;
        let (mut horizontal, mut vertical) = (None, None);
        for _ in 0..2 {
            let side = if self.name("left") {
                Some((true, false))
            } else if self.name("right") {
                Some((true, true))
            } else if self.name("top") {
                Some((false, false))
            } else if self.name("bottom") {
                Some((false, true))
            } else {
                None
            };
            let Some((x, end)) = side else {
                break;
            };
            let axis = if x { &mut horizontal } else { &mut vertical };
            if axis.replace(end).is_some() {
                return Err(Error::Invalid);
            }
            self.cursor.take()?;
            self.cursor.spaces()?;
        }
        match (horizontal, vertical) {
            (Some(right), Some(bottom)) => Ok(GradientDirection::Corner { right, bottom }),
            (Some(right), None) => Ok(GradientDirection::Angle(if right { 90.0 } else { 270.0 })),
            (None, Some(bottom)) => Ok(GradientDirection::Angle(if bottom { 180.0 } else { 0.0 })),
            _ => Err(Error::Invalid),
        }
    }
    fn prelude(&mut self) -> Result<GradientDirection, Error> {
        let mut direction = None;
        let mut interpolation = false;
        for _ in 0..2 {
            if self.name("in") {
                if interpolation {
                    return Err(Error::Invalid);
                }
                self.cursor.take()?;
                self.cursor.spaces()?;
                if !self.name("srgb") {
                    return Err(Error::Invalid);
                }
                self.cursor.take()?;
                interpolation = true;
            } else {
                if direction.is_some() {
                    return Err(Error::Invalid);
                }
                direction = Some(self.direction()?);
            }
            self.cursor.spaces()?;
            if self.kind() == Some(Kind::Comma) {
                break;
            }
        }
        if self.cursor.take()?.kind != Kind::Comma {
            return Err(Error::Invalid);
        }
        self.cursor.spaces()?;
        Ok(direction.unwrap_or(GradientDirection::Angle(180.0)))
    }
    fn color(&mut self) -> Result<GradientColor, Error> {
        let first = self.cursor.take()?;
        if first.kind == Kind::Ident && first.is_name("currentcolor") {
            return Ok(GradientColor::CurrentColor);
        }
        let mut end = first.end;
        if first.kind == Kind::Function {
            if !(first.is_name("rgb")
                || first.is_name("rgba")
                || first.is_name("hsl")
                || first.is_name("hsla"))
            {
                return Err(Error::Invalid);
            }
            loop {
                let token = self.cursor.take()?;
                // Current strict legacy colors have no nested functions/blocks.
                if token.open().is_some() {
                    return Err(Error::Invalid);
                }
                if token.kind == Kind::Close(')') {
                    end = token.end;
                    break;
                }
                if matches!(token.kind, Kind::Close(_)) {
                    return Err(Error::Invalid);
                }
            }
        } else if !matches!(first.kind, Kind::Ident | Kind::Hash(_)) {
            return Err(Error::Invalid);
        }
        let clean = super::strip_comments(&self.source[first.start..end]);
        let raw = clean.as_str();
        if !supports_color(raw) {
            return Err(Error::Invalid);
        }
        parse_color(raw)
            .map(GradientColor::Color)
            .ok_or(Error::Invalid)
    }
    fn position(&mut self) -> Result<GradientPosition, Error> {
        if self.kind() == Some(Kind::Number) {
            self.zero()?;
            self.cursor.terms += 1;
            if self.cursor.terms > super::MAX_CALC_TERMS {
                return Err(Error::Limit);
            }
            return Ok(GradientPosition::default());
        }
        if !matches!(
            self.kind(),
            Some(Kind::Dimension | Kind::Percentage | Kind::Function)
        ) {
            return Err(Error::Invalid);
        }
        let value = self.cursor.operand(0)?;
        if !value.px.is_finite() || !value.percent.is_finite() {
            return Err(Error::Invalid);
        }
        Ok(GradientPosition {
            px: value.px,
            percent: value.percent,
        })
    }
    fn gradient(&mut self) -> Result<ParsedGradient, Error> {
        let token = self.cursor.take()?;
        if token.kind != Kind::Function || !token.is_name("linear-gradient") {
            return Err(Error::Invalid);
        }
        self.cursor.spaces()?;
        let direction = if self.name("to")
            || self.name("in")
            || matches!(self.kind(), Some(Kind::Number | Kind::Dimension))
        {
            self.prelude()?
        } else {
            GradientDirection::Angle(180.0)
        };
        let mut result = ParsedGradient {
            direction,
            stops: [EMPTY_STOP; MAX_GRADIENT_STOPS],
            len: 0,
        };
        for _ in 0..MAX_AUTHORED_STOPS {
            let color = self.color()?;
            self.cursor.spaces()?;
            let mut positions = [None; 2];
            let mut count = 0;
            while !matches!(self.kind(), Some(Kind::Comma | Kind::Close(')'))) {
                if count == 2 {
                    return Err(Error::Invalid);
                }
                positions[count] = Some(self.position()?);
                count += 1;
                self.cursor.spaces()?;
            }
            result.push(color, positions[0])?;
            if count == 2 {
                result.push(color, positions[1])?;
            }
            let separator = self.cursor.take()?;
            if separator.kind == Kind::Close(')') {
                return Ok(result);
            }
            self.cursor.spaces()?;
        }
        Err(Error::Invalid)
    }
}

pub(super) fn parse_image(
    value: &str,
    units: [f32; 4],
    work: &mut usize,
) -> Result<Option<ParsedImage>, Error> {
    if value.len() > 4096 {
        return Err(Error::Limit);
    }
    // Pays bounded lexical/name/color scans and conversions in this module,
    // including strict legacy color validation followed by RGBA decoding.
    // Token storage/takes and every calculation operand debit additionally.
    spend(work, value.len().saturating_mul(48) + 64)?;
    let tokens = selectors::tokens(value, work)?;
    let mut parser = Parser {
        source: value,
        cursor: CalculationParser {
            tokens: &tokens,
            at: 0,
            terms: 0,
            units,
            work,
        },
    };
    let result = (|| {
        parser.cursor.spaces()?;
        let image = if parser.name("none") {
            parser.cursor.take()?;
            ParsedImage::None
        } else {
            ParsedImage::Gradient(parser.gradient()?)
        };
        parser.cursor.spaces()?;
        if parser.cursor.at != tokens.len() {
            return Err(Error::Invalid);
        }
        Ok(image)
    })();
    match result {
        Ok(v) => Ok(Some(v)),
        Err(Error::Limit) => Err(Error::Limit),
        Err(_) => Ok(None),
    }
}

/// Returns borrowed image/color components only after validating the entire
/// single-layer shorthand. No longhand is emitted from a malformed prefix.
pub(crate) fn valid(name: &str, value: &str, work: &mut usize) -> Result<bool, Error> {
    if css_wide(value) {
        return Ok(true);
    }
    let result = match name {
        "background-image" => {
            parse_image(value, [16.0, 16.0, 800.0, 600.0], work).map(|value| value.is_some())
        }
        _ => Ok(false),
    };
    match result {
        Err(Error::Limit) => Err(Error::Limit),
        Err(_) => Ok(false),
        value => value,
    }
}

#[derive(Default)]
pub(super) struct GradientPool {
    values: BTreeMap<Vec<u64>, Arc<LinearGradient>>,
    bytes: usize,
}
impl GradientPool {
    pub(super) fn intern(
        &mut self,
        parsed: &ParsedGradient,
        work: &mut usize,
    ) -> Result<Option<Arc<LinearGradient>>, Error> {
        // Fixed stack key, including explicit missing-position/currentColor tags.
        // Each B-tree probe has at most 11 comparisons per level, each bounded
        // by the incoming key length. Prepay lookup plus possible insertion.
        let key_len = 3 + 4 * parsed.len;
        let levels = self.values.len().saturating_add(1).ilog2() as usize + 1;
        spend(work, 128 + parsed.len * 16 + 22 * levels * (key_len + 1))?;
        let mut key = [0u64; 3 + 4 * MAX_GRADIENT_STOPS];
        match parsed.direction {
            GradientDirection::Angle(v) => {
                key[0] = 0;
                key[1] = if v == 0.0 { 0 } else { v.to_bits() };
            }
            GradientDirection::Corner { right, bottom } => {
                key[0] = 1;
                key[1] = u64::from(right);
                key[2] = u64::from(bottom);
            }
        }
        for (index, stop) in parsed.stops().iter().enumerate() {
            let start = 3 + index * 4;
            key[start] = match stop.color {
                GradientColor::CurrentColor => 1 << 32,
                GradientColor::Color(c) => u64::from(u32::from_be_bytes([c.r, c.g, c.b, c.a])),
            };
            if let Some(position) = stop.position {
                key[start + 1] = 1;
                key[start + 2] = if position.px == 0.0 {
                    0
                } else {
                    position.px.to_bits()
                };
                key[start + 3] = if position.percent == 0.0 {
                    0
                } else {
                    position.percent.to_bits()
                };
            }
        }
        let key = &key[..key_len];
        if let Some(shared) = self.values.get(key) {
            return Ok(Some(Arc::clone(shared)));
        }
        // Charge every retained allocation, including Arc headers, the exact key
        // and stop arrays and a whole maximum-capacity B-tree node per entry.
        // There cannot be more retained nodes than entries; this deliberately
        // covers occupancy slack, edges, allocator alignment and map metadata.
        let node = 11
            * (std::mem::size_of::<Vec<u64>>() + std::mem::size_of::<Arc<LinearGradient>>())
            + 16 * std::mem::size_of::<usize>()
            + 64;
        let bytes = node
            + key_len * 8
            + std::mem::size_of::<LinearGradient>()
            + std::mem::size_of_val(parsed.stops())
            + 64;
        if bytes > MAX_RETAINED_GRADIENT_BYTES.saturating_sub(self.bytes) {
            return Ok(None);
        }
        let value = Arc::new(LinearGradient {
            direction: parsed.direction,
            stops: Arc::from(parsed.stops()),
        });
        self.values.insert(key.to_vec(), Arc::clone(&value));
        self.bytes += bytes;
        Ok(Some(value))
    }
}

#[cfg(test)]
mod tests;
