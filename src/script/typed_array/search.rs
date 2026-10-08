//! Number TypedArray searches with captured bounds and fresh element witnesses.
use super::*;

#[derive(Clone, Copy)]
pub(super) enum SearchKind {
    Includes,
    IndexOf,
    LastIndexOf,
}

impl SearchKind {
    fn missing(self) -> Value {
        match self {
            Self::Includes => Value::Bool(false),
            Self::IndexOf | Self::LastIndexOf => Value::Number(-1.0),
        }
    }
}

impl Runtime {
    fn typed_array_search_integer(&mut self, value: Value, doc: &mut Document) -> Result<f64> {
        let number = self.splice_number(value, doc)?;
        // Coercion and its callbacks precede admission of the normalization.
        self.work(4)?;
        Ok(integer_or_infinity(number))
    }

    fn typed_array_search_element(&mut self, record: Record, index: usize) -> Result<Option<f64>> {
        self.work(4)?;
        let Some(live) = self
            .typed_array_live(record)?
            .filter(|live| index < live.length)
        else {
            return Ok(None);
        };
        // No callback occurs between this live witness and the byte read. Its
        // presence also supplies HasProperty for these integer-indexed objects;
        // an absent canonical index never falls through to a prototype.
        let offset = self.typed_array_offset(live, record.kind, index)?;
        let bytes = self.buffer_view_read(live.buffer, offset, record.kind.width())?;
        self.work(record.kind.codec().work())?;
        Ok(Some(record.kind.codec().decode(bytes, true)))
    }

    pub(super) fn typed_array_at(
        &mut self,
        record: Record,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let length = self.typed_array_length(record)?;
        self.work(4)?;
        // Even an initially empty valid view converts its index argument.
        let relative = self.typed_array_search_integer(
            arguments.first().cloned().unwrap_or(Value::Undefined),
            doc,
        )?;
        self.work(6)?;
        let index = if relative < 0.0 {
            length as f64 + relative
        } else {
            relative
        };
        if index < 0.0 || index >= length as f64 {
            return Ok(Value::Undefined);
        }
        Ok(self
            .typed_array_search_element(record, index as usize)?
            .map_or(Value::Undefined, Value::Number))
    }

    pub(super) fn typed_array_search(
        &mut self,
        kind: SearchKind,
        record: Record,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8)?;
        let length = self.typed_array_length(record)?;
        if length == 0 {
            return Ok(kind.missing());
        }
        self.work(6)?;
        let reverse = matches!(kind, SearchKind::LastIndexOf);
        let includes = matches!(kind, SearchKind::Includes);
        // Omission differs from explicit undefined only for lastIndexOf.
        let relative = if reverse && arguments.len() < 2 {
            (length - 1) as f64
        } else {
            self.typed_array_search_integer(
                arguments.get(1).cloned().unwrap_or(Value::Undefined),
                doc,
            )?
        };
        self.work(8)?;
        let start = if reverse {
            if relative < 0.0 {
                length as f64 + relative
            } else {
                relative.min((length - 1) as f64)
            }
        } else if relative < 0.0 {
            (length as f64 + relative).max(0.0)
        } else {
            relative.min(length as f64)
        };
        if start < 0.0 || start >= length as f64 {
            return Ok(kind.missing());
        }
        let mut index = start as usize;
        loop {
            // Admit every captured position, comparison, direction branch and
            // advance, including positions made absent by argument coercion.
            self.work(16)?;
            let element = self.typed_array_search_element(record, index)?;
            let found = match (element, arguments.first()) {
                (Some(number), Some(Value::Number(search))) => {
                    number == *search || (includes && number.is_nan() && search.is_nan())
                }
                (None, None | Some(Value::Undefined)) => includes,
                _ => false,
            };
            if found {
                return Ok(if includes {
                    Value::Bool(true)
                } else {
                    Value::Number(index as f64)
                });
            }
            if reverse {
                if index == 0 {
                    break;
                }
                index -= 1;
            } else {
                // Admitted backing storage bounds length by MAX_HEAP, below
                // usize::MAX and the exact-integer range of binary64.
                index += 1;
                if index == length {
                    break;
                }
            }
        }
        Ok(kind.missing())
    }
}

#[cfg(test)]
mod tests;
