//! Persistent inline declarations. Attribute text can omit pending semantics.
//! All checked commit failures precede paired attribute/state publication.
use super::mutation_budget::{add, map_nodes, movement, mul, search};
use super::*;
use crate::cssom::{InlineStyle, PreparedInlineStyle};
use std::sync::Arc;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub(super) struct InlineStyleEntry {
    block: Arc<InlineStyle>,
    retained_cost: usize,
    release_work: usize,
}

fn spend(work: &mut usize, cost: usize) -> Result<(), DomDataError> {
    if cost > *work {
        *work = 0;
        return Err(DomDataError::LimitExceeded);
    }
    *work -= cost;
    Ok(())
}

fn lookup_work(count: usize) -> Result<usize, DomDataError> {
    add(8, search(count, 0)?)
}

fn insert_work(count: usize, key_bytes: usize) -> Result<usize, DomDataError> {
    add(
        32,
        add(search(count, key_bytes)?, mul(4, movement(count)?)?)?,
    )
}

impl Document {
    fn is_inline_style_attribute(
        &self,
        node: NodeId,
        name: &str,
        namespace: Option<AttributeNamespace>,
    ) -> bool {
        namespace.is_none()
            && match self.namespace(node) {
                Some(Namespace::Html) => name.eq_ignore_ascii_case("style"),
                Some(_) => name == "style",
                None => false,
            }
    }

