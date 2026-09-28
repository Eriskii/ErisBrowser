//! Symbol identities and property keys. Descriptions never identify a symbol.
use super::*;
use std::cmp::Ordering;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Symbol(Rc<Data>);

#[derive(Debug)]
struct Data {
    description: Option<JsString>,
    well_known: Option<u8>,
}

#[derive(Default)]
pub(super) struct State {
    registry: BTreeMap<JsString, Symbol>,
    well_known: BTreeMap<&'static str, Symbol>,
}

const WELL_KNOWN: &[&str] = &[
    "asyncDispose",
    "asyncIterator",
    "dispose",
    "hasInstance",
    "isConcatSpreadable",
    "iterator",
    "match",
    "matchAll",
    "replace",
    "search",
    "species",
    "split",
    "toPrimitive",
    "toStringTag",
    "unscopables",
];

impl Symbol {
    pub(super) fn unique(description: Option<JsString>) -> Self {
        Self(Rc::new(Data {
            description,
            well_known: None,
        }))
    }

    pub(super) fn well_known(index: u8, description: JsString) -> Self {
        Self(Rc::new(Data {
            description: Some(description),
            well_known: Some(index),
        }))
    }

    pub(super) fn description(&self) -> Option<&JsString> {
        self.0.description.as_ref()
    }

    fn identity(&self) -> (bool, usize) {
        match self.0.well_known {
            Some(index) => (false, usize::from(index)),
            None => (true, Rc::as_ptr(&self.0) as usize),
        }
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.identity() == other.identity()
    }
}
impl Eq for Symbol {}
impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity().cmp(&other.identity())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum PropertyKey {
    String(JsString),
    Symbol(Symbol),
}

impl PropertyKey {
    pub fn as_string(&self) -> Option<&JsString> {
        match self {
            Self::String(key) => Some(key),
            Self::Symbol(_) => None,
        }
    }

    pub fn value(&self) -> Value {
        match self {
            Self::String(key) => Value::String(key.clone()),
            Self::Symbol(key) => Value::Symbol(key.clone()),
        }
    }

    pub fn byte_len(&self) -> usize {
        self.as_string().map_or(0, JsString::byte_len)
    }
}

impl From<JsString> for PropertyKey {
    fn from(value: JsString) -> Self {
        Self::String(value)
    }
}
impl From<&JsString> for PropertyKey {
    fn from(value: &JsString) -> Self {
        Self::String(value.clone())
    }
}
impl From<&str> for PropertyKey {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}
impl From<String> for PropertyKey {
    fn from(value: String) -> Self {
        Self::String(value.into())
    }
}
impl From<Symbol> for PropertyKey {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}

impl From<&PropertyKey> for PropertyKey {
    fn from(value: &PropertyKey) -> Self {
        value.clone()
    }
}

