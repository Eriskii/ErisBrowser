//! Independent literals frozen before inline background integration.
//! Intended as the child `tests` module of layout::inline_backgrounds.
use super::*;
use crate::graphics::box_decoration::{BoxDecoration, BoxPaint};
use crate::graphics::{Canvas, ImageStore, RasterImage};
use std::sync::Arc;

const WHITE: u32 = 0xffffff;
const RED: u32 = 0xff0000;
const BLUE: u32 = 0x0000ff;
const GREEN: u32 = 0x00ff00;
const BASE: &str = "<style>html,body{margin:0}main{font-size:20px;line-height:26px}i{display:inline-block;width:4px;height:26px;vertical-align:top}span{border-left:1px solid transparent;border-right:2px solid transparent;padding-left:2px;padding-right:1px}</style>";

fn prepare(source: &str, width: f32, height: f32) -> (Document, Vec<ComputedStyle>) {
    let doc = Document::parse(source);
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), width, height);
    (doc, styles)
}
fn paint(commands: &[DrawCommand], width: u32, height: u32, images: &ImageStore) -> Vec<u32> {
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.paint(commands, &Fonts::new(), images, 0.0, 0.0);
    assert!(
        !canvas.exhausted(),
        "literal scene stays within the unchanged paint budget"
    );
    canvas.pixels
}
fn layout_source(source: &str, width: u32, height: u32) -> LayoutResult {
    let (doc, styles) = prepare(source, width as f32, height as f32);
    layout(&doc, &styles, width as f32, height as f32, &Fonts::new())
}
fn decorations(commands: &[DrawCommand]) -> Vec<&BoxDecoration> {
    commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::BoxDecoration(value) => Some(value.as_ref()),
            _ => None,
        })
        .collect()
}
fn gradient_geometry(commands: &[DrawCommand]) -> Vec<(Rect, Rect, [bool; 2])> {
    decorations(commands)
        .into_iter()
        .filter_map(|value| match &value.paint {
            BoxPaint::LinearGradient { tile, repeat, .. } => {
                assert!(value.cutout.is_none());
                Some((value.shape.rect, *tile, *repeat))
            }
            _ => None,
        })
        .collect()
}
fn image_geometry(commands: &[DrawCommand]) -> Vec<(Rect, Rect, [bool; 2], &str)> {
    decorations(commands)
        .into_iter()
        .filter_map(|value| match &value.paint {
            BoxPaint::Image { tile, repeat, key } => {
                assert!(value.cutout.is_none());
                Some((value.shape.rect, *tile, *repeat, key.as_str()))
            }
            _ => None,
        })
        .collect()
}
fn color_shapes(commands: &[DrawCommand], packed: u32) -> Vec<Rect> {
    decorations(commands)
        .into_iter()
        .filter_map(|value| match value.paint {
            BoxPaint::Color(color)
                if color.packed() == packed && color.a == 255 && value.cutout.is_none() =>
            {
                Some(value.shape.rect)
            }
            _ => None,
        })
        .collect()
}
fn clips(commands: &[DrawCommand]) -> Vec<Rect> {
    commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::PushClip { rect } => Some(*rect),
            _ => None,
        })
        .collect()
}
fn palette(value: u8) -> u32 {
    match value {
        b'W' => WHITE,
        b'R' => RED,
        b'B' => BLUE,
        b'G' => GREEN,
        b'q' => 0xc0c0ff,
        b'd' => 0xf0f0ff,
        _ => panic!("unknown literal palette entry"),
    }
}
// These expectation writers place literal rectangles/rows. They do not resolve
// style, tile coordinates, sampling, shape coverage, or production geometry.
fn literal_rect(frame: &mut [u32], stride: usize, area: [usize; 4], color: u32) {
    let [x, y, width, height] = area;
    for row in y..y + height {
        frame[row * stride + x..row * stride + x + width].fill(color);
    }
}
fn literal_band(frame: &mut [u32], stride: usize, start_y: usize, height: usize, row: &str) {
    for y in start_y..start_y + height {
        for (x, c) in row.bytes().enumerate() {
            frame[y * stride + x] = palette(c);
        }
    }
}
fn store() -> ImageStore {
    let mut result = ImageStore::new();
    result.insert(
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
    result
}
fn loaded() -> LoadedBackground<'static> {
    LoadedBackground {
        key: "tile",
        natural: RasterNaturalSize {
            width: 4,
            height: 2,
        },
    }
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
    crate::css::BackgroundData::test_replace_size(
        &mut style.backgrounds,
        BackgroundSize::Explicit {
            width: Length::Percent(100.0),
            height: Length::Auto,
        },
    );
    crate::css::BackgroundData::test_replace_repeat(&mut style.backgrounds, [false; 2]);
}

