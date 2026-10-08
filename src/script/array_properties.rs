//! Array exotic properties. Logical length does not allocate absent elements.
use super::*;

pub(super) struct ArrayLength {
    pub value: u32,
    pub writable: bool,
}

impl Runtime {
    pub(super) fn define_property_key(
        &mut self,
        receiver: &Value,
        key: &PropertyKey,
        desc: PropertyDescriptor,
        doc: &mut Document,
    ) -> Result<bool> {
        if let PropertyKey::String(key) = key
            && let typed_array::Exotic::Handled(result) =
                self.typed_array_define(receiver, key, &desc, doc)?
        {
            return Ok(result);
        }
        self.define_property_key_ordinary(receiver, key, desc, doc)
    }

    pub(super) fn define_property_key_for_set(
        &mut self,
        receiver: &Value,
        key: &mut typed_array::SetKey<'_>,
        desc: PropertyDescriptor,
        doc: &mut Document,
    ) -> Result<bool> {
        if let typed_array::Exotic::Handled(result) =
            self.typed_array_define_for_set(receiver, key, &desc, doc)?
        {
            return Ok(result);
        }
        self.define_property_key_ordinary(receiver, key.property(), desc, doc)
    }

    fn define_property_key_ordinary(
        &mut self,
        receiver: &Value,
        key: &PropertyKey,
        desc: PropertyDescriptor,
        doc: &mut Document,
    ) -> Result<bool> {
        if let Value::Array(id) = receiver
            && key
                .as_string()
                .is_some_and(|key| key == &JsString::from("length"))
        {
            return self.array_set_length(*id, desc, doc);
        }
        self.define_own_key(receiver, key, desc)
    }

    // Ordinary descriptor validation has already succeeded. Dense storage is
    // only a cache for ordinary data elements; all other elements live in the
    // property table. Neither path allocates gaps in a sparse array.
    pub(super) fn store_array_property(
        &mut self,
        id: usize,
        key: &JsString,
        property: &Property,
    ) -> Result<bool> {
        if key == &JsString::from("length") {
            let PropertyValue::Data {
                value: Value::Number(length),
                writable,
            } = property.value
            else {
                unreachable!("validated array length descriptor")
            };
            self.array_lengths[id].value = length as u32;
            self.array_lengths[id].writable = writable;
            return Ok(true);
        }
        let Some(index) = json_array_index(key).map(|index| index as usize) else {
            return Ok(false);
        };
        if self.objects[self.array_properties[id]].contains_key(PropertyKey::String(key.clone())) {
            return Ok(false);
        }
        let PropertyValue::Data {
            value,
            writable: true,
        } = &property.value
        else {
            return Ok(false);
        };
        if !property.enumerable || !property.configurable {
            return Ok(false);
        }
        if index < self.arrays[id].len() {
            self.arrays[id][index] = value.clone();
            self.array_holes[id].remove(&index);
            return Ok(true);
        }
        if index == self.arrays[id].len() && index < 65_536 {
            self.charge(std::mem::size_of::<Value>())?;
            self.arrays[id]
                .try_reserve(1)
                .map_err(|_| ScriptError::resource("array element allocation failed"))?;
            self.arrays[id].push(value.clone());
            return Ok(true);
        }
        Ok(false)
    }

