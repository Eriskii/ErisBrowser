//! Exact paint-input identity for the bounded native benchmark only.
//!
//! EBNI version 1 uses fixed field order, little-endian integers/raw finite
//! float bits, one-byte tags, and u32-length-prefixed UTF-8/byte strings. Image
//! keys are sorted by UTF-8 bytes; alias IDs name the first sorted entry sharing
//! the same Arc. No pointer, process, generation, timing-of-capture, font-cache,
//! or GPU state is serialized. Source and bundled fonts are host-manifest inputs.
//! This is an input comparison, not a pixel oracle or renderer admission check.
use super::{Browser, Selection, TaskState};
use eris::graphics::{Color, DrawCommand, ImageStore, RasterImage, Rect};
use std::sync::Arc;
use winit::dpi::PhysicalSize;

pub(super) const MAX_IDENTITY_BYTES: usize = 64 * 1024;

// Bound sparse-map traversal before iterating. These are existing bridge caps,
// not new normal-renderer policy. Other traversal bounds derive from the byte
// limit, except the painter's 128 nested typed scopes.
const MAX_IMAGES: usize = 256;
const MAX_IMAGE_CAPACITY: usize = 512;
const MAX_SCOPE_DEPTH: usize = 128;

struct Bytes(Vec<u8>);
impl Bytes {
    fn new() -> Result<Self, String> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(MAX_IDENTITY_BYTES)
            .map_err(|_| "benchmark identity allocation")?;
        if bytes.capacity() > MAX_IDENTITY_BYTES {
            return Err("benchmark identity allocation capacity".into());
        }
        Ok(Self(bytes))
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.0
            .len()
            .checked_add(bytes.len())
            .filter(|length| *length <= MAX_IDENTITY_BYTES)
            .ok_or("benchmark identity exceeds 64 KiB")?;
        // The entire permitted output capacity was fallibly reserved once.
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn byte(&mut self, value: u8) -> Result<(), String> {
        self.append(&[value])
    }
    fn boolean(&mut self, value: bool) -> Result<(), String> {
        self.byte(u8::from(value))
    }
    fn u32(&mut self, value: u32) -> Result<(), String> {
        self.append(&value.to_le_bytes())
    }
    fn count(&mut self, value: usize) -> Result<(), String> {
        self.u32(u32::try_from(value).map_err(|_| "benchmark identity integer overflow")?)
    }
    fn f32(&mut self, value: f32) -> Result<(), String> {
        if !value.is_finite() {
            return Err("benchmark identity nonfinite scalar".into());
        }
        self.u32(value.to_bits())
    }
    fn blob(&mut self, value: &[u8]) -> Result<(), String> {
        // Reject oversize before writing even its prefix.
        self.0
            .len()
            .checked_add(4)
            .and_then(|length| length.checked_add(value.len()))
            .filter(|length| *length <= MAX_IDENTITY_BYTES)
            .ok_or("benchmark identity exceeds 64 KiB")?;
        self.count(value.len())?;
        self.append(value)
    }
    fn text(&mut self, value: &str) -> Result<(), String> {
        self.blob(value.as_bytes())
    }
    fn color(&mut self, value: Color) -> Result<(), String> {
        self.append(&[value.r, value.g, value.b, value.a])
    }
    fn rect(&mut self, value: Rect) -> Result<(), String> {
        if value.width < 0.0 || value.height < 0.0 {
            return Err("benchmark identity negative extent".into());
        }
        self.f32(value.x)?;
        self.f32(value.y)?;
        self.f32(value.width)?;
        self.f32(value.height)
    }
}

