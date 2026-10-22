use super::*;
use crate::css::BackgroundImage;
use crate::graphics::{Canvas, ImageStore, RasterImage};
use std::sync::Arc;

fn prepare(source: &str, width: f32, height: f32) -> (Document, Vec<ComputedStyle>) {
    let doc = Document::parse(source);
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), width, height);
    (doc, styles)
}
fn url(style: &mut ComputedStyle) {
    crate::css::BackgroundData::test_replace_image(
        &mut style.backgrounds,
        Some(BackgroundImage::Url(Arc::new(
            crate::css::url::ResolvedUrl::Absolute(
                ::url::Url::parse("https://example.test/tile.png").unwrap(),
            ),
        ))),
    );
}
fn natural() -> RasterNaturalSize {
    RasterNaturalSize {
        width: 4,
        height: 2,
    }
}
fn loaded(key: &str) -> LoadedBackground<'_> {
    LoadedBackground {
        key,
        natural: natural(),
    }
}
fn center() -> BackgroundPosition {
    BackgroundPosition {
        x: PositionComponent {
            from_end: false,
            offset: Length::Percent(50.0),
        },
        y: PositionComponent {
            from_end: false,
            offset: Length::Percent(50.0),
        },
    }
}
fn size(width: Length, height: Length) -> BackgroundSize {
    BackgroundSize::Explicit { width, height }
}
fn rasters(result: &LayoutResult) -> Vec<(Rect, Rect, f32, [bool; 2], &str)> {
    result
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::BackgroundImage {
                rect,
                tile,
                radius,
                repeat,
                key,
            } => Some((*rect, *tile, *radius, *repeat, key.as_str())),
            DrawCommand::BoxDecoration(value) => match &value.paint {
                crate::graphics::box_decoration::BoxPaint::Image { tile, repeat, key } => {
                    let radius = value.shape.radii[0][0];
                    assert_eq!(value.shape.radii, [[radius; 2]; 4]);
                    Some((value.shape.rect, *tile, radius, *repeat, key.as_str()))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}
fn store() -> ImageStore {
    let mut images = ImageStore::new();
    images.insert(
        "tile".into(),
        Arc::new(RasterImage {
            width: 4,
            height: 2,
            rgba: [
                255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255,
            ]
            .repeat(2),
        }),
    );
    images.insert(
        "blue".into(),
        Arc::new(RasterImage {
            width: 4,
            height: 2,
            rgba: [0, 0, 255, 255].repeat(8),
        }),
    );
    images
}
fn pixels(result: &LayoutResult, width: u32, height: u32) -> Vec<u32> {
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.paint(&result.commands, &Fonts::new(), &store(), 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas.pixels
}
fn engine<'a>(
    doc: &'a Document,
    styles: &'a [ComputedStyle],
    fonts: &'a Fonts,
    backgrounds: &'a [Option<LoadedBackground<'a>>],
    ranges: &'a [BackgroundBindingRange],
) -> Engine<'a> {
    Engine {
        doc,
        styles,
        fonts,
        backgrounds: BackgroundBindings {
            ranges,
            slots: backgrounds,
        },
        background_key_bytes_left: 65_536,
        viewport: Size {
            width: 100.0,
            height: 100.0,
        },
        commands: Vec::new(),
        owners: Vec::new(),
        hits: Vec::new(),
        boxes: Vec::new(),
        positioned: Vec::new(),
        paint_owner: PaintOwner {
            node: doc.root,
            background: false,
            paint_visibility: Visibility::Visible,
            inline_layer: InlineLayer::None,
            inline_prefix: 0.0,
            inline_sides: 0,
            background_image_index: 0,
        },
        flow_height_reference: Some(100.0),
        content_height_measure: None,
        position_work_left: MAX_POSITION_WORK,
        visits: 0,
        hits_created: 0,
        glyphs_left: MAX_GLYPHS,
        emitted_glyphs_left: MAX_GLYPHS,
        intrinsic_work_left: Counter::new(MAX_GLYPHS),
        flex_work_left: MAX_FLEX_WORK,
        float_work_left: MAX_FLOAT_WORK,
        grid_work_left: MAX_GRID_WORK,
        grid_height_reference: None,
        floats_left: MAX_FLOATS,
        floats: FloatContext::default(),
        fragment_root: None,
        commands_created: 0,
        open_clips: 0,
        canvas_background_node: None,
        inline_gradients: false,
        fallback: ComputedStyle::default(),
        disclosure_hidden: doc.disclosure_hidden_mask(),
    }
}