    fn array_set_length(
        &mut self,
        id: usize,
        mut desc: PropertyDescriptor,
        doc: &mut Document,
    ) -> Result<bool> {
        let receiver = Value::Array(id);
        let key = PropertyKey::String(JsString::from("length"));
        let Some(value) = desc.value.clone() else {
            return self.ordinary_define_own_key(&receiver, &key, desc);
        };
        // These are two distinct, observable conversions, in this order.
        let new_length = to_i32(self.number_value(value.clone(), doc)?) as u32;
        let number = self.number_value(value, doc)?;
        if number != new_length as f64 {
            return Err(ScriptError::range_error("invalid array length"));
        }
        desc.value = Some(Value::Number(new_length as f64));
        // Conversion hooks may have changed both length and writability.
        let old_length = self.array_lengths[id].value;
        if new_length >= old_length {
            return self.ordinary_define_own_key(&receiver, &key, desc);
        }
        if !self.array_lengths[id].writable {
            return Ok(false);
        }
        let final_readonly = desc.writable == Some(false);
        if final_readonly {
            desc.writable = Some(true);
        }
        // Only actual own keys participate. A 2^32-1 length with two elements
        // costs two deletions, not billions of gap visits or allocations.
        let keys = self.own_keys(&receiver)?;
        self.work(1 + self.arrays[id].len() + self.array_holes[id].len())?;
        if !self.ordinary_define_own_key(&receiver, &key, desc)? {
            return Ok(false);
        }
        for key in keys.into_iter().rev() {
            let Some(index) = json_array_index(&key).filter(|index| *index >= new_length) else {
                continue;
            };
            match self.delete_property(receiver.clone(), &key) {
                Ok(true) => {}
                Ok(false) => {
                    self.array_lengths[id].value = index + 1;
                    self.array_lengths[id].writable = !final_readonly;
                    self.trim_array_storage(id);
                    return Ok(false);
                }
                Err(error) => {
                    // Resource termination is not catchable by script. Keep
                    // the surviving storage valid for later runtime cleanup.
                    self.array_lengths[id].value = old_length;
                    return Err(error);
                }
            }
        }
        self.array_lengths[id].writable = !final_readonly;
        self.trim_array_storage(id);
        Ok(true)
    }

    fn trim_array_storage(&mut self, id: usize) {
        let length = self.array_lengths[id].value as usize;
        self.arrays[id].truncate(length);
        self.array_holes[id].retain(|index| *index < length);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean(runtime: &Runtime) {
        assert_eq!(runtime.calls, 0);
        assert_eq!(runtime.stack_units, 0);
        assert_eq!(runtime.eval_depth, 0);
        assert_eq!(runtime.json_depth, 0);
        assert!(runtime.frames.is_empty());
    }

    #[test]
    fn frozen_descriptor_cases_preserve_all_before_sources() {
        for fixture in include_str!("../../tests/conformance/array-descriptors.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (name, source) = fixture.split_once('\n').unwrap();
            for strict in [false, true] {
                let mut runtime = Runtime::new();
                let mut document = Document::parse("");
                for helper in [
                    include_str!("../../tests/upstream/test262/harness/sta.js"),
                    include_str!("../../tests/upstream/test262/harness/assert.js"),
                    include_str!("../../tests/upstream/test262/harness/propertyHelper.js"),
                ] {
                    runtime.execute(helper, &mut document).unwrap();
                }
                let result = if strict {
                    runtime.execute_strict(source, &mut document)
                } else {
                    runtime.execute(source, &mut document)
                };
                assert!(result.is_ok(), "{name} strict={strict}: {result:?}");
                clean(&runtime);
            }
        }
    }

    #[test]
    fn reflect_definition_reports_rejection_and_preserves_conversion_order() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        for helper in [
            include_str!("../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../tests/upstream/test262/harness/propertyHelper.js"),
        ] {
            runtime.execute(helper, &mut document).unwrap();
        }
        runtime.execute(r#"
            var a=[1,2,3],log='',k={toString:function(){log+='k';return 'length';}};
            var d={get value(){log+='v';return {valueOf:function(){log+='n';return 1;}};}};
            Object.defineProperty(a,'1',{configurable:false});
            assert.sameValue(Reflect.defineProperty(a,k,d),false);
            assert.sameValue(log,'kvnn');assert.sameValue(a.length,2);
            assert.sameValue(a.hasOwnProperty('2'),false);
            assert.throws(TypeError,function(){Reflect.defineProperty(null,k,d);});
            assert.sameValue(log,'kvnn');
            var symbol=Symbol();assert.sameValue(Reflect.defineProperty(a,symbol,{value:4}),true);
            assert.sameValue(a[symbol],4);
            assert.throws(TypeError,function(){Reflect.defineProperty(a,'x',{get:0});});
            verifyProperty(Reflect.defineProperty,'name',{value:'defineProperty',writable:false,enumerable:false,configurable:true});
            verifyProperty(Reflect.defineProperty,'length',{value:3,writable:false,enumerable:false,configurable:true});
            assert.throws(TypeError,function(){new Reflect.defineProperty(a,'x',{});});
        "#, &mut document).unwrap();
        clean(&runtime);
    }

    #[test]
    fn sparse_array_length_and_shrink_use_actual_storage() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let Value::Array(id) = runtime
            .execute(
                "var a=new Array(4294967295);a[4294967294]=7;a[4000000000]=8;a[2]=9;a;",
                &mut document,
            )
            .unwrap()
        else {
            panic!("array")
        };
        assert!(runtime.arrays[id].is_empty());
        assert!(runtime.array_holes[id].is_empty());
        assert_eq!(runtime.array_lengths[id].value, u32::MAX);
        assert_eq!(runtime.own_keys(&Value::Array(id)).unwrap().len(), 4);
        runtime.execute("a.length=3;", &mut document).unwrap();
        assert_eq!(runtime.array_lengths[id].value, 3);
        assert!(
            runtime
                .own_property(&Value::Array(id), &"4294967294".into())
                .is_none()
        );
        assert_eq!(
            runtime.get(Value::Array(id), "2", &mut document).unwrap(),
            Value::Number(9.0)
        );
        clean(&runtime);
    }

