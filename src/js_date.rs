//! Pure ECMAScript Date arithmetic and the required date-time interchange format.
//!
//! Calendar arithmetic is proleptic Gregorian. Number operations in MakeTime,
//! MakeDate and MakeDay deliberately retain their specified rounding order.
//! https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-makeday

#[path = "js_date/time_zone.rs"]
pub(crate) mod time_zone;

pub(crate) const MAX_TIME: f64 = 8.64e15;
const DAY_MS: i64 = 86_400_000;
const FAST_WORK: usize = 128;
// Includes all 18-word scans/divisions, fixed stack copies and IEEE rounding
// checks. Each path has fewer than 200 fixed-size helper operations; no loop
// is wider than LIMBS. Keep this conservative precharge before the wide path.
const WIDE_WORK: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DateParts {
    pub(crate) year: i32,
    pub(crate) month: u8,
    pub(crate) day: u8,
    pub(crate) hour: u8,
    pub(crate) minute: u8,
    pub(crate) second: u8,
    pub(crate) millisecond: u16,
    pub(crate) weekday: u8,
}

pub(crate) fn time_clip(time: f64) -> f64 {
    if !time.is_finite() || time.abs() > MAX_TIME {
        return f64::NAN;
    }
    // ToIntegerOrInfinity maps both signed zeroes (and truncated fractions) to 0.
    let value = time.trunc();
    if value == 0.0 { 0.0 } else { value }
}

pub(crate) fn full_year(year: f64) -> f64 {
    if !year.is_finite() {
        return f64::NAN;
    }
    let year = year.trunc();
    if (0.0..=99.0).contains(&year) {
        1900.0 + year
    } else {
        year
    }
}

pub(crate) fn make_time(hour: f64, minute: f64, second: f64, millisecond: f64) -> f64 {
    if !hour.is_finite() || !minute.is_finite() || !second.is_finite() || !millisecond.is_finite() {
        return f64::NAN;
    }
    // Do not reassociate these operations or replace them with mul_add.
    ((integer(hour) * 3_600_000.0 + integer(minute) * 60_000.0) + integer(second) * 1_000.0)
        + integer(millisecond)
}

fn integer(value: f64) -> f64 {
    let value = value.trunc();
    if value == 0.0 { 0.0 } else { value }
}

pub(crate) fn make_date(day: f64, time: f64) -> f64 {
    if !day.is_finite() || !time.is_finite() {
        return f64::NAN;
    }
    let value = day * DAY_MS as f64 + time;
    if value.is_finite() { value } else { f64::NAN }
}

/// Precharge this before make_day. Classification uses no wide integer work.
/// Conservative bounds also cover the constant-size ordinary calendar path.
pub(crate) fn make_day_work(year: f64, month: f64, day: f64) -> usize {
    if !year.is_finite() || !month.is_finite() || !day.is_finite() || fast_day_inputs(year, month) {
        FAST_WORK
    } else {
        WIDE_WORK
    }
}

fn fast_day_inputs(year: f64, month: f64) -> bool {
    let Some((quotient, _)) = ordinary_month(month.trunc()) else {
        return false;
    };
    let balanced = year.trunc() + quotient;
    balanced >= i32::MIN as f64 && balanced <= i32::MAX as f64
}

fn ordinary_month(month: f64) -> Option<(f64, u8)> {
    // The upper f64 bound is 2^63, not i64::MAX (which rounds to 2^63).
    if month < i64::MIN as f64 || month >= -(i64::MIN as f64) {
        return None;
    }
    let value = month as i64;
    Some((value.div_euclid(12) as f64, value.rem_euclid(12) as u8))
}