#[test]
fn forced_two_fragments_use_separate_border_padding_and_content_strips() {
    for (origin, width, x, rows) in [
        ("border-box", 14.0, [0.0, -7.0], ["RRRRRRR", "BBBBBBB"]),
        ("padding-box", 11.0, [1.0, -6.0], ["BRRRRRB", "BBBBBRR"]),
        ("content-box", 8.0, [3.0, -4.0], ["BBBRRRR", "BBBBRRR"]),
    ] {
        let source = format!(
            "{BASE}<style>main{{width:20px}}span{{background-image:linear-gradient(to right,red 50%,blue 50%);background-origin:{origin};background-clip:border-box;background-size:100% 100%}}</style><main><span><i></i><br><i></i></span></main>"
        );
        let result = layout_source(&source, 12, 60);
        assert_eq!(
            gradient_geometry(&result.commands),
            [
                (
                    rect(0.0, 7.0, 14.0, 26.0),
                    rect(x[0], 7.0, width, 26.0),
                    [true; 2]
                ),
                (
                    rect(-7.0, 33.0, 14.0, 26.0),
                    rect(x[1], 33.0, width, 26.0),
                    [true; 2]
                ),
            ],
            "{origin}"
        );
        let windows = clips(&result.commands);
        assert!(windows.contains(&rect(0.0, 7.0, 7.0, 26.0)));
        assert!(windows.contains(&rect(0.0, 33.0, 7.0, 26.0)));
        let mut expected = vec![WHITE; 12 * 60];
        literal_band(&mut expected, 12, 7, 26, rows[0]);
        literal_band(&mut expected, 12, 33, 26, rows[1]);
        assert_eq!(
            paint(&result.commands, 12, 60, &ImageStore::new()),
            expected,
            "{origin}"
        );
    }
}

fn soft_source(extra: &str) -> String {
    format!(
        "{BASE}<style>main{{width:10px}}span{{{extra}}}</style><main><span id=a><i></i><i></i><i></i><i></i></span></main>"
    )
}

