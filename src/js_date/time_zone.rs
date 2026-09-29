//! Bounded, immutable TZif/POSIX rules. RFC 9636 §§3–3.3 and ECMA-262 UTC(t).
//! No environment, filesystem, clock, or mutable cache is consulted here.
use super::{days_from_civil, days_in_month, parts};
use std::{fmt, sync::Arc};

pub const MAX_ZONE_SOURCE_BYTES: usize = 1_048_576;
pub const MAX_POSIX_BYTES: usize = 1024;
const MAX_TRANSITIONS: usize = 65_536;
const MAX_DESIGNATIONS: usize = 65_536;
const MIN_OFFSET: i32 = -89_999;
const MAX_OFFSET: i32 = 93_599;
pub const MAX_ABS_OFFSET_SECONDS: i32 = MAX_OFFSET;
// Also admits local inputs just outside TimeClip which become valid after UTC conversion.
pub const MAX_ZONE_MILLIS: i64 = 8_640_000_000_000_000 + 8 * 86_400_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZonePayloadKind {
    Tzif,
    Posix2024,
    ExplicitUtc,
}
#[derive(Clone, Debug)]
pub struct ZonePayload {
    pub kind: ZonePayloadKind,
    pub bytes: Arc<[u8]>,
}
impl ZonePayload {
    pub fn new(kind: ZonePayloadKind, bytes: &[u8]) -> Result<Self, ZoneError> {
        validate_payload_size(kind, bytes.len())?;
        Ok(Self {
            kind,
            bytes: Arc::from(bytes),
        })
    }
}
fn validate_payload_size(kind: ZonePayloadKind, len: usize) -> Result<(), ZoneError> {
    let max = match kind {
        ZonePayloadKind::Tzif => MAX_ZONE_SOURCE_BYTES,
        ZonePayloadKind::Posix2024 => MAX_POSIX_BYTES,
        ZonePayloadKind::ExplicitUtc => 0,
    };
    if len > max {
        Err(ZoneError::TooLarge)
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneError {
    TooLarge,
    Allocation,
    WorkLimit,
    Malformed,
    UnsupportedVersion,
    LeapAware,
    IncompleteCoverage,
    UnsupportedOffset,
    MissingRules,
    OutOfRange,
}
impl ZoneError {
    pub fn is_work_limit(self) -> bool {
        self == Self::WorkLimit
    }
}
impl fmt::Display for ZoneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "timezone {self:?}")
    }
}
impl std::error::Error for ZoneError {}
#[derive(Debug)]
pub struct ZoneBudget {
    pub work_left: usize,
    pub allocation_left: usize,
}
impl Default for ZoneBudget {
    fn default() -> Self {
        Self {
            work_left: 8_388_608,
            allocation_left: 4_194_304,
        }
    }
}
fn work(left: &mut usize, amount: usize) -> Result<(), ZoneError> {
    if amount > *left {
        *left = 0;
        return Err(ZoneError::WorkLimit);
    }
    *left -= amount;
    Ok(())
}
impl ZoneBudget {
    fn alloc(&mut self, bytes: usize) -> Result<(), ZoneError> {
        self.allocation_left = self
            .allocation_left
            .checked_sub(bytes)
            .ok_or(ZoneError::Allocation)?;
        Ok(())
    }
    fn vector<T>(&mut self, count: usize) -> Result<Vec<T>, ZoneError> {
        self.alloc(
            count
                .checked_mul(std::mem::size_of::<T>())
                .ok_or(ZoneError::TooLarge)?,
        )?;
        let mut v = Vec::new();
        v.try_reserve_exact(count)
            .map_err(|_| ZoneError::Allocation)?;
        Ok(v)
    }
}
#[derive(Clone, Copy, Debug)]
struct TimeType {
    offset: i32,
    dst: bool,
    designation: u8,
    designation_end: u16,
}
#[derive(Clone, Copy, Debug)]
struct Transition {
    second: i64,
    kind: u8,
}
#[derive(Debug)]
pub struct TimeZoneSnapshot {
    transitions: Vec<Transition>,
    types: Vec<TimeType>,
    offsets: Vec<i32>,
    // Ends of finite local intervals; searched independently of UTC ordering.
    local_ends: Vec<i64>,
    tail: Option<Posix>,
}
impl TimeZoneSnapshot {
    pub fn parse(payload: &ZonePayload, budget: &mut ZoneBudget) -> Result<Self, ZoneError> {
        validate_payload_size(payload.kind, payload.bytes.len())?;
        work(
            &mut budget.work_left,
            payload.bytes.len().saturating_mul(3).saturating_add(1),
        )?;
        match payload.kind {
            ZonePayloadKind::ExplicitUtc => Self::fixed(0, budget),
            ZonePayloadKind::Posix2024 => {
                let tail = Posix::parse(&payload.bytes, true, budget)?;
                let mut result = Self::fixed(tail.standard, budget)?;
                if let Some(dst) = tail.daylight {
                    result.offsets.push(dst);
                }
                result.tail = Some(tail);
                Ok(result)
            }
            ZonePayloadKind::Tzif => Self::tzif(&payload.bytes, budget),
        }
    }
    fn fixed(offset: i32, budget: &mut ZoneBudget) -> Result<Self, ZoneError> {
        let mut types = budget.vector(1)?;
        types.push(TimeType {
            offset,
            dst: false,
            designation: 0,
            designation_end: 0,
        });
        let mut offsets = budget.vector(2)?;
        offsets.push(offset);
        Ok(Self {
            transitions: Vec::new(),
            types,
            offsets,
            local_ends: Vec::new(),
            tail: None,
        })
    }
    fn tzif(bytes: &[u8], budget: &mut ZoneBudget) -> Result<Self, ZoneError> {
        let mut cursor = Cursor { bytes, at: 0 };
        let header = Header::read(&mut cursor)?;
        let first = Block::read(&mut cursor, header, 4, budget)?;
        let (block, tail) = if header.version == 0 {
            if cursor.at != bytes.len() {
                return Err(ZoneError::Malformed);
            }
            (first, None)
        } else {
            drop(first);
            let second = Header::read(&mut cursor)?;
            if second.version != header.version {
                return Err(ZoneError::Malformed);
            }
            let block = Block::read(&mut cursor, second, 8, budget)?;
            let footer = cursor.take(bytes.len() - cursor.at)?;
            if footer.len() < 2 || footer[0] != b'\n' || footer[footer.len() - 1] != b'\n' {
                return Err(ZoneError::Malformed);
            }
            let body = &footer[1..footer.len() - 1];
            let tail = if body.is_empty() {
                None
            } else {
                Some(Posix::parse(body, header.version >= b'3', budget)?)
            };
            (block, tail)
        };
        if !block.transitions.is_empty() && tail.is_none() {
            return Err(ZoneError::IncompleteCoverage);
        }
        // Only reachable types must provide local time; unused -00 records are harmless.
        if (!block.transitions.is_empty() || tail.is_none()) && block.name(0)? == b"-00" {
            return Err(ZoneError::IncompleteCoverage);
        }
        for tr in &block.transitions {
            if block.name(tr.kind as usize)? == b"-00" {
                return Err(ZoneError::IncompleteCoverage);
            }
        }
        if let (Some(last), Some(rule)) = (block.transitions.last(), &tail) {
            // Calendar arithmetic is bounded by signed-i64 milliseconds. Outlandish TZif
            // tail anchors beyond that domain are an explicit unsupported host input.
            let ms = last.second.checked_mul(1000).ok_or(ZoneError::OutOfRange)?;
            let (offset, dst) = rule.state(ms, &mut budget.work_left)?;
            let ty = block.types[last.kind as usize];
            if offset != ty.offset
                || dst != ty.dst
                || rule.name(dst) != block.name(last.kind as usize)?
            {
                return Err(ZoneError::Malformed);
            }
        }
        let mut offsets = budget.vector(block.types.len() + 2)?;
        for ty in &block.types {
            offsets.push(ty.offset);
        }
        if let Some(rule) = &tail {
            offsets.push(rule.standard);
            if let Some(d) = rule.daylight {
                offsets.push(d);
            }
        }
        work(&mut budget.work_left, offsets.len() * 10)?;
        offsets.sort_unstable();
        offsets.dedup();
        let mut local_ends = budget.vector(block.transitions.len())?;
        let mut before = block.types[0].offset;
        for tr in &block.transitions {
            let end = i128::from(tr.second) * 1000 + i128::from(before) * 1000;
            if let Ok(end) = i64::try_from(end) {
                local_ends.push(end);
            }
            before = block.types[tr.kind as usize].offset;
        }
        let sort_work = local_ends
            .len()
            .saturating_mul(2 * (usize::BITS - local_ends.len().leading_zeros()) as usize + 1);
        work(&mut budget.work_left, sort_work)?;
        local_ends.sort_unstable();
        local_ends.dedup();
        Ok(Self {
            transitions: block.transitions,
            types: block.types,
            offsets,
            local_ends,
            tail,
        })
    }
    pub fn offset_at_utc_ms(&self, epoch_ms: i64, left: &mut usize) -> Result<i32, ZoneError> {
        work(left, 1)?;
        if epoch_ms.unsigned_abs() > MAX_ZONE_MILLIS as u64 {
            return Err(ZoneError::OutOfRange);
        }
        self.offset_unchecked(epoch_ms, left)
    }
    fn offset_unchecked(&self, ms: i64, left: &mut usize) -> Result<i32, ZoneError> {
        let second = ms.div_euclid(1000);
        if let Some(tail) = &self.tail
            && self.transitions.last().is_none_or(|t| second >= t.second)
        {
            return Ok(tail.state(ms, left)?.0);
        }
        let (mut low, mut high) = (0, self.transitions.len());
        while low < high {
            work(left, 1)?;
            let mid = low + (high - low) / 2;
            if self.transitions[mid].second <= second {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        work(left, 1)?;
        let kind = if low == 0 {
            0
        } else {
            self.transitions[low - 1].kind as usize
        };
        Ok(self.types[kind].offset)
    }
    fn candidates(&self, local: i64, left: &mut usize) -> Result<Option<(i64, i64)>, ZoneError> {
        let mut found: Option<(i64, i64)> = None;
        for &offset in &self.offsets {
            work(left, 1)?;
            let utc = local
                .checked_sub(i64::from(offset) * 1000)
                .ok_or(ZoneError::OutOfRange)?;
            if self.offset_unchecked(utc, left)? == offset {
                found = Some(match found {
                    None => (utc, utc),
                    Some((a, b)) => (a.min(utc), b.max(utc)),
                });
            }
        }
        Ok(found)
    }
    pub fn utc_from_local_ms(&self, local: i64, left: &mut usize) -> Result<i64, ZoneError> {
        work(left, 1)?;
        if local.unsigned_abs() > MAX_ZONE_MILLIS as u64 {
            return Err(ZoneError::OutOfRange);
        }
        if let Some((earliest, _)) = self.candidates(local, left)? {
            return Ok(earliest);
        }
        // Outside the union of local intervals, its last preceding endpoint is
        // necessarily the end of an actually reachable interval. Overlapping folds
        // do not require a guessed DST flag or a two-candidate assumption.
        let (mut low, mut high) = (0, self.local_ends.len());
        while low < high {
            work(left, 1)?;
            let m = low + (high - low) / 2;
            if self.local_ends[m] <= local {
                low = m + 1;
            } else {
                high = m;
            }
        }
        let mut end = low.checked_sub(1).map(|i| self.local_ends[i]);
        if let Some(tail) = &self.tail {
            let year = parts(local).year;
            for y in year - 2..=year + 2 {
                for (utc, _, _) in tail.events(y, left)? {
                    if self
                        .transitions
                        .last()
                        .is_some_and(|t| i128::from(utc) < i128::from(t.second) * 1000)
                    {
                        continue;
                    }
                    let before = self
                        .offset_unchecked(utc.checked_sub(1).ok_or(ZoneError::OutOfRange)?, left)?;
                    let candidate = utc
                        .checked_add(i64::from(before) * 1000)
                        .ok_or(ZoneError::OutOfRange)?;
                    if candidate <= local {
                        end = Some(end.map_or(candidate, |e| e.max(candidate)));
                    }
                }
            }
        }
        let preceding = end
            .and_then(|e| e.checked_sub(1))
            .ok_or(ZoneError::IncompleteCoverage)?;
        let (_, latest) = self
            .candidates(preceding, left)?
            .ok_or(ZoneError::IncompleteCoverage)?;
        let offset = self.offset_unchecked(latest, left)?;
        local
            .checked_sub(i64::from(offset) * 1000)
            .ok_or(ZoneError::OutOfRange)
    }
}
struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ZoneError> {
        let end = self.at.checked_add(n).ok_or(ZoneError::TooLarge)?;
        let out = self.bytes.get(self.at..end).ok_or(ZoneError::Malformed)?;
        self.at = end;
        Ok(out)
    }
    fn u32(&mut self) -> Result<usize, ZoneError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().unwrap()) as usize)
    }
}
#[derive(Clone, Copy)]
struct Header {
    version: u8,
    ut: usize,
    standard: usize,
    count: usize,
    types: usize,
    chars: usize,
}
impl Header {
    fn read(c: &mut Cursor<'_>) -> Result<Self, ZoneError> {
        if c.take(4)? != b"TZif" {
            return Err(ZoneError::Malformed);
        }
        let version = c.take(1)?[0];
        if !matches!(version, 0 | b'2' | b'3' | b'4') {
            return Err(ZoneError::UnsupportedVersion);
        }
        if c.take(15)?.iter().any(|&b| b != 0) {
            return Err(ZoneError::Malformed);
        }
        let ut = c.u32()?;
        let standard = c.u32()?;
        let leaps = c.u32()?;
        let count = c.u32()?;
        let types = c.u32()?;
        let chars = c.u32()?;
        if leaps != 0 {
            return Err(ZoneError::LeapAware);
        }
        if count > MAX_TRANSITIONS || types > 256 || chars > MAX_DESIGNATIONS {
            return Err(ZoneError::TooLarge);
        }
        if types == 0
            || chars == 0
            || (ut != 0 && ut != types)
            || (standard != 0 && standard != types)
        {
            return Err(ZoneError::Malformed);
        }
        Ok(Self {
            version,
            ut,
            standard,
            count,
            types,
            chars,
        })
    }
}
struct Block<'a> {
    transitions: Vec<Transition>,
    types: Vec<TimeType>,
    names: &'a [u8],
}
impl<'a> Block<'a> {
    fn read(
        c: &mut Cursor<'a>,
        h: Header,
        width: usize,
        budget: &mut ZoneBudget,
    ) -> Result<Self, ZoneError> {
        let times = c.take(h.count * width)?;
        let indices = c.take(h.count)?;
        let rawtypes = c.take(h.types * 6)?;
        let names = c.take(h.chars)?;
        let std = c.take(h.standard)?;
        let ut = c.take(h.ut)?;
        let mut transitions = budget.vector(h.count)?;
        let mut types = budget.vector(h.types)?;
        for (i, raw) in rawtypes.chunks_exact(6).enumerate() {
            let offset = i32::from_be_bytes(raw[..4].try_into().unwrap());
            if !(MIN_OFFSET..=MAX_OFFSET).contains(&offset) {
                return Err(ZoneError::UnsupportedOffset);
            }
            if raw[4] > 1 || raw[5] as usize >= names.len() {
                return Err(ZoneError::Malformed);
            }
            let name = &names[raw[5] as usize..];
            work(&mut budget.work_left, name.len())?;
            let name_len = name
                .iter()
                .position(|&b| b == 0)
                .ok_or(ZoneError::Malformed)?;
            let designation_end = (raw[5] as usize + name_len) as u16;
            if std.get(i).is_some_and(|&b| b > 1)
                || ut
                    .get(i)
                    .is_some_and(|&b| b > 1 || (b == 1 && std.get(i) != Some(&1)))
            {
                return Err(ZoneError::Malformed);
            }
            types.push(TimeType {
                offset,
                dst: raw[4] == 1,
                designation: raw[5],
                designation_end,
            });
        }
        let mut prev = None;
        for (i, raw) in times.chunks_exact(width).enumerate() {
            let second = if width == 4 {
                i64::from(i32::from_be_bytes(raw.try_into().unwrap()))
            } else {
                i64::from_be_bytes(raw.try_into().unwrap())
            };
            if prev.is_some_and(|p| second <= p) || indices[i] as usize >= types.len() {
                return Err(ZoneError::Malformed);
            }
            transitions.push(Transition {
                second,
                kind: indices[i],
            });
            prev = Some(second);
        }
        Ok(Self {
            transitions,
            types,
            names,
        })
    }
    fn name(&self, index: usize) -> Result<&[u8], ZoneError> {
        let ty = self.types[index];
        // End was found once under Block::read's scan charge. Transition count
        // must never multiply designation scans.
        Ok(&self.names[ty.designation as usize..ty.designation_end as usize])
    }
}
#[derive(Clone, Copy, Debug)]
enum RuleDay {
    Julian(u16),
    Ordinal(u16),
    Month { month: u8, week: u8, weekday: u8 },
}
#[derive(Clone, Copy, Debug)]
struct Rule {
    day: RuleDay,
    seconds: i32,
}
#[derive(Debug)]
struct Posix {
    standard: i32,
    daylight: Option<i32>,
    start: Rule,
    end: Rule,
    names: Vec<u8>,
    split: usize,
    all_year: bool,
    never_dst: bool,
}
impl Posix {
    fn parse(bytes: &[u8], extended: bool, budget: &mut ZoneBudget) -> Result<Self, ZoneError> {
        if bytes.len() > MAX_POSIX_BYTES {
            return Err(ZoneError::TooLarge);
        }
        let mut p = PosixParser {
            bytes,
            at: 0,
            extended,
        };
        let std = p.name()?;
        let standard = -p.time(24, true)?;
        let dummy = Rule {
            day: RuleDay::Ordinal(0),
            seconds: 0,
        };
        let mut result = Self {
            standard,
            daylight: None,
            start: dummy,
            end: dummy,
            names: budget.vector(bytes.len())?,
            split: std.len(),
            all_year: false,
            never_dst: false,
        };
        result.names.extend_from_slice(std);
        if p.at == bytes.len() {
            if result.name(false) == b"-00" {
                return Err(ZoneError::IncompleteCoverage);
            }
            return Ok(result);
        }
        let dst = p.name()?;
        result.names.extend_from_slice(dst);
        let daylight = if p.peek() == Some(b',') || p.peek().is_none() {
            standard + 3600
        } else {
            -p.time(24, true)?
        };
        if !(MIN_OFFSET..=MAX_OFFSET).contains(&daylight) {
            return Err(ZoneError::UnsupportedOffset);
        }
        result.daylight = Some(daylight);
        if p.at == bytes.len() {
            return Err(ZoneError::MissingRules);
        }
        p.byte(b',')?;
        result.start = p.rule()?;
        p.byte(b',')?;
        result.end = p.rule()?;
        if p.at != bytes.len() {
            return Err(ZoneError::Malformed);
        }
        // Compare actual instants over one Gregorian cycle, not one lexical
        // spelling. RFC9636 §3.3.1 / Appendix A includes positive and negative
        // permanent DST; J1 and signed-time normalizations must be equivalent.
        result.all_year = true;
        result.never_dst = true;
        for year in 2000..2400 {
            work(&mut budget.work_left, 48)?;
            let start = result.start.at(year, standard)?;
            result.all_year &= start == result.end.at(year - 1, daylight)?;
            result.never_dst &= start == result.end.at(year, daylight)?;
            if !result.all_year && !result.never_dst {
                break;
            }
        }
        if (!result.all_year && result.name(false) == b"-00")
            || (!result.never_dst && result.name(true) == b"-00")
        {
            return Err(ZoneError::IncompleteCoverage);
        }
        Ok(result)
    }
    fn name(&self, dst: bool) -> &[u8] {
        if dst {
            &self.names[self.split..]
        } else {
            &self.names[..self.split]
        }
    }
    fn events(&self, year: i32, left: &mut usize) -> Result<[(i64, i32, i32); 2], ZoneError> {
        work(left, 24)?;
        let dst = self.daylight.unwrap_or(self.standard);
        Ok([
            (self.start.at(year, self.standard)?, self.standard, dst),
            (self.end.at(year, dst)?, dst, self.standard),
        ])
    }
    fn state(&self, ms: i64, left: &mut usize) -> Result<(i32, bool), ZoneError> {
        work(left, 1)?;
        let Some(dst) = self.daylight else {
            return Ok((self.standard, false));
        };
        if self.all_year {
            return Ok((dst, true));
        }
        if self.never_dst {
            return Ok((self.standard, false));
        }
        let year = parts(ms).year;
        let mut latest = None;
        // Adjacent years cover signed167-hour rules. On a coincident boundary,
        // later nominal-year events win; within one year the end wins. Continuous
        // all-year and empty-DST intervals were established during validation.
        for y in year - 1..=year + 1 {
            for (i, (at, _, after)) in self.events(y, left)?.into_iter().enumerate() {
                if at <= ms && latest.is_none_or(|(old, _, _)| at >= old) {
                    latest = Some((at, after, i == 0));
                }
            }
        }
        let (_, offset, dst) = latest.ok_or(ZoneError::IncompleteCoverage)?;
        Ok((offset, dst))
    }
}
impl Rule {
    fn at(self, year: i32, before: i32) -> Result<i64, ZoneError> {
        let jan = days_from_civil(year, 0, 1);
        let day = match self.day {
            RuleDay::Ordinal(n) => jan + i64::from(n),
            RuleDay::Julian(n) => {
                jan + i64::from(n) - 1 + i64::from(n >= 60 && days_in_month(year, 1) == 29)
            }
            RuleDay::Month {
                month,
                week,
                weekday,
            } => {
                let first = days_from_civil(year, month - 1, 1);
                let first_weekday = (first + 4).rem_euclid(7);
                let mut d = 1
                    + (i64::from(weekday) - first_weekday).rem_euclid(7)
                    + 7 * (i64::from(week) - 1);
                if d > i64::from(days_in_month(year, month - 1)) {
                    d -= 7;
                }
                first + d - 1
            }
        };
        day.checked_mul(86_400_000)
            .and_then(|v| v.checked_add(i64::from(self.seconds - before) * 1000))
            .ok_or(ZoneError::OutOfRange)
    }
}
struct PosixParser<'a> {
    bytes: &'a [u8],
    at: usize,
    extended: bool,
}
impl<'a> PosixParser<'a> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }
    fn byte(&mut self, b: u8) -> Result<(), ZoneError> {
        if self.peek() != Some(b) {
            return Err(ZoneError::Malformed);
        }
        self.at += 1;
        Ok(())
    }
    fn number(&mut self, max: u32) -> Result<u32, ZoneError> {
        let start = self.at;
        let mut n = 0u32;
        while let Some(b'0'..=b'9') = self.peek() {
            n = n
                .checked_mul(10)
                .and_then(|v| v.checked_add(u32::from(self.bytes[self.at] - b'0')))
                .ok_or(ZoneError::Malformed)?;
            self.at += 1;
            if n > max {
                return Err(ZoneError::Malformed);
            }
        }
        if start == self.at {
            Err(ZoneError::Malformed)
        } else {
            Ok(n)
        }
    }
    fn name(&mut self) -> Result<&'a [u8], ZoneError> {
        let quoted = self.peek() == Some(b'<');
        if quoted {
            self.at += 1;
        }
        let start = self.at;
        while self.peek().is_some_and(|b| {
            b.is_ascii_alphabetic() || (quoted && (b.is_ascii_digit() || matches!(b, b'+' | b'-')))
        }) {
            self.at += 1;
        }
        let end = self.at;
        if quoted {
            self.byte(b'>')?;
        }
        if end - start < 3 {
            return Err(ZoneError::Malformed);
        }
        Ok(&self.bytes[start..end])
    }
    fn time(&mut self, max: u32, signed: bool) -> Result<i32, ZoneError> {
        let sign = match self.peek() {
            Some(b'-') if signed => {
                self.at += 1;
                -1
            }
            Some(b'+') if signed => {
                self.at += 1;
                1
            }
            _ => 1,
        };
        let mut n = self.number(max)? * 3600;
        if self.peek() == Some(b':') {
            self.at += 1;
            n += self.number(59)? * 60;
            if self.peek() == Some(b':') {
                self.at += 1;
                n += self.number(59)?;
            }
        }
        Ok(sign * n as i32)
    }
    fn rule(&mut self) -> Result<Rule, ZoneError> {
        let day = match self.peek() {
            Some(b'J') => {
                self.at += 1;
                let n = self.number(365)?;
                if n == 0 {
                    return Err(ZoneError::Malformed);
                }
                RuleDay::Julian(n as u16)
            }
            Some(b'M') => {
                self.at += 1;
                let month = self.number(12)? as u8;
                self.byte(b'.')?;
                let week = self.number(5)? as u8;
                self.byte(b'.')?;
                let weekday = self.number(6)? as u8;
                if month == 0 || week == 0 {
                    return Err(ZoneError::Malformed);
                }
                RuleDay::Month {
                    month,
                    week,
                    weekday,
                }
            }
            _ => RuleDay::Ordinal(self.number(365)? as u16),
        };
        let seconds = if self.peek() == Some(b'/') {
            self.at += 1;
            self.time(if self.extended { 167 } else { 24 }, self.extended)?
        } else {
            7200
        };
        Ok(Rule { day, seconds })
    }
}
#[cfg(test)]
#[path = "time_zone/tests.rs"]
mod tests;
