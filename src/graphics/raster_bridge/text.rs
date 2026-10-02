//! Fonts-aware admission is separate from the original rectangle/image bridge.
//! Coverage is retained only by a bounded frame session, then packed once into
//! the plan. Neither callbacks nor the returned plan retain font or text leases.
use super::*;
use crate::graphics::{
    Fonts,
    text_masks::{self, MaskInfo, Session, TextRun},
};
use eris_raster_core::{MAX_GPU_BUFFER_BYTES, PARAM_STRIDE, SourceMask, scope::CoordinateState};
use std::cell::Cell;

const MAX_TEXT_BYTES: usize = 65_536;
const MAX_RUN_BYTES: usize = 32_768;
const MAX_INPUT_SCALARS: usize = 4_096;
const MAX_ROW_ENTRIES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextBridgeStats {
    pub input_bytes: usize,
    pub input_scalars: usize,
    pub preparation: text_masks::Stats,
    pub row_entries: usize,
    /// Conservative pre-raster bound, including all original image sources,
    /// all image-command LUT ceilings, and slots for non-drawing operations.
    pub gpu_buffer_upper_bound: u64,
    pub cpu_pixel_upper_bound: u64,
}

#[derive(Debug)]
pub struct FontBridgePlan {
    bridge: BridgePlan,
    text: TextBridgeStats,
}
impl FontBridgePlan {
    pub fn plan(&self) -> &Plan {
        self.bridge.plan()
    }
    pub fn stats(&self) -> &BridgeStats {
        self.bridge.stats()
    }
    pub fn text_stats(&self) -> &TextBridgeStats {
        &self.text
    }
}

struct PreparationLedger {
    operations: usize,
    sources: usize,
    rows: usize,
    mask_bytes: usize,
    image_bytes: usize,
    image_lut_words: usize,
    target_bytes: u64,
    cpu_pixels: u64,
    cpu_limit: u64,
    gpu_upper: u64,
}

fn preparation_error(error: text_masks::Error, index: Option<usize>) -> FallbackReason {
    use text_masks::Error;
    let kind = match error {
        Error::CpuPixels | Error::PixelAllowance => FallbackKind::CpuPaintBudget,
        Error::RunTextBytes | Error::FrameTextBytes | Error::FrameScalars => {
            FallbackKind::TextLimit
        }
        Error::Geometry | Error::RowIndex => FallbackKind::InvalidGeometry,
        Error::Allocation => FallbackKind::AllocationFailure,
        _ => FallbackKind::MaskPreparation,
    };
    FallbackReason::new(kind, index)
}
impl PreparationLedger {
    fn gpu_upper(&self) -> Option<u64> {
        let words = self
            .mask_bytes
            .checked_add(self.rows)?
            .checked_add(self.image_lut_words)?;
        self.target_bytes
            .checked_add(u64::try_from(self.image_bytes).ok()?)?
            .checked_add(u64::try_from(words.checked_mul(4)?).ok()?)?
            .checked_add(
                u64::try_from(self.operations.checked_add(1)?.checked_mul(PARAM_STRIDE)?).ok()?,
            )
    }

    fn preflight(&mut self, info: MaskInfo, is_new: bool, index: usize) -> Result<()> {
        let at = Some(index);
        // A failed frame never reuses this ledger. Increment before any mask
        // allocation, dependency raster scratch, row generation or packing.
        self.operations = self
            .operations
            .checked_add(1)
            .filter(|&n| n <= MAX_COMMANDS)
            .ok_or_else(|| FallbackReason::new(FallbackKind::CommandLimit, at))?;
        self.rows = self
            .rows
            .checked_add(info.height())
            .filter(|&n| n <= MAX_ROW_ENTRIES)
            .ok_or_else(|| FallbackReason::new(FallbackKind::GlyphRowLimit, at))?;
        if is_new {
            self.sources = self
                .sources
                .checked_add(1)
                .filter(|&n| n <= MAX_SOURCE_ENTRIES)
                .ok_or_else(|| FallbackReason::new(FallbackKind::MaskPreparation, at))?;
            self.mask_bytes = self
                .mask_bytes
                .checked_add(info.area())
                .ok_or_else(|| FallbackReason::new(FallbackKind::MaskPreparation, at))?;
        }
        self.cpu_pixels = self
            .cpu_pixels
            .checked_add(info.area().max(1) as u64)
            .filter(|&n| n <= self.cpu_limit)
            .ok_or_else(|| FallbackReason::new(FallbackKind::CpuPaintBudget, at))?;
        self.gpu_upper = self
            .gpu_upper()
            .filter(|&n| n <= MAX_GPU_BUFFER_BYTES)
            .ok_or_else(|| FallbackReason::new(FallbackKind::PlannerLimit, at))?;
        Ok(())
    }
}

