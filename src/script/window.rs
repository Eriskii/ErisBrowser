//! Reflection over the supported Window object-environment binding records.
use super::*;

pub(super) struct GlobalProperty {
    pub order: u64,
    pub property: Property,
}

impl Runtime {
    pub(super) fn instantiate_global_statements(
        &mut self,
        unit: &Rc<code::Unit>,
        body: &[code::StmtId],
    ) -> Result<()> {
        self.work(body.len().saturating_add(1))?;
        self.validate_lexical(unit, body, 1)?;
        // All cross-script lexical conflicts precede property compatibility
        // checks. No declaration is published until these checks succeed.
        self.hoist_vars_mode(unit, body, 1, false)?;
        for statement in body {
            if let code::Stmt::Function(name, _) = unit.stmt(*statement)
                && self.environments[1].bindings.contains_key(name)
            {
                return Err(ScriptError::syntax(format!(
                    "global lexical binding conflicts with function '{name}'"
                )));
            }
        }
        let mut functions = BTreeMap::new();
        for (index, statement) in body.iter().enumerate().rev() {
            self.tick()?;
            if let code::Stmt::Function(name, _) = unit.stmt(*statement) {
                let comparisons = (functions.len() + 1).ilog2() as usize + 1;
                self.work(comparisons.saturating_mul(1 + name.len()))?;
                if functions.contains_key(name.as_str()) {
                    continue;
                }
                if self.environments[0]
                    .bindings
                    .get(name)
                    .is_some_and(|binding| {
                        !(binding.deletable
                            || binding.accessor.is_none() && binding.mutable && binding.enumerable)
                    })
                {
                    return Err(ScriptError::type_error(format!(
                        "global function conflicts with {name} property"
                    )));
                }
                self.charge(128)?;
                functions.insert(name.as_str(), index);
            }
        }
        self.instantiate_lexical(unit, body, 1)?;
        // Last declarations win, in source order of those last declarations.
        // Function properties are created before any new var properties.
        for (index, statement) in body.iter().enumerate() {
            self.tick()?;
            if let code::Stmt::Function(name, code) = unit.stmt(*statement) {
                self.work(
                    ((functions.len() + 1).ilog2() as usize + 1).saturating_mul(1 + name.len()),
                )?;
                if functions.get(name.as_str()) != Some(&index) {
                    continue;
                }
                let function = self.function_value(&code::FunctionRef::new(unit, *code), 1)?;
                let key = self.global_name_key(name)?;
                let descriptor = if self.environments[0]
                    .bindings
                    .get(name)
                    .is_some_and(|binding| !binding.deletable)
                {
                    PropertyDescriptor {
                        value: Some(function),
                        ..PropertyDescriptor::default()
                    }
                } else {
                    PropertyDescriptor::data_property(function, true, true, false)
                };
                if !self.define_own(&Value::Window, &key, descriptor)? {
                    return Err(ScriptError::type_error(format!(
                        "cannot define global {name} function"
                    )));
                }
            }
        }
        self.hoist_vars(unit, body, 1)
    }

    pub(super) fn store_window_property(
        &mut self,
        key: &JsString,
        property: Property,
    ) -> Result<()> {
        if let Some(kind) = TrackedGlobal::from_key(key) {
            return self.store_global_property(kind.name(), property);
        }
        self.work(1 + key.len())?;
        self.charge(32 + key.len().saturating_mul(6))?;
        match key.to_utf8() {
            Ok(name) => {
                if event_handler_name(&name) {
                    return Err(ScriptError::unsupported(
                        "Window event-handler descriptor definition is not implemented",
                    ));
                }
                self.store_global_property(&name, property)
            }
            Err(_) => {
                if let Some(current) = self.global_non_scalar.get_mut(key) {
                    current.property = property;
                    return Ok(());
                }
                self.charge(256 + key.byte_len())?;
                let order = self.next_global_order;
                self.next_global_order = order.checked_add(1).ok_or_else(|| {
                    ScriptError::resource("global property creation counter exhausted")
                })?;
                self.global_non_scalar
                    .insert(key.clone(), GlobalProperty { order, property });
                Ok(())
            }
        }
    }