#[test]
fn raster_tile_natural_ratio_sizes_positions_and_used_clamps_are_literal() {
    let area = rect(10.0, 20.0, 12.0, 6.0);
    let rows = [
        (
            BackgroundSize::default(),
            BackgroundPosition::default(),
            rect(10.0, 20.0, 4.0, 2.0),
        ),
        (
            BackgroundSize::default(),
            center(),
            rect(14.0, 22.0, 4.0, 2.0),
        ),
        (
            size(Length::Px(8.0), Length::Auto),
            center(),
            rect(12.0, 21.0, 8.0, 4.0),
        ),
        (
            size(Length::Auto, Length::Px(6.0)),
            center(),
            rect(10.0, 20.0, 12.0, 6.0),
        ),
        (
            size(Length::Percent(50.0), Length::Auto),
            center(),
            rect(13.0, 21.5, 6.0, 3.0),
        ),
        (
            size(
                Length::Calc {
                    px: -2.0,
                    percent: 50.0,
                    has_percent: true,
                },
                Length::Auto,
            ),
            center(),
            rect(14.0, 22.0, 4.0, 2.0),
        ),
        (
            size(Length::Px(8.0), Length::Px(6.0)),
            center(),
            rect(12.0, 20.0, 8.0, 6.0),
        ),
        (
            size(Length::Px(16.0), Length::Auto),
            center(),
            rect(8.0, 19.0, 16.0, 8.0),
        ),
    ];
    for (size, position, expected) in rows {
        assert_eq!(
            background_tile_with_raster(area, size, position, natural()),
            Some(expected)
        );
    }
    let percent = BackgroundPosition {
        x: PositionComponent {
            from_end: false,
            offset: Length::Percent(25.0),
        },
        y: PositionComponent {
            from_end: false,
            offset: Length::Percent(75.0),
        },
    };
    let edge = BackgroundPosition {
        x: PositionComponent {
            from_end: true,
            offset: Length::Px(2.0),
        },
        y: PositionComponent {
            from_end: true,
            offset: Length::Px(1.0),
        },
    };
    assert_eq!(
        background_tile_with_raster(area, BackgroundSize::default(), percent, natural()),
        Some(rect(12.0, 23.0, 4.0, 2.0))
    );
    assert_eq!(
        background_tile_with_raster(area, BackgroundSize::default(), edge, natural()),
        Some(rect(16.0, 23.0, 4.0, 2.0))
    );
    assert_eq!(
        background_tile_with_raster(
            rect(0.0, 0.0, 6.0, 6.0),
            BackgroundSize::Contain,
            center(),
            natural()
        ),
        Some(rect(0.0, 1.5, 6.0, 3.0))
    );
    assert_eq!(
        background_tile_with_raster(
            rect(0.0, 0.0, 6.0, 6.0),
            BackgroundSize::Cover,
            center(),
            natural()
        ),
        Some(rect(-3.0, 0.0, 12.0, 6.0))
    );
    for size in [
        size(Length::Px(0.0), Length::Auto),
        size(Length::Auto, Length::Px(0.0)),
        size(
            Length::Calc {
                px: -4.0,
                percent: 25.0,
                has_percent: true,
            },
            Length::Auto,
        ),
    ] {
        assert_eq!(
            background_tile_with_raster(area, size, center(), natural()),
            None
        );
    }
}