impl Runtime {
    pub(super) fn initialize_symbols(&mut self) -> Result<()> {
        let constructor = self.native_properties["Symbol"];
        let prototype = self.prototypes["Symbol"];
        self.objects[constructor].insert_property(
            "length".into(),
            Property::data(Value::Number(0.0), false, false, true),
        );
        for (index, &name) in WELL_KNOWN.iter().enumerate() {
            self.work(name.len() + 1)?;
            self.charge(512 + name.len() * 8)?;
            let symbol = Symbol::well_known(index as u8, format!("Symbol.{name}").into());
            self.objects[constructor].insert_property(
                name.into(),
                Property::data(Value::Symbol(symbol.clone()), false, false, false),
            );
            self.symbols.well_known.insert(name, symbol);
        }
        for (name, length) in [("for", 1), ("keyFor", 1)] {
            let function = self.intrinsic_function(&format!("Symbol.{name}"), name, length)?;
            self.objects[constructor].insert_hidden(name.into(), function);
        }
        for name in ["toString", "valueOf"] {
            let function = self.intrinsic_function(&format!("Symbol.{name}"), name, 0)?;
            self.objects[prototype].insert_hidden(name.into(), function);
        }
        let getter = self.intrinsic_function("Symbol.description", "get description", 0)?;
        self.objects[prototype].insert_property(
            "description".into(),
            Property {
                value: PropertyValue::Accessor {
                    get: getter,
                    set: Value::Undefined,
                },
                enumerable: false,
                configurable: true,
            },
        );
        let method = self.intrinsic_function("Symbol.toPrimitive", "[Symbol.toPrimitive]", 1)?;
        let key = self.well_known_key("toPrimitive");
        self.objects[prototype].insert_property(key, Property::data(method, false, false, true));
        let tag = self.well_known_key("toStringTag");
        for (id, name) in [
            (prototype, "Symbol"),
            (self.native_properties["Math"], "Math"),
            (self.native_properties["JSON"], "JSON"),
        ] {
            self.charge(256)?;
            self.objects[id].insert_property(
                tag.clone(),
                Property::data(Value::String(name.into()), false, false, true),
            );
        }
        let function =
            self.intrinsic_function("Object.getOwnPropertySymbols", "getOwnPropertySymbols", 1)?;
        self.objects[self.native_properties["Object"]]
            .insert_hidden("getOwnPropertySymbols".into(), function);
        self.charge(1024)?;
        let reflect = self.object_ordered([])?;
        let Value::Object(id) = reflect else {
            unreachable!()
        };
        let own_keys = self.intrinsic_function("Reflect.ownKeys", "ownKeys", 1)?;
        self.objects[id].insert_hidden("ownKeys".into(), own_keys);
        self.objects[id].insert_property(
            tag,
            Property::data(Value::String("Reflect".into()), false, false, true),
        );
        self.environments[0].bindings.insert(
            "Reflect".into(),
            Binding {
                value: reflect,
                accessor: None,
                mutable: true,
                initialized: true,
                strict_immutable: false,
                global_property: true,
                enumerable: false,
                deletable: true,
            },
        );
        let method = self.intrinsic_function("Function.hasInstance", "[Symbol.hasInstance]", 1)?;
        let key = self.well_known_key("hasInstance");
        let id = self.functions[self.function_prototype].properties;
        self.objects[id].insert_property(key, Property::data(method, false, false, false));
        Ok(())
    }

    pub(super) fn well_known_key(&self, name: &str) -> PropertyKey {
        PropertyKey::Symbol(self.symbols.well_known[name].clone())
    }

