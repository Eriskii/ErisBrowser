//! Observable RegExp splitting and species construction.
use super::*;

impl Runtime {
    pub(super) fn initialize_regexp_symbols(&mut self) -> Result<()> {
        let split = self.intrinsic_function("RegExp.symbolSplit", "[Symbol.split]", 2)?;
        let key = self.well_known_key("split");
        self.charge(256)?;
        self.objects[self.prototypes["RegExp"]]
            .insert_property(key, Property::data(split, true, false, true));
        let getter = self.intrinsic_function("RegExp.species", "get [Symbol.species]", 0)?;
        let key = self.well_known_key("species");
        self.charge(256)?;
        self.objects[self.native_properties["RegExp"]].insert_property(
            key,
            Property {
                value: PropertyValue::Accessor {
                    get: getter,
                    set: Value::Undefined,
                },
                enumerable: false,
                configurable: true,
            },
        );
        Ok(())
    }

    fn regexp_species(&mut self, receiver: Value, doc: &mut Document) -> Result<Value> {
        let default = Self::native("RegExp", Value::Window);
        let constructor = self.get(receiver, "constructor", doc)?;
        if constructor == Value::Undefined {
            return Ok(default);
        }
        if !js_object(&constructor) {
            return Err(ScriptError::type_error(
                "RegExp constructor must be an object",
            ));
        }
        let species = self.get_property_key(constructor, &self.well_known_key("species"), doc)?;
        if matches!(species, Value::Null | Value::Undefined) {
            Ok(default)
        } else if self.is_constructor(species.clone())? {
            Ok(species)
        } else {
            Err(ScriptError::type_error(
                "RegExp species must be a constructor",
            ))
        }
    }

    pub(super) fn regexp_symbol_split(
        &mut self,
        receiver: Value,
        argument: Value,
        limit: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        if !js_object(&receiver) {
            return Err(ScriptError::type_error(
                "RegExp split receiver must be an object",
            ));
        }
        let text = self.string_hint(argument, doc)?;
        let constructor = self.regexp_species(receiver.clone(), doc)?;
        let flags = self.get(receiver.clone(), "flags", doc)?;
        let flags = self.string_hint(flags, doc)?;
        self.work(flags.len().saturating_mul(3).saturating_add(1))?;
        let unicode = flags.units().contains(&0x75) || flags.units().contains(&0x76);
        let flags = if flags.units().contains(&0x79) {
            Value::String(flags)
        } else {
            let length = flags.len().saturating_add(1);
            if length > MAX_STRING {
                return Err(ScriptError::resource("script string limit exceeded"));
            }
            self.charge(length * 2)?;
            let mut units = Vec::new();
            units
                .try_reserve_exact(length)
                .map_err(|_| ScriptError::resource("RegExp split flags allocation failed"))?;
            units.extend_from_slice(flags.units());
            units.push(0x79);
            self.string(units)?
        };
        self.charge(2 * std::mem::size_of::<Value>())?;
        let splitter = self.construct_with_target(
            constructor.clone(),
            vec![receiver, flags],
            constructor,
            doc,
        )?;
        // Allocate the fresh output before limit conversion. It never escapes to
        // author code during construction, so indexed appends cannot invoke setters.
        let array = self.array(Vec::new())?;
        let Value::Array(id) = array else {
            unreachable!()
        };
        let limit = if limit == Value::Undefined {
            u32::MAX as usize
        } else {
            to_i32(self.number_value(limit, doc)?) as u32 as usize
        };
        if limit == 0 {
            return Ok(array);
        }
        if text.is_empty() {
            if self.regexp_exec(splitter, &text, doc)? == Value::Null {
                self.split_append(id, Value::String(text))?;
            }
            return Ok(array);
        }
        let mut previous = 0;
        let mut position = 0;
        while position < text.len() {
            self.tick()?;
            self.regexp_last_index(splitter.clone(), position, doc)?;
            let result = self.regexp_exec(splitter.clone(), &text, doc)?;
            if result == Value::Null {
                position = split_next_index(&text, position, unicode);
                continue;
            }
            let end = self.get(splitter.clone(), "lastIndex", doc)?;
            let end = integer_or_infinity(self.number_value(end, doc)?)
                .clamp(0.0, 9007199254740991.0)
                .min(text.len() as f64) as usize;
            if end == previous {
                position = split_next_index(&text, position, unicode);
                continue;
            }
            let piece = self.split_substring(&text, previous, position)?;
            self.split_append(id, piece)?;
            if self.arrays[id].len() == limit {
                return Ok(array);
            }
            previous = end;
            let length = self.get(result.clone(), "length", doc)?;
            let length = integer_or_infinity(self.number_value(length, doc)?)
                .clamp(0.0, 9007199254740991.0) as usize;
            for index in 1..length {
                self.tick()?;
                self.charge(32)?;
                let capture = self.get(result.clone(), &index.to_string(), doc)?;
                self.split_append(id, capture)?;
                if self.arrays[id].len() == limit {
                    return Ok(array);
                }
            }
            position = previous;
        }
        let tail = self.split_substring(&text, previous, text.len())?;
        self.split_append(id, tail)?;
        Ok(array)
    }