    #[test]
    fn sparse_property_heap_refusal_precedes_storage_and_length_changes() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let array = runtime.array(Vec::new()).unwrap();
        let Value::Array(id) = array else {
            unreachable!()
        };
        runtime.allocated = MAX_HEAP;
        let error = runtime
            .define_property_key(
                &array,
                &PropertyKey::String("4294967294".into()),
                PropertyDescriptor::data_property(Value::Number(1.0), true, true, true),
                &mut document,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.array_lengths[id].value, 0);
        assert!(runtime.arrays[id].is_empty());
        assert!(
            runtime.objects[runtime.array_properties[id]]
                .values
                .is_empty()
        );
        clean(&runtime);
    }

    #[test]
    fn array_length_conversion_recursion_cannot_be_caught() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        let result = runtime.execute(
            "var caught=false,a=[],v={valueOf:function(){Object.defineProperty(a,'length',{value:v});return 0;}};try{a.length=v;}catch(e){caught=true;}",
            &mut document,
        );
        assert!(result.unwrap_err().is_resource_limit());
        assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
        clean(&runtime);
    }

    #[test]
    fn sparse_json_key_snapshots_refuse_before_indexed_callbacks() {
        for source in [
            "var a=new Array(4294967295);Object.defineProperty(a,'0',{get:function(){hits++;return 1;}});JSON.stringify(a);",
            "JSON.parse('[0,1]',function(k,v){if(k==='0'){this[1]=new Array(4294967295);Object.defineProperty(this[1],'0',{get:function(){hits++;return 1;}});}return v;});",
        ] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let source = format!("var hits=0,caught=false;try{{{source}}}catch(e){{caught=true;}}");
            assert!(
                runtime
                    .execute(&source, &mut document)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(1, "hits").unwrap().1, Value::Number(0.0));
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            clean(&runtime);
        }
    }

    #[test]
    fn interrupted_shrink_keeps_surviving_indices_below_length() {
        for steps in 1..100 {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            let array = runtime.array(vec![Value::Number(1.0); 12]).unwrap();
            let Value::Array(id) = array else {
                unreachable!()
            };
            runtime.steps = steps;
            let result = runtime.array_set_length(
                id,
                PropertyDescriptor {
                    value: Some(Value::Number(0.0)),
                    ..PropertyDescriptor::default()
                },
                &mut document,
            );
            assert!(result.is_ok() || result.unwrap_err().is_resource_limit());
            for index in 0..12 {
                if runtime
                    .own_property(&array, &index.to_string().into())
                    .is_some()
                {
                    assert!(index < runtime.array_lengths[id].value);
                }
            }
            clean(&runtime);
        }
    }
}