    pub(super) fn symbol_text(&mut self, symbol: &Symbol) -> Result<JsString> {
        let length = symbol
            .description()
            .map_or(0, JsString::len)
            .saturating_add(8);
        if length > MAX_STRING {
            return Err(ScriptError::resource(
                "symbol description string limit exceeded",
            ));
        }
        self.work(1 + length / 8)?;
        self.charge(length.saturating_mul(4) + 32)?;
        let mut text = Vec::new();
        text.try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("symbol description allocation failed"))?;
        text.extend("Symbol(".encode_utf16());
        if let Some(description) = symbol.description() {
            text.extend_from_slice(description.units());
        }
        text.push(u16::from(b')'));
        Ok(text.into())
    }

    fn this_symbol(&self, value: Value) -> Result<Symbol> {
        match value {
            Value::Symbol(symbol) => Ok(symbol),
            Value::Object(id) => match &self.objects[id].boxed {
                Some(Value::Symbol(symbol)) => Ok(symbol.clone()),
                _ => Err(ScriptError::type_error("receiver is not a symbol")),
            },
            _ => Err(ScriptError::type_error("receiver is not a symbol")),
        }
    }

    pub(super) fn symbol_native(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let argument = args.first().cloned().unwrap_or(Value::Undefined);
        match name {
            "Symbol" => {
                let description = if argument == Value::Undefined {
                    None
                } else {
                    Some(self.string_hint(argument, doc)?)
                };
                self.charge(64 + description.as_ref().map_or(0, JsString::byte_len))?;
                Ok(Value::Symbol(Symbol::unique(description)))
            }
            "Symbol.for" => {
                let key = self.string_hint(argument, doc)?;
                let comparisons =
                    1 + self.symbols.registry.len().checked_ilog2().unwrap_or(0) as usize;
                self.work(
                    1 + comparisons
                        .saturating_mul(6)
                        .saturating_mul(1 + key.len() / 8),
                )?;
                if let Some(symbol) = self.symbols.registry.get(&key) {
                    return Ok(Value::Symbol(symbol.clone()));
                }
                self.charge(256 + key.byte_len().saturating_mul(2))?;
                let symbol = Symbol::unique(Some(key.clone()));
                self.symbols.registry.insert(key, symbol.clone());
                Ok(Value::Symbol(symbol))
            }
            "Symbol.keyFor" => {
                let Value::Symbol(symbol) = argument else {
                    return Err(ScriptError::type_error("Symbol.keyFor requires a symbol"));
                };
                self.work(self.symbols.registry.len() + 1)?;
                Ok(self
                    .symbols
                    .registry
                    .iter()
                    .find(|(_, value)| **value == symbol)
                    .map_or(Value::Undefined, |(key, _)| Value::String(key.clone())))
            }
            "Symbol.description" => {
                let symbol = self.this_symbol(receiver)?;
                Ok(symbol
                    .description()
                    .cloned()
                    .map_or(Value::Undefined, Value::String))
            }
            "Symbol.toString" => {
                let symbol = self.this_symbol(receiver)?;
                self.symbol_text(&symbol).map(Value::String)
            }
            "Symbol.valueOf" | "Symbol.toPrimitive" => {
                self.this_symbol(receiver).map(Value::Symbol)
            }
            _ => Err(ScriptError::unsupported(
                "symbol operation is not implemented",
            )),
        }
    }

    pub(super) fn exotic_primitive(
        &mut self,
        value: &Value,
        hint: &str,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        if !js_object(value) {
            return Ok(None);
        }
        let key = self.well_known_key("toPrimitive");
        let method = self.get_property_key(value.clone(), &key, doc)?;
        if matches!(method, Value::Undefined | Value::Null) {
            return Ok(None);
        }
        if !json_callable(&method) {
            return Err(ScriptError::type_error(
                "Symbol.toPrimitive must be callable",
            ));
        }
        self.charge(std::mem::size_of::<Value>() + 64)?;
        let result = self.call(method, vec![Value::String(hint.into())], value.clone(), doc)?;
        if js_object(&result) {
            return Err(ScriptError::type_error(
                "Symbol.toPrimitive must return a primitive",
            ));
        }
        Ok(Some(result))
    }

    pub(super) fn property_key(&mut self, value: Value, doc: &mut Document) -> Result<PropertyKey> {
        match self.string_primitive(value, doc)? {
            Value::Symbol(symbol) => Ok(PropertyKey::Symbol(symbol)),
            Value::String(text) => Ok(PropertyKey::String(text)),
            primitive => {
                self.charge(1024)?;
                Ok(PropertyKey::String(primitive.js_string()))
            }
        }
    }

    pub(super) fn set_key_function_name(
        &mut self,
        function: &Value,
        key: &PropertyKey,
        prefix: Option<&str>,
    ) -> Result<()> {
        match key {
            PropertyKey::String(key) => self.set_function_name(function, key, prefix),
            PropertyKey::Symbol(symbol) => {
                let name = if let Some(description) = symbol.description() {
                    let length = description.len().saturating_add(2);
                    if length > MAX_STRING {
                        return Err(ScriptError::resource("function name string limit exceeded"));
                    }
                    self.work(1 + length / 8)?;
                    self.charge(length.saturating_mul(4) + 32)?;
                    let mut units = Vec::new();
                    units
                        .try_reserve_exact(length)
                        .map_err(|_| ScriptError::resource("function name allocation failed"))?;
                    units.push(u16::from(b'['));
                    units.extend_from_slice(description.units());
                    units.push(u16::from(b']'));
                    JsString::from(units)
                } else {
                    JsString::default()
                };
                self.set_function_name(function, &name, prefix)
            }
        }
    }
}