    fn split_substring(&mut self, text: &JsString, start: usize, end: usize) -> Result<Value> {
        let length = end - start;
        self.work(length.saturating_add(1))?;
        self.charge(length.saturating_mul(2))?;
        self.string(&text.units()[start..end])
    }

    fn split_append(&mut self, id: usize, value: Value) -> Result<()> {
        if self.arrays[id].len() >= 65536 {
            return Err(ScriptError::resource("array length limit exceeded"));
        }
        self.tick()?;
        self.charge(std::mem::size_of::<Value>())?;
        self.arrays[id]
            .try_reserve(1)
            .map_err(|_| ScriptError::resource("RegExp split array allocation failed"))?;
        self.arrays[id].push(value);
        Ok(())
    }
}

fn split_next_index(text: &JsString, index: usize, unicode: bool) -> usize {
    let pair = unicode
        && text
            .units()
            .get(index)
            .is_some_and(|u| (0xd800..=0xdbff).contains(u))
        && text
            .units()
            .get(index + 1)
            .is_some_and(|u| (0xdc00..=0xdfff).contains(u));
    index + if pair { 2 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness() -> (Runtime, Document) {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<!doctype html><title>RegExp split</title>");
        for source in [
            include_str!("../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../tests/upstream/test262/harness/propertyHelper.js"),
        ] {
            runtime.execute(source, &mut doc).unwrap();
        }
        (runtime, doc)
    }

    fn clean(runtime: &Runtime) {
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);
        assert_eq!(runtime.eval_depth, 0);
        assert_eq!(runtime.json_depth, 0);
        assert!(runtime.frames.is_empty());
    }

    #[test]
    fn frozen_split_cases_cover_species_order_execution_and_utf16() {
        for fixture in include_str!("../../tests/conformance/regexp-split.js")
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
                if name == "captures-ignore-array-prototype-setter" {
                    // Preserve the original fixture's unrelated prerequisite.
                    let error = result.unwrap_err();
                    assert_eq!(error.name(), "UnsupportedFeature");
                    assert!(
                        error
                            .to_string()
                            .contains("array indexed/length descriptor")
                    );
                } else {
                    assert!(result.is_ok(), "{name}, strict={strict}: {result:?}");
                }
                clean(&runtime);
            }
        }
        let (mut runtime, mut doc) = harness();
        runtime.execute("var called=false;Object.defineProperty(Object.prototype,'1',{set:function(){called=true;},configurable:true});var a='abc'.split(/(b)/);assert.sameValue(a[1],'b');assert.sameValue(a.length,3);assert.sameValue(called,false);", &mut doc).unwrap();
        clean(&runtime);
    }

    #[test]
    fn split_callbacks_and_hostile_capture_lengths_share_limits() {
        for source in [
            "var a={toString:function(){return RegExp.prototype[Symbol.split].call(/x/,a);}};RegExp.prototype[Symbol.split].call(/x/,a);",
            "var r=/x/;Object.defineProperty(r,'constructor',{get:function(){return ''.split(r);}});''.split(r);",
            "var r=/x/;Object.defineProperty(r,'flags',{get:function(){return ''.split(r);}});''.split(r);",
            "var r=/x/,c={};r.constructor=c;Object.defineProperty(c,Symbol.species,{get:function(){return ''.split(r);}});''.split(r);",
            "var r=/x/,c={};r.constructor=c;c[Symbol.species]=function(){return ''.split(r);};''.split(r);",
            "var r={flags:'',constructor:{}};r.constructor[Symbol.species]=function(){return {exec:function(){return RegExp.prototype[Symbol.split].call(r,'x');}};};RegExp.prototype[Symbol.split].call(r,'x');",
            "var r={flags:'',constructor:{}};r.constructor[Symbol.species]=function(){return {exec:function(){this.lastIndex=1;return {length:Infinity};}};};RegExp.prototype[Symbol.split].call(r,'x');",
            "var r={flags:'',constructor:{}};r.constructor[Symbol.species]=function(){var n=0;return {exec:function(){this.lastIndex=(n++%2)?0:1;return {};}};};RegExp.prototype[Symbol.split].call(r,'xy');",
        ] {
            let (mut runtime, mut doc) = harness();
            let error = runtime.execute(source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit(), "{source}: {error}");
            clean(&runtime);
        }
    }

    #[test]
    fn split_heap_exhaustion_prevents_species_construction() {
        let (mut runtime, mut doc) = harness();
        let function = runtime.execute("var calls=0,r={flags:'',constructor:{}};r.constructor[Symbol.species]=function(){calls++;return {exec:function(){return null;}};};RegExp.prototype[Symbol.split]", &mut doc).unwrap();
        let receiver = runtime.execute("r", &mut doc).unwrap();
        runtime.allocated = MAX_HEAP;
        let error = runtime
            .call(
                function,
                vec![Value::String("x".into())],
                receiver,
                &mut doc,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(
            runtime.environments[0].bindings["calls"].value,
            Value::Number(0.0)
        );
        clean(&runtime);
    }
}
