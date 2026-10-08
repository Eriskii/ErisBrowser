//! Number TypedArray construction with ordered coercions and private backing.
use super::*;

impl Runtime {
    fn typed_array_reserve_record(&mut self) -> Result<()> {
        self.tick()?;
        let records = &self.typed_arrays.records;
        if records.len() < records.capacity() {
            return Ok(());
        }
        let capacity = records
            .capacity()
            .checked_mul(2)
            .map(|count| count.max(4))
            .ok_or_else(|| ScriptError::resource("TypedArray record capacity overflow"))?;
        let bytes = capacity
            .checked_mul(std::mem::size_of::<Record>())
            .ok_or_else(|| ScriptError::resource("TypedArray record allocation overflow"))?;
        let moved = records
            .len()
            .checked_mul(std::mem::size_of::<Record>())
            .ok_or_else(|| ScriptError::resource("TypedArray record movement overflow"))?;
        let additional = capacity - records.len();
        self.work(1 + moved.div_ceil(8).saturating_mul(2))?;
        self.charge(bytes)?;
        self.typed_arrays
            .records
            .try_reserve_exact(additional)
            .map_err(|_| ScriptError::resource("TypedArray record allocation failed"))
    }

    fn typed_array_shell(
        &mut self,
        kind: Kind,
        new_target: &Value,
        doc: &mut Document,
    ) -> Result<(Value, usize)> {
        // NewTarget's getter can construct more views before this shell exists.
        let prototype = self.splice_named_get(new_target, "prototype", doc)?;
        self.work(4)?;
        let prototype = if js_object(&prototype) {
            prototype
        } else {
            Value::Object(
                self.typed_arrays.prototypes[kind.index()]
                    .expect("TypedArray intrinsic initialized"),
            )
        };
        let object = self.object_ordered([])?;
        let Value::Object(object_id) = object else {
            unreachable!()
        };
        // No author callback lies between object creation and this append. A
        // failed reserve leaves an unreachable ordinary shell; a later failure
        // leaves an unreachable authentic record with its current backing.
        self.typed_array_reserve_record()?;
        self.work(8)?;
        self.objects[object_id].prototype = Some(prototype);
        let index = self.typed_arrays.records.len();
        debug_assert!(
            self.typed_arrays
                .records
                .last()
                .is_none_or(|r| r.object_id < object_id)
        );
        self.typed_arrays.records.push(Record {
            object_id,
            kind,
            backing: None,
        });
        Ok((object, index))
    }

    fn typed_array_initialize(&mut self, index: usize, backing: Backing) -> Result<Record> {
        // Publication has no fallible work after its first record write.
        self.work(8)?;
        debug_assert!(self.typed_arrays.records[index].backing.is_none());
        self.typed_arrays.records[index].backing = Some(backing);
        Ok(self.typed_arrays.records[index])
    }

    fn typed_array_byte_length(&mut self, kind: Kind, length: u64) -> Result<u64> {
        self.work(4)?;
        length
            .checked_mul(kind.width() as u64)
            .ok_or_else(|| ScriptError::range_error("TypedArray byte length overflow"))
    }

    fn typed_array_allocate_backing(&mut self, index: usize, length: u64) -> Result<Record> {
        self.tick()?;
        let kind = self.typed_arrays.records[index].kind;
        let bytes = self.typed_array_byte_length(kind, length)?;
        let buffer = self.buffer_allocate_intrinsic(bytes)?;
        // A successful allocation obeys the existing absolute MAX_HEAP host
        // capacity. No host-sized cast of an unadmitted requested length occurs.
        let length = usize::try_from(length)
            .map_err(|_| ScriptError::resource("TypedArray element length overflow"))?;
        self.typed_array_initialize(
            index,
            Backing {
                buffer,
                offset: 0,
                length: ViewLength::Fixed(length),
            },
        )
    }