/// Capture only a loaded, current, quiescent page. This never paints, clones a
/// snapshot/image, or changes selection. A refusal aborts benchmark comparison;
/// it does not change normal CPU/native rendering limits or fallback behavior.
pub(super) fn capture(browser: &Browser, size: PhysicalSize<u32>) -> Result<Vec<u8>, String> {
    let snapshot = browser
        .snapshot
        .as_ref()
        .ok_or("benchmark identity requires a snapshot")?;
    if browser.loading
        || browser.closing
        || browser.worker_error.is_some()
        || browser.startup_error.is_some()
        || snapshot.generation != browser.generation()
        || snapshot.processed_edit_sequence != browser.edit_sequence
        || snapshot.task_state != TaskState::Idle
    {
        return Err("benchmark identity requires a current loaded idle page".into());
    }
    eris_raster_core::Profile::Native.validate_viewport(size.width, size.height)?;
    if !browser.zoom.is_finite() || browser.zoom <= 0.0 || !browser.scroll.is_finite() {
        return Err("benchmark identity invalid scroll or zoom".into());
    }
    if !snapshot.layout.content_height.is_finite() || snapshot.layout.content_height < 0.0 {
        return Err("benchmark identity invalid content height".into());
    }
    if snapshot.layout.commands.len() > MAX_IDENTITY_BYTES
        || snapshot.layout.hit_regions.len() > MAX_IDENTITY_BYTES
    {
        return Err("benchmark identity traversal budget".into());
    }
    let mut bytes = Bytes::new()?;
    bytes.append(b"EBNI\x01")?;
    bytes.u32(size.width)?;
    bytes.u32(size.height)?;
    bytes.f32(browser.scroll)?;
    bytes.f32(browser.zoom)?;
    bytes.text(&snapshot.title)?;
    bytes.text(&browser.address)?;
    bytes.boolean(browser.address_focused)?;

    // Both painters normalize against address or input_value before using the
    // selection. Capture the effective selection, plus its normalization input.
    bytes.text(&browser.input_value)?;
    let mut selection = Selection {
        caret: browser.selection.caret,
        anchor: browser.selection.anchor,
    };
    selection.normalize(if browser.address_focused {
        &browser.address
    } else {
        &browser.input_value
    });
    bytes.count(selection.caret)?;
    bytes.count(selection.anchor)?;

    bytes.boolean(browser.focused.is_some())?;
    if let Some(node) = browser.focused {
        if node >= snapshot.document.nodes.len() {
            return Err("benchmark identity invalid focused node".into());
        }
        bytes.count(node)?;
        // find(), not the last matching hit: this is the painter's exact rule.
        let hit = snapshot
            .layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == node);
        bytes.boolean(hit.is_some())?;
        if let Some(hit) = hit {
            bytes.rect(hit.rect)?;
            bytes.boolean(hit.fixed)?;
        }
    }
    bytes.f32(snapshot.layout.content_height)?;
    bytes.text(&browser.status)?;
    if browser.status.is_empty() {
        // The default status renders these inputs. With an explicit status,
        // load_ms is deliberately irrelevant, including its bit pattern.
        if !snapshot.load_ms.is_finite() || snapshot.load_ms < 0.0 {
            return Err("benchmark identity invalid rendered load time".into());
        }
        bytes.count(snapshot.document.nodes.len())?;
        bytes.count(snapshot.diagnostics.len())?;
        bytes.append(&snapshot.load_ms.to_bits().to_le_bytes())?;
    }
    commands(&mut bytes, &snapshot.layout.commands)?;
    images(&mut bytes, &snapshot.images)?;
    Ok(bytes.0)
}

