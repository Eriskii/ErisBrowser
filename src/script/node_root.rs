//! Ordinary roots for represented nodes after observable dictionary conversion.
//! Template hosts are not ShadowRoots. Traversal owns no storage; object options
//! separately allocate a paid literal key and retain ordinary Get/VM charges.
use super::*;
use node_predicates::{local_node, node_id};

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 1;

fn invalid() -> ScriptError {
    ScriptError::type_error("invalid Node root tree")
}

impl Runtime {
    pub(super) fn install_node_root(&mut self, prototype: usize) -> Result<()> {
        self.work(4)?;
        let method = self.node_data_function("getRootNode", "getRootNode", 0)?;
        self.dom_proto_named(
            prototype,
            "getRootNode",
            Property::data(method, true, true, true),
        )
    }

    fn node_root_options(&mut self, args: &[Value], doc: &mut Document) -> Result<bool> {
        self.work(8)?;
        let Some(options) = args
            .first()
            .filter(|value| !matches!(value, Value::Null | Value::Undefined))
        else {
            self.work(2)?;
            return Ok(false);
        };
        if !js_object(options) {
            return Err(ScriptError::type_error(
                "getRootNode options must be an object or nullish",
            ));
        }
        // This literal has no legacy virtual host fallback. Event.composed is
        // an ordinary accessor; Style does not recognize composed as CSS.
        // One exact-reserve Vec and Rc key, then one ordinary inherited Get
        // with the original options receiver. No key cache or absent retry.
        let key = self.dom_proto_text("composed")?;
        let value = self
            .lookup_property(options, &key, doc)?
            .unwrap_or(Value::Undefined);
        self.work(2)?;
        // Undefined selects the default false; all other values use the same
        // noncoercive truth test. No authored conversion hook is requested.
        Ok(value.truthy())
    }

    pub(super) fn node_get_root_node(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        self.work(16)?;
        let mut current = node_id(&receiver, doc)?;
        let _composed = self.node_root_options(args, doc)?;
        // No ShadowRoot kind is represented, so both composed values use the
        // ordinary root. Never follow a template fragment's host. Revisit this
        // branch when an actual ShadowRoot representation is introduced.
        // Only the authentic ID survives the Get; all links below are fresh.
        let mut edges = 0;
        loop {
            self.work(12)?;
            let node = local_node(doc, current)?;
            let Some(parent) = node.parent else {
                self.work(4)?;
                return if current == doc.root {
                    if matches!(node.kind, NodeKind::Document) {
                        Ok(Value::Document)
                    } else {
                        Err(invalid())
                    }
                } else {
                    Ok(Value::Node(current))
                };
            };
            if edges == crate::dom::MAX_DEPTH || parent == current {
                return Err(invalid());
            }
            self.work(12)?;
            let parent_node = local_node(doc, parent)?;
            if !matches!(
                parent_node.kind,
                NodeKind::Document | NodeKind::Element(_) | NodeKind::DocumentFragment { .. }
            ) {
                return Err(invalid());
            }
            let scan = parent_node
                .children
                .len()
                .checked_mul(4)
                .and_then(|count| count.checked_add(1))
                .ok_or_else(|| ScriptError::resource("Node root scan overflow"))?;
            self.work(scan)?;
            let mut found = false;
            for &child in &parent_node.children {
                if child == current {
                    if found {
                        return Err(invalid());
                    }
                    found = true;
                }
            }
            if !found {
                return Err(invalid());
            }
            // The endpoint at exactly MAX_DEPTH edges is still processed.
            edges += 1;
            current = parent;
        }
    }
}
