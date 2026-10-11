use super::*;
use crate::graphics::gradient::{
    GradientColor, GradientDirection, GradientPosition, GradientStop, LinearGradient,
};
use crate::graphics::{Canvas, ImageStore};
use std::sync::Arc;

fn split(first: Color, last: Color) -> Arc<LinearGradient> {
    Arc::new(LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: [(first, 0.0), (first, 50.0), (last, 50.0), (last, 100.0)]
            .map(|(color, percent)| GradientStop {
                color: GradientColor::Color(color),
                position: Some(GradientPosition { px: 0.0, percent }),
            })
            .into(),
    })
}

fn prepare(source: &str, width: f32, height: f32) -> (Document, Vec<ComputedStyle>) {
    let doc = Document::parse(source);
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), width, height);
    (doc, styles)
}

fn pixels(result: &LayoutResult, width: u32, height: u32) -> Vec<u32> {
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.paint(
        &result.commands,
        &Fonts::new(),
        &ImageStore::new(),
        0.0,
        0.0,
    );
    assert!(!canvas.exhausted());
    canvas.pixels
}

fn gradients(result: &LayoutResult) -> Vec<(Rect, Rect, f32, Color)> {
    result
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::LinearGradient {
                rect,
                tile,
                radius,
                current_color,
                stop_scale,
                ..
            } => {
                assert_eq!(*stop_scale, 1.0);
                Some((*rect, *tile, *radius, *current_color))
            }
            DrawCommand::BoxDecoration(value) => match &value.paint {
                crate::graphics::box_decoration::BoxPaint::LinearGradient {
                    tile,
                    current_color,
                    stop_scale,
                    ..
                } => {
                    assert_eq!(*stop_scale, 1.0);
                    let radius = value.shape.radii[0][0];
                    assert_eq!(value.shape.radii, [[radius; 2]; 4]);
                    Some((value.shape.rect, *tile, radius, *current_color))
                }
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn engine<'a>(doc: &'a Document, styles: &'a [ComputedStyle], fonts: &'a Fonts) -> Engine<'a> {
    Engine {
        doc,
        styles,
        fonts,
        backgrounds: &[],
        background_key_bytes_left: MAX_BACKGROUND_KEY_BYTES,
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
            inline_layer: InlineLayer::None,
            inline_prefix: 0.0,
            inline_sides: 0,
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
fn padding_tile_repeats_under_transparent_borders_and_keeps_color_below_image() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:4px;height:2px;padding:1px;border:1px solid transparent}</style><div id=a></div>",
        8.0,
        6.0,
    );
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    let result = layout(&doc, &styles, 8.0, 6.0, &Fonts::new());
    let g = gradients(&result);
    assert_eq!(
        (g[0].0, g[0].1),
        (rect(0.0, 0.0, 8.0, 6.0), rect(1.0, 1.0, 6.0, 4.0))
    );
    let row = [
        0x0000ff, 0xff0000, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0x0000ff, 0xff0000,
    ];
    assert_eq!(pixels(&result, 8, 6), row.repeat(6));

    styles[a].padding = crate::css::Edges::all(Length::Px(0.0));
    styles[a].border_width = crate::css::Edges::all(0.0);
    styles[a].background_color = Color::rgb(0, 255, 0);
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::TRANSPARENT,
        Color::rgb(0, 0, 255),
    )));
    let result = layout(&doc, &styles, 4.0, 2.0, &Fonts::new());
    assert_eq!(
        pixels(&result, 4, 2),
        [0x00ff00, 0x00ff00, 0x0000ff, 0x0000ff].repeat(2)
    );
}

