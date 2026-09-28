//! Reflection over the supported Window object-environment binding records.
use super::*;

impl Runtime {
    pub(super) fn global_creation_order(&mut self, name: &str) -> Result<u64> {
        if let Some(binding) = self.environments[0].bindings.get(name)
            && binding.global_property
        {
            return Ok(binding.global_order);
        }
        let order = self.next_global_order;
        self.next_global_order = order
            .checked_add(1)
            .ok_or_else(|| ScriptError::resource("global property creation counter exhausted"))?;
        Ok(order)
    }

    pub(super) fn window_reflected_property(&mut self, key: &JsString) -> Result<Option<Property>> {
        // Global binding records are authoritative. Reflection does not invoke
        // a getter, inspect the separate lexical environment or consult prototypes.
        let comparisons = 1 + self.environments[0]
            .bindings
            .len()
            .checked_ilog2()
            .unwrap_or(0) as usize;
        self.work(1 + key.len().saturating_mul(comparisons))?;
        self.charge(24 + key.len().saturating_mul(6))?;
        match key.to_utf8() {
            Ok(key) => {
                if event_handler_name(&key) {
                    return Err(ScriptError::unsupported(
                        "Window event-handler descriptor reflection is not implemented",
                    ));
                }
                Ok(self.environments[0]
                    .bindings
                    .get(&key)
                    .filter(|binding| binding.global_property)
                    .map(Binding::property))
            }
            // Binding names are scalar UTF-8. Never alias a lone surrogate to
            // U+FFFD; arbitrary UTF-16 Window property storage is separate work.
            Err(_) => Ok(None),
        }
    }