pub(crate) fn make_day(year: f64, month: f64, day: f64) -> f64 {
    if !year.is_finite() || !month.is_finite() || !day.is_finite() {
        return f64::NAN;
    }
    let year = year.trunc();
    let month = month.trunc();
    let day = day.trunc();
    let (balanced_year, balanced_month) = if let Some((quotient, remainder)) = ordinary_month(month)
    {
        (year + quotient, remainder)
    } else {
        // floor is on the mathematical integer, not on a rounded f64 division.
        let (quotient, remainder) = Wide::from_float(month).div_floor(12);
        (year + quotient.to_float(), remainder as u8)
    };
    if !balanced_year.is_finite() {
        return f64::NAN;
    }
    let first_day = if balanced_year >= i32::MIN as f64 && balanced_year <= i32::MAX as f64 {
        // Even at an i32 year the timestamp ULP is <=8192ms, much less than a
        // day. A finite timestamp on the month's first day always exists.
        days_from_civil(balanced_year as i32, balanced_month, 1) as f64
    } else {
        let year = Wide::from_float(balanced_year);
        let first_day = wide_day_from_year(year)
            .add(Wide::small(
                month_start(wide_leap(year), balanced_month) as i64
            ));
        // Step 8 requires an actual finite Number in that calendar day. The
        // rounded start of this/next year also bounds YearFromTime's inverse.
        let start = wide_day_from_year(year).mul(86_400_000).to_float();
        let end = wide_day_from_year(year.add(Wide::small(1)))
            .mul(86_400_000)
            .to_float();
        let lower = first_day.mul(86_400_000);
        let upper = first_day.add(Wide::small(1)).mul(86_400_000);
        let mut candidate = lower.to_float();
        if !candidate.is_finite() {
            return f64::NAN;
        }
        if Wide::from_float(candidate).cmp(&lower).is_lt() {
            candidate = next_up(candidate);
        }
        candidate = candidate.max(start);
        if !candidate.is_finite()
            || candidate >= end
            || Wide::from_float(candidate).cmp(&upper).is_ge()
        {
            return f64::NAN;
        }
        first_day.to_float()
    };
    // This addition and subtraction are Number operations, in this order.
    (first_day + day) - 1.0
}

fn next_up(value: f64) -> f64 {
    if value == 0.0 {
        f64::from_bits(1)
    } else if value < 0.0 {
        f64::from_bits(value.to_bits() - 1)
    } else {
        f64::from_bits(value.to_bits() + 1)
    }
}

fn leap(year: i64) -> bool {
    year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
}

fn month_start(is_leap: bool, month: u8) -> u16 {
    const DAYS: [u16; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    DAYS[month as usize] + u16::from(is_leap && month >= 2)
}

/// Month is zero-based, day is one-based. Inputs are validated by the caller.
pub(crate) fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let year = i64::from(year);
    365 * (year - 1970) + (year - 1969).div_euclid(4) - (year - 1901).div_euclid(100)
        + (year - 1601).div_euclid(400)
        + i64::from(month_start(leap(year), month))
        + i64::from(day)
        - 1
}

pub(crate) fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 => {
            if leap(i64::from(year)) {
                29
            } else {
                28
            }
        }
        3 | 5 | 8 | 10 => 30,
        _ => 31,
    }
}

/// Defined for every i64 millisecond count, including margins beyond TimeClip.
/// Its largest year magnitude is below 293 million, hence fits i32.
pub(crate) fn parts(time_ms: i64) -> DateParts {
    let day = time_ms.div_euclid(DAY_MS);
    let within_day = time_ms.rem_euclid(DAY_MS);
    // March-based 400-year eras; all intermediates stay within i64.
    let shifted = day + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let march_month = (5 * day_of_year + 2) / 153;
    let date = day_of_year - (153 * march_month + 2) / 5 + 1;
    let month = march_month + if march_month < 10 { 2 } else { -10 };
    year += i64::from(month < 2);
    DateParts {
        year: year as i32,
        month: month as u8,
        day: date as u8,
        hour: (within_day / 3_600_000) as u8,
        minute: (within_day / 60_000 % 60) as u8,
        second: (within_day / 1000 % 60) as u8,
        millisecond: (within_day % 1000) as u16,
        weekday: (day + 4).rem_euclid(7) as u8,
    }
}