#[test]
fn rounded_shortcut_requires_opaque_border_and_combined_interior() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}#a{width:8px;height:8px;border:2px solid red;border-radius:4px}</style><div id=a></div>",
        12.0,
        12.0,
    );
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(0, 255, 0),
        Color::rgb(0, 0, 255),
    )));
    let fonts = Fonts::new();
    assert!(engine(&doc, &styles, &fonts).rounded_border(a));
    let result = layout(&doc, &styles, 12.0, 12.0, &fonts);
    let g = gradients(&result);
    assert_eq!(
        (g[0].0, g[0].1, g[0].2),
        (rect(2.0, 2.0, 8.0, 8.0), rect(2.0, 2.0, 8.0, 8.0), 2.0)
    );
    let p = pixels(&result, 12, 12);
    assert_eq!(
        [p[0], p[6], p[6 * 12 + 4], p[6 * 12 + 8]],
        [0xffffff, 0xff0000, 0x00ff00, 0x0000ff]
    );
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::TRANSPARENT,
        Color::rgb(0, 0, 255),
    )));
    assert!(!engine(&doc, &styles, &fonts).rounded_border(a));
    styles[a].background_color = Color::rgb(0, 255, 0);
    assert!(engine(&doc, &styles, &fonts).rounded_border(a));
    styles[a].border_color.a = 128;
    assert!(!engine(&doc, &styles, &fonts).rounded_border(a));
    styles[a].border_color.a = 255;
    styles[a].border_width.left = 1.0;
    assert!(!engine(&doc, &styles, &fonts).rounded_border(a));
    // These last two combinations retain the existing general border
    // limitation; this test does not claim a conformant rounded-ring raster.
}

#[test]
fn canvas_uses_root_padding_tile_and_transparent_root_image_blocks_body_propagation() {
    let (doc, mut styles) = prepare(
        "<style>html{width:8px;height:4px;box-sizing:border-box;padding:1px;border:1px solid transparent}body{margin:0;width:2px;height:2px}</style>",
        12.0,
        8.0,
    );
    let html = doc.query_selector("html").unwrap();
    let body = doc.query_selector("body").unwrap();
    styles[body].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    let g = gradients(&result);
    assert_eq!(g.len(), 1);
    assert_eq!(
        (g[0].0, g[0].1),
        (rect(0.0, 0.0, 12.0, 8.0), rect(1.0, 1.0, 6.0, 2.0))
    );
    assert_eq!(
        &pixels(&result, 12, 8)[..12],
        &[
            0x0000ff, 0xff0000, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0x0000ff, 0xff0000,
            0xff0000, 0xff0000, 0x0000ff, 0x0000ff
        ]
    );
    styles[html].background_image = Some(BackgroundImage::Gradient(split(
        Color::TRANSPARENT,
        Color::TRANSPARENT,
    )));
    styles[body].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(0, 0, 255),
        Color::rgb(0, 0, 255),
    )));
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert_eq!(gradients(&result).len(), 2);
    let p = pixels(&result, 12, 8);
    assert_eq!((p[10], p[2 * 12 + 2]), (0xffffff, 0x0000ff));
    styles[html].background_image = None;
    styles[html].opacity = 0.5;
    styles[body].opacity = 0.0;
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert_eq!(pixels(&result, 12, 8)[10], 0x8080ff);
    styles[html].opacity = 1.0;
    styles[html].padding = crate::css::Edges::all(Length::Px(0.0));
    styles[html].border_width = crate::css::Edges::all(0.0);
    styles[html].height = Length::Px(0.0);
    styles[html].background_color = Color::rgb(0, 255, 0);
    styles[html].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    styles[body].background_image = None;
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert!(gradients(&result).is_empty());
    assert_eq!(pixels(&result, 12, 8), vec![0x00ff00; 96]);
}

#[test]
fn propagated_body_gradient_resolves_current_color_on_root_with_root_padding_tile() {
    let (doc, styles) = prepare(
        "<style>html{margin:0;width:8px;height:4px;box-sizing:border-box;padding:1px;border:1px solid transparent;color:red}body{margin:0;width:2px;height:2px;color:blue;background-image:linear-gradient(to right,currentColor 50%,lime 50%)}</style>",
        12.0,
        8.0,
    );
    let html = doc.query_selector("html").unwrap();
    let body = doc.query_selector("body").unwrap();
    assert_eq!(styles[html].color, Color::rgb(255, 0, 0));
    assert_eq!(styles[body].color, Color::rgb(0, 0, 255));
    assert!(styles[html].background_image.is_none());
    assert_eq!(styles[html].background_color.a, 0);
    let Some(BackgroundImage::Gradient(source)) = styles[body].background_image.as_ref() else {
        panic!("expected gradient")
    };
    assert!(matches!(source.stops[0].color, GradientColor::CurrentColor));
    let result = layout(&doc, &styles, 12.0, 8.0, &Fonts::new());
    assert_eq!(
        gradients(&result),
        vec![(
            rect(0.0, 0.0, 12.0, 8.0),
            rect(1.0, 1.0, 6.0, 2.0),
            0.0,
            Color::rgb(255, 0, 0),
        )]
    );
    // The body's computed image remains intact but has no local used image.
    // Root tile repeats across the canvas, including through the body box.
    assert!(styles[body].background_image.is_some());
    let row = [
        0x00ff00, 0xff0000, 0xff0000, 0xff0000, 0x00ff00, 0x00ff00, 0x00ff00, 0xff0000, 0xff0000,
        0xff0000, 0x00ff00, 0x00ff00,
    ];
    assert_eq!(pixels(&result, 12, 8), row.repeat(8));
}