#[test]
fn raster_tile_rejects_invalid_natural_and_unrepresentable_geometry() {
    for natural in [
        RasterNaturalSize {
            width: 0,
            height: 2,
        },
        RasterNaturalSize {
            width: 4,
            height: 0,
        },
    ] {
        assert_eq!(
            background_tile_with_raster(
                rect(0.0, 0.0, 4.0, 2.0),
                BackgroundSize::default(),
                center(),
                natural
            ),
            None
        );
    }
    let mut position = BackgroundPosition::default();
    position.x.offset = Length::Px(1_000_000.0);
    assert_eq!(
        background_tile_with_raster(
            rect(0.0, 0.0, 4.0, 2.0),
            BackgroundSize::default(),
            position,
            natural()
        ),
        Some(rect(1_000_000.0, 0.0, 4.0, 2.0))
    );
    assert_eq!(
        background_tile_with_raster(
            rect(2.0, 0.0, 4.0, 2.0),
            BackgroundSize::default(),
            position,
            natural()
        ),
        None
    );
    for size in [
        size(Length::Px(f32::INFINITY), Length::Auto),
        size(Length::Fr(1.0), Length::Auto),
        size(Length::Px(1_000_000.0), Length::Px(1_000_001.0)),
    ] {
        assert_eq!(
            background_tile_with_raster(rect(0.0, 0.0, 4.0, 2.0), size, center(), natural()),
            None
        );
    }
    // Zero positioning height does not erase a raster's actual natural size.
    assert_eq!(
        background_tile_with_raster(
            rect(0.0, 0.0, 4.0, 0.0),
            BackgroundSize::default(),
            BackgroundPosition::default(),
            natural()
        ),
        Some(rect(0.0, 0.0, 4.0, 2.0))
    );
    assert_eq!(
        background_tile_with_raster(
            rect(0.0, 0.0, 4.0, 0.0),
            BackgroundSize::Contain,
            center(),
            natural()
        ),
        None
    );
}

#[test]
fn principal_url_uses_decoded_dimensions_padding_phase_and_preserves_hits() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:4px;height:2px;padding:1px;border:1px solid transparent}</style><div id=a data-eris-natural-width=999 data-eris-natural-height=777></div>",
        8.0,
        6.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let baseline = layout(&doc, &styles, 8.0, 6.0, &Fonts::new());
    url(&mut styles[a]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    let result = layout_with_backgrounds(&doc, &styles, 8.0, 6.0, &Fonts::new(), &bindings);
    assert_eq!(
        rasters(&result),
        vec![(
            rect(0.0, 0.0, 8.0, 6.0),
            rect(1.0, 1.0, 4.0, 2.0),
            0.0,
            [true, true],
            "tile"
        )]
    );
    assert_eq!(
        pixels(&result, 8, 6),
        [
            0x0000ff, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0xff0000, 0xff0000, 0x0000ff
        ]
        .repeat(6)
    );
    assert_eq!(
        result
            .hit_regions
            .iter()
            .map(|h| (h.node, h.rect, h.fixed))
            .collect::<Vec<_>>(),
        baseline
            .hit_regions
            .iter()
            .map(|h| (h.node, h.rect, h.fixed))
            .collect::<Vec<_>>()
    );
    assert_eq!(doc.attr(a, "data-eris-natural-width"), Some("999"));
    assert!(rasters(&layout(&doc, &styles, 8.0, 6.0, &Fonts::new())).is_empty());
}

