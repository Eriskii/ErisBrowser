//! Bounded, borrowed admission of browser display lists to the offscreen probe.
//!
//! A refusal applies to the entire original frame. This module neither paints a
//! CPU reference nor submits GPU work, and never fetches a missing image key.
use crate::{
    Command, Frame, MAX_COMMANDS, MAX_COORDINATE, MAX_HEIGHT, MAX_SCOPES, MAX_SOURCE_ENTRIES,
    MAX_WIDTH, Plan, Rect, SourceImage,
};
use eris::graphics::{DrawCommand, ImageStore, RasterImage};
use std::{cmp::Ordering, sync::Arc};

pub const MAX_STORE_CAPACITY: usize = 512;
pub const MAX_KEY_BYTES: usize = 4_096;
pub const MAX_STORE_KEY_BYTES: usize = 65_536;
pub const MAX_COMMAND_KEY_BYTES: usize = 65_536;
pub const MAX_SORT_COMPARISON_BYTES: usize = 16_777_216;
pub const MAX_LOOKUP_WORK: usize = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FallbackKind {
    UnsupportedText,
    UnsupportedRoundedRect,
    UnsupportedOpacity,
    InvalidGeometry,
    InvalidImage,
    InvalidScope,
    ViewportLimit,
    CommandLimit,
    ScopeLimit,
    ImageStoreLimit,
    KeyBudget,
    CpuPaintBudget,
    PlannerLimit,
    AllocationFailure,
    TextLimit,
    MaskPreparation,
    GlyphRowLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FallbackReason {
    pub kind: FallbackKind,
    /// Original display-list index; never an index in the lowered list.
    pub command_index: Option<usize>,
}
impl FallbackReason {
    fn new(kind: FallbackKind, command_index: Option<usize>) -> Self {
        Self {
            kind,
            command_index,
        }
    }

    pub fn category(&self) -> &'static str {
        match self.kind {
            FallbackKind::UnsupportedText => "unsupported-text",
            FallbackKind::UnsupportedRoundedRect => "unsupported-rounded-rectangle",
            FallbackKind::UnsupportedOpacity => "unsupported-opacity",
            FallbackKind::InvalidGeometry => "invalid-geometry",
            FallbackKind::InvalidImage => "invalid-image",
            FallbackKind::InvalidScope => "invalid-scope",
            FallbackKind::ViewportLimit => "viewport-budget",
            FallbackKind::CommandLimit => "command-budget",
            FallbackKind::ScopeLimit => "scope-budget",
            FallbackKind::ImageStoreLimit => "image-store-budget",
            FallbackKind::KeyBudget => "key-budget",
            FallbackKind::CpuPaintBudget => "cpu-paint-budget",
            FallbackKind::PlannerLimit => "planner-budget",
            FallbackKind::AllocationFailure => "allocation-failure",
            FallbackKind::TextLimit => "text-budget",
            FallbackKind::MaskPreparation => "mask-preparation",
            FallbackKind::GlyphRowLimit => "glyph-row-budget",
        }
    }
}
impl std::fmt::Display for FallbackReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.category())?;
        if let Some(index) = self.command_index {
            write!(f, " at original command {index}")?;
        }
        Ok(())
    }
}
impl std::error::Error for FallbackReason {}

type Result<T> = std::result::Result<T, FallbackReason>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BridgeStats {
    pub original_commands: usize,
    pub lowered_commands: usize,
    pub image_store_entries: usize,
    /// Distinct Arc identities, including unreferenced store entries.
    pub unique_sources: usize,
    /// Distinct identities named by any Image command, including hidden images.
    pub referenced_sources: usize,
    /// All unique stored RGBA bytes, counted once per Arc identity.
    pub total_rgba_bytes: usize,
    pub referenced_rgba_bytes: usize,
    /// Original Image commands whose key is absent, including repeated misses.
    pub missing_images: usize,
}

#[derive(Debug)]
pub struct BridgePlan {
    plan: Plan,
    stats: BridgeStats,
}
impl BridgePlan {
    pub fn plan(&self) -> &Plan {
        &self.plan
    }
    pub fn stats(&self) -> &BridgeStats {
        &self.stats
    }
}

