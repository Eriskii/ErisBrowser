//! The mutable resource phase is explicit; layout itself never grants I/O.
use super::*;
use crate::{
    css::BackgroundImage,
    layout::{BackgroundBindingRange, BackgroundBindings, LoadedBackground, RasterNaturalSize},
};
use std::mem::size_of;

const PASS_WORK: usize = 16 * 1024 * 1024;
const PASS_METADATA: usize = 16 * 1024 * 1024;

fn admit_slots(metadata: usize, count: usize, work: &mut usize) -> bool {
    let complete = count
        .checked_mul(size_of::<Option<LoadedBackground<'_>>>())
        .and_then(|bytes| metadata.checked_add(bytes))
        .and_then(|bytes| bytes.checked_add(64));
    complete.is_some_and(|bytes| bytes <= PASS_METADATA)
        && spend(work, count.saturating_mul(16).saturating_add(64))
}

fn spend(left: &mut usize, cost: usize) -> bool {
    if cost > *left {
        *left = 0;
        false
    } else {
        *left -= cost;
        true
    }
}

fn requested(url: &Url) -> &str {
    // A serialized URL escapes payload '#'; only its fragment delimiter remains.
    url.as_str()
        .split_once('#')
        .map_or(url.as_str(), |(request, _)| request)
}

fn absolute(image: &Option<BackgroundImage>) -> Option<&Url> {
    match image.as_ref()? {
        BackgroundImage::Url(value) => match value.as_ref() {
            css::url::ResolvedUrl::Absolute(url) => Some(url),
            _ => None,
        },
        _ => None,
    }
}

fn has_urls(style: &css::ComputedStyle, work: &mut usize) -> Option<bool> {
    let images = style.backgrounds.image_slots();
    if images.is_empty()
        || images.len() > css::MAX_BACKGROUND_LAYERS
        || !spend(work, images.len().saturating_mul(8).saturating_add(8))
    {
        return None;
    }
    Some(images.iter().any(|image| absolute(image).is_some()))
}

struct BackgroundSelection {
    nodes: Vec<NodeId>,
    ranges: Vec<BackgroundBindingRange>,
    slot_count: usize,
}
impl BackgroundSelection {
    // Allocation is admitted during selection. Allocate before mutable fetch,
    // then populate borrowed values only after resource mutations are complete.
    fn empty_slots<'a>(&self) -> Option<Vec<Option<LoadedBackground<'a>>>> {
        let mut slots = Vec::new();
        slots.try_reserve_exact(self.slot_count).ok()?;
        slots.resize(self.slot_count, None);
        Some(slots)
    }
}

fn valid_raster(image: &RasterImage) -> bool {
    (1..=4096).contains(&image.width)
        && (1..=4096).contains(&image.height)
        && (image.width as usize)
            .checked_mul(image.height as usize)
            .and_then(|n| n.checked_mul(4))
            == Some(image.rgba.len())
}

impl Page {
    /// Compute the winning styles, prepare their image resources with this
    /// page's original fetcher, then lay out that exact style vector. Ordinary
    /// resource refusals leave an unavailable image; transport cancellation can
    /// still terminate the surrounding worker exchange.
    pub fn prepare_layout(&mut self, width: f32, height: f32, fonts: &Fonts) -> LayoutResult {
        let styles =
            css::compute_styles_from_sources(&self.document, &self.stylesheets(), width, height);
        let mut work = PASS_WORK;
        let Some(selected) = self.background_nodes(&styles, &mut work) else {
            return layout::layout(&self.document, &styles, width, height, fonts);
        };
        let Some(mut bindings) = selected.empty_slots() else {
            return layout::layout(&self.document, &styles, width, height, fonts);
        };
        // Prepay the existing Arc-identity union before its temporary hash set.
        let entries = self
            .images
            .len()
            .saturating_add(self.resources.as_ref().map_or(0, |r| r.image_cache.len()));
        if !spend(&mut work, entries.saturating_mul(64).saturating_add(64)) {
            self.background_diagnostic(1, "background preparation work budget exceeded");
            return layout::layout(&self.document, &styles, width, height, fonts);
        }
        let mut live_bytes = self.retained_image_bytes();
        'nodes: for &id in &selected.nodes {
            for image in styles[id].backgrounds.image_slots() {
                if let Some(url) = absolute(image) {
                    self.prepare_background(url, &mut live_bytes, &mut work);
                }
                if work == 0 {
                    break 'nodes;
                }
            }
        }
        self.layout_with_background_nodes(
            &styles,
            (&selected, &mut bindings),
            &mut work,
            [width, height],
            fonts,
        )
    }

    pub(super) fn layout_with_background_resources(
        &self,
        styles: &[css::ComputedStyle],
        width: f32,
        height: f32,
        fonts: &Fonts,
    ) -> LayoutResult {
        let mut work = PASS_WORK;
        let Some(selected) = self.background_nodes(styles, &mut work) else {
            return layout::layout(&self.document, styles, width, height, fonts);
        };
        let Some(mut bindings) = selected.empty_slots() else {
            return layout::layout(&self.document, styles, width, height, fonts);
        };
        self.layout_with_background_nodes(
            styles,
            (&selected, &mut bindings),
            &mut work,
            [width, height],
            fonts,
        )
    }

    fn layout_with_background_nodes<'a>(
        &'a self,
        styles: &[css::ComputedStyle],
        table: (&BackgroundSelection, &mut [Option<LoadedBackground<'a>>]),
        work: &mut usize,
        viewport: [f32; 2],
        fonts: &Fonts,
    ) -> LayoutResult {
        let (selected, bindings) = table;
        'nodes: for &id in &selected.nodes {
            let start = selected.ranges[id].start as usize;
            for (layer, image) in styles[id].backgrounds.image_slots().iter().enumerate() {
                if let Some(url) = absolute(image) {
                    bindings[start + layer] = self.cached_background(url, work);
                }
                if *work == 0 {
                    break 'nodes;
                }
            }
        }
        layout::layout_with_backgrounds(
            &self.document,
            styles,
            viewport[0],
            viewport[1],
            fonts,
            BackgroundBindings {
                ranges: &selected.ranges,
                slots: bindings,
            },
        )
    }

    // One bounded connected-tree preorder, including hidden nodes for the
    // root/body canvas-owner exception. No offscreen/zero-tile fetch heuristic.
    fn background_nodes(
        &self,
        styles: &[css::ComputedStyle],
        work: &mut usize,
    ) -> Option<BackgroundSelection> {
        if self.resources.is_none() || self.policy_blocks_styles {
            return None;
        }
        let count = self.document.nodes.len();
        let entries = self
            .images
            .len()
            .saturating_add(self.resources.as_ref()?.image_cache.len());
        let slot = size_of::<BackgroundBindingRange>()
            + size_of::<NodeId>()
            + size_of::<(NodeId, usize, bool)>()
            + size_of::<bool>();
        let metadata = count
            .checked_mul(slot)?
            .checked_add(entries.checked_mul(64)?)?
            .checked_add(256)?;
        if count > crate::dom::MAX_NODES
            || metadata > PASS_METADATA
            || !spend(work, count.saturating_mul(32).saturating_add(64))
        {
            return None;
        }
        let mut any_url = false;
        for style in styles.iter().take(count) {
            if has_urls(style, work)? {
                any_url = true;
                break;
            }
        }
        if !any_url {
            return None;
        }
        let mut seen = Vec::new();
        let mut pending = Vec::new();
        let mut selected = Vec::new();
        seen.try_reserve_exact(count).ok()?;
        pending.try_reserve_exact(count).ok()?;
        selected.try_reserve_exact(count).ok()?;
        seen.resize(count, false);
        if count == 0 {
            return None;
        }
        pending.push((self.document.root, 0usize, false));
        let mut edges = count;
        let mut html = None;
        let mut body = None;
        while let Some((id, depth, inherited_hidden)) = pending.pop() {
            let Some(visited) = seen.get_mut(id) else {
                continue;
            };
            if *visited {
                continue;
            }
            *visited = true;
            let node = &self.document.nodes[id];
            let tag = self.html_tag(id);
            if tag == Some("template") {
                continue;
            }
            if tag == Some("html") && html.is_none() {
                html = Some(id);
            }
            if tag == Some("body") && body.is_none() {
                body = Some(id);
            }
            let style = styles.get(id);
            let hidden = inherited_hidden
                || style.is_none_or(|s| s.display == css::Display::None)
                || tag == Some("input")
                    && self
                        .document
                        .attr(id, "type")
                        .is_some_and(|t| t.eq_ignore_ascii_case("hidden"));
            if !hidden && has_urls(style?, work)? {
                selected.push(id);
            }
            if depth >= 128 {
                continue;
            }
            // A valid DOM has at most N-1 connected child edges. Refuse malformed
            // public host data before extending an unbounded pending stack.
            if node.children.len() > edges
                || node.children.len() > count.saturating_sub(pending.len())
            {
                return None;
            }
            edges -= node.children.len();
            let closed = tag == Some("details") && self.document.attr(id, "open").is_none();
            let summary = closed.then(|| self.document.first_summary(id)).flatten();
            pending.extend(
                node.children
                    .iter()
                    .rev()
                    .map(|&child| (child, depth + 1, hidden || closed && Some(child) != summary)),
            );
        }
        if let Some(id) = layout::canvas_background_owner(styles, html, body)
            && has_urls(styles.get(id)?, work)?
        {
            if !spend(work, selected.len().saturating_mul(4).saturating_add(4)) {
                return None;
            }
            if !selected.contains(&id) {
                selected.push(id);
            }
        }
        if selected.is_empty() || !spend(work, selected.len().saturating_mul(16).saturating_add(64))
        {
            return None;
        }
        let mut ranges = Vec::new();
        ranges.try_reserve_exact(count).ok()?;
        ranges.resize(count, BackgroundBindingRange::default());
        let mut slot_count = 0usize;
        for &id in &selected {
            let len = styles.get(id)?.backgrounds.image_count();
            if !(1..=css::MAX_BACKGROUND_LAYERS).contains(&len) {
                return None;
            }
            ranges[id] = BackgroundBindingRange {
                start: u32::try_from(slot_count).ok()?,
                len: u8::try_from(len).ok()?,
            };
            slot_count = slot_count.checked_add(len)?;
        }
        if !admit_slots(metadata, slot_count, work) {
            return None;
        }
        Some(BackgroundSelection {
            nodes: selected,
            ranges,
            slot_count,
        })
    }

    fn cached_background<'a>(
        &'a self,
        url: &Url,
        work: &mut usize,
    ) -> Option<LoadedBackground<'a>> {
        if !spend(
            work,
            url.as_str().len().saturating_mul(8).saturating_add(16),
        ) {
            return None;
        }
        let resources = self.resources.as_ref()?;
        let request = requested(url);
        let key = resources.background_keys.get(request)?;
        if !spend(work, key.len().saturating_mul(4).saturating_add(16)) {
            return None;
        }
        let (image, _) = resources.image_cache.get(request)?;
        let published = self.images.get(key)?;
        if !Arc::ptr_eq(image, published) || !valid_raster(image) {
            return None;
        }
        Some(LoadedBackground {
            key,
            natural: RasterNaturalSize {
                width: image.width,
                height: image.height,
            },
        })
    }

    fn background_diagnostic(&mut self, bit: u8, message: &'static str) {
        let Some(resources) = self.resources.as_mut() else {
            return;
        };
        if resources.background_diagnostics & bit != 0 {
            return;
        }
        resources.background_diagnostics |= bit;
        if self.diagnostics.len() < 255 {
            self.diagnostics.push(message.into());
        }
    }

    fn prepare_background(&mut self, url: &Url, live_bytes: &mut usize, work: &mut usize) {
        if self.cached_background(url, work).is_some() {
            return;
        }
        if *work == 0 {
            self.background_diagnostic(1, "background preparation work budget exceeded");
            return;
        }
        let request = requested(url);
        let Some(resources) = self.resources.as_mut() else {
            return;
        };
        let cost = url.as_str().len().saturating_mul(8).saturating_add(128);
        if !spend(&mut resources.resource_url_work_left, cost) {
            self.background_diagnostic(2, "background resource URL work budget exceeded");
            return;
        }
        // Includes transient failure-query ownership, requested URL clone and
        // retained success/failure map key. No refund, including cache failures.
        // The URL clone still contains its fragment until set_fragment(None).
        // Reserve against that full spelling even though cache keys omit it.
        let metadata = url.as_str().len().saturating_mul(3).saturating_add(512);
        if !spend(&mut resources.background_metadata_left, metadata) {
            self.background_diagnostic(4, "background resource metadata budget exceeded");
            return;
        }
        let failed_key = (ResourceKind::Image, request.to_owned());
        if resources.failed.contains(&failed_key) {
            return;
        }
        let cached = resources.image_cache.get(request).cloned();
        let (image, clean) = if let Some(cached) = cached {
            cached
        } else {
            let remaining = MAX_DECODED_IMAGE_BYTES
                .saturating_sub(resources.decoded_image_bytes)
                .min(MAX_DECODED_IMAGE_BYTES.saturating_sub(*live_bytes));
            let mut target = url.clone();
            target.set_fragment(None);
            let result = resources
                .fetcher
                .fetch(&target, Some(&self.url), ResourceKind::Image)
                .and_then(|mut response| {
                    let image = if let Some(image) = response.decoded_image.take() {
                        Ok(image)
                    } else if response
                        .content_type
                        .split(';')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .eq_ignore_ascii_case("image/svg+xml")
                    {
                        crate::svg::render_with_budget(&response.text(), None, None, remaining)
                    } else {
                        decode_image_with_budget(&response.bytes, remaining)
                    }?;
                    if !valid_raster(&image) || image.rgba.len() > remaining {
                        return Err("decoded image budget exceeded".into());
                    }
                    Ok((Arc::new(image), response.origin_clean))
                });
            match result {
                Ok((image, clean)) => {
                    resources.decoded_image_bytes += image.rgba.len();
                    *live_bytes += image.rgba.len();
                    resources
                        .image_cache
                        .insert(failed_key.1.clone(), (image.clone(), clean));
                    (image, clean)
                }
                Err(_) => {
                    resources.failed.insert(failed_key);
                    self.background_diagnostic(
                        8,
                        "background image unavailable (fetch, decode or image budget refusal)",
                    );
                    return;
                }
            }
        };
        if !valid_raster(&image) {
            return;
        }
        self.publish_background_key(failed_key.1, image, clean);
    }

    fn publish_background_key(&mut self, request: String, image: Arc<RasterImage>, clean: bool) {
        let Some(resources) = self.resources.as_mut() else {
            return;
        };
        if !resources.background_keys.contains_key(&request)
            && resources.background_keys.len() >= net::MAX_RESOURCES
        {
            return;
        }
        let key = loop {
            if !spend(&mut resources.resource_url_work_left, 256)
                || !spend(&mut resources.background_metadata_left, 64)
            {
                return;
            }
            let serial = resources.background_key_serial;
            let Some(next) = serial.checked_add(1) else {
                return;
            };
            resources.background_key_serial = next;
            let key = format!("eris-css-background:{serial}");
            if !self.images.contains_key(&key) && !self.image_origins.contains_key(&key) {
                break key;
            }
        };
        let cost = request
            .len()
            .saturating_mul(2)
            .saturating_add(key.len().saturating_mul(4))
            .saturating_add(512);
        if !spend(&mut resources.resource_url_work_left, cost)
            || !spend(&mut resources.background_metadata_left, cost)
        {
            return;
        }
        // Prefixes are not ownership: the preceding collision checks protect
        // authored img aliases, and every layout rechecks the exact cached Arc.
        self.images.insert(key.clone(), image);
        self.image_origins.insert(key.clone(), clean);
        resources.background_keys.insert(request, key);
    }
}

#[cfg(test)]
#[path = "background_slot_admission_tests.rs"]
mod slot_admission_tests;
