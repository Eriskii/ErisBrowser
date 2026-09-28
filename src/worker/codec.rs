//! Versioned length-prefixed protocol; no generic deserializer can allocate an
//! attacker-declared collection before checking its count and remaining bytes.
use super::{Command, Init, Reply, Snapshot};
use crate::{
    dom::{
        AttributeNamespace, Doctype, Document, DocumentMode, Element, MAX_DOM_BYTES, MAX_NODES,
        Namespace, Node, NodeKind,
    },
    graphics::{Color, DrawCommand, ImageStore, RasterImage, Rect},
    layout::{HitAction, HitRegion, LayoutResult},
    page::Navigation,
};
use std::{
    collections::{BTreeMap, HashMap},
    io::{Read, Write},
    path::PathBuf,
    sync::Arc,
};
pub(super) const MAX_FRAME: usize = 128 * 1024 * 1024;
pub(super) const MAX_REQUEST: usize = 18 * 1024 * 1024;
const MAX_STRING: usize = 16 * 1024 * 1024;
const MAX_IMAGES: usize = 64 * 1024 * 1024;
const MAX_COMMANDS: usize = 200_000;
const MAGIC: &[u8] = b"ERW7";
type Result<T> = std::result::Result<T, String>;

pub(super) fn read_frame(input: &mut impl Read, limit: usize) -> Result<Vec<u8>> {
    let mut length = [0; 4];
    input.read_exact(&mut length).map_err(|e| e.to_string())?;
    let length = u32::from_le_bytes(length) as usize;
    if length < 5 || length > limit {
        return Err("IPC frame length outside limit".into());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| "IPC allocation failed")?;
    bytes.resize(length, 0);
    input.read_exact(&mut bytes).map_err(|e| e.to_string())?;
    Ok(bytes)
}
pub(super) fn write_frame(output: &mut impl Write, bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_FRAME {
        return Err("IPC frame exceeds limit".into());
    }
    output
        .write_all(&(bytes.len() as u32).to_le_bytes())
        .and_then(|()| output.write_all(bytes))
        .and_then(|()| output.flush())
        .map_err(|e| e.to_string())
}
struct Encoder {
    bytes: Vec<u8>,
    failed: bool,
}
impl Encoder {
    fn new(tag: u8) -> Self {
        let mut e = Self {
            bytes: Vec::new(),
            failed: false,
        };
        e.raw(MAGIC);
        e.byte(tag);
        e
    }
    fn raw(&mut self, bytes: &[u8]) {
        if self.failed || bytes.len() > MAX_FRAME.saturating_sub(self.bytes.len()) {
            self.failed = true;
            return;
        }
        if self.bytes.try_reserve(bytes.len()).is_err() {
            self.failed = true;
            return;
        }
        self.bytes.extend_from_slice(bytes);
    }
    fn byte(&mut self, n: u8) {
        self.raw(&[n]);
    }
    fn boolean(&mut self, b: bool) {
        self.byte(u8::from(b));
    }
    fn u32(&mut self, n: usize) {
        if n > u32::MAX as usize {
            self.failed = true;
        }
        self.raw(&(n as u32).to_le_bytes());
    }
    fn u64(&mut self, n: u64) {
        self.raw(&n.to_le_bytes());
    }
    fn f32(&mut self, n: f32) {
        self.raw(&n.to_le_bytes());
    }
    fn f64(&mut self, n: f64) {
        self.raw(&n.to_le_bytes());
    }
    fn string(&mut self, s: &str) {
        self.u32(s.len());
        self.raw(s.as_bytes());
    }
    fn optional_string(&mut self, s: Option<&str>) {
        self.boolean(s.is_some());
        if let Some(s) = s {
            self.string(s);
        }
    }
    fn rect(&mut self, r: Rect) {
        for n in [r.x, r.y, r.width, r.height] {
            self.f32(n);
        }
    }
    fn color(&mut self, c: Color) {
        self.raw(&[c.r, c.g, c.b, c.a]);
    }
    fn navigation(&mut self, n: &Navigation) {
        self.string(&n.address);
        self.optional_string(n.form_body.as_deref());
    }
    fn finish(self) -> Result<Vec<u8>> {
        if self.failed {
            Err("IPC output budget exceeded".into())
        } else {
            Ok(self.bytes)
        }
    }
}
struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8], tag: u8) -> Result<Self> {
        if bytes.len() > MAX_FRAME || bytes.get(..4) != Some(MAGIC) || bytes.get(4) != Some(&tag) {
            return Err("invalid IPC version/message kind".into());
        }
        Ok(Self { bytes, offset: 5 })
    }
    fn raw(&mut self, length: usize) -> Result<&'a [u8]> {
        if length > self.bytes.len().saturating_sub(self.offset) {
            return Err("truncated IPC message".into());
        }
        let bytes = &self.bytes[self.offset..self.offset + length];
        self.offset += length;
        Ok(bytes)
    }
    fn byte(&mut self) -> Result<u8> {
        Ok(self.raw(1)?[0])
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid IPC boolean".into()),
        }
    }
    fn count(&mut self, maximum: usize) -> Result<usize> {
        let n = u32::from_le_bytes(self.raw(4)?.try_into().map_err(|_| "invalid u32")?) as usize;
        if n > maximum {
            Err("IPC collection limit exceeded".into())
        } else {
            Ok(n)
        }
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.raw(8)?.try_into().map_err(|_| "invalid u64")?,
        ))
    }
    fn f32(&mut self) -> Result<f32> {
        let n = f32::from_le_bytes(self.raw(4)?.try_into().map_err(|_| "invalid f32")?);
        if !n.is_finite() || n.abs() > 16_000_000.0 {
            Err("invalid IPC geometry".into())
        } else {
            Ok(n)
        }
    }
    fn f64(&mut self) -> Result<f64> {
        let n = f64::from_le_bytes(self.raw(8)?.try_into().map_err(|_| "invalid f64")?);
        if !n.is_finite() || !(0.0..=86_400_000.0).contains(&n) {
            Err("invalid IPC timing".into())
        } else {
            Ok(n)
        }
    }
    fn string(&mut self, maximum: usize) -> Result<String> {
        let length = self.count(maximum.min(MAX_STRING))?;
        Ok(std::str::from_utf8(self.raw(length)?)
            .map_err(|_| "invalid IPC UTF-8")?
            .to_owned())
    }
    fn budget_string(&mut self, budget: &mut usize) -> Result<String> {
        // DOM character data can exceed the metadata limit after HTML replaces
        // raw-text NULs with U+FFFD. Its shared retained-byte cap still applies.
        let length = self.count((*budget).min(MAX_DOM_BYTES))?;
        let s = std::str::from_utf8(self.raw(length)?)
            .map_err(|_| "invalid IPC UTF-8")?
            .to_owned();
        *budget -= s.len();
        Ok(s)
    }
    fn optional_string(&mut self, maximum: usize) -> Result<Option<String>> {
        if self.boolean()? {
            Ok(Some(self.string(maximum)?))
        } else {
            Ok(None)
        }
    }
    fn rect(&mut self) -> Result<Rect> {
        let r = Rect {
            x: self.f32()?,
            y: self.f32()?,
            width: self.f32()?,
            height: self.f32()?,
        };
        if r.width < 0.0 || r.height < 0.0 {
            Err("negative IPC rectangle size".into())
        } else {
            Ok(r)
        }
    }
    fn color(&mut self) -> Result<Color> {
        let c = self.raw(4)?;
        Ok(Color::rgba(c[0], c[1], c[2], c[3]))
    }
    fn navigation(&mut self) -> Result<Navigation> {
        Ok(Navigation {
            address: self.string(MAX_STRING)?,
            form_body: self.optional_string(crate::net::MAX_FORM_BODY_BYTES)?,
        })
    }
    fn end(self) -> Result<()> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err("trailing IPC bytes".into())
        }
    }
}

