//! A bounded inherited pair of nonnegative computed lengths.
use super::{CalculationParser, css_wide};
use crate::selectors::{self, Error, Kind, Token, spend};

fn lexical_zero(raw: &str) -> bool {
    raw.trim_start_matches(['+', '-'])
        .split(['e', 'E'])
        .next()
        .is_some_and(|mantissa| mantissa.bytes().all(|b| b == b'0' || b == b'.'))
}

// Supports scanners already prepay their decoded-name scans. Final validation
// below still rejects unknown units/functions; this only admits escaped tokens
// to that same grammar instead of rejecting their raw spelling early.
pub(super) fn escape_allowed(token: Token<'_>) -> bool {
    !token.raw.contains('\0')
        && (matches!(token.kind, Kind::Dimension | Kind::Function)
            || token.kind == Kind::Ident && css_wide(token.raw))
}

pub(crate) fn valid(value: &str, work: &mut usize) -> Result<bool, Error> {
    if value.len() > 4096 {
        return Err(Error::Limit);
    }
    if css_wide(value.trim()) {
        return Ok(true);
    }
    Ok(parse(value, [16.0, 16.0, 800.0, 600.0], work)?.is_some())
}

pub(super) fn parse(
    value: &str,
    units: [f32; 4],
    work: &mut usize,
) -> Result<Option<[f32; 2]>, Error> {
    if value.len() > 4096 {
        return Err(Error::Limit);
    }
    let result = (|| {
        // One token stream and one terms/depth counter cover the entire pair.
        let tokens = selectors::tokens(value, work)?;
        let mut parser = CalculationParser {
            tokens: &tokens,
            at: 0,
            terms: 0,
            units,
            work,
        };
        let mut pair = [0.0; 2];
        let mut count = 0;
        parser.spaces()?;
        while parser.at < tokens.len() {
            if count == 2 {
                return Err(Error::Invalid);
            }
            let token = tokens[parser.at];
            spend(parser.work, token.raw.len().saturating_mul(3) + 4)?;
            pair[count] = if token.kind == Kind::Number {
                parser.take()?;
                if !lexical_zero(token.raw) {
                    return Err(Error::Invalid);
                }
                0.0
            } else {
                if !matches!(token.kind, Kind::Dimension | Kind::Function) {
                    return Err(Error::Invalid);
                }
                if token.kind == Kind::Dimension {
                    let end = selectors::number_end(token.raw).ok_or(Error::Invalid)?;
                    // Reject negative literals even when conversion underflows.
                    if token.raw.starts_with('-') && !lexical_zero(&token.raw[..end]) {
                        return Err(Error::Invalid);
                    }
                }
                let value = parser.operand(0)?;
                if value.has_percent
                    || !value.px.is_finite()
                    || value.px.abs() > f64::from(f32::MAX)
                {
                    return Err(Error::Invalid);
                }
                // Range-check after complete additive calculation, preserving
                // cancellation. The finite used-length ceiling is unchanged.
                value.px.clamp(0.0, 1_000_000.0) as f32
            };
            // Canonical positive zero, including a literal -0px.
            if pair[count] == 0.0 {
                pair[count] = 0.0;
            }
            count += 1;
            parser.spaces()?;
        }
        spend(parser.work, 8)?;
        if count == 0 {
            return Err(Error::Invalid);
        }
        if count == 1 {
            pair[1] = pair[0];
        }
        Ok(pair)
    })();
    match result {
        Ok(pair) => Ok(Some(pair)),
        Err(Error::Limit) => Err(Error::Limit),
        Err(_) => Ok(None),
    }
}

#[cfg(test)]
#[path = "border_spacing/tests.rs"]
mod tests;
