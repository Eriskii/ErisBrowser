//! Constructor targets and bounded array-like argument lists.
use super::*;

impl Runtime {
    pub(super) fn new_target(&mut self, mut env: usize) -> Result<Value> {
        loop {
            self.tick()?;
            if let Some(target) = &self.environments[env].new_target_binding {
                return Ok(target.clone());
            }
            env = self.environments[env].parent.ok_or_else(|| {
                ScriptError::reference("missing new.target execution environment")
            })?;
        }
    }

    pub(super) fn is_constructor(&mut self, mut value: Value) -> Result<bool> {
        loop {
            self.tick()?;
            match value {
                Value::Function(id) => {
                    if let Some(bound) = &self.functions[id].bound {
                        value = bound.target.clone();
                    } else {
                        return Ok(self.functions[id].code.constructable);
                    }
                }
                Value::Native(native) => {
                    return Ok(native.receiver == Value::Window
                        && matches!(
                            native.name.as_str(),
                            "Object"
                                | "Function"
                                | "Array"
                                | "String"
                                | "Number"
                                | "Boolean"
                                | "Symbol"
                                | "RegExp"
                                | "Error"
                                | "TypeError"
                                | "SyntaxError"
                                | "ReferenceError"
                                | "RangeError"
                                | "EvalError"
                                | "URIError"
                                | "Event"
                                | "CustomEvent"
                                | "ToggleEvent"
                                | "EventTarget"
                                | "DOMException"
                                | "AbortController"
                                | "AbortSignal"
                        ));
                }
                _ => return Ok(false),
            }
        }
    }