pub(super) fn encode_init(init: &Init) -> Result<Vec<u8>> {
    let mut e = Encoder::new(0);
    e.boolean(init.scripts);
    e.u64(init.generation);
    e.finish()
}
pub(super) fn decode_init(bytes: &[u8]) -> Result<Init> {
    let mut d = Decoder::new(bytes, 0)?;
    let init = Init {
        scripts: d.boolean()?,
        generation: d.u64()?,
    };
    d.end()?;
    Ok(init)
}
pub(super) fn encode_command(command: &Command) -> Result<Vec<u8>> {
    let mut e = Encoder::new(1);
    match command {
        Command::Load { navigation } => {
            e.byte(0);
            e.navigation(navigation);
        }
        Command::Click { node } => {
            e.byte(1);
            e.u32(*node);
        }
        Command::DefaultSummary { node } => {
            e.byte(5);
            e.u32(*node);
        }
        Command::Edit {
            sequence,
            node,
            value,
        } => {
            e.byte(2);
            e.u64(*sequence);
            e.u32(*node);
            e.string(value);
        }
        Command::Fragment { address } => {
            e.byte(3);
            e.string(address);
        }
        Command::Render { width, height } => {
            e.byte(4);
            e.f32(*width);
            e.f32(*height);
        }
    }
    let bytes = e.finish()?;
    if bytes.len() > MAX_REQUEST {
        return Err("IPC request exceeds limit".into());
    }
    Ok(bytes)
}
pub(super) fn decode_command(bytes: &[u8]) -> Result<Command> {
    let mut d = Decoder::new(bytes, 1)?;
    let command = match d.byte()? {
        0 => Command::Load {
            navigation: d.navigation()?,
        },
        1 => Command::Click {
            node: d.count(MAX_NODES - 1)?,
        },
        2 => Command::Edit {
            sequence: d.u64()?,
            node: d.count(MAX_NODES - 1)?,
            value: d.string(65_536)?,
        },
        3 => Command::Fragment {
            address: d.string(MAX_STRING)?,
        },
        4 => Command::Render {
            width: d.f32()?,
            height: d.f32()?,
        },
        5 => Command::DefaultSummary {
            node: d.count(MAX_NODES - 1)?,
        },
        _ => return Err("unknown page command".into()),
    };
    d.end()?;
    Ok(command)
}
pub(super) fn encode_error(error: &str) -> Result<Vec<u8>> {
    let mut e = Encoder::new(2);
    e.boolean(false);
    e.string(&error.chars().take(2048).collect::<String>());
    e.finish()
}
pub(super) fn encode_reply(reply: &Reply) -> Result<Vec<u8>> {
    let mut e = Encoder::new(2);
    e.boolean(true);
    e.boolean(reply.navigation.is_some());
    if let Some(n) = &reply.navigation {
        e.navigation(n);
    }
    e.boolean(reply.snapshot.is_some());
    if let Some(s) = &reply.snapshot {
        encode_snapshot(&mut e, s);
    }
    e.finish()
}
pub(super) fn decode_reply(bytes: &[u8]) -> Result<Reply> {
    let mut d = Decoder::new(bytes, 2)?;
    if !d.boolean()? {
        let error = d.string(8192)?;
        d.end()?;
        return Err(error);
    }
    let navigation = if d.boolean()? {
        Some(d.navigation()?)
    } else {
        None
    };
    let snapshot = if d.boolean()? {
        Some(decode_snapshot(&mut d)?)
    } else {
        None
    };
    d.end()?;
    Ok(Reply {
        snapshot,
        navigation,
    })
}
fn encode_snapshot(e: &mut Encoder, s: &Snapshot) {
    e.u64(s.generation);
    e.u64(s.processed_edit_sequence);
    e.string(&s.title);
    e.string(&s.url);
    e.f64(s.load_ms);
    e.u32(s.diagnostics.len());
    for diagnostic in &s.diagnostics {
        e.string(diagnostic);
    }
    e.u32(s.document.root);
    e.boolean(s.document.scripting_enabled());
    e.byte(match s.document.mode() {
        DocumentMode::NoQuirks => 0,
        DocumentMode::LimitedQuirks => 1,
        DocumentMode::Quirks => 2,
    });
    e.string(s.document.character_set());
    e.boolean(s.document.frozen_base().is_some());
    if let Some((node, url)) = s.document.frozen_base() {
        e.u32(node);
        e.string(url.as_str());
    }
    e.u32(s.document.nodes.len());
    for node in &s.document.nodes {
        e.boolean(node.parent.is_some());
        if let Some(parent) = node.parent {
            e.u32(parent);
        }
        e.u32(node.children.len());
        for &child in &node.children {
            e.u32(child);
        }
        match &node.kind {
            NodeKind::Document => e.byte(0),
            NodeKind::DocumentFragment { host } => {
                e.byte(6);
                e.boolean(host.is_some());
                if let Some(host) = host {
                    e.u32(*host);
                }
            }
            NodeKind::Element(el) => {
                e.byte(1);
                e.byte(match el.namespace {
                    Namespace::Html => 0,
                    Namespace::Svg => 1,
                    Namespace::MathMl => 2,
                });
                e.string(&el.tag);
                e.u32(el.attrs.len());
                for (k, v) in &el.attrs {
                    e.string(k);
                    e.string(v);
                }
                e.u32(el.attr_namespaces.len());
                for (name, namespace) in &el.attr_namespaces {
                    e.string(name);
                    e.byte(match namespace {
                        AttributeNamespace::XLink => 0,
                        AttributeNamespace::Xml => 1,
                        AttributeNamespace::Xmlns => 2,
                    });
                }
                e.boolean(el.template_contents.is_some());
                if let Some(contents) = el.template_contents {
                    e.u32(contents);
                }
            }
            NodeKind::Text(s) => {
                e.byte(2);
                e.string(s);
            }
            NodeKind::Comment(s) => {
                e.byte(3);
                e.string(s);
            }
            NodeKind::Doctype(d) => {
                e.byte(4);
                e.string(&d.name);
                e.optional_string(d.public_id.as_deref());
                e.optional_string(d.system_id.as_deref());
                e.boolean(d.force_quirks);
            }
            NodeKind::ProcessingInstruction { target, data } => {
                e.byte(5);
                e.string(target);
                e.string(data);
            }
        }
    }
    // Multiple element keys can refer to one decoded raster. Preserve sharing.
    let mut indices = HashMap::new();
    let mut rasters = Vec::new();
    for image in s.images.values() {
        let pointer = Arc::as_ptr(image);
        if let std::collections::hash_map::Entry::Vacant(entry) = indices.entry(pointer) {
            entry.insert(rasters.len());
            rasters.push(image);
        }
    }
    e.u32(rasters.len());
    for image in rasters {
        e.u32(image.width as usize);
        e.u32(image.height as usize);
        e.u32(image.rgba.len());
        e.raw(&image.rgba);
    }
    e.u32(s.images.len());
    for (key, image) in &s.images {
        e.string(key);
        e.u32(indices[&Arc::as_ptr(image)]);
    }
    e.f32(s.layout.content_height);
    e.u32(s.layout.hit_regions.len());
    for hit in &s.layout.hit_regions {
        e.u32(hit.node);
        e.rect(hit.rect);
        e.boolean(hit.fixed);
        e.byte(match hit.action {
            HitAction::Node => 0,
            HitAction::DefaultSummary => 1,
        });
    }
    e.u32(s.layout.commands.len());
    for command in &s.layout.commands {
        match command {
            DrawCommand::PushClip { rect } => {
                e.byte(0);
                e.rect(*rect);
            }
            DrawCommand::PopClip => e.byte(1),
            DrawCommand::PushFixed => e.byte(6),
            DrawCommand::PopFixed => e.byte(7),
            DrawCommand::PushOpacity { opacity } => {
                e.byte(8);
                e.f32(*opacity);
            }
            DrawCommand::PopOpacity => e.byte(9),
            DrawCommand::Rect {
                rect,
                color,
                radius,
            } => {
                e.byte(2);
                e.rect(*rect);
                e.color(*color);
                e.f32(*radius);
            }
            DrawCommand::Text {
                x,
                y,
                text,
                size,
                color,
                bold,
                italic,
                monospace,
            } => {
                e.byte(3);
                e.f32(*x);
                e.f32(*y);
                e.string(text);
                e.f32(*size);
                e.color(*color);
                e.boolean(*bold);
                e.boolean(*italic);
                e.boolean(*monospace);
            }
            DrawCommand::Image { rect, key } => {
                e.byte(4);
                e.rect(*rect);
                e.string(key);
            }
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => {
                e.byte(5);
                for n in [x1, y1, x2, y2] {
                    e.f32(*n);
                }
                e.color(*color);
                e.f32(*width);
            }
        }
    }
}
fn decode_snapshot(d: &mut Decoder<'_>) -> Result<Snapshot> {
    let generation = d.u64()?;
    let processed_edit_sequence = d.u64()?;
    let title = d.string(2048)?;
    let url = d.string(MAX_STRING)?;
    let load_ms = d.f64()?;
    let mut diagnostics = Vec::new();
    for _ in 0..d.count(256)? {
        diagnostics.push(d.string(8192)?);
    }
    let root = d.count(MAX_NODES - 1)?;
    let scripting = d.boolean()?;
    let mode = match d.byte()? {
        0 => DocumentMode::NoQuirks,
        1 => DocumentMode::LimitedQuirks,
        2 => DocumentMode::Quirks,
        _ => return Err("unknown IPC document mode".into()),
    };
    let encoding_name = d.string(64)?;
    let encoding = encoding_rs::Encoding::for_label(encoding_name.as_bytes())
        .filter(|encoding| encoding.name() == encoding_name)
        .ok_or("invalid IPC document encoding")?;
    let frozen_base = if d.boolean()? {
        let node = d.count(MAX_NODES - 1)?;
        let mut base_url_bytes = MAX_DOM_BYTES;
        let url = d.budget_string(&mut base_url_bytes)?;
        Some((
            node,
            url::Url::parse(&url).map_err(|_| "invalid IPC base URL")?,
        ))
    } else {
        None
    };
    let count = d.count(MAX_NODES)?;
    let mut nodes = Vec::new();
    let mut dom_bytes = MAX_DOM_BYTES;
    let mut children_left = MAX_NODES;
    let mut attrs_left = 1_000_000;
    for _ in 0..count {
        let parent = if d.boolean()? {
            Some(d.count(count.saturating_sub(1))?)
        } else {
            None
        };
        let child_count = d.count(children_left)?;
        children_left -= child_count;
        let mut children = Vec::new();
        for _ in 0..child_count {
            children.push(d.count(count.saturating_sub(1))?);
        }
        let kind = match d.byte()? {
            0 => NodeKind::Document,
            1 => {
                let namespace = match d.byte()? {
                    0 => Namespace::Html,
                    1 => Namespace::Svg,
                    2 => Namespace::MathMl,
                    _ => return Err("unknown IPC element namespace".into()),
                };
                let tag = d.budget_string(&mut dom_bytes)?;
                let mut attrs = BTreeMap::new();
                let count = d.count(attrs_left.min(1024))?;
                attrs_left -= count;
                for _ in 0..count {
                    let key = d.budget_string(&mut dom_bytes)?;
                    let value = d.budget_string(&mut dom_bytes)?;
                    if attrs.insert(key, value).is_some() {
                        return Err("duplicate IPC attribute".into());
                    }
                }
                let mut attr_namespaces = BTreeMap::new();
                // Namespace metadata must refer to an already bounded attribute;
                // reject impossible maps before allocating their entries.
                for _ in 0..d.count(attrs.len())? {
                    let name = d.budget_string(&mut dom_bytes)?;
                    let namespace = match d.byte()? {
                        0 => AttributeNamespace::XLink,
                        1 => AttributeNamespace::Xml,
                        2 => AttributeNamespace::Xmlns,
                        _ => return Err("unknown IPC attribute namespace".into()),
                    };
                    if !attrs.contains_key(&name)
                        || AttributeNamespace::from_qualified_name(&name) != Some(namespace)
                    {
                        return Err("invalid IPC attribute namespace binding".into());
                    }
                    if attr_namespaces.insert(name, namespace).is_some() {
                        return Err("duplicate IPC attribute namespace".into());
                    }
                }
                NodeKind::Element(Element {
                    namespace,
                    tag,
                    attrs,
                    attr_namespaces,
                    template_contents: if d.boolean()? {
                        Some(d.count(MAX_NODES - 1)?)
                    } else {
                        None
                    },
                })
            }
            2 => NodeKind::Text(d.budget_string(&mut dom_bytes)?),
            3 => NodeKind::Comment(d.budget_string(&mut dom_bytes)?),
            4 => {
                let name = d.budget_string(&mut dom_bytes)?;
                let public_id = if d.boolean()? {
                    Some(d.budget_string(&mut dom_bytes)?)
                } else {
                    None
                };
                let system_id = if d.boolean()? {
                    Some(d.budget_string(&mut dom_bytes)?)
                } else {
                    None
                };
                NodeKind::Doctype(Doctype {
                    name,
                    public_id,
                    system_id,
                    force_quirks: d.boolean()?,
                })
            }
            5 => NodeKind::ProcessingInstruction {
                target: d.budget_string(&mut dom_bytes)?,
                data: d.budget_string(&mut dom_bytes)?,
            },
            6 => NodeKind::DocumentFragment {
                host: if d.boolean()? {
                    Some(d.count(count.saturating_sub(1))?)
                } else {
                    None
                },
            },
            _ => return Err("unknown IPC DOM node kind".into()),
        };
        nodes.push(Node {
            parent,
            children,
            kind,
        });
    }
    let mut document = Document::from_snapshot(nodes, root, scripting, mode)?;
    document.set_encoding(encoding);
    document.initialize_url(url::Url::parse(&url).map_err(|_| "invalid IPC document URL")?);
    if let Some((node, base)) = frozen_base {
        document.restore_frozen_base(node, base)?;
    } else if document.frozen_base().is_some() {
        return Err("missing IPC frozen base URL".into());
    }
    let mut images = ImageStore::new();
    let mut rasters = Vec::new();
    let mut image_bytes = MAX_IMAGES;
    for _ in 0..d.count(MAX_NODES)? {
        let width = d.count(8192)? as u32;
        let height = d.count(8192)? as u32;
        let length = d.count(image_bytes)?;
        if width == 0 || height == 0 || width as u64 * height as u64 * 4 != length as u64 {
            return Err("invalid IPC raster dimensions".into());
        }
        image_bytes -= length;
        rasters.push(Arc::new(RasterImage {
            width,
            height,
            rgba: d.raw(length)?.to_vec(),
        }));
    }
    let mut keys_bytes = MAX_STRING;
    for _ in 0..d.count(MAX_NODES)? {
        let key = d.budget_string(&mut keys_bytes)?;
        let index = d.count(rasters.len().saturating_sub(1))?;
        let image = rasters
            .get(index)
            .ok_or("invalid IPC raster reference")?
            .clone();
        if images.insert(key, image).is_some() {
            return Err("duplicate IPC image key".into());
        }
    }
    let content_height = d.f32()?;
    if content_height < 0.0 {
        return Err("negative content height".into());
    }
    let mut hit_regions = Vec::new();
    for _ in 0..d.count(MAX_COMMANDS)? {
        let node = d.count(document.nodes.len() - 1)?;
        let rect = d.rect()?;
        let fixed = d.boolean()?;
        let action = match d.byte()? {
            0 => HitAction::Node,
            1 if document.namespace(node) == Some(Namespace::Html)
                && document.tag(node) == Some("details")
                && document.first_summary(node).is_none()
                && document.is_active_node(node)
                && !document.disclosure_hidden(node) =>
            {
                HitAction::DefaultSummary
            }
            _ => return Err("invalid IPC hit action".into()),
        };
        hit_regions.push(HitRegion {
            node,
            rect,
            fixed,
            action,
        });
    }
    let mut commands = Vec::new();
    let mut text_bytes = 8 * 1024 * 1024;
    let mut glyphs = 500_000usize;
    // A shared typed stack prevents closures from escaping another scope kind.
    #[derive(PartialEq)]
    enum Scope {
        Clip,
        Fixed,
        Opacity,
    }
    let mut scopes = Vec::new();
    for _ in 0..d.count(MAX_COMMANDS)? {
        let command = match d.byte()? {
            0 => {
                if scopes.len() >= 128 {
                    return Err("IPC clip depth exceeded".into());
                }
                scopes.push(Scope::Clip);
                DrawCommand::PushClip { rect: d.rect()? }
            }
            1 => {
                if scopes.pop() != Some(Scope::Clip) {
                    return Err("unbalanced IPC clips".into());
                }
                DrawCommand::PopClip
            }
            2 => {
                let rect = d.rect()?;
                let color = d.color()?;
                let radius = d.f32()?;
                if radius < 0.0 {
                    return Err("negative IPC radius".into());
                }
                DrawCommand::Rect {
                    rect,
                    color,
                    radius,
                }
            }
            3 => {
                let x = d.f32()?;
                let y = d.f32()?;
                let text = d.budget_string(&mut text_bytes)?;
                glyphs = glyphs
                    .checked_sub(text.chars().count())
                    .ok_or("IPC glyph budget exceeded")?;
                let size = d.f32()?;
                if !(0.0..=512.0).contains(&size) {
                    return Err("invalid IPC font size".into());
                }
                DrawCommand::Text {
                    x,
                    y,
                    text,
                    size,
                    color: d.color()?,
                    bold: d.boolean()?,
                    italic: d.boolean()?,
                    monospace: d.boolean()?,
                }
            }
            4 => {
                let rect = d.rect()?;
                let key = d.budget_string(&mut keys_bytes)?;
                // Failed, blocked, or still unavailable images legitimately
                // produce commands without a raster. The painter skips them;
                // decoding a reference never fetches or opens its key.
                DrawCommand::Image { rect, key }
            }
            5 => {
                let x1 = d.f32()?;
                let y1 = d.f32()?;
                let x2 = d.f32()?;
                let y2 = d.f32()?;
                let color = d.color()?;
                let width = d.f32()?;
                if width < 0.0 {
                    return Err("negative IPC line width".into());
                }
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                    width,
                }
            }
            6 => {
                if scopes.len() >= 128 {
                    return Err("IPC display scope depth exceeded".into());
                }
                scopes.push(Scope::Fixed);
                DrawCommand::PushFixed
            }
            7 => {
                if scopes.pop() != Some(Scope::Fixed) {
                    return Err("unbalanced IPC fixed scopes".into());
                }
                DrawCommand::PopFixed
            }
            8 => {
                if scopes.len() >= 128 {
                    return Err("IPC display scope depth exceeded".into());
                }
                let opacity = d.f32()?;
                if !(0.0..=1.0).contains(&opacity) {
                    return Err("invalid IPC group opacity".into());
                }
                scopes.push(Scope::Opacity);
                DrawCommand::PushOpacity { opacity }
            }
            9 => {
                if scopes.pop() != Some(Scope::Opacity) {
                    return Err("unbalanced IPC opacity scopes".into());
                }
                DrawCommand::PopOpacity
            }
            _ => return Err("unknown IPC display command".into()),
        };
        commands.push(command);
    }
    if !scopes.is_empty() {
        return Err("unclosed IPC display scopes".into());
    }
    Ok(Snapshot {
        generation,
        processed_edit_sequence,
        layout: LayoutResult {
            commands,
            hit_regions,
            content_height,
        },
        images,
        document,
        title,
        url,
        diagnostics,
        load_ms,
    })
}

