//! ECMAScript URI percent transforms over lossless UTF-16 code units.
//! The visitor allocates no storage and emits at most twelve units at a time.
//! Callers must charge traversal work and output storage before collecting it.

#[derive(Clone, Copy, Debug)]
pub(crate) enum Mode {
    Encode { component: bool },
    Decode { component: bool },
}
impl Mode {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "encodeURI" => Self::Encode { component: false },
            "encodeURIComponent" => Self::Encode { component: true },
            "decodeURI" => Self::Decode { component: false },
            "decodeURIComponent" => Self::Decode { component: true },
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Error {
    Malformed,
    OutputLimit,
}

const RESERVED: &[u8] = b";/?:@&=+$,#";
const HEX: &[u8] = b"0123456789ABCDEF";

fn unescaped(unit: u16, component: bool) -> bool {
    u8::try_from(unit).is_ok_and(|byte| {
        byte.is_ascii_alphanumeric()
            || b"_-.!~*'()".contains(&byte)
            || !component && RESERVED.contains(&byte)
    })
}

fn hex(unit: u16) -> Result<u8, Error> {
    match unit {
        0x30..=0x39 => Ok((unit - 0x30) as u8),
        0x41..=0x46 => Ok((unit - 0x41 + 10) as u8),
        0x61..=0x66 => Ok((unit - 0x61 + 10) as u8),
        _ => Err(Error::Malformed),
    }
}

fn octet(input: &[u16], at: usize) -> Result<u8, Error> {
    let triple = input
        .get(at..at.saturating_add(3))
        .ok_or(Error::Malformed)?;
    if triple.len() != 3 || triple[0] != u16::from(b'%') {
        return Err(Error::Malformed);
    }
    Ok(hex(triple[1])? * 16 + hex(triple[2])?)
}

pub(crate) fn visit(
    input: &[u16],
    mode: Mode,
    mut emit: impl FnMut(&[u16]) -> Result<(), Error>,
) -> Result<(), Error> {
    let mut at = 0;
    while at < input.len() {
        let start = at;
        let unit = input[at];
        match mode {
            Mode::Encode { component } => {
                at += 1;
                if unescaped(unit, component) {
                    emit(&input[start..at])?;
                    continue;
                }
                let point = if (0xd800..=0xdbff).contains(&unit) {
                    let low = *input.get(at).ok_or(Error::Malformed)?;
                    if !(0xdc00..=0xdfff).contains(&low) {
                        return Err(Error::Malformed);
                    }
                    at += 1;
                    0x10000 + ((u32::from(unit) - 0xd800) << 10) + (u32::from(low) - 0xdc00)
                } else {
                    u32::from(unit)
                };
                let scalar = char::from_u32(point).ok_or(Error::Malformed)?;
                let mut bytes = [0; 4];
                let bytes = scalar.encode_utf8(&mut bytes).as_bytes();
                let mut escaped = [0; 12];
                for (index, &byte) in bytes.iter().enumerate() {
                    escaped[index * 3] = u16::from(b'%');
                    escaped[index * 3 + 1] = u16::from(HEX[usize::from(byte >> 4)]);
                    escaped[index * 3 + 2] = u16::from(HEX[usize::from(byte & 15)]);
                }
                emit(&escaped[..bytes.len() * 3])?;
            }
            Mode::Decode { component } => {
                if unit != u16::from(b'%') {
                    at += 1;
                    // Decode leaves raw UTF-16 untouched, even lone surrogates.
                    emit(&input[start..at])?;
                    continue;
                }
                let first = octet(input, at)?;
                let count = match first {
                    0..=0x7f => 1,
                    0xc2..=0xdf => 2,
                    0xe0..=0xef => 3,
                    0xf0..=0xf4 => 4,
                    _ => return Err(Error::Malformed),
                };
                if input.len() - at < count * 3 {
                    return Err(Error::Malformed);
                }
                let mut bytes = [0; 4];
                bytes[0] = first;
                for (index, byte) in bytes.iter_mut().enumerate().take(count).skip(1) {
                    *byte = octet(input, at + index * 3)?;
                }
                at += count * 3;
                if count == 1 && !component && RESERVED.contains(&first) {
                    emit(&input[start..at])?;
                    continue;
                }
                // Rust's strict UTF-8 validation rejects bad continuations,
                // overlong forms, surrogate scalars and values above U+10FFFF.
                let text = std::str::from_utf8(&bytes[..count]).map_err(|_| Error::Malformed)?;
                let scalar = text.chars().next().ok_or(Error::Malformed)?;
                let mut units = [0; 2];
                emit(scalar.encode_utf16(&mut units))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transformed(input: &[u16], mode: Mode) -> Result<Vec<u16>, Error> {
        let mut output = Vec::new();
        visit(input, mode, |units| {
            output.extend_from_slice(units);
            Ok(())
        })?;
        Ok(output)
    }

    #[test]
    fn uri_scalar_boundaries_and_all_isolated_surrogates() {
        let encode = Mode::Encode { component: true };
        let decode = Mode::Decode { component: true };
        for (point, escaped) in [
            (0, "%00"),
            (0x7f, "%7F"),
            (0x80, "%C2%80"),
            (0x7ff, "%DF%BF"),
            (0x800, "%E0%A0%80"),
            (0xd7ff, "%ED%9F%BF"),
            (0xe000, "%EE%80%80"),
            (0xffff, "%EF%BF%BF"),
            (0x10000, "%F0%90%80%80"),
            (0x10ffff, "%F4%8F%BF%BF"),
        ] {
            let mut units = [0; 2];
            let text = char::from_u32(point).unwrap().encode_utf16(&mut units);
            let expected: Vec<_> = escaped.encode_utf16().collect();
            assert_eq!(transformed(text, encode).unwrap(), expected);
            assert_eq!(transformed(&expected, decode).unwrap(), text);
        }
        for unit in 0xd800..=0xdfff {
            assert_eq!(transformed(&[unit], encode), Err(Error::Malformed));
            assert_eq!(transformed(&[unit], decode).unwrap(), [unit]);
        }
    }

    #[test]
    fn uri_rejects_malformed_utf8_and_stops_at_sink_failure() {
        let decode = Mode::Decode { component: true };
        for input in [
            "%",
            "%0",
            "%GG",
            "%u0041",
            "%80",
            "%BF",
            "%C0%80",
            "%C1%BF",
            "%C2",
            "%C2A0",
            "%C2%20",
            "%E0%80%80",
            "%ED%A0%80",
            "%ED%BF%BF",
            "%F0%80%80%80",
            "%F4%90%80%80",
            "%F5%80%80%80",
            "%F8%88%80%80%80",
            "%FF",
            "%E2%82",
            "%E2%82x",
            "%E2%82%AC%80",
        ] {
            let units: Vec<_> = input.encode_utf16().collect();
            assert_eq!(
                transformed(&units, decode),
                Err(Error::Malformed),
                "{input}"
            );
        }
        let mut calls = 0;
        assert_eq!(
            visit(&[97, 98, 99], decode, |_| {
                calls += 1;
                Err(Error::OutputLimit)
            }),
            Err(Error::OutputLimit)
        );
        assert_eq!(calls, 1);
    }
}
