//! Page lifecycle connecting independent parsing, scripting, layout and paint.
use crate::{
    css,
    dom::{Document, Namespace, NodeId},
    graphics::{Fonts, ImageStore, RasterImage},
    layout::{self, LayoutResult},
    net::{self, Fetcher, ResourceKind},
    script::Runtime,
};
use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    sync::Arc,
    time::Instant,
};

pub(crate) const MAX_DECODED_IMAGE_BYTES: usize = 64 * 1024 * 1024;
const MAX_STYLE_BYTES: usize = 8 * 1024 * 1024;
const MAX_SCRIPT_BYTES: usize = 1024 * 1024;
const MAX_FORM_BYTES: usize = net::MAX_FORM_BODY_BYTES;
use url::Url;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Navigation {
    pub address: String,
    pub form_body: Option<String>,
}

/// Only details toggle tasks are currently hosted. This does not represent a
/// complete HTML event loop or the JavaScript microtask queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Idle,
    Pending,
    Suspended,
}
impl Navigation {
    pub fn get(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            form_body: None,
        }
    }
}

pub struct Page {
    pub url: Url,
    pub document: Document,
    pub runtime: Runtime,
    pub images: ImageStore,
    pub diagnostics: Vec<String>,
    pub load_ms: f64,
    pub scripts_enabled: bool,
    image_origins: HashMap<String, bool>,
    external_styles: HashMap<NodeId, crate::stylesheet_loading::Sources>,
    inline_styles: HashMap<NodeId, (Arc<str>, crate::stylesheet_loading::Sources)>,
    policy_blocks_styles: bool,
    tasks_suspended: bool,
}

