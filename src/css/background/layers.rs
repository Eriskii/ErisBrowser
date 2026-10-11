//! Authored lists stay independent. Only BackgroundData::layer cycles values.
use super::*;
use crate::css::{BackgroundImage, Rule, WinningSource};

const DEFAULTS: [&str; 7] = [
    "transparent",
    "none",
    "auto auto",
    "0% 0%",
    "repeat",
    "padding-box",
    "border-box",
];

struct Slices<'a> {
    items: [&'a str; MAX_BACKGROUND_LAYERS],
    len: usize,
}
impl<'a> Slices<'a> {
    fn one(value: &'a str) -> Self {
        let mut items = [""; MAX_BACKGROUND_LAYERS];
        items[0] = value;
        Self { items, len: 1 }
    }
    fn values(&self) -> &[&'a str] {
        &self.items[..self.len]
    }
}

/// The tokenizer owns string/URL/escape recognition. No byte comma scan can
/// mistake punctuation inside a payload or nested function for a layer edge.
fn split<'a>(value: &'a str, work: &mut usize) -> Result<Option<Slices<'a>>, Error> {
    let tokens = tokens(value, work)?;
    // One fixed-depth visit per token, including rejected input and all slices.
    spend(work, tokens.len().saturating_mul(2) + 1)?;
    accepted((|| {
        let mut result = Slices {
            items: [""; MAX_BACKGROUND_LAYERS],
            len: 0,
        };
        let mut closes = [')'; super::super::MAX_CALC_DEPTH + 3];
        let (mut depth, mut start) = (0, 0);
        for (i, t) in tokens.iter().enumerate() {
            if let Some(open) = t.open() {
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
            } else if let Kind::Close(close) = t.kind {
                if depth == 0 || closes[depth - 1] != close {
                    return Err(Error::Invalid);
                }
                depth -= 1;
            } else if t.kind == Kind::Comma && depth == 0 {
                push_slice(&mut result, value, &tokens[start..i])?;
                start = i + 1;
            }
        }
        if depth != 0 {
            return Err(Error::Invalid);
        }
        push_slice(&mut result, value, &tokens[start..])?;
        Ok(result)
    })())
}
fn push_slice<'a>(out: &mut Slices<'a>, value: &'a str, tokens: &[Token<'a>]) -> Result<(), Error> {
    let tokens = selectors::trim(tokens);
    if tokens.is_empty() || out.len == MAX_BACKGROUND_LAYERS {
        return Err(Error::Invalid);
    }
    out.items[out.len] = &value[tokens[0].start..tokens[tokens.len() - 1].end];
    out.len += 1;
    Ok(())
}

struct Values<T: Copy> {
    items: [T; MAX_BACKGROUND_LAYERS],
    len: usize,
}
fn list<T: Copy>(
    value: &str,
    work: &mut usize,
    mut parse: impl FnMut(&str, &mut usize) -> Result<Option<T>, Error>,
) -> Result<Option<Values<T>>, Error> {
    // Preserve all existing singleton parser work cuts without an extra scan.
    if let Some(first) = parse(value, work)? {
        return Ok(Some(Values {
            items: [first; MAX_BACKGROUND_LAYERS],
            len: 1,
        }));
    }
    let Some(parts) = split(value, work)? else {
        return Ok(None);
    };
    if parts.len < 2 {
        return Ok(None);
    }
    let Some(first) = parse(parts.items[0], work)? else {
        return Ok(None);
    };
    let mut result = Values {
        items: [first; MAX_BACKGROUND_LAYERS],
        len: parts.len,
    };
    for (i, part) in parts.values().iter().enumerate().skip(1) {
        let Some(item) = parse(part, work)? else {
            return Ok(None);
        };
        result.items[i] = item;
    }
    Ok(Some(result))
}