#[test]
fn canvas_uses_root_area_and_unavailable_root_keeps_body_local() {
    let (doc, mut styles) = prepare(
        "<style>html{width:8px;height:4px;box-sizing:border-box;padding:1px;border:1px solid transparent}body{margin:0;width:2px;height:2px}</style>",
        12.0,
        8.0,
    );
    let html = doc.query_selector("html").unwrap();
    let body = doc.query_selector("body").unwrap();
    url(&mut styles[body]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[body] = Some(loaded("tile"));
    let result = layout_with_backgrounds(&doc, &styles, 12.0, 8.0, &Fonts::new(), &bindings);
    assert_eq!(
        rasters(&result),
        vec![(
            rect(0.0, 0.0, 12.0, 8.0),
            rect(1.0, 1.0, 4.0, 2.0),
            0.0,
            [true, true],
            "tile"
        )]
    );
    assert_eq!(
        pixels(&result, 12, 8),
        [
            0x0000ff, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0xff0000, 0xff0000, 0x0000ff,
            0x0000ff, 0xff0000, 0xff0000, 0x0000ff
        ]
        .repeat(8)
    );
    crate::css::BackgroundData::test_replace_image(
        &mut styles[html].backgrounds,
        Some(BackgroundImage::Url(Arc::new(
            crate::css::url::ResolvedUrl::Empty,
        ))),
    );
    let result = layout_with_backgrounds(&doc, &styles, 12.0, 8.0, &Fonts::new(), &bindings);
    assert_eq!(
        rasters(&result),
        vec![(
            rect(2.0, 2.0, 2.0, 2.0),
            rect(2.0, 2.0, 4.0, 2.0),
            0.0,
            [true, true],
            "tile"
        )]
    );
    let p = pixels(&result, 12, 8);
    assert_eq!((p[10], p[26]), (0xffffff, 0xff0000));
}

#[test]
fn unavailable_inline_color_and_loaded_virtual_strip_have_literal_pixels() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0;width:4px;font-size:1px;line-height:2px}i{display:inline-block;width:2px;height:2px;vertical-align:top}span{background:lime;background-size:100% auto}</style><span id=a><i></i><i></i><i></i><i></i></span>",
        4.0,
        4.0,
    );
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    let empty = vec![None; doc.nodes.len()];
    let result = layout_with_backgrounds(&doc, &styles, 4.0, 4.0, &Fonts::new(), &empty);
    assert!(rasters(&result).is_empty());
    assert_eq!(
        pixels(&result, 4, 4),
        [
            vec![0xffffff; 4],
            vec![0x00ff00; 4],
            vec![0xc0ffc0; 4],
            vec![0x00ff00; 4]
        ]
        .concat()
    );
    let mut bindings = empty;
    bindings[a] = Some(loaded("tile"));
    let result = layout_with_backgrounds(&doc, &styles, 4.0, 4.0, &Fonts::new(), &bindings);
    assert_eq!(
        pixels(&result, 4, 4),
        [
            vec![0xffffff; 4],
            vec![0xff0000; 4],
            vec![0xd0c091; 4],
            vec![0x0000ff; 4]
        ]
        .concat()
    );
    let images = rasters(&result);
    assert_eq!(images.len(), 2);
    assert_eq!(
        images.iter().map(|v| v.1).collect::<Vec<_>>(),
        vec![rect(0.0, 1.05, 8.0, 4.0), rect(-4.0, 3.05, 8.0, 4.0)]
    );
}

#[test]
fn nested_inline_url_background_order_is_ancestor_then_child() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0;width:6px;font-size:1px;line-height:2px}i{display:inline-block;width:2px;height:2px;vertical-align:top}span{background-size:100% auto}</style><span id=a><i></i><b id=b><i></i><i></i></b></span>",
        6.0,
        2.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let b = doc.query_selector("#b").unwrap();
    url(&mut styles[a]);
    url(&mut styles[b]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    bindings[b] = Some(loaded("blue"));
    let result = layout_with_backgrounds(&doc, &styles, 6.0, 2.0, &Fonts::new(), &bindings);
    assert_eq!(
        pixels(&result, 6, 2),
        [
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xff0000, 0xff0000,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff
        ]
    );
}

