//! String builtins using ECMAScript conversion and lossless UTF-16 storage.
use super::*;

impl Runtime {
    pub(super) fn string_rfind(
        &mut self,
        text: &[u16],
        needle: &[u16],
        start: usize,
    ) -> Result<Option<usize>> {
        if needle.is_empty() {
            return Ok(Some(start.min(text.len())));
        }
        let Some(last) = text.len().checked_sub(needle.len()) else {
            return Ok(None);
        };
        for index in (0..=start.min(last)).rev() {
            // Charge each comparison's worst-case code-unit work before it.
            self.work(1 + needle.len() / 8)?;
            if text[index..index + needle.len()] == *needle {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }

    pub(super) fn string_match_search(
        &mut self,
        name: &str,
        receiver: Value,
        pattern: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "String method receiver is null or undefined",
            ));
        }
        let key = self.well_known_key(name);
        // GetMethod precedes receiver conversion and is only performed for an
        // object argument. The hook receives the original receiver unchanged.
        if js_object(&pattern) {
            let method = self.get_property_key(pattern.clone(), &key, doc)?;
            if !matches!(method, Value::Null | Value::Undefined) {
                if !json_callable(&method) {
                    return Err(ScriptError::type_error(
                        "String symbol hook must be callable",
                    ));
                }
                self.charge(std::mem::size_of::<Value>())?;
                return self.call(method, vec![receiver], pattern, doc);
            }
        }
        let text = self.string_hint(receiver, doc)?;
        let regexp = self.regexp_create(pattern, Value::Undefined, doc)?;
        // Invoke must observe changes to the intrinsic prototype's method.
        let method = self.get_property_key(regexp.clone(), &key, doc)?;
        self.charge(std::mem::size_of::<Value>())?;
        self.call(method, vec![Value::String(text)], regexp, doc)
    }

    pub(super) fn string_split_hook(
        &mut self,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "String.split receiver is null or undefined",
            ));
        }
        let separator = arguments.first().cloned().unwrap_or(Value::Undefined);
        // Primitive separators are not boxed for the protocol lookup.
        if !js_object(&separator) {
            return Ok(None);
        }
        let method =
            self.get_property_key(separator.clone(), &self.well_known_key("split"), doc)?;
        if matches!(method, Value::Null | Value::Undefined) {
            return Ok(None);
        }
        if !json_callable(&method) {
            return Err(ScriptError::type_error("Symbol.split must be callable"));
        }
        self.charge(2 * std::mem::size_of::<Value>())?;
        self.call(
            method,
            vec![
                receiver,
                arguments.get(1).cloned().unwrap_or(Value::Undefined),
            ],
            separator,
            doc,
        )
        .map(Some)
    }

    pub(super) fn is_regexp(&mut self, value: Value, doc: &mut Document) -> Result<bool> {
        if !js_object(&value) {
            return Ok(false);
        }
        let matcher = self.get_property_key(value.clone(), &self.well_known_key("match"), doc)?;
        Ok(if matcher == Value::Undefined {
            self.regexp_slot(&value).is_some()
        } else {
            matcher.truthy()
        })
    }

    pub(super) fn string_concat(
        &mut self,
        receiver: Value,
        arguments: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        if matches!(receiver, Value::Null | Value::Undefined) {
            return Err(ScriptError::type_error(
                "String.concat receiver is null or undefined",
            ));
        }
        let receiver = self.string_hint(receiver, doc)?;
        let mut length = receiver.len();
        if length > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        if arguments.is_empty() {
            return Ok(Value::String(receiver));
        }
        let count = arguments.len().saturating_add(1);
        self.work(count)?;
        self.charge(64 + count.saturating_mul(std::mem::size_of::<JsString>()))?;
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("string concat fragments allocation failed"))?;
        parts.push(receiver);
        for argument in arguments {
            // Hooks run left-to-right and can change a later argument's hooks.
            // Retain converted strings so no hook is repeated during copying.
            let text = self.string_hint(argument.clone(), doc)?;
            length = length.saturating_add(text.len());
            if length > MAX_STRING {
                return Err(ScriptError::resource("script string limit exceeded"));
            }
            parts.push(text);
        }
        // Both the temporary UTF-16 buffer and its Rc allocation coexist during
        // conversion. Reserve their storage and copying work before allocation.
        self.work(1 + count + length / 4)?;
        self.charge(64 + length.saturating_mul(4))?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("string concat output allocation failed"))?;
        for part in parts {
            units.extend_from_slice(part.units());
        }
        Ok(Value::String(units.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness() -> (Runtime, Document) {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<!doctype html><title>String concat</title>");
        for source in [
            include_str!("../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../tests/upstream/test262/harness/propertyHelper.js"),
        ] {
            runtime.execute(source, &mut doc).unwrap();
        }
        (runtime, doc)
    }

    fn check(source: &str) {
        for strict in [false, true] {
            let (mut runtime, mut doc) = harness();
            let result = if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            };
            assert!(result.is_ok(), "strict={strict}: {result:?}; {source}");
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(runtime.eval_depth, 0);
            assert!(runtime.frames.is_empty());
        }
    }

    #[test]
    fn last_index_of_frozen_cases_cover_conversion_order_and_utf16() {
        for fixture in include_str!("../../tests/conformance/string-last-index-of.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (_, source) = fixture.split_once('\n').unwrap();
            check(source);
        }
    }

    #[test]
    fn reverse_search_matches_forward_enumeration_for_code_units() {
        let mut runtime = Runtime::new();
        let allocated = runtime.allocated;
        let alphabet = [0, b'a' as u16, 0xd800, 0xdfff];
        let mut seed = 0xe2152026u32;
        let mut next = || {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 16) as usize
        };
        for _ in 0..4096 {
            let text: Vec<_> = (0..next() % 24).map(|_| alphabet[next() % 4]).collect();
            let needle: Vec<_> = (0..next() % 9).map(|_| alphabet[next() % 4]).collect();
            for start in [0, next() % 26, usize::MAX] {
                let expected = if needle.is_empty() {
                    Some(start.min(text.len()))
                } else {
                    text.windows(needle.len())
                        .enumerate()
                        .take_while(|(index, _)| *index <= start)
                        .filter_map(|(index, window)| (window == needle).then_some(index))
                        .last()
                };
                runtime.steps = MAX_STEPS;
                assert_eq!(
                    runtime.string_rfind(&text, &needle, start).unwrap(),
                    expected,
                    "{text:?}, {needle:?}, {start}"
                );
            }
        }
        assert_eq!(runtime.allocated, allocated);
    }

    #[test]
    fn reverse_search_charges_comparisons_before_matching() {
        let mut runtime = Runtime::new();
        let text = vec![b'a' as u16; 4096];
        let mut needle = vec![b'a' as u16; 2048];
        runtime.steps = 1;
        assert!(
            runtime
                .string_rfind(&text, &needle, usize::MAX)
                .unwrap_err()
                .is_resource_limit()
        );
        needle[2047] = b'b' as u16;
        runtime.steps = MAX_STEPS;
        assert!(
            runtime
                .string_rfind(&text, &needle, usize::MAX)
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn last_index_of_recursive_conversions_share_limits_and_unwind() {
        for source in [
            "var r={toString:function(){return String.prototype.lastIndexOf.call(r,'a');}};String.prototype.lastIndexOf.call(r,'a');",
            "var s={toString:function(){return 'a'.lastIndexOf(s);}};'a'.lastIndexOf(s);",
            "var p={valueOf:function(){return 'a'.lastIndexOf('a',p);}};'a'.lastIndexOf('a',p);",
        ] {
            let (mut runtime, mut doc) = harness();
            let wrapped = format!("var caught=false;try{{{source}}}catch(e){{caught=true;}}");
            assert!(
                runtime
                    .execute(&wrapped, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(runtime.eval_depth, 0);
            assert_eq!(runtime.json_depth, 0);
            assert!(runtime.frames.is_empty());
        }
    }

    #[test]
    fn string_conversion_frozen_cases_cover_receivers_arguments_and_order() {
        for fixture in include_str!("../../tests/conformance/string-conversion.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (_, source) = fixture.split_once('\n').unwrap();
            check(source);
        }
    }

    #[test]
    fn string_split_custom_hooks_keep_raw_arguments_and_abrupt_order() {
        for fixture in include_str!("../../tests/conformance/string-split-hooks.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (_, source) = fixture.split_once('\n').unwrap();
            check(source);
        }
    }

    #[test]
    fn string_conversion_callbacks_share_resource_guards_and_unwind() {
        for source in [
            "var a=function(){};a.toString=function(){return String.prototype.slice.call(a);};String.prototype.slice.call(a);",
            "var a=[];a.toString=function(){return 'a'.includes(a);};'a'.includes(a);",
            "var a={};Object.defineProperty(a,Symbol.match,{get:function(){return 'a'.startsWith(a);}});'a'.startsWith(a);",
            "var a={};a[Symbol.split]=function(){return ''.split(a);};''.split(a);",
            "var a={};Object.defineProperty(a,Symbol.split,{get:function(){return ''.split(a);}});''.split(a);",
        ] {
            let (mut runtime, mut doc) = harness();
            let error = runtime.execute(source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit(), "{source}: {error}");
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(runtime.json_depth, 0);
            assert_eq!(runtime.eval_depth, 0);
            assert!(runtime.frames.is_empty());
        }
        let (mut runtime, mut doc) = harness();
        let function = runtime.execute("var calls=0,sep={};sep[Symbol.split]=function(){calls++;return 1;};String.prototype.split", &mut doc).unwrap();
        let separator = runtime.execute("sep", &mut doc).unwrap();
        runtime.allocated = MAX_HEAP;
        let error = runtime
            .call(
                function,
                vec![separator],
                Value::String("".into()),
                &mut doc,
            )
            .unwrap_err();
        assert!(error.is_resource_limit());
        assert_eq!(
            runtime.environments[0].bindings["calls"].value,
            Value::Number(0.0)
        );
        assert_eq!(runtime.stack_units, 0);
    }

    #[test]
    fn frozen_concat_probes_retain_every_source_and_language_mode() {
        for line in include_str!("../../tests/fixtures/string-concat.tsv").lines() {
            let (name, source) = line.split_once('\t').unwrap();
            if name == "metadata-identity" {
                // This frozen probe omitted restore:true. The upstream helper
                // deletes the configurable method before its next assertion.
                for strict in [false, true] {
                    let (mut runtime, mut doc) = harness();
                    let error = if strict {
                        runtime.execute_strict(source, &mut doc)
                    } else {
                        runtime.execute(source, &mut doc)
                    }
                    .unwrap_err();
                    assert_eq!(error.name(), "TypeError");
                    assert!(
                        runtime
                            .own_property(
                                &Value::Object(runtime.prototypes["String"]),
                                &"concat".into()
                            )
                            .is_none()
                    );
                }
            } else {
                check(source);
            }
        }
    }

    #[test]
    fn concat_argument_evaluation_precedes_ordered_live_conversion() {
        check(
            r#"
            var log='',first={toString:function(){log+='b';return 'B';}};
            var receiver={toString:function(){log+='r';first={toString:function(){return 'new';}};return 'R';}};
            function argument(){log+='a';return first;}
            assert.sameValue(String.prototype.concat.call(receiver,argument()),'RB');
            assert.sameValue(log,'arb');
            var later={toString:function(){return 'old';}},n=0;
            var initial={toString:function(){n++;later.toString=function(){return 'new';};return 'start';}};
            assert.sameValue(String.prototype.concat.call(initial,later),'startnew');assert.sameValue(n,1);
            var shared={toString:function(){n++;return n;}};
            n=0;assert.sameValue(String.prototype.concat.call(shared,shared,shared),'123');
            var ex={};ex[Symbol.toPrimitive]=function(h){assert.sameValue(h,'string');return false;};
            assert.sameValue(''.concat(ex),'false');
        "#,
        );
    }

    #[test]
    fn concat_abrupt_conversion_keeps_earlier_effects_and_skips_later_hooks() {
        check(
            r#"
            var seen,reason={},log='',a={},b={},c={};
            a[Symbol.toPrimitive]=function(h){log+='a';return 'A';};
            b[Symbol.toPrimitive]=function(h){log+='b';throw reason;};
            c[Symbol.toPrimitive]=function(h){log+='c';return 'C';};
            try{'x'.concat(a,b,c);}catch(e){seen=e;}
            assert.sameValue(seen,reason);assert.sameValue(log,'ab');
            log='';b[Symbol.toPrimitive]=function(){log+='b';return Symbol();};
            assert.throws(TypeError,function(){'x'.concat(a,b,c);});assert.sameValue(log,'ab');
            log='';b[Symbol.toPrimitive]=function(){log+='b';return {};};
            assert.throws(TypeError,function(){'x'.concat(a,b,c);});assert.sameValue(log,'ab');
            var wrong={toString:1,valueOf:2};
            assert.throws(TypeError,function(){'x'.concat(wrong);});
            assert.sameValue('after'.concat('error'),'aftererror');
        "#,
        );
    }

    #[test]
    fn concat_uses_canonical_intrinsic_identity_and_live_prototype_properties() {
        check(
            r#"
            verifyProperty(String.prototype,'concat',{writable:true,enumerable:false,configurable:true},{restore:true});
            verifyProperty(String.prototype.concat,'name',{value:'concat',writable:false,enumerable:false,configurable:true},{restore:true});
            verifyProperty(String.prototype.concat,'length',{value:1,writable:false,enumerable:false,configurable:true},{restore:true});
            assert.sameValue(''.concat,String.prototype.concat);
            assert.sameValue(String.prototype.concat.hasOwnProperty('prototype'),false);
        "#,
        );
        check(
            r#"
            var saved=String.prototype.concat,descriptor=Object.getOwnPropertyDescriptor(String.prototype,'concat');
            assert.sameValue(''.concat,saved);assert.sameValue(new String('x').concat,saved);
            assert.sameValue(Object.getPrototypeOf(saved),Function.prototype);
            assert.sameValue(delete String.prototype.concat,true);
            assert.sameValue(''.concat,undefined);assert.sameValue(new String('x').concat,undefined);
            assert.sameValue(saved.call('a','b'),'ab');
            String.prototype.concat=function(){return 'replacement';};
            assert.sameValue(''.concat(),'replacement');
            Object.defineProperty(String.prototype,'concat',descriptor);
            assert.sameValue('a'.concat('b'),'ab');
            assert.throws(TypeError,function(){new saved();});
            var target=document.createElement('div');target.textContent='\ud800'.concat('x');
            assert.sameValue(target.textContent,'\ufffdx');
        "#,
        );
    }

    #[test]
    fn concat_bounds_fragment_and_output_storage_before_allocation() {
        let (mut runtime, mut doc) = harness();
        runtime
            .execute(
                "var visits=0;var argument={toString:function(){visits++;return 'x';}};",
                &mut doc,
            )
            .unwrap();
        let argument = runtime.lookup(1, "argument").unwrap().1;
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .string_concat(Value::String("r".into()), &[argument], &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "visits").unwrap().1, Value::Number(0.0));
        assert_eq!(runtime.calls, 0);
        assert!(runtime.frames.is_empty());

        // The fragments fit, but the final buffer and Rc storage do not.
        let mut runtime = Runtime::new();
        runtime.allocated = MAX_HEAP - 256;
        let text = JsString::from(vec![0xd800; 1024]);
        assert!(
            runtime
                .string_concat(
                    Value::String(text.clone()),
                    &[Value::String(text)],
                    &mut doc
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.allocated > MAX_HEAP);

        let mut runtime = Runtime::new();
        runtime.steps = 0;
        assert!(
            runtime
                .string_concat(Value::String("r".into()), &[Value::Null], &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
    }

    #[test]
    fn concat_preserves_maximum_code_units_and_rejects_overflow_before_next_hook() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("");
        let receiver = JsString::from(vec![0xd800; MAX_STRING - 1]);
        let result = runtime
            .string_concat(
                Value::String(receiver.clone()),
                &[Value::String(vec![0xdfff].into())],
                &mut doc,
            )
            .unwrap();
        let Value::String(text) = result else {
            panic!("primitive string expected")
        };
        assert_eq!(text.len(), MAX_STRING);
        assert_eq!(text.units()[0], 0xd800);
        assert_eq!(text.units()[MAX_STRING - 1], 0xdfff);
        assert_eq!(receiver.len(), MAX_STRING - 1);

        let (mut runtime, mut doc) = harness();
        runtime
            .execute(
                "var visits=0;var argument={toString:function(){visits++;return 'tail';}};",
                &mut doc,
            )
            .unwrap();
        let argument = runtime.lookup(1, "argument").unwrap().1;
        assert!(
            runtime
                .string_concat(
                    Value::String(receiver),
                    &[Value::String("ab".into()), argument],
                    &mut doc
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.lookup(1, "visits").unwrap().1, Value::Number(0.0));

        // Empty concatenation needs no duplicate code-unit buffer.
        let mut runtime = Runtime::new();
        runtime.allocated = MAX_HEAP;
        let value = Value::String(vec![0xd800].into());
        assert_eq!(
            runtime.string_concat(value.clone(), &[], &mut doc).unwrap(),
            value
        );
    }

    #[test]
    fn concat_callbacks_share_limits_and_unwind_without_author_recovery() {
        for source in [
            "var o={toString:function(){return ''.concat(o);}};var caught=false;try{''.concat(o);}catch(e){caught=true;}",
            "var o={toString:function(){while(true){}}};var caught=false;try{''.concat(o);}catch(e){caught=true;}",
            "var o={toString:function(){var x='x';while(true){x=x.concat(x);}}};var caught=false;try{''.concat(o);}catch(e){caught=true;}",
        ] {
            let (mut runtime, mut doc) = harness();
            assert!(
                runtime
                    .execute(source, &mut doc)
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert_eq!(runtime.lookup(1, "caught").unwrap().1, Value::Bool(false));
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(runtime.eval_depth, 0);
            assert!(runtime.frames.is_empty());
        }
    }
}