#[derive(Clone, Copy)]
enum ScopeTag {
    Clip,
    Fixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct KeyWork {
    sort_bytes: usize,
    lookup_units: usize,
}

fn bounded_scalar(value: f32) -> bool {
    value.is_finite() && value.abs() <= MAX_COORDINATE
}
fn convert_rect(rect: eris::graphics::Rect) -> Rect {
    Rect::new(rect.x, rect.y, rect.width, rect.height)
}
fn validate_rect(rect: Rect, index: Option<usize>) -> Result<()> {
    if [rect.x, rect.y, rect.width, rect.height]
        .into_iter()
        .all(bounded_scalar)
    {
        Ok(())
    } else {
        Err(FallbackReason::new(FallbackKind::InvalidGeometry, index))
    }
}
fn validate_frame(frame: Frame) -> Result<()> {
    if frame.width == 0 || frame.height == 0 || frame.width > MAX_WIDTH || frame.height > MAX_HEIGHT
    {
        return Err(FallbackReason::new(FallbackKind::ViewportLimit, None));
    }
    if frame.clear > 0x00ff_ffff
        || ![
            frame.document_offset.0,
            frame.document_offset.1,
            frame.viewport_offset.0,
            frame.viewport_offset.1,
        ]
        .into_iter()
        .all(bounded_scalar)
    {
        return Err(FallbackReason::new(FallbackKind::InvalidGeometry, None));
    }
    validate_rect(frame.caller_clip, None)
}

fn cpu_work(frame: Frame, commands: usize) -> Result<()> {
    let refusal = || FallbackReason::new(FallbackKind::CpuPaintBudget, None);
    let area = u64::from(frame.width)
        .checked_mul(u64::from(frame.height))
        .ok_or_else(refusal)?;
    let budget = area
        .checked_mul(16)
        .ok_or_else(refusal)?
        .clamp(1_000_000, 32_000_000);
    let upper = u64::try_from(commands)
        .ok()
        .and_then(|n| n.checked_mul(area))
        .ok_or_else(refusal)?;
    if upper > budget {
        return Err(refusal());
    }
    Ok(())
}

fn add_key(length: usize, total: &mut usize, cap: usize, index: Option<usize>) -> Result<()> {
    let refusal = || FallbackReason::new(FallbackKind::KeyBudget, index);
    if length > MAX_KEY_BYTES {
        return Err(refusal());
    }
    let next = total.checked_add(length).ok_or_else(refusal)?;
    if next > cap {
        return Err(refusal());
    }
    *total = next;
    Ok(())
}

fn key_work(
    entries: usize,
    store_bytes: usize,
    command_bytes: usize,
    images: usize,
) -> Result<KeyWork> {
    let refusal = || FallbackReason::new(FallbackKind::KeyBudget, None);
    // In insertion sort a key can participate in at most N-1 comparisons;
    // charge both compared lengths, conservatively bounded by this sum.
    let sort_bytes = entries
        .saturating_sub(1)
        .checked_mul(store_bytes)
        .ok_or_else(refusal)?;
    // ceil(log2(N+1)) + 1, including the empty table without subtraction.
    let comparisons = (usize::BITS - entries.leading_zeros()) as usize + 1;
    let lookup_units = command_bytes
        .checked_add(images)
        .and_then(|n| n.checked_mul(comparisons))
        .ok_or_else(refusal)?;
    if sort_bytes > MAX_SORT_COMPARISON_BYTES || lookup_units > MAX_LOOKUP_WORK {
        return Err(refusal());
    }
    Ok(KeyWork {
        sort_bytes,
        lookup_units,
    })
}

fn line_rect(x1: f32, y1: f32, x2: f32, y2: f32, width: f32, index: usize) -> Result<Rect> {
    if ![x1, y1, x2, y2, width].into_iter().all(bounded_scalar) {
        return Err(FallbackReason::new(
            FallbackKind::InvalidGeometry,
            Some(index),
        ));
    }
    // Preserve Canvas's f32 scalar order. Translation happens exactly once,
    // later in the existing planner, rather than on either endpoint here.
    let rect = Rect::new(
        x1.min(x2),
        y1.min(y2),
        (x2 - x1).abs().max(width),
        (y2 - y1).abs().max(width),
    );
    validate_rect(rect, Some(index))?;
    Ok(rect)
}

fn preflight_commands(commands: &[DrawCommand]) -> Result<(usize, usize)> {
    preflight_commands_by(commands, |index, _| {
        Err(FallbackReason::new(
            FallbackKind::UnsupportedText,
            Some(index),
        ))
    })
}

fn preflight_commands_by(
    commands: &[DrawCommand],
    mut text: impl FnMut(usize, &DrawCommand) -> Result<()>,
) -> Result<(usize, usize)> {
    let mut scopes = [ScopeTag::Clip; MAX_SCOPES];
    let mut depth = 0;
    let mut key_bytes = 0;
    let mut image_commands = 0;
    for (index, command) in commands.iter().enumerate() {
        let at = Some(index);
        match command {
            DrawCommand::Rect { rect, radius, .. } => {
                validate_rect(convert_rect(*rect), at)?;
                if !bounded_scalar(*radius) {
                    return Err(FallbackReason::new(FallbackKind::InvalidGeometry, at));
                }
                if *radius != 0.0 {
                    return Err(FallbackReason::new(
                        FallbackKind::UnsupportedRoundedRect,
                        at,
                    ));
                }
            }
            DrawCommand::Image { rect, key } => {
                validate_rect(convert_rect(*rect), at)?;
                add_key(key.len(), &mut key_bytes, MAX_COMMAND_KEY_BYTES, at)?;
                image_commands += 1;
            }
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                ..
            } => {
                line_rect(*x1, *y1, *x2, *y2, *width, index)?;
            }
            DrawCommand::PushClip { rect } => {
                validate_rect(convert_rect(*rect), at)?;
                if depth == MAX_SCOPES {
                    return Err(FallbackReason::new(FallbackKind::ScopeLimit, at));
                }
                scopes[depth] = ScopeTag::Clip;
                depth += 1;
            }
            DrawCommand::PushFixed => {
                if depth == MAX_SCOPES {
                    return Err(FallbackReason::new(FallbackKind::ScopeLimit, at));
                }
                scopes[depth] = ScopeTag::Fixed;
                depth += 1;
            }
            DrawCommand::PopClip | DrawCommand::PopFixed => {
                if depth == 0 {
                    return Err(FallbackReason::new(FallbackKind::InvalidScope, at));
                }
                depth -= 1;
                if !matches!(
                    (command, scopes[depth]),
                    (DrawCommand::PopClip, ScopeTag::Clip)
                        | (DrawCommand::PopFixed, ScopeTag::Fixed)
                ) {
                    return Err(FallbackReason::new(FallbackKind::InvalidScope, at));
                }
            }
            DrawCommand::Text { .. } => {
                text(index, command)?;
            }
            DrawCommand::PushOpacity { .. } | DrawCommand::PopOpacity => {
                return Err(FallbackReason::new(FallbackKind::UnsupportedOpacity, at));
            }
        }
    }
    if depth != 0 {
        return Err(FallbackReason::new(FallbackKind::InvalidScope, None));
    }
    Ok((key_bytes, image_commands))
}

