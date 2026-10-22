//! Independent literal pixels and refusal seams, frozen before production edits.
use super::*;
use crate::graphics::{Canvas, ImageStore, RasterImage};
use std::sync::Arc;

const RED: &str = "linear-gradient(#ff0000,#ff0000)";
const BLUE: &str = "linear-gradient(#0000ff,#0000ff)";
const GREEN: &str = "linear-gradient(#00ff00,#00ff00)";
fn prepare(source: &str, width: u32, height: u32) -> (Document, Vec<ComputedStyle>) {
    let doc = Document::parse(source);
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), width as f32, height as f32);
    (doc, styles)
}
fn source(rule: &str) -> String {
    format!(
        "<style>html,body{{margin:0;padding:0}}html{{background:white}}#box{{{rule}}}</style><div id=box></div>"
    )
}
fn store() -> ImageStore {
    let mut images = ImageStore::new();
    images.insert(
        "tile".into(),
        Arc::new(RasterImage {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 255, 0, 255],
        }),
    );
    images.insert(
        "wide".into(),
        Arc::new(RasterImage {
            width: 4,
            height: 2,
            rgba: [
                255, 0, 0, 255, 255, 0, 0, 255, 0, 0, 255, 255, 0, 0, 255, 255,
            ]
            .repeat(2),
        }),
    );
    images
}
fn pixels(commands: &[DrawCommand], width: u32, height: u32) -> Vec<u32> {
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.paint(commands, &Fonts::new(), &store(), 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas.pixels
}
fn rows(values: &[&str]) -> Vec<u32> {
    values
        .iter()
        .flat_map(|row| {
            row.bytes().map(|c| match c {
                b'W' => 0xffffff,
                b'R' => 0xff0000,
                b'G' => 0x00ff00,
                b'B' => 0x0000ff,
                _ => panic!("literal palette"),
            })
        })
        .collect()
}
fn render(source: &str, width: u32, height: u32) -> Vec<u32> {
    let (doc, styles) = prepare(source, width, height);
    pixels(
        &layout(&doc, &styles, width as f32, height as f32, &Fonts::new()).commands,
        width,
        height,
    )
}
fn loaded(key: &'static str, width: u32, height: u32) -> Option<LoadedBackground<'static>> {
    Some(LoadedBackground {
        key,
        natural: RasterNaturalSize { width, height },
    })
}
fn bindings(nodes: usize, node: NodeId, len: u8) -> Vec<BackgroundBindingRange> {
    let mut ranges = vec![BackgroundBindingRange::default(); nodes];
    ranges[node] = BackgroundBindingRange { start: 0, len };
    ranges
}
fn engine<'a>(
    doc: &'a Document,
    styles: &'a [ComputedStyle],
    fonts: &'a Fonts,
    backgrounds: BackgroundBindings<'a>,
) -> Engine<'a> {
    Engine {
        doc,
        styles,
        fonts,
        backgrounds,
        background_key_bytes_left: MAX_BACKGROUND_KEY_BYTES,
        viewport: Size {
            width: 32.0,
            height: 32.0,
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
        flow_height_reference: Some(32.0),
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
fn authored_none_consumes_position_and_top_image_is_first() {
    let s = source(&format!(
        "width:6px;height:1px;background-color:#00ff00;background-image:{RED},none,{BLUE};background-size:2px 1px;background-position:0 0,2px 0,4px 0;background-repeat:no-repeat"
    ));
    assert_eq!(render(&s, 6, 1), rows(&["RRGGBB"]));
    for (images, expected) in [
        (format!("{RED},{BLUE}"), "RRBB"),
        (format!("{BLUE},{RED}"), "BBBB"),
    ] {
        let sizes = if expected == "RRBB" {
            "2px 1px,4px 1px"
        } else {
            "4px 1px,2px 1px"
        };
        assert_eq!(
            render(
                &source(&format!(
                    "width:4px;height:1px;background-image:{images};background-size:{sizes};background-repeat:no-repeat"
                )),
                4,
                1
            ),
            rows(&[expected])
        );
    }
}
#[test]
fn short_lists_cycle_and_inherited_excess_positions_remain_available() {
    let s = source(&format!(
        "width:6px;height:1px;background-color:#00ff00;background-image:none,{BLUE},{RED};background-size:2px 1px;background-position:0 0,4px 0;background-repeat:no-repeat"
    ));
    assert_eq!(render(&s, 6, 1), rows(&["RRGGBB"]));
    let s = format!(
        "<style>html,body{{margin:0}}html{{background:white}}main{{background-position:0 0,2px 0,4px 0}}div{{width:6px;height:1px;background-color:#00ff00;background-image:{RED},none,{BLUE};background-position:inherit;background-size:2px 1px;background-repeat:no-repeat}}</style><main><div></div></main>"
    );
    assert_eq!(render(&s, 6, 1), rows(&["RRGGBB"]));
}
#[test]
fn color_uses_bottom_none_layer_clip_once() {
    let s = source(
        "box-sizing:border-box;width:8px;height:6px;border:1px solid transparent;padding:1px;background-color:#00ff00;background-image:none,none;background-clip:border-box,content-box",
    );
    assert_eq!(
        render(&s, 8, 6),
        rows(&[
            "WWWWWWWW", "WWWWWWWW", "WWGGGGWW", "WWGGGGWW", "WWWWWWWW", "WWWWWWWW"
        ])
    );
}
#[test]
fn each_image_selects_its_own_origin_and_clip() {
    let s = source(&format!(
        "box-sizing:border-box;width:8px;height:6px;border:1px solid transparent;padding:1px;background-image:{RED},{BLUE};background-origin:content-box,border-box;background-clip:content-box,padding-box;background-size:2px 2px,8px 6px;background-repeat:no-repeat"
    ));
    assert_eq!(
        render(&s, 8, 6),
        rows(&[
            "WWWWWWWW", "WBBBBBBW", "WBRRBBBW", "WBRRBBBW", "WBBBBBBW", "WWWWWWWW"
        ])
    );
}
#[test]
fn borrowed_url_slot_and_unavailable_upper_image_do_not_shift_layers() {
    let s = source(&format!(
        "width:6px;height:1px;background-color:#00ff00;background-image:url(tile.png),none,{BLUE};background-size:2px 1px;background-position:0 0,2px 0,4px 0;background-repeat:no-repeat"
    ));
    let (doc, styles) = prepare(&s, 6, 1);
    let id = doc.query_selector("#box").unwrap();
    let ranges = bindings(doc.nodes.len(), id, 3);
    let slots = [loaded("tile", 2, 1), None, None];
    let result = layout_with_backgrounds(
        &doc,
        &styles,
        6.0,
        1.0,
        &Fonts::new(),
        BackgroundBindings {
            ranges: &ranges,
            slots: &slots,
        },
    );
    assert_eq!(pixels(&result.commands, 6, 1), rows(&["RGGGBB"]));
    let s = source(&format!(
        "width:6px;height:1px;background-image:url(missing.png),none,{BLUE};background-size:6px 1px;background-repeat:no-repeat"
    ));
    assert_eq!(render(&s, 6, 1), rows(&["BBBBBB"]));
}
#[test]
fn malformed_borrowed_ranges_fail_closed_without_changing_semantic_index() {
    let slots = [loaded("tile", 2, 1), None, loaded("wide", 4, 2)];
    let ranges = [
        BackgroundBindingRange { start: 0, len: 3 },
        BackgroundBindingRange {
            start: u32::MAX,
            len: 2,
        },
        BackgroundBindingRange { start: 1, len: 3 },
    ];
    let view = BackgroundBindings {
        ranges: &ranges,
        slots: &slots,
    };
    assert_eq!(view.get(0, 0).unwrap().key, "tile");
    assert!(view.get(0, 1).is_none());
    assert_eq!(view.get(0, 2).unwrap().key, "wide");
    assert!(view.get(0, 3).is_none());
    assert!(view.get(1, 0).is_none());
    assert!(view.get(2, 0).is_none());
    assert!(view.get(3, 0).is_none());
}
#[test]
fn canvas_owner_distinguishes_single_none_from_multiple_none() {
    for (image, expected) in [
        ("none", ["RRRRRR", "RRRRRR", "RRRRRR"]),
        ("none,none", ["RRWWWW", "WWWWWW", "WWWWWW"]),
    ] {
        let s = format!(
            "<style>html,body{{margin:0;padding:0}}html{{background-image:{image}}}body{{width:2px;height:1px;background:#ff0000}}</style><body></body>"
        );
        let (doc, styles) = prepare(&s, 6, 3);
        let html = doc.first_html_element("html");
        let body = doc.first_html_element("body");
        assert_eq!(
            canvas_background_owner(&styles, html, body),
            if image == "none" { body } else { html }
        );
        assert_eq!(
            pixels(
                &layout(&doc, &styles, 6.0, 3.0, &Fonts::new()).commands,
                6,
                3
            ),
            rows(&expected)
        );
    }
}
#[test]
fn stretched_table_retains_layer_index_and_independent_natural_size() {
    let s = format!(
        "<style>html,body{{margin:0}}html{{background:white}}table{{position:relative;left:-2px;top:-2px;width:8px}}td{{width:4px;padding:0}}#box{{height:0;background-image:{GREEN},url(wide.png);background-size:1px 1px,auto 100%;background-position:1px 0,center;background-repeat:no-repeat}}#tall{{height:6px}}</style><table><tr><td id=box></td><td id=tall></td></tr></table>"
    );
    let (doc, styles) = prepare(&s, 8, 6);
    let id = doc.query_selector("#box").unwrap();
    let ranges = bindings(doc.nodes.len(), id, 2);
    let slots = [None, loaded("wide", 4, 2)];
    let result = layout_with_backgrounds(
        &doc,
        &styles,
        8.0,
        6.0,
        &Fonts::new(),
        BackgroundBindings {
            ranges: &ranges,
            slots: &slots,
        },
    );
    assert_eq!(
        pixels(&result.commands, 8, 6),
        rows(&[
            "RGBBWWWW", "RRBBWWWW", "RRBBWWWW", "RRBBWWWW", "RRBBWWWW", "RRBBWWWW"
        ])
    );
}
fn inline_boxes(node: NodeId) -> [BoxGeometry; 2] {
    [
        (0.0, 0.0, inline_backgrounds::FIRST),
        (2.0, 4.0, inline_backgrounds::LAST),
    ]
    .map(|(y, prefix, sides)| {
        let area = rect(0.0, y, 4.0, 1.0);
        BoxGeometry {
            node,
            border: area,
            padding: area,
            content: area,
            fixed: false,
            inline: true,
            inline_prefix: prefix,
            inline_basis: 8.0,
            inline_sides: sides,
        }
    })
}
#[test]
fn inline_virtual_layers_keep_phase_when_an_earlier_group_is_refused() {
    let s = format!(
        "<style>#a{{background-image:{GREEN},linear-gradient(to right,#ff0000 4px,#0000ff 4px);background-size:1px 1px,8px 1px;background-position:5px 0,0 0;background-repeat:no-repeat}}</style><span id=a></span>"
    );
    let (doc, styles) = prepare(&s, 4, 3);
    let id = doc.query_selector("#a").unwrap();
    let fonts = Fonts::new();
    let geometry = inline_boxes(id);
    for refused in [false, true] {
        let mut e = engine(&doc, &styles, &fonts, BackgroundBindings::default());
        e.boxes.extend(geometry);
        if refused {
            e.commands_created = MAX_COMMANDS - 5;
        }
        e.paint_inline_background(geometry[0], 0.0, inline_backgrounds::FIRST);
        if refused {
            assert!(e.commands.is_empty());
            assert_eq!(e.open_clips, 0);
            e.commands_created = 0;
        }
        e.paint_inline_background(geometry[1], 4.0, inline_backgrounds::LAST);
        e.finish_inline_backgrounds();
        assert_eq!(e.open_clips, 0);
        assert_eq!(e.boxes[1].inline_prefix, 4.0);
        assert_eq!(
            pixels(&e.commands, 4, 3),
            rows(&[if refused { "WWWW" } else { "RRRR" }, "WWWW", "BGBB"])
        );
    }
}
#[test]
fn principal_reserves_all_thirteen_slots_or_keeps_geometry_without_paint() {
    let images = [RED; 8].join(",");
    let s = source(&format!("width:8px;height:1px;background-image:{images}"));
    let (doc, styles) = prepare(&s, 8, 1);
    let id = doc.query_selector("#box").unwrap();
    let fonts = Fonts::new();
    for remaining in [12, 13] {
        let mut e = engine(&doc, &styles, &fonts, BackgroundBindings::default());
        e.commands_created = MAX_COMMANDS - remaining;
        let result = e.layout_box_sized(id, 0.0, 0.0, 8.0, (Some(8.0), Some(1.0)), 0);
        assert_eq!((result.size.width, result.size.height), (8.0, 1.0));
        assert_eq!(e.boxes[0].border, rect(0.0, 0.0, 8.0, 1.0));
        assert_eq!(
            result.paint_slots.len(),
            if remaining == 13 { 13 } else { 0 }
        );
        assert_eq!(e.commands.len(), if remaining == 13 { 13 } else { 0 });
        assert_eq!(e.open_clips, 0);
    }
}
#[test]
fn cumulative_key_refusal_keeps_lower_slot_and_does_not_refund() {
    let s = source(
        "width:6px;height:1px;background-color:#00ff00;background-image:url(tile.png),url(tile.png);background-size:2px 1px;background-position:0 0,4px 0;background-repeat:no-repeat",
    );
    let (doc, styles) = prepare(&s, 6, 1);
    let id = doc.query_selector("#box").unwrap();
    let ranges = bindings(doc.nodes.len(), id, 2);
    let slots = [loaded("tile", 2, 1), loaded("tile", 2, 1)];
    let fonts = Fonts::new();
    for (allowance, expected, left) in [(8, "RGGGRG", 0), (7, "GGGGRG", 3)] {
        let mut e = engine(
            &doc,
            &styles,
            &fonts,
            BackgroundBindings {
                ranges: &ranges,
                slots: &slots,
            },
        );
        e.background_key_bytes_left = allowance;
        e.layout_box_sized(id, 0.0, 0.0, 6.0, (Some(6.0), Some(1.0)), 0);
        assert_eq!(e.background_key_bytes_left, left);
        assert_eq!(pixels(&e.commands, 6, 1), rows(&[expected]));
    }
}
#[test]
fn translucent_top_layer_blends_over_the_opaque_lower_layer() {
    let s = source(&format!(
        "width:2px;height:1px;background-image:linear-gradient(rgba(255,0,0,0.5),rgba(255,0,0,0.5)),{BLUE}"
    ));
    assert_eq!(render(&s, 2, 1), vec![0x80007f, 0x80007f]);
}
