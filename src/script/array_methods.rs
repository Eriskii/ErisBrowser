//! Array methods expressed through live property operations, not dense storage.
use super::*;

#[cfg(test)]
mod find_tests;

const MAX_ARRAY_LIKE_LENGTH: u64 = 9_007_199_254_740_991;

impl Runtime {
    pub(super) fn array_method(
        &mut self,
        receiver: Value,
        name: &str,
        args: Vec<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let object = self.coerce_object(receiver)?;
        if self.property_object(&object).is_none() {
            return Err(ScriptError::unsupported(
                "host array-like methods are not implemented",
            ));
        }
        self.charge(64)?;
        let length_key = JsString::from("length");
        let length_value = self.reduce_get(&object, &length_key, doc)?;
        let length = integer_or_infinity(self.number_value(length_value, doc)?)
            .clamp(0.0, MAX_ARRAY_LIKE_LENGTH as f64) as u64;
        match name {
            "push" | "unshift" => {
                let count = u64::try_from(args.len())
                    .map_err(|_| ScriptError::type_error("array-like length overflow"))?;
                let new_length = length
                    .checked_add(count)
                    .filter(|value| *value <= MAX_ARRAY_LIKE_LENGTH)
                    .ok_or_else(|| ScriptError::type_error("array-like length overflow"))?;
                if name == "unshift" && count != 0 {
                    for from in (0..length).rev() {
                        self.tick()?;
                        let source = self.reduce_index_key(from)?;
                        let destination = self.reduce_index_key(from + count)?;
                        if self.reduce_property_in(&object, &source, doc)?.is_some() {
                            let value = self.reduce_get(&object, &source, doc)?;
                            self.set_key_strict(object.clone(), &destination, value, true, doc)?;
                        } else {
                            self.array_method_delete(&object, &destination)?;
                        }
                    }
                }
                let first = if name == "push" { length } else { 0 };
                for (offset, value) in args.into_iter().enumerate() {
                    self.tick()?;
                    let key = self.reduce_index_key(first + offset as u64)?;
                    self.set_key_strict(object.clone(), &key, value, true, doc)?;
                }
                self.set_key_strict(
                    object,
                    &length_key,
                    Value::Number(new_length as f64),
                    true,
                    doc,
                )?;
                Ok(Value::Number(new_length as f64))
            }
            "pop" | "shift" => {
                if length == 0 {
                    self.set_key_strict(object, &length_key, Value::Number(0.0), true, doc)?;
                    return Ok(Value::Undefined);
                }
                let last = length - 1;
                let key = self.reduce_index_key(if name == "pop" { last } else { 0 })?;
                let result = self.reduce_get(&object, &key, doc)?;
                if name == "shift" {
                    for from in 1..length {
                        self.tick()?;
                        let source = self.reduce_index_key(from)?;
                        let destination = self.reduce_index_key(from - 1)?;
                        if self.reduce_property_in(&object, &source, doc)?.is_some() {
                            let value = self.reduce_get(&object, &source, doc)?;
                            self.set_key_strict(object.clone(), &destination, value, true, doc)?;
                        } else {
                            self.array_method_delete(&object, &destination)?;
                        }
                    }
                }
                let key = self.reduce_index_key(last)?;
                self.array_method_delete(&object, &key)?;
                self.set_key_strict(object, &length_key, Value::Number(last as f64), true, doc)?;
                Ok(result)
            }
            "join" => {
                let separator = match args.first() {
                    None | Some(Value::Undefined) => {
                        self.charge(32)?;
                        JsString::from(",")
                    }
                    Some(value) => self.string_hint(value.clone(), doc)?,
                };
                self.work(self.active_array_joins.len().saturating_add(1))?;
                if self.active_array_joins.contains(&object) {
                    return Ok(Value::String(JsString::default()));
                }
                self.charge(32 + std::mem::size_of::<Value>())?;
                self.active_array_joins
                    .try_reserve_exact(1)
                    .map_err(|_| ScriptError::resource("array join stack allocation failed"))?;
                self.active_array_joins.push(object.clone());
                let completion = (|| {
                    let mut output = Vec::new();
                    for index in 0..length {
                        self.tick()?;
                        // Append before Get: a budget failure cannot perform
                        // the next indexed getter's effects.
                        if index != 0 {
                            self.array_join_append(&mut output, &separator)?;
                        }
                        let key = self.reduce_index_key(index)?;
                        let value = self.reduce_get(&object, &key, doc)?;
                        if !matches!(value, Value::Null | Value::Undefined) {
                            let text = self.string_hint(value, doc)?;
                            self.array_join_append(&mut output, &text)?;
                        }
                    }
                    // Vec -> shared UTF-16 storage may copy.
                    self.charge(output.len().saturating_mul(2).saturating_add(24))?;
                    Ok(Value::String(output.into()))
                })();
                self.active_array_joins.pop();
                completion
            }
            "includes" | "indexOf" => {
                let includes = name == "includes";
                let missing = if includes {
                    Value::Bool(false)
                } else {
                    Value::Number(-1.0)
                };
                if length == 0 {
                    return Ok(missing);
                }
                let start = self.array_method_index(
                    args.get(1).cloned().unwrap_or(Value::Undefined),
                    length,
                    doc,
                )?;
                let search = args.first().cloned().unwrap_or(Value::Undefined);
                for index in start..length {
                    self.tick()?;
                    let key = self.reduce_index_key(index)?;
                    if !includes && self.reduce_property_in(&object, &key, doc)?.is_none() {
                        continue;
                    }
                    let value = self.reduce_get(&object, &key, doc)?;
                    let equal = if includes
                        && matches!((&search, &value), (Value::Number(a), Value::Number(b)) if a.is_nan() && b.is_nan())
                    {
                        true
                    } else {
                        self.binary_value("===", search.clone(), value, doc)? == Value::Bool(true)
                    };
                    if equal {
                        return Ok(if includes {
                            Value::Bool(true)
                        } else {
                            Value::Number(index as f64)
                        });
                    }
                }
                Ok(missing)
            }
            "slice" => {
                let start = self.array_method_index(
                    args.first().cloned().unwrap_or(Value::Undefined),
                    length,
                    doc,
                )?;
                let end = match args.get(1) {
                    None | Some(Value::Undefined) => length,
                    Some(value) => self.array_method_index(value.clone(), length, doc)?,
                };
                let count = end.saturating_sub(start);
                let result = self.array_method_result(&object, count, doc)?;
                for offset in 0..count {
                    self.tick()?;
                    let key = self.reduce_index_key(start + offset)?;
                    if self.reduce_property_in(&object, &key, doc)?.is_some() {
                        let value = self.reduce_get(&object, &key, doc)?;
                        self.array_method_create(&result, offset, value, doc)?;
                    }
                }
                self.set_key_strict(
                    result.clone(),
                    &length_key,
                    Value::Number(count as f64),
                    true,
                    doc,
                )?;
                Ok(result)
            }
            "find" | "findIndex" | "findLast" | "findLastIndex" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                if !json_callable(&callback) {
                    return Err(ScriptError::type_error("array callback must be callable"));
                }
                let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
                let descending = matches!(name, "findLast" | "findLastIndex");
                let return_index = matches!(name, "findIndex" | "findLastIndex");
                for offset in 0..length {
                    self.tick()?;
                    let index = if descending {
                        length - 1 - offset
                    } else {
                        offset
                    };
                    let key = self.reduce_index_key(index)?;
                    // FindViaPredicate visits holes too. Read each value live
                    // and retain it even if the predicate changes this slot.
                    let value = self.reduce_get(&object, &key, doc)?;
                    self.charge(32 + 3 * std::mem::size_of::<Value>())?;
                    let mut parameters = Vec::new();
                    parameters
                        .try_reserve_exact(3)
                        .map_err(|_| ScriptError::resource("array callback allocation failed"))?;
                    parameters.extend([value.clone(), Value::Number(index as f64), object.clone()]);
                    let returned =
                        self.call(callback.clone(), parameters, this_arg.clone(), doc)?;
                    if returned.truthy() {
                        return Ok(if return_index {
                            Value::Number(index as f64)
                        } else {
                            value
                        });
                    }
                }
                Ok(if return_index {
                    Value::Number(-1.0)
                } else {
                    Value::Undefined
                })
            }
            "forEach" | "map" | "filter" | "every" | "some" => {
                let callback = args.first().cloned().unwrap_or(Value::Undefined);
                if !json_callable(&callback) {
                    return Err(ScriptError::type_error("array callback must be callable"));
                }
                let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
                let result = match name {
                    "map" => Some(self.array_method_result(&object, length, doc)?),
                    "filter" => Some(self.array_method_result(&object, 0, doc)?),
                    _ => None,
                };
                let mut selected = 0;
                for index in 0..length {
                    self.tick()?;
                    let key = self.reduce_index_key(index)?;
                    if self.reduce_property_in(&object, &key, doc)?.is_none() {
                        continue;
                    }
                    let value = self.reduce_get(&object, &key, doc)?;
                    self.charge(32 + 3 * std::mem::size_of::<Value>())?;
                    let mut parameters = Vec::new();
                    parameters
                        .try_reserve_exact(3)
                        .map_err(|_| ScriptError::resource("array callback allocation failed"))?;
                    parameters.extend([value.clone(), Value::Number(index as f64), object.clone()]);
                    let returned =
                        self.call(callback.clone(), parameters, this_arg.clone(), doc)?;
                    match (&result, name) {
                        (None, "every") if !returned.truthy() => return Ok(Value::Bool(false)),
                        (None, "some") if returned.truthy() => return Ok(Value::Bool(true)),
                        (Some(result), "map") => {
                            self.array_method_create(result, index, returned, doc)?
                        }
                        (Some(result), "filter") if returned.truthy() => {
                            self.array_method_create(result, selected, value, doc)?;
                            selected += 1;
                        }
                        _ => {}
                    }
                }
                if let Some(result) = result {
                    let result_length = if name == "map" { length } else { selected };
                    self.set_key_strict(
                        result.clone(),
                        &length_key,
                        Value::Number(result_length as f64),
                        true,
                        doc,
                    )?;
                    Ok(result)
                } else {
                    Ok(match name {
                        "every" => Value::Bool(true),
                        "some" => Value::Bool(false),
                        _ => Value::Undefined,
                    })
                }
            }
            _ => Err(ScriptError::unsupported("array method is not implemented")),
        }
    }

    fn array_method_index(&mut self, value: Value, length: u64, doc: &mut Document) -> Result<u64> {
        let number = integer_or_infinity(self.number_value(value, doc)?);
        Ok(if number < 0.0 {
            (length as f64 + number).max(0.0) as u64
        } else {
            number.min(length as f64) as u64
        })
    }

    fn array_method_delete(&mut self, object: &Value, key: &JsString) -> Result<()> {
        if self.delete_property(object.clone(), key)? {
            Ok(())
        } else {
            Err(ScriptError::type_error(
                "array method cannot delete property",
            ))
        }
    }

    fn array_method_create(
        &mut self,
        object: &Value,
        index: u64,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        let key = PropertyKey::String(self.reduce_index_key(index)?);
        if self.define_property_key(
            object,
            &key,
            PropertyDescriptor::data_property(value, true, true, true),
            doc,
        )? {
            Ok(())
        } else {
            Err(ScriptError::type_error(
                "array result property cannot be defined",
            ))
        }
    }

    fn array_method_result(
        &mut self,
        original: &Value,
        length: u64,
        doc: &mut Document,
    ) -> Result<Value> {
        if matches!(original, Value::Array(_)) {
            self.charge(64)?;
            let constructor = self.reduce_get(original, &JsString::from("constructor"), doc)?;
            let constructor = if js_object(&constructor) {
                let key = self.well_known_key("species");
                match self.get_property_key(constructor, &key, doc)? {
                    Value::Null => Value::Undefined,
                    value => value,
                }
            } else {
                constructor
            };
            if !matches!(constructor, Value::Undefined)
                && !matches!(&constructor, Value::Native(native) if native.name == "Array")
            {
                if !self.is_constructor(constructor)? {
                    return Err(ScriptError::type_error(
                        "array species is not a constructor",
                    ));
                }
                return Err(ScriptError::unsupported(
                    "custom array species construction is not implemented",
                ));
            }
        }
        if length > u64::from(u32::MAX) {
            return Err(ScriptError::range_error("invalid array length"));
        }
        let result = self.array(Vec::new())?;
        self.charge(64)?;
        self.set_key_strict(
            result.clone(),
            &JsString::from("length"),
            Value::Number(length as f64),
            true,
            doc,
        )?;
        Ok(result)
    }

    pub(super) fn array_join_append(
        &mut self,
        output: &mut Vec<u16>,
        text: &JsString,
    ) -> Result<()> {
        let length = output
            .len()
            .checked_add(text.len())
            .filter(|length| *length <= MAX_STRING)
            .ok_or_else(|| ScriptError::resource("script string limit exceeded"))?;
        self.work(1 + text.len() / 8)?;
        if length > output.capacity() {
            let capacity = length
                .max(output.capacity().saturating_mul(2))
                .clamp(16, MAX_STRING);
            // Charge the complete replacement allocation and potential copy,
            // not only the increment, before requesting checked capacity.
            self.work(1 + output.len() / 8)?;
            self.charge(capacity.saturating_mul(2))?;
            output
                .try_reserve_exact(capacity - output.len())
                .map_err(|_| ScriptError::resource("array join allocation failed"))?;
        }
        output.extend_from_slice(text.units());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness() -> (Runtime, Document) {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("<!doctype html><title>Array methods</title>");
        for source in [
            include_str!("../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../tests/upstream/test262/harness/assert.js"),
        ] {
            runtime.execute(source, &mut document).unwrap();
        }
        (runtime, document)
    }

    fn check(source: &str) {
        for strict in [false, true] {
            let (mut runtime, mut document) = harness();
            let result = if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            };
            assert!(result.is_ok(), "strict={strict}: {result:?}\n{source}");
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert!(runtime.frames.is_empty());
            assert!(runtime.active_array_joins.is_empty());
        }
    }

    #[test]
    fn generic_array_mutators_use_observable_properties_and_lengths() {
        check(
            r#"
            var p=Array.prototype,o={0:'a',1:'b',length:2};
            assert.sameValue(p.push.call(o,'c'),3);assert.sameValue(o[2],'c');
            assert.sameValue(p.pop.call(o),'c');assert.sameValue(o.length,2);
            assert.sameValue(p.unshift.call(o,'x','y'),4);
            assert.sameValue(p.join.call(o,'|'),'x|y|a|b');
            assert.sameValue(p.shift.call(o),'x');
            assert.sameValue(p.join.call(o,'|'),'y|a|b');assert.sameValue(3 in o,false);
            var locked={length:0};Object.defineProperty(locked,'length',{writable:false});
            assert.throws(TypeError,function(){p.push.call(locked);});
            assert.throws(TypeError,function(){p.unshift.call(locked);});
            assert.throws(TypeError,function(){p.pop.call(locked);});
            assert.throws(TypeError,function(){p.shift.call(locked);});
            assert.throws(TypeError,function(){p.pop.call('x');});
        "#,
        );
    }

    #[test]
    fn generic_array_mutators_preserve_live_holes_getters_and_partial_errors() {
        check(
            r#"
            var p=Array.prototype,o=Object.create({1:'inherited'});o[0]='a';o.length=3;
            assert.sameValue(p.shift.call(o),'a');assert.sameValue(o[0],'inherited');
            assert.sameValue(Object.getOwnPropertyDescriptor(o,'1'),undefined);
            assert.sameValue(o.length,2);
            var moved={0:'a',length:3},log='';
            Object.defineProperty(moved,'2',{configurable:true,get:function(){log+='g';delete moved[0];return 'c';}});
            assert.sameValue(p.unshift.call(moved,'x'),4);assert.sameValue(log,'g');
            assert.sameValue(moved[0],'x');assert.sameValue(moved[3],'c');
            assert.sameValue(1 in moved,false);assert.sameValue(2 in moved,false);
            var locked={0:'a',1:'b',length:2};Object.defineProperty(locked,'length',{writable:false});
            assert.throws(TypeError,function(){p.pop.call(locked);});
            assert.sameValue(1 in locked,false);assert.sameValue(locked.length,2);
            var partial={0:'a',1:'b',length:2};Object.defineProperty(partial,'1',{writable:false});
            assert.throws(TypeError,function(){p.unshift.call(partial,'x');});
            assert.sameValue(partial[2],'b');assert.sameValue(partial[0],'a');assert.sameValue(partial.length,2);
            var writes={length:1},seen='';Object.defineProperty(writes,'1',{set:function(v){seen+=v;assert.sameValue(this,writes);assert.sameValue(this.length,1);}});
            assert.sameValue(p.push.call(writes,'b','c'),3);assert.sameValue(seen,'b');assert.sameValue(writes[2],'c');
        "#,
        );
    }

    #[test]
    fn generic_array_searches_distinguish_holes_nan_and_live_from_index() {
        check(
            r#"
            var p=Array.prototype,a=[,undefined,NaN,-0];
            assert.sameValue(a.includes(undefined),true);assert.sameValue(a.indexOf(undefined),1);
            assert.sameValue(a.includes(NaN),true);assert.sameValue(a.indexOf(NaN),-1);
            assert.sameValue(a.includes(0),true);assert.sameValue(a.indexOf(0),3);
            assert.sameValue(a.indexOf(undefined,-3),1);assert.sameValue(a.indexOf(undefined,2),-1);
            assert.sameValue(a.includes(NaN,Infinity),false);
            var calls=0,from={valueOf:function(){calls++;throw 9;}};
            assert.sameValue(p.includes.call({length:0},1,from),false);
            assert.sameValue(p.indexOf.call({length:0},1,from),-1);assert.sameValue(calls,0);
            var o={length:2,0:'old'},start={valueOf:function(){o[1]='new';o.length=0;return -1;}};
            assert.sameValue(p.indexOf.call(o,'new',start),1);
            var inherited=Object.create({1:'yes'});inherited.length=2;
            assert.sameValue(p.includes.call(inherited,'yes'),true);assert.sameValue(p.indexOf.call(inherited,'yes'),1);
            assert.sameValue(p.indexOf.call('\uD800x','\uD800'),0);
        "#,
        );
    }

    #[test]
    fn generic_array_join_uses_string_hint_live_reads_and_utf16() {
        check(
            r#"
            var p=Array.prototype,log='',o={length:2};
            Object.defineProperty(o,'0',{get:function(){log+='0';return {toString:function(){log+='s';o[1]='tail';return '\uD800';},valueOf:function(){throw 7;}};}});
            var separator={toString:function(){log+='|';return '\uDC00';}};
            assert.sameValue(p.join.call(o,separator),'\uD800\uDC00tail');assert.sameValue(log,'|0s');
            var a=[1];a.toString=function(){return 'custom';};assert.sameValue([a].join(),'custom');
            var b=[1];b.join=function(){return 'own';};assert.sameValue([b].join(),'own');
            assert.sameValue([undefined,null,,3].join(':'),':::3');
            assert.throws(TypeError,function(){[].join(Symbol('separator'));});
            assert.throws(TypeError,function(){[Symbol('value')].join();});
            var marker={};try{p.join.call({length:0},{toString:function(){throw marker;}});throw 1;}catch(e){assert.sameValue(e,marker);}
        "#,
        );
    }

    #[test]
    fn generic_array_callbacks_validate_after_length_and_keep_this_and_three_args() {
        check(
            r#"
            var p=Array.prototype,methods=['map','filter','forEach'];
            for(var i=0;i<methods.length;i++){
                var method=p[methods[i]],marker={},got=false,o={};
                Object.defineProperty(o,'length',{get:function(){got=true;throw marker;}});
                try{method.call(o,null);throw 1;}catch(e){assert.sameValue(e,marker);}
                assert.sameValue(got,true);assert.throws(TypeError,function(){method.call({length:0},null);});
                var receiver={},seen=0,input={0:'v',length:1};
                method.call(input,function(value,index,obj){'use strict';assert.sameValue(this,receiver);assert.sameValue(arguments.length,3);assert.sameValue(value,'v');assert.sameValue(index,0);assert.sameValue(obj,input);seen++;return true;},receiver);
                assert.sameValue(seen,1);
                method.call(input,function(){'use strict';assert.sameValue(this,undefined);});
            }
            var sparse=[1,,3],visited='';
            var mapped=sparse.map(function(value,index,obj){visited+=index;assert.sameValue(obj,sparse);if(index===0){obj[1]=2;delete obj[2];obj.push(4);}return value*2;});
            assert.sameValue(visited,'01');assert.sameValue(mapped.length,3);
            assert.sameValue(mapped[0],2);assert.sameValue(mapped[1],4);assert.sameValue(2 in mapped,false);
            var original=[1,2,3],filtered=original.filter(function(value,index,obj){obj[index]=99;return index!==1;});
            assert.sameValue(filtered.join(','),'1,3');
            var empty=[,,],calls=0;assert.sameValue(empty.forEach(function(){calls++;}),undefined);assert.sameValue(calls,0);
        "#,
        );
    }

    #[test]
    fn generic_array_results_define_own_indices_and_preserve_slice_order() {
        check(
            r#"
            var p=Array.prototype,setterCalls=0;
            Object.defineProperty(p,'0',{configurable:true,set:function(){setterCalls++;}});
            var input=Object.create(null);input.length=2;input[0]='a';input[1]='b';
            var mapped=p.map.call(input,function(x){return x;});
            var filtered=p.filter.call(input,function(){return true;});
            var sliced=p.slice.call(input);
            delete p[0];
            assert.sameValue(setterCalls,0);assert.sameValue(mapped[0],'a');assert.sameValue(filtered[0],'a');assert.sameValue(sliced[0],'a');
            var o={0:'a',2:'c',length:3},log='';
            var start={valueOf:function(){log+='s';return -3;}},end={valueOf:function(){log+='e';o[1]='b';return 3;}};
            var copy=p.slice.call(o,start,end);assert.sameValue(log,'se');assert.sameValue(copy.join(','),'a,b,c');
            var holes=p.slice.call({length:3,1:undefined});assert.sameValue(holes.length,3);
            assert.sameValue(0 in holes,false);assert.sameValue(1 in holes,true);assert.sameValue(2 in holes,false);
            assert.sameValue(p.slice.call('\uD800x',0,1)[0],'\uD800');
        "#,
        );
    }

    #[test]
    fn generic_array_species_defaults_are_live_and_custom_construction_is_explicit() {
        check(
            r#"
            var a=[1],log='',c={};
            Object.defineProperty(a,'constructor',{configurable:true,get:function(){log+='c';return c;}});
            Object.defineProperty(c,Symbol.species,{get:function(){log+='s';return null;}});
            assert.sameValue(a.map(function(x){log+='m';return x+1;})[0],2);assert.sameValue(log,'csm');
            log='';a.forEach(function(){});assert.sameValue(log,'');
            var marker={};Object.defineProperty(a,'constructor',{get:function(){throw marker;},configurable:true});
            try{a.slice();throw 1;}catch(e){assert.sameValue(e,marker);}
            var bad=[1];bad.constructor=7;assert.throws(TypeError,function(){bad.slice();});
            var invalid=[1];invalid.constructor={};invalid.constructor[Symbol.species]=3;
            assert.throws(TypeError,function(){invalid.map(function(x){return x;});});
            var receiver=[1],queried=false;
            Object.defineProperty(receiver,'constructor',{get:function(){queried=true;return Array;}});
            assert.throws(TypeError,function(){receiver.map(null);});assert.sameValue(queried,false);
        "#,
        );
    }

    #[test]
    fn generic_array_join_cycles_preserve_overrides_and_cleanup_after_throw() {
        check(
            r#"
            var a=[1];a.push(a,2);assert.sameValue(a.join(','),'1,,2');
            var b=[],c=[];b.push(c);c.push(b);assert.sameValue(b.join(),'');
            var custom=[];custom.toString=function(){return 'custom';};
            custom.push(custom);assert.sameValue(custom.join(),'custom');
            var marker={},o=[1];Object.defineProperty(o,'0',{configurable:true,get:function(){throw marker;}});
            try{o.join();throw 1;}catch(e){assert.sameValue(e,marker);}
            delete o[0];o[0]='ok';assert.sameValue(o.join(),'ok');
        "#,
        );
        let (mut runtime, mut doc) = harness();
        runtime
            .execute("var a=[];a.push(a);a.join();", &mut doc)
            .unwrap();
        assert!(runtime.active_array_joins.is_empty());
    }

    #[test]
    fn generic_array_custom_species_is_unsupported_before_construction() {
        for name in ["slice", "map", "filter"] {
            let (mut runtime, mut doc) = harness();
            let source = format!(
                "var called=0,a=[1];a.constructor={{}};a.constructor[Symbol.species]=function(){{called++;}};a.{name}(function(x){{return x;}});"
            );
            let error = runtime.execute(&source, &mut doc).unwrap_err();
            assert!(error.is_unsupported(), "{name}: {error:?}");
            assert_eq!(runtime.lookup(1, "called").unwrap().1, Value::Number(0.0));
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert!(runtime.active_array_joins.is_empty());
        }
        check(
            r#"
            var marker={},source={0:1,length:70000};
            try{Array.prototype.map.call(source,function(){throw marker;});throw 1;}catch(e){assert.sameValue(e,marker);}
            assert.throws(RangeError,function(){Array.prototype.map.call({length:4294967296},function(){});});
            var copy=Array.prototype.slice.call({length:Infinity},9007199254740989);
            assert.sameValue(copy.length,2);assert.sameValue(0 in copy,false);assert.sameValue(1 in copy,false);
        "#,
        );
    }

    #[test]
    fn generic_array_methods_share_sparse_work_and_callback_recursion_limits() {
        for source in [
            "Array.prototype.indexOf.call({length:Infinity},'missing');",
            "Array.prototype.includes.call({length:Infinity},'missing');",
            "Array.prototype.unshift.call({length:9007199254740990},1);",
            "Array.prototype.forEach.call({length:Infinity},function(){});",
            "var o={};Object.defineProperty(o,'length',{get:function(){return Array.prototype.join.call(o);}});Array.prototype.join.call(o);",
            "var a=[1];a.map(function again(){return a.map(again);});",
        ] {
            let (mut runtime, mut doc) = harness();
            let source = format!("var caught=false;try{{{source}}}catch(e){{caught=true;}}");
            let error = runtime.execute(&source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit(), "{source}: {error:?}");
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert!(runtime.frames.is_empty());
            assert!(runtime.active_array_joins.is_empty());
        }
    }

    #[test]
    fn generic_array_full_range_streams_and_overflow_precedes_writes() {
        check(
            r#"
            var p=Array.prototype,o={length:9007199254740990};
            assert.sameValue(p.push.call(o,'x'),9007199254740991);
            assert.sameValue(o[9007199254740990],'x');
            assert.throws(TypeError,function(){p.push.call(o,'y');});
            assert.sameValue(o[9007199254740991],undefined);
            assert.sameValue(p.pop.call(o),'x');assert.sameValue(o.length,9007199254740990);
            var a={length:9007199254740991},marker={};
            Object.defineProperty(a,'9007199254740990',{get:function(){throw marker;}});
            try{p.pop.call(a);throw 1;}catch(e){assert.sameValue(e,marker);}
            assert.sameValue(p.includes.call(a,1,Infinity),false);
        "#,
        );
    }

    #[test]
    fn generic_array_callback_preflight_keeps_completed_getter_effects() {
        fn prepared() -> (Runtime, Document, Value, Value) {
            let (mut runtime, mut document) = harness();
            let object = runtime.execute("var reads=0,calls=0,o={length:1};Object.defineProperty(o,'0',{get:function(){reads++;return 4;}});function cb(){calls++;return 1;}o", &mut document).unwrap();
            let callback = runtime.lookup(1, "cb").unwrap().1;
            (runtime, document, object, callback)
        }
        // Measure the exact same immutable setup and length/Has/Get prefix.
        // A fresh runtime then has one byte less than its callback vector needs.
        let (mut measure, mut document, object, _) = prepared();
        let before = measure.allocated;
        measure.tick().unwrap();
        measure.coerce_object(object.clone()).unwrap();
        measure.charge(64).unwrap();
        let value = measure
            .reduce_get(&object, &JsString::from("length"), &mut document)
            .unwrap();
        measure.number_value(value, &mut document).unwrap();
        measure.tick().unwrap();
        let key = measure.reduce_index_key(0).unwrap();
        assert!(measure.reduce_property(&object, &key).unwrap().is_some());
        measure.reduce_get(&object, &key, &mut document).unwrap();
        let prefix = measure.allocated - before;
        let vector = 32 + 3 * std::mem::size_of::<Value>();

        for method in ["forEach", "every", "some"] {
            let (mut runtime, mut document, object, callback) = prepared();
            runtime.allocated = MAX_HEAP - prefix - vector + 1;
            let error = runtime
                .array_method(object, method, vec![callback], &mut document)
                .unwrap_err();
            assert!(error.is_resource_limit(), "{method}: {error:?}");
            assert_eq!(runtime.allocated, MAX_HEAP + 1);
            assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(1.0));
            assert_eq!(runtime.lookup(1, "calls").unwrap().1, Value::Number(0.0));
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert!(runtime.frames.is_empty());
            assert!(runtime.active_array_joins.is_empty());
        }
    }

    #[test]
    fn generic_array_private_join_reservation_and_callback_failure_are_bounded() {
        let (mut runtime, _) = harness();
        let text = JsString::from("abcd");
        let mut output = Vec::new();
        runtime.allocated = MAX_HEAP - 31;
        let error = runtime.array_join_append(&mut output, &text).unwrap_err();
        assert!(error.is_resource_limit());
        assert!(output.is_empty());
        assert_eq!(output.capacity(), 0);

        let (mut runtime, mut doc) = harness();
        let object = runtime.execute("var reads=0,o={length:1};Object.defineProperty(o,'0',{get:function(){reads++;return 4;}});o", &mut doc).unwrap();
        let callback = runtime
            .execute("function cb(){throw 'callback ran';}cb", &mut doc)
            .unwrap();
        runtime.allocated = MAX_HEAP;
        let error = runtime
            .array_method(object, "forEach", vec![callback], &mut doc)
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(runtime.lookup(1, "reads").unwrap().1, Value::Number(0.0));
    }

    #[test]
    fn frozen_array_predicate_cases_preserve_all_original_sources() {
        for fixture in include_str!("../../tests/conformance/array-predicates.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (name, source) = fixture.split_once('\n').unwrap();
            for strict in [false, true] {
                let (mut runtime, mut document) = harness();
                runtime
                    .execute(
                        include_str!("../../tests/upstream/test262/harness/propertyHelper.js"),
                        &mut document,
                    )
                    .unwrap();
                let result = if strict {
                    runtime.execute_strict(source, &mut document)
                } else {
                    runtime.execute(source, &mut document)
                };
                assert!(result.is_ok(), "{name}, strict={strict}: {result:?}");
                assert_eq!(runtime.calls, 0);
                assert_eq!(runtime.stack_units, 0);
                assert_eq!(runtime.eval_depth, 0);
                assert!(runtime.frames.is_empty());
            }
        }
    }

    #[test]
    fn array_predicate_recursion_and_sparse_scans_share_terminal_limits() {
        for method in ["every", "some"] {
            for body in [
                format!(
                    "var o={{get length(){{return Array.prototype.{method}.call(o,function(){{}});}}}};Array.prototype.{method}.call(o,function(){{}});"
                ),
                format!(
                    "var o={{length:{{valueOf:function(){{return Array.prototype.{method}.call(o,function(){{}});}}}}}};Array.prototype.{method}.call(o,function(){{}});"
                ),
                format!(
                    "var o={{length:1,get 0(){{return Array.prototype.{method}.call(o,function(){{}});}}}};Array.prototype.{method}.call(o,function(){{}});"
                ),
                format!("function cb(){{return [1].{method}(cb);}}[1].{method}(cb);"),
                format!("[1].{method}(function(){{while(true){{}}}});"),
                format!(
                    "Array.prototype.{method}.call({{length:Infinity}},function(){{throw 'unreachable';}});"
                ),
            ] {
                let (mut runtime, mut document) = harness();
                let source = format!(
                    "var caught=false,finalized=false;try{{{body}}}catch(e){{caught=true;}}finally{{finalized=true;}}"
                );
                assert!(
                    runtime
                        .execute(&source, &mut document)
                        .unwrap_err()
                        .is_resource_limit(),
                    "{body}"
                );
                assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
                assert_eq!(
                    runtime.lookup(1, "finalized").unwrap().1,
                    Value::Bool(false)
                );
                assert_eq!(runtime.calls, 0);
                assert_eq!(runtime.stack_units, 0);
                assert_eq!(runtime.eval_depth, 0);
                assert!(runtime.frames.is_empty());
            }
        }
    }

    #[test]
    fn array_predicate_early_return_does_not_allocate_from_logical_length() {
        for method in ["every", "some"] {
            let mut allocations = Vec::new();
            for length in [1.0, 4_294_967_295.0, 9_007_199_254_740_991.0] {
                let (mut runtime, mut document) = harness();
                let object = runtime
                    .object_ordered([
                        ("length".into(), Value::Number(length)),
                        ("0".into(), Value::Number(1.0)),
                    ])
                    .unwrap();
                let callback = runtime
                    .execute(
                        if method == "some" {
                            "function cb(){return true;}cb"
                        } else {
                            "function cb(){return false;}cb"
                        },
                        &mut document,
                    )
                    .unwrap();
                let before = runtime.allocated;
                let arrays = runtime.arrays.len();
                assert_eq!(
                    runtime
                        .array_method(object, method, vec![callback], &mut document)
                        .unwrap(),
                    Value::Bool(method == "some")
                );
                allocations.push(runtime.allocated - before);
                assert_eq!(runtime.arrays.len(), arrays);
                assert_eq!(runtime.calls, 0);
                assert_eq!(runtime.stack_units, 0);
                assert!(runtime.frames.is_empty());
            }
            assert!(allocations.iter().all(|amount| *amount == allocations[0]));
        }
    }
}