#[test]
fn soft_three_fragments_keep_origin_clip_and_ratio_sizing_independent() {
    let source = soft_source(
        "background-image:linear-gradient(to right,red 50%,blue 50%);background-origin:content-box;background-clip:content-box;background-size:100% 100%",
    );
    let result = layout_source(&source, 10, 90);
    assert_eq!(
        gradient_geometry(&result.commands),
        [
            (
                rect(3.0, 7.0, 16.0, 26.0),
                rect(3.0, 7.0, 16.0, 26.0),
                [true; 2]
            ),
            (
                rect(-4.0, 33.0, 16.0, 26.0),
                rect(-4.0, 33.0, 16.0, 26.0),
                [true; 2]
            ),
            (
                rect(-12.0, 59.0, 16.0, 26.0),
                rect(-12.0, 59.0, 16.0, 26.0),
                [true; 2]
            ),
        ]
    );
    let mut expected = vec![WHITE; 10 * 90];
    for (y, row) in [(7, "WWWRRRRWWW"), (33, "RRRRBBBBWW"), (59, "BBBBWWWWWW")] {
        literal_band(&mut expected, 10, y, 26, row);
    }
    assert_eq!(
        paint(&result.commands, 10, 90, &ImageStore::new()),
        expected
    );

    for (clip, width, x, rows) in [
        (
            "border-box",
            22.0,
            [0.0, -7.0, -15.0],
            ["BBBBBBBWWW", "BBBBBBBBWW", "BBBBBBBWWW"],
        ),
        (
            "padding-box",
            19.0,
            [1.0, -6.0, -14.0],
            ["WBBBBBBWWW", "BBBBBBBBWW", "BBBBBWWWWW"],
        ),
        (
            "content-box",
            16.0,
            [3.0, -4.0, -12.0],
            ["WWWBBBBWWW", "BBBBBBBBWW", "BBBBWWWWWW"],
        ),
    ] {
        let result = layout_source(
            &soft_source(&format!("background-color:blue;background-clip:{clip}")),
            10,
            90,
        );
        assert_eq!(
            color_shapes(&result.commands, BLUE),
            [
                rect(x[0], 7.0, width, 26.0),
                rect(x[1], 33.0, width, 26.0),
                rect(x[2], 59.0, width, 26.0),
            ],
            "{clip}"
        );
        let mut expected = vec![WHITE; 10 * 90];
        for (y, row) in [7, 33, 59].into_iter().zip(rows) {
            literal_band(&mut expected, 10, y, 26, row);
        }
        assert_eq!(
            paint(&result.commands, 10, 90, &ImageStore::new()),
            expected,
            "{clip}"
        );
    }

    let (doc, mut styles) = prepare(&source, 10.0, 90.0);
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded());
    let result = layout_with_backgrounds(&doc, &styles, 10.0, 90.0, &Fonts::new(), &bindings);
    assert_eq!(
        image_geometry(&result.commands),
        [
            (
                rect(3.0, 7.0, 16.0, 26.0),
                rect(3.0, 7.0, 16.0, 8.0),
                [false; 2],
                "tile"
            ),
            (
                rect(-4.0, 33.0, 16.0, 26.0),
                rect(-4.0, 33.0, 16.0, 8.0),
                [false; 2],
                "tile"
            ),
            (
                rect(-12.0, 59.0, 16.0, 26.0),
                rect(-12.0, 59.0, 16.0, 8.0),
                [false; 2],
                "tile"
            ),
        ]
    );
    let mut expected = vec![WHITE; 10 * 90];
    for (y, row) in [(7, "WWWRRRRWWW"), (33, "RRRRBBBBWW"), (59, "BBBBWWWWWW")] {
        literal_band(&mut expected, 10, y, 8, row);
    }
    assert_eq!(paint(&result.commands, 10, 90, &store()), expected);
}

#[test]
fn nested_font_boxes_and_relative_child_do_not_widen_the_parent() {
    let source = "<style>html,body{margin:0}main{position:absolute;top:20px;width:30px;font-size:20px;line-height:26px}i{display:inline-block;width:4px;height:26px;vertical-align:top}span{padding:0 2px;background:red;background-clip:content-box}b{font-size:40px;padding:0 1px;position:relative;left:5px;top:3px;background:blue;background-clip:content-box}</style><main><span><i id=first></i><b><i id=c0></i><i id=c1></i></b><i id=last></i></span></main>";
    let (doc, styles) = prepare(source, 30.0, 70.0);
    let result = layout(&doc, &styles, 30.0, 70.0, &Fonts::new());
    assert_eq!(
        color_shapes(&result.commands, RED),
        [rect(2.0, 27.0, 18.0, 26.0)]
    );
    assert_eq!(
        color_shapes(&result.commands, BLUE),
        [rect(12.0, 11.0, 8.0, 52.0)]
    );
    for (id, area) in [
        ("#first", rect(2.0, 20.0, 4.0, 26.0)),
        ("#c0", rect(12.0, 23.0, 4.0, 26.0)),
        ("#c1", rect(16.0, 23.0, 4.0, 26.0)),
        ("#last", rect(16.0, 20.0, 4.0, 26.0)),
    ] {
        let node = doc.query_selector(id).unwrap();
        assert!(
            result
                .hit_regions
                .iter()
                .any(|h| h.node == node && h.rect == area),
            "{id}"
        );
    }
    let mut expected = vec![WHITE; 30 * 70];
    literal_rect(&mut expected, 30, [2, 27, 18, 26], RED);
    literal_rect(&mut expected, 30, [12, 11, 8, 52], BLUE);
    assert_eq!(
        paint(&result.commands, 30, 70, &ImageStore::new()),
        expected
    );
}