pub(super) fn is_fetch_request(bytes: &[u8]) -> bool {
    bytes.get(..4) == Some(MAGIC) && bytes.get(4) == Some(&3)
}
pub(super) fn encode_broker_init(init: &super::broker::BrokerInit) -> Result<Vec<u8>> {
    let mut e = Encoder::new(5);
    e.navigation(&init.navigation);
    e.optional_string(
        init.root
            .as_ref()
            .map(|p| p.to_str().ok_or("broker root is not UTF-8"))
            .transpose()?,
    );
    e.finish()
}
pub(super) fn decode_broker_init(bytes: &[u8]) -> Result<super::broker::BrokerInit> {
    let mut d = Decoder::new(bytes, 5)?;
    let navigation = d.navigation()?;
    let root = d.optional_string(65_536)?.map(PathBuf::from);
    d.end()?;
    if root
        .as_ref()
        .is_some_and(|p| !p.is_absolute() || !p.is_dir())
    {
        return Err("invalid broker file root".into());
    }
    Ok(super::broker::BrokerInit { navigation, root })
}
pub(super) fn encode_fetch_request(request: &super::broker::FetchRequest) -> Result<Vec<u8>> {
    use crate::net::ResourceKind;
    let mut e = Encoder::new(3);
    e.string(request.url.as_str());
    e.byte(match request.kind {
        ResourceKind::Document => 0,
        ResourceKind::Style => 1,
        ResourceKind::Script => 2,
        ResourceKind::Image => 3,
    });
    e.optional_string(request.form_body.as_deref());
    let bytes = e.finish()?;
    if bytes.len() > MAX_REQUEST {
        return Err("resource request exceeds IPC budget".into());
    }
    Ok(bytes)
}
pub(super) fn decode_fetch_request(bytes: &[u8]) -> Result<super::broker::FetchRequest> {
    use crate::net::ResourceKind;
    let mut d = Decoder::new(bytes, 3)?;
    let url = url::Url::parse(&d.string(MAX_STRING)?).map_err(|e| e.to_string())?;
    let kind = match d.byte()? {
        0 => ResourceKind::Document,
        1 => ResourceKind::Style,
        2 => ResourceKind::Script,
        3 => ResourceKind::Image,
        _ => return Err("unknown resource kind".into()),
    };
    let form_body = d.optional_string(crate::net::MAX_FORM_BODY_BYTES)?;
    d.end()?;
    Ok(super::broker::FetchRequest {
        url,
        kind,
        form_body,
    })
}
pub(super) fn encode_resource(
    resource: &std::result::Result<crate::net::Resource, String>,
) -> Result<Vec<u8>> {
    let mut e = Encoder::new(4);
    e.boolean(resource.is_ok());
    match resource {
        Err(error) => e.string(&error.chars().take(2048).collect::<String>()),
        Ok(r) => {
            if r.bytes.len() > crate::net::MAX_RESOURCE_BYTES
                || r.headers.len() > 1024
                || r.headers
                    .iter()
                    .map(|(k, v)| k.len() + v.len())
                    .sum::<usize>()
                    > 256 * 1024
            {
                return Err("broker response budget exceeded".into());
            }
            e.string(r.url.as_str());
            e.u32(r.status as usize);
            e.string(&r.content_type);
            e.u32(r.headers.len());
            for (k, v) in &r.headers {
                e.string(k);
                e.string(v);
            }
            e.u32(r.bytes.len());
            e.raw(&r.bytes);
            e.boolean(r.origin_clean);
            e.boolean(r.decoded_image.is_some());
            if let Some(image) = &r.decoded_image {
                validate_opaque_resource(r)?;
                encode_raster(&mut e, image, MAX_IMAGES)?;
            }
        }
    }
    e.finish()
}
pub(super) fn decode_resource(
    bytes: &[u8],
) -> Result<std::result::Result<crate::net::Resource, String>> {
    let mut d = Decoder::new(bytes, 4)?;
    let result = if d.boolean()? {
        let url = url::Url::parse(&d.string(MAX_STRING)?).map_err(|e| e.to_string())?;
        let status = d.count(599)? as u16;
        if status < 100 {
            return Err("invalid response status".into());
        }
        let content_type = d.string(8192)?;
        let mut headers = BTreeMap::new();
        let mut budget = 256 * 1024;
        for _ in 0..d.count(1024)? {
            let key = d.budget_string(&mut budget)?;
            let value = d.budget_string(&mut budget)?;
            if key.is_empty()
                || !key
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&c))
            {
                return Err("invalid response header name".into());
            }
            if headers.insert(key.to_ascii_lowercase(), value).is_some() {
                return Err("duplicate response header".into());
            }
        }
        let length = d.count(crate::net::MAX_RESOURCE_BYTES)?;
        let bytes = d.raw(length)?.to_vec();
        let origin_clean = d.boolean()?;
        let decoded_image = if d.boolean()? {
            Some(decode_raster(&mut d, MAX_IMAGES)?)
        } else {
            None
        };
        let resource = crate::net::Resource {
            url,
            status,
            content_type,
            headers,
            bytes,
            origin_clean,
            decoded_image,
        };
        if resource.decoded_image.is_some() {
            validate_opaque_resource(&resource)?;
        }
        Ok(resource)
    } else {
        Err(d.string(8192)?)
    };
    d.end()?;
    Ok(result)
}

