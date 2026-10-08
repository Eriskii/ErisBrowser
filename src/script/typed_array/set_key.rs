//! One Set operation may reuse only its immutable key classification.
use super::*;

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
enum Source<'a> {
    Text(&'a JsString),
    Property(&'a PropertyKey),
}

// No Clone/Copy, object identity, live view, descriptor or owned allocation.
// The numeric payload is initialized once after the eight-unit admission.
pub(in crate::script) struct SetKey<'a> {
    source: Source<'a>,
    index: Option<Index>,
}

impl<'a> SetKey<'a> {
    pub(in crate::script) fn new(text: &'a JsString) -> Self {
        Self {
            source: Source::Text(text),
            index: None,
        }
    }

    pub(in crate::script) fn from_property(key: &'a PropertyKey) -> Option<Self> {
        match key {
            PropertyKey::String(_) => Some(Self {
                source: Source::Property(key),
                index: None,
            }),
            PropertyKey::Symbol(_) => None,
        }
    }

    pub(in crate::script) fn text(&self) -> &'a JsString {
        match self.source {
            Source::Text(text) => text,
            Source::Property(PropertyKey::String(text)) => text,
            Source::Property(PropertyKey::Symbol(_)) => unreachable!("string-only Set key"),
        }
    }

    // Only Reflect/splice continuations constructed from a PropertyKey use
    // this accessor. It returns the same borrowed key, never a new Rc clone.
    pub(in crate::script) fn property(&self) -> &'a PropertyKey {
        match self.source {
            Source::Property(key) => key,
            Source::Text(_) => unreachable!("property-key Set continuation"),
        }
    }
}

impl Runtime {
    fn typed_array_probe_set_key(
        &mut self,
        target: &Value,
        key: &SetKey<'_>,
    ) -> Result<Option<(Record, Index)>> {
        // The exact old record search remains first, even for a reused token.
        let Some(record) = self.typed_array_record(target)? else {
            return Ok(None);
        };
        let index = if let Some(index) = &key.index {
            self.work(4)?;
            *index
        } else {
            self.typed_array_index(key.text())?
        };
        Ok(Some((record, index)))
    }

    fn typed_array_remember_set_key(&mut self, key: &mut SetKey<'_>, index: Index) -> Result<()> {
        if key.index.is_none() {
            self.work(8)?;
            key.index = Some(index);
        }
        Ok(())
    }

    pub(in crate::script) fn typed_array_set_for_set(
        &mut self,
        target: &Value,
        key: &mut SetKey<'_>,
        receiver: &Value,
        value: Value,
        doc: &mut Document,
    ) -> Result<Exotic<bool>> {
        let Some((record, index)) = self.typed_array_probe_set_key(target, key)? else {
            return Ok(Exotic::Ordinary);
        };
        let result = match index {
            Index::Ordinary => Exotic::Ordinary,
            Index::Numeric(number) => {
                self.typed_array_set_index(target, receiver, value, record, number, doc)?
            }
        };
        if matches!(result, Exotic::Ordinary) {
            // A completed numeric Set neither constructs a token nor pays a
            // new debit after mutation. Fallthrough has not written anything.
            self.typed_array_remember_set_key(key, index)?;
        }
        Ok(result)
    }

    pub(in crate::script) fn typed_array_own_property_for_set(
        &mut self,
        target: &Value,
        key: &mut SetKey<'_>,
        read_value: bool,
    ) -> Result<Exotic<Option<Property>>> {
        let Some((record, index)) = self.typed_array_probe_set_key(target, key)? else {
            return Ok(Exotic::Ordinary);
        };
        // An ordinary target can reach an authentic Receiver for the first
        // time here; definition must reuse its key but refresh its live state.
        self.typed_array_remember_set_key(key, index)?;
        match index {
            Index::Ordinary => Ok(Exotic::Ordinary),
            Index::Numeric(number) => self.typed_array_own_index(record, number, read_value),
        }
    }

    pub(in crate::script) fn typed_array_define_for_set(
        &mut self,
        target: &Value,
        key: &mut SetKey<'_>,
        descriptor: &PropertyDescriptor,
        doc: &mut Document,
    ) -> Result<Exotic<bool>> {
        let Some((record, index)) = self.typed_array_probe_set_key(target, key)? else {
            return Ok(Exotic::Ordinary);
        };
        match index {
            Index::Ordinary => Ok(Exotic::Ordinary),
            Index::Numeric(number) => {
                self.typed_array_define_index(record, number, descriptor, doc)
            }
        }
    }
}