#[test]
fn inline_pending_slots_are_private_balanced_and_budget_refusal_keeps_color() {
    let (doc, mut styles) = prepare("<span id=a></span>", 4.0, 2.0);
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    styles[a].background_color = Color::rgb(0, 255, 0);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    let fonts = Fonts::new();
    let area = rect(0.0, 0.0, 4.0, 2.0);
    let geometry = BoxGeometry {
        node: a,
        border: area,
        padding: area,
        content: area,
        fixed: false,
        inline: true,
        inline_prefix: 0.0,
        inline_sides: 3,
        inline_basis: 4.0,
    };
    let background_ranges = singleton_ranges(bindings.len());
    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.boxes.push(geometry);
    e.commands_created = MAX_COMMANDS - 4;
    e.paint_inline_background(geometry, 0.0, 3);
    assert!(e.commands.is_empty());
    assert_eq!(e.open_clips, 0);
    e.commands_created = MAX_COMMANDS - 5;
    e.paint_inline_background(geometry, 0.0, 3);
    assert_eq!((e.commands.len(), e.open_clips), (5, 0));
    assert!(matches!(e.commands[2], DrawCommand::Rect {color,..} if color == Color::TRANSPARENT));
    assert!(e.owners[2].inline_layer == InlineLayer::Image);
    // 5 classification + node table + 8 geometry + 5 slots + 4*16 shapes.
    let scan = doc.nodes.len() + 82;
    e.position_work_left = scan - 1;
    e.finish_inline_backgrounds();
    assert_eq!(e.position_work_left, scan - 6);
    assert!(matches!(e.commands[2], DrawCommand::Rect { .. }));
    assert!(!e.inline_gradients);

    // One-shot failure uses a fresh preparation, not a runtime budget refill.
    let background_ranges = singleton_ranges(bindings.len());
    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.boxes.push(geometry);
    e.paint_inline_background(geometry, 0.0, 3);
    e.position_work_left = scan + 4;
    e.finish_inline_backgrounds();
    assert_eq!(e.position_work_left, 4);
    assert_eq!(e.background_key_bytes_left, 65_536);
    assert!(matches!(e.commands[2], DrawCommand::Rect { .. }));
    // A fresh preparation with exact scan+key work resolves one real image.
    let background_ranges = singleton_ranges(bindings.len());
    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.boxes.push(geometry);
    e.paint_inline_background(geometry, 0.0, 3);
    e.position_work_left = scan + 5;
    e.finish_inline_backgrounds();
    assert_eq!(
        (e.position_work_left, e.background_key_bytes_left),
        (0, 65_532)
    );
    assert!(matches!(&e.commands[2], DrawCommand::BoxDecoration(value)
        if matches!(&value.paint, crate::graphics::box_decoration::BoxPaint::Image {key,..} if key == "tile")));
    let background_ranges = singleton_ranges(bindings.len());
    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.boxes.push(geometry);
    e.paint_inline_background(geometry, 0.0, 3);
    e.background_key_bytes_left = 0;
    e.finish_inline_backgrounds();
    e.restack();
    let result = LayoutResult {
        commands: e.commands,
        hit_regions: e.hits,
        content_height: 2.0,
        node_visibility: vec![Visibility::Visible; doc.nodes.len()],
    };
    assert!(rasters(&result).is_empty());
    assert_eq!(pixels(&result, 4, 2), vec![0x00ff00; 8]);
}

#[test]
fn background_key_utf8_caps_work_cuts_and_no_refund_are_exact() {
    let key = "x".repeat(4096);
    let mut bytes = 65_536;
    let mut work = 65_552;
    for _ in 0..16 {
        assert_eq!(
            clone_background_key(&key, &mut bytes, &mut work).as_deref(),
            Some(key.as_str())
        );
    }
    assert_eq!((bytes, work), (0, 0));
    assert!(clone_background_key("x", &mut bytes, &mut work).is_none());
    let mut bytes = 65_536;
    let mut work = 20_000;
    assert!(clone_background_key(&"x".repeat(4097), &mut bytes, &mut work).is_none());
    assert!(clone_background_key("", &mut bytes, &mut work).is_none());
    assert_eq!((bytes, work), (65_536, 20_000));
    for (room, fee, success, left_bytes, left_work) in
        [(2, 3, true, 0, 0), (1, 3, false, 1, 3), (2, 2, false, 2, 2)]
    {
        let (mut bytes, mut work) = (room, fee);
        assert_eq!(
            clone_background_key("é", &mut bytes, &mut work).is_some(),
            success
        );
        assert_eq!((bytes, work), (left_bytes, left_work));
    }
    let (doc, mut styles) = prepare("<div id=a></div>", 4.0, 2.0);
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    let fonts = Fonts::new();
    let background_ranges = singleton_ranges(bindings.len());
    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.commands_created = MAX_COMMANDS;
    let command = e
        .background_command(
            a,
            0,
            rect(0.0, 0.0, 4.0, 2.0),
            rect(0.0, 0.0, 4.0, 2.0),
            0.0,
            Color::BLACK,
            false,
        )
        .unwrap();
    assert_eq!(
        (e.background_key_bytes_left, e.position_work_left),
        (65_532, MAX_POSITION_WORK - 5)
    );
    assert!(!e.push(command));
    assert_eq!(
        (e.background_key_bytes_left, e.position_work_left),
        (65_532, MAX_POSITION_WORK - 5)
    );
}