// A finite f64 integer has <=1024 bits. Calendar scaling by 366 days/year
// and 86,400,000ms/day adds <36 bits. Eighteen limbs (1152 bits) therefore
// cover all intermediates, including negative floors and the year+1 boundary.
// No heap storage, recursion, or input-sized loops are used here.
const LIMBS: usize = 18;
#[derive(Clone, Copy)]
struct Wide {
    words: [u64; LIMBS],
    negative: bool,
}

impl Wide {
    fn small(value: i64) -> Self {
        let mut result = Self {
            words: [0; LIMBS],
            negative: value < 0,
        };
        result.words[0] = value.unsigned_abs();
        result
    }

    fn from_float(value: f64) -> Self {
        debug_assert!(value.is_finite() && value.fract() == 0.0);
        let bits = value.to_bits();
        let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
        if exponent < 0 {
            return Self::small(0);
        }
        let significand = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
        let mut result = Self::small(0);
        result.negative = value < 0.0;
        if exponent < 52 {
            result.words[0] = significand >> (52 - exponent);
        } else {
            let shift = (exponent - 52) as usize;
            result.words[shift / 64] = significand << (shift % 64);
            if !shift.is_multiple_of(64) {
                result.words[shift / 64 + 1] = significand >> (64 - shift % 64);
            }
        }
        result
    }

    fn is_zero(self) -> bool {
        self.words.iter().all(|&v| v == 0)
    }

    fn abs_cmp(&self, other: &Self) -> std::cmp::Ordering {
        for i in (0..LIMBS).rev() {
            let order = self.words[i].cmp(&other.words[i]);
            if !order.is_eq() {
                return order;
            }
        }
        std::cmp::Ordering::Equal
    }

    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.negative != other.negative {
            return other.negative.cmp(&self.negative);
        }
        let order = self.abs_cmp(other);
        if self.negative {
            order.reverse()
        } else {
            order
        }
    }

    fn neg(mut self) -> Self {
        if !self.is_zero() {
            self.negative = !self.negative;
        }
        self
    }

    fn add(self, other: Self) -> Self {
        let mut result = self;
        if self.negative == other.negative {
            let mut carry = 0_u128;
            for (out, &rhs) in result.words.iter_mut().zip(other.words.iter()) {
                let sum = u128::from(*out) + u128::from(rhs) + carry;
                *out = sum as u64;
                carry = sum >> 64;
            }
            debug_assert_eq!(carry, 0);
        } else {
            let (larger, smaller) = if self.abs_cmp(&other).is_lt() {
                (other, self)
            } else {
                (self, other)
            };
            result = larger;
            let mut borrow = false;
            for (out, &rhs) in result.words.iter_mut().zip(smaller.words.iter()) {
                let (a, b1) = out.overflowing_sub(rhs);
                let (b, b2) = a.overflowing_sub(u64::from(borrow));
                *out = b;
                borrow = b1 || b2;
            }
            debug_assert!(!borrow);
            if result.is_zero() {
                result.negative = false;
            }
        }
        result
    }

    fn mul(mut self, multiplier: u32) -> Self {
        let mut carry = 0_u128;
        for out in &mut self.words {
            let product = u128::from(*out) * u128::from(multiplier) + carry;
            *out = product as u64;
            carry = product >> 64;
        }
        debug_assert_eq!(carry, 0);
        self
    }

    fn div_floor(self, divisor: u32) -> (Self, u32) {
        let mut quotient = self;
        let mut remainder = 0_u128;
        for out in quotient.words.iter_mut().rev() {
            let value = (remainder << 64) | u128::from(*out);
            *out = (value / u128::from(divisor)) as u64;
            remainder = value % u128::from(divisor);
        }
        if quotient.is_zero() {
            quotient.negative = false;
        }
        if self.negative && remainder != 0 {
            quotient = quotient.add(Self::small(-1));
            remainder = u128::from(divisor) - remainder;
        }
        (quotient, remainder as u32)
    }

    fn bit(self, index: usize) -> bool {
        self.words[index / 64] & (1 << (index % 64)) != 0
    }

    fn to_float(self) -> f64 {
        let Some(top) = self.words.iter().rposition(|&w| w != 0) else {
            return 0.0;
        };
        let mut exponent = top * 64 + (63 - self.words[top].leading_zeros() as usize);
        if exponent > 1023 {
            return if self.negative {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            };
        }
        if exponent <= 52 {
            let value = self.words[0] as f64;
            return if self.negative { -value } else { value };
        }
        let shift = exponent - 52;
        let index = shift / 64;
        let mut significand = self.words[index] >> (shift % 64);
        if !shift.is_multiple_of(64) {
            significand |= self.words[index + 1] << (64 - shift % 64);
        }
        let guard = self.bit(shift - 1);
        let low = shift - 1;
        let sticky = self.words[..low / 64].iter().any(|&w| w != 0)
            || self.words[low / 64] & ((1_u64 << (low % 64)) - 1) != 0;
        if guard && (sticky || significand & 1 != 0) {
            significand += 1;
            if significand == 1 << 53 {
                significand >>= 1;
                exponent += 1;
            }
        }
        let bits = if exponent > 1023 {
            0x7ff0_0000_0000_0000
        } else {
            ((exponent as u64 + 1023) << 52) | (significand & ((1 << 52) - 1))
        };
        f64::from_bits(bits | (u64::from(self.negative) << 63))
    }
}