#[test]
fn empty_decorated_owner_paints_its_padding_and_border_but_not_empty_content() {
    for (clip, area) in [
        ("border-box", Some([20, 18, 8, 31])),
        ("padding-box", Some([21, 19, 5, 29])),
        ("content-box", None),
    ] {
        let source = format!(
            "<style>html,body{{margin:0}}main{{position:absolute;left:20px;top:20px;font-size:20px;line-height:26px}}span{{border-style:solid;border-color:transparent;border-width:1px 2px 1px 1px;padding:1px 3px 2px 2px;background:blue;background-clip:{clip}}}</style><main><span></span></main>"
        );
        let result = layout_source(&source, 36, 54);
        let mut expected = vec![WHITE; 36 * 54];
        match area {
            Some([x, y, width, height]) => {
                assert_eq!(
                    color_shapes(&result.commands, BLUE),
                    [rect(x as f32, y as f32, width as f32, height as f32)]
                );
                literal_rect(&mut expected, 36, [x, y, width, height], BLUE);
            }
            None => assert!(color_shapes(&result.commands, BLUE).is_empty()),
        }
        assert_eq!(
            paint(&result.commands, 36, 54, &ImageStore::new()),
            expected,
            "{clip}"
        );
    }
}

#[test]
fn rounded_virtual_shape_has_corners_only_at_physical_ends() {
    let result = layout_source(
        &soft_source("background:red;background-clip:border-box;border-radius:4px"),
        10,
        90,
    );
    assert_eq!(
        color_shapes(&result.commands, RED),
        [
            rect(0.0, 7.0, 22.0, 26.0),
            rect(-7.0, 33.0, 22.0, 26.0),
            rect(-15.0, 59.0, 22.0, 26.0),
        ]
    );
    for value in decorations(&result.commands) {
        if matches!(value.paint, BoxPaint::Color(color) if color.packed() == RED && color.a == 255)
        {
            assert_eq!(value.shape.radii, [[4.0; 2]; 4]);
        }
    }
    let pixels = paint(&result.commands, 10, 90, &ImageStore::new());
    // Independent extreme grid witnesses: at the outer4px corner every sample
    // is outside; at virtualx7/14/15 no corner constraint can exclude a sample.
    for (x, y, expected) in [
        (0, 7, WHITE),
        (0, 33, RED),
        (7, 33, RED),
        (6, 59, WHITE),
        (0, 60, RED),
    ] {
        assert_eq!(pixels[y * 10 + x], expected, "({x},{y})");
    }
}

