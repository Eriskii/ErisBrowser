//! Non-shared Number TypedArrays with private brands and fresh buffer witnesses.
use super::array_buffer::BufferId;
use super::scalar_codec::Codec;
use super::*;

#[cfg(test)]
mod call_tests;
mod constructors;
mod fill;
mod index;
mod intrinsics;
mod reverse;
mod search;
mod set_key;
#[cfg(test)]
mod tests;
mod to_reversed;
mod views;

use index::Index;
pub(super) use intrinsics::{GLOBAL_COUNT, GLOBAL_NAME_MAX, METADATA_OBJECTS};
pub(super) use set_key::SetKey;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum Kind {
    Int8,
    Uint8,
    Uint8Clamped,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Float16,
    Float32,
    Float64,
}

impl Kind {
    pub(super) const ALL: [Self; 10] = [
        Self::Int8,
        Self::Uint8,
        Self::Uint8Clamped,
        Self::Int16,
        Self::Uint16,
        Self::Int32,
        Self::Uint32,
        Self::Float16,
        Self::Float32,
        Self::Float64,
    ];

    pub(super) const fn index(self) -> usize {
        self as usize
    }

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Int8 => "Int8Array",
            Self::Uint8 => "Uint8Array",
            Self::Uint8Clamped => "Uint8ClampedArray",
            Self::Int16 => "Int16Array",
            Self::Uint16 => "Uint16Array",
            Self::Int32 => "Int32Array",
            Self::Uint32 => "Uint32Array",
            Self::Float16 => "Float16Array",
            Self::Float32 => "Float32Array",
            Self::Float64 => "Float64Array",
        }
    }

    pub(super) fn named(name: &str) -> Option<Self> {
        Some(match name {
            "Int8Array" => Self::Int8,
            "Uint8Array" => Self::Uint8,
            "Uint8ClampedArray" => Self::Uint8Clamped,
            "Int16Array" => Self::Int16,
            "Uint16Array" => Self::Uint16,
            "Int32Array" => Self::Int32,
            "Uint32Array" => Self::Uint32,
            "Float16Array" => Self::Float16,
            "Float32Array" => Self::Float32,
            "Float64Array" => Self::Float64,
            _ => return None,
        })
    }

    pub(super) const fn width(self) -> usize {
        match self {
            Self::Int8 | Self::Uint8 | Self::Uint8Clamped => 1,
            Self::Int16 | Self::Uint16 | Self::Float16 => 2,
            Self::Int32 | Self::Uint32 | Self::Float32 => 4,
            Self::Float64 => 8,
        }
    }

    fn codec(self) -> Codec {
        match self {
            Self::Int8 => Codec::Int8,
            Self::Uint8 | Self::Uint8Clamped => Codec::Uint8,
            Self::Int16 => Codec::Int16,
            Self::Uint16 => Codec::Uint16,
            Self::Int32 => Codec::Int32,
            Self::Uint32 => Codec::Uint32,
            Self::Float16 => Codec::Float16,
            Self::Float32 => Codec::Float32,
            Self::Float64 => Codec::Float64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ViewLength {
    Fixed(usize),
    Tracking,
}

#[derive(Clone, Copy, Debug)]
struct Backing {
    buffer: BufferId,
    offset: usize,
    length: ViewLength,
}

#[derive(Clone, Copy, Debug)]
struct Record {
    object_id: usize,
    kind: Kind,
    // Reserve the brand beside shell creation, before recursive source hooks.
    // Abrupt construction can leave an unreachable, uninitialized record.
    backing: Option<Backing>,
}

#[derive(Default)]
pub(super) struct State {
    records: Vec<Record>,
    intrinsic: Option<Value>,
    prototype: Option<usize>,
    pub(super) constructors: [Option<Value>; 10],
    prototypes: [Option<usize>; 10],
}

#[derive(Clone, Copy)]
struct Live {
    buffer: BufferId,
    offset: usize,
    length: usize,
}

// Handled(None) is terminal absence, never permission to search prototypes.
pub(super) enum Exotic<T> {
    Ordinary,
    Handled(T),
}

fn clamp_u8(number: f64) -> f64 {
    if number.is_nan() || number <= 0.0 {
        return 0.0;
    }
    if number >= 255.0 {
        return 255.0;
    }
    let low = number.floor();
    let midpoint = low + 0.5;
    if number > midpoint || (number == midpoint && low as u8 & 1 != 0) {
        low + 1.0
    } else {
        low
    }
}

impl Runtime {
    // Machine native dispatch creates one temporary owned name while binding
    // the actual receiver. Admit its copy and retirement before allocating.
    pub(super) fn typed_array_call_preflight(&mut self, name: &str) -> Result<()> {
        self.work(4 + name.len())?;
        self.charge(32 + name.len())
    }

    fn typed_array_record(&mut self, value: &Value) -> Result<Option<Record>> {
        let Value::Object(id) = value else {
            return Ok(None);
        };
        let mut first = 0;
        let mut last = self.typed_arrays.records.len();
        while first < last {
            self.work(4)?;
            let middle = first + (last - first) / 2;
            match self.typed_arrays.records[middle].object_id.cmp(id) {
                Ordering::Less => first = middle + 1,
                Ordering::Greater => last = middle,
                Ordering::Equal => return Ok(Some(self.typed_arrays.records[middle])),
            }
        }
        Ok(None)
    }

    fn typed_array_live(&mut self, record: Record) -> Result<Option<Live>> {
        self.work(12)?;
        let Some(backing) = record.backing else {
            return Ok(None);
        };
        let metadata = self.buffer_view_metadata(backing.buffer)?;
        let Some(bytes) = metadata.byte_length else {
            return Ok(None);
        };
        let Some(remaining) = bytes.checked_sub(backing.offset) else {
            return Ok(None);
        };
        let length = match backing.length {
            ViewLength::Tracking => remaining / record.kind.width(),
            ViewLength::Fixed(length) => {
                let bytes = length
                    .checked_mul(record.kind.width())
                    .ok_or_else(|| ScriptError::resource("TypedArray byte length overflow"))?;
                if bytes > remaining {
                    return Ok(None);
                }
                length
            }
        };
        Ok(Some(Live {
            buffer: backing.buffer,
            offset: backing.offset,
            length,
        }))
    }

    fn typed_array_length(&mut self, record: Record) -> Result<usize> {
        self.typed_array_live(record)?
            .map(|live| live.length)
            .ok_or_else(|| ScriptError::type_error("TypedArray is detached or out of bounds"))
    }

    fn typed_array_offset(&mut self, live: Live, kind: Kind, index: usize) -> Result<usize> {
        self.work(4)?;
        index
            .checked_mul(kind.width())
            .and_then(|offset| live.offset.checked_add(offset))
            .ok_or_else(|| ScriptError::resource("TypedArray element offset overflow"))
    }

    fn typed_array_read_number(&mut self, record: Record, index: usize) -> Result<f64> {
        let live = self
            .typed_array_live(record)?
            .filter(|live| index < live.length)
            .ok_or_else(|| ScriptError::type_error("TypedArray element is out of bounds"))?;
        let offset = self.typed_array_offset(live, record.kind, index)?;
        let bytes = self.buffer_view_read(live.buffer, offset, record.kind.width())?;
        self.work(record.kind.codec().work())?;
        // TypedArrays consistently use little-endian storage. DataView retains
        // its independent explicit byte-order argument on the same backing.
        Ok(record.kind.codec().decode(bytes, true))
    }

    fn typed_array_write_number(
        &mut self,
        record: Record,
        index: usize,
        number: f64,
    ) -> Result<()> {
        let Some(live) = self
            .typed_array_live(record)?
            .filter(|live| index < live.length)
        else {
            return Ok(());
        };
        let offset = self.typed_array_offset(live, record.kind, index)?;
        let number = if record.kind == Kind::Uint8Clamped {
            self.work(16)?;
            clamp_u8(number)
        } else {
            number
        };
        self.work(record.kind.codec().work())?;
        let bytes = record.kind.codec().encode(number, true);
        self.buffer_view_write(live.buffer, offset, &bytes[..record.kind.width()])
    }

    fn typed_array_valid_index(&mut self, record: Record, index: f64) -> Result<Option<usize>> {
        self.work(8)?;
        if !index.is_finite()
            || index < 0.0
            || index.fract() != 0.0
            || (index == 0.0 && index.is_sign_negative())
        {
            return Ok(None);
        }
        // Backing storage is bounded by MAX_HEAP, well below 2^53. Compare
        // against that live length before any float-to-usize conversion.
        Ok(self.typed_array_live(record)?.and_then(|live| {
            if index < live.length as f64 {
                Some(index as usize)
            } else {
                None
            }
        }))
    }

    fn typed_array_key(&mut self, value: &Value, key: &JsString) -> Result<Option<(Record, f64)>> {
        let Some(record) = self.typed_array_record(value)? else {
            return Ok(None);
        };
        Ok(match self.typed_array_index(key)? {
            Index::Ordinary => None,
            Index::Numeric(index) => Some((record, index)),
        })
    }

    pub(super) fn typed_array_own_property(
        &mut self,
        value: &Value,
        key: &JsString,
        read_value: bool,
    ) -> Result<Exotic<Option<Property>>> {
        let Some((record, number)) = self.typed_array_key(value, key)? else {
            return Ok(Exotic::Ordinary);
        };
        self.typed_array_own_index(record, number, read_value)
    }

    fn typed_array_own_index(
        &mut self,
        record: Record,
        number: f64,
        read_value: bool,
    ) -> Result<Exotic<Option<Property>>> {
        let Some(index) = self.typed_array_valid_index(record, number)? else {
            return Ok(Exotic::Handled(None));
        };
        let value = if read_value {
            Value::Number(self.typed_array_read_number(record, index)?)
        } else {
            Value::Undefined
        };
        Ok(Exotic::Handled(Some(Property::data(
            value, true, true, true,
        ))))
    }

    fn typed_array_set_index(
        &mut self,
        target: &Value,
        receiver: &Value,
        value: Value,
        record: Record,
        number: f64,
        doc: &mut Document,
    ) -> Result<Exotic<bool>> {
        if target != receiver {
            return Ok(if self.typed_array_valid_index(record, number)?.is_none() {
                Exotic::Handled(true)
            } else {
                // OrdinarySetWithOwnDescriptor uses the writable indexed
                // descriptor and the original, distinct receiver.
                Exotic::Ordinary
            });
        }
        // Coercion precedes validity, including invalid canonical numbers.
        let value = self.splice_number(value, doc)?;
        if let Some(index) = self.typed_array_valid_index(record, number)? {
            self.typed_array_write_number(record, index, value)?;
        }
        Ok(Exotic::Handled(true))
    }

    pub(super) fn typed_array_define(
        &mut self,
        target: &Value,
        key: &JsString,
        descriptor: &PropertyDescriptor,
        doc: &mut Document,
    ) -> Result<Exotic<bool>> {
        let Some((record, number)) = self.typed_array_key(target, key)? else {
            return Ok(Exotic::Ordinary);
        };
        self.typed_array_define_index(record, number, descriptor, doc)
    }

    fn typed_array_define_index(
        &mut self,
        record: Record,
        number: f64,
        descriptor: &PropertyDescriptor,
        doc: &mut Document,
    ) -> Result<Exotic<bool>> {
        if self.typed_array_valid_index(record, number)?.is_none()
            || descriptor.get.is_some()
            || descriptor.set.is_some()
            || descriptor.configurable == Some(false)
            || descriptor.enumerable == Some(false)
            || descriptor.writable == Some(false)
        {
            return Ok(Exotic::Handled(false));
        }
        if let Some(value) = &descriptor.value {
            let value = self.splice_number(value.clone(), doc)?;
            if let Some(index) = self.typed_array_valid_index(record, number)? {
                self.typed_array_write_number(record, index, value)?;
            }
        }
        Ok(Exotic::Handled(true))
    }

    pub(super) fn typed_array_delete(
        &mut self,
        target: &Value,
        key: &JsString,
    ) -> Result<Exotic<bool>> {
        let Some((record, number)) = self.typed_array_key(target, key)? else {
            return Ok(Exotic::Ordinary);
        };
        Ok(Exotic::Handled(
            self.typed_array_valid_index(record, number)?.is_none(),
        ))
    }

    pub(super) fn typed_array_own_length(&mut self, value: &Value) -> Result<Option<usize>> {
        let Some(record) = self.typed_array_record(value)? else {
            return Ok(None);
        };
        Ok(Some(
            self.typed_array_live(record)?.map_or(0, |live| live.length),
        ))
    }

    pub(super) fn typed_array_has_records(&self) -> bool {
        !self.typed_arrays.records.is_empty()
    }

    pub(super) fn typed_array_is_view(&mut self, value: &Value) -> Result<bool> {
        Ok(self.typed_array_record(value)?.is_some())
    }

    pub(super) fn typed_array_can_prevent_extensions(&mut self, value: &Value) -> Result<bool> {
        let Some(record) = self.typed_array_record(value)? else {
            return Ok(true);
        };
        let Some(backing) = record.backing else {
            return Ok(true);
        };
        Ok(!self.buffer_view_metadata(backing.buffer)?.resizable)
    }

    pub(super) fn typed_array_iterator_length(&mut self, value: &Value) -> Result<Option<usize>> {
        self.typed_array_record(value)?
            .map(|record| self.typed_array_length(record))
            .transpose()
    }

    pub(super) fn typed_array_native(
        &mut self,
        method: &str,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        if method == "getSpecies" {
            return Ok(receiver);
        }
        let record = self.typed_array_record(&receiver)?;
        if method == "getTag" {
            return match record {
                Some(record) => Ok(Value::String(self.dom_proto_text(record.kind.name())?)),
                None => Ok(Value::Undefined),
            };
        }
        let record =
            record.ok_or_else(|| ScriptError::type_error("receiver is not a TypedArray"))?;
        if method == "getBuffer" {
            let backing = record
                .backing
                .ok_or_else(|| ScriptError::type_error("uninitialized TypedArray"))?;
            return self.buffer_view_value(backing.buffer);
        }
        match method {
            "subarray" => self.typed_array_subarray(receiver, record, arguments, doc),
            "join" => self.typed_array_join(record, arguments, doc),
            "fill" => self.typed_array_fill(receiver, record, arguments, doc),
            "reverse" => self.typed_array_reverse(receiver, record),
            "toReversed" => self.typed_array_to_reversed(record, doc),
            "at" => self.typed_array_at(record, arguments, doc),
            "includes" => {
                self.typed_array_search(search::SearchKind::Includes, record, arguments, doc)
            }
            "indexOf" => {
                self.typed_array_search(search::SearchKind::IndexOf, record, arguments, doc)
            }
            "lastIndexOf" => {
                self.typed_array_search(search::SearchKind::LastIndexOf, record, arguments, doc)
            }
            "getLength" | "getByteLength" | "getByteOffset" => {
                let live = self.typed_array_live(record)?;
                let number = live.map_or(0, |live| match method {
                    "getLength" => live.length,
                    "getByteLength" => live.length * record.kind.width(),
                    _ => live.offset,
                });
                Ok(Value::Number(number as f64))
            }
            "keys" | "values" | "entries" => {
                self.typed_array_length(record)?;
                self.iterator_native(
                    match method {
                        "keys" => "array.keys",
                        "values" => "array.values",
                        _ => "array.entries",
                    },
                    receiver,
                    arguments,
                    doc,
                )
            }
            _ => Err(ScriptError::unsupported("unknown TypedArray intrinsic")),
        }
    }
}