fn commands(bytes: &mut Bytes, commands: &[DrawCommand]) -> Result<(), String> {
    bytes.count(commands.len())?;
    let mut scopes = [0u8; MAX_SCOPE_DEPTH];
    let mut depth = 0usize;
    for command in commands {
        let (tag, push, pop) = match command {
            DrawCommand::PushClip { .. } => (0, Some(0), None),
            DrawCommand::PopClip => (1, None, Some(0)),
            DrawCommand::PushFixed => (2, Some(1), None),
            DrawCommand::PopFixed => (3, None, Some(1)),
            DrawCommand::PushOpacity { .. } => (4, Some(2), None),
            DrawCommand::PopOpacity => (5, None, Some(2)),
            DrawCommand::Rect { .. } => (6, None, None),
            DrawCommand::Text { .. } => (7, None, None),
            DrawCommand::Image { .. } => (8, None, None),
            DrawCommand::Line { .. } => (9, None, None),
        };
        if let Some(scope) = push {
            let slot = scopes
                .get_mut(depth)
                .ok_or("benchmark identity scope budget")?;
            *slot = scope;
            depth += 1;
        }
        if let Some(scope) = pop {
            depth = depth
                .checked_sub(1)
                .ok_or("benchmark identity scope underflow")?;
            if scopes[depth] != scope {
                return Err("benchmark identity mismatched scope".into());
            }
        }
        bytes.byte(tag)?;
        match command {
            DrawCommand::PushClip { rect } => bytes.rect(*rect)?,
            DrawCommand::PopClip
            | DrawCommand::PushFixed
            | DrawCommand::PopFixed
            | DrawCommand::PopOpacity => {}
            DrawCommand::PushOpacity { opacity } => {
                if !(0.0..=1.0).contains(opacity) {
                    return Err("benchmark identity invalid opacity".into());
                }
                bytes.f32(*opacity)?;
            }
            DrawCommand::Rect {
                rect,
                color,
                radius,
            } => {
                if *radius < 0.0 {
                    return Err("benchmark identity negative radius".into());
                }
                bytes.rect(*rect)?;
                bytes.color(*color)?;
                bytes.f32(*radius)?;
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
                if *size <= 0.0 {
                    return Err("benchmark identity invalid text size".into());
                }
                bytes.f32(*x)?;
                bytes.f32(*y)?;
                bytes.text(text)?;
                bytes.f32(*size)?;
                bytes.color(*color)?;
                bytes.boolean(*bold)?;
                bytes.boolean(*italic)?;
                bytes.boolean(*monospace)?;
            }
            DrawCommand::Image { rect, key } => {
                bytes.rect(*rect)?;
                bytes.text(key)?;
            }
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => {
                if *width < 0.0 {
                    return Err("benchmark identity negative line width".into());
                }
                for value in [x1, y1, x2, y2, width] {
                    bytes.f32(*value)?;
                }
                bytes.color(*color)?;
            }
        }
    }
    if depth != 0 {
        return Err("benchmark identity unclosed scope".into());
    }
    Ok(())
}