    pub(super) fn window_own_keys(&mut self) -> Result<Vec<JsString>> {
        self.work(self.environments[0].bindings.len().saturating_add(1))?;
        let (count, bytes) = self.environments[0]
            .bindings
            .iter()
            .filter(|(_, binding)| binding.global_property)
            .fold((0usize, 0usize), |(n, bytes), (name, _)| {
                (n + 1, bytes.saturating_add(name.len()))
            });
        // The snapshot and output vectors coexist. Include UTF-16 key storage,
        // reference-counted headers, scans and comparison work before allocation.
        type Entry = (Option<u32>, u64, JsString);
        let per_key = std::mem::size_of::<Entry>() + std::mem::size_of::<JsString>() + 64;
        self.charge(64 + bytes.saturating_mul(6) + count.saturating_mul(per_key))?;
        self.work(
            1 + bytes.saturating_mul(2)
                + count.saturating_mul(2 + count.checked_ilog2().unwrap_or(0) as usize),
        )?;
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("global key snapshot allocation failed"))?;
        for (name, binding) in &self.environments[0].bindings {
            if binding.global_property {
                let key = JsString::from(name.as_str());
                entries.push((json_array_index(&key), binding.global_order, key));
            }
        }
        entries.sort_unstable_by_key(|(index, order, _)| match index {
            Some(index) => (false, u64::from(*index)),
            None => (true, *order),
        });
        let mut keys = Vec::new();
        keys.try_reserve_exact(count)
            .map_err(|_| ScriptError::resource("global key result allocation failed"))?;
        keys.extend(entries.into_iter().map(|(_, _, key)| key));
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn harness() -> (Runtime, Document) {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<!doctype html><title>Window reflection</title>");
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
    fn original_window_probes_retain_all_sources_and_explicit_definition_gaps() {
        for line in include_str!("../../tests/fixtures/window-reflection.tsv").lines() {
            let (name, source) = line.split_once('\t').unwrap();
            if matches!(name, "window-defined-data" | "window-defined-accessor") {
                for strict in [false, true] {
                    let (mut runtime, mut doc) = harness();
                    let error = if strict {
                        runtime.execute_strict(source, &mut doc)
                    } else {
                        runtime.execute(source, &mut doc)
                    }
                    .unwrap_err();
                    assert!(error.is_unsupported(), "{name}: {error}");
                }
            } else {
                check(source);
            }
        }
    }

    #[test]
    fn reflected_intrinsic_flags_and_readonly_shadowing_match_records() {
        check(
            r#"
            var names=['String','Number','Boolean','Symbol','parseInt','parseFloat',
              'encodeURI','decodeURI','encodeURIComponent','decodeURIComponent',
              'isNaN','isFinite','Object','Array','Function','RegExp','Error','TypeError',
              'SyntaxError','ReferenceError','RangeError','EvalError','URIError','eval',
              'Math','JSON','Reflect'];
            for(var i=0;i<names.length;i++){
              var d=Object.getOwnPropertyDescriptor(window,names[i]);
              assert.sameValue(d.value,window[names[i]]);
              assert.sameValue(d.writable,true);assert.sameValue(d.enumerable,false);
              assert.sameValue(d.configurable,true);
            }
            assert.sameValue(window.hasOwnProperty('this'),false);
            assert.sameValue(window.hasOwnProperty('toString'),false);
            assert.sameValue(window.propertyIsEnumerable('NaN'),false);
            var savedMath=Math,savedJSON=JSON;
            Math=7;JSON=8;assert.sameValue(window.Math,7);assert.sameValue(window.JSON,8);
            window.Math=savedMath;window.JSON=savedJSON;
            var keys=Object.keys(window);
            assert.sameValue(keys.indexOf('Math'),-1);assert.sameValue(keys.indexOf('JSON'),-1);
        "#,
        );
    }

    #[test]
    fn snapshots_preserve_index_creation_and_symbol_order_across_live_changes() {
        check(
            r#"
            window.zz=1;window.aa=2;window['4294967295']=3;
            window[4294967294]=4;window[10]=5;window[2]=6;window['01']=7;
            var a=Object.getOwnPropertyNames(window);
            assert.sameValue(a[0],'2');assert.sameValue(a[1],'10');assert.sameValue(a[2],'4294967294');
            assert(a.indexOf('zz')<a.indexOf('aa'));
            assert(a.indexOf('aa')<a.indexOf('4294967295'));
            assert(a.indexOf('4294967295')<a.indexOf('01'));
            window.zz=9;
            var b=Object.getOwnPropertyNames(window);assert.sameValue(b.join('|'),a.join('|'));
            delete window.zz;window.zz=10;
            b=Object.getOwnPropertyNames(window);assert(b.indexOf('01')<b.indexOf('zz'));
            var before=b.indexOf('globalThis');
            Object.defineProperty(window,'globalThis',{value:window});
            b=Object.getOwnPropertyNames(window);assert.sameValue(b.indexOf('globalThis'),before);
            delete window.globalThis;window.globalThis=window;
            b=Object.getOwnPropertyNames(window);assert.sameValue(b[b.length-1],'globalThis');
            var s1=Symbol('1'),s2=Symbol('2');window[s1]=1;window[s2]=2;
            delete window[s1];window[s1]=3;
            var all=Reflect.ownKeys(window);
            assert.sameValue(all[all.length-2],s2);assert.sameValue(all[all.length-1],s1);
            assert.sameValue(Object.getOwnPropertyNames(window).length,all.length-2);
        "#,
        );
    }

    #[test]
    fn reflection_converts_keys_before_receiver_checks_without_invoking_getters() {
        check(
            r#"
            var calls=0,gets=0,key={};
            Object.defineProperty(window,'globalThis',{get:function(){gets++;throw 8;},configurable:true});
            key[Symbol.toPrimitive]=function(h){assert.sameValue(h,'string');calls++;return 'globalThis';};
            assert.sameValue(Object.prototype.hasOwnProperty.call(window,key),true);
            assert.sameValue(Object.prototype.propertyIsEnumerable.call(window,key),false);
            assert.sameValue(calls,2);assert.sameValue(gets,0);
            Object.getOwnPropertyNames(window);Object.keys(window);Reflect.ownKeys(window);
            assert.sameValue(gets,0);
            var thrown={},seen;key[Symbol.toPrimitive]=function(){throw thrown;};
            try{Object.prototype.hasOwnProperty.call(null,key);}catch(e){seen=e;}
            assert.sameValue(seen,thrown);
            seen=undefined;
            try{Object.prototype.propertyIsEnumerable.call(undefined,key);}catch(e){seen=e;}
            assert.sameValue(seen,thrown);
            assert.sameValue(window.hasOwnProperty('NaN\ud800'),false);
            assert.sameValue(window.hasOwnProperty('NaN\ufffd'),false);
            assert.sameValue(window.hasOwnProperty('NaN\u0000'),false);
        "#,
        );
    }

    #[test]
    fn for_in_checks_live_deletion_and_preserves_hidden_own_shadowing() {
        check(
            r#"
            window.zzFirst=1;window.aaSecond=2;window.mmThird=3;
            Object.prototype.NaN=1;Object.prototype.zzInherited=1;
            var seen='',hidden=false,inherited=false;
            for(var key in window){
              if(key==='zzFirst'){seen+='1';delete window.aaSecond;window.later=1;}
              if(key==='aaSecond')seen+='2';
              if(key==='mmThird')seen+='3';
              if(key==='later')seen+='4';
              if(key==='NaN')hidden=true;
              if(key==='zzInherited')inherited=true;
            }
            assert.sameValue(seen,'13');assert.sameValue(hidden,false);assert.sameValue(inherited,true);
            assert(Object.keys(window).indexOf('later')>=0);
            assert.sameValue(Object.keys(window).indexOf('zzInherited'),-1);
        "#,
        );
    }

    #[test]
    fn snapshot_storage_work_and_creation_counter_fail_before_publication() {
        let mut runtime = Runtime::new();
        let count = runtime.environments[0].bindings.len();
        let order = runtime.next_global_order;
        runtime.allocated = MAX_HEAP;
        assert!(runtime.window_own_keys().unwrap_err().is_resource_limit());
        assert_eq!(runtime.environments[0].bindings.len(), count);
        assert_eq!(runtime.next_global_order, order);
        runtime.allocated = 0;
        runtime.steps = 0;
        assert!(runtime.window_own_keys().unwrap_err().is_resource_limit());
        assert_eq!(runtime.allocated, 0);
        runtime.steps = MAX_STEPS;
        runtime.next_global_order = u64::MAX;
        assert!(
            runtime
                .define(0, "counterRefusal", Value::Null, true)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(
            !runtime.environments[0]
                .bindings
                .contains_key("counterRefusal")
        );
        assert_eq!(runtime.environments[0].bindings.len(), count);
        let prior = runtime.environments[0].bindings["globalThis"].global_order;
        assert_eq!(runtime.global_creation_order("globalThis").unwrap(), prior);
        assert!(std::mem::size_of::<Binding>() <= BINDING_BYTES);
    }
}