#[test]
fn inline_slice_continues_through_three_atomic_fragments_and_wrapping() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0;width:4px;font-size:1px;line-height:2px}i{display:inline-block;width:2px;height:2px;vertical-align:top}</style><span id=a><i></i><i></i><i></i></span>",
        4.0,
        4.0,
    );
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    let result = layout(&doc, &styles, 4.0, 4.0, &Fonts::new());
    let g = gradients(&result);
    assert_eq!(
        g.iter().map(|v| v.1).collect::<Vec<_>>(),
        [rect(0.0, 1.05, 6.0, 1.3), rect(-4.0, 3.05, 6.0, 1.3)]
    );
    assert_eq!(
        pixels(&result, 4, 4),
        [
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xff0000, 0xff0000, 0xff0000, 0x0000ff,
            0xffc0c0, 0xffc0c0, 0xffc0c0, 0xc0c0ff, 0x0000ff, 0x0000ff, 0xffffff, 0xffffff
        ]
    );
}

#[test]
fn nested_inline_uses_ancestor_color_and_nominal_owner_height() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0;width:6px;font-size:1px;line-height:2px}i{display:inline-block;width:2px;height:2px;vertical-align:top}span{color:red}b{color:blue}</style><span id=a><i></i><b><i></i><i></i></b></span>",
        6.0,
        2.0,
    );
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(Arc::new(LinearGradient {
        direction: GradientDirection::Angle(0.0),
        stops: [GradientStop {
            color: GradientColor::CurrentColor,
            position: None,
        }]
        .into(),
    })));
    let result = layout(&doc, &styles, 6.0, 2.0, &Fonts::new());
    assert_eq!(gradients(&result).len(), 1);
    assert!(
        gradients(&result)
            .iter()
            .all(|g| g.3 == Color::rgb(255, 0, 0))
    );
    assert_eq!(
        pixels(&result, 6, 2),
        [vec![0xffffff; 6], vec![0xff0000; 6]].concat()
    );
    let nested = doc.query_selector("b").unwrap();
    styles[nested].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(0, 0, 255),
        Color::rgb(0, 0, 255),
    )));
    let result = layout(&doc, &styles, 6.0, 2.0, &Fonts::new());
    assert_eq!(
        pixels(&result, 6, 2),
        [
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xff0000, 0xff0000,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff
        ]
    );
}

#[test]
fn table_cell_background_stretches_while_bottom_aligned_child_moves() {
    // Existing table layout has fixed 2px spacing, not CSS border-spacing.
    // Width14 pays 4+4 cells and three2px gaps; the row spans both cells+gap.
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}table{width:14px}tr{height:6px}td{width:4px;height:2px;padding:0;vertical-align:bottom}i{display:block;width:1px;height:1px}</style><table><tr id=r><td id=a><i id=c></i></td><td></td></tr></table>",
        14.0,
        10.0,
    );
    let a = doc.query_selector("#a").unwrap();
    let row = doc.query_selector("#r").unwrap();
    let child = doc.query_selector("#c").unwrap();
    let baseline = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    for (node, expected) in [
        (row, rect(2.0, 2.0, 10.0, 6.0)),
        (a, rect(2.0, 2.0, 4.0, 6.0)),
        (child, rect(2.0, 6.0, 1.0, 1.0)),
    ] {
        assert_eq!(
            baseline
                .hit_regions
                .iter()
                .find(|h| h.node == node)
                .unwrap()
                .rect,
            expected
        );
    }
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    styles[row].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(0, 255, 0),
        Color::rgb(0, 255, 0),
    )));
    let result = layout(&doc, &styles, 14.0, 10.0, &Fonts::new());
    let g = gradients(&result);
    assert!(
        g.iter()
            .any(|v| v.0 == rect(2.0, 2.0, 10.0, 6.0) && v.1 == v.0)
    );
    assert!(
        g.iter()
            .any(|v| v.0 == rect(2.0, 2.0, 4.0, 6.0) && v.1 == v.0)
    );
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

