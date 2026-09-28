//! String builtins using ECMAScript conversion and lossless UTF-16 storage.
use super::*;

impl Runtime {
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