impl Page {
    /// Apply a same-document URL change without reloading or refreezing a base.
    pub fn navigate_fragment(&mut self, target: Url) -> bool {
        if !same_document_url(&self.url, &target) {
            return false;
        }
        self.document.set_url(target.clone());
        self.url = target;
        true
    }
    pub fn load(address: &str, scripts_enabled: bool) -> Result<Self, String> {
        Self::load_navigation(&Navigation::get(address), scripts_enabled)
    }
    pub fn load_navigation(navigation: &Navigation, scripts_enabled: bool) -> Result<Self, String> {
        let start = Instant::now();
        let url = net::parse_address(&navigation.address)?;
        if navigation.form_body.is_some() && !matches!(url.scheme(), "http" | "https") {
            return Err("POST form submissions require HTTP or HTTPS".into());
        }
        if url.scheme() == "eris" && url.path() == "home" {
            return Ok(Self::from_html(
                url,
                include_str!("../assets/home.html"),
                scripts_enabled,
            ));
        }
        if url.scheme() == "about" && url.path() == "blank" {
            return Ok(Self::from_html(
                url,
                "<!doctype html><title>Blank</title>",
                scripts_enabled,
            ));
        }
        let mut fetcher = Fetcher::for_document(&url);
        let response = fetcher.fetch_document(&url, navigation.form_body.as_deref())?;
        let csp = response.headers.contains_key("content-security-policy");
        let mime = response
            .content_type
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        let html_encoding = (mime == "text/html" || mime.is_empty())
            .then(|| crate::text_encoding::html_encoding(&response.bytes, &response.content_type));
        let html = if let Some(selected) = html_encoding {
            crate::text_encoding::decode(&response.bytes, selected.encoding)
        } else if mime.starts_with("image/") {
            format!(
                "<!doctype html><title>Image</title><body><img src=\"{}\"></body>",
                escape_html(response.url.as_str())
            )
        } else {
            format!(
                "<!doctype html><title>Text</title><pre>{}</pre>",
                escape_html(&response.text())
            )
        };
        let mut page = Self::unexecuted(response.url, &html, scripts_enabled);
        if let Some(selected) = html_encoding {
            let mut encoding = selected.encoding;
            if !selected.certain
                && encoding != encoding_rs::UTF_16LE
                && encoding != encoding_rs::UTF_16BE
                && let Some(declared) = page.document.encoding_declaration()
                && declared != encoding
            {
                // Reparse cached bytes before any author code or resource loads.
                // The original navigation (including POST) is never repeated.
                let decoded = crate::text_encoding::decode(&response.bytes, declared);
                page = Self::unexecuted(page.url, &decoded, scripts_enabled);
                encoding = declared;
            }
            page.document.set_encoding(encoding);
        }
        page.document.initialize_url(page.url.clone());
        let environment_encoding =
            encoding_rs::Encoding::for_label(page.document.character_set().as_bytes())
                .expect("document encoding is canonical");
        page.apply_author_policy(csp);
        let ids = page.document.query_selector_all("style, link, script, img");
        let base = crate::document_url::base_url(&page.document);
        let mut styles = crate::stylesheet_loading::Loader::new(&page.url, environment_encoding);
        let mut script_sources: HashMap<NodeId, Arc<str>> = HashMap::new();
        let mut text_cache: HashMap<(ResourceKind, String, &'static str), Arc<str>> =
            HashMap::new();
        let mut image_cache: HashMap<String, (Arc<RasterImage>, bool)> = HashMap::new();
        let mut failed = HashSet::new();
        let mut decoded_bytes = 0usize;
        let mut script_bytes = 0usize;
        let mut resource_url_work = 32usize * 1024 * 1024;
        let mut style_text_left = MAX_STYLE_BYTES;
        let mut style_child_visits_left = crate::dom::MAX_STYLE_CHILD_VISITS;
        let mut inline_style_count = 0usize;
        let mut style_text_exhausted = false;
        for id in ids {
            if page.policy_blocks_styles {
                break;
            }
            if !page.is_active_node(id) {
                continue;
            }
            if page.document.tag(id) == Some("style")
                && matches!(
                    page.document.namespace(id),
                    Some(Namespace::Html | Namespace::Svg)
                )
            {
                if style_text_exhausted || inline_style_count >= crate::dom::MAX_INLINE_STYLES {
                    continue;
                }
                let original = match page.inline_style_source(
                    id,
                    &mut style_text_left,
                    &mut style_child_visits_left,
                ) {
                    Ok(Some(source)) => source,
                    Ok(None) => continue,
                    Err(error) => {
                        page.diagnostics.push(error);
                        style_text_exhausted = true;
                        continue;
                    }
                };
                inline_style_count += 1;
                let original: Arc<str> = original.into();
                match styles.inline(&mut fetcher, &original, &base, &mut page.diagnostics) {
                    Ok(expanded) => {
                        page.inline_styles.insert(id, (original, expanded));
                    }
                    Err(error) => page.diagnostics.push(error),
                }
                continue;
            }
            let tag = page.html_tag(id).unwrap_or("");
            let (href, kind) = match tag {
                "link"
                    if page.css_type_supported(id)
                        && page.document.attr(id, "rel").is_some_and(|r| {
                            r.split_ascii_whitespace()
                                .any(|s| s.eq_ignore_ascii_case("stylesheet"))
                        }) =>
                {
                    (page.document.attr(id, "href"), ResourceKind::Style)
                }
                "script"
                    if page.scripts_enabled
                        && page.document.attr(id, "type").is_none_or(|kind| {
                            kind.is_empty() || net::is_javascript_mime(kind)
                        }) =>
                {
                    (page.document.attr(id, "src"), ResourceKind::Script)
                }
                "img" => (page.document.attr(id, "src"), ResourceKind::Image),
                _ => continue,
            };
            let Some(href) = href.map(str::to_owned) else {
                continue;
            };
            // A long base copied for thousands of tiny references can otherwise
            // multiply allocation and parsing work before fetch limits apply.
            let work = base
                .as_str()
                .len()
                .saturating_add(href.len().saturating_mul(8))
                .saturating_add(1);
            if work > resource_url_work {
                page.diagnostics
                    .push("resource URL work budget exceeded".into());
                break;
            }
            resource_url_work -= work;
            let mut target = match crate::document_url::parse(&page.document, &base, &href) {
                Ok(url) => url,
                Err(error) => {
                    page.diagnostics.push(format!("resource URL: {error}"));
                    continue;
                }
            };
            target.set_fragment(None);
            if kind == ResourceKind::Style {
                match styles.external(&mut fetcher, &target, &mut page.diagnostics) {
                    Ok(source) => {
                        page.external_styles.insert(id, source);
                    }
                    Err(error) => page.diagnostics.push(format!("stylesheet: {error}")),
                }
                continue;
            }
            let key = target.to_string();
            let fallback_encoding = if kind == ResourceKind::Script {
                page.document
                    .attr(id, "charset")
                    .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()))
                    .unwrap_or(environment_encoding)
            } else {
                environment_encoding
            };
            let text_key = (kind, key.clone(), fallback_encoding.name());
            if failed.contains(&(kind, key.clone())) {
                continue;
            }
            if kind == ResourceKind::Image {
                if let Some((image, origin_clean)) = image_cache.get(&key).cloned() {
                    page.attach_image(id, href, image, origin_clean);
                    continue;
                }
            } else if let Some(source) = text_cache.get(&text_key).cloned() {
                script_sources.insert(id, source);
                continue;
            }
            let mut resource = match fetcher.fetch(&target, Some(&page.url), kind) {
                Ok(resource) => resource,
                Err(error) => {
                    failed.insert((kind, key));
                    page.diagnostics
                        .push(format!("resource {}: {error}", diagnostic_url(&href)));
                    continue;
                }
            };
            match kind {
                ResourceKind::Script => {
                    let encoding = crate::text_encoding::script_encoding(
                        &resource.bytes,
                        &resource.content_type,
                        None,
                        fallback_encoding,
                    );
                    let source = crate::text_encoding::decode(&resource.bytes, encoding);
                    if source.len() > MAX_SCRIPT_BYTES.saturating_sub(script_bytes) {
                        failed.insert((kind, key));
                        page.diagnostics
                            .push("script source budget exceeded".into());
                        continue;
                    }
                    script_bytes += source.len();
                    let source: Arc<str> = source.into();
                    text_cache.insert(text_key, source.clone());
                    script_sources.insert(id, source);
                }
                ResourceKind::Image => {
                    let remaining = MAX_DECODED_IMAGE_BYTES.saturating_sub(decoded_bytes);
                    let result = if let Some(image) = resource.decoded_image.take() {
                        Ok(image)
                    } else if resource
                        .content_type
                        .split(';')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .eq_ignore_ascii_case("image/svg+xml")
                    {
                        crate::svg::render_with_budget(&resource.text(), None, None, remaining)
                    } else {
                        decode_image_with_budget(&resource.bytes, remaining)
                    };
                    match result {
                        Ok(image) => {
                            if image.rgba.len()
                                > MAX_DECODED_IMAGE_BYTES.saturating_sub(decoded_bytes)
                            {
                                failed.insert((kind, key));
                                page.diagnostics
                                    .push("decoded image budget exceeded".into());
                                continue;
                            }
                            decoded_bytes += image.rgba.len();
                            let image = Arc::new(image);
                            image_cache.insert(key, (image.clone(), resource.origin_clean));
                            page.attach_image(id, href, image, resource.origin_clean);
                        }
                        Err(error) => {
                            failed.insert((kind, key));
                            page.diagnostics
                                .push(format!("image {}: {error}", diagnostic_url(&href)));
                        }
                    }
                }
                ResourceKind::Document | ResourceKind::Style => {}
            }
        }
        page.run_scripts(&script_sources);
        page.refresh_inline_svg();
        page.load_ms = start.elapsed().as_secs_f64() * 1000.0;
        Ok(page)
    }
    fn unexecuted(url: Url, html: &str, scripts_enabled: bool) -> Self {
        let mut document = Document::parse_with_scripting(html, scripts_enabled);
        document.initialize_url(url.clone());
        Self {
            url,
            document,
            runtime: Runtime::new(),
            images: HashMap::new(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
            scripts_enabled,
            image_origins: HashMap::new(),
            external_styles: HashMap::new(),
            inline_styles: HashMap::new(),
            policy_blocks_styles: false,
            tasks_suspended: false,
        }
    }
    pub fn from_html(url: Url, html: &str, scripts_enabled: bool) -> Self {
        let started = Instant::now();
        let mut page = Self::unexecuted(url, html, scripts_enabled);
        page.apply_author_policy(false);
        page.run_scripts(&HashMap::new());
        page.refresh_inline_svg();
        page.load_ms = started.elapsed().as_secs_f64() * 1000.0;
        page
    }
    fn apply_author_policy(&mut self, header_csp: bool) {
        let meta_csp = self
            .document
            .query_selector_all("meta")
            .into_iter()
            .any(|id| {
                self.html_tag(id) == Some("meta")
                    && self.is_active_node(id)
                    && self
                        .document
                        .attr(id, "http-equiv")
                        .is_some_and(|v| v.eq_ignore_ascii_case("content-security-policy"))
            });
        if !(header_csp || meta_csp) || self.policy_blocks_styles {
            return;
        }
        // This deliberately refuses all active content instead of partially implementing CSP.
        self.scripts_enabled = false;
        self.policy_blocks_styles = true;
        for id in 0..self.document.nodes.len() {
            self.document.remove_attr(id, "style");
        }
        self.diagnostics.push("CSP present: scripts, author styles and external resources disabled by conservative policy".into());
    }
    fn html_tag(&self, id: NodeId) -> Option<&str> {
        (self.document.namespace(id) == Some(Namespace::Html))
            .then(|| self.document.tag(id))
            .flatten()
    }
    fn is_active_node(&self, id: NodeId) -> bool {
        self.document.is_active_node(id)
    }
    /// Image provenance for future pixel-reading APIs; unknown images are not
    /// implicitly clean. Cached aliases retain the original redirect taint.
    pub fn image_origin_clean(&self, key: &str) -> Option<bool> {
        self.image_origins.get(key).copied()
    }
    fn attach_image(
        &mut self,
        id: NodeId,
        key: String,
        image: Arc<RasterImage>,
        origin_clean: bool,
    ) {
        self.document
            .set_attr(id, "data-eris-natural-width", &image.width.to_string());
        self.document
            .set_attr(id, "data-eris-natural-height", &image.height.to_string());
        self.image_origins.insert(key.clone(), origin_clean);
        self.images.insert(key, image);
    }
    pub fn refresh_inline_svg(&mut self) {
        // A DOM replacement must release rasters belonging to removed or now-inert SVG nodes.
        self.images
            .retain(|key, _| !key.starts_with("eris-inline-svg:"));
        self.image_origins
            .retain(|key, _| !key.starts_with("eris-inline-svg:"));
        let mut seen = HashSet::new();
        let mut image_bytes = self
            .images
            .values()
            .filter(|image| seen.insert(Arc::as_ptr(image)))
            .map(|image| image.rgba.len())
            .sum::<usize>();
        let mut source_bytes = 0usize;
        let mut count = 0usize;
        for id in self.document.query_selector_all("svg") {
            if !self.is_active_node(id) || self.document.namespace(id) != Some(Namespace::Svg) {
                continue;
            }
            let mut ancestor = self.document.nodes[id].parent;
            let mut nested = false;
            while let Some(node) = ancestor {
                if self.document.namespace(node) == Some(Namespace::Svg)
                    && self.document.tag(node) == Some("svg")
                {
                    nested = true;
                    break;
                }
                ancestor = self.document.nodes[node].parent;
            }
            if nested {
                continue;
            }
            if count >= 16 {
                self.diagnostics
                    .push("inline SVG count budget exceeded".into());
                break;
            }
            count += 1;
            let source = self.document.outer_html(id);
            source_bytes += source.len();
            if source_bytes > MAX_SCRIPT_BYTES {
                self.diagnostics
                    .push("inline SVG source budget exceeded".into());
                break;
            }
            match crate::svg::render_with_budget(
                &source,
                None,
                None,
                MAX_DECODED_IMAGE_BYTES.saturating_sub(image_bytes),
            ) {
                Ok(image) => {
                    if image.rgba.len() > MAX_DECODED_IMAGE_BYTES.saturating_sub(image_bytes) {
                        self.diagnostics
                            .push("decoded image budget exceeded".into());
                        break;
                    }
                    image_bytes += image.rgba.len();
                    self.attach_image(id, format!("eris-inline-svg:{id}"), Arc::new(image), true);
                }
                Err(error) => self.diagnostics.push(format!("inline SVG: {error}")),
            }
        }
    }
    pub fn error(address: &str, message: &str) -> Self {
        let html = format!(
            "<!doctype html><title>Unable to open page</title><style>body{{font-family:sans-serif;background:#131620;color:#edf0fa;margin:60px;max-width:850px}}h1{{font-size:36px}}p{{line-height:1.6;color:#bec7dc}}pre{{background:#202638;padding:24px;white-space:pre-wrap}}</style><h1>Unable to open page</h1><p>{}</p><pre>{}</pre><p>Use the address bar to try another address.</p>",
            escape_html(address),
            escape_html(message)
        );
        Self::from_html(
            Url::parse("eris:error").expect("constant URL"),
            &html,
            false,
        )
    }
    fn run_scripts(&mut self, external: &HashMap<NodeId, Arc<str>>) {
        if !self.scripts_enabled {
            return;
        }
        let mut source_bytes = 0usize;
        let mut count = 0usize;
        for id in self.document.query_selector_all("script") {
            // SVG script execution is not implemented by this HTML script loader.
            if self.html_tag(id) != Some("script") || !self.is_active_node(id) {
                continue;
            }
            if count >= 64 {
                self.diagnostics
                    .push("page script count budget exceeded".into());
                break;
            }
            count += 1;
            let kind = self.document.attr(id, "type").unwrap_or("");
            if !kind.is_empty() && !net::is_javascript_mime(kind) {
                self.diagnostics
                    .push(format!("unsupported script type: {kind}"));
                continue;
            }
            let source = if self.document.attr(id, "src").is_some() {
                external.get(&id).cloned()
            } else {
                Some(Arc::from(self.document.text_content(id)))
            };
            if let Some(source) = source {
                source_bytes += source.len();
                if source_bytes > 1024 * 1024 {
                    self.diagnostics
                        .push("page script source budget exceeded".into());
                    break;
                }
                if let Err(error) = self.runtime.execute(&source, &mut self.document) {
                    self.diagnostics.push(format!("script: {error}"));
                }
            }
        }
        if let Err(error) = self.runtime.dispatch_dom_content_loaded(&mut self.document) {
            self.diagnostics.push(format!("DOMContentLoaded: {error}"));
        }
        self.disclosure_checkpoint();
    }
    pub fn stylesheets(&self) -> Vec<css::StyleSource> {
        if self.policy_blocks_styles {
            return Vec::new();
        }
        let mut sources = Vec::new();
        let mut bytes = 0usize;
        let mut text_left = MAX_STYLE_BYTES;
        let mut child_visits_left = crate::dom::MAX_STYLE_CHILD_VISITS;
        let mut inline_count = 0usize;
        for id in self.document.query_selector_all("style, link") {
            if sources.len() >= 256 {
                return sources;
            }
            if !self.is_active_node(id) || !self.css_type_supported(id) {
                continue;
            }
            let media = self.document.attr(id, "media");
            if media.is_some_and(|media| !crate::stylesheet_loading::valid_media_condition(media)) {
                continue;
            }
            let style_element = self.document.tag(id) == Some("style")
                && matches!(
                    self.document.namespace(id),
                    Some(Namespace::Html | Namespace::Svg)
                );
            let parts = if style_element {
                if inline_count >= crate::dom::MAX_INLINE_STYLES {
                    return sources;
                }
                let current =
                    match self.inline_style_source(id, &mut text_left, &mut child_visits_left) {
                        Ok(Some(source)) => source,
                        Ok(None) => continue,
                        Err(_) => return sources,
                    };
                inline_count += 1;
                self.inline_styles
                    .get(&id)
                    .filter(|(original, _)| original.as_ref() == current)
                    .map(|(_, expanded)| expanded.clone())
                    .unwrap_or_else(|| vec![css::StyleSource::new(current)])
            } else if let Some(source) = self
                .external_styles
                .get(&id)
                .filter(|_| self.html_tag(id) == Some("link"))
            {
                source.clone()
            } else {
                continue;
            };
            for mut source in parts {
                let size = source
                    .source
                    .len()
                    .saturating_add(source.media.iter().map(|m| m.len()).sum::<usize>())
                    .saturating_add(media.map_or(0, str::len));
                if sources.len() >= 256 || size > MAX_STYLE_BYTES.saturating_sub(bytes) {
                    return sources;
                }
                bytes += size;
                if let Some(media) = media {
                    source.media.push(media.into());
                }
                sources.push(source);
            }
        }
        sources
    }
    fn css_type_supported(&self, node: NodeId) -> bool {
        if self.document.tag(node) == Some("style") {
            return self.document.is_css_style_element(node);
        }
        self.document.attr(node, "type").is_none_or(|kind| {
            kind.trim().is_empty()
                || kind
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .eq_ignore_ascii_case("text/css")
        })
    }
    /// Validate style metadata before reserving child visits or copying text.
    /// Shared by initial loading and every later stylesheet collection.
    fn inline_style_source(
        &self,
        node: NodeId,
        bytes_left: &mut usize,
        child_visits_left: &mut usize,
    ) -> Result<Option<String>, String> {
        if !self.document.is_css_style_element(node)
            || self
                .document
                .attr(node, "media")
                .is_some_and(|media| !crate::stylesheet_loading::valid_media_condition(media))
        {
            return Ok(None);
        }
        self.document
            .child_text_content_bounded(node, bytes_left, child_visits_left)
            .map(Some)
            .ok_or_else(|| "inline stylesheet text/traversal budget exceeded".into())
    }
    pub fn layout(&self, width: f32, height: f32, fonts: &Fonts) -> LayoutResult {
        let styles =
            css::compute_styles_from_sources(&self.document, &self.stylesheets(), width, height);
        layout::layout(&self.document, &styles, width, height, fonts)
    }
    pub fn title(&self) -> String {
        let title = self.document.title();
        if title.is_empty() {
            self.url.to_string()
        } else {
            title
        }
    }
    fn disabled_control(&self, node: NodeId) -> bool {
        self.document.disabled_control(node)
    }
    fn interaction_blocked(&self, node: NodeId) -> bool {
        self.document.interaction_blocked(node)
    }
    pub fn task_state(&self) -> TaskState {
        if !self.scripts_enabled {
            TaskState::Idle
        } else if self.tasks_suspended {
            TaskState::Suspended
        } else if self.document.has_pending_details_toggles() {
            TaskState::Pending
        } else {
            TaskState::Idle
        }
    }
    pub fn disclosure_checkpoint(&mut self) {
        if self.task_state() == TaskState::Pending
            && let Err(error) = self.runtime.dispatch_details_toggles(&mut self.document)
        {
            // A persistent allocation failure must not drive an idle IPC loop.
            // Keep unstarted tasks but require a new document before retrying.
            self.tasks_suspended = true;
            self.diagnostics.push(format!(
                "toggle: {error}; automatic task dispatch suspended until reload"
            ));
        }
    }
    /// Continue one bounded batch without requiring a click or edit. Rendering
    /// itself never runs author callbacks, keeping snapshots deterministic.
    pub fn run_pending_tasks(&mut self) {
        if self.task_state() == TaskState::Pending {
            self.disclosure_checkpoint();
            // Quota failures can still leave visible, valid partial mutations.
            self.refresh_inline_svg();
        }
    }
    pub fn click(&mut self, id: NodeId) -> Option<Navigation> {
        let navigation = self.click_impl(id, false);
        self.disclosure_checkpoint();
        if self.scripts_enabled {
            // Click/default actions and their queued toggle callbacks may all
            // mutate SVG, including before a handled error or quota failure.
            self.refresh_inline_svg();
        }
        navigation
    }
    pub fn click_default_summary(&mut self, id: NodeId) {
        self.click_impl(id, true);
        self.disclosure_checkpoint();
        if self.scripts_enabled {
            self.refresh_inline_svg();
        }
    }
    fn click_impl(&mut self, id: NodeId, default_summary: bool) -> Option<Navigation> {
        let original_id = id;
        if self.interaction_blocked(id) || self.document.disclosure_hidden(id) {
            return None;
        }
        if default_summary
            && (self.html_tag(id) != Some("details") || self.document.first_summary(id).is_some())
        {
            return None;
        }
        // Activation belongs to the original click path. Listener mutations
        // must not substitute a different summary by reparenting the target.
        let summary = if default_summary {
            None
        } else {
            let mut node = Some(id);
            let mut summary = None;
            for _ in 0..crate::dom::MAX_DEPTH {
                let Some(current) = node else { break };
                match self.html_tag(current) {
                    Some("summary") => {
                        summary = Some(current);
                        break;
                    }
                    Some("input" | "button" | "select" | "textarea") => break,
                    Some("a") if self.document.attr(current, "href").is_some() => break,
                    _ => {}
                }
                node = self
                    .document
                    .nodes
                    .get(current)
                    .and_then(|node| node.parent);
            }
            summary
        };
        if self.scripts_enabled {
            if let Err(error) = self.runtime.dispatch_click(id, &mut self.document) {
                self.diagnostics.push(format!("click: {error}"));
            }
            if self.runtime.last_default_prevented {
                return None;
            }
        }
        if default_summary {
            if self.html_tag(id) == Some("details") && self.document.first_summary(id).is_none() {
                self.toggle_details(id);
            }
            return None;
        }
        if let Some(summary) = summary {
            if let Some(details) = self.document.summary_details(summary) {
                self.toggle_details(details);
            }
            return None;
        }
        if self.interaction_blocked(id) {
            return None;
        }
        let mut node = Some(id);
        while let Some(id) = node {
            if self.html_tag(id) == Some("input") {
                match self
                    .document
                    .attr(id, "type")
                    .unwrap_or("text")
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "checkbox" => {
                        if self.document.attr(id, "checked").is_some() {
                            self.document.remove_attr(id, "checked");
                        } else {
                            self.document.set_attr(id, "checked", "");
                        }
                        return None;
                    }
                    "radio" => {
                        let name = self.document.attr(id, "name").unwrap_or("").to_owned();
                        let form = self.ancestor_form(id);
                        for radio in self.document.query_selector_all("input") {
                            if self.html_tag(radio) == Some("input")
                                && self.is_active_node(radio)
                                && !name.is_empty()
                                && self
                                    .document
                                    .attr(radio, "type")
                                    .is_some_and(|kind| kind.eq_ignore_ascii_case("radio"))
                                && self.document.attr(radio, "name") == Some(&name)
                                && self.ancestor_form(radio) == form
                            {
                                self.document.remove_attr(radio, "checked");
                            }
                        }
                        self.document.set_attr(id, "checked", "");
                        return None;
                    }
                    "submit" => {
                        return self
                            .ancestor_form(id)
                            .and_then(|form| self.submit_form(form, Some(id)));
                    }
                    _ => return None,
                }
            }
            if self.html_tag(id) == Some("button") {
                if self
                    .document
                    .attr(id, "type")
                    .unwrap_or("submit")
                    .eq_ignore_ascii_case("submit")
                {
                    return self
                        .ancestor_form(id)
                        .and_then(|form| self.submit_form(form, Some(id)));
                }
                return None;
            }
            if self.html_tag(id) == Some("form") && original_id == id {
                return self.submit_form(id, None);
            }
            if self.html_tag(id) == Some("a")
                && let Some(href) = self.document.attr(id, "href")
            {
                return self.resolve_navigation(href).ok().map(Navigation::get);
            }
            node = self.document.nodes.get(id).and_then(|n| n.parent);
        }
        None
    }
    fn toggle_details(&mut self, id: NodeId) {
        if self.document.attr(id, "open").is_some() {
            self.document.remove_attr(id, "open");
        } else {
            self.document.set_attr(id, "open", "");
        }
    }
    pub fn can_edit_control(&self, node: NodeId) -> bool {
        self.document.can_edit_control(node)
    }
    fn ancestor_form(&self, mut node: NodeId) -> Option<NodeId> {
        for _ in 0..crate::dom::MAX_DEPTH {
            if self.html_tag(node) == Some("form") {
                return Some(node);
            }
            node = self.document.nodes.get(node)?.parent?;
        }
        None
    }
    pub fn submit_form(&mut self, form: NodeId, submitter: Option<NodeId>) -> Option<Navigation> {
        if self.html_tag(form) != Some("form")
            || self.interaction_blocked(form)
            || submitter.is_some_and(|node| {
                self.interaction_blocked(node) || self.ancestor_form(node) != Some(form)
            })
        {
            return None;
        }
        if self.scripts_enabled {
            if let Err(error) = self
                .runtime
                .dispatch_event(form, "submit", &mut self.document)
            {
                self.diagnostics.push(format!("submit: {error}"));
            }
            if self.runtime.last_default_prevented {
                return None;
            }
        }
        if !self.is_active_node(form) || self.interaction_blocked(form) {
            return None;
        }
        let method = submitter
            .and_then(|node| self.document.attr(node, "formmethod"))
            .or_else(|| self.document.attr(form, "method"))
            .unwrap_or("get")
            .to_ascii_lowercase();
        if !matches!(method.as_str(), "get" | "post") {
            self.diagnostics
                .push("form: only GET and POST submission are implemented".into());
            return None;
        }
        let encoding = submitter
            .and_then(|node| self.document.attr(node, "formenctype"))
            .or_else(|| self.document.attr(form, "enctype"))
            .unwrap_or("application/x-www-form-urlencoded");
        if method == "post" && !encoding.eq_ignore_ascii_case("application/x-www-form-urlencoded") {
            self.diagnostics
                .push("form: POST supports only application/x-www-form-urlencoded".into());
            return None;
        }
        let action = submitter
            .and_then(|node| self.document.attr(node, "formaction"))
            .or_else(|| self.document.attr(form, "action"))
            .filter(|action| !action.is_empty())
            .unwrap_or(self.url.as_str());
        let mut target = self
            .resolve_navigation(action)
            .ok()
            .and_then(|u| Url::parse(&u).ok())?;
        if method == "post" {
            if !matches!(target.scheme(), "http" | "https") {
                self.diagnostics
                    .push("form: POST submissions require HTTP or HTTPS".into());
                return None;
            }
            if self.url.scheme() == "https" && target.scheme() != "https" {
                self.diagnostics
                    .push("form: HTTPS downgrade submission blocked".into());
                return None;
            }
        }
        let mut pairs = Vec::new();
        let mut form_bytes = 0usize;
        for node in self
            .document
            .query_selector_all("input, textarea, select, button")
        {
            if self.html_tag(node).is_none()
                || !self.is_active_node(node)
                || self.ancestor_form(node) != Some(form)
                || self.disabled_control(node)
            {
                continue;
            }
            let Some(name) = self.document.attr(node, "name").filter(|s| !s.is_empty()) else {
                continue;
            };
            let kind = self
                .document
                .attr(node, "type")
                .unwrap_or("text")
                .to_ascii_lowercase();
            if matches!(kind.as_str(), "reset" | "button" | "file") {
                continue;
            }
            if (kind == "submit" || self.html_tag(node) == Some("button"))
                && submitter != Some(node)
            {
                continue;
            }
            if matches!(kind.as_str(), "checkbox" | "radio")
                && self.document.attr(node, "checked").is_none()
            {
                continue;
            }
            let value = if self.html_tag(node) == Some("textarea") {
                self.document.text_content(node)
            } else if self.html_tag(node) == Some("select") {
                let mut options = Vec::new();
                let mut pending = self.document.nodes[node].children.clone();
                pending.reverse();
                while let Some(child) = pending.pop() {
                    if self.html_tag(child) == Some("option") {
                        if !self.disabled_control(child) && self.is_active_node(child) {
                            options.push(child);
                        }
                    } else if self.html_tag(child) == Some("optgroup") {
                        pending.extend(self.document.nodes[child].children.iter().rev().copied());
                    }
                }
                let multiple = self.document.attr(node, "multiple").is_some();
                let selected = options
                    .iter()
                    .copied()
                    .filter(|&id| self.document.attr(id, "selected").is_some())
                    .collect::<Vec<_>>();
                let selected = if multiple {
                    selected
                } else {
                    selected
                        .first()
                        .or_else(|| options.first())
                        .copied()
                        .into_iter()
                        .collect()
                };
                for option in selected {
                    let value = self
                        .document
                        .attr(option, "value")
                        .map(str::to_owned)
                        .unwrap_or_else(|| self.document.text_content(option));
                    form_bytes += name.len() + value.len();
                    if form_bytes > MAX_FORM_BYTES {
                        self.diagnostics
                            .push("form submission byte budget exceeded".into());
                        return None;
                    }
                    pairs.push((name.to_owned(), value));
                }
                continue;
            } else {
                self.document
                    .attr(node, "value")
                    .unwrap_or(if matches!(kind.as_str(), "checkbox" | "radio") {
                        "on"
                    } else {
                        ""
                    })
                    .into()
            };
            form_bytes += name.len() + value.len();
            if form_bytes > MAX_FORM_BYTES {
                self.diagnostics
                    .push("form submission byte budget exceeded".into());
                return None;
            }
            pairs.push((name.to_owned(), value));
        }
        let body = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(
                pairs
                    .into_iter()
                    .map(|(name, value)| (form_line_breaks(&name), form_line_breaks(&value))),
            )
            .finish();
        if body.len() > MAX_FORM_BYTES {
            self.diagnostics
                .push("form submission byte budget exceeded".into());
            return None;
        }
        if method == "post" {
            Some(Navigation {
                address: target.to_string(),
                form_body: Some(body),
            })
        } else {
            target.set_query(Some(&body));
            Some(Navigation::get(target.to_string()))
        }
    }
    pub fn resolve_navigation(&self, address: &str) -> Result<String, String> {
        if address.len() > net::MAX_RESOURCE_BYTES * 2 {
            return Err("navigation URL exceeds size limit".into());
        }
        let target = crate::document_url::parse(
            &self.document,
            &crate::document_url::base_url(&self.document),
            address,
        )
        .map_err(|e| e.to_string())?;
        if !target.username().is_empty() || target.password().is_some() {
            return Err("URLs containing credentials are blocked".into());
        }
        let mut current = self.url.clone();
        let mut without_fragment = target.clone();
        current.set_fragment(None);
        without_fragment.set_fragment(None);
        if current == without_fragment {
            return Ok(target.to_string());
        }
        if target.scheme() == "file" && self.url.scheme() != "file" {
            return Err("remote page cannot navigate to local files".into());
        }
        if !matches!(target.scheme(), "http" | "https" | "file") {
            return Err("page requested unsupported navigation scheme".into());
        }
        if target.scheme() == "file" {
            let root = self
                .url
                .to_file_path()
                .map_err(|_| "invalid local URL")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            let path = target
                .to_file_path()
                .map_err(|_| "invalid local URL")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !root.parent().is_some_and(|r| path.starts_with(r)) {
                return Err("link escapes the opened document directory".into());
            }
        }
        Ok(target.to_string())
    }
}
pub(crate) fn same_document_url(left: &Url, right: &Url) -> bool {
    let mut left = left.clone();
    let mut right = right.clone();
    left.set_fragment(None);
    right.set_fragment(None);
    left == right
}
fn form_line_breaks(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            result.push_str("\r\n");
        } else if ch == '\n' {
            result.push_str("\r\n");
        } else {
            result.push(ch);
        }
    }
    result
}
fn diagnostic_url(value: &str) -> String {
    let mut result: String = value.chars().take(200).collect();
    if result.len() < value.len() {
        result.push('…');
    }
    result
}
pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn find_fragment(document: &Document, fragment: &str) -> Option<NodeId> {
    let bytes = net::percent_decode(fragment).ok()?;
    let fragment = String::from_utf8_lossy(&bytes);
    (0..document.nodes.len()).find(|&node| {
        document.attr(node, "id") == Some(fragment.as_ref())
            || document.attr(node, "name") == Some(fragment.as_ref())
    })
}
pub fn decode_image(bytes: &[u8]) -> Result<RasterImage, String> {
    decode_image_with_budget(bytes, MAX_DECODED_IMAGE_BYTES)
}
pub(crate) fn decode_image_with_budget(bytes: &[u8], budget: usize) -> Result<RasterImage, String> {
    use image::ImageDecoder;
    if budget < 4 {
        return Err("decoded image budget exhausted".into());
    }
    crate::image_limits::validate_webp(bytes, budget.min(MAX_DECODED_IMAGE_BYTES))?;
    let (mut decoder, mut limits) =
        bounded_image_decoder(bytes, MAX_DECODED_IMAGE_BYTES as u64).map_err(|e| e.to_string())?;
    let (width, height) = decoder.dimensions();
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || u64::from(width) * u64::from(height) * 4 > budget.min(MAX_DECODED_IMAGE_BYTES) as u64
    {
        return Err("decoded image exceeds geometry or remaining byte budget".into());
    }
    // Match ImageReader::decode's native-buffer reservation before conversion.
    limits
        .reserve(decoder.total_bytes())
        .map_err(|e| e.to_string())?;
    decoder.set_limits(limits).map_err(|e| e.to_string())?;
    let image = image::DynamicImage::from_decoder(decoder)
        .map_err(|e| e.to_string())?
        .into_rgba8();
    Ok(RasterImage {
        width: image.width(),
        height: image.height(),
        rgba: image.into_raw(),
    })
}