#[test]
fn gradient_fragment_translation_stretch_and_overflow_slots_are_explicit() {
    let (doc, mut styles) = prepare(
        "<style>#a{height:2px;overflow:hidden}</style><div id=a></div>",
        10.0,
        10.0,
    );
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    let fonts = Fonts::new();
    let mut e = engine(&doc, &styles, &fonts);
    let mut f = e.fragment(a, 4.0, 4.0, 0);
    assert!(f.gradient);
    assert!(matches!(f.commands[6], DrawCommand::PushClip { .. }));
    stretch_fragment(&mut f, 6.0);
    translate_fragment(&mut f, 3.0, 5.0);
    assert!(
        matches!(f.commands[1],DrawCommand::LinearGradient{rect:r,tile:t,..} if r==rect(3.0,5.0,4.0,6.0) && t==r)
    );
    assert!(matches!(f.commands[6],DrawCommand::PushClip{rect:r} if r==rect(3.0,5.0,4.0,6.0)));
}

#[test]
fn fixed_gradient_keeps_tile_origin_while_escaping_clip_inside_opacity() {
    let (doc, mut styles) = prepare(
        "<style>body{margin:0}main{opacity:.5;overflow:hidden;width:1px;height:1px;margin-top:100px}i{position:fixed;left:2px;top:2px;width:4px;height:4px}</style><main><i id=a></i></main>",
        10.0,
        10.0,
    );
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    let result = layout(&doc, &styles, 10.0, 10.0, &Fonts::new());
    let mut canvas = Canvas::new(10, 10).unwrap();
    canvas.paint_with_viewport(
        &result.commands,
        &Fonts::new(),
        &ImageStore::new(),
        (0.0, -100.0),
        (0.0, 0.0),
    );
    assert_eq!((canvas.pixels[22], canvas.pixels[24]), (0xff8080, 0x8080ff));
    assert!(result.hit_regions.iter().any(|h| h.node == a && h.fixed));
    let g = gradients(&result);
    assert_eq!(
        (g[0].0, g[0].1),
        (rect(2.0, 2.0, 4.0, 4.0), rect(2.0, 2.0, 4.0, 4.0))
    );
}

