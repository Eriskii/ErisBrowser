//! ECMAScript decimal notation and exact shortest-decimal midpoint selection.
use super::*;

#[cfg(test)]
mod tests;

// Rust's shortest conversion can select the upper decimal at an exact tie.
// ECMA-262 selects the even decimal significand. Keep the shortest digit
// generator, and compare adjacent midpoints using only exact integer factors.
fn even_significand(
    number: f64,
    digits: &str,
    position: i32,
    debit: &mut impl FnMut(usize, usize) -> Result<()>,
) -> Result<Option<u64>> {
    debit(4, 0)?;
    let decimal_exponent = position - digits.len() as i32;
    if !(-24..=22).contains(&decimal_exponent) {
        return Ok(None);
    }
    debit(3, 0)?;
    if digits.as_bytes().last().is_some_and(|digit| digit & 1 == 0) {
        return Ok(None);
    }
    debit(16, 0)?;
    let bits = number.to_bits();
    let exponent = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = (bits & 0x000f_ffff_ffff_ffff) | if exponent == 0 { 0 } else { 1 << 52 };
    let zeros = mantissa.trailing_zeros();
    let odd = mantissa >> zeros;
    let binary_exponent = if exponent == 0 {
        -1074
    } else {
        exponent - 1075
    } + zeros as i32;
    // (2*s +/- 1)*10^q/2 has binary exponent q-1 after cancelling fives.
    if binary_exponent != decimal_exponent - 1 {
        return Ok(None);
    }
    debit(4 + 3 * digits.len(), 0)?;
    let overflow = || ScriptError::resource("decimal correction integer overflow");
    let significand = digits.bytes().try_fold(0u64, |n, digit| {
        n.checked_mul(10)
            .and_then(|n| n.checked_add(u64::from(digit - b'0')))
            .ok_or_else(overflow)
    })?;
    debit(8, 0)?;
    let doubled = significand.checked_mul(2).ok_or_else(overflow)?;
    for (numerator, adjacent) in [
        (
            doubled.checked_sub(1).ok_or_else(overflow)?,
            significand.checked_sub(1).ok_or_else(overflow)?,
        ),
        (
            doubled.checked_add(1).ok_or_else(overflow)?,
            significand.checked_add(1).ok_or_else(overflow)?,
        ),
    ] {
        let count = decimal_exponent.unsigned_abs() as usize;
        debit(4 + 4 * count, 0)?;
        let mut factor = numerator;
        let mut exact = true;
        for _ in 0..count {
            if decimal_exponent < 0 {
                if factor % 5 != 0 {
                    exact = false;
                    break;
                }
                factor /= 5;
            } else {
                // An odd binary64 significand fits in 53 bits. Stop before
                // either a useless larger product or host-integer overflow.
                if factor > ((1u64 << 53) - 1) / 5 {
                    exact = false;
                    break;
                }
                factor *= 5;
            }
        }
        if exact && factor == odd {
            // A changed u64 decimal has at most one more digit. Reserve its
            // conversion, subsequent zero trim and one short String block.
            debit(8 + 4 * (digits.len() + 1), 32)?;
            return Ok(Some(adjacent));
        }
    }
    Ok(None)
}

// Callers retain their original formatting/storage fees. The callback admits
// only the new exact correction operations, before executing or allocating
// them. Both runtime and compiled literal property names use this same path.
pub(super) fn format(
    number: f64,
    mut debit: impl FnMut(usize, usize) -> Result<()>,
) -> Result<String> {
    if !number.is_finite() {
        return Ok(Value::Number(number).to_string());
    }
    if number == 0.0 {
        return Ok("0".into());
    }
    let raw = number.abs().to_string();
    let (mantissa, exponent) = raw
        .split_once(['e', 'E'])
        .map(|(m, e)| (m, e.parse::<i32>().unwrap_or(0)))
        .unwrap_or((&raw, 0));
    let decimal = mantissa.find('.').unwrap_or(mantissa.len()) as i32;
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let leading = digits.bytes().take_while(|byte| *byte == b'0').count();
    let digits = digits[leading..].trim_end_matches('0');
    let position = decimal + exponent - leading as i32;
    let corrected = even_significand(number, digits, position, &mut debit)?;
    let corrected = corrected.map(|significand| significand.to_string());
    let position = position - digits.len() as i32
        + corrected.as_ref().map_or(digits.len(), String::len) as i32;
    let digits = corrected.as_deref().unwrap_or(digits).trim_end_matches('0');
    let power = position - 1;
    let mut output = if number.is_sign_negative() {
        "-".to_owned()
    } else {
        String::new()
    };
    if (-6..21).contains(&power) {
        if position <= 0 {
            output.push_str("0.");
            output.push_str(&"0".repeat((-position) as usize));
            output.push_str(digits);
        } else if position as usize >= digits.len() {
            output.push_str(digits);
            output.push_str(&"0".repeat(position as usize - digits.len()));
        } else {
            output.push_str(&digits[..position as usize]);
            output.push('.');
            output.push_str(&digits[position as usize..]);
        }
    } else {
        output.push_str(&digits[..1]);
        if digits.len() > 1 {
            output.push('.');
            output.push_str(&digits[1..]);
        }
        output.push('e');
        if power >= 0 {
            output.push('+');
        }
        output.push_str(&power.to_string());
    }
    Ok(output)
}

impl Runtime {
    pub(super) fn number_text(&mut self, number: f64) -> Result<String> {
        format(number, |work, bytes| {
            self.work(work)?;
            self.charge(bytes)
        })
    }

    pub(super) fn primitive_js_string(&mut self, value: &Value) -> Result<JsString> {
        Ok(match value {
            Value::String(text) => text.clone(),
            Value::Number(number) => self.number_text(*number)?.into(),
            _ => value.to_string().into(),
        })
    }
}