    // The caller has already checked the live own/inherited descriptor. Updating
    // its data value must not re-enter identifier assignment or reset attributes.
    pub(super) fn window_write_data(&mut self, key: &JsString, value: Value) -> Result<()> {
        if let Some(kind) = TrackedGlobal::from_key(key) {
            if let Some(binding) = self.environments[0].bindings.get_mut(kind.name()) {
                binding.value = value;
                return Ok(());
            }
        } else {
            self.work(1 + key.len())?;
            self.charge(32 + key.len().saturating_mul(6))?;
            match key.to_utf8() {
                Ok(name) => {
                    if let Some(binding) = self.environments[0].bindings.get_mut(&name) {
                        binding.value = value;
                        return Ok(());
                    }
                }
                Err(_) => {
                    if let Some(current) = self.global_non_scalar.get_mut(key)
                        && let PropertyValue::Data {
                            value: previous, ..
                        } = &mut current.property.value
                    {
                        *previous = value;
                        return Ok(());
                    }
                }
            }
        }
        self.store_window_property(key, Property::data(value, true, true, true))
    }

    pub(super) fn window_lookup_budget(&mut self, key: &JsString) -> Result<()> {
        if TrackedGlobal::from_key(key).is_some() {
            return Ok(());
        }
        let comparisons = (self.environments[0].bindings.len() + self.global_non_scalar.len() + 1)
            .ilog2() as usize
            + 1;
        self.work(1 + comparisons.saturating_mul(key.len()))?;
        // UTF-16 decoding may reserve and grow a temporary UTF-8 buffer, even
        // for a rejected lone-surrogate name. Charge before decoding.
        self.charge(32 + key.len().saturating_mul(6))
    }