    pub(super) fn argument_list(
        &mut self,
        object: Value,
        doc: &mut Document,
    ) -> Result<Vec<Value>> {
        if !js_object(&object) {
            return Err(ScriptError::type_error("arguments list must be an object"));
        }
        let length = self.get(object.clone(), "length", doc)?;
        if let Value::String(text) = &length {
            self.work(1 + text.len() / 16)?;
        }
        let length =
            integer_or_infinity(self.number_value(length, doc)?).clamp(0.0, 9007199254740991.0);
        if length > 65536.0 {
            return Err(ScriptError::resource("apply argument limit exceeded"));
        }
        let length = length as usize;
        self.charge(length.saturating_mul(std::mem::size_of::<Value>()))?;
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("argument list allocation failed"))?;
        for index in 0..length {
            self.tick()?;
            arguments.push(self.get(object.clone(), &index.to_string(), doc)?);
        }
        Ok(arguments)
    }

    fn constructor_prototype(
        &mut self,
        target: Value,
        intrinsic: &str,
        doc: &mut Document,
    ) -> Result<Value> {
        let prototype = self.get(target, "prototype", doc)?;
        Ok(if js_object(&prototype) {
            prototype
        } else if intrinsic == "Array" {
            Value::Array(self.array_prototype.expect("Array intrinsic initialized"))
        } else {
            Value::Object(self.prototypes[intrinsic])
        })
    }

    fn constructed_prototype(&mut self, object: Value, prototype: Value) -> Value {
        let id = self
            .property_object(&object)
            .expect("allocated constructor result");
        self.objects[id].prototype = Some(prototype);
        object
    }

    pub(super) fn construct(
        &mut self,
        constructor: Value,
        arguments: Vec<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        if !self.is_constructor(constructor.clone())? {
            return Err(ScriptError::type_error("value is not a constructor"));
        }
        self.construct_with_target(constructor.clone(), arguments, constructor, doc)
    }

    pub(super) fn construct_with_target(
        &mut self,
        constructor: Value,
        arguments: Vec<Value>,
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.enter_stack(4)?;
        let result = self.construct_inner(constructor, arguments, new_target, doc);
        self.stack_units -= 4;
        result
    }

    fn construct_inner(
        &mut self,
        constructor: Value,
        arguments: Vec<Value>,
        mut new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        match &constructor {
            Value::Function(id) if self.functions[*id].bound.is_some() => {
                let count = self.functions[*id]
                    .bound
                    .as_ref()
                    .unwrap()
                    .arguments
                    .len()
                    .saturating_add(arguments.len());
                if count > 65536 {
                    return Err(ScriptError::resource(
                        "bound constructor argument limit exceeded",
                    ));
                }
                self.work(1 + count)?;
                self.charge(count.saturating_mul(std::mem::size_of::<Value>()))?;
                let mut combined = Vec::new();
                combined
                    .try_reserve_exact(count)
                    .map_err(|_| ScriptError::resource("bound constructor allocation failed"))?;
                let bound = self.functions[*id].bound.as_ref().unwrap();
                let target = bound.target.clone();
                combined.extend_from_slice(&bound.arguments);
                combined.extend(arguments);
                if new_target == constructor {
                    new_target = target.clone();
                }
                self.construct_with_target(target, combined, new_target, doc)
            }
            Value::Function(id) if self.functions[*id].code.constructable => {
                let prototype = self.constructor_prototype(new_target.clone(), "Object", doc)?;
                let instance = self.object_ordered([])?;
                let instance = self.constructed_prototype(instance, prototype);
                let result = self.call_with_new_target(
                    constructor,
                    arguments,
                    instance.clone(),
                    new_target,
                    doc,
                )?;
                Ok(if js_object(&result) { result } else { instance })
            }
            Value::Native(native) if native.receiver == Value::Window => {
                let name = native.name.as_str();
                match name {
                    // Symbol has [[Construct]] and can serve as another constructor's
                    // newTarget. Its own construction always throws before coercion.
                    "Symbol" => Err(ScriptError::type_error("Symbol cannot be constructed")),
                    "String" | "Number" | "Boolean" => {
                        if name == "String" && matches!(arguments.first(), Some(Value::Symbol(_))) {
                            return Err(ScriptError::type_error(
                                "cannot convert a symbol to a string",
                            ));
                        }
                        // Primitive conversion precedes the observable prototype lookup.
                        let primitive =
                            self.call(constructor.clone(), arguments, Value::Window, doc)?;
                        let prototype = self.constructor_prototype(new_target, name, doc)?;
                        let instance = self.coerce_object(primitive)?;
                        Ok(self.constructed_prototype(instance, prototype))
                    }
                    "Object" if constructor != new_target => {
                        // An alternate constructor target creates a fresh object and ignores value.
                        let prototype = self.constructor_prototype(new_target, "Object", doc)?;
                        let instance = self.object_ordered([])?;
                        Ok(self.constructed_prototype(instance, prototype))
                    }
                    "Array" | "Error" | "TypeError" | "SyntaxError" | "ReferenceError"
                    | "RangeError" | "EvalError" | "URIError" => {
                        let prototype = self.constructor_prototype(new_target, name, doc)?;
                        let instance = self.call(constructor, arguments, Value::Window, doc)?;
                        Ok(self.constructed_prototype(instance, prototype))
                    }
                    "RegExp" => {
                        let prototype = self.constructor_prototype(new_target, name, doc)?;
                        let instance = self.regexp_create(
                            arguments.first().cloned().unwrap_or(Value::Undefined),
                            arguments.get(1).cloned().unwrap_or(Value::Undefined),
                            false,
                            doc,
                        )?;
                        Ok(self.constructed_prototype(instance, prototype))
                    }
                    "Object" | "Function" => self.call(constructor, arguments, Value::Window, doc),
                    "Event" | "CustomEvent" | "ToggleEvent" | "EventTarget" | "DOMException"
                    | "AbortController" | "AbortSignal" => {
                        if constructor != new_target {
                            return Err(ScriptError::unsupported(
                                "alternate Web IDL constructor targets are not implemented",
                            ));
                        }
                        self.event_construct(name, &arguments, doc)
                    }
                    _ => Err(ScriptError::type_error("value is not a constructor")),
                }
            }
            _ => Err(ScriptError::type_error("value is not a constructor")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn harness() -> (Runtime, Document) {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<body></body>");
        for source in [
            include_str!("../../tests/upstream/test262/harness/sta.js"),
            include_str!("../../tests/upstream/test262/harness/assert.js"),
            include_str!("../../tests/upstream/test262/harness/propertyHelper.js"),
        ] {
            runtime.execute(source, &mut doc).unwrap();
        }
        (runtime, doc)
    }
    #[test]
    fn construction_semantics_in_both_modes() {
        for fixture in include_str!("../../tests/conformance/construction.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (name, source) = fixture.split_once('\n').unwrap();
            for strict in [false, true] {
                let (mut runtime, mut doc) = harness();
                let script = format!("{}\n{source}", if strict { "'use strict';" } else { "" });
                runtime
                    .execute(&script, &mut doc)
                    .unwrap_or_else(|error| panic!("{name} strict={strict}: {error}"));
                assert!(runtime.frames.is_empty());
                assert_eq!(runtime.calls, 0);
                assert_eq!(runtime.stack_units, 0);
            }
        }
    }
    #[test]
    fn symbol_constructor_targets_in_both_modes() {
        for fixture in include_str!("../../tests/conformance/symbol-constructor-targets.js")
            .split("// CASE: ")
            .skip(1)
        {
            let (name, source) = fixture.split_once('\n').unwrap();
            for strict in [false, true] {
                let (mut runtime, mut doc) = harness();
                let script = format!("{}\n{source}", if strict { "'use strict';" } else { "" });
                runtime
                    .execute(&script, &mut doc)
                    .unwrap_or_else(|error| panic!("{name} strict={strict}: {error}"));
                assert!(runtime.frames.is_empty());
                assert_eq!(runtime.calls, 0);
                assert_eq!(runtime.stack_units, 0);
            }
        }
    }
    #[test]
    fn symbol_constructor_callbacks_and_argument_lists_retain_resource_limits() {
        for source in [
            "try{Reflect.construct(Symbol,{length:Infinity});}catch(e){console.log('caught');}",
            "function read(){return read();}try{Reflect.construct(Symbol,{get length(){return read();}});}catch(e){console.log('caught');}",
            "var B=Symbol.bind(null);Object.defineProperty(B,'prototype',{get:function(){return Reflect.construct(Object,[],B);}});try{Reflect.construct(Object,[],B);}catch(e){console.log('caught');}",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("<body>");
            let error = runtime.execute(source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit(), "{source}: {error}");
            assert!(runtime.console.is_empty());
            assert!(runtime.frames.is_empty());
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
        }
    }
    #[test]
    fn new_target_parse_contexts_and_assignment_restrictions() {
        for source in [
            "new.target;",
            "()=>new.target;",
            "(x=new.target)=>x;",
            "()=>()=>new.target;",
            "function f(){new.target=1;}",
            "function f(){new.target++;}",
            "function f(){++new.target;}",
            "function f(){new.other;}",
            r"function f(){new.targ\u0065t;}",
            r"function f(){n\u0065w.target;}",
            "function f(){} new.target;",
            "function f(){} ()=>new.target;",
        ] {
            for strict in [false, true] {
                let error = parser::Parser::program_context(source, false, strict).unwrap_err();
                assert!(error.is_parse_error(), "{source}: {error}");
                assert_eq!(error.intrinsic_error_name(), Some("SyntaxError"));
            }
        }
        for source in [
            "function f(x=new.target){return new.target;}",
            "()=>function(){return ()=>new.target;};",
            "function f(){return (x=new.target)=>x;}",
            "({get x(){return new.target;}, set x(v){new.target;}, m(){return new.target;}});",
            "function f(){return new\n.\ntarget;}",
            "function f(){return new new.target();}",
        ] {
            parser::Parser::program(source).unwrap_or_else(|e| panic!("{source}: {e}"));
            parser_legacy::Parser::program(source)
                .unwrap_or_else(|e| panic!("legacy {source}: {e}"));
        }
    }
    #[test]
    fn reflection_resources_abort_without_catching_or_leaking_activations() {
        for source in [
            "try{Reflect.apply(function(){},null,{length:Infinity});}catch(e){console.log('caught');}",
            "try{Reflect.construct(function(){},{length:65537});}catch(e){console.log('caught');}",
            "function F(){Reflect.construct(F,[]);}try{new F();}catch(e){console.log('caught');}",
            "var B=(function(){}).bind(null);Object.defineProperty(B,'prototype',{get:function(){return Reflect.construct(function(){},[],B);}});try{Reflect.construct(function(){},[],B);}catch(e){console.log('caught');}",
            "function f(){Reflect.apply(f,null,[]);}try{f();}catch(e){console.log('caught');}",
        ] {
            let mut runtime = Runtime::new();
            let mut doc = Document::parse("<body>");
            let error = runtime.execute(source, &mut doc).unwrap_err();
            assert!(error.is_resource_limit(), "{source}: {error}");
            assert!(runtime.console.is_empty());
            assert!(runtime.frames.is_empty());
            assert_eq!(runtime.calls, 0);
            assert_eq!(runtime.stack_units, 0);
            assert_eq!(
                runtime
                    .execute("function f(){return new.target;} f();", &mut doc)
                    .unwrap(),
                Value::Undefined
            );
        }
    }
    #[test]
    fn argument_list_prepays_storage_before_reading_indices() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<body>");
        let list = runtime
            .execute("({length:65536,get 0(){console.log('read');}})", &mut doc)
            .unwrap();
        runtime.allocated = MAX_HEAP - 128;
        assert!(
            runtime
                .argument_list(list, &mut doc)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.console.is_empty());
    }
    #[test]
    fn nonconstructor_bound_target_is_rejected_before_argument_copy() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<body>");
        let target = runtime.execute("(()=>0).bind(null,1)", &mut doc).unwrap();
        runtime.allocated = MAX_HEAP;
        let error = runtime.construct(target, Vec::new(), &mut doc).unwrap_err();
        assert_eq!(error.intrinsic_error_name(), Some("TypeError"));
        assert!(!error.is_resource_limit());
    }
    #[test]
    fn alternate_host_targets_remain_explicitly_unsupported() {
        let mut runtime = Runtime::new();
        let mut doc = Document::parse("<body>");
        let error = runtime
            .execute("Reflect.construct(Event,['x'],function(){});", &mut doc)
            .unwrap_err();
        assert!(error.is_unsupported());
    }
}