#[test]
fn pending_inline_admission_is_atomic_and_zero_positioning_boxes_are_not_pending() {
    let (doc, mut styles) = prepare("<span id=a></span><div id=z></div>", 10.0, 10.0);
    let a = doc.query_selector("#a").unwrap();
    let z = doc.query_selector("#z").unwrap();
    let image = split(Color::rgb(255, 0, 0), Color::rgb(0, 0, 255));
    styles[a].background_image = Some(BackgroundImage::Gradient(Arc::clone(&image)));
    styles[z].background_image = Some(BackgroundImage::Gradient(image));
    let fonts = Fonts::new();
    let area = rect(0.0, 0.0, 2.0, 2.0);
    let geometry = BoxGeometry {
        node: a,
        border: area,
        padding: area,
        content: area,
        fixed: false,
        inline: true,
        inline_prefix: 0.0,
        inline_sides: 3,
        inline_basis: 10.0,
    };
    let mut e = engine(&doc, &styles, &fonts);
    e.boxes.push(geometry);
    e.commands_created = MAX_COMMANDS - 4;
    e.paint_inline_background(geometry, 0.0, 3);
    assert!(e.commands.is_empty());
    assert_eq!(e.open_clips, 0);
    e.commands_created = MAX_COMMANDS - 5;
    e.paint_inline_background(geometry, 0.0, 3);
    assert_eq!(
        (e.commands.len(), e.commands_created, e.open_clips),
        (5, MAX_COMMANDS, 0)
    );
    // 5 classification + node table + 8 geometry + 5 slots + 4*16 shapes.
    let cost = doc.nodes.len() + 82;
    e.position_work_left = cost - 1;
    e.finish_inline_backgrounds();
    assert!(
        matches!(e.commands[2], DrawCommand::Rect { color, .. } if color == Color::TRANSPARENT)
    );
    assert!(e.owners[2].inline_layer == InlineLayer::Image);
    assert_eq!(e.position_work_left, cost - 6); // Reached first scan stays charged.
    assert!(!e.inline_gradients); // Finalization is deliberately one-shot.

    let mut e = engine(&doc, &styles, &fonts);
    e.boxes.push(geometry);
    e.paint_inline_background(geometry, 0.0, 3);
    e.position_work_left = cost;
    e.finish_inline_backgrounds();
    assert_eq!(e.position_work_left, 0);
    assert!(matches!(&e.commands[2], DrawCommand::BoxDecoration(value)
        if matches!(&value.paint, crate::graphics::box_decoration::BoxPaint::LinearGradient {tile,..}
            if *tile == rect(0.0,0.0,2.0,2.0))));

    // A real zero-width tile stays zero while a separate inline slot is pending.
    let mut e = engine(&doc, &styles, &fonts);
    e.paint_owner = PaintOwner {
        node: z,
        background: true,
        inline_layer: InlineLayer::None,
        inline_prefix: 0.0,
        inline_sides: 0,
    };
    e.push(DrawCommand::LinearGradient {
        rect: rect(0.0, 0.0, 4.0, 4.0),
        tile: rect(2.0, 2.0, 0.0, 0.0),
        radius: 0.0,
        gradient: match styles[z].background_image.clone().unwrap() {
            BackgroundImage::Gradient(value) => value,
            _ => panic!("expected gradient"),
        },
        current_color: Color::BLACK,
        stop_scale: 1.0,
        repeat: [true; 2],
    });
    e.boxes.push(geometry);
    e.paint_inline_background(geometry, 0.0, 3);
    e.position_work_left = MAX_POSITION_WORK;
    e.finish_inline_backgrounds();
    assert!(matches!(e.commands[0], DrawCommand::LinearGradient {tile,..} if tile.width == 0.0));
}

#[test]
fn principal_gradient_slots_fit_the_existing_entry_reservation() {
    let (doc, mut styles) = prepare("<div id=a><div id=child></div></div>", 10.0, 10.0);
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_image = Some(BackgroundImage::Gradient(split(
        Color::rgb(255, 0, 0),
        Color::rgb(0, 0, 255),
    )));
    styles[a].height = Length::Px(2.0);
    let fonts = Fonts::new();
    for (remaining, expected) in [(8, 0), (9, 6)] {
        let mut e = engine(&doc, &styles, &fonts);
        e.commands_created = MAX_COMMANDS - remaining;
        e.layout_box(a, 0.0, 0.0, 4.0, Some(4.0), 0);
        assert_eq!(e.commands.len(), expected);
        assert!(e.commands_created <= MAX_COMMANDS);
        assert_eq!(e.open_clips, 0);
        if expected != 0 {
            assert!(matches!(e.commands[1], DrawCommand::LinearGradient { .. }));
            assert_eq!(e.hits.len(), 1); // Child entry refuses before reserving anything.
            assert!(
                e.owners
                    .iter()
                    .all(|owner| owner.node == a && owner.background)
            );
        }
    }
}

#[test]
fn canvas_geometry_scan_exact_and_one_short_preserve_remaining_paint_work() {
    let (doc, styles) = prepare("<div id=a></div>", 10.0, 10.0);
    let a = doc.query_selector("#a").unwrap();
    let fonts = Fonts::new();
    let mut e = engine(&doc, &styles, &fonts);
    for node in [doc.root, doc.root, a] {
        e.boxes.push(BoxGeometry {
            node,
            border: Rect::default(),
            padding: rect(2.0, 3.0, 4.0, 5.0),
            content: Rect::default(),
            fixed: false,
            inline: false,
            inline_prefix: 0.0,
            inline_basis: 0.0,
            inline_sides: 0,
        });
    }
    e.position_work_left = 2;
    assert_eq!(e.canvas_gradient_tile(a), None);
    assert_eq!(e.position_work_left, 2);
    e.position_work_left = 3;
    assert_eq!(e.canvas_gradient_tile(a), Some(rect(2.0, 3.0, 4.0, 5.0)));
    assert_eq!(e.position_work_left, 0);
    e.position_work_left = 3;
    assert_eq!(e.canvas_gradient_tile(usize::MAX), None);
    assert_eq!(e.position_work_left, 0);
}
