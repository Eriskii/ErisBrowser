//! Script-facing append conversion and pre-insertion checks.
use super::*;

impl Runtime {
    pub(super) fn dom_append(
        &mut self,
        parent: NodeId,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<()> {
        // Web IDL converts every union argument before the DOM algorithm starts.
        // Author conversion can mutate the destination; no tree admission is
        // cached across it.
        self.work(args.len() + 1)?;
        self.charge(
            args.len()
                .saturating_mul(std::mem::size_of::<AppendValue>())
                + 32,
        )?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(args.len())
            .map_err(|_| ScriptError::resource("DOM argument allocation failed"))?;
        for value in args {
            values.push(match value {
                Value::Node(child) => AppendValue::Node(*child),
                Value::Document => AppendValue::Node(doc.root),
                _ => AppendValue::Text(self.dom_string(value.clone(), doc)?),
            });
        }

        // Materialize every string before moving any node. A quota refusal
        // leaves previously created text detached and all argument nodes in
        // their original locations (apart from author conversion effects).
        for value in &mut values {
            self.tick()?;
            if let AppendValue::Text(text) = value {
                self.ensure_dom_capacity(doc, 1)?;
                if !doc.admits_text_node(text.len()) {
                    return Err(ScriptError::resource("DOM text storage limit exceeded"));
                }
                self.work(1 + text.len())?;
                self.charge(text.len())?;
                *value = AppendValue::Node(doc.create_text_node(text));
            }
        }

        let child = if values.len() == 1 {
            let AppendValue::Node(child) = values[0] else {
                unreachable!()
            };
            child
        } else {
            self.ensure_dom_capacity(doc, 1)?;
            self.tick()?;
            let fragment = doc.create_document_fragment();
            for value in values {
                let AppendValue::Node(child) = value else {
                    unreachable!()
                };
                // Preserve the argument sequence, including duplicates.
                // Earlier moves survive a later assembly or final failure.
                self.dom_checked_append(fragment, child, doc)?;
            }
            fragment
        };
        self.dom_checked_append(parent, child, doc)
    }

    // DOM pre-insert validity for append: reference child is null and the
    // exclusion list is empty. Parser/internal insertion retains its own path.
    pub(super) fn dom_checked_append(
        &mut self,
        parent: NodeId,
        child: NodeId,
        doc: &mut Document,
    ) -> Result<()> {
        self.work(4)?;
        if parent >= doc.nodes.len() || child >= doc.nodes.len() {
            return Err(ScriptError::resource("invalid DOM node reference"));
        }
        if !matches!(
            doc.nodes[parent].kind,
            NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
        ) {
            return Err(self.dom_throw("HierarchyRequestError", "node cannot have children")?);
        }
        let mut ancestor = Some(parent);
        let mut depth = 0;
        while let Some(id) = ancestor {
            self.tick()?;
            if id == child {
                return Err(
                    self.dom_throw("HierarchyRequestError", "insertion would create a cycle")?
                );
            }
            if depth >= crate::dom::MAX_DEPTH {
                return Err(ScriptError::resource("DOM insertion depth limit exceeded"));
            }
            depth += 1;
            let node = &doc.nodes[id];
            ancestor = node.parent.or(match node.kind {
                NodeKind::DocumentFragment { host } => host,
                _ => None,
            });
        }
        if matches!(doc.nodes[child].kind, NodeKind::Document) {
            return Err(self.dom_throw("HierarchyRequestError", "cannot insert a Document")?);
        }
        let destination_is_document = matches!(doc.nodes[parent].kind, NodeKind::Document);
        let is_doctype = matches!(doc.nodes[child].kind, NodeKind::Doctype(_));
        if is_doctype && !destination_is_document {
            return Err(self.dom_throw("HierarchyRequestError", "doctype requires a Document")?);
        }
        if destination_is_document {
            let mut elements = usize::from(matches!(doc.nodes[child].kind, NodeKind::Element(_)));
            if matches!(doc.nodes[child].kind, NodeKind::Text(_)) {
                return Err(
                    self.dom_throw("HierarchyRequestError", "Document cannot contain text")?
                );
            }
            if matches!(doc.nodes[child].kind, NodeKind::DocumentFragment { .. }) {
                for &id in &doc.nodes[child].children {
                    self.tick()?;
                    match doc.nodes[id].kind {
                        NodeKind::Text(_) => {
                            return Err(self.dom_throw(
                                "HierarchyRequestError",
                                "Document cannot contain text",
                            )?);
                        }
                        NodeKind::Element(_) => elements += 1,
                        _ => {}
                    }
                    if elements > 1 {
                        return Err(self.dom_throw(
                            "HierarchyRequestError",
                            "Document already has an element",
                        )?);
                    }
                }
            }
            if elements != 0 || is_doctype {
                for &id in &doc.nodes[parent].children {
                    self.tick()?;
                    let invalid = match doc.nodes[id].kind {
                        NodeKind::Element(_) => true,
                        NodeKind::Doctype(_) => is_doctype,
                        _ => false,
                    };
                    if invalid {
                        return Err(self
                            .dom_throw("HierarchyRequestError", "invalid Document child order")?);
                    }
                }
            }
        }
        // The paid raw-mutator preflight checks the unchanged implementation
        // depth bounds as well as mutation work/storage. No callback or DOM
        // change occurs between this admission and the actual mutation.
        if !self.charge_dom_append_admission(parent, child, doc)? {
            return Err(ScriptError::resource("DOM insertion depth limit exceeded"));
        }
        doc.append_child(parent, child);
        Ok(())
    }
}