    pub(crate) fn inline_style<'a>(
        &'a self,
        node: NodeId,
        work: &mut usize,
    ) -> Result<Option<&'a InlineStyle>, DomDataError> {
        spend(work, lookup_work(self.inline_styles.len())?)?;
        if !matches!(
            self.nodes.get(node).map(|n| &n.kind),
            Some(NodeKind::Element(_))
        ) {
            return Err(DomDataError::InvalidNode);
        }
        Ok(self
            .inline_styles
            .get(&node)
            .map(|entry| entry.block.as_ref()))
    }

    pub(crate) fn prepay_inline_style_invalidation(
        &self,
        node: NodeId,
        name: &str,
        namespace: Option<AttributeNamespace>,
        work: &mut usize,
    ) -> Result<(), DomDataError> {
        if !self.is_inline_style_attribute(node, name, namespace) {
            return Ok(());
        }
        let count = self.inline_styles.len();
        spend(work, lookup_work(count)?)?;
        if let Some(entry) = self.inline_styles.get(&node) {
            // External setters inspect release credit and then remove the entry.
            // Removals use the same conservative two-search/repair allowance.
            let cost = add(
                32,
                add(
                    mul(2, search(count, 0)?)?,
                    add(mul(4, movement(count)?)?, entry.release_work)?,
                )?,
            )?;
            spend(work, cost)?;
        } else {
            // The later external setter still probes the map for release credit.
            spend(work, search(count, 0)?)?;
        }
        Ok(())
    }

    pub(super) fn inline_style_retained_credit(
        &self,
        node: NodeId,
        name: &str,
        namespace: Option<AttributeNamespace>,
    ) -> usize {
        if self.is_inline_style_attribute(node, name, namespace) {
            self.inline_styles
                .get(&node)
                .map_or(0, |entry| entry.retained_cost)
        } else {
            0
        }
    }

    pub(super) fn invalidate_inline_style_after_attribute_write(&mut self, node: NodeId) {
        if let Some(entry) = self.inline_styles.remove(&node) {
            self.inline_style_bytes = self
                .inline_style_bytes
                .checked_sub(entry.retained_cost)
                .expect("inline style total includes every retained entry");
            self.retained_bytes = self
                .retained_bytes
                .checked_sub(entry.retained_cost)
                .expect("DOM retained bytes include typed inline styles");
        }
    }

    pub(crate) fn commit_inline_style_checked(
        &mut self,
        node: NodeId,
        prepared: PreparedInlineStyle,
        budget: &mut DomMutationBudget,
    ) -> Result<(), DomDataError> {
        // Prepared owns its serialized attribute and has already prepaid its
        // own release even on these early errors. No truncating setter is used.
        budget.work(16)?;
        let Some(Node {
            kind: NodeKind::Element(element),
            ..
        }) = self.nodes.get(node)
        else {
            return Err(DomDataError::InvalidNode);
        };
        let count = self.inline_styles.len();
        let attrs = element.attrs.len();
        let namespaces = element.attr_namespaces.len();
        if self.nodes.len() > MAX_NODES
            || self.retained_bytes > MAX_DOM_BYTES
            || self.inline_style_bytes > self.retained_bytes
            || count > MAX_NODES
            || attrs > 1024
            || namespaces > attrs
        {
            return Err(DomDataError::InvalidData);
        }
        budget.work(add(
            search(count, 0)?,
            add(search(attrs, 5)?, search(namespaces, 5)?)?,
        )?)?;
        let old_attribute = element.attrs.get("style");
        if element.attr_namespaces.contains_key("style") {
            return Err(DomDataError::InvalidData);
        }
        if attrs == 1024 && old_attribute.is_none() {
            return Err(DomDataError::LimitExceeded);
        }
        let old_entry = self.inline_styles.get(&node);
        if old_entry.is_some() && old_attribute.is_none() {
            return Err(DomDataError::InvalidData);
        }
        if old_entry.is_none() && count == MAX_NODES {
            return Err(DomDataError::LimitExceeded);
        }
        let old_entry_bytes = old_entry.map_or(0, |entry| entry.retained_cost);
        let old_release = old_entry.map_or(0, |entry| entry.release_work);
        let old_attribute_bytes = old_attribute.map_or(0, |text| text.len() + 5);
        let old_attribute_release = old_attribute.map_or(0, |text| text.len() + 8);
        let entry_bytes = add(
            prepared.style().retained_bytes(),
            map_nodes::<NodeId, InlineStyleEntry>(0)?,
        )?;
        let attribute_bytes = add(5, prepared.serialized().len())?;
        let retained = self
            .retained_bytes
            .checked_sub(old_entry_bytes)
            .and_then(|bytes| bytes.checked_sub(old_attribute_bytes))
            .ok_or(DomDataError::InvalidData)?;
        let retained = add(retained, add(entry_bytes, attribute_bytes)?)?;
        let inline_bytes = self
            .inline_style_bytes
            .checked_sub(old_entry_bytes)
            .ok_or(DomDataError::InvalidData)?;
        let inline_bytes = add(inline_bytes, entry_bytes)?;
        if retained > MAX_DOM_BYTES {
            return Err(DomDataError::LimitExceeded);
        }
        budget.work(add(
            add(insert_work(count, 0)?, insert_work(attrs, 5)?)?,
            add(26, add(old_release, old_attribute_release)?)?,
        )?)?;
        // The prepared block/text/Arc were already charged by CSSOM staging.
        // Only a new attribute key and conservative tree envelopes remain.
        budget.charge(5)?;
        if count == 0 || count >= 11 {
            budget.charge(map_nodes::<NodeId, InlineStyleEntry>(count)?)?;
        }
        if attrs == 0 || attrs >= 11 {
            budget.charge(map_nodes::<String, String>(attrs)?)?;
        }
        let mut key = String::new();
        key.try_reserve_exact(5)
            .map_err(|_| DomDataError::AllocationFailed)?;
        key.push_str("style");
        let release_work = prepared.style().release_work();
        let (block, text) = prepared.into_parts();
        // No Result, callback, parser or admission decision follows. Like the
        // existing DOM metadata paths, BTreeMap allocation is not OOM recovery.
        let NodeKind::Element(element) = &mut self.nodes[node].kind else {
            unreachable!()
        };
        element.attrs.insert(key, text);
        self.inline_styles.insert(
            node,
            InlineStyleEntry {
                block,
                retained_cost: entry_bytes,
                release_work,
            },
        );
        self.retained_bytes = retained;
        self.inline_style_bytes = inline_bytes;
        Ok(())
    }

    /// UI snapshots intentionally project attributes, not private declaration
    /// state. The authoritative Page retains its typed state between renders.
    pub(crate) fn clone_for_ui_snapshot(&self) -> Self {
        self.clone_with_inline_styles(false)
    }

    fn clone_with_inline_styles(&self, include: bool) -> Self {
        Self {
            nodes: self.nodes.clone(),
            root: self.root,
            retained_bytes: if include {
                self.retained_bytes
            } else {
                self.retained_bytes
                    .checked_sub(self.inline_style_bytes)
                    .expect("DOM retained bytes include typed inline styles")
            },
            inline_styles: if include {
                self.inline_styles.clone()
            } else {
                BTreeMap::new()
            },
            inline_style_bytes: if include { self.inline_style_bytes } else { 0 },
            scripting_enabled: self.scripting_enabled,
            mode: self.mode,
            encoding: self.encoding,
            encoding_declaration: self.encoding_declaration,
            url: self.url.clone(),
            first_base: self.first_base.clone(),
            base_tracking: self.base_tracking,
            base_href_bytes: self.base_href_bytes,
            details_groups: self.details_groups.clone(),
            details_toggles: self.details_toggles.clone(),
            details_trackers: self.details_trackers.clone(),
            details_active: self.details_active,
            details_sequence: self.details_sequence,
            details_name_bytes: self.details_name_bytes,
            details_summaries: self.details_summaries.clone(),
        }
    }
}

impl Clone for Document {
    fn clone(&self) -> Self {
        self.clone_with_inline_styles(true)
    }
}