fn images(bytes: &mut Bytes, images: &ImageStore) -> Result<(), String> {
    if images.len() > MAX_IMAGES || images.capacity() > MAX_IMAGE_CAPACITY {
        return Err("benchmark identity image-store traversal budget".into());
    }
    // Check all supplied key/payload lengths before sorting or reserving the
    // bounded borrowed metadata. Unused and aliased keys remain in the identity.
    let mut size = bytes
        .0
        .len()
        .checked_add(4)
        .ok_or("benchmark identity size overflow")?;
    for (key, image) in images {
        let expected = usize::try_from(image.width)
            .ok()
            .and_then(|width| {
                usize::try_from(image.height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or("benchmark identity image size overflow")?;
        if image.width == 0 || image.height == 0 || image.rgba.len() != expected {
            return Err("benchmark identity malformed image".into());
        }
        size = size
            .checked_add(20)
            .and_then(|size| size.checked_add(key.len()))
            .and_then(|size| size.checked_add(image.rgba.len()))
            .filter(|size| *size <= MAX_IDENTITY_BYTES)
            .ok_or("benchmark identity exceeds 64 KiB")?;
    }
    let mut ordered: Vec<(&String, &Arc<RasterImage>)> = Vec::new();
    ordered
        .try_reserve_exact(images.len())
        .map_err(|_| "benchmark image metadata allocation")?;
    ordered.extend(images);
    // In-place sort allocates no additional storage. At most 256 keys totaling
    // less than 64 KiB have already been admitted before comparisons begin.
    ordered.sort_unstable_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    bytes.count(ordered.len())?;
    for (index, (key, image)) in ordered.iter().enumerate() {
        let alias = ordered[..index]
            .iter()
            .position(|(_, prior)| Arc::ptr_eq(prior, image))
            .unwrap_or(index);
        bytes.text(key)?;
        bytes.count(alias)?;
        bytes.u32(image.width)?;
        bytes.u32(image.height)?;
        bytes.blob(&image.rgba)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser() -> Browser {
        let mut browser = super::super::tests::editing_browser("<input value='text'>");
        browser.status = "Fixed benchmark status".into();
        browser
    }
    fn identity(browser: &Browser) -> Vec<u8> {
        capture(browser, PhysicalSize::new(1280, 880)).unwrap()
    }
    fn image() -> Arc<RasterImage> {
        Arc::new(RasterImage {
            width: 2,
            height: 1,
            rgba: vec![1, 2, 3, 0, 4, 5, 6, 255],
        })
    }
    fn rect() -> Rect {
        Rect {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        }
    }
    fn changed(mutate: impl FnOnce(&mut Browser)) {
        let mut browser = browser();
        let before = identity(&browser);
        mutate(&mut browser);
        assert_ne!(before, identity(&browser));
    }

    #[test]
    fn image_order_is_canonical_but_aliases_and_every_payload_byte_matter() {
        let mut a = browser();
        let mut b = browser();
        let shared = image();
        a.snapshot
            .as_mut()
            .unwrap()
            .images
            .insert("z".into(), shared.clone());
        a.snapshot
            .as_mut()
            .unwrap()
            .images
            .insert("a".into(), shared.clone());
        b.snapshot
            .as_mut()
            .unwrap()
            .images
            .insert("a".into(), shared.clone());
        b.snapshot
            .as_mut()
            .unwrap()
            .images
            .insert("z".into(), shared);
        assert_eq!(identity(&a), identity(&b));
        b.snapshot
            .as_mut()
            .unwrap()
            .images
            .insert("z".into(), image());
        assert_ne!(
            identity(&a),
            identity(&b),
            "equal bytes do not erase alias accounting"
        );
        let before = identity(&b);
        let raster = Arc::make_mut(b.snapshot.as_mut().unwrap().images.get_mut("z").unwrap());
        raster.rgba[0] ^= 1; // Hidden RGB under alpha zero must still be bound.
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        let raster = Arc::make_mut(b.snapshot.as_mut().unwrap().images.get_mut("z").unwrap());
        (raster.width, raster.height) = (1, 2);
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        let raster = b.snapshot.as_mut().unwrap().images.remove("z").unwrap();
        b.snapshot
            .as_mut()
            .unwrap()
            .images
            .insert("other".into(), raster);
        assert_ne!(before, identity(&b));
    }

    #[test]
    fn ui_and_overlay_inputs_are_bound_without_mutating_selection() {
        changed(|b| b.address.push('x'));
        changed(|b| b.snapshot.as_mut().unwrap().title.push('x'));
        changed(|b| b.status.push('x'));
        changed(|b| b.input_value.push('x'));
        changed(|b| b.address_focused = true);
        changed(|b| b.scroll = 0.25);
        changed(|b| b.zoom = 1.0005);
        changed(|b| b.snapshot.as_mut().unwrap().layout.content_height += 1.0);
        let mut b = browser();
        let before = identity(&b);
        assert_ne!(before, capture(&b, PhysicalSize::new(1279, 880)).unwrap());
        b.address_focused = true;
        let before = identity(&b);
        b.selection.caret = 1;
        assert_ne!(before, identity(&b));
        b.selection.caret = usize::MAX;
        let normalized = identity(&b);
        assert_eq!(b.selection.caret, usize::MAX);
        b.selection.caret = b.address.len();
        assert_eq!(normalized, identity(&b));
        b.selection.anchor = 1;
        assert_ne!(normalized, identity(&b));
    }

    #[test]
    fn focus_uses_first_matching_geometry_and_fixed_flag() {
        let mut b = browser();
        let first = b.snapshot.as_ref().unwrap().layout.hit_regions[0].clone();
        let before = identity(&b);
        b.focused = Some(first.node);
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        b.snapshot.as_mut().unwrap().layout.hit_regions[0].rect.x += 1.0;
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        b.snapshot.as_mut().unwrap().layout.hit_regions[0].fixed ^= true;
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        b.snapshot.as_mut().unwrap().layout.hit_regions.push(first);
        assert_eq!(
            before,
            identity(&b),
            "later duplicate is not the painted hit"
        );
    }

    #[test]
    fn explicit_status_excludes_unused_load_time_and_notice_metadata() {
        let mut b = browser();
        let before = identity(&b);
        b.snapshot.as_mut().unwrap().load_ms = f64::NAN;
        b.snapshot
            .as_mut()
            .unwrap()
            .diagnostics
            .push("unused notice".into());
        assert_eq!(before, identity(&b));
        b.status.clear();
        assert!(capture(&b, PhysicalSize::new(1280, 880)).is_err());
        b.snapshot.as_mut().unwrap().load_ms = 1.0;
        let before = identity(&b);
        b.snapshot.as_mut().unwrap().load_ms = 2.0;
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        b.snapshot
            .as_mut()
            .unwrap()
            .diagnostics
            .push("rendered count".into());
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        b.snapshot.as_mut().unwrap().document.nodes.pop();
        assert_ne!(before, identity(&b));
    }

    #[test]
    fn all_command_variants_are_distinct_and_order_and_raw_float_bits_matter() {
        let variants = [
            vec![DrawCommand::PushClip { rect: rect() }, DrawCommand::PopClip],
            vec![DrawCommand::PushFixed, DrawCommand::PopFixed],
            vec![
                DrawCommand::PushOpacity { opacity: 1.0 },
                DrawCommand::PopOpacity,
            ],
            vec![DrawCommand::Rect {
                rect: rect(),
                color: Color::WHITE,
                radius: 0.0,
            }],
            vec![DrawCommand::Text {
                x: 1.0,
                y: 2.0,
                text: "é".into(),
                size: 12.0,
                color: Color::BLACK,
                bold: false,
                italic: false,
                monospace: false,
            }],
            vec![DrawCommand::Image {
                rect: rect(),
                key: "missing".into(),
            }],
            vec![DrawCommand::Line {
                x1: 1.0,
                y1: 2.0,
                x2: 3.0,
                y2: 4.0,
                color: Color::WHITE,
                width: 1.0,
            }],
        ];
        let mut seen = Vec::new();
        let mut b = browser();
        for commands in variants {
            b.snapshot.as_mut().unwrap().layout.commands = commands;
            let encoded = identity(&b);
            assert!(!seen.contains(&encoded));
            seen.push(encoded);
        }
        b.snapshot.as_mut().unwrap().layout.commands = vec![
            DrawCommand::Rect {
                rect: rect(),
                color: Color::BLACK,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: rect(),
                color: Color::WHITE,
                radius: 0.0,
            },
        ];
        let before = identity(&b);
        b.snapshot.as_mut().unwrap().layout.commands.reverse();
        assert_ne!(before, identity(&b));
        let before = identity(&b);
        if let DrawCommand::Rect { radius, .. } =
            &mut b.snapshot.as_mut().unwrap().layout.commands[0]
        {
            *radius = -0.0;
        }
        assert_ne!(before, identity(&b));
    }

    #[test]
    fn text_flags_color_and_content_are_not_omitted() {
        let mut b = browser();
        b.snapshot.as_mut().unwrap().layout.commands = vec![DrawCommand::Text {
            x: 1.0,
            y: 2.0,
            text: "A".into(),
            size: 12.0,
            color: Color::WHITE,
            bold: false,
            italic: false,
            monospace: false,
        }];
        for field in 0..8 {
            let before = identity(&b);
            let DrawCommand::Text {
                x,
                y,
                text,
                size,
                color,
                bold,
                italic,
                monospace,
            } = &mut b.snapshot.as_mut().unwrap().layout.commands[0]
            else {
                unreachable!()
            };
            match field {
                0 => *x += 0.25,
                1 => *y += 0.25,
                2 => text.push('é'),
                3 => *size += 1.0,
                4 => color.a -= 1,
                5 => *bold = true,
                6 => *italic = true,
                7 => *monospace = true,
                _ => unreachable!(),
            }
            assert_ne!(before, identity(&b));
        }
    }

    #[test]
    fn exact_byte_boundary_accepts_and_one_more_refuses_without_growth() {
        let mut bytes = Bytes::new().unwrap();
        bytes.append(&[0; MAX_IDENTITY_BYTES]).unwrap();
        let capacity = bytes.0.capacity();
        assert!(bytes.byte(0).is_err());
        assert_eq!(bytes.0.len(), MAX_IDENTITY_BYTES);
        assert_eq!(bytes.0.capacity(), capacity);
        let mut b = browser();
        let extra = MAX_IDENTITY_BYTES - identity(&b).len();
        b.status = "x".repeat(b.status.len() + extra);
        assert_eq!(identity(&b).len(), MAX_IDENTITY_BYTES);
        b.status.push('x');
        assert!(capture(&b, PhysicalSize::new(1280, 880)).is_err());
    }

    #[test]
    fn stale_loading_pending_and_error_states_refuse() {
        let bad: [fn(&mut Browser); 9] = [
            |b| b.snapshot = None,
            |b| b.loading = true,
            |b| b.closing = true,
            |b| b.worker_error = Some("stopped".into()),
            |b| b.startup_error = Some("failed".into()),
            |b| b.snapshot.as_mut().unwrap().generation += 1,
            |b| b.edit_sequence += 1,
            |b| b.snapshot.as_mut().unwrap().task_state = TaskState::Pending,
            |b| b.focused = Some(usize::MAX),
        ];
        for mutate in bad {
            let mut b = browser();
            mutate(&mut b);
            assert!(capture(&b, PhysicalSize::new(1280, 880)).is_err());
        }
    }

    #[test]
    fn invalid_geometry_scopes_images_and_sparse_store_refuse() {
        let mut b = browser();
        for size in [PhysicalSize::new(0, 880), PhysicalSize::new(1281, 880)] {
            assert!(capture(&b, size).is_err());
        }
        b.scroll = f32::INFINITY;
        assert!(capture(&b, PhysicalSize::new(1280, 880)).is_err());
        b.scroll = 0.0;
        for commands in [
            vec![DrawCommand::PopFixed],
            vec![
                DrawCommand::PushClip { rect: rect() },
                DrawCommand::PopFixed,
            ],
            vec![DrawCommand::PushFixed],
            vec![
                DrawCommand::PushOpacity { opacity: f32::NAN },
                DrawCommand::PopOpacity,
            ],
            vec![DrawCommand::Rect {
                rect: Rect {
                    width: -1.0,
                    ..rect()
                },
                color: Color::WHITE,
                radius: 0.0,
            }],
        ] {
            b.snapshot.as_mut().unwrap().layout.commands = commands;
            assert!(capture(&b, PhysicalSize::new(1280, 880)).is_err());
        }
        b.snapshot.as_mut().unwrap().layout.commands.clear();
        b.snapshot.as_mut().unwrap().images.insert(
            "bad".into(),
            Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0; 3],
            }),
        );
        assert!(capture(&b, PhysicalSize::new(1280, 880)).is_err());
        b.snapshot.as_mut().unwrap().images = ImageStore::with_capacity(1024);
        assert!(capture(&b, PhysicalSize::new(1280, 880)).is_err());
    }
}
