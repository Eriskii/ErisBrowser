//! Generic array operations with observable property access and shared budgets.
use super::*;

impl Runtime {
    pub(super) fn array_last_index_of(
        &mut self,
        receiver: Value,
        search: Value,
        from_index: Option<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let object = self.coerce_object(receiver)?;
        if self.property_object(&object).is_none() {
            return Err(ScriptError::unsupported(
                "host array-like lastIndexOf is not implemented",
            ));
        }
        self.charge(64)?;
        let length = self.reduce_get(&object, &JsString::from("length"), doc)?;
        let length = integer_or_infinity(self.number_value(length, doc)?)
            .clamp(0.0, 9_007_199_254_740_991.0) as u64;
        if length == 0 {
            return Ok(Value::Number(-1.0));
        }
        // An absent argument starts at the end; explicit undefined starts at
        // zero. Conversion can mutate the receiver, but length stays captured.
        let start = if let Some(value) = from_index {
            let index = integer_or_infinity(self.number_value(value, doc)?);
            if index < 0.0 {
                length as f64 + index
            } else {
                index.min((length - 1) as f64)
            }
        } else {
            (length - 1) as f64
        };
        if start < 0.0 {
            return Ok(Value::Number(-1.0));
        }
        // Never allocate a collection from the logical length. HasProperty
        // skips holes but sees inherited entries; Get observes live getters.
        for index in (0..=start as u64).rev() {
            self.tick()?;
            let key = self.reduce_index_key(index)?;
            if self.reduce_property(&object, &key)?.is_none() {
                continue;
            }
            let element = self.reduce_get(&object, &key, doc)?;
            // Reuse strict equality's string-work charging and identity rules.
            if self.binary_value("===", search.clone(), element, doc)? == Value::Bool(true) {
                return Ok(Value::Number(index as f64));
            }
        }
        Ok(Value::Number(-1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness() -> (Runtime, Document) {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("<!doctype html><title>Array lastIndexOf</title>");
        for source in [
            include_str!("../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../tests/upstream/test262/harness/propertyHelper.js"),
        ] {
            runtime.execute(source, &mut document).unwrap();
        }
        (runtime, document)
    }

    fn clean(runtime: &Runtime) {
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);
        assert_eq!(runtime.eval_depth, 0);
        assert_eq!(runtime.json_depth, 0);
        assert!(runtime.frames.is_empty());
    }

    #[test]
    fn frozen_last_index_of_cases_cover_holes_conversion_and_live_properties() {
        for fixture in include_str!("../../tests/conformance/array-last-index-of.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (name, source) = fixture.split_once('\n').unwrap();
            for strict in [false, true] {
                let (mut runtime, mut doc) = harness();
                let result = if strict {
                    runtime.execute_strict(source, &mut doc)
                } else {
                    runtime.execute(source, &mut doc)
                };
                assert!(result.is_ok(), "{name}, strict={strict}: {result:?}");
                clean(&runtime);
            }
        }
    }

    #[test]
    fn last_index_of_callbacks_and_sparse_ranges_share_limits() {
        for source in [
            "var o={};Object.defineProperty(o,'length',{get:function(){return Array.prototype.lastIndexOf.call(o,1);}});Array.prototype.lastIndexOf.call(o,1);",
            "var o={length:{valueOf:function(){return Array.prototype.lastIndexOf.call(o,1);}}};Array.prototype.lastIndexOf.call(o,1);",
            "var p={valueOf:function(){return [1].lastIndexOf(1,p);}};[1].lastIndexOf(1,p);",
            "var o={length:1};Object.defineProperty(o,'0',{get:function(){return Array.prototype.lastIndexOf.call(o,1);}});Array.prototype.lastIndexOf.call(o,1);",
            "Array.prototype.lastIndexOf.call({length:Infinity},1);",
        ] {
            let (mut runtime, mut doc) = harness();
            let source = format!("var caught=false;try{{{source}}}catch(e){{caught=true;}}");
            assert!(
                runtime
                    .execute(&source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            clean(&runtime);
        }
    }

    #[test]
    fn last_index_of_heap_refusal_precedes_index_getter() {
        let (mut runtime, mut doc) = harness();
        let object = runtime.execute("var calls=0,o={length:1};Object.defineProperty(o,'0',{get:function(){calls++;return 'x';}});o", &mut doc).unwrap();
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .array_last_index_of(object, Value::String("x".into()), None, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(0.0));
        clean(&runtime);
    }
}