    fn typed_array_from_buffer(
        &mut self,
        index: usize,
        buffer: BufferId,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<()> {
        self.work(8)?;
        let kind = self.typed_arrays.records[index].kind;
        let width = kind.width() as u64;
        let offset =
            self.buffer_index(arguments.get(1).cloned().unwrap_or(Value::Undefined), doc)?;
        if offset % width != 0 {
            return Err(ScriptError::range_error(
                "TypedArray byte offset is not aligned",
            ));
        }
        // IsFixedLengthArrayBuffer precedes explicit length conversion. Read
        // only its immutable resize capability here; do not reject detachment
        // until after that conversion has had its specified opportunity to run.
        let resizable = self.buffer_view_metadata(buffer)?.resizable;
        let explicit = arguments
            .get(2)
            .filter(|value| !matches!(value, Value::Undefined));
        let length = explicit
            .map(|value| self.buffer_index(value.clone(), doc))
            .transpose()?;
        let current = self
            .buffer_view_metadata(buffer)?
            .byte_length
            .ok_or_else(|| ScriptError::type_error("TypedArray buffer is detached"))?
            as u64;
        self.work(8)?;
        let view_length = if let Some(length) = length {
            let bytes = self.typed_array_byte_length(kind, length)?;
            if offset.checked_add(bytes).is_none_or(|end| end > current) {
                return Err(ScriptError::range_error("TypedArray view exceeds buffer"));
            }
            ViewLength::Fixed(length as usize)
        } else if resizable {
            if offset > current {
                return Err(ScriptError::range_error("TypedArray offset exceeds buffer"));
            }
            // An odd trailing byte is allowed for a tracking multibyte view.
            ViewLength::Tracking
        } else {
            if !current.is_multiple_of(width) {
                return Err(ScriptError::range_error(
                    "TypedArray buffer length is not aligned",
                ));
            }
            let remaining = current
                .checked_sub(offset)
                .ok_or_else(|| ScriptError::range_error("TypedArray offset exceeds buffer"))?;
            ViewLength::Fixed((remaining / width) as usize)
        };
        self.typed_array_initialize(
            index,
            Backing {
                buffer,
                offset: offset as usize,
                length: view_length,
            },
        )?;
        Ok(())
    }

    fn typed_array_from_typed(&mut self, index: usize, source: Record) -> Result<()> {
        self.work(4)?;
        let length = self.typed_array_length(source)?;
        let kind = self.typed_arrays.records[index].kind;
        let bytes = self.typed_array_byte_length(kind, length as u64)?;
        let source_backing = source.backing.expect("validated TypedArray backing");
        let buffer = if kind == source.kind {
            // No Number decode/re-encode: preserve every floating NaN payload
            // and signed zero byte. Intrinsic clone invokes no authored hooks.
            let bytes = usize::try_from(bytes)
                .map_err(|_| ScriptError::resource("TypedArray byte length overflow"))?;
            self.buffer_clone_range_intrinsic(source_backing.buffer, source_backing.offset, bytes)?
        } else {
            self.buffer_allocate_intrinsic(bytes)?
        };
        let backing = Backing {
            buffer,
            offset: 0,
            length: ViewLength::Fixed(length),
        };
        if kind != source.kind {
            // A copied temporary record lets scalar helpers reach the allocated
            // destination without publishing slots before this callback-free
            // copy finishes. All selected kinds have Number content.
            let target = Record {
                backing: Some(backing),
                ..self.typed_arrays.records[index]
            };
            for element in 0..length {
                self.work(4)?;
                let value = self.typed_array_read_number(source, element)?;
                self.typed_array_write_number(target, element, value)?;
            }
        }
        self.typed_array_initialize(index, backing)?;
        Ok(())
    }

    fn typed_array_list_push(&mut self, values: &mut Vec<Value>, value: Value) -> Result<()> {
        // Includes the retained handle's eventual destruction, also when a
        // later conversion/append refuses. Values move; strings are not copied.
        self.work(4)?;
        if values.len() == values.capacity() {
            let capacity = values
                .capacity()
                .checked_mul(2)
                .map(|count| count.max(4))
                .ok_or_else(|| ScriptError::resource("TypedArray list capacity overflow"))?;
            let bytes = capacity
                .checked_mul(std::mem::size_of::<Value>())
                .ok_or_else(|| ScriptError::resource("TypedArray list allocation overflow"))?;
            let moved = values
                .len()
                .checked_mul(std::mem::size_of::<Value>())
                .ok_or_else(|| ScriptError::resource("TypedArray list movement overflow"))?;
            self.work(1 + moved.div_ceil(8).saturating_mul(2))?;
            self.charge(bytes)?;
            values
                .try_reserve_exact(capacity - values.len())
                .map_err(|_| ScriptError::resource("TypedArray list allocation failed"))?;
        }
        values.push(value);
        Ok(())
    }

    fn typed_array_from_iterable(
        &mut self,
        index: usize,
        source: Value,
        method: Value,
        doc: &mut Document,
    ) -> Result<()> {
        self.work(4)?;
        let iterator = self.call(method, Vec::new(), source, doc)?;
        if !js_object(&iterator) {
            return Err(ScriptError::type_error(
                "TypedArray iterator is not an object",
            ));
        }
        let next = self.splice_named_get(&iterator, "next", doc)?;
        let mut values = Vec::new();
        loop {
            // Pays fixed loop/handle/test work, including disposal of the
            // current step/value if a subsequent list reservation refuses.
            self.work(8)?;
            let step = self.call(next.clone(), Vec::new(), iterator.clone(), doc)?;
            if !js_object(&step) {
                return Err(ScriptError::type_error(
                    "TypedArray iterator result is not an object",
                ));
            }
            if self.splice_named_get(&step, "done", doc)?.truthy() {
                break;
            }
            let value = self.splice_named_get(&step, "value", doc)?;
            self.typed_array_list_push(&mut values, value)?;
        }
        // IteratorToList is already exhausted before any element ToNumber.
        // Step/getter errors and later conversion errors do not IteratorClose;
        // terminal Resource/Unsupported failures never invoke authored cleanup.
        let target = self.typed_array_allocate_backing(index, values.len() as u64)?;
        for (element, value) in values.into_iter().enumerate() {
            self.work(4)?;
            let number = self.splice_number(value, doc)?;
            self.typed_array_write_number(target, element, number)?;
        }
        Ok(())
    }

    fn typed_array_from_array_like(
        &mut self,
        index: usize,
        source: &Value,
        doc: &mut Document,
    ) -> Result<()> {
        let length = self.splice_named_get(source, "length", doc)?;
        let number = self.splice_number(length, doc)?;
        self.work(4)?;
        let length = integer_or_infinity(number).clamp(0.0, 9_007_199_254_740_991.0) as u64;
        let target = self.typed_array_allocate_backing(index, length)?;
        for element in 0..length {
            self.work(4)?;
            let key = self.reduce_index_key(element)?;
            let value = self.reduce_get(source, &key, doc)?;
            let number = self.splice_number(value, doc)?;
            self.typed_array_write_number(target, element as usize, number)?;
        }
        Ok(())
    }

    pub(in crate::script) fn typed_array_constructor(
        &mut self,
        kind: Kind,
        arguments: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(4)?;
        if matches!(new_target, Value::Undefined) {
            return Err(ScriptError::type_error(
                "TypedArray constructor requires new",
            ));
        }
        let source = arguments.first().cloned().unwrap_or(Value::Undefined);
        if !js_object(&source) {
            let length = if arguments.is_empty() {
                0
            } else {
                self.buffer_index(source, doc)?
            };
            let (object, index) = self.typed_array_shell(kind, &new_target, doc)?;
            self.typed_array_allocate_backing(index, length)?;
            return Ok(object);
        }
        // Authentic classification and all authored source access occur after
        // prototype lookup and immediate shell/None-record publication.
        let (object, index) = self.typed_array_shell(kind, &new_target, doc)?;
        if let Some(record) = self.typed_array_record(&source)? {
            self.typed_array_from_typed(index, record)?;
        } else if let Some(buffer) = self.buffer_probe(&source)? {
            self.typed_array_from_buffer(index, buffer, arguments, doc)?;
        } else {
            let method = self.splice_symbol_get(&source, "iterator", doc)?;
            if matches!(method, Value::Undefined | Value::Null) {
                self.typed_array_from_array_like(index, &source, doc)?;
            } else {
                if !json_callable(&method) {
                    return Err(ScriptError::type_error(
                        "TypedArray iterator method is not callable",
                    ));
                }
                self.typed_array_from_iterable(index, source, method, doc)?;
            }
        }
        Ok(object)
    }
}
