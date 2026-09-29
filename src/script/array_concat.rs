//! Streaming concat with live spreadability and same-realm species.
use super::*;

#[cfg(test)]
mod tests;

const MAX_LENGTH: u64 = 9_007_199_254_740_991;

// Installation and primitive boxing use the same conservative B=6 bounds as
// the shared splice operations. Keep their private accounting API unchanged.
fn tree_bound(count: usize) -> (usize, usize) {
    if count == 0 {
        return (0, 0);
    }
    let mut capacity = count.saturating_add(1) / 2;
    let mut nodes = 1usize;
    while capacity >= 6 {
        capacity /= 6;
        nodes += 1;
    }
    (count.min(11usize.saturating_mul(nodes)), nodes)
}

fn search_work(count: usize, units: usize) -> usize {
    tree_bound(count).0.saturating_mul(1 + units / 8)
}

impl Runtime {
    pub(super) fn install_array_concat(&mut self) -> Result<()> {
        self.work(
            64 + search_work(self.native_properties.len(), 12).saturating_mul(2)
                + tree_bound(self.native_properties.len())
                    .1
                    .saturating_add(1)
                    .saturating_mul(128)
                + search_work(self.prototypes.len(), 5),
        )?;
        self.charge(1280)?;
        let function = self.intrinsic_function("Array.concat", "concat", 1)?;
        let object = self.prototypes["Array"];
        self.work(
            search_work(self.objects[object].values.len(), 6).saturating_mul(2)
                + tree_bound(self.objects[object].values.len())
                    .1
                    .saturating_add(1)
                    .saturating_mul(128),
        )?;
        self.objects[object].insert_hidden("concat".into(), function);
        Ok(())
    }

    pub(super) fn array_concat(
        &mut self,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        if !js_object(&receiver) && !matches!(receiver, Value::Null | Value::Undefined) {
            self.work(search_work(self.prototypes.len(), 7).saturating_mul(2))?;
        }
        let object = self.coerce_object(receiver)?;
        // Construction can mutate every input before any spreadability read.
        let result = self.splice_species(&object, 0, doc)?;
        let mut next_index = 0;
        for item in std::iter::once(&object).chain(arguments.iter()) {
            self.concat_append(&result, item, &mut next_index, doc)?;
        }
        self.splice_length(&result, next_index, doc)?;
        Ok(result)
    }

    fn concat_spreadable(&mut self, item: &Value, doc: &mut Document) -> Result<bool> {
        // Primitive arguments never consult prototype spreadability hooks.
        if !js_object(item) {
            return Ok(false);
        }
        let flag = self.splice_symbol_get(item, "isConcatSpreadable", doc)?;
        Ok(if flag == Value::Undefined {
            matches!(item, Value::Array(_))
        } else {
            flag.truthy()
        })
    }

    fn concat_append(
        &mut self,
        result: &Value,
        item: &Value,
        next_index: &mut u64,
        doc: &mut Document,
    ) -> Result<()> {
        self.tick()?;
        debug_assert!(*next_index <= MAX_LENGTH);
        if self.concat_spreadable(item, doc)? {
            let length = self.splice_named_get(item, "length", doc)?;
            let length = integer_or_infinity(self.splice_number(length, doc)?)
                .clamp(0.0, MAX_LENGTH as f64) as u64;
            if length > MAX_LENGTH - *next_index {
                return Err(ScriptError::type_error(
                    "concat length exceeds safe integer range",
                ));
            }
            for source_index in 0..length {
                self.tick()?;
                let from = self.reduce_index_key(source_index)?;
                if self.splice_property(item, &from)?.is_some() {
                    let value = self.splice_get(item, &from, doc)?;
                    // Get must precede destination-key work/definition. The
                    // result may alias this item or any later argument.
                    let to = self.reduce_index_key(*next_index)?;
                    self.splice_define(result, to, value, true, doc)?;
                }
                // Holes advance the index without deleting existing output.
                *next_index += 1;
            }
        } else {
            if *next_index >= MAX_LENGTH {
                return Err(ScriptError::type_error(
                    "concat length exceeds safe integer range",
                ));
            }
            let to = self.reduce_index_key(*next_index)?;
            self.splice_define(result, to, item.clone(), true, doc)?;
            *next_index += 1;
        }
        Ok(())
    }
}