    pub(super) fn global_name_key(&mut self, name: &str) -> Result<JsString> {
        if let Some(kind) = TrackedGlobal::from_name(name) {
            return Ok(self.global_key(kind));
        }
        self.work(1 + name.len() / 8)?;
        self.charge(64 + name.len().saturating_mul(4))?;
        Ok(name.into())
    }

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
        let comparisons = 1
            + (self.environments[0].bindings.len() + self.global_non_scalar.len())
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
            Err(_) => Ok(self.global_non_scalar.get(key).map(|p| p.property.clone())),
        }
    }

    pub(super) fn window_own_keys(&mut self) -> Result<Vec<JsString>> {
        self.work(
            self.environments[0]
                .bindings
                .len()
                .saturating_add(self.global_non_scalar.len())
                .saturating_add(1),
        )?;
        let (mut count, mut bytes) = self.environments[0]
            .bindings
            .iter()
            .filter(|(_, binding)| binding.global_property)
            .fold((0usize, 0usize), |(n, bytes), (name, _)| {
                (n + 1, bytes.saturating_add(name.len()))
            });
        count += self.global_non_scalar.len();
        bytes = self
            .global_non_scalar
            .keys()
            .fold(bytes, |n, key| n.saturating_add(key.byte_len()));
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
        for (key, record) in &self.global_non_scalar {
            entries.push((None, record.order, key.clone()));
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
    fn original_window_probes_retain_all_sources() {
        for line in include_str!("../../tests/fixtures/window-reflection.tsv").lines() {
            let (_, source) = line.split_once('\t').unwrap();
            check(source);
        }
    }

    #[test]
    fn original_global_binding_and_rhs_probes_retain_all_sources() {
        for line in include_str!("../../tests/fixtures/window-global-bindings.tsv").lines() {
            let (_, source) = line.split_once('\t').unwrap();
            check(source);
        }
    }

    #[test]
    fn author_this_properties_cannot_replace_execution_or_event_receivers() {
        check(
            r#"
            var w=window, gets=0, receiver={n:3};
            Object.defineProperty(w,'this',{get:function(){gets++;return 7;},configurable:true});
            assert.sameValue(this,w);assert.sameValue(typeof this,'object');
            function ordinary(a=this){return [this,a,()=>this];}
            var values=ordinary.call(receiver);
            assert.sameValue(values[0],receiver);assert.sameValue(values[1],receiver);
            assert.sameValue(values[2].call(w),receiver);
            var lexical=()=>this;assert.sameValue(lexical.call(receiver),w);
            function strict(){'use strict';return this;}
            assert.sameValue(strict.call(3),3);assert.sameValue(strict.call(null),null);
            assert.sameValue(strict(),undefined);
            var bound=ordinary.bind(receiver);assert.sameValue(bound()[0],receiver);
            function C(){this.n=4;this.arrow=()=>this;}
            var instance=new C();assert.sameValue(instance.n,4);assert.sameValue(instance.arrow(),instance);
            var hits=0;w.addEventListener('identity',function(e){
                assert.sameValue(this,w);assert.sameValue(e.target,w);hits++;
            });
            globalThis=9;w.dispatchEvent(new Event('identity'));
            assert.sameValue(hits,1);assert.sameValue(this,w);assert.sameValue(gets,0);
            assert.sameValue(w['this'],7);assert.sameValue(gets,1);
            delete w['this'];assert.sameValue(this,w);assert.sameValue(w['this'],undefined);
        "#,
        );
    }

    #[test]
    fn global_accessors_use_live_records_and_identifier_call_receivers() {
        check(
            r#"
            var w=window,log='',value=1,reason={},seen;
            Object.defineProperty(w,'access',{get:function(){assert.sameValue(this,w);log+='g';return value;},
                set:function(v){assert.sameValue(this,w);log+='s';value=v;},configurable:true});
            access+=2;assert.sameValue(log,'gs');assert.sameValue(value,3);
            Object.defineProperty(w,'bare',{get:function(){return function(){'use strict';return this;};},configurable:true});
            assert.sameValue(bare(),undefined);assert.sameValue(w.bare(),w);
            Object.defineProperty(w,'access',{set:function(){throw reason;}});
            try{access=9;}catch(e){seen=e;}assert.sameValue(seen,reason);
            Object.defineProperty(w,'access',{get:function(){delete w.access;return 5;}});
            assert.sameValue(access,5);assert.sameValue(typeof access,'undefined');
            assert.throws(ReferenceError,function(){return access;});
            w.access=1;
            function replace(){Object.defineProperty(w,'access',{value:2,writable:false});return 3;}
            var caught;try{access=replace();}catch(e){caught=e;}
            if((function(){return this;})()===undefined)assert(caught instanceof TypeError);
            else assert.sameValue(caught,undefined);
            assert.sameValue(access,2);
            Object.defineProperty(Object.prototype,'inheritedReadonly',{value:8,writable:false,configurable:true});
            assert.sameValue(inheritedReadonly,8);assert.sameValue(w.hasOwnProperty('inheritedReadonly'),false);
            assert.throws(TypeError,function(){'use strict';inheritedReadonly=9;});
        "#,
        );
    }

    #[test]
    fn utf16_global_keys_preserve_descriptors_and_shared_creation_order() {
        check(
            r#"
            var w=window,lo='\ud800',hi='\udfff',replacement='\ufffd',s=Symbol(),n=0;
            w[lo]=1;w.middle=2;
            Object.defineProperty(w,hi,{get:function(){assert.sameValue(this,w);n++;return 3;},configurable:true});
            w[replacement]=4;w[s]=5;w[7]=6;w[2]=7;
            var names=Object.getOwnPropertyNames(w);
            assert.sameValue(names[0],'2');assert.sameValue(names[1],'7');
            assert(names.indexOf(lo)<names.indexOf('middle'));
            assert(names.indexOf('middle')<names.indexOf(hi));assert(names.indexOf(hi)<names.indexOf(replacement));
            assert.sameValue(n,0);assert.sameValue(w[hi],3);assert.sameValue(n,1);
            assert.sameValue(w.propertyIsEnumerable(lo),true);assert.sameValue(w.propertyIsEnumerable(hi),false);
            Object.defineProperty(w,lo,{value:9,writable:false,configurable:false});
            assert.sameValue(w[lo],9);assert.sameValue(w[replacement],4);
            assert.throws(TypeError,function(){'use strict';w[lo]=10;});
            assert.throws(TypeError,function(){Object.defineProperty(w,lo,{get:function(){}});});
            assert.throws(TypeError,function(){'use strict';delete w[lo];});
            delete w[hi];w[hi]=8;
            names=Reflect.ownKeys(w);assert.sameValue(names[names.length-2],hi);assert.sameValue(names[names.length-1],s);
            assert.sameValue(w[hi],8);assert.sameValue(w.hasOwnProperty(hi),true);
        "#,
        );
    }

    #[test]
    fn global_declarations_validate_before_publication_across_scripts() {
        for strict in [false, true] {
            for descriptor in [
                "{value:1,writable:false,enumerable:true}",
                "{value:1,writable:true,enumerable:false}",
                "{get:function(){throw 'getter must not run';},enumerable:true}",
            ] {
                let (mut runtime, mut doc) = harness();
                runtime.execute(&format!("Object.defineProperty(window,'locked',{descriptor});let lexicalConflict=1;"),&mut doc).unwrap();
                for (source, expected) in [
                    (
                        "let untouchedLexical;var untouchedVar;function okay(){}function locked(){}",
                        "TypeError",
                    ),
                    ("function locked(){}var lexicalConflict;", "SyntaxError"),
                    (
                        "function locked(){}function lexicalConflict(){}",
                        "SyntaxError",
                    ),
                ] {
                    let error = if strict {
                        runtime.execute_strict(source, &mut doc)
                    } else {
                        runtime.execute(source, &mut doc)
                    }
                    .unwrap_err();
                    assert_eq!(error.name(), expected, "{source}: {error}");
                    assert!(
                        !runtime.environments[1]
                            .bindings
                            .contains_key("untouchedLexical")
                    );
                    assert!(
                        !runtime.environments[0]
                            .bindings
                            .contains_key("untouchedVar")
                    );
                    assert!(!runtime.environments[0].bindings.contains_key("okay"));
                    assert_eq!(runtime.calls, 0);
                    assert!(runtime.frames.is_empty());
                }
            }
            for descriptor in [
                "{value:1,writable:false,enumerable:false,configurable:true}",
                "{get:function(){throw 'getter must not run';},configurable:true}",
                "{value:1,writable:true,enumerable:true,configurable:false}",
            ] {
                let (mut runtime, mut doc) = harness();
                runtime
                    .execute(
                        &format!("Object.defineProperty(window,'replaceable',{descriptor});"),
                        &mut doc,
                    )
                    .unwrap();
                let source = "var newVar;function replaceable(){return 1;}function middle(){}function replaceable(){return 2;}assert.sameValue(replaceable(),2);verifyProperty(window,'replaceable',{writable:true,enumerable:true,configurable:false});var names=Object.getOwnPropertyNames(window);assert(names.indexOf('middle')<names.indexOf('newVar'));";
                if strict {
                    runtime.execute_strict(source, &mut doc)
                } else {
                    runtime.execute(source, &mut doc)
                }
                .unwrap();
            }
        }
        check(
            "var firstVar;function duplicate(){return 1;}function between(){}function duplicate(){return 2;}var keys=Object.getOwnPropertyNames(window);assert(keys.indexOf('between')<keys.indexOf('duplicate'));assert(keys.indexOf('duplicate')<keys.indexOf('firstVar'));assert.sameValue(duplicate(),2);",
        );
    }

    #[test]
    fn global_vars_preserve_own_records_and_shadow_inherited_properties() {
        for strict in [false, true] {
            let (mut runtime, mut doc) = harness();
            runtime.execute(r#"
                var sets=0;
                Object.defineProperty(window,'existing',{get:function(){return 1;},set:function(v){sets=v;},configurable:true});
                Object.defineProperty(Object.prototype,'fromProto',{value:8,writable:false,configurable:true});
            "#,&mut doc).unwrap();
            let source = "var existing;assert.sameValue(sets,0);var existing=9;assert.sameValue(sets,9);assert.sameValue(existing,1);var fromProto;assert.sameValue(fromProto,undefined);verifyProperty(window,'fromProto',{writable:true,enumerable:true,configurable:false});";
            if strict {
                runtime.execute_strict(source, &mut doc)
            } else {
                runtime.execute(source, &mut doc)
            }
            .unwrap();
            runtime.execute("let existing=4;assert.sameValue(existing,4);assert.sameValue(window.existing,1);",&mut doc).unwrap();
            assert_eq!(
                runtime
                    .execute("let fromProto;", &mut doc)
                    .unwrap_err()
                    .name(),
                "SyntaxError"
            );
        }
    }

    #[test]
    fn general_global_storage_and_callbacks_share_uncatchable_limits() {
        for key in [JsString::from("generalName"), JsString::from(vec![0xd800])] {
            for work_limit in [false, true] {
                let mut runtime = Runtime::new();
                let count = runtime.environments[0].bindings.len();
                let order = runtime.next_global_order;
                if work_limit {
                    runtime.steps = 0;
                } else {
                    runtime.allocated = MAX_HEAP;
                }
                assert!(
                    runtime
                        .define_own(
                            &Value::Window,
                            &key,
                            PropertyDescriptor::data_property(Value::Null, true, true, true)
                        )
                        .unwrap_err()
                        .is_resource_limit()
                );
                assert_eq!(runtime.environments[0].bindings.len(), count);
                assert!(runtime.global_non_scalar.is_empty());
                assert_eq!(runtime.next_global_order, order);
            }
            let mut runtime = Runtime::new();
            runtime.next_global_order = u64::MAX;
            assert!(
                runtime
                    .define_own(
                        &Value::Window,
                        &key,
                        PropertyDescriptor::data_property(Value::Null, true, true, true)
                    )
                    .unwrap_err()
                    .is_resource_limit()
            );
            assert!(runtime.own_property(&Value::Window, &key).is_none());
        }
        for source in [
            "Object.defineProperty(window,'recursiveGlobal',{get:function(){return recursiveGlobal;}});var caught=false;try{recursiveGlobal;}catch(e){caught=true;}",
            "Object.defineProperty(window,'recursiveGlobal',{set:function(v){recursiveGlobal=v;}});var caught=false;try{recursiveGlobal=1;}catch(e){caught=true;}",
            "Object.defineProperty(window,'loopGlobal',{get:function(){while(true){}}});var caught=false;try{loopGlobal;}catch(e){caught=true;}",
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