fn validate_opaque_resource(r: &crate::net::Resource) -> Result<()> {
    if r.origin_clean
        || !r.bytes.is_empty()
        || !r.headers.is_empty()
        || !r.content_type.is_empty()
        || r.status != 200
    {
        return Err("decoded opaque image contains response metadata".into());
    }
    Ok(())
}
fn encode_raster(e: &mut Encoder, image: &RasterImage, budget: usize) -> Result<()> {
    let expected = raster_length(image.width as usize, image.height as usize, budget)?;
    if image.rgba.len() != expected {
        return Err("incorrect decoded image length".into());
    }
    e.u32(image.width as usize);
    e.u32(image.height as usize);
    e.raw(&image.rgba);
    Ok(())
}
fn raster_length(width: usize, height: usize, budget: usize) -> Result<usize> {
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return Err("invalid decoded image dimensions".into());
    }
    let length = width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(4))
        .ok_or("decoded image size overflow")?;
    if length > budget.min(MAX_IMAGES) {
        return Err("decoded image budget exceeded".into());
    }
    Ok(length)
}
fn decode_raster(d: &mut Decoder<'_>, budget: usize) -> Result<RasterImage> {
    let width = d.count(4096)?;
    let height = d.count(4096)?;
    let length = raster_length(width, height, budget)?;
    Ok(RasterImage {
        width: width as u32,
        height: height as u32,
        rgba: d.raw(length)?.to_vec(),
    })
}
pub(super) fn encode_decoder_init() -> Result<Vec<u8>> {
    Encoder::new(6).finish()
}
pub(super) fn decode_decoder_init(bytes: &[u8]) -> Result<()> {
    Decoder::new(bytes, 6)?.end()
}
pub(super) fn encode_image_request(
    content_type: &str,
    bytes: &[u8],
    budget: usize,
) -> Result<Vec<u8>> {
    if content_type.len() > 8192
        || bytes.len() > crate::net::MAX_RESOURCE_BYTES
        || budget > MAX_IMAGES
    {
        return Err("image decoder request exceeds budget".into());
    }
    let mut e = Encoder::new(7);
    e.string(content_type);
    e.u32(budget);
    e.u32(bytes.len());
    e.raw(bytes);
    e.finish()
}
pub(super) fn decode_image_request(bytes: &[u8]) -> Result<(String, &[u8], usize)> {
    let mut d = Decoder::new(bytes, 7)?;
    let content_type = d.string(8192)?;
    let budget = d.count(MAX_IMAGES)?;
    let length = d.count(crate::net::MAX_RESOURCE_BYTES)?;
    let source = d.raw(length)?;
    d.end()?;
    Ok((content_type, source, budget))
}
pub(super) fn encode_image_result(result: &Result<RasterImage>, budget: usize) -> Result<Vec<u8>> {
    let mut e = Encoder::new(8);
    e.boolean(result.is_ok());
    match result {
        Ok(image) => encode_raster(&mut e, image, budget)?,
        // Decoder diagnostics cannot contain original body fragments.
        Err(_) => e.string("image decoding failed"),
    }
    e.finish()
}
pub(super) fn decode_image_result(bytes: &[u8], budget: usize) -> Result<Result<RasterImage>> {
    let mut d = Decoder::new(bytes, 8)?;
    let result = if d.boolean()? {
        Ok(decode_raster(&mut d, budget)?)
    } else {
        let _ = d.string(8192)?;
        Err("image decoding failed".into())
    };
    d.end()?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn opaque_image_protocol_preserves_pixels_without_original_response_metadata() {
        let image = RasterImage {
            width: 2,
            height: 1,
            rgba: vec![1, 2, 3, 255, 4, 5, 6, 255],
        };
        let mut resource = crate::net::Resource {
            url: url::Url::parse("https://other.test/requested.png").unwrap(),
            status: 200,
            content_type: String::new(),
            headers: BTreeMap::new(),
            bytes: Vec::new(),
            origin_clean: false,
            decoded_image: Some(image.clone()),
        };
        let encoded = encode_resource(&Ok(resource)).unwrap();
        resource = decode_resource(&encoded).unwrap().unwrap();
        assert_eq!(resource.decoded_image.as_ref().unwrap().rgba, image.rgba);
        assert!(!resource.origin_clean);
        assert!(
            resource.bytes.is_empty()
                && resource.headers.is_empty()
                && resource.content_type.is_empty()
        );
        for cut in 0..encoded.len() {
            assert!(decode_resource(&encoded[..cut]).is_err());
        }
        resource.bytes = b"private body".to_vec();
        assert!(encode_resource(&Ok(resource)).is_err());
        let encoded = encode_image_result(&Ok(image.clone()), 8).unwrap();
        assert_eq!(
            decode_image_result(&encoded, 8).unwrap().unwrap().rgba,
            image.rgba
        );
        assert!(decode_image_result(&encoded, 7).is_err());
        for cut in 0..encoded.len() {
            assert!(decode_image_result(&encoded[..cut], 8).is_err());
        }
        let mut forged = encoded.clone();
        forged[6..10].copy_from_slice(&4097u32.to_le_bytes());
        assert!(decode_image_result(&forged, MAX_IMAGES).is_err());
        let mut forged = encoded;
        forged[6..10].copy_from_slice(&0u32.to_le_bytes());
        assert!(decode_image_result(&forged, MAX_IMAGES).is_err());
        let error = encode_image_result(&Err("private original body".into()), 8).unwrap();
        assert!(!String::from_utf8_lossy(&error).contains("private"));
        assert_eq!(
            decode_image_result(&error, 8).unwrap().unwrap_err(),
            "image decoding failed"
        );
    }
    #[test]
    fn decoder_requests_reject_truncation_and_trailing_data() {
        let encoded = encode_image_request("image/png", b"encoded bytes", 256).unwrap();
        assert_eq!(
            decode_image_request(&encoded).unwrap(),
            ("image/png".to_owned(), b"encoded bytes".as_slice(), 256)
        );
        for cut in 0..encoded.len() {
            assert!(decode_image_request(&encoded[..cut]).is_err());
        }
        let mut extra = encoded;
        extra.push(0);
        assert!(decode_image_request(&extra).is_err());
        assert!(encode_image_request("image/png", b"", MAX_IMAGES + 1).is_err());
    }
    #[test]
    fn resource_protocol_preserves_authority_fields_and_rejects_truncated_payloads() {
        use crate::net::{Resource, ResourceKind};
        let request = super::super::broker::FetchRequest {
            url: url::Url::parse("https://example.test/submit?x=1").unwrap(),
            kind: ResourceKind::Document,
            form_body: Some("q=%F0%9F%A6%80".into()),
        };
        let bytes = encode_fetch_request(&request).unwrap();
        let decoded = decode_fetch_request(&bytes).unwrap();
        assert_eq!(decoded.url, request.url);
        assert_eq!(decoded.kind, request.kind);
        assert_eq!(decoded.form_body, request.form_body);
        for cut in 0..bytes.len() {
            assert!(decode_fetch_request(&bytes[..cut]).is_err());
        }
        let resource = Resource {
            url: url::Url::parse("https://other.test/final").unwrap(),
            status: 200,
            content_type: "text/html; charset=utf-8".into(),
            headers: BTreeMap::from([(
                "content-security-policy".into(),
                "default-src 'none'".into(),
            )]),
            bytes: b"<p>broker response</p>".to_vec(),
            origin_clean: true,
            decoded_image: None,
        };
        let encoded = encode_resource(&Ok(resource)).unwrap();
        let decoded = decode_resource(&encoded).unwrap().unwrap();
        assert_eq!(decoded.url.as_str(), "https://other.test/final");
        assert_eq!(decoded.status, 200);
        assert_eq!(
            decoded.headers.get("content-security-policy").unwrap(),
            "default-src 'none'"
        );
        assert_eq!(decoded.bytes, b"<p>broker response</p>");
        for cut in 0..encoded.len() {
            assert!(decode_resource(&encoded[..cut]).is_err());
        }
        let mut extra = encoded;
        extra.push(0);
        assert!(decode_resource(&extra).is_err());
        let error = encode_resource(&Err("fetch rejected".into())).unwrap();
        assert_eq!(
            decode_resource(&error).unwrap().unwrap_err(),
            "fetch rejected"
        );
    }

    #[test]
    fn resource_protocol_rejects_header_aliases_and_declared_oversized_bodies() {
        for headers in [
            vec![
                ("Content-Type", "text/html"),
                ("content-type", "text/plain"),
            ],
            vec![("bad\r\nname", "value")],
        ] {
            let mut e = Encoder::new(4);
            e.boolean(true);
            e.string("https://example.test/");
            e.u32(200);
            e.string("text/html");
            e.u32(headers.len());
            for (key, value) in headers {
                e.string(key);
                e.string(value);
            }
            e.u32(0);
            assert!(decode_resource(&e.finish().unwrap()).is_err());
        }
        let mut e = Encoder::new(4);
        e.boolean(true);
        e.string("https://example.test/");
        e.u32(200);
        e.string("text/html");
        e.u32(0);
        e.u32(crate::net::MAX_RESOURCE_BYTES + 1);
        assert!(
            decode_resource(&e.finish().unwrap())
                .unwrap_err()
                .contains("limit")
        );
    }

    #[test]
    fn foreign_namespaces_survive_snapshot_validation() {
        let r = reply_with_html(
            "<svg viewBox='0 0 4 4'><linearGradient id=gradient xlink:href='#x'/></svg><math><mi id=math>x</mi></math>",
        );
        let snapshot = decode_reply(&encode_reply(&r).unwrap())
            .unwrap()
            .snapshot
            .unwrap();
        let doc = snapshot.document;
        let svg = doc.query_selector("svg").unwrap();
        let gradient = doc.query_selector("#gradient").unwrap();
        assert_eq!(doc.namespace(svg), Some(Namespace::Svg));
        assert_eq!(doc.attr(svg, "viewBox"), Some("0 0 4 4"));
        assert_eq!(doc.tag(gradient), Some("linearGradient"));
        if let NodeKind::Element(element) = &doc.nodes[gradient].kind {
            assert_eq!(
                element.attr_namespaces.get("xlink:href"),
                Some(&AttributeNamespace::XLink)
            );
        } else {
            panic!("gradient element missing");
        }
        assert_eq!(
            doc.namespace(doc.query_selector("#math").unwrap()),
            Some(Namespace::MathMl)
        );
    }
    #[test]
    fn namespace_metadata_requires_existing_attributes_before_allocation() {
        let mut e = Encoder::new(99);
        e.u64(1); // generation
        e.u64(0); // edit sequence
        e.string("");
        e.string("about:blank");
        e.f64(0.0);
        e.u32(0); // diagnostics
        e.u32(0); // root
        e.boolean(false);
        e.byte(0); // no-quirks mode
        e.string("UTF-8");
        e.boolean(false); // no frozen base
        e.u32(1); // nodes
        e.boolean(false); // parent
        e.u32(0); // children
        e.byte(1); // element
        e.byte(1); // SVG
        e.string("g");
        e.u32(0); // no attributes
        e.u32(1024); // impossible namespace metadata, no payload supplied
        let bytes = e.finish().unwrap();
        let mut d = Decoder::new(&bytes, 99).unwrap();
        let error = match decode_snapshot(&mut d) {
            Ok(_) => panic!("impossible namespace map accepted"),
            Err(error) => error,
        };
        assert_eq!(error, "IPC collection limit exceeded");
        let mut r = reply_with_html("<svg xlink:href='#x'/>");
        let doc = &mut r.snapshot.as_mut().unwrap().document;
        let id = doc.query_selector("svg").unwrap();
        if let NodeKind::Element(element) = &mut doc.nodes[id].kind {
            element
                .attr_namespaces
                .insert("xlink:href".into(), AttributeNamespace::Xml);
        }
        let error = match decode_reply(&encode_reply(&r).unwrap()) {
            Ok(_) => panic!("wrong attribute namespace accepted"),
            Err(error) => error,
        };
        assert_eq!(error, "invalid IPC attribute namespace binding");
    }
    fn reply() -> Reply {
        reply_with_html("<p>Hello</p>")
    }
    #[test]
    fn frozen_base_roundtrip_preserves_old_fragment_and_rejects_forged_identity() {
        let old = "https://example.test/page#old-base-marker";
        let current = "https://example.test/page#current";
        let mut reply = reply_with_html("<base href='javascript:ignored'><base href='/later/'>");
        let snapshot = reply.snapshot.as_mut().unwrap();
        snapshot
            .document
            .initialize_url(url::Url::parse(old).unwrap());
        snapshot.document.set_url(url::Url::parse(current).unwrap());
        snapshot.url = current.into();
        let bytes = encode_reply(&reply).unwrap();
        let decoded = decode_reply(&bytes).unwrap().snapshot.unwrap();
        assert_eq!(decoded.document.url().as_str(), current);
        assert_eq!(decoded.document.base_url().as_str(), old);
        let start = bytes
            .windows(old.len())
            .position(|part| part == old.as_bytes())
            .unwrap();
        let mut forged = bytes.clone();
        forged[start - 8..start - 4].copy_from_slice(&0u32.to_le_bytes());
        assert!(decode_reply(&forged).is_err());
        let mut missing = bytes;
        missing[start - 9] = 0;
        missing.drain(start - 8..start + old.len());
        assert!(
            matches!(decode_reply(&missing), Err(error) if error == "missing IPC frozen base URL")
        );
    }
    #[test]
    fn percent_encoded_frozen_base_uses_its_own_dom_sized_budget() {
        let href = format!("/{}", "é".repeat(3 * 1024 * 1024));
        let mut reply = reply_with_html(&format!("<base href='{href}'>"));
        let snapshot = reply.snapshot.as_mut().unwrap();
        snapshot.url = "https://example.test/page".into();
        snapshot
            .document
            .initialize_url(url::Url::parse(&snapshot.url).unwrap());
        assert!(snapshot.document.base_url().as_str().len() > MAX_STRING);
        let bytes = encode_reply(&reply).unwrap();
        let decoded = decode_reply(&bytes).unwrap().snapshot.unwrap();
        assert_eq!(
            decoded.document.base_url(),
            reply.snapshot.as_ref().unwrap().document.base_url()
        );
    }
    #[test]
    fn template_fragments_mode_and_encoding_roundtrip_with_graph_validation() {
        let mut r = reply_with_html(
            "<!doctype html PUBLIC '-//W3C//DTD XHTML 1.0 Transitional//EN' 'x'><template id=t><b id=x>inert</b><template id=n><i>nested</i></template></template><p>live</p>",
        );
        r.snapshot
            .as_mut()
            .unwrap()
            .document
            .set_encoding(encoding_rs::SHIFT_JIS);
        let mut doc = decode_reply(&encode_reply(&r).unwrap())
            .unwrap()
            .snapshot
            .unwrap()
            .document;
        assert_eq!(doc.mode(), DocumentMode::LimitedQuirks);
        assert_eq!(doc.character_set(), "Shift_JIS");
        assert!(doc.query_selector("#x").is_none());
        let template = doc.query_selector("#t").unwrap();
        assert!(doc.nodes[template].children.is_empty());
        let contents = doc.template_contents(template).unwrap();
        assert!(
            matches!(doc.nodes[contents].kind, NodeKind::DocumentFragment { host: Some(id) } if id == template)
        );
        let child = doc.query_selector_from(contents, "#x").unwrap();
        assert_eq!(doc.text_content(child), "inert");
        let body = doc.query_selector("body").unwrap();
        doc.append_child(body, contents);
        assert!(doc.query_selector("#x").is_some());
        assert!(doc.nodes[contents].children.is_empty());

        // A compromised renderer must not send host ownership cycles or aliases.
        let snapshot = r.snapshot.as_mut().unwrap();
        let template = snapshot.document.query_selector("#t").unwrap();
        let contents = snapshot.document.template_contents(template).unwrap();
        snapshot.document.nodes[contents].kind = NodeKind::DocumentFragment {
            host: Some(contents),
        };
        assert!(decode_reply(&encode_reply(&r).unwrap()).is_err());
    }
    #[test]
    fn snapshot_mode_and_encoding_are_validated_before_node_allocation() {
        for (mode, encoding, expected) in [
            (3, "UTF-8", "unknown IPC document mode"),
            (0, "unknown", "invalid IPC document encoding"),
            (0, "utf8", "invalid IPC document encoding"),
        ] {
            let mut e = Encoder::new(99);
            e.u64(1);
            e.u64(0);
            e.string("");
            e.string("about:blank");
            e.f64(0.0);
            e.u32(0);
            e.u32(0);
            e.boolean(false);
            e.byte(mode);
            e.string(encoding);
            // No node count/payload supplied: rejection must happen first.
            let bytes = e.finish().unwrap();
            let mut d = Decoder::new(&bytes, 99).unwrap();
            assert!(matches!(decode_snapshot(&mut d), Err(error) if error == expected));
        }
    }
    fn reply_with_html(html: &str) -> Reply {
        let page =
            crate::page::Page::from_html(url::Url::parse("about:blank").unwrap(), html, false);
        Reply {
            navigation: None,
            snapshot: Some(Snapshot {
                generation: 7,
                processed_edit_sequence: 3,
                layout: page.layout(400.0, 300.0, &crate::graphics::Fonts::new()),
                images: page.images,
                document: page.document,
                title: "test".into(),
                url: "about:blank".into(),
                diagnostics: vec![],
                load_ms: 0.0,
            }),
        }
    }
    #[test]
    fn details_generated_summary_actions_round_trip_and_reject_forged_targets() {
        let mut reply = reply_with_html("<details id=d>contents</details><p id=p>ordinary</p>");
        let decoded = decode_reply(&encode_reply(&reply).unwrap()).unwrap();
        assert!(
            decoded
                .snapshot
                .unwrap()
                .layout
                .hit_regions
                .iter()
                .any(|hit| hit.action == HitAction::DefaultSummary)
        );
        let snapshot = reply.snapshot.as_mut().unwrap();
        let ordinary = snapshot.document.query_selector("#p").unwrap();
        let hit = snapshot
            .layout
            .hit_regions
            .iter_mut()
            .find(|hit| hit.action == HitAction::DefaultSummary)
            .unwrap();
        hit.node = ordinary;
        assert!(decode_reply(&encode_reply(&reply).unwrap()).is_err());
        let mut reply = reply_with_html("<details id=d><summary>actual</summary></details>");
        let snapshot = reply.snapshot.as_mut().unwrap();
        let details = snapshot.document.query_selector("#d").unwrap();
        snapshot
            .layout
            .hit_regions
            .iter_mut()
            .find(|hit| hit.node == details)
            .unwrap()
            .action = HitAction::DefaultSummary;
        assert!(decode_reply(&encode_reply(&reply).unwrap()).is_err());
        let command =
            decode_command(&encode_command(&Command::DefaultSummary { node: 7 }).unwrap()).unwrap();
        assert!(matches!(command, Command::DefaultSummary { node: 7 }));
    }
    #[test]
    fn unavailable_images_keep_the_rendered_fallback_without_fetching() {
        let r = reply_with_html(
            "<img src='missing.png' alt='fallback' width=40 height=30><video poster='unavailable.png'></video>",
        );
        let snapshot = decode_reply(&encode_reply(&r).unwrap())
            .unwrap()
            .snapshot
            .unwrap();
        assert!(snapshot.images.is_empty());
        assert!(snapshot.layout.commands.iter().any(
            |command| matches!(command, DrawCommand::Image { key, .. } if key == "missing.png")
        ));
        assert!(snapshot.layout.commands.iter().any(
            |command| matches!(command, DrawCommand::Image { key, .. } if key == "unavailable.png")
        ));
        let mut canvas = crate::graphics::Canvas::new(400, 300).unwrap();
        canvas.paint(
            &snapshot.layout.commands,
            &crate::graphics::Fonts::new(),
            &snapshot.images,
            0.0,
            0.0,
        );
        assert!(
            canvas
                .pixels
                .iter()
                .any(|pixel| *pixel != Color::WHITE.packed())
        );
    }
    #[test]
    fn all_rendered_text_obeys_the_shared_ipc_glyph_budget() {
        let image = format!("<img width=1 height=1 alt='{}'>", "x🦀".repeat(128));
        let html = image.repeat(2_000);
        let r = reply_with_html(&html);
        let glyphs: usize = r
            .snapshot
            .as_ref()
            .unwrap()
            .layout
            .commands
            .iter()
            .map(|command| match command {
                DrawCommand::Text { text, .. } => text.chars().count(),
                _ => 0,
            })
            .sum();
        assert!(glyphs <= 500_000, "renderer emitted {glyphs} glyphs");
        assert!(decode_reply(&encode_reply(&r).unwrap()).is_ok());
    }
    #[test]
    fn nested_user_agent_font_sizes_fit_the_painter_and_ipc_limits() {
        for depth in [32, 100] {
            let html = format!(
                "{}<u>text<br>next</u>{}",
                "<big>".repeat(depth),
                "</big>".repeat(depth)
            );
            let r = reply_with_html(&html);
            assert!(decode_reply(&encode_reply(&r).unwrap()).is_ok());
        }
    }
    #[test]
    fn expanded_preserved_tabs_do_not_exceed_the_emitted_glyph_budget() {
        let html = format!("<pre>{}</pre>", "\t".repeat(150_000));
        let r = reply_with_html(&html);
        let decoded = decode_reply(&encode_reply(&r).unwrap()).unwrap();
        let glyphs: usize = decoded
            .snapshot
            .unwrap()
            .layout
            .commands
            .iter()
            .map(|command| match command {
                DrawCommand::Text { text, .. } => text.chars().count(),
                _ => 0,
            })
            .sum();
        assert_eq!(glyphs, 500_000);
    }
    #[test]
    fn expanded_dom_character_data_uses_dom_budget_instead_of_metadata_limit() {
        let mut r = reply();
        // Six MiB of HTML NULs become eighteen MiB of U+FFFD in raw text,
        // inside both the source cap and the retained DOM cap.
        let source = format!("<script>{}</script>", "\0".repeat(6 * 1024 * 1024));
        let document = Document::parse(&source);
        drop(source);
        assert!(document.retained_bytes() > MAX_STRING);
        r.snapshot.as_mut().unwrap().document = document;
        r.snapshot.as_mut().unwrap().layout.hit_regions.clear();
        let expected_bytes = r.snapshot.as_ref().unwrap().document.retained_bytes();
        let encoded = encode_reply(&r).unwrap();
        drop(r);
        let decoded = decode_reply(&encoded).unwrap().snapshot.unwrap();
        assert_eq!(decoded.document.retained_bytes(), expected_bytes);
        assert!(decoded.document.nodes.iter().any(
            |node| matches!(&node.kind, NodeKind::Text(text) if text.len() == 18 * 1024 * 1024)
        ));
    }
    #[test]
    fn per_element_attribute_limits_match_dom_mutation_limits() {
        let mut r = reply();
        let document = &mut r.snapshot.as_mut().unwrap().document;
        let element = document.query_selector("p").unwrap();
        for index in 0..1024 {
            document.set_attr(element, &format!("a{index}"), "");
        }
        assert!(decode_reply(&encode_reply(&r).unwrap()).is_ok());
        let NodeKind::Element(element) =
            &mut r.snapshot.as_mut().unwrap().document.nodes[element].kind
        else {
            unreachable!()
        };
        element.attrs.insert("one-too-many".into(), String::new());
        assert!(decode_reply(&encode_reply(&r).unwrap()).is_err());
    }
    #[test]
    fn scalar_and_shared_string_limits_reject_before_payload_allocation() {
        let mut encoded = Encoder::new(9);
        encoded.string("abc");
        encoded.string("def");
        let bytes = encoded.finish().unwrap();
        let mut d = Decoder::new(&bytes, 9).unwrap();
        let mut budget = 5;
        assert_eq!(d.budget_string(&mut budget).unwrap(), "abc");
        assert_eq!(budget, 2);
        assert!(d.budget_string(&mut budget).is_err());
        assert_eq!(budget, 2);
        for (claimed, mut budget) in [
            (MAX_STRING + 1, MAX_STRING),
            (MAX_DOM_BYTES + 1, MAX_DOM_BYTES),
            (u32::MAX as usize, MAX_DOM_BYTES),
        ] {
            let mut encoded = Encoder::new(9);
            encoded.u32(claimed);
            let bytes = encoded.finish().unwrap();
            let mut d = Decoder::new(&bytes, 9).unwrap();
            if budget == MAX_STRING {
                assert!(d.string(budget).is_err());
            } else {
                assert!(d.budget_string(&mut budget).is_err());
            }
        }
    }
    #[test]
    fn error_frames_require_exact_consumption() {
        let mut encoded = encode_error("worker failed").unwrap();
        assert_eq!(decode_reply(&encoded).err().unwrap(), "worker failed");
        encoded.push(0);
        assert_eq!(decode_reply(&encoded).err().unwrap(), "trailing IPC bytes");
    }
    #[test]
    fn geometry_clips_and_raster_references_are_validated_independently() {
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        for commands in [
            vec![DrawCommand::Rect {
                rect: Rect {
                    width: -1.0,
                    ..rect
                },
                color: Color::BLACK,
                radius: 0.0,
            }],
            vec![DrawCommand::Rect {
                rect,
                color: Color::BLACK,
                radius: f32::INFINITY,
            }],
            vec![DrawCommand::Line {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
                color: Color::BLACK,
                width: -1.0,
            }],
            vec![DrawCommand::Text {
                x: 0.0,
                y: 0.0,
                text: "x".into(),
                size: 513.0,
                color: Color::BLACK,
                bold: false,
                italic: false,
                monospace: false,
            }],
            vec![DrawCommand::PopClip],
            vec![DrawCommand::PopFixed],
            vec![DrawCommand::PopOpacity],
            vec![DrawCommand::PushOpacity { opacity: 0.5 }],
            vec![
                DrawCommand::PushOpacity { opacity: f32::NAN },
                DrawCommand::PopOpacity,
            ],
            vec![
                DrawCommand::PushOpacity { opacity: -0.1 },
                DrawCommand::PopOpacity,
            ],
            vec![
                DrawCommand::PushOpacity { opacity: 1.1 },
                DrawCommand::PopOpacity,
            ],
            vec![
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PushFixed,
                DrawCommand::PopOpacity,
                DrawCommand::PopFixed,
            ],
            vec![
                DrawCommand::PushClip { rect },
                DrawCommand::PushOpacity { opacity: 0.5 },
                DrawCommand::PopClip,
                DrawCommand::PopOpacity,
            ],
            vec![DrawCommand::PushFixed],
            vec![
                DrawCommand::PushFixed,
                DrawCommand::PopClip,
                DrawCommand::PopFixed,
            ],
            vec![
                DrawCommand::PushClip { rect },
                DrawCommand::PushFixed,
                DrawCommand::PopClip,
                DrawCommand::PopFixed,
            ],
            vec![
                DrawCommand::PushFixed,
                DrawCommand::PushClip { rect },
                DrawCommand::PopFixed,
            ],
            vec![DrawCommand::PushClip { rect }],
            (0..129)
                .map(|_| DrawCommand::PushClip { rect })
                .chain((0..129).map(|_| DrawCommand::PopClip))
                .collect(),
        ] {
            let mut r = reply();
            r.snapshot.as_mut().unwrap().layout.commands = commands;
            assert!(decode_reply(&encode_reply(&r).unwrap()).is_err());
        }
        let mut r = reply();
        let key = "unique-raster-key";
        r.snapshot.as_mut().unwrap().images.insert(
            key.into(),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            }),
        );
        let mut bytes = encode_reply(&r).unwrap();
        let index = bytes
            .windows(key.len())
            .position(|bytes| bytes == key.as_bytes())
            .unwrap()
            + key.len();
        bytes[index..index + 4].copy_from_slice(&1u32.to_le_bytes());
        assert!(decode_reply(&bytes).is_err());
    }
    #[test]
    fn clipped_float_layout_at_command_quota_survives_snapshot_validation() {
        let source = format!(
            "<style>.clip{{overflow:hidden;width:40px}}.float{{float:left;width:40px}}.leaf{{height:1px;overflow:hidden}}</style>{}{}{}",
            "<div class=clip><div class=float>".repeat(16),
            "<div class=leaf>x</div>".repeat(30_000),
            "</div></div>".repeat(16),
        );
        let reply = reply_with_html(&source);
        let snapshot = decode_reply(&encode_reply(&reply).unwrap())
            .unwrap()
            .snapshot
            .unwrap();
        assert!(snapshot.layout.commands.len() <= MAX_COMMANDS);
        let mut clips = 0;
        for command in &snapshot.layout.commands {
            match command {
                DrawCommand::PushClip { .. } => clips += 1,
                DrawCommand::PopClip => {
                    assert!(clips > 0);
                    clips -= 1;
                }
                _ => {}
            }
        }
        assert_eq!(clips, 0);
    }
    #[test]
    fn opacity_snapshot_scopes_preserve_nested_fixed_geometry() {
        let mut original = reply();
        original.snapshot.as_mut().unwrap().layout.commands = vec![
            DrawCommand::PushOpacity { opacity: 0.5 },
            DrawCommand::PushFixed,
            DrawCommand::PushClip {
                rect: Rect {
                    x: 2.0,
                    y: 3.0,
                    width: 20.0,
                    height: 30.0,
                },
            },
            DrawCommand::PushOpacity { opacity: 0.25 },
            DrawCommand::Rect {
                rect: Rect {
                    x: 4.0,
                    y: 5.0,
                    width: 10.0,
                    height: 12.0,
                },
                color: Color::BLACK,
                radius: 0.0,
            },
            DrawCommand::PopOpacity,
            DrawCommand::PopClip,
            DrawCommand::PopFixed,
            DrawCommand::PopOpacity,
        ];
        let bytes = encode_reply(&original).unwrap();
        let result = decode_reply(&bytes).unwrap();
        assert_eq!(encode_reply(&result).unwrap(), bytes);
        let mut oversized = original;
        oversized.snapshot.as_mut().unwrap().layout.commands = (0..43)
            .flat_map(|_| {
                [
                    DrawCommand::PushFixed,
                    DrawCommand::PushOpacity { opacity: 0.5 },
                    DrawCommand::PushClip {
                        rect: Rect::default(),
                    },
                ]
            })
            .chain((0..43).flat_map(|_| {
                [
                    DrawCommand::PopClip,
                    DrawCommand::PopOpacity,
                    DrawCommand::PopFixed,
                ]
            }))
            .collect();
        assert!(decode_reply(&encode_reply(&oversized).unwrap()).is_err());
    }
    #[test]
    fn snapshot_round_trip_and_truncation_rejection() {
        let bytes = encode_reply(&reply()).unwrap();
        let decoded = decode_reply(&bytes).unwrap().snapshot.unwrap();
        assert_eq!(decoded.generation, 7);
        assert_eq!(
            decoded.document.text_content(decoded.document.root),
            "Hello"
        );
        for end in 0..bytes.len() {
            assert!(decode_reply(&bytes[..end]).is_err());
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(decode_reply(&extra).is_err());
    }
    #[test]
    fn rejects_bad_graph_geometry_images_and_lengths() {
        let mut r = reply();
        let s = r.snapshot.as_mut().unwrap();
        s.document.nodes[0].children.push(0);
        assert!(decode_reply(&encode_reply(&r).unwrap()).is_err());
        let mut r = reply();
        r.snapshot.as_mut().unwrap().layout.content_height = f32::NAN;
        assert!(decode_reply(&encode_reply(&r).unwrap()).is_err());
        let mut r = reply();
        r.snapshot.as_mut().unwrap().images.insert(
            "bad".into(),
            Arc::new(RasterImage {
                width: 2,
                height: 2,
                rgba: vec![0],
            }),
        );
        assert!(decode_reply(&encode_reply(&r).unwrap()).is_err());
        assert!(read_frame(&mut &u32::MAX.to_le_bytes()[..], MAX_FRAME).is_err());
        let mut bytes = encode_reply(&reply()).unwrap();
        bytes[5] = 2;
        assert!(decode_reply(&bytes).is_err());
    }
    #[test]
    fn raster_aliases_preserve_sharing() {
        let mut r = reply();
        let image = Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        });
        let images = &mut r.snapshot.as_mut().unwrap().images;
        images.insert("one".into(), image.clone());
        images.insert("two".into(), image);
        let s = decode_reply(&encode_reply(&r).unwrap())
            .unwrap()
            .snapshot
            .unwrap();
        assert!(Arc::ptr_eq(&s.images["one"], &s.images["two"]));
    }
    #[test]
    fn deterministic_protocol_mutations_do_not_panic() {
        let bytes = encode_reply(&reply()).unwrap();
        for i in 0..1000 {
            let mut mutated = bytes.clone();
            let index = (i * 7919) % bytes.len();
            mutated[index] ^= ((i % 255) + 1) as u8;
            let _ = decode_reply(&mutated);
        }
    }
}