fn bounded_image_decoder(
    bytes: &[u8],
    max_alloc: u64,
) -> image::ImageResult<(impl image::ImageDecoder + '_, image::Limits)> {
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(max_alloc.min(MAX_DECODED_IMAGE_BYTES as u64));
    // Limits must precede decoder construction: PNG ancillary metadata may be
    // decompressed while reading dimensions. Reuse that configured decoder.
    reader.limits(limits.clone());
    Ok((reader.into_decoder()?, limits))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn idle_task_batches_finish_without_input_and_layout_never_dispatches() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            &format!(
                "<!doctype html><input id=n value=0 readonly>{}<script>let count=0;const n=document.getElementById('n');document.addEventListener('toggle',()=>{{count++;n.value=count;}},true);</script>",
                "<details open></details>".repeat(150)
            ),
            true,
        );
        let output = page.document.query_selector("#n").unwrap();
        assert_eq!(page.document.attr(output, "value").unwrap(), "64");
        assert_eq!(page.task_state(), TaskState::Pending);
        let fonts = Fonts::new();
        page.layout(320.0, 240.0, &fonts);
        page.layout(640.0, 480.0, &fonts);
        assert_eq!(page.document.attr(output, "value").unwrap(), "64");
        page.run_pending_tasks();
        assert_eq!(page.document.attr(output, "value").unwrap(), "128");
        assert_eq!(page.task_state(), TaskState::Pending);
        page.run_pending_tasks();
        assert_eq!(page.document.attr(output, "value").unwrap(), "150");
        assert_eq!(page.task_state(), TaskState::Idle);
        page.run_pending_tasks();
        assert_eq!(page.document.attr(output, "value").unwrap(), "150");
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    }
    #[test]
    fn task_quota_suspends_retries_without_losing_unstarted_work_or_svg_mutations() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            &format!(
                "<style>body{{margin:0}}details{{display:none}}svg{{position:absolute;left:0;top:40px}}</style><input id=n value=0 readonly>{}<svg width=20 height=10><rect id=r width=10 height=10 fill=red /></svg><script>let count=0;const n=document.getElementById('n');document.addEventListener('toggle',e=>{{count++;n.value=count;if(count===65){{e.target.open=false;const r=document.getElementById('r');r.setAttribute('fill','blue');r.setAttribute('width','20');while(true){{}}}}}},true);</script>",
                "<details open></details>".repeat(150)
            ),
            true,
        );
        let output = page.document.query_selector("#n").unwrap();
        assert_eq!(page.document.attr(output, "value").unwrap(), "64");
        assert_eq!(painted_inline_svg_pixels(&page), (0xff0000, 0xffffff));
        page.run_pending_tasks();
        assert_eq!(page.document.attr(output, "value").unwrap(), "65");
        assert_eq!(page.task_state(), TaskState::Suspended);
        assert!(page.document.has_pending_details_toggles());
        assert_eq!(painted_inline_svg_pixels(&page), (0x0000ff, 0x0000ff));
        let diagnostics = page.diagnostics.clone();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("suspended until reload"));
        for _ in 0..3 {
            page.run_pending_tasks();
            page.disclosure_checkpoint();
        }
        assert_eq!(page.document.attr(output, "value").unwrap(), "65");
        assert_eq!(page.diagnostics, diagnostics);
        assert!(page.document.has_pending_details_toggles());
    }
    #[test]
    fn persistent_allocation_failure_suspends_task_preflight_without_consuming_the_task() {
        let html = "<details id=d></details><input id=n value=untouched><script>const d=document.getElementById('d');const out=document.getElementById('n');d.ontoggle=()=>{out.value='ran';};</script>";
        let mut page = Page::from_html(Url::parse("https://example.test/").unwrap(), html, true);
        assert_eq!(page.task_state(), TaskState::Idle);
        page.runtime
            .execute("d.open=true", &mut page.document)
            .unwrap();
        assert_eq!(page.task_state(), TaskState::Pending);
        let queued = page.document.peek_details_toggle().unwrap();
        let output = page.document.query_selector("#n").unwrap();

        // Each execution gets a fresh instruction budget, but retained runtime
        // allocations share one document budget. Reach that budget through the
        // public interpreter instead of directly changing its private counters.
        let mut exhausted = false;
        for _ in 0..100 {
            if let Err(error) = page.runtime.execute("new Array(10000)", &mut page.document) {
                assert!(error.is_resource_limit(), "{error}");
                assert!(error.message.contains("allocation"), "{error}");
                exhausted = true;
                break;
            }
        }
        assert!(
            exhausted,
            "fixture must exhaust the persistent allocation budget"
        );
        assert!(page.diagnostics.is_empty());
        page.run_pending_tasks();
        assert_eq!(page.task_state(), TaskState::Suspended);
        assert_eq!(page.document.peek_details_toggle(), Some(queued));
        assert!(!page.document.details_toggle_running());
        assert_eq!(page.document.attr(output, "value"), Some("untouched"));
        let diagnostics = page.diagnostics.clone();
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains("allocation limit"));
        assert!(diagnostics[0].contains("suspended until reload"));
        for _ in 0..10 {
            page.run_pending_tasks();
            page.disclosure_checkpoint();
        }
        assert_eq!(page.document.peek_details_toggle(), Some(queued));
        assert!(!page.document.details_toggle_running());
        assert_eq!(page.document.attr(output, "value"), Some("untouched"));
        assert_eq!(page.diagnostics, diagnostics);

        let mut fresh = Page::from_html(Url::parse("https://example.test/").unwrap(), html, true);
        fresh
            .runtime
            .execute("d.open=true", &mut fresh.document)
            .unwrap();
        assert_eq!(fresh.task_state(), TaskState::Pending);
        fresh.run_pending_tasks();
        assert_eq!(fresh.task_state(), TaskState::Idle);
        assert!(!fresh.document.has_pending_details_toggles());
        let output = fresh.document.query_selector("#n").unwrap();
        assert_eq!(fresh.document.attr(output, "value"), Some("ran"));
        assert!(fresh.diagnostics.is_empty());
    }
    #[test]
    fn disabled_scripts_never_schedule_parse_generated_tasks() {
        for html in [
            "<details open></details>",
            "<meta http-equiv=Content-Security-Policy content=\"default-src 'none'\"><details open></details>",
        ] {
            for scripts in [false, true] {
                if scripts && !html.starts_with("<meta") {
                    continue;
                }
                let mut page =
                    Page::from_html(Url::parse("https://example.test/").unwrap(), html, scripts);
                assert!(page.document.has_pending_details_toggles());
                assert_eq!(page.task_state(), TaskState::Idle);
                page.run_pending_tasks();
                assert!(page.document.has_pending_details_toggles());
                assert_eq!(page.task_state(), TaskState::Idle);
            }
        }
    }
    fn painted_inline_svg_pixels(page: &Page) -> (u32, u32) {
        let fonts = Fonts::new();
        let layout = page.layout(80.0, 80.0, &fonts);
        let mut canvas = crate::graphics::Canvas::new(80, 80).unwrap();
        canvas.clear(crate::graphics::Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        (canvas.pixels[45 * 80 + 5], canvas.pixels[45 * 80 + 15])
    }
    #[test]
    fn details_toggle_callbacks_refresh_inline_svg_after_mutation_and_errors() {
        for summary in ["<summary id=s>Show</summary>", ""] {
            for suffix in ["", "throw new Error('handled');", "while(true){}"] {
                let mut page = Page::from_html(
                    Url::parse("https://example.test/").unwrap(),
                    &format!(
                        "<style>body{{margin:0}}svg{{position:absolute;left:0;top:40px}}</style><details id=d>{summary}content</details><svg width=20 height=10><rect id=r width=10 height=10 fill=red /></svg><script>document.getElementById('d').ontoggle=()=>{{const r=document.getElementById('r');r.setAttribute('fill','blue');r.setAttribute('width','20');{suffix}}};</script>"
                    ),
                    true,
                );
                assert_eq!(painted_inline_svg_pixels(&page), (0xff0000, 0xffffff));
                let details = page.document.query_selector("#d").unwrap();
                if let Some(summary) = page.document.query_selector("#s") {
                    page.click(summary);
                } else {
                    page.click_default_summary(details);
                }
                assert!(page.document.attr(details, "open").is_some());
                assert_eq!(
                    painted_inline_svg_pixels(&page),
                    (0x0000ff, 0x0000ff),
                    "summary={summary:?}, callback suffix={suffix:?}"
                );
            }
        }
    }
    #[test]
    fn input_callbacks_and_their_toggle_checkpoint_refresh_inline_svg() {
        for toggle in [false, true] {
            for suffix in ["", "throw new Error('handled');", "while(true){}"] {
                let action = if toggle {
                    "document.getElementById('d').open=true;"
                } else {
                    "update();"
                };
                let mut page = Page::from_html(
                    Url::parse("https://example.test/").unwrap(),
                    &format!(
                        "<style>body{{margin:0}}svg{{position:absolute;left:0;top:40px}}</style><input id=i><details id=d><summary>Show</summary>content</details><svg width=20 height=10><rect id=r width=10 height=10 fill=red /></svg><script>function update(){{const r=document.getElementById('r');r.setAttribute('fill','blue');r.setAttribute('width','20');{suffix}}}document.getElementById('d').ontoggle=update;document.getElementById('i').oninput=()=>{{{action}}};</script>"
                    ),
                    true,
                );
                assert_eq!(painted_inline_svg_pixels(&page), (0xff0000, 0xffffff));
                let input = page.document.query_selector("#i").unwrap();
                crate::worker::apply_edit(&mut page, input, "edited");
                assert_eq!(page.document.attr(input, "value"), Some("edited"));
                assert_eq!(
                    painted_inline_svg_pixels(&page),
                    (0x0000ff, 0x0000ff),
                    "toggle={toggle}, callback suffix={suffix:?}"
                );
            }
        }
    }
    #[test]
    fn details_activation_cancellation_grouping_and_event_order() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<details id=a name=g open><summary>A</summary></details><details id=b name=g><summary id=s><span id=target>B</span></summary><input id=inside></details><p id=log></p><script>const a=document.getElementById('a');const b=document.getElementById('b');const s=document.getElementById('s');const log=document.getElementById('log');s.addEventListener('click',()=>{log.textContent+='click:'+b.open+';';});b.addEventListener('toggle',e=>{log.textContent+=e.oldState+'>'+e.newState+';';});</script>",
            true,
        );
        let a = page.document.query_selector("#a").unwrap();
        let b = page.document.query_selector("#b").unwrap();
        let target = page.document.query_selector("#target").unwrap();
        let log = page.document.query_selector("#log").unwrap();
        page.click(target);
        assert!(page.document.attr(a, "open").is_none());
        assert!(page.document.attr(b, "open").is_some());
        assert_eq!(page.document.text_content(log), "click:false;closed>open;");
        page.runtime
            .execute("s.onclick=e=>e.preventDefault();", &mut page.document)
            .unwrap();
        page.click(target);
        assert!(page.document.attr(b, "open").is_some());
        page.runtime
            .execute("s.onclick=e=>e.stopPropagation();", &mut page.document)
            .unwrap();
        page.click(target);
        assert!(page.document.attr(b, "open").is_none());
    }
    #[test]
    fn details_activation_keeps_original_summary_when_listener_moves_the_target() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<details id=a><summary id=s><span id=t>A</span></summary></details><details id=b><summary id=other>B</summary></details><script>const t=document.getElementById('t');t.onclick=()=>document.getElementById('other').appendChild(t);</script>",
            true,
        );
        let target = page.document.query_selector("#t").unwrap();
        page.click(target);
        assert!(
            page.document
                .attr(page.document.query_selector("#a").unwrap(), "open")
                .is_some()
        );
        assert!(
            page.document
                .attr(page.document.query_selector("#b").unwrap(), "open")
                .is_none()
        );
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<details id=a><summary id=s><span id=t>A</span></summary></details><script>const t=document.getElementById('t');t.onclick=()=>t.remove();</script>",
            true,
        );
        page.click(page.document.query_selector("#t").unwrap());
        assert!(
            page.document
                .attr(page.document.query_selector("#a").unwrap(), "open")
                .is_some()
        );
    }
    #[test]
    fn details_nested_controls_and_background_do_not_activate_a_summary() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<details id=d><summary id=s><a id=link href='/next'>Go</a><button id=button type=button>button</button><input id=check type=checkbox></summary><input id=closed></details><details id=fallback>hidden</details>",
            false,
        );
        let details = page.document.query_selector("#d").unwrap();
        let link = page.document.query_selector("#link").unwrap();
        assert!(page.click(link).unwrap().address.ends_with("/next"));
        for selector in ["#button", "#check", "#d"] {
            page.click(page.document.query_selector(selector).unwrap());
        }
        assert!(page.document.attr(details, "open").is_none());
        assert!(
            page.document
                .attr(page.document.query_selector("#check").unwrap(), "checked")
                .is_some()
        );
        let fallback = page.document.query_selector("#fallback").unwrap();
        page.click(fallback);
        assert!(page.document.attr(fallback, "open").is_none());
        page.click_default_summary(fallback);
        assert!(page.document.attr(fallback, "open").is_some());
        page.click_default_summary(details);
        assert!(
            page.document.attr(details, "open").is_none(),
            "forged fallback action is ignored"
        );
    }
    #[test]
    fn details_closed_contents_still_run_scripts_supply_styles_and_submit_controls() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<p id=out>before</p><form id=form action='/save'><details><summary>Show</summary><input id=value name=answer value=42><style>#out{color:red}</style><script>document.getElementById('out').textContent='ran';</script></details></form>",
            true,
        );
        assert_eq!(
            page.document
                .text_content(page.document.query_selector("#out").unwrap()),
            "ran"
        );
        assert!(
            page.stylesheets()
                .iter()
                .any(|source| source.source.contains("#out"))
        );
        let field = page.document.query_selector("#value").unwrap();
        assert!(!page.can_edit_control(field));
        let navigation = page
            .submit_form(page.document.query_selector("#form").unwrap(), None)
            .unwrap();
        assert!(navigation.address.ends_with("/save?answer=42"));
    }
    #[test]
    fn inert_style_metadata_is_checked_before_any_text_collection() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<svg><style id=media media='screen }'><style id=type type='text/css; charset=utf-8'></style></style></svg>",
            false,
        );
        let media = page.document.query_selector("#media").unwrap();
        let wrong_type = page.document.query_selector("#type").unwrap();
        page.document
            .set_text_content(wrong_type, &"x".repeat(4096));
        let mut bytes = 0;
        let mut visits = 0;
        for id in [media, wrong_type] {
            assert_eq!(
                page.inline_style_source(id, &mut bytes, &mut visits),
                Ok(None)
            );
            assert_eq!((bytes, visits), (0, 0));
        }
        assert!(page.stylesheets().is_empty());
        page.document.remove_attr(wrong_type, "type");
        assert!(
            page.inline_style_source(wrong_type, &mut bytes, &mut visits)
                .is_err()
        );
    }
    #[test]
    fn html_and_svg_style_use_direct_child_text_without_changing_text_content() {
        for namespace in [Namespace::Html, Namespace::Svg] {
            let mut page = Page::from_html(
                Url::parse("https://example.test/").unwrap(),
                "<p id=x>text</p>",
                false,
            );
            let style = page.document.create_element_ns(namespace, "style");
            page.document.set_text_content(style, "p{color:green}");
            let child = page.document.create_element_ns(namespace, "g");
            page.document.set_text_content(child, "p{color:red}");
            page.document.append_child(style, child);
            let body = page.document.query_selector("body").unwrap();
            page.document.append_child(body, style);
            assert_eq!(
                page.document.text_content(style),
                "p{color:green}p{color:red}"
            );
            let sources = page.stylesheets();
            assert_eq!(sources.len(), 1);
            assert_eq!(sources[0].source.as_ref(), "p{color:green}");
            let styles = css::compute_styles_from_sources(&page.document, &sources, 400.0, 300.0);
            assert_eq!(
                styles[page.document.query_selector("#x").unwrap()].color,
                crate::graphics::Color::rgb(0, 128, 0)
            );
        }
    }
    #[test]
    fn initial_loading_and_recollection_share_style_type_and_text_rules() {
        // Initial loading caches only accepted direct child text. Invalid style
        // metadata must not even send an otherwise invalid import to the loader.
        let source = "<style type='text/css; charset=utf-8'>@import 'http://[bad';</style><style type=' text/css'>@import 'http://[bad';</style><style media='screen }'>@import 'http://[bad';</style><svg><style id=outer><style id=inner>p{color:green}</style></style></svg><p id=x>text</p>";
        let encoded: String = url::form_urlencoded::byte_serialize(source.as_bytes()).collect();
        // Form encoding represents spaces as '+', while data URLs preserve '+'.
        let address = format!("data:text/html,{}", encoded.replace('+', "%20"));
        let page = Page::load(&address, false).unwrap();
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        let outer = page.document.query_selector("#outer").unwrap();
        let inner = page.document.query_selector("#inner").unwrap();
        assert_eq!(page.inline_styles.len(), 2);
        assert_eq!(page.inline_styles[&outer].0.as_ref(), "");
        assert_eq!(page.inline_styles[&inner].0.as_ref(), "p{color:green}");
        let sources = page.stylesheets();
        assert_eq!(
            sources
                .iter()
                .map(|source| source.source.len())
                .sum::<usize>(),
            "p{color:green}".len()
        );
        let styles = css::compute_styles_from_sources(&page.document, &sources, 400.0, 300.0);
        assert_eq!(
            styles[page.document.query_selector("#x").unwrap()].color,
            crate::graphics::Color::rgb(0, 128, 0)
        );
    }
    #[test]
    fn recollecting_inline_styles_has_shared_byte_and_count_limits() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<style id=a></style><style id=b></style><style id=c>overflow</style>",
            false,
        );
        let text = " ".repeat(MAX_STYLE_BYTES / 2);
        for selector in ["#a", "#b"] {
            page.document
                .set_text_content(page.document.query_selector(selector).unwrap(), &text);
        }
        let sources = page.stylesheets();
        assert_eq!(sources.len(), 2);
        assert_eq!(
            sources
                .iter()
                .map(|source| source.source.len())
                .sum::<usize>(),
            MAX_STYLE_BYTES
        );
        let page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            &"<style>x</style>".repeat(crate::dom::MAX_INLINE_STYLES + 1),
            false,
        );
        assert_eq!(page.stylesheets().len(), crate::dom::MAX_INLINE_STYLES);
    }
    #[test]
    fn png_ancillary_metadata_obeys_limits_during_decoder_construction() {
        use image::ImageDecoder;
        let icc = include_bytes!("../tests/fixtures/png-ancillary-icc-4k.png");
        let exif = include_bytes!("../tests/fixtures/png-ancillary-exif-4k.png");
        let (mut full, _) = bounded_image_decoder(icc, 16384).unwrap();
        assert_eq!(full.icc_profile().unwrap().unwrap().len(), 4096);
        let (mut limited, _) = bounded_image_decoder(icc, 1024).unwrap();
        assert!(
            limited.icc_profile().unwrap().is_none(),
            "over-budget optional ICC metadata must be discarded"
        );
        let (mut full, _) = bounded_image_decoder(exif, 16384).unwrap();
        assert_eq!(full.exif_metadata().unwrap().unwrap().len(), 4096);
        assert!(matches!(
            bounded_image_decoder(exif, 1024),
            Err(image::ImageError::Limits(_))
        ));
    }
    #[test]
    fn image_decoders_enforce_remaining_pixel_budget_before_raster_allocation() {
        let mut encoded = Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(3, 2, image::Rgba([11, 22, 33, 255]))
            .write_to(&mut encoded, image::ImageFormat::Png)
            .unwrap();
        let bytes = encoded.into_inner();
        assert_eq!(decode_image_with_budget(&bytes, 24).unwrap().rgba.len(), 24);
        assert!(decode_image_with_budget(&bytes, 23).is_err());
        let svg = "<svg width='3' height='2'><rect width='3' height='2'/></svg>";
        assert_eq!(
            crate::svg::render_with_budget(svg, None, None, 24)
                .unwrap()
                .rgba
                .len(),
            24
        );
        assert!(crate::svg::render_with_budget(svg, None, None, 23).is_err());
        for source in [b"{\"secret\":42}".as_slice(), b"<html>private</html>"] {
            assert!(decode_image_with_budget(source, 64).is_err());
        }
    }
    #[test]
    fn builtin_pages_preserve_query_and_fragment_without_resource_fetches() {
        for address in [
            "eris:home#section",
            "eris:home?theme=dark#section",
            "about:blank#target",
            "about:blank?x=1#target",
        ] {
            let page = Page::load(address, false).unwrap();
            assert_eq!(page.url.as_str(), address);
            assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
            if address.starts_with("about:") {
                assert_eq!(page.title(), "Blank");
            }
        }
        assert!(Page::load("about:unsupported#target", false).is_err());
        assert!(Page::load("eris:unsupported?x=1", false).is_err());
    }
    #[test]
    fn inert_ancestors_prevent_text_editing_and_detached_forms_do_not_submit() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<div inert><input id=blocked><textarea id=blocked-text></textarea></div><input id=editable><div id=holder><form method=post onsubmit=\"document.getElementById('holder').textContent='removed'\"><input name=private value=test><button>Send</button></form></div>",
            true,
        );
        assert!(!page.can_edit_control(page.document.query_selector("#blocked").unwrap()));
        assert!(!page.can_edit_control(page.document.query_selector("#blocked-text").unwrap()));
        assert!(page.can_edit_control(page.document.query_selector("#editable").unwrap()));
        assert!(
            page.click(page.document.query_selector("button").unwrap())
                .is_none()
        );
        assert_eq!(
            page.document
                .text_content(page.document.query_selector("#holder").unwrap()),
            "removed"
        );
    }
    #[test]
    fn page_cannot_navigate_to_files_or_javascript() {
        let p = Page::from_html(
            Url::parse("https://example.com/").unwrap(),
            "<a href='file:///etc/passwd'>bad</a>",
            false,
        );
        assert!(p.resolve_navigation("file:///etc/passwd").is_err());
        assert!(p.resolve_navigation("javascript:alert(1)").is_err());
        assert_eq!(p.resolve_navigation("/a").unwrap(), "https://example.com/a");
    }
    #[test]
    fn invalid_media_wrappers_and_non_css_types_cannot_introduce_rules() {
        let page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<style>p{color:green}</style><style media=\"print {} p {color:red} @media screen\">p{background:blue}</style><style type=text/plain>p{color:red}</style><style media=print>} p{color:red}</style><p id=x>text</p>",
            false,
        );
        let x = page.document.query_selector("#x").unwrap();
        let styles =
            css::compute_styles_from_sources(&page.document, &page.stylesheets(), 400.0, 300.0);
        assert_eq!(styles[x].color, crate::graphics::Color::rgb(0, 128, 0));
        assert_eq!(styles[x].background_color.a, 0);
    }
    #[test]
    fn plaintext_escaping() {
        assert_eq!(escape_html("<script>&\""), "&lt;script&gt;&amp;&quot;");
    }
    #[test]
    fn foreign_elements_do_not_acquire_html_script_form_or_metadata_behavior() {
        let mut page = Page::from_html(
            Url::parse("https://example.test/").unwrap(),
            "<svg><title>SVG label</title><script>document.title='wrong';</script><form id=foreign-form><input id=foreign-input name=leak value=no /></form></svg><title>HTML title</title><p id=result>ready</p><script>document.getElementById('result').textContent='html ran';</script>",
            true,
        );
        assert_eq!(page.title(), "HTML title");
        assert_eq!(
            page.document
                .text_content(page.document.query_selector("#result").unwrap()),
            "html ran"
        );
        let input = page.document.query_selector("#foreign-input").unwrap();
        let form = page.document.query_selector("#foreign-form").unwrap();
        assert_eq!(page.document.namespace(input), Some(Namespace::Svg));
        assert!(!page.can_edit_control(input));
        assert!(page.submit_form(form, None).is_none());
        let body = page.document.query_selector("body").unwrap();
        let html_svg = page.document.create_element("svg");
        page.document.append_child(body, html_svg);
        page.refresh_inline_svg();
        assert!(
            !page
                .images
                .contains_key(&format!("eris-inline-svg:{html_svg}"))
        );
    }
    #[test]
    fn repeated_cached_stylesheets_have_a_bounded_cascade_input() {
        let mut page = Page::from_html(
            Url::parse("https://example.test").unwrap(),
            &"<link rel=stylesheet href=large.css>".repeat(300),
            false,
        );
        let source: Arc<str> = " ".repeat(64 * 1024).into();
        for id in page.document.query_selector_all("link") {
            page.external_styles
                .insert(id, vec![css::StyleSource::new(source.clone())]);
        }
        let stylesheets = page.stylesheets();
        assert_eq!(
            stylesheets
                .iter()
                .map(|sheet| sheet.source.len())
                .sum::<usize>(),
            MAX_STYLE_BYTES
        );
        assert_eq!(stylesheets.len(), 128);
    }
}
