//! Checked ProcessingInstruction construction and represented CharacterData.
//! CharacterData stores exact DOMString units. Pseudo-attribute methods and
//! mutation observers/ranges remain separate gaps.
use super::*;
use crate::dom::DomDataError;

mod character_data;
#[cfg(test)]
mod tests;
mod xml_name;

pub(super) const PREFIX: &str = "DOM.PI.";
pub(super) const METADATA_OBJECTS: usize = 10;

pub(super) fn dom_data_error(error: DomDataError) -> ScriptError {
    match error {
        DomDataError::InvalidNode => ScriptError::type_error("invalid CharacterData receiver"),
        DomDataError::InvalidData => {
            ScriptError::unsupported("nonscalar DOM data is not supported by this script operation")
        }
        DomDataError::LimitExceeded => ScriptError::resource("DOM character data limit exceeded"),
        DomDataError::AllocationFailed => {
            ScriptError::resource("DOM character data allocation failed")
        }
    }
}

impl Runtime {
    fn pi_function(&mut self, suffix: &str, display: &str, length: usize) -> Result<Value> {
        let properties =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        let name = self.dom_proto_text(display)?;
        self.dom_proto_named(
            properties,
            "length",
            Property::data(Value::Number(length as f64), false, false, true),
        )?;
        self.dom_proto_named(
            properties,
            "name",
            Property::data(Value::String(name), false, false, true),
        )?;
        let length = PREFIX.len() + suffix.len();
        self.work(8 + length)?;
        self.charge(std::mem::size_of::<Native>() + 32 + length)?;
        let mut name = String::new();
        name.try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("PI native name allocation failed"))?;
        name.push_str(PREFIX);
        name.push_str(suffix);
        let properties = std::num::NonZeroUsize::new(properties)
            .ok_or_else(|| ScriptError::resource("PI metadata ID is zero"))?;
        Ok(Value::Native(Rc::new(Native {
            properties: Some(properties),
            name,
            receiver: Value::Undefined,
        })))
    }

    pub(super) fn install_pi_members(&mut self, interface: &str, prototype: usize) -> Result<()> {
        self.work(4)?;
        match interface {
            "Document" => {
                let function = self.pi_function("create", "createProcessingInstruction", 2)?;
                self.dom_proto_named(
                    prototype,
                    "createProcessingInstruction",
                    Property::data(function, true, true, true),
                )?;
            }
            "ProcessingInstruction" => {
                let get = self.pi_function("target", "get target", 0)?;
                self.dom_proto_named(
                    prototype,
                    "target",
                    Property {
                        value: PropertyValue::Accessor {
                            get,
                            set: Value::Undefined,
                        },
                        enumerable: true,
                        configurable: true,
                    },
                )?;
            }
            "CharacterData" => {
                let get = self.pi_function("data", "get data", 0)?;
                let set = self.pi_function("setData", "set data", 1)?;
                self.dom_proto_named(
                    prototype,
                    "data",
                    Property {
                        value: PropertyValue::Accessor { get, set },
                        enumerable: true,
                        configurable: true,
                    },
                )?;
                let get = self.pi_function("length", "get length", 0)?;
                self.dom_proto_named(
                    prototype,
                    "length",
                    Property {
                        value: PropertyValue::Accessor {
                            get,
                            set: Value::Undefined,
                        },
                        enumerable: true,
                        configurable: true,
                    },
                )?;
                self.work(character_data::METHODS.len())?;
                for &(name, length) in character_data::METHODS {
                    let function = self.pi_function(name, name, length)?;
                    self.dom_proto_named(
                        prototype,
                        name,
                        Property::data(function, true, true, true),
                    )?;
                }
            }
            "Text" => self.install_text_operations(prototype)?,
            _ => {}
        }
        Ok(())
    }

    pub(super) fn pi_call_preflight(&mut self, name: &str) -> Result<()> {
        // Reached temporary Native name copy in the invocation bridge; keep
        // this refusal inside its cleanup-preserving result expression.
        self.work(4 + name.len())?;
        self.charge(32 + name.len())
    }

    pub(super) fn pi_construct(
        &mut self,
        args: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        self.tick()?;
        let Some(target) = args.first() else {
            return Err(ScriptError::type_error(
                "ProcessingInstruction requires a target",
            ));
        };
        let target = self.string_hint(target.clone(), doc)?;
        let data = match args.get(1) {
            None | Some(Value::Undefined) => JsString::default(),
            Some(value) => self.string_hint(value.clone(), doc)?,
        };
        let prototype = self.dom_constructor_override("ProcessingInstruction", new_target, doc)?;
        self.pi_initialize(target, data, prototype, doc)
    }

    fn pi_initialize(
        &mut self,
        target: JsString,
        data: JsString,
        prototype: Option<Value>,
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(
            xml_name::xml_name_work(target.len())
                .ok_or_else(|| ScriptError::resource("XML Name work overflow"))?,
        )?;
        if !xml_name::xml_name_utf16(target.units()) {
            return Err(self.pi_invalid_character("invalid processing instruction target")?);
        }
        self.work(1 + 2 * data.len())?;
        if data
            .units()
            .windows(2)
            .any(|pair| pair == [u16::from(b'?'), u16::from(b'>')])
        {
            return Err(self.pi_invalid_character("processing instruction data contains ?>")?);
        }
        let target = self.pi_scalar_string(&target)?;
        let plan = self.plan_dom_data(data.units().iter().copied(), data.len(), data.len())?;
        let bytes = target
            .len()
            .checked_add(plan.stored_bytes())
            .ok_or_else(|| ScriptError::resource("PI text length overflow"))?;
        // Conversions and NewTarget callbacks have finished. Fresh admission
        // precedes every node/override publication; strings are moved, not copied.
        if !doc.admits_text_node(bytes) {
            return Err(ScriptError::resource("PI node or text limit exceeded"));
        }
        let data = self.emit_dom_data(plan)?;
        self.ensure_dom_capacity(doc, 1)?;
        self.dom_admit_override(&prototype)?;
        self.work(8)?;
        if doc.nodes.len() == doc.nodes.capacity() {
            self.work(1 + 2 * doc.nodes.len())?;
            self.charge(
                (doc.nodes.len() + 1)
                    .checked_mul(std::mem::size_of::<crate::dom::Node>())
                    .ok_or_else(|| ScriptError::resource("PI node storage overflow"))?,
            )?;
        }
        let id = doc
            .create_processing_instruction_owned(target, data)
            .map_err(dom_data_error)?;
        self.dom_publish_override(id, prototype);
        Ok(Value::Node(id))
    }

    fn pi_invalid_character(&mut self, message: &str) -> Result<ScriptError> {
        let name = self.dom_proto_text("InvalidCharacterError")?;
        let message = self.dom_proto_text(message)?;
        let value = self.dom_exception(name, message)?;
        self.thrown_error(value)
    }

    fn pi_scalar_string(&mut self, text: &JsString) -> Result<String> {
        self.work(1 + text.len())?;
        let mut bytes = 0usize;
        for scalar in char::decode_utf16(text.units().iter().copied()) {
            bytes += scalar
                .map_err(|_| {
                    ScriptError::unsupported(
                        "unpaired surrogate DOMString storage is not implemented",
                    )
                })?
                .len_utf8();
        }
        self.work(1 + text.len())?;
        self.charge(bytes)?;
        let mut result = String::new();
        result
            .try_reserve_exact(bytes)
            .map_err(|_| ScriptError::resource("PI text allocation failed"))?;
        for scalar in char::decode_utf16(text.units().iter().copied()) {
            result.push(scalar.unwrap());
        }
        Ok(result)
    }

    fn pi_dom_string(&mut self, text: &str) -> Result<Value> {
        // Scalar scan, encoding and Rc copy, all before the bounded allocation.
        self.work(1 + 3 * text.len())?;
        let length = text.encode_utf16().count();
        if length > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        self.charge(64 + 4 * length)?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("DOM getter string allocation failed"))?;
        units.extend(text.encode_utf16());
        Ok(Value::String(units.into()))
    }

    fn pi_dom_data(&mut self, text: &crate::dom::DomString) -> Result<Value> {
        // Preserve exact document units; scalar payloads retain the prior
        // UTF8 scan/encoding/copy admission, without a projection temporary.
        self.work(1 + 3 * text.stored_bytes())?;
        let length = text.units().count();
        if length > MAX_STRING {
            return Err(ScriptError::resource("script string limit exceeded"));
        }
        self.charge(64 + 4 * length)?;
        let mut units = Vec::new();
        units
            .try_reserve_exact(length)
            .map_err(|_| ScriptError::resource("DOM getter string allocation failed"))?;
        units.extend(text.units());
        Ok(Value::String(units.into()))
    }

    pub(super) fn pi_native(
        &mut self,
        method: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(8 + method.len())?;
        if method.ends_with("Data") && method != "setData" {
            return self.character_data_native(method, receiver, args, doc);
        }
        if method == "create" {
            if receiver != Value::Document {
                return Err(ScriptError::type_error(
                    "createProcessingInstruction requires Document",
                ));
            }
            if args.len() < 2 {
                return Err(ScriptError::type_error(
                    "createProcessingInstruction requires target and data",
                ));
            }
            let target = self.string_hint(args[0].clone(), doc)?;
            let data = self.string_hint(args[1].clone(), doc)?;
            return self.pi_initialize(target, data, None, doc);
        }
        let Value::Node(id) = receiver else {
            return Err(ScriptError::type_error(
                "CharacterData accessor requires a genuine node",
            ));
        };
        let node = doc
            .nodes
            .get(id)
            .ok_or_else(|| ScriptError::type_error("invalid CharacterData node"))?;
        let data = match &node.kind {
            NodeKind::ProcessingInstruction { target, .. } if method == "target" => {
                return self.pi_dom_string(target);
            }
            NodeKind::Text(text)
            | NodeKind::Comment(text)
            | NodeKind::ProcessingInstruction { data: text, .. }
                if method != "target" =>
            {
                text
            }
            _ => {
                return Err(ScriptError::type_error(
                    "incompatible CharacterData receiver",
                ));
            }
        };
        match method {
            "data" => self.pi_dom_data(data),
            "length" => {
                self.work(1 + data.stored_bytes())?;
                Ok(Value::Number(data.units().count() as f64))
            }
            "setData" => {
                let value = args.first().cloned().unwrap_or(Value::Undefined);
                let text = if matches!(value, Value::Null) {
                    JsString::default()
                } else {
                    self.string_hint(value, doc)?
                };
                let plan =
                    self.plan_dom_data(text.units().iter().copied(), text.len(), text.len())?;
                self.work(8)?;
                doc.check_character_data_replacement(id, plan.stored_bytes())
                    .map_err(dom_data_error)?;
                let text = self.emit_dom_data(plan)?;
                self.work(8)?;
                doc.replace_character_data(id, text)
                    .map_err(dom_data_error)?;
                Ok(Value::Undefined)
            }
            _ => Err(ScriptError::type_error("unknown PI operation")),
        }
    }
}
