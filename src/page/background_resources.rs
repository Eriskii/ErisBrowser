//! The mutable resource phase is explicit; layout itself never grants I/O.
use super::*;
use crate::{
    css::BackgroundImage,
    layout::{LoadedBackground, RasterNaturalSize},
};
use std::mem::size_of;

const PASS_WORK: usize = 16 * 1024 * 1024;
const PASS_METADATA: usize = 16 * 1024 * 1024;

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

fn absolute(style: &css::ComputedStyle) -> Option<&Url> {
    match style.background_image.as_ref()? {
        BackgroundImage::Url(value) => match value.as_ref() {
            css::url::ResolvedUrl::Absolute(url) => Some(url),
            _ => None,
        },
        _ => None,
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
        let Some(nodes) = self.background_nodes(&styles, &mut work) else {
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
        for &id in &nodes {
            if let Some(url) = styles.get(id).and_then(absolute) {
                self.prepare_background(url, &mut live_bytes, &mut work);
            }
            if work == 0 {
                break;
            }
        }
        self.layout_with_background_nodes(&styles, &nodes, &mut work, width, height, fonts)
    }

    pub(super) fn layout_with_background_resources(
        &self,
        styles: &[css::ComputedStyle],
        width: f32,
        height: f32,
        fonts: &Fonts,
    ) -> LayoutResult {
        let mut work = PASS_WORK;
        let Some(nodes) = self.background_nodes(styles, &mut work) else {
            return layout::layout(&self.document, styles, width, height, fonts);
        };
        self.layout_with_background_nodes(styles, &nodes, &mut work, width, height, fonts)
    }

    fn layout_with_background_nodes(
        &self,
        styles: &[css::ComputedStyle],
        nodes: &[NodeId],
        work: &mut usize,
        width: f32,
        height: f32,
        fonts: &Fonts,
    ) -> LayoutResult {
        let mut bindings = Vec::new();
        if bindings
            .try_reserve_exact(self.document.nodes.len())
            .is_err()
        {
            return layout::layout(&self.document, styles, width, height, fonts);
        }
        bindings.resize(self.document.nodes.len(), None);
        for &id in nodes {
            if let Some(url) = styles.get(id).and_then(absolute) {
                bindings[id] = self.cached_background(url, work);
            }
            if *work == 0 {
                break;
            }
        }
        layout::layout_with_backgrounds(&self.document, styles, width, height, fonts, &bindings)
    }

    // One bounded connected-tree preorder, including hidden nodes for the
    // root/body canvas-owner exception. No offscreen/zero-tile fetch heuristic.
    fn background_nodes(
        &self,
        styles: &[css::ComputedStyle],
        work: &mut usize,
    ) -> Option<Vec<NodeId>> {
        if self.resources.is_none() || self.policy_blocks_styles {
            return None;
        }
        let count = self.document.nodes.len();
        let entries = self
            .images
            .len()
            .saturating_add(self.resources.as_ref()?.image_cache.len());
        let slot = size_of::<Option<LoadedBackground<'_>>>()
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
        if !styles.iter().take(count).any(|s| absolute(s).is_some()) {
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
            return Some(selected);
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
            if !hidden && style.and_then(absolute).is_some() {
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
        let has_background = |id: &NodeId| {
            styles
                .get(*id)
                .is_some_and(|s| s.background_color.a > 0 || s.background_image.is_some())
        };
        let owner = html.filter(has_background).or_else(|| {
            body.filter(|id| {
                styles
                    .get(*id)
                    .is_some_and(|s| s.display != css::Display::None)
                    && has_background(id)
            })
        });
        if let Some(id) = owner.filter(|&id| styles.get(id).and_then(absolute).is_some())
            && !selected.contains(&id)
        {
            selected.push(id);
        }
        Some(selected)
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