// This private constructor copies the established layout-test seam. Only the
// agreed compact inline owner fields differ; no fetch/resource selector is added.
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
        background_key_bytes_left: MAX_BACKGROUND_KEY_BYTES,
        viewport: Size {
            width: 40.0,
            height: 90.0,
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
            background_image_index: 0,
        },
        flow_height_reference: Some(90.0),
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
fn three_boxes(node: NodeId) -> [BoxGeometry; 3] {
    [
        BoxGeometry {
            node,
            border: rect(0.0, 7.0, 7.0, 26.0),
            padding: rect(1.0, 7.0, 6.0, 26.0),
            content: rect(3.0, 7.0, 4.0, 26.0),
            fixed: false,
            inline: true,
            inline_prefix: 0.0,
            inline_sides: 1,
            inline_basis: 10.0,
        },
        BoxGeometry {
            node,
            border: rect(0.0, 33.0, 8.0, 26.0),
            padding: rect(0.0, 33.0, 8.0, 26.0),
            content: rect(0.0, 33.0, 8.0, 26.0),
            fixed: false,
            inline: true,
            inline_prefix: 7.0,
            inline_sides: 0,
            inline_basis: 10.0,
        },
        BoxGeometry {
            node,
            border: rect(0.0, 59.0, 7.0, 26.0),
            padding: rect(0.0, 59.0, 5.0, 26.0),
            content: rect(0.0, 59.0, 4.0, 26.0),
            fixed: false,
            inline: true,
            inline_prefix: 15.0,
            inline_sides: 2,
            inline_basis: 10.0,
        },
    ]
}
fn assert_recorded_geometry(e: &Engine<'_>, expected: &[BoxGeometry]) {
    assert_eq!(e.boxes.len(), expected.len());
    for (actual, expected) in e.boxes.iter().zip(expected) {
        assert_eq!(
            (
                actual.node,
                actual.border,
                actual.padding,
                actual.content,
                actual.fixed,
                actual.inline,
                actual.inline_prefix,
                actual.inline_sides
            ),
            (
                expected.node,
                expected.border,
                expected.padding,
                expected.content,
                expected.fixed,
                expected.inline,
                expected.inline_prefix,
                expected.inline_sides
            )
        );
    }
}
fn publish(e: &mut Engine<'_>, g: BoxGeometry) {
    e.paint_inline_background(g, g.inline_prefix, g.inline_sides);
}

#[test]
fn command_and_key_refusal_do_not_shorten_strips_or_advance_from_paint_slots() {
    let source = soft_source(
        "background-color:lime;background-origin:content-box;background-clip:content-box",
    );
    let (doc, mut styles) = prepare(&source, 10.0, 90.0);
    let a = doc.query_selector("#a").unwrap();
    url(&mut styles[a]);
    let mut bindings = vec![None; doc.nodes.len()];
    bindings[a] = Some(loaded());
    let fonts = Fonts::new();
    let geometry = three_boxes(a);

    let background_ranges = singleton_ranges(bindings.len());

    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.boxes.extend(geometry);
    e.commands_created = MAX_COMMANDS - 4;
    publish(&mut e, geometry[0]);
    assert!(e.commands.is_empty());
    assert!(e.owners.is_empty());
    assert_eq!((e.commands_created, e.open_clips), (MAX_COMMANDS - 4, 0));
    assert_recorded_geometry(&e, &geometry);

    // Fresh exact boundary: the first group fits; both later groups refuse.
    // Their already completed geometry still determines the full16px origin.
    let background_ranges = singleton_ranges(bindings.len());
    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.boxes.extend(geometry);
    e.commands_created = MAX_COMMANDS - 5;
    for g in geometry {
        publish(&mut e, g);
    }
    assert_eq!(
        (
            e.commands.len(),
            e.owners.len(),
            e.commands_created,
            e.open_clips
        ),
        (5, 5, MAX_COMMANDS, 0)
    );
    e.finish_inline_backgrounds();
    assert_recorded_geometry(&e, &geometry);
    assert_eq!(
        image_geometry(&e.commands),
        [(
            rect(3.0, 7.0, 16.0, 26.0),
            rect(3.0, 7.0, 16.0, 8.0),
            [false; 2],
            "tile"
        )]
    );
    let mut expected = vec![WHITE; 10 * 90];
    literal_rect(&mut expected, 10, [3, 7, 4, 26], GREEN);
    literal_rect(&mut expected, 10, [3, 7, 4, 8], RED);
    assert_eq!(paint(&e.commands, 10, 90, &store()), expected);

    // Synthetic nonadjacent publication checks metadata, without replenishing
    // any budget: all geometry exists, but no call publishes the middle slot.
    let background_ranges = singleton_ranges(bindings.len());
    let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
    e.boxes.extend(geometry);
    publish(&mut e, geometry[0]);
    publish(&mut e, geometry[2]);
    e.finish_inline_backgrounds();
    assert_eq!(
        image_geometry(&e.commands),
        [
            (
                rect(3.0, 7.0, 16.0, 26.0),
                rect(3.0, 7.0, 16.0, 8.0),
                [false; 2],
                "tile"
            ),
            (
                rect(-12.0, 59.0, 16.0, 26.0),
                rect(-12.0, 59.0, 16.0, 8.0),
                [false; 2],
                "tile"
            ),
        ]
    );
    literal_rect(&mut expected, 10, [0, 59, 4, 26], GREEN);
    literal_rect(&mut expected, 10, [0, 59, 4, 8], BLUE);
    assert_eq!(paint(&e.commands, 10, 90, &store()), expected);
    assert_recorded_geometry(&e, &geometry);

    for missing in [false, true] {
        let unavailable = vec![None; doc.nodes.len()];
        let background_ranges = singleton_ranges(doc.nodes.len());
        let mut e = engine(
            &doc,
            &styles,
            &fonts,
            if missing { &unavailable } else { &bindings },
            &background_ranges,
        );
        e.boxes.extend(geometry);
        if !missing {
            e.background_key_bytes_left = 0;
        }
        for g in geometry {
            publish(&mut e, g);
        }
        e.finish_inline_backgrounds();
        assert!(image_geometry(&e.commands).is_empty());
        assert_eq!(e.open_clips, 0);
        assert_recorded_geometry(&e, &geometry);
        let mut expected = vec![WHITE; 10 * 90];
        literal_rect(&mut expected, 10, [3, 7, 4, 26], GREEN);
        literal_rect(&mut expected, 10, [0, 33, 8, 26], GREEN);
        literal_rect(&mut expected, 10, [0, 59, 4, 26], GREEN);
        assert_eq!(
            paint(&e.commands, 10, 90, &store()),
            expected,
            "missing={missing}"
        );
    }
}