pub(in crate::css) struct ExpandedParts<'a> {
    layers: [[&'a str; 7]; MAX_BACKGROUND_LAYERS],
    len: usize,
}
impl ExpandedParts<'_> {
    fn size(&self, index: usize) -> usize {
        if index == 0 {
            self.layers[self.len - 1][0].len()
        } else {
            self.layers[..self.len]
                .iter()
                .map(|p| p[index].len())
                .sum::<usize>()
                + 2 * (self.len - 1)
        }
    }
    pub(in crate::css) fn debit(&self, work: &mut usize) -> Result<(), Error> {
        spend(work, (0..7).map(|i| self.size(i)).sum::<usize>() + 448)
    }
    pub(in crate::css) fn value(&self, index: usize) -> String {
        if index == 0 {
            return self.layers[self.len - 1][0].into();
        }
        let mut out = String::with_capacity(self.size(index));
        for (i, layer) in self.layers[..self.len].iter().enumerate() {
            if i != 0 {
                out.push_str(", ");
            }
            out.push_str(layer[index]);
        }
        out
    }
}
pub(in crate::css) fn expanded_parts<'a>(
    value: &'a str,
    work: &mut usize,
) -> Result<Option<ExpandedParts<'a>>, Error> {
    if let Some(parts) = parts(value, work)? {
        return Ok(Some(ExpandedParts {
            layers: [parts; MAX_BACKGROUND_LAYERS],
            len: 1,
        }));
    }
    let Some(slices) = split(value, work)? else {
        return Ok(None);
    };
    if slices.len < 2 {
        return Ok(None);
    }
    let mut out = ExpandedParts {
        layers: [DEFAULTS; MAX_BACKGROUND_LAYERS],
        len: slices.len,
    };
    for (i, slice) in slices.values().iter().enumerate() {
        let Some((parts, color)) = parts_with_color(slice, work)? else {
            return Ok(None);
        };
        if color && i + 1 != slices.len {
            return Ok(None);
        }
        out.layers[i] = parts;
    }
    if (0..7).any(|i| out.size(i) > 4096) {
        return Ok(None);
    }
    Ok(Some(out))
}

