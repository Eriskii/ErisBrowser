//! Ordinary connection state from the shared paid root walk. Template hosts
//! are not parent links. Getter traversal owns no storage and invokes no hooks.
use super::*;

#[cfg(test)]
mod tests;

pub(super) const METADATA_OBJECTS: usize = 1;

impl Runtime {
    pub(super) fn install_node_connected(
        &mut self,
        prototype: usize,
        metadata_keys: [&PropertyKey; 2],
    ) -> Result<()> {
        // Transfer the existing immutable length/name keys before metadata work.
        self.work(8)?;
        self.work(4)?;
        let get = self.node_data_function_with_keys(
            "getIsConnected",
            "get isConnected",
            0,
            metadata_keys,
        )?;
        self.dom_proto_named(
            prototype,
            "isConnected",
            Property {
                value: PropertyValue::Accessor {
                    get,
                    set: Value::Undefined,
                },
                enumerable: true,
                configurable: true,
            },
        )
    }

    pub(super) fn node_is_connected(&mut self, receiver: Value, doc: &Document) -> Result<Value> {
        self.work(16)?;
        let id = node_predicates::node_id(&receiver, doc)?;
        let root = self.node_ordinary_root_from_id(id, doc)?;
        self.work(2)?;
        Ok(Value::Bool(matches!(root, Value::Document)))
    }
}
