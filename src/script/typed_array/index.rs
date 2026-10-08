//! CanonicalNumericIndexString for an already converted UTF-16 property key.
//! Numeric does not imply a live element: negative zero, NaN, infinities,
//! fractions and out-of-range integers remain terminal numeric keys.
use crate::script::{JsString, Result, Runtime, Value};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
pub(in crate::script) enum Index {
    Ordinary,
    Numeric(f64),
}

impl Runtime {
    pub(in crate::script) fn typed_array_index(&mut self, key: &JsString) -> Result<Index> {
        // Binary64's longest canonical spelling has 25 UTF-16 units. Length
        // rejection neither scans the key nor creates temporary storage.
        self.work(8)?;
        let length = key.len();
        if length == 0 || length > 25 {
            return Ok(Index::Ordinary);
        }
        let units = key.units();
        for unit in units {
            self.work(4)?;
            if *unit > 0x7f {
                return Ok(Index::Ordinary);
            }
        }

        self.work(4)?;
        let special: Option<(&[u8], f64)> = match length {
            2 => Some((b"-0", -0.0)),
            3 => Some((b"NaN", f64::NAN)),
            8 => Some((b"Infinity", f64::INFINITY)),
            9 => Some((b"-Infinity", f64::NEG_INFINITY)),
            _ => None,
        };
        if let Some((literal, number)) = special {
            let mut matches = true;
            for (unit, byte) in units.iter().zip(literal) {
                self.work(6)?;
                if *unit != u16::from(*byte) {
                    matches = false;
                    break;
                }
            }
            if matches {
                self.work(2)?;
                return Ok(Index::Numeric(number));
            }
        }

        // Every normalized integer through 2^53-1 has an exact and unique
        // nearest decimal spelling. Larger magnitudes require roundtripping.
        self.work(8)?;
        let negative = units[0] == u16::from(b'-');
        let start = usize::from(negative);
        let normalized = start < length && (length - start == 1 || units[start] != u16::from(b'0'));
        if normalized {
            let mut magnitude = 0u64;
            let mut integer = true;
            for unit in &units[start..] {
                self.work(16)?;
                let unit = *unit;
                if !(u16::from(b'0')..=u16::from(b'9')).contains(&unit) {
                    integer = false;
                    break;
                }
                let next = magnitude
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(u64::from(unit - u16::from(b'0'))));
                match next {
                    Some(value) if value < (1u64 << 53) => magnitude = value,
                    _ => {
                        integer = false;
                        break;
                    }
                }
            }
            if integer {
                self.work(4)?;
                let number = magnitude as f64;
                return Ok(Index::Numeric(if negative { -number } else { number }));
            }
        }

        // After specials and safe integers, a finite canonical spelling can
        // contain only this decimal alphabet. Reject metadata/expando names
        // before primitive parsing or formatter scratch allocation. This is
        // an alphabet filter, not a second numeric grammar or parser.
        self.work(4)?;
        for unit in units {
            self.work(10)?;
            if !matches!(*unit, 0x30..=0x39 | 0x2e | 0x65 | 0x2b | 0x2d) {
                return Ok(Index::Ordinary);
            }
        }

        // This primitive String conversion cannot invoke author hooks. It
        // retains the existing parse tariff and temporary ASCII admission.
        self.work(4)?;
        let number = self.primitive_number_value(Value::String(key.clone()))?;
        // number_text admits only the additional exact decimal correction;
        // this new caller must also admit the inherited formatter's scratch.
        self.work(128)?;
        self.charge(1024)?;
        let formatted = self.number_text(number)?;
        self.work(4)?;
        if formatted.len() != length {
            return Ok(Index::Ordinary);
        }
        let bytes = formatted.as_bytes();
        for (unit, byte) in units.iter().zip(bytes) {
            self.work(6)?;
            if *unit != u16::from(*byte) {
                return Ok(Index::Ordinary);
            }
        }
        self.work(2)?;
        Ok(Index::Numeric(number))
    }
}
