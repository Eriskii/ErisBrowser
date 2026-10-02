//! CharacterData operations over fresh node data and exact UTF-16 indices.
use super::*;

#[cfg(test)]
mod tests;

pub(super) const METHODS: &[(&str, usize)] = &[
    ("substringData", 2),
    ("appendData", 1),
    ("insertData", 2),
    ("deleteData", 2),
    ("replaceData", 3),
];

#[derive(Clone, Copy)]
enum Operation {
    Substring,
    Append,
    Insert,
    Delete,
    Replace,
}
impl Operation {
    fn from_name(name: &str) -> Result<Self> {
        match name {
            "substringData" => Ok(Self::Substring),
            "appendData" => Ok(Self::Append),
            "insertData" => Ok(Self::Insert),
            "deleteData" => Ok(Self::Delete),
            "replaceData" => Ok(Self::Replace),
            _ => Err(ScriptError::type_error("unknown CharacterData operation")),
        }
    }
    fn arity(self) -> usize {
        match self {
            Self::Append => 1,
            Self::Replace => 3,
            _ => 2,
        }
    }
}

fn current_data(doc: &Document, id: NodeId) -> Result<&str> {
    match doc.nodes.get(id).map(|node| &node.kind) {
        Some(NodeKind::Text(text) | NodeKind::Comment(text))
        | Some(NodeKind::ProcessingInstruction { data: text, .. }) => Ok(text),
        _ => Err(ScriptError::type_error(
            "CharacterData method requires a genuine node",
        )),
    }
}
fn work_overflow() -> ScriptError {
    ScriptError::resource("CharacterData work overflow")
}

impl Runtime {
    pub(super) fn character_data_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // Bound all five fixed-name comparisons before dispatch, plus brand
        // and signature decisions. This path owns no retained author borrow.
        self.work(8 + 5 * name.len())?;
        let operation = Operation::from_name(name)?;
        let Value::Node(id) = receiver else {
            return Err(ScriptError::type_error(
                "CharacterData method requires a genuine node",
            ));
        };
        current_data(doc, id)?;
        if args.len() < operation.arity() {
            return Err(ScriptError::type_error(
                "missing CharacterData method argument",
            ));
        }
        // Web IDL unsigned long is a wrapping Number conversion, not ToIndex.
        // Finish all callbacks before taking any data or length snapshot.
        let offset = match operation {
            Operation::Append => 0,
            _ => to_i32(self.number_value(args[0].clone(), doc)?) as u32 as usize,
        };
        let count = match operation {
            Operation::Substring | Operation::Delete | Operation::Replace => {
                to_i32(self.number_value(args[1].clone(), doc)?) as u32 as usize
            }
            _ => 0,
        };
        let inserted = match operation {
            Operation::Append => self.string_hint(args[0].clone(), doc)?,
            Operation::Insert => self.string_hint(args[1].clone(), doc)?,
            Operation::Replace => self.string_hint(args[2].clone(), doc)?,
            _ => JsString::default(),
        };
        let old = current_data(doc, id)?;
        if matches!(operation, Operation::Substring) {
            return self.character_data_substring(old, offset, count);
        }
        self.work(1 + old.len())?;
        let length = old.encode_utf16().count();
        let offset = if matches!(operation, Operation::Append) {
            length
        } else {
            offset
        };
        if offset > length {
            return Err(self.character_data_index_error()?);
        }
        let count = count.min(length - offset);
        let result = self.character_data_splice(old, length, offset, count, &inserted)?;
        // The owned output is already paid. This fresh checked publication
        // reuses replacement-aware DOM admission and moves without copying.
        self.work(8)?;
        doc.replace_character_data(id, result)
            .map_err(dom_data_error)?;
        Ok(Value::Undefined)
    }

    fn character_data_index_error(&mut self) -> Result<ScriptError> {
        let name = self.dom_proto_text("IndexSizeError")?;
        let message = self.dom_proto_text("offset exceeds CharacterData length")?;
        let value = self.dom_exception(name, message)?;
        self.thrown_error(value)
    }

    fn character_data_substring(
        &mut self,
        old: &str,
        offset: usize,
        count: usize,
    ) -> Result<Value> {
        let mut units = old.encode_utf16();
        for _ in 0..offset {
            // Includes attempted EOF: at most four UTF8 bytes, the UTF16
            // iterator state, and the surrounding loop decision per step.
            self.work(8)?;
            if units.next().is_none() {
                return Err(self.character_data_index_error()?);
            }
        }
        let mut selected = 0usize;
        for _ in 0..count {
            self.work(8)?;
            if units.next().is_none() {
                break;
            }
            selected += 1;
            if selected > MAX_STRING {
                return Err(ScriptError::resource("script string limit exceeded"));
            }
        }
        let work = offset
            .checked_add(selected)
            .and_then(|n| n.checked_mul(8))
            .and_then(|n| n.checked_add(selected))
            .and_then(|n| n.checked_add(1))
            .ok_or_else(work_overflow)?;
        self.work(work)?;
        self.charge(64 + 4 * selected)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(selected)
            .map_err(|_| ScriptError::resource("CharacterData substring allocation failed"))?;
        output.extend(old.encode_utf16().skip(offset).take(selected));
        Ok(Value::String(output.into()))
    }

    fn character_data_splice(
        &mut self,
        old: &str,
        length: usize,
        offset: usize,
        count: usize,
        inserted: &JsString,
    ) -> Result<String> {
        let end = offset.checked_add(count).ok_or_else(work_overflow)?;
        let result_units = length
            .checked_sub(count)
            .and_then(|n| n.checked_add(inserted.len()))
            .ok_or_else(work_overflow)?;
        // Each pass can decode the old UTF8 prefix twice (suffix.skip starts
        // at zero), visit the insertion, then decode/size all combined units.
        let pass = old
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(inserted.len()))
            .and_then(|n| {
                result_units
                    .checked_mul(2)
                    .and_then(|units| n.checked_add(units))
            })
            .and_then(|n| n.checked_add(8))
            .ok_or_else(work_overflow)?;
        let sequence = || {
            old.encode_utf16()
                .take(offset)
                .chain(inserted.units().iter().copied())
                .chain(old.encode_utf16().skip(end))
        };
        self.work(pass)?;
        let mut bytes = 0usize;
        for scalar in char::decode_utf16(sequence()) {
            let scalar = scalar.map_err(|_| {
                ScriptError::unsupported("unpaired surrogate DOMString storage is not implemented")
            })?;
            bytes = bytes
                .checked_add(scalar.len_utf8())
                .ok_or_else(work_overflow)?;
        }
        // The complete splice determines representability: an inserted lone
        // unit can validly pair with a retained prefix or suffix code unit.
        self.work(pass.checked_add(bytes).ok_or_else(work_overflow)?)?;
        self.charge(bytes)?;
        let mut output = String::new();
        output
            .try_reserve_exact(bytes)
            .map_err(|_| ScriptError::resource("CharacterData replacement allocation failed"))?;
        for scalar in char::decode_utf16(sequence()) {
            output.push(scalar.unwrap());
        }
        Ok(output)
    }
}