fn wide_day_from_year(year: Wide) -> Wide {
    year.add(Wide::small(-1970))
        .mul(365)
        .add(year.add(Wide::small(-1969)).div_floor(4).0)
        .add(year.add(Wide::small(-1901)).div_floor(100).0.neg())
        .add(year.add(Wide::small(-1601)).div_floor(400).0)
}

fn wide_leap(year: Wide) -> bool {
    year.div_floor(4).1 == 0 && (year.div_floor(100).1 != 0 || year.div_floor(400).1 == 0)
}

#[cfg(test)]
#[path = "js_date/tests.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum IsoParse {
    Invalid,
    Unrecognized,
    /// Explicit offsets have already been subtracted. Only an absent-offset
    /// date-time needs local-zone conversion; date-only forms are UTC.
    Value {
        local_ms: f64,
        is_local: bool,
    },
}

/// Allocation-free required-format parser over original UTF-16 code units.
/// Caller precharges input work. At most 32 units are consumed; no legacy
/// heuristics, Unicode replacement, whitespace trimming or timezone lookup.
pub(crate) fn parse_iso(input: &[u16]) -> IsoParse {
    let Some(&first) = input.first() else {
        return IsoParse::Unrecognized;
    };
    let signed = matches!(first, 43 | 45);
    let mut parser = IsoCursor {
        input,
        at: usize::from(signed),
    };
    let Some(mut year) = parser.digits(if signed { 6 } else { 4 }) else {
        return if signed {
            IsoParse::Invalid
        } else {
            IsoParse::Unrecognized
        };
    };
    if !signed && !matches!(parser.peek(), None | Some(45 | 84)) {
        return IsoParse::Unrecognized;
    }
    if first == 45 {
        if year == 0 {
            return IsoParse::Invalid;
        }
        year = -year;
    }
    if input.len() > 32 {
        return IsoParse::Invalid;
    }
    let mut month = 1;
    let mut day = 1;
    if parser.eat(b'-') {
        let Some(value) = parser.digits(2) else {
            return IsoParse::Invalid;
        };
        month = value;
        if parser.eat(b'-') {
            let Some(value) = parser.digits(2) else {
                return IsoParse::Invalid;
            };
            day = value;
        }
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return IsoParse::Invalid;
    }
    // The required format bounds DD to 01..31; calendar balancing uses MakeDay.
    let base = days_from_civil(year, (month - 1) as u8, day as u8) * DAY_MS;
    if parser.peek().is_none() {
        return iso_value(base, false);
    }
    if !parser.eat(b'T') {
        return IsoParse::Invalid;
    }
    let Some(hour) = parser.digits(2) else {
        return IsoParse::Invalid;
    };
    if !parser.eat(b':') {
        return IsoParse::Invalid;
    }
    let Some(minute) = parser.digits(2) else {
        return IsoParse::Invalid;
    };
    let mut second = 0;
    let mut millisecond = 0;
    if parser.eat(b':') {
        let Some(value) = parser.digits(2) else {
            return IsoParse::Invalid;
        };
        second = value;
        if parser.eat(b'.') {
            let Some(value) = parser.digits(3) else {
                return IsoParse::Invalid;
            };
            millisecond = value;
        }
    }
    if hour > 24
        || minute > 59
        || second > 59
        || (hour == 24 && (minute != 0 || second != 0 || millisecond != 0))
    {
        return IsoParse::Invalid;
    }
    let mut time = base
        + i64::from(hour) * 3_600_000
        + i64::from(minute) * 60_000
        + i64::from(second) * 1000
        + i64::from(millisecond);
    let is_local = if parser.eat(b'Z') {
        false
    } else if matches!(parser.peek(), Some(43 | 45)) {
        let negative = parser.eat(b'-');
        if !negative {
            parser.eat(b'+');
        }
        let Some(hour) = parser.digits(2) else {
            return IsoParse::Invalid;
        };
        if !parser.eat(b':') {
            return IsoParse::Invalid;
        }
        let Some(minute) = parser.digits(2) else {
            return IsoParse::Invalid;
        };
        if hour > 23 || minute > 59 {
            return IsoParse::Invalid;
        }
        let offset = i64::from(hour * 60 + minute) * 60_000;
        time += if negative { offset } else { -offset };
        false
    } else {
        true
    };
    if parser.peek().is_some() {
        return IsoParse::Invalid;
    }
    iso_value(time, is_local)
}