pub fn plan_snapshot_with_fonts(
    snapshot: &crate::worker::Snapshot,
    frame: Frame,
    fonts: &Fonts,
) -> Result<FontBridgePlan> {
    plan_display_list_with_fonts(&snapshot.layout.commands, &snapshot.images, frame, fonts)
}

pub fn plan_display_list_with_fonts(
    commands: &[DrawCommand],
    images: &ImageStore,
    frame: Frame,
    fonts: &Fonts,
) -> Result<FontBridgePlan> {
    validate_frame(frame)?;
    if commands.len() > MAX_COMMANDS {
        return Err(FallbackReason::new(FallbackKind::CommandLimit, None));
    }
    if images.len() > MAX_SOURCE_ENTRIES || images.capacity() > MAX_STORE_CAPACITY {
        return Err(FallbackReason::new(FallbackKind::ImageStoreLimit, None));
    }
    let (mut input_bytes, mut input_scalars, mut text_commands) = (0usize, 0usize, 0usize);
    let (command_key_bytes, image_commands) = preflight_commands_by(commands, |index, command| {
        let DrawCommand::Text {
            x, y, size, text, ..
        } = command
        else {
            unreachable!()
        };
        let at = Some(index);
        if ![*x, *y, *size].into_iter().all(bounded_scalar) {
            return Err(FallbackReason::new(FallbackKind::InvalidGeometry, at));
        }
        input_bytes = input_bytes
            .checked_add(text.len())
            .filter(|&n| n <= MAX_TEXT_BYTES && text.len() <= MAX_RUN_BYTES)
            .ok_or_else(|| FallbackReason::new(FallbackKind::TextLimit, at))?;
        input_scalars = input_scalars
            .checked_add(text.chars().count())
            .filter(|&n| n <= MAX_INPUT_SCALARS)
            .ok_or_else(|| FallbackReason::new(FallbackKind::TextLimit, at))?;
        text_commands += 1;
        Ok(())
    })?;
    let nontext_commands = commands.len() - text_commands;
    cpu_work(frame, nontext_commands)?;
    let mut store_key_bytes = 0;
    for key in images.keys() {
        add_key(key.len(), &mut store_key_bytes, MAX_STORE_KEY_BYTES, None)?;
    }
    key_work(
        images.len(),
        store_key_bytes,
        command_key_bytes,
        image_commands,
    )?;
    let mut entries = reserve(images.len())?;
    for (key, image) in images {
        entries.push((key.as_str(), image));
    }
    sort_entries(&mut entries);
    let mut unique = reserve(images.len())?;
    let mut image_ids = [None; MAX_COMMANDS];
    let mut missing_images = 0;
    for (index, command) in commands.iter().enumerate() {
        if let DrawCommand::Image { key, .. } = command {
            if let Some(image) = find_image(&entries, key) {
                image_ids[index] = Some(source_id(&mut unique, image));
            } else {
                missing_images += 1;
            }
        }
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
    let total_rgba_bytes = eris_raster_core::validate_source_images(&sources)
        .map_err(|_| FallbackReason::new(FallbackKind::InvalidImage, None))?
        * 4;
    let referenced_rgba_bytes = sources[..referenced_sources]
        .iter()
        .map(|s| s.rgba.len())
        .sum();
    let area = u64::from(frame.width) * u64::from(frame.height);
    let cpu_limit = (area * 16).clamp(1_000_000, 32_000_000);
    let cpu_nontext = area * nontext_commands as u64;
    let mut ledger = PreparationLedger {
        operations: nontext_commands,
        sources: unique.len(),
        rows: 0,
        mask_bytes: 0,
        image_bytes: total_rgba_bytes,
        image_lut_words: image_commands * (frame.width as usize + frame.height as usize),
        target_bytes: area * 8,
        cpu_pixels: cpu_nontext,
        cpu_limit,
        gpu_upper: 0,
    };
    ledger.gpu_upper = ledger
        .gpu_upper()
        .filter(|&n| n <= MAX_GPU_BUFFER_BYTES)
        .ok_or_else(|| FallbackReason::new(FallbackKind::PlannerLimit, None))?;
    let mut session = Session::new(fonts, cpu_limit - cpu_nontext)
        .map_err(|error| preparation_error(error, None))?;
    let mut state = CoordinateState::new(frame)
        .map_err(|_| FallbackReason::new(FallbackKind::InvalidScope, None))?;
    let mut lowered = reserve(MAX_COMMANDS)?;
    let mut rows = reserve::<Vec<i32>>(MAX_COMMANDS)?;
    let mut keys = reserve(MAX_SOURCE_ENTRIES)?;
    for (index, command) in commands.iter().enumerate() {
        let at = Some(index);
        let lowered_command = match command {
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
                let offset = state.offset();
                let clip = state.clip();
                let clip = crate::graphics::Rect {
                    x: clip.x,
                    y: clip.y,
                    width: clip.width,
                    height: clip.height,
                };
                let run = TextRun {
                    x: *x + offset.0,
                    y: *y + offset.1,
                    text,
                    size: *size,
                    color: *color,
                    bold: *bold,
                    italic: *italic,
                    monospace: *monospace,
                };
                let refusal = Cell::new(None);
                session
                    .visit(
                        run,
                        clip,
                        |info, _, is_new| {
                            ledger.preflight(info, is_new, index).map_err(|error| {
                                refusal.set(Some(error));
                            })
                        },
                        |mask, placement| {
                            let info = mask.info();
                            let source = match keys.iter().position(|key| *key == info.key()) {
                                Some(source) => source,
                                None => {
                                    let source = keys.len();
                                    keys.push(info.key());
                                    source
                                }
                            };
                            let mut origins = reserve(info.height()).map_err(|error| {
                                refusal.set(Some(error));
                            })?;
                            for row in 0..info.height() {
                                origins.push(placement.row_x(row).map_err(|_| {
                                    refusal.set(Some(FallbackReason::new(
                                        FallbackKind::InvalidGeometry,
                                        at,
                                    )));
                                })?);
                            }
                            let color = placement.color();
                            lowered.push(Command::Glyph {
                                source: source as u32,
                                rows: rows.len() as u32,
                                y: placement.origin_y(),
                                rgba: [color.r, color.g, color.b, color.a],
                            });
                            rows.push(origins);
                            Ok(())
                        },
                    )
                    .map_err(|error| {
                        refusal
                            .get()
                            .unwrap_or_else(|| preparation_error(error, at))
                    })?;
                continue;
            }
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
            DrawCommand::Image { rect, .. } => {
                let Some(source) = image_ids[index] else {
                    continue;
                };
                Command::Image {
                    rect: convert_rect(*rect),
                    source,
                }
            }
            DrawCommand::PushClip { rect } => Command::PushClip(convert_rect(*rect)),
            DrawCommand::PopClip => Command::PopClip,
            DrawCommand::PushFixed => Command::PushFixed,
            DrawCommand::PopFixed => Command::PopFixed,
            DrawCommand::PushOpacity { .. } | DrawCommand::PopOpacity => {
                unreachable!("preflight refused opacity")
            }
        };
        state
            .apply(&lowered_command)
            .map_err(|_| FallbackReason::new(FallbackKind::InvalidScope, at))?;
        lowered.push(lowered_command);
    }
    state
        .finish()
        .map_err(|_| FallbackReason::new(FallbackKind::InvalidScope, None))?;
    let mut masks = reserve(keys.len())?;
    if session.masks().len() != keys.len() {
        return Err(FallbackReason::new(FallbackKind::MaskPreparation, None));
    }
    for (mask, key) in session.masks().zip(&keys) {
        let info = mask.info();
        if info.key() != *key {
            return Err(FallbackReason::new(FallbackKind::MaskPreparation, None));
        }
        masks.push(SourceMask {
            width: info.width() as u32,
            height: info.height() as u32,
            coverage: mask.coverage(),
        });
    }
    let mut row_tables = reserve(rows.len())?;
    for row in &rows {
        row_tables.push(row.as_slice());
    }
    let plan = eris_raster_core::plan_with_masks(frame, &lowered, &sources, &masks, &row_tables)
        .map_err(|_| FallbackReason::new(FallbackKind::PlannerLimit, None))?;
    if plan.gpu_buffer_bytes() > ledger.gpu_upper {
        return Err(FallbackReason::new(FallbackKind::PlannerLimit, None));
    }
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
    let text = TextBridgeStats {
        input_bytes,
        input_scalars,
        preparation: session.stats(),
        row_entries: ledger.rows,
        gpu_buffer_upper_bound: ledger.gpu_upper,
        cpu_pixel_upper_bound: ledger.cpu_pixels,
    };
    Ok(FontBridgePlan {
        bridge: BridgePlan { plan, stats },
        text,
    })
}

#[cfg(test)]
mod tests;
