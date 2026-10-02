use super::*;
use crate::graphics::Color;

fn text(value: &str) -> DrawCommand {
    DrawCommand::Text {
        x: 0.0,
        y: 0.0,
        text: value.into(),
        size: 16.0,
        color: Color::BLACK,
        bold: false,
        italic: false,
        monospace: false,
    }
}
fn empty() -> ImageStore {
    ImageStore::new()
}
fn frame() -> Frame {
    Frame::new(64, 32, 0xffffff)
}
fn refusal(commands: &[DrawCommand], frame: Frame) -> FallbackReason {
    plan_display_list_with_fonts(commands, &empty(), frame, &Fonts::new()).unwrap_err()
}

#[test]
fn old_entry_keeps_text_fallback_while_new_entry_admits_masks() {
    let commands = [text("AV")];
    assert_eq!(
        plan_display_list(&commands, &empty(), frame())
            .unwrap_err()
            .kind,
        FallbackKind::UnsupportedText
    );
    let prepared =
        plan_display_list_with_fonts(&commands, &empty(), frame(), &Fonts::new()).unwrap();
    assert!(
        prepared
            .plan()
            .draws()
            .iter()
            .any(|d| d.kind() == eris_raster_core::DrawKind::Glyph)
    );
    assert_eq!(prepared.text_stats().preparation.occurrences, 2);
    assert_eq!(prepared.text_stats().preparation.cold_requests, 2);
    assert!(prepared.plan().has_input());
    assert!(!prepared.plan().has_images());
}

#[test]
fn all_text_inputs_are_counted_even_when_coarsely_culled() {
    let mut hidden = text(&"x".repeat(MAX_INPUT_SCALARS + 1));
    if let DrawCommand::Text { color, y, .. } = &mut hidden {
        *color = Color::TRANSPARENT;
        *y = -100_000.0;
    }
    assert_eq!(
        refusal(&[hidden], frame()),
        FallbackReason::new(FallbackKind::TextLimit, Some(0))
    );
    let oversized = text(&"x".repeat(MAX_RUN_BYTES + 1));
    assert_eq!(refusal(&[oversized], frame()).kind, FallbackKind::TextLimit);
    let mut invalid = text("");
    if let DrawCommand::Text { x, color, .. } = &mut invalid {
        *x = f32::NAN;
        *color = Color::TRANSPARENT;
    }
    assert_eq!(
        refusal(&[invalid], frame()),
        FallbackReason::new(FallbackKind::InvalidGeometry, Some(0))
    );
}

#[test]
fn no_outline_repetitions_consume_operations_and_cpu_work() {
    // Audited regular DejaVu cmap: U+200D is an empty zero-advance glyph.
    let commands = [text(&"\u{200d}".repeat(MAX_COMMANDS))];
    let prepared =
        plan_display_list_with_fonts(&commands, &empty(), frame(), &Fonts::new()).unwrap();
    assert_eq!(prepared.stats().lowered_commands, MAX_COMMANDS);
    assert_eq!(prepared.text_stats().preparation.cold_requests, 1);
    assert_eq!(prepared.text_stats().preparation.occurrences, MAX_COMMANDS);
    assert_eq!(
        prepared.text_stats().cpu_pixel_upper_bound,
        MAX_COMMANDS as u64
    );
    assert_eq!(prepared.text_stats().row_entries, 0);
    assert_eq!(prepared.plan().draws().len(), 1);
    assert_eq!(
        refusal(&[text(&"\u{200d}".repeat(MAX_COMMANDS + 1))], frame()).kind,
        FallbackKind::CommandLimit
    );
}

#[test]
fn text_charges_remaining_cpu_allowance_before_mask_allocation() {
    let f = Frame::new(250, 200, 0xffffff);
    let mut commands = vec![
        DrawCommand::Rect {
            rect: crate::graphics::Rect::default(),
            color: Color::TRANSPARENT,
            radius: 0.0
        };
        20
    ];
    commands.push(text("\u{200d}"));
    assert_eq!(
        refusal(&commands, f),
        FallbackReason::new(FallbackKind::CpuPaintBudget, Some(20))
    );
    // An alpha-zero text still validates its input but needs no glyph work.
    if let DrawCommand::Text { color, .. } = &mut commands[20] {
        *color = Color::TRANSPARENT;
    }
    let prepared = plan_display_list_with_fonts(&commands, &empty(), f, &Fonts::new()).unwrap();
    assert_eq!(prepared.text_stats().cpu_pixel_upper_bound, 1_000_000);
    assert_eq!(prepared.text_stats().preparation.cold_requests, 0);
}

#[test]
fn complete_structural_preflight_precedes_all_glyph_preparation() {
    let commands = [
        text("AV"),
        DrawCommand::PushOpacity { opacity: 0.0 },
        DrawCommand::PopOpacity,
    ];
    assert_eq!(
        refusal(&commands, frame()),
        FallbackReason::new(FallbackKind::UnsupportedOpacity, Some(1))
    );
    let commands = [text("A"), DrawCommand::PushFixed, DrawCommand::PopClip];
    assert_eq!(
        refusal(&commands, frame()),
        FallbackReason::new(FallbackKind::InvalidScope, Some(2))
    );
    let commands = [
        text("A"),
        DrawCommand::Rect {
            rect: crate::graphics::Rect::default(),
            color: Color::TRANSPARENT,
            radius: 1.0,
        },
    ];
    assert_eq!(
        refusal(&commands, frame()),
        FallbackReason::new(FallbackKind::UnsupportedRoundedRect, Some(1))
    );
}

#[test]
fn image_and_mask_sources_share_the_existing_entry_budget() {
    let image = Arc::new(RasterImage {
        width: 1,
        height: 1,
        rgba: vec![0, 0, 0, 255],
    });
    let mut images = ImageStore::new();
    for n in 0..MAX_SOURCE_ENTRIES {
        images.insert(format!("{n:03}"), Arc::new((*image).clone()));
    }
    let error = plan_display_list_with_fonts(&[text("\u{200d}")], &images, frame(), &Fonts::new())
        .unwrap_err();
    assert_eq!(
        error,
        FallbackReason::new(FallbackKind::MaskPreparation, Some(0))
    );
    images.clear();
    images.insert(
        "bad-unused".into(),
        Arc::new(RasterImage {
            width: 1,
            height: 1,
            rgba: vec![],
        }),
    );
    assert_eq!(
        plan_display_list_with_fonts(&[text("A")], &images, frame(), &Fonts::new())
            .unwrap_err()
            .kind,
        FallbackKind::InvalidImage
    );
}

#[test]
fn existing_cpu_font_cache_does_not_change_admission_or_packed_plan() {
    let fonts = Fonts::new();
    let commands = [text("AVA To")];
    let cold = plan_display_list_with_fonts(&commands, &empty(), frame(), &fonts).unwrap();
    let mut canvas = crate::graphics::Canvas::new(64, 32).unwrap();
    canvas.paint_with_viewport(&commands, &fonts, &empty(), (0.0, 0.0), (0.0, 0.0));
    assert!(!canvas.exhausted());
    let warm = plan_display_list_with_fonts(&commands, &empty(), frame(), &fonts).unwrap();
    assert_eq!(cold.text_stats().preparation, warm.text_stats().preparation);
    assert_eq!(cold.plan().parameters(), warm.plan().parameters());
    assert_eq!(cold.plan().input_bytes(), warm.plan().input_bytes());
}