impl Runtime {
    // Bound functions re-enter InstanceofOperator on their target. Keep that
    // delegation iterative, including lookup of a target's custom hook.
    pub(super) fn instance_of(
        &mut self,
        value: Value,
        mut target: Value,
        mut ordinary: bool,
        doc: &mut Document,
    ) -> Result<bool> {
        for _ in 0..MAX_DEPTH {
            self.tick()?;
            if !ordinary {
                if !js_object(&target) {
                    return Err(ScriptError::type_error(
                        "instanceof right-hand side is not an object",
                    ));
                }
                let hook = self.get_property_key(
                    target.clone(),
                    &self.well_known_key("hasInstance"),
                    doc,
                )?;
                if !matches!(hook, Value::Null | Value::Undefined) {
                    if !json_callable(&hook) {
                        return Err(ScriptError::type_error(
                            "Symbol.hasInstance is not callable",
                        ));
                    }
                    if matches!(&hook, Value::Native(native) if native.name == "Function.hasInstance")
                    {
                        ordinary = true;
                    } else {
                        self.charge(std::mem::size_of::<Value>() + 32)?;
                        return Ok(self.call(hook, vec![value], target, doc)?.truthy());
                    }
                }
            }
            if !json_callable(&target) {
                return if ordinary {
                    Ok(false)
                } else {
                    Err(ScriptError::type_error(
                        "instanceof right-hand side is not callable",
                    ))
                };
            }
            if let Value::Function(id) = &target
                && let Some(bound) = &self.functions[*id].bound
            {
                target = bound.target.clone();
                ordinary = false;
                continue;
            }
            if !js_object(&value) {
                return Ok(false);
            }
            let prototype = self.get(target, "prototype", doc)?;
            if !js_object(&prototype) {
                return Err(ScriptError::type_error(
                    "constructor prototype is not an object",
                ));
            }
            let mut cursor = self.prototype_of(&value);
            for _ in 0..MAX_DEPTH {
                self.tick()?;
                let Some(current) = cursor else {
                    return Ok(false);
                };
                if current == prototype {
                    return Ok(true);
                }
                cursor = self.prototype_of(&current);
            }
            return Err(ScriptError::resource("prototype chain limit exceeded"));
        }
        Err(ScriptError::resource("bound function chain limit exceeded"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(source: &str) {
        for strict in [false, true] {
            let mut runtime = Runtime::new();
            let mut document = Document::parse("");
            for harness in [
                include_str!("../../tests/upstream/test262/harness/sta.js"),
                include_str!("../../tests/upstream/test262/harness/assert.js"),
                include_str!("../../tests/upstream/test262/harness/compareArray.js"),
            ] {
                runtime.execute(harness, &mut document).unwrap();
            }
            if strict {
                runtime.execute_strict(source, &mut document)
            } else {
                runtime.execute(source, &mut document)
            }
            .unwrap_or_else(|e| panic!("strict={strict}: {e}"));
            assert_eq!(
                (
                    runtime.calls,
                    runtime.stack_units,
                    runtime.eval_depth,
                    runtime.frames.len()
                ),
                (0, 0, 0, 0)
            );
        }
    }

    #[test]
    fn identity_registry_descriptions_boxing_and_intrinsic_descriptors() {
        check(
            r#"
            var a=Symbol('same'),b=Symbol('same');
            assert.sameValue(typeof a,'symbol');assert.notSameValue(a,b);assert.sameValue(a,a);
            assert.sameValue(a.description,'same');assert.sameValue(Symbol().description,undefined);
            assert.sameValue(Symbol(undefined).description,undefined);assert.sameValue(Symbol('').description,'');
            assert.sameValue(Symbol(null).description,'null');assert.sameValue(Symbol('\uD800').description,'\uD800');
            assert.sameValue(String(a),'Symbol(same)');assert.sameValue(Symbol().toString(),'Symbol()');
            assert.sameValue(Symbol.for('x'),Symbol.for({toString(){return 'x';}}));
            assert.notSameValue(Symbol.for('same'),a);assert.sameValue(Symbol.keyFor(a),undefined);
            assert.sameValue(Symbol.keyFor(Symbol.for('\uD800')),'\uD800');
            assert.notSameValue(Symbol.for('\uD800'),Symbol.for('\uFFFD'));
            assert.sameValue(Symbol.keyFor(Symbol.for()),'undefined');
            assert.throws(TypeError,()=>Symbol.keyFor('x'));assert.throws(TypeError,()=>Symbol.keyFor(Object(a)));
            var boxed=Object(a);assert.notSameValue(boxed,a);assert.sameValue(typeof boxed,'object');
            assert.sameValue(boxed.valueOf(),a);assert.sameValue(boxed.description,'same');assert.sameValue(boxed==a,true);
            assert.sameValue(Object.getPrototypeOf(boxed),Symbol.prototype);assert.sameValue(boxed instanceof Symbol,true);
            assert.throws(TypeError,()=>new Symbol('x'));assert.throws(TypeError,()=>Symbol(a));
            assert.throws(TypeError,()=>Symbol.for(a));assert.throws(TypeError,()=>Symbol.prototype.valueOf());
            assert.throws(TypeError,()=>Symbol.prototype.toString.call({}));
            var getter=Object.getOwnPropertyDescriptor(Symbol.prototype,'description').get;
            assert.sameValue(getter.call(a),'same');assert.throws(TypeError,()=>getter.call(Symbol.prototype));
            assert.sameValue(Symbol.length,0);assert.sameValue(Symbol.name,'Symbol');
            var d=Object.getOwnPropertyDescriptor(Symbol,'iterator');
            assert.sameValue(typeof d.value,'symbol');assert.sameValue(d.writable,false);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
            assert.sameValue(Symbol.iterator.description,'Symbol.iterator');assert.sameValue(Symbol.keyFor(Symbol.iterator),undefined);
            assert.notSameValue(Symbol.iterator,Symbol.for('Symbol.iterator'));
            d=Object.getOwnPropertyDescriptor(Symbol.prototype,Symbol.toPrimitive);
            assert.sameValue(d.writable,false);assert.sameValue(d.configurable,true);assert.sameValue(d.enumerable,false);
            assert.sameValue(d.value.name,'[Symbol.toPrimitive]');assert.sameValue(d.value.length,1);
            assert.sameValue(d.value.call(a,'ignored'),a);
        "#,
        );
    }

    #[test]
    fn property_keys_preserve_identity_descriptors_order_and_receivers() {
        check(
            r#"
            var a=Symbol('x'),b=Symbol('x'),o={};o[a]=1;o['Symbol(x)']=2;o[b]=3;o[2]=4;o.a=5;
            assert.sameValue(o[a],1);assert.sameValue(o[b],3);assert.sameValue(o['Symbol(x)'],2);
            assert.sameValue(a in o,true);assert.sameValue(Symbol('x') in o,false);
            assert.compareArray(Object.keys(o),['2','Symbol(x)','a']);
            assert.compareArray(Object.getOwnPropertyNames(o),['2','Symbol(x)','a']);
            assert.compareArray(Object.getOwnPropertySymbols(o),[a,b]);
            var trace='';for(var k in o){trace+=k+';';}assert.sameValue(trace,'2;Symbol(x);a;');
            assert.sameValue(o.hasOwnProperty(a),true);assert.sameValue(o.propertyIsEnumerable(a),true);
            var d=Object.getOwnPropertyDescriptor(o,a);assert.sameValue(d.value,1);assert.sameValue(d.writable,true);
            assert.sameValue(delete o[a],true);o[a]=6;assert.compareArray(Object.getOwnPropertySymbols(o),[b,a]);
            var p={},seen;Object.defineProperty(p,a,{get(){return this.mark;},set(v){seen=this;this.mark=v;},configurable:true});
            var child=Object.create(p);child[a]=9;assert.sameValue(seen,child);assert.sameValue(child[a],9);
            assert.sameValue(child.hasOwnProperty(a),false);assert.sameValue(a in child,true);
            Object.defineProperties(o,{[a]:{value:7,writable:false,enumerable:false,configurable:false}});
            assert.sameValue(o[a],7);assert.sameValue(o.propertyIsEnumerable(a),false);
            assert.throws(TypeError,function(){'use strict';o[a]=8;});
            assert.throws(TypeError,function(){'use strict';delete o[a];});
            assert.throws(TypeError,()=>Object.defineProperty(o,a,{value:8}));
            var frozen=Object.preventExtensions({});assert.throws(TypeError,function(){'use strict';frozen[b]=1;});
            var array=[];array[b]=4;assert.sameValue(array.length,0);assert.sameValue(array[b],4);
            assert.compareArray(Object.getOwnPropertySymbols(array),[b]);
            function f(){}f[b]=8;assert.sameValue(f[b],8);
            window[b]=11;assert.sameValue(window[b],11);assert.sameValue(Object.getOwnPropertyDescriptor(window,b).value,11);
            assert.sameValue(delete window[b],true);assert.sameValue(window[b],undefined);
        "#,
        );
    }

    #[test]
    fn primitive_hooks_symbol_conversions_and_deferred_reference_keys() {
        check(
            r#"
            var s=Symbol('x'),log='';var o={[Symbol.toPrimitive](hint){log+=hint+';';return 1;},valueOf(){throw 1;},toString(){throw 2;}};
            assert.sameValue(Number(o),1);assert.sameValue(o+2,3);assert.sameValue(o<2,true);
            assert.sameValue(o==1,true);assert.sameValue(String(o),'1');assert.sameValue(`${o}`,'1');
            var target={1:4};assert.sameValue(target[o],4);
            assert.sameValue(log,'number;default;number;default;string;string;string;');
            var sym={[Symbol.toPrimitive](){return s;}};target[s]=7;assert.sameValue(target[sym],7);
            assert.sameValue(sym==s,true);assert.sameValue(sym===s,false);
            assert.throws(TypeError,()=>Number(s));assert.throws(TypeError,()=>+s);assert.throws(TypeError,()=>s+1);
            assert.throws(TypeError,()=>s+'');assert.throws(TypeError,()=>s<1);assert.throws(TypeError,()=>`${s}`);
            assert.throws(TypeError,()=>String(Object(s)));assert.throws(TypeError,()=>new String(s));
            assert.throws(TypeError,()=>String(sym));assert.throws(TypeError,()=>Symbol(sym));
            assert.throws(TypeError,()=>Number({[Symbol.toPrimitive]:1}));
            assert.throws(TypeError,()=>String({[Symbol.toPrimitive](){return {};}}));
            assert.sameValue(Number({[Symbol.toPrimitive]:null,valueOf(){return 3;}}),3);
            var order='',key={[Symbol.toPrimitive](hint){order+='k';assert.sameValue(hint,'string');return s;}};
            target[key]=(order+='r',10);assert.sameValue(order,'rk');assert.sameValue(target[s],10);
            order='';target[key]+=(order+='r',1);assert.sameValue(order,'kr');assert.sameValue(target[s],11);
            order='';try{null[key]=(order+='r',1);}catch(e){assert.sameValue(e instanceof TypeError,true);order+='e';}
            assert.sameValue(order,'re');
        "#,
        );
    }

    #[test]
    fn computed_literal_names_and_mutable_to_string_tags() {
        check(
            r#"
            var s=Symbol('method'),empty=Symbol(''),absent=Symbol();
            var o={[s](){return this;},[empty]:function(){},[absent]:()=>1};
            assert.sameValue(o[s](),o);assert.sameValue(o[s].name,'[method]');
            assert.sameValue(o[empty].name,'[]');assert.sameValue(o[absent].name,'');
            var p={get [s](){return 1;},set [s](v){}};
            var d=Object.getOwnPropertyDescriptor(p,s);assert.sameValue(d.get.name,'get [method]');assert.sameValue(d.set.name,'set [method]');
            var tag=Symbol.toStringTag,call=Object.prototype.toString;
            assert.sameValue(call.call(s),'[object Symbol]');assert.sameValue(call.call(Symbol.prototype),'[object Symbol]');
            assert.sameValue(call.call(Math),'[object Math]');assert.sameValue(call.call(JSON),'[object JSON]');
            assert.sameValue(call.call({[tag]:'Custom'}),'[object Custom]');
            assert.sameValue(call.call({[tag]:'\uD800'}),'[object \uD800]');
            var proto={[tag]:'Inherited'};assert.sameValue(call.call(Object.create(proto)),'[object Inherited]');
            var hits=0;assert.sameValue(call.call({get [tag](){hits++;return 4;}}),'[object Object]');assert.sameValue(hits,1);
            var reason={};try{call.call({get [tag](){throw reason;}});throw 0;}catch(e){assert.sameValue(e,reason);}
            assert.sameValue(delete Math[tag],true);assert.sameValue(call.call(Math),'[object Object]');
            Object.defineProperty(JSON,tag,{value:'Changed'});assert.sameValue(call.call(JSON),'[object Changed]');
        "#,
        );
    }

    #[test]
    fn json_omits_symbol_keys_and_values_but_observes_replacer_results() {
        check(
            r#"
            var s=Symbol('secret'),hits=0;var o={a:1,b:s,get [s](){hits++;throw 1;}};
            assert.sameValue(JSON.stringify(s),undefined);assert.sameValue(JSON.stringify(o),'{"a":1}');
            assert.sameValue(hits,0);assert.sameValue(JSON.stringify([s,1]),'[null,1]');
            assert.sameValue(JSON.stringify(Object(s)),'{}');
            assert.sameValue(JSON.stringify({x:s},function(key,v){return v===s?'kept':v;}),'{"x":"kept"}');
            assert.sameValue(JSON.stringify({x:1},[s,'x']),'{"x":1}');
            assert.sameValue(JSON.stringify({toJSON(){return s;}}),undefined);
            assert.throws(TypeError,()=>JSON.parse(s));
            assert.sameValue(JSON.parse({[Symbol.toPrimitive](hint){assert.sameValue(hint,'string');return '3';}}),3);
        "#,
        );
    }
    #[test]
    fn reflection_and_instance_hooks_preserve_order_and_bound_target_delegation() {
        check(
            r#"
            var a=Symbol('a'),b=Symbol('b'),o={z:1};o[a]=2;o[2]=3;o.x=4;o[1]=5;o[b]=6;
            assert.compareArray(Reflect.ownKeys(o),['1','2','z','x',a,b]);
            assert.compareArray(Reflect.ownKeys(Object('xy')),['0','1','length']);
            assert.compareArray(Reflect.ownKeys([1]),['0','length']);
            assert.throws(TypeError,function(){Reflect.ownKeys('x');});
            assert.throws(TypeError,function(){Reflect.ownKeys(Symbol());});
            assert.compareArray(Object.getOwnPropertySymbols('x'),[]);
            assert.sameValue(Reflect.ownKeys.length,1);assert.sameValue(Reflect.ownKeys.name,'ownKeys');
            assert.sameValue(Object.prototype.toString.call(Reflect),'[object Reflect]');
            var ordinary=Function.prototype[Symbol.hasInstance];
            assert.sameValue(ordinary.name,'[Symbol.hasInstance]');assert.sameValue(ordinary.length,1);
            var d=Object.getOwnPropertyDescriptor(Function.prototype,Symbol.hasInstance);
            assert.sameValue(d.writable,false);assert.sameValue(d.enumerable,false);assert.sameValue(d.configurable,false);
            assert.sameValue(ordinary.call({},1),false);assert.sameValue(ordinary.call(Symbol(),{}),false);
            var called=0, target={};target[Symbol.hasInstance]=function(v){assert.sameValue(this,target);called++;return v===3?'yes':0;};
            assert.sameValue(3 instanceof target,true);assert.sameValue(4 instanceof target,false);assert.sameValue(called,2);
            target[Symbol.hasInstance]=1;assert.throws(TypeError,function(){return 3 instanceof target;});
            target[Symbol.hasInstance]=null;assert.throws(TypeError,function(){return 3 instanceof target;});
            assert.throws(TypeError,function(){return 3 instanceof 3;});
            function F(){}var value=new F();var bound=F.bind(null);assert.sameValue(value instanceof bound,true);
            Object.defineProperty(F,Symbol.hasInstance,{value:function(v){return v===7;},configurable:true});
            assert.sameValue(7 instanceof bound,true);assert.sameValue(value instanceof bound,false);
            assert.sameValue(ordinary.call(bound,7),true);
            assert.sameValue(ordinary.call(F,value),true);assert.sameValue(ordinary.call(F,7),false);
            delete F[Symbol.hasInstance];assert.sameValue(value instanceof bound,true);
            F.prototype=3;assert.sameValue(ordinary.call(F,1),false);assert.throws(TypeError,function(){ordinary.call(F,{});});
            var thrown={};Object.defineProperty(target,Symbol.hasInstance,{get:function(){throw thrown;}});
            try{3 instanceof target;assert(false);}catch(e){assert.sameValue(e,thrown);}
        "#,
        );
    }

    #[test]
    fn numeric_consumers_reject_symbols_and_observe_conversion_mutations() {
        check(
            r#"
            var s=Symbol();
            assert.throws(TypeError,function(){Math.abs(s);});assert.throws(TypeError,function(){Math.pow(1,s);});
            assert.throws(TypeError,function(){Math.min(NaN,s);});assert.throws(TypeError,function(){Math.max(NaN,s);});
            assert.throws(TypeError,function(){(function(){}).apply(null,{length:s});});
            assert.throws(TypeError,function(){[1].slice(s);});assert.throws(TypeError,function(){[1].slice(0,s);});
            assert.throws(TypeError,function(){[1].join(s);});assert.throws(TypeError,function(){var a=[];a.length=s;});
            assert.sameValue(Math.min(0,-0),-0);assert.sameValue(Math.max(-0,0),0);
            assert.sameValue(Math.abs(-3,s),3);assert.sameValue(Math.round(-0.1),-0);assert.sameValue(Math.round(0),0);
            var order='',a={};a[Symbol.toPrimitive]=function(h){order+=h;return 2;};
            assert.sameValue(Math.pow(a,a),4);assert.sameValue(order,'numbernumber');
            order='';assert.sameValue((function(x,y){return x+y;}).apply(null,{0:3,1:4,length:a}),7);assert.sameValue(order,'number');
            var array=[1,2,3],index={};index[Symbol.toPrimitive]=function(h){assert.sameValue(h,'number');array.length=0;return 0;};
            var copy=array.slice(index);assert.sameValue(copy.length,3);assert.sameValue(0 in copy,false);assert.sameValue(2 in copy,false);
            array=[1,2];var separator={};separator[Symbol.toPrimitive]=function(h){assert.sameValue(h,'string');array.length=0;return '-';};
            assert.sameValue(array.join(separator),'-');
        "#,
        );
    }
    #[test]
    fn removed_primitive_hook_uses_mutable_ordinary_conversion_methods() {
        check(
            r#"
            delete Symbol.prototype[Symbol.toPrimitive];
            var gets=0,calls=0;
            Object.defineProperty(Symbol.prototype,'valueOf',{get:function(){gets++;return function(){calls++;return 123;};},configurable:true});
            assert.sameValue(Object(Symbol())==123,true);assert.sameValue(+Object(Symbol()),123);
            assert.sameValue(String(Object(Symbol())),'Symbol()');assert.sameValue(gets,2);assert.sameValue(calls,2);
            Object.defineProperty(Symbol.prototype,'toString',{value:function(){return 'foo';},configurable:true});
            assert.sameValue(''+Object(Symbol()),'123');assert.sameValue({foo:7}[Object(Symbol())],7);
            Object.defineProperty(Symbol.prototype,'valueOf',{value:null,configurable:true});
            assert.sameValue(Number(Object(Symbol())),NaN);assert.sameValue(String(Object(Symbol())),'foo');
            Object.defineProperty(Symbol.prototype,'toString',{value:undefined});
            assert.throws(TypeError,function(){Number(Object(Symbol()));});
            assert.throws(TypeError,function(){String(Object(Symbol()));});
        "#,
        );
    }

    #[test]
    fn symbol_allocations_precede_mutation_and_callbacks_cannot_catch_limits() {
        let mut runtime = Runtime::new();
        let mut document = Document::parse("");
        runtime
            .execute("var o={};var key=Symbol('key');", &mut document)
            .unwrap();
        let object = runtime.environments[0].bindings["o"].value.clone();
        let Value::Symbol(symbol) = runtime.environments[0].bindings["key"].value.clone() else {
            panic!()
        };
        let key = PropertyKey::Symbol(symbol);
        let registry_len = runtime.symbols.registry.len();
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .symbol_native(
                    "Symbol.for",
                    Value::Undefined,
                    &[Value::String("new-key".into())],
                    &mut document
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.symbols.registry.len(), registry_len);
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .define_own_key(
                    &object,
                    &key,
                    PropertyDescriptor::data_property(Value::Number(1.0), true, true, true)
                )
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.own_property_key(&object, &key).is_none());
        runtime.allocated = MAX_HEAP;
        let too_long = Symbol::unique(Some(JsString::from(vec![97; MAX_STRING])));
        assert!(
            runtime
                .symbol_text(&too_long)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, MAX_HEAP);
        for source in [
            "var caught=false,o={};o[Symbol.toPrimitive]=function(){while(true){}};try{+o;}catch(e){caught=true;}",
            "var caught=false,o={};o[Symbol.toPrimitive]=function(){return +o;};try{+o;}catch(e){caught=true;}",
            "var caught=false,o={};o[Symbol.hasInstance]=function(v){return v instanceof o;};try{1 instanceof o;}catch(e){caught=true;}",
            "var caught=false;try{var i=0;while(true){Symbol.for('key'+i++);}}catch(e){caught=true;}",
        ] {
            let mut runtime = Runtime::new();
            assert!(
                runtime
                    .execute(source, &mut document)
                    .unwrap_err()
                    .is_resource_limit(),
                "{source}"
            );
            assert_eq!(
                runtime.environments[0].bindings["caught"].value,
                Value::Bool(false)
            );
            assert_eq!(
                (
                    runtime.calls,
                    runtime.stack_units,
                    runtime.eval_depth,
                    runtime.frames.len()
                ),
                (0, 0, 0, 0)
            );
        }
    }
    #[test]
    fn display_conversion_charges_symbol_descriptions_before_formatting() {
        let mut runtime = Runtime::new();
        let value = Value::Symbol(Symbol::unique(Some(JsString::from(vec![0xd800; 8192]))));
        runtime.allocated = MAX_HEAP;
        assert!(
            runtime
                .display_value(&value)
                .unwrap_err()
                .is_resource_limit()
        );
        assert!(runtime.allocated > MAX_HEAP);
        runtime.allocated = 0;
        runtime.steps = 0;
        assert!(
            runtime
                .display_value(&value)
                .unwrap_err()
                .is_resource_limit()
        );
        assert_eq!(runtime.allocated, 0);
    }
}