fn reserve<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| FallbackReason::new(FallbackKind::AllocationFailure, None))?;
    Ok(values)
}

type Entry<'a> = (&'a str, &'a Arc<RasterImage>);
fn sort_entries(entries: &mut [Entry<'_>]) {
    // Explicit bounded insertion sort, without hidden allocation or key copies.
    for i in 1..entries.len() {
        let mut j = i;
        while j > 0 && entries[j].0.cmp(entries[j - 1].0) == Ordering::Less {
            entries.swap(j, j - 1);
            j -= 1;
        }
    }
}
fn find_image<'a>(entries: &[Entry<'a>], key: &str) -> Option<&'a Arc<RasterImage>> {
    let (mut left, mut right) = (0, entries.len());
    while left < right {
        let middle = left + (right - left) / 2;
        // Exactly one lexical comparison per iteration. No hashing of author
        // strings, and at most key.len()+1 compared units per iteration.
        match key.cmp(entries[middle].0) {
            Ordering::Less => right = middle,
            Ordering::Greater => left = middle + 1,
            Ordering::Equal => return Some(entries[middle].1),
        }
    }
    None
}
fn source_id<'a>(sources: &mut Vec<&'a Arc<RasterImage>>, image: &'a Arc<RasterImage>) -> u32 {
    if let Some(index) = sources.iter().position(|old| Arc::ptr_eq(old, image)) {
        index as u32
    } else {
        let index = sources.len();
        // At most images.len() unique identities can be inserted; the complete
        // capacity was fallibly reserved before the first call.
        sources.push(image);
        index as u32
    }
}

pub fn plan_snapshot(snapshot: &eris::worker::Snapshot, frame: Frame) -> Result<BridgePlan> {
    plan_display_list(&snapshot.layout.commands, &snapshot.images, frame)
}