pub(in crate::css) fn valid_list(name: &str, value: &str, work: &mut usize) -> Result<bool, Error> {
    let units = [16.0, 16.0, 800.0, 600.0];
    match name {
        "background" => Ok(expanded_parts(value, work)?.is_some()),
        "background-image" => {
            if gradient::parse_image(value, units, work)?.is_some() {
                return Ok(true);
            }
            let Some(slices) = split(value, work)? else {
                return Ok(false);
            };
            if slices.len < 2 {
                return Ok(false);
            }
            for slice in slices.values() {
                if gradient::parse_image(slice, units, work)?.is_none() {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        "background-size" => Ok(list(value, work, |s, w| parse_size(s, units, w))?.is_some()),
        "background-position" => {
            Ok(list(value, work, |s, w| parse_position(s, units, w))?.is_some())
        }
        "background-repeat" => Ok(list(value, work, parse_repeat)?.is_some()),
        "background-origin" | "background-clip" => Ok(list(value, work, parse_box)?.is_some()),
        _ => Ok(false),
    }
}

fn image(
    parsed: gradient::ParsedImage<'_>,
    pool: &mut BackgroundStorage,
    source: WinningSource,
    rules: &[Rule],
    base: &::url::Url,
    work: &mut usize,
) -> Result<Option<BackgroundImage>, Error> {
    match parsed {
        gradient::ParsedImage::None => Ok(None),
        gradient::ParsedImage::Gradient(value) => pool
            .gradient(&value, work)?
            .map(BackgroundImage::Gradient)
            .map(Some)
            .ok_or(Error::Limit),
        gradient::ParsedImage::Url(value) => pool
            .url(value, source, rules, base, work)?
            .map(BackgroundImage::Url)
            .map(Some)
            .ok_or(Error::Limit),
    }
}

#[allow(clippy::too_many_arguments)]
pub(in crate::css) fn apply(
    name: &str,
    value: &str,
    units: [f32; 4],
    builder: &mut BackgroundBuilder,
    pool: &mut BackgroundStorage,
    source: WinningSource,
    rules: &[Rule],
    base: &::url::Url,
    work: &mut usize,
) -> Result<bool, Error> {
    match name {
        "background-image" => {
            let mut images: [Option<BackgroundImage>; MAX_BACKGROUND_LAYERS] =
                std::array::from_fn(|_| None);
            let count = if let Some(parsed) = gradient::parse_image(value, units, work)? {
                images[0] = image(parsed, pool, source, rules, base, work)?;
                1
            } else {
                let Some(slices) = split(value, work)? else {
                    return Ok(true);
                };
                if slices.len < 2 {
                    return Ok(true);
                }
                for (i, slice) in slices.values().iter().enumerate() {
                    let Some(parsed) = gradient::parse_image(slice, units, work)? else {
                        return Ok(true);
                    };
                    images[i] = image(parsed, pool, source, rules, base, work)?;
                }
                slices.len
            };
            pool.set_images(builder, &images[..count], work)?;
        }
        "background-size" => {
            if let Some(v) = list(value, work, |s, w| parse_size(s, units, w))? {
                pool.set_sizes(builder, &v.items[..v.len], work)?;
            }
        }
        "background-position" => {
            if let Some(v) = list(value, work, |s, w| parse_position(s, units, w))? {
                pool.set_positions(builder, &v.items[..v.len], work)?;
            }
        }
        "background-repeat" => {
            if let Some(v) = list(value, work, parse_repeat)? {
                pool.set_repeats(builder, &v.items[..v.len], work)?;
            }
        }
        "background-origin" => {
            if let Some(v) = list(value, work, parse_box)? {
                pool.set_origins(builder, &v.items[..v.len], work)?;
            }
        }
        "background-clip" => {
            if let Some(v) = list(value, work, parse_box)? {
                pool.set_clips(builder, &v.items[..v.len], work)?;
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

/// CSSOM specified-value serialization uses the coordinating image length.
/// Only an exact singleton initial dependent may be omitted at another length;
/// equal repeated values are deliberately not collapsed or cycled here.
pub(crate) fn serialize(values: &[&str], work: &mut usize) -> Result<String, Error> {
    if values.iter().any(|v| css_wide(v)) {
        return Ok(String::new());
    }
    let mut lists: [Slices<'_>; 6] = std::array::from_fn(|i| Slices::one(values[i + 1]));
    for (i, value) in values[1..].iter().enumerate() {
        let Some(parsed) = split(value, work)? else {
            return Ok(String::new());
        };
        lists[i] = parsed;
    }
    let count = lists[0].len;
    for (i, list) in lists.iter().enumerate().skip(1) {
        if list.len != count && !(list.len == 1 && list.items[0] == DEFAULTS[i + 1]) {
            return Ok(String::new());
        }
    }
    let mut out = String::new();
    for layer in 0..count {
        let mut v = DEFAULTS;
        v[0] = if layer + 1 == count {
            values[0]
        } else {
            "transparent"
        };
        for (i, list) in lists.iter().enumerate() {
            v[i + 1] = list.items[if list.len == 1 { 0 } else { layer }];
        }
        if layer != 0 {
            out.push_str(", ");
        }
        // Keep the established singleton spelling, except a colorless None
        // layer in a list must be 'none': a color is forbidden before the last.
        let color = if count > 1 && v[0] == "transparent" {
            ""
        } else {
            v[0]
        };
        let mut piece = if v[2] == "auto auto" && v[3] == "0% 0%" && v[4] == "repeat" {
            if v[1] == "none" {
                if color.is_empty() {
                    "none".to_owned()
                } else {
                    color.into()
                }
            } else if color.is_empty() || color == "transparent" {
                v[1].into()
            } else {
                format!("{} {}", v[1], color)
            }
        } else {
            format!(
                "{} {} / {} {}{}{}",
                v[1],
                v[3],
                v[2],
                v[4],
                if color.is_empty() { "" } else { " " },
                color
            )
        };
        if v[5] != "padding-box" || v[6] != "border-box" {
            piece.push(' ');
            piece.push_str(v[5]);
            if v[5] != v[6] {
                piece.push(' ');
                piece.push_str(v[6]);
            }
        }
        out.push_str(&piece);
    }
    Ok(out)
}