#[test]
fn fractional_shape_y_and_physical_ends_keep_their_independent_grid_coverage() {
    let (doc, mut styles) = prepare("<span id=a></span>", 4.0, 4.0);
    let a = doc.query_selector("#a").unwrap();
    styles[a].background_color = Color::rgb(0, 0, 255);
    crate::css::BackgroundData::test_replace_clip(
        &mut styles[a].backgrounds,
        BackgroundBox::Border,
    );
    let fonts = Fonts::new();
    let bindings = vec![None; doc.nodes.len()];
    for (area, sides, expected_clip, rows) in [
        (
            rect(0.0, 1.05, 4.0, 1.3),
            3,
            rect(0.0, 1.0, 4.0, 2.0),
            ["WWWW", "BBBB", "qqqq", "WWWW"],
        ),
        (
            rect(0.05, 1.05, 3.3, 1.3),
            3,
            rect(0.0, 1.0, 4.0, 2.0),
            ["WWWW", "BBBq", "qqqd", "WWWW"],
        ),
        // The existing hard internal horizontal cut is deliberately preserved.
        (
            rect(0.05, 1.05, 3.3, 1.3),
            0,
            rect(0.05, 1.0, 3.3, 2.0),
            ["WWWW", "WBBq", "Wqqd", "WWWW"],
        ),
    ] {
        let g = BoxGeometry {
            node: a,
            border: area,
            padding: area,
            content: area,
            fixed: false,
            inline: true,
            inline_prefix: 0.0,
            inline_sides: sides,
            inline_basis: 4.0,
        };
        let background_ranges = singleton_ranges(bindings.len());
        let mut e = engine(&doc, &styles, &fonts, &bindings, &background_ranges);
        e.boxes.push(g);
        publish(&mut e, g);
        e.finish_inline_backgrounds();
        assert_eq!(clips(&e.commands), [expected_clip]);
        assert_eq!(color_shapes(&e.commands, BLUE), [area]);
        let mut expected = vec![WHITE; 16];
        for (y, row) in rows.into_iter().enumerate() {
            literal_band(&mut expected, 4, y, 1, row);
        }
        assert_eq!(
            paint(&e.commands, 4, 4, &ImageStore::new()),
            expected,
            "sides={sides}, area={area:?}"
        );
        assert_eq!(e.open_clips, 0);
    }
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