#[test]
fn table_row_and_stretched_cell_recompute_raster_tile_from_final_height() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}table{width:14px}tr{height:6px}td{width:4px;height:2px;padding:0;vertical-align:bottom}i{display:block;width:1px;height:1px}</style><table><tr id=r><td id=a><i id=c></i></td><td></td></tr></table>",
        14.0,
        10.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let r = doc.query_selector("#r").unwrap();
    let c = doc.query_selector("#c").unwrap();
    let baseline = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    url(&mut styles[a]);
    url(&mut styles[r]);
    crate::css::BackgroundData::test_replace_size(
        &mut styles[a].backgrounds,
        size(Length::Auto, Length::Percent(100.0)),
    );
    crate::css::BackgroundData::test_replace_position(&mut styles[a].backgrounds, center());
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    bindings[r] = Some(loaded("blue"));
    let result = layout_with_backgrounds(&doc, &styles, 14.0, 10.0, &Fonts::new(), &bindings);
    let images = rasters(&result);
    for paint in [rect(2.0, 2.0, 4.0, 6.0), rect(8.0, 2.0, 4.0, 6.0)] {
        assert!(images.contains(&(paint, rect(2.0, 2.0, 4.0, 2.0), 0.0, [true, true], "blue")));
    }
    assert!(images.contains(&(
        rect(2.0, 2.0, 4.0, 6.0),
        rect(-2.0, 2.0, 12.0, 6.0),
        0.0,
        [true, true],
        "tile"
    )));
    assert_eq!(
        result
            .hit_regions
            .iter()
            .map(|h| (h.node, h.rect))
            .collect::<Vec<_>>(),
        baseline
            .hit_regions
            .iter()
            .map(|h| (h.node, h.rect))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        result
            .hit_regions
            .iter()
            .find(|h| h.node == c)
            .unwrap()
            .rect,
        rect(2.0, 6.0, 1.0, 1.0)
    );
}

#[test]
fn flex_and_grid_reflow_use_final_geometry_without_moving_hits() {
    for rule in [
        "display:flex;align-items:stretch",
        "display:grid;grid-template-columns:6px 6px;grid-template-rows:6px",
    ] {
        let (doc, mut styles) = prepare(
            &format!(
                "<style>body{{margin:0}}main{{width:12px;height:6px;{rule}}}i{{display:block;width:6px}}</style><main><i id=a></i><i></i></main>"
            ),
            12.0,
            6.0,
        );
        let a = doc.query_selector("#a").unwrap();
        let baseline = layout(&doc, &styles, 12.0, 6.0, &Fonts::new());
        url(&mut styles[a]);
        crate::css::BackgroundData::test_replace_size(
            &mut styles[a].backgrounds,
            BackgroundSize::Contain,
        );
        crate::css::BackgroundData::test_replace_position(&mut styles[a].backgrounds, center());
        crate::css::BackgroundData::test_replace_repeat(&mut styles[a].backgrounds, [false, false]);
        let mut bindings = vec![None; doc.nodes.len()];
        bindings[a] = Some(loaded("tile"));
        let result = layout_with_backgrounds(&doc, &styles, 12.0, 6.0, &Fonts::new(), &bindings);
        assert!(rasters(&result).contains(&(
            rect(0.0, 0.0, 6.0, 6.0),
            rect(0.0, 1.5, 6.0, 3.0),
            0.0,
            [false, false],
            "tile"
        )));
        assert_eq!(
            result
                .hit_regions
                .iter()
                .map(|h| (h.node, h.rect))
                .collect::<Vec<_>>(),
            baseline
                .hit_regions
                .iter()
                .map(|h| (h.node, h.rect))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn url_pixels_do_not_prove_opaque_rounded_interior() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:8px;height:8px;border:2px solid red;border-radius:4px;background-repeat:no-repeat;background-size:0px 0px}</style><div id=a></div>",
        12.0,
        12.0,
    );
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    let fonts = Fonts::new();
    assert!(
        !engine(
            &doc,
            &styles,
            &fonts,
            &bindings,
            &singleton_ranges(bindings.len())
        )
        .rounded_border(a)
    );
    let result = layout_with_backgrounds(&doc, &styles, 12.0, 12.0, &fonts, &bindings);
    assert!(rasters(&result).is_empty());
    assert_eq!(pixels(&result, 12, 12)[6 * 12 + 6], 0xffffff);
    styles[a].background_color = Color::rgb(0, 255, 0);
    crate::css::BackgroundData::test_replace_size(
        &mut styles[a].backgrounds,
        BackgroundSize::default(),
    );
    assert!(
        engine(
            &doc,
            &styles,
            &fonts,
            &bindings,
            &singleton_ranges(bindings.len())
        )
        .rounded_border(a)
    );
    let result = layout_with_backgrounds(&doc, &styles, 12.0, 12.0, &fonts, &bindings);
    assert_eq!(
        rasters(&result),
        vec![(
            rect(2.0, 2.0, 8.0, 8.0),
            rect(2.0, 2.0, 4.0, 2.0),
            2.0,
            [false, false],
            "tile"
        )]
    );
    assert_eq!(pixels(&result, 12, 12)[6 * 12 + 6], 0x00ff00);
}

