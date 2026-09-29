//! Date internal slots, ordered conversions and bounded host-clock access.
use super::*;
use crate::js_date::{self, DateParts, IsoParse};
use std::fmt::Write;

#[cfg(test)]
mod tests;

const DAY_MS: f64 = 86_400_000.0;
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl Runtime {
    pub(super) fn initialize_date(&mut self) -> Result<()> {
        let constructor = self.native_properties["Date"];
        let prototype = self.prototypes["Date"];
        for (method, length) in [("now", 0), ("parse", 1), ("UTC", 7)] {
            self.work(32)?;
            self.charge(1024)?;
            let function = self.intrinsic_function(&format!("Date.{method}"), method, length)?;
            self.objects[constructor].insert_hidden(method.into(), function);
        }
        for (method, length) in [
            ("getDate", 0),
            ("getDay", 0),
            ("getFullYear", 0),
            ("getHours", 0),
            ("getMilliseconds", 0),
            ("getMinutes", 0),
            ("getMonth", 0),
            ("getSeconds", 0),
            ("getTime", 0),
            ("getTimezoneOffset", 0),
            ("getUTCDate", 0),
            ("getUTCDay", 0),
            ("getUTCFullYear", 0),
            ("getUTCHours", 0),
            ("getUTCMilliseconds", 0),
            ("getUTCMinutes", 0),
            ("getUTCMonth", 0),
            ("getUTCSeconds", 0),
            ("setDate", 1),
            ("setFullYear", 3),
            ("setHours", 4),
            ("setMilliseconds", 1),
            ("setMinutes", 3),
            ("setMonth", 2),
            ("setSeconds", 2),
            ("setTime", 1),
            ("setUTCDate", 1),
            ("setUTCFullYear", 3),
            ("setUTCHours", 4),
            ("setUTCMilliseconds", 1),
            ("setUTCMinutes", 3),
            ("setUTCMonth", 2),
            ("setUTCSeconds", 2),
            ("toDateString", 0),
            ("toISOString", 0),
            ("toJSON", 1),
            ("toLocaleDateString", 0),
            ("toLocaleString", 0),
            ("toLocaleTimeString", 0),
            ("toString", 0),
            ("toTimeString", 0),
            ("toUTCString", 0),
            ("valueOf", 0),
            ("getYear", 0),
            ("setYear", 1),
        ] {
            self.work(64)?;
            self.charge(1024)?;
            let function = self.intrinsic_function(&format!("Date.{method}"), method, length)?;
            self.objects[prototype].insert_hidden(method.into(), function);
        }
        self.charge(2048)?;
        self.work(64)?;
        // This is an alias with the original function identity and name.
        let utc = self.intrinsic_function("Date.toUTCString", "toUTCString", 0)?;
        self.objects[prototype].insert_hidden("toGMTString".into(), utc);
        let primitive = self.intrinsic_function("Date.toPrimitive", "[Symbol.toPrimitive]", 1)?;
        let key = self.well_known_key("toPrimitive");
        self.objects[prototype].insert_property(key, Property::data(primitive, false, false, true));
        // Date.prototype deliberately has no [[DateValue]].
        Ok(())
    }

    fn date_slot(&self, receiver: &Value) -> Result<(usize, f64)> {
        if let Value::Object(id) = receiver
            && let Some(value) = self.objects[*id].date_value
        {
            return Ok((*id, value));
        }
        Err(ScriptError::type_error(
            "Date method requires a Date internal slot",
        ))
    }

    pub(super) fn date_now(&mut self) -> Result<f64> {
        let value = self.date_host.now_unix_millis(&mut self.steps);
        match value {
            Ok(value) => Ok(js_date::time_clip(value as f64)),
            Err(error) if error.is_work_limit() => {
                Err(ScriptError::resource("Date clock work limit"))
            }
            Err(_) => Err(ScriptError::unsupported("Date host clock is unavailable")),
        }
    }

    fn date_offset(&mut self, utc: i64) -> Result<i32> {
        let zone = self
            .date_host
            .zone()
            .map_err(|_| ScriptError::unsupported("Date host local timezone is not configured"))?;
        match zone.offset_at_utc_ms(utc, &mut self.steps) {
            Ok(offset) => Ok(offset),
            Err(error) if error.is_work_limit() => {
                Err(ScriptError::resource("Date timezone work limit"))
            }
            Err(_) => Err(ScriptError::unsupported(
                "Date host timezone cannot represent this instant",
            )),
        }
    }

    fn date_local(&mut self, utc: f64) -> Result<f64> {
        if !utc.is_finite() {
            return Ok(f64::NAN);
        }
        Ok(utc + f64::from(self.date_offset(utc as i64)?) * 1000.0)
    }

    fn date_utc(&mut self, local: f64) -> Result<f64> {
        // Every accepted host offset is bounded by the timezone parser.
        // Beyond this guard no possible offset can produce an unclipped Date.
        // Required author conversions have already happened at each call site.
        if !local.is_finite()
            || local.abs()
                > js_date::MAX_TIME + f64::from(crate::date_host::MAX_ABS_OFFSET_SECONDS) * 1000.0
        {
            return Ok(f64::NAN);
        }
        let zone = self
            .date_host
            .zone()
            .map_err(|_| ScriptError::unsupported("Date host local timezone is not configured"))?;
        match zone.utc_from_local_ms(local as i64, &mut self.steps) {
            Ok(value) => Ok(value as f64),
            Err(error) if error.is_work_limit() => {
                Err(ScriptError::resource("Date timezone work limit"))
            }
            Err(_) => Err(ScriptError::unsupported(
                "Date host timezone cannot represent this local time",
            )),
        }
    }

    fn date_make_day(&mut self, year: f64, month: f64, day: f64) -> Result<f64> {
        self.work(js_date::make_day_work(year, month, day))?;
        Ok(js_date::make_day(year, month, day))
    }

    fn date_components(&mut self, args: &[Value], doc: &mut Document) -> Result<f64> {
        let mut numbers = [f64::NAN, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        // Even NaN or infinity in an earlier argument does not skip conversion
        // of later present arguments. Extras are evaluated by the caller only.
        for (index, value) in args.iter().take(7).enumerate() {
            numbers[index] = self.number_value(value.clone(), doc)?;
        }
        let [year, month, day, hour, minute, second, millisecond] = numbers;
        let day = self.date_make_day(js_date::full_year(year), month, day)?;
        self.work(32)?;
        Ok(js_date::make_date(
            day,
            js_date::make_time(hour, minute, second, millisecond),
        ))
    }

    pub(super) fn date_constructor(
        &mut self,
        args: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let time = match args {
            [] => self.date_now()?,
            [value] => {
                if let Ok((_, time)) = self.date_slot(value) {
                    time
                } else {
                    let primitive = self.number_hint_primitive(value.clone(), doc)?;
                    if let Value::String(text) = primitive {
                        self.date_parse(&text)?
                    } else {
                        self.primitive_number_value(primitive)?
                    }
                }
            }
            _ => {
                let local = self.date_components(args, doc)?;
                self.date_utc(local)?
            }
        };
        let time = js_date::time_clip(time);
        // This observable Get occurs after all argument coercion/clock access.
        let prototype = self.constructor_prototype(new_target, "Date", doc)?;
        let object = self.object_ordered([])?;
        let Value::Object(id) = object else {
            unreachable!()
        };
        self.objects[id].prototype = Some(prototype);
        self.objects[id].date_value = Some(time);
        Ok(object)
    }

    pub(super) fn date_native(
        &mut self,
        method: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let first = || args.first().cloned().unwrap_or(Value::Undefined);
        match method {
            "now" => return self.date_now().map(Value::Number),
            "UTC" => {
                return self
                    .date_components(args, doc)
                    .map(js_date::time_clip)
                    .map(Value::Number);
            }
            "parse" => {
                let text = self.string_hint(first(), doc)?;
                return self.date_parse(&text).map(Value::Number);
            }
            "toPrimitive" => {
                if !js_object(&receiver) {
                    return Err(ScriptError::type_error(
                        "Date primitive receiver must be an object",
                    ));
                }
                let Value::String(hint) = first() else {
                    return Err(ScriptError::type_error("invalid Date primitive hint"));
                };
                let keys = if hint.units().iter().copied().eq("string".encode_utf16())
                    || hint.units().iter().copied().eq("default".encode_utf16())
                {
                    ["toString", "valueOf"]
                } else if hint.units().iter().copied().eq("number".encode_utf16()) {
                    ["valueOf", "toString"]
                } else {
                    return Err(ScriptError::type_error("invalid Date primitive hint"));
                };
                // OrdinaryToPrimitive deliberately bypasses the exotic hook.
                // Otherwise invoking this method would recursively invoke itself.
                for key in keys {
                    self.charge(64)?;
                    let method = self.get(receiver.clone(), key, doc)?;
                    if json_callable(&method) {
                        let value = self.call(method, Vec::new(), receiver.clone(), doc)?;
                        if !js_object(&value) {
                            return Ok(value);
                        }
                    }
                }
                return Err(ScriptError::type_error(
                    "Date receiver cannot convert to a primitive",
                ));
            }
            "toJSON" => {
                let object = self.coerce_object(receiver)?;
                let primitive = self.primitive_with_hint(object.clone(), "number", doc)?;
                if matches!(primitive, Value::Number(value) if !value.is_finite()) {
                    return Ok(Value::Null);
                }
                let method = self.get(object.clone(), "toISOString", doc)?;
                return self.call(method, Vec::new(), object, doc);
            }
            _ => {}
        }
        let (id, time) = self.date_slot(&receiver)?;
        if method.starts_with("set") {
            return self
                .date_set(id, time, method, args, doc)
                .map(Value::Number);
        }
        match method {
            "valueOf" | "getTime" => return Ok(Value::Number(time)),
            "toISOString" => {
                if time.is_nan() {
                    return Err(ScriptError::range_error("invalid Date time value"));
                }
                self.work(128)?;
                self.charge(512)?;
                return Ok(Value::String(js_date::format_iso(time as i64).into()));
            }
            "toString" | "toDateString" | "toTimeString" | "toUTCString" | "toLocaleString"
            | "toLocaleDateString" | "toLocaleTimeString" => {
                return self.date_format(time, method);
            }
            _ => {}
        }
        if time.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        if method == "getTimezoneOffset" {
            let local = self.date_local(time)?;
            return Ok(Value::Number((time - local) / 60_000.0));
        }
        let (local, field) = if let Some(field) = method.strip_prefix("getUTC") {
            (time, field)
        } else {
            (
                self.date_local(time)?,
                method.strip_prefix("get").unwrap_or(method),
            )
        };
        self.work(128)?;
        let p = js_date::parts(local as i64);
        let number = match field {
            "Date" => f64::from(p.day),
            "Day" => f64::from(p.weekday),
            "FullYear" => f64::from(p.year),
            "Year" => f64::from(p.year) - 1900.0,
            "Hours" => f64::from(p.hour),
            "Milliseconds" => f64::from(p.millisecond),
            "Minutes" => f64::from(p.minute),
            "Month" => f64::from(p.month),
            "Seconds" => f64::from(p.second),
            _ => return Err(ScriptError::unsupported("unknown Date method")),
        };
        Ok(Value::Number(number))
    }

    fn date_set(
        &mut self,
        id: usize,
        saved: f64,
        method: &str,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<f64> {
        if method == "setTime" {
            let number =
                self.number_value(args.first().cloned().unwrap_or(Value::Undefined), doc)?;
            let value = js_date::time_clip(number);
            self.objects[id].date_value = Some(value);
            return Ok(value);
        }
        let (utc, field) = if let Some(field) = method.strip_prefix("setUTC") {
            (true, field)
        } else {
            (false, method.strip_prefix("set").unwrap_or(method))
        };
        let full_year = matches!(field, "FullYear" | "Year");
        let mut local = saved;
        // Annex B setYear converts the captured time to local before ToNumber.
        if field == "Year" {
            local = if saved.is_nan() {
                0.0
            } else {
                self.date_local(saved)?
            };
        }
        let first = self.number_value(args.first().cloned().unwrap_or(Value::Undefined), doc)?;
        if field == "FullYear" {
            local = if saved.is_nan() {
                0.0
            } else if utc {
                saved
            } else {
                self.date_local(saved)?
            };
        }
        let count = match field {
            "FullYear" | "Minutes" => 3,
            "Hours" => 4,
            "Month" | "Seconds" => 2,
            "Date" | "Milliseconds" | "Year" => 1,
            _ => return Err(ScriptError::unsupported("unknown Date setter")),
        };
        let mut supplied = [None; 4];
        supplied[0] = Some(first);
        for (index, value) in args.iter().take(count).enumerate().skip(1) {
            supplied[index] = Some(self.number_value(value.clone(), doc)?);
        }
        // Returning NaN here does not write the slot. A coercion callback may
        // have changed this originally-invalid Date; that change must survive.
        if saved.is_nan() && !full_year {
            return Ok(f64::NAN);
        }
        if !full_year && !utc {
            local = self.date_local(saved)?;
        }
        self.work(160)?;
        let p = js_date::parts(local as i64);
        let within = local.rem_euclid(DAY_MS);
        let day = (local / DAY_MS).floor();
        let changed = match field {
            "FullYear" | "Year" => {
                let year = if field == "Year" {
                    js_date::full_year(first)
                } else {
                    first
                };
                let month = supplied[1].unwrap_or(f64::from(p.month));
                let date = supplied[2].unwrap_or(f64::from(p.day));
                js_date::make_date(self.date_make_day(year, month, date)?, within)
            }
            "Month" => js_date::make_date(
                self.date_make_day(
                    f64::from(p.year),
                    first,
                    supplied[1].unwrap_or(f64::from(p.day)),
                )?,
                within,
            ),
            "Date" => js_date::make_date(
                self.date_make_day(f64::from(p.year), f64::from(p.month), first)?,
                within,
            ),
            "Hours" => js_date::make_date(
                day,
                js_date::make_time(
                    first,
                    supplied[1].unwrap_or(f64::from(p.minute)),
                    supplied[2].unwrap_or(f64::from(p.second)),
                    supplied[3].unwrap_or(f64::from(p.millisecond)),
                ),
            ),
            "Minutes" => js_date::make_date(
                day,
                js_date::make_time(
                    f64::from(p.hour),
                    first,
                    supplied[1].unwrap_or(f64::from(p.second)),
                    supplied[2].unwrap_or(f64::from(p.millisecond)),
                ),
            ),
            "Seconds" => js_date::make_date(
                day,
                js_date::make_time(
                    f64::from(p.hour),
                    f64::from(p.minute),
                    first,
                    supplied[1].unwrap_or(f64::from(p.millisecond)),
                ),
            ),
            "Milliseconds" => js_date::make_date(
                day,
                js_date::make_time(
                    f64::from(p.hour),
                    f64::from(p.minute),
                    f64::from(p.second),
                    first,
                ),
            ),
            _ => unreachable!(),
        };
        let value = js_date::time_clip(if utc {
            changed
        } else {
            self.date_utc(changed)?
        });
        self.objects[id].date_value = Some(value);
        Ok(value)
    }

    pub(super) fn date_format(&mut self, time: f64, method: &str) -> Result<Value> {
        self.work(256)?;
        // Every supported output is ASCII and under 128 bytes. Cover the UTF-8
        // buffer, bounded formatting temporaries and UTF-16 conversion together.
        self.charge(2048)?;
        if time.is_nan() {
            return Ok(Value::String("Invalid Date".into()));
        }
        let utc = method == "toUTCString";
        let offset = if utc {
            0
        } else {
            self.date_offset(time as i64)?
        };
        let p = js_date::parts((time + f64::from(offset) * 1000.0) as i64);
        let mut text = String::new();
        text.try_reserve_exact(128)
            .map_err(|_| ScriptError::resource("Date string allocation failed"))?;
        if utc {
            write!(
                text,
                "{}, {:02} {} ",
                WEEKDAYS[usize::from(p.weekday)],
                p.day,
                MONTHS[usize::from(p.month)]
            )
            .unwrap();
            write_year(&mut text, p.year);
            write!(text, " {:02}:{:02}:{:02} GMT", p.hour, p.minute, p.second).unwrap();
        } else {
            if !matches!(method, "toTimeString" | "toLocaleTimeString") {
                write_date(&mut text, &p);
            }
            if !matches!(method, "toDateString" | "toLocaleDateString") {
                if !text.is_empty() {
                    text.push(' ');
                }
                let sign = if offset < 0 { '-' } else { '+' };
                let seconds = offset.unsigned_abs();
                write!(
                    text,
                    "{:02}:{:02}:{:02} GMT{}{:02}{:02}",
                    p.hour,
                    p.minute,
                    p.second,
                    sign,
                    (seconds / 3600) % 24,
                    (seconds / 60) % 60
                )
                .unwrap();
                // The optional zone name preserves second-precision historical
                // offsets for parsing our own strings without losing an instant.
                write!(
                    text,
                    " (UTC{}{:02}:{:02}:{:02})",
                    sign,
                    seconds / 3600,
                    (seconds / 60) % 60,
                    seconds % 60
                )
                .unwrap();
            }
        }
        Ok(Value::String(text.into()))
    }

    fn date_parse(&mut self, text: &JsString) -> Result<f64> {
        // Include the fixed-size calendar arithmetic as well as the UTF-16 scan.
        self.work(256 + text.len())?;
        match js_date::parse_iso(text.units()) {
            IsoParse::Value { local_ms, is_local } => Ok(js_date::time_clip(if is_local {
                self.date_utc(local_ms)?
            } else {
                local_ms
            })),
            IsoParse::Invalid => Ok(f64::NAN),
            IsoParse::Unrecognized => {
                // Required own-format round trips plus explicitly documented
                // ASCII surrounding whitespace. No lossy UTF-16 conversion.
                let units = text.trimmed_units();
                if units.len() > 128 || units.iter().any(|u| *u > 0x7f) {
                    return Ok(f64::NAN);
                }
                self.charge(256)?;
                let mut ascii = [0u8; 128];
                for (dest, unit) in ascii.iter_mut().zip(units) {
                    *dest = *unit as u8;
                }
                let source = std::str::from_utf8(&ascii[..units.len()]).unwrap();
                let Some((year, month, day, hour, minute, second, offset)) = parse_own_date(source)
                else {
                    return Ok(f64::NAN);
                };
                let days = self.date_make_day(f64::from(year), f64::from(month), f64::from(day))?;
                self.work(32)?;
                Ok(js_date::time_clip(
                    js_date::make_date(
                        days,
                        js_date::make_time(
                            f64::from(hour),
                            f64::from(minute),
                            f64::from(second),
                            0.0,
                        ),
                    ) - f64::from(offset) * 1000.0,
                ))
            }
        }
    }
}

fn write_year(text: &mut String, year: i32) {
    if year < 0 {
        text.push('-');
    }
    write!(text, "{:04}", year.unsigned_abs()).unwrap();
}
fn write_date(text: &mut String, p: &DateParts) {
    write!(
        text,
        "{} {} {:02} ",
        WEEKDAYS[usize::from(p.weekday)],
        MONTHS[usize::from(p.month)],
        p.day
    )
    .unwrap();
    write_year(text, p.year);
}

// This accepts exactly our DateString/TimeString forms (and the UTC form),
// including the numeric UTC designation used to preserve historical seconds.
fn parse_own_date(text: &str) -> Option<(i32, u8, u8, u8, u8, u8, i32)> {
    if !text.is_ascii() {
        return None;
    }
    fn digits(text: &str) -> Option<u32> {
        if text.is_empty() || text.len() > 7 || !text.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        text.parse().ok()
    }
    let mut words = text.split(' ');
    let weekday = words.next()?;
    let utc = weekday.ends_with(',');
    let weekday = weekday.strip_suffix(',').unwrap_or(weekday);
    if !WEEKDAYS.contains(&weekday) {
        return None;
    }
    let a = words.next()?;
    let b = words.next()?;
    let (month, day) = if utc { (b, a) } else { (a, b) };
    let month = MONTHS.iter().position(|name| *name == month)? as u8;
    let day = u8::try_from(digits(day)?).ok()?;
    if !(1..=31).contains(&day) {
        return None;
    }
    let year = words.next()?;
    let (negative, magnitude) = if let Some(rest) = year.strip_prefix('-') {
        (true, rest)
    } else {
        (false, year)
    };
    if magnitude.len() < 4 {
        return None;
    }
    let year = i32::try_from(digits(magnitude)?).ok()? * if negative { -1 } else { 1 };
    let clock = words.next()?;
    let bytes = clock.as_bytes();
    if bytes.len() != 8 || bytes[2] != b':' || bytes[5] != b':' {
        return None;
    }
    let hour = u8::try_from(digits(&clock[..2])?).ok()?;
    let minute = u8::try_from(digits(&clock[3..5])?).ok()?;
    let second = u8::try_from(digits(&clock[6..])?).ok()?;
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let zone = words.next()?;
    if utc {
        if zone != "GMT" || words.next().is_some() {
            return None;
        }
        return Some((year, month, day, hour, minute, second, 0));
    }
    let zone = zone.strip_prefix("GMT")?;
    if zone.len() != 5 {
        return None;
    }
    let sign = match zone.as_bytes()[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hours = digits(&zone[1..3])?;
    let minutes = digits(&zone[3..5])?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    let mut offset = i32::try_from(hours * 3600 + minutes * 60).ok()? * sign;
    if let Some(exact) = words.next() {
        let exact = exact.strip_prefix("(UTC")?.strip_suffix(')')?;
        let exact_sign = match exact.as_bytes().first()? {
            b'+' => 1,
            b'-' => -1,
            _ => return None,
        };
        let mut parts = exact[1..].split(':');
        let exact_hours = digits(parts.next()?)?;
        let exact_minutes = digits(parts.next()?)?;
        let exact_seconds = digits(parts.next()?)?;
        if parts.next().is_some()
            || exact_hours % 24 != hours
            || exact_minutes != minutes
            || exact_seconds > 59
            || exact_sign != sign
        {
            return None;
        }
        let total = exact_hours
            .checked_mul(3600)?
            .checked_add(exact_minutes * 60)?
            .checked_add(exact_seconds)?;
        offset = i32::try_from(total).ok()? * sign;
    }
    if words.next().is_some() {
        return None;
    }
    Some((year, month, day, hour, minute, second, offset))
}