pub fn plan_display_list(
    commands: &[DrawCommand],
    images: &ImageStore,
    frame: Frame,
) -> Result<BridgePlan> {
    validate_frame(frame)?;
    if commands.len() > MAX_COMMANDS {
        return Err(FallbackReason::new(FallbackKind::CommandLimit, None));
    }
    // HashMap::iter scans capacity, not just live entries. Refuse a sparsely
    // reserved map before visiting it or allocating any borrowed table.
    if images.len() > MAX_SOURCE_ENTRIES || images.capacity() > MAX_STORE_CAPACITY {
        return Err(FallbackReason::new(FallbackKind::ImageStoreLimit, None));
    }
    let (command_key_bytes, image_commands) = preflight_commands(commands)?;
    let mut store_key_bytes = 0;
    for key in images.keys() {
        add_key(key.len(), &mut store_key_bytes, MAX_STORE_KEY_BYTES, None)?;
    }
    let work = key_work(
        images.len(),
        store_key_bytes,
        command_key_bytes,
        image_commands,
    )?;
    debug_assert!(work.sort_bytes <= MAX_SORT_COMPARISON_BYTES);
    debug_assert!(work.lookup_units <= MAX_LOOKUP_WORK);
    cpu_work(frame, commands.len())?;
    // All input counts, key-byte work and CPU pixel work have been checked.
    // Temporary metadata holds only references and fixed-size commands; no
    // snapshot, image, author key or source RGBA allocation is cloned.
    let mut entries = reserve(images.len())?;
    for (key, image) in images {
        entries.push((key.as_str(), image));
    }
    sort_entries(&mut entries);
    let mut unique = reserve(images.len())?;
    let mut lowered = reserve(commands.len())?;
    let mut missing_images = 0;
    for (index, command) in commands.iter().enumerate() {
        let lowered_command = match command {
            DrawCommand::Rect {
                rect,
                color,
                radius,
            } => Command::Rect {
                rect: convert_rect(*rect),
                rgba: [color.r, color.g, color.b, color.a],
                radius: *radius,
            },
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => Command::Rect {
                rect: line_rect(*x1, *y1, *x2, *y2, *width, index)?,
                rgba: [color.r, color.g, color.b, color.a],
                radius: 0.0,
            },
            DrawCommand::Image { rect, key } => {
                let Some(image) = find_image(&entries, key) else {
                    missing_images += 1;
                    continue;
                };
                Command::Image {
                    rect: convert_rect(*rect),
                    source: source_id(&mut unique, image),
                }
            }
            DrawCommand::PushClip { rect } => Command::PushClip(convert_rect(*rect)),
            DrawCommand::PopClip => Command::PopClip,
            DrawCommand::PushFixed => Command::PushFixed,
            DrawCommand::PopFixed => Command::PopFixed,
            // The complete immutable list was checked before table allocation.
            DrawCommand::Text { .. }
            | DrawCommand::PushOpacity { .. }
            | DrawCommand::PopOpacity => unreachable!("preflight rejected unsupported commands"),
        };
        lowered.push(lowered_command);
    }
    let referenced_sources = unique.len();
    for (_, image) in &entries {
        source_id(&mut unique, image);
    }
    let mut sources = reserve(unique.len())?;
    for image in &unique {
        sources.push(SourceImage {
            width: image.width,
            height: image.height,
            rgba: &image.rgba,
        });
    }
    // Reuse the planner's exact all-source validator, including unused/malformed
    // storage, before any pixel-sized copy. The planner repeats this bounded
    // metadata check; no allocation or alpha scan substitutes for it.
    let pixels = crate::source_pixels(&sources)
        .map_err(|_| FallbackReason::new(FallbackKind::InvalidImage, None))?;
    let total_rgba_bytes = pixels
        .checked_mul(4)
        .ok_or_else(|| FallbackReason::new(FallbackKind::InvalidImage, None))?;
    let referenced_rgba_bytes = sources[..referenced_sources]
        .iter()
        .try_fold(0usize, |total, source| total.checked_add(source.rgba.len()))
        .ok_or_else(|| FallbackReason::new(FallbackKind::InvalidImage, None))?;
    let stats = BridgeStats {
        original_commands: commands.len(),
        lowered_commands: lowered.len(),
        image_store_entries: images.len(),
        unique_sources: unique.len(),
        referenced_sources,
        total_rgba_bytes,
        referenced_rgba_bytes,
        missing_images,
    };
    let plan = crate::plan_with_images(frame, &lowered, &sources)
        .map_err(|_| FallbackReason::new(FallbackKind::PlannerLimit, None))?;
    Ok(BridgePlan { plan, stats })
}

#[cfg(test)]
mod tests;

mod text;
pub use text::{
    FontBridgePlan, TextBridgeStats, plan_display_list_with_fonts, plan_snapshot_with_fonts,
};