fn iso_value(time: i64, is_local: bool) -> IsoParse {
    if !is_local && time.unsigned_abs() > MAX_TIME as u64 {
        IsoParse::Invalid
    } else {
        IsoParse::Value {
            local_ms: time as f64,
            is_local,
        }
    }
}

struct IsoCursor<'a> {
    input: &'a [u16],
    at: usize,
}
impl IsoCursor<'_> {
    fn peek(&self) -> Option<u16> {
        self.input.get(self.at).copied()
    }
    fn eat(&mut self, value: u8) -> bool {
        if self.peek() == Some(u16::from(value)) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn digits(&mut self, count: usize) -> Option<i32> {
        let mut result = 0;
        for _ in 0..count {
            let value = self.peek()?;
            if !(48..=57).contains(&value) {
                return None;
            }
            result = result * 10 + i32::from(value - 48);
            self.at += 1;
        }
        Some(result)
    }
}

/// Input must be an integral, valid clipped instant. Output has 24 or 27 ASCII
/// bytes; caller precharges bounded String storage. No host/locale operations.
pub(crate) fn format_iso(time_ms: i64) -> String {
    use std::fmt::Write;
    debug_assert!(time_ms.unsigned_abs() <= MAX_TIME as u64);
    let p = parts(time_ms);
    let mut output = String::with_capacity(27);
    if (0..=9999).contains(&p.year) {
        let _ = write!(output, "{:04}", p.year);
    } else {
        let _ = write!(
            output,
            "{}{:06}",
            if p.year < 0 { '-' } else { '+' },
            p.year.unsigned_abs()
        );
    }
    let _ = write!(
        output,
        "-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        p.month + 1,
        p.day,
        p.hour,
        p.minute,
        p.second,
        p.millisecond
    );
    output
}