#[test]
fn fixed_url_background_escapes_ancestor_clip_but_keeps_opacity() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}main{opacity:.5;overflow:hidden;width:1px;height:1px;margin-top:100px}i{position:fixed;left:2px;top:2px;width:4px;height:4px}</style><main><i id=a></i></main>",
        10.0,
        10.0,
    );
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    let result = layout_with_backgrounds(&doc, &styles, 10.0, 10.0, &Fonts::new(), &bindings);
    assert_eq!(
        rasters(&result),
        vec![(
            rect(2.0, 2.0, 4.0, 4.0),
            rect(2.0, 2.0, 4.0, 2.0),
            0.0,
            [true, true],
            "tile"
        )]
    );
    let mut canvas = Canvas::new(10, 10).unwrap();
    canvas.paint_with_viewport(
        &result.commands,
        &Fonts::new(),
        &store(),
        (0.0, -100.0),
        (0.0, 0.0),
    );
    assert_eq!((canvas.pixels[22], canvas.pixels[24]), (0xff8080, 0x8080ff));
    assert!(result.hit_regions.iter().any(|h| h.node == a && h.fixed));
}

#[test]
fn url_principal_entry_reservation_and_missing_binding_are_color_safe() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}</style><div id=a><div id=child></div></div>",
        4.0,
        2.0,
    );
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    styles[a].height = Length::Px(2.0);
    styles[a].background_color = Color::rgb(0, 255, 0);
    let fonts = Fonts::new();
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded("tile"));
    for (remaining, count) in [(8, 0), (9, 6)] {
        let background_ranges = singleton_ranges(bindings.len());
        let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
        e.commands_created = MAX_COMMANDS - remaining;
        e.layout_box(a, 0.0, 0.0, 4.0, Some(4.0), 0);
        assert_eq!(e.commands.len(), count);
        assert_eq!(e.open_clips, 0);
        assert!(e.commands_created <= MAX_COMMANDS);
        assert!(
            e.owners
                .iter()
                .all(|owner| owner.node == a && owner.background)
        );
        if count != 0 {
            assert!(matches!(&e.commands[1],DrawCommand::BackgroundImage{key,..} if key=="tile"));
            assert_eq!(e.hits.len(), 1);
        }
    }
    bindings[a] = None;
    let result = layout_with_backgrounds(&doc, &styles, 4.0, 2.0, &fonts, &bindings);
    assert!(rasters(&result).is_empty());
    assert_eq!(pixels(&result, 4, 2), vec![0x00ff00; 8]);
}

// Adapt legacy singleton fixtures to the borrowed, layer-indexed binding view.
fn singleton_ranges(count: usize) -> Vec<BackgroundBindingRange> {
    (0..count)
        .map(|index| BackgroundBindingRange {
            start: u32::try_from(index).unwrap(),
            len: 1,
        })
        .collect()
}
fn layout_with_backgrounds(
    doc: &Document,
    styles: &[ComputedStyle],
    width: f32,
    height: f32,
    fonts: &Fonts,
    slots: &[Option<LoadedBackground<'_>>],
) -> LayoutResult {
    let ranges = singleton_ranges(slots.len());
    crate::layout::layout_with_backgrounds(
        doc,
        styles,
        width,
        height,
        fonts,
        BackgroundBindings {
            ranges: &ranges,
            slots,
        },
    )
}
