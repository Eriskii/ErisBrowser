//! Literal table-row coverage and independent admission boundaries.
use super::*;
use crate::graphics::{Canvas, ImageStore, RasterImage};
use std::sync::Arc;

const TWO_ROWS: &str = "<table id=t><tr id=r><td id=a></td><td id=b></td></tr><tr id=s><td id=c></td><td id=d></td></tr></table>";
const ONE_ROW: &str = "<table id=t><tr id=r><td id=a></td><td id=b></td></tr></table>";
fn prepare(rules: &str, body: &str, width: u32, height: u32) -> (Document, Vec<ComputedStyle>) {
    let doc = Document::parse(&format!(
        "<style>html,body{{margin:0;padding:0;background:white;font-size:0;line-height:0}}table{{width:14px;border-spacing:2px 1px;background:yellow;box-sizing:content-box}}table,tr,td{{margin:0;padding:0;border:0}}tr{{height:2px}}td{{width:4px;height:2px;vertical-align:top;background:transparent}}{rules}</style>{body}"
    ));
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), width as f32, height as f32);
    (doc, styles)
}
fn store() -> ImageStore {
    let mut store = ImageStore::new();
    store.insert(
        "rowk".into(),
        Arc::new(RasterImage {
            width: 2,
            height: 1,
            rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
        }),
    );
    store
}
fn pixels(output: &LayoutResult, width: u32, height: u32) -> Vec<u32> {
    let mut canvas = Canvas::new(width, height).unwrap();
    canvas.paint(&output.commands, &Fonts::new(), &store(), 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas.pixels
}
fn literal(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| {
            row.bytes().map(|c| match c {
                b'Y' => 0xffff00,
                b'R' => 0xff0000,
                b'B' => 0x0000ff,
                b'W' => 0xffffff,
                b'L' => 0x8080ff,
                b'P' => 0xff8080,
                _ => panic!("literal palette"),
            })
        })
        .collect()
}
fn render(rules: &str, body: &str, width: u32, height: u32) -> Vec<u32> {
    let (doc, styles) = prepare(rules, body, width, height);
    pixels(
        &layout(&doc, &styles, width as f32, height as f32, &Fonts::new()),
        width,
        height,
    )
}
fn row_bindings(doc: &Document, len: u8) -> Vec<BackgroundBindingRange> {
    let mut ranges = vec![BackgroundBindingRange::default(); doc.nodes.len()];
    ranges[doc.query_selector("#r").unwrap()] = BackgroundBindingRange { start: 0, len };
    ranges
}
fn loaded(key: &str) -> Option<LoadedBackground<'_>> {
    Some(LoadedBackground {
        key,
        natural: RasterNaturalSize {
            width: 2,
            height: 1,
        },
    })
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
fn fragment(node: NodeId, border: Rect) -> Fragment {
    Fragment {
        size: Size {
            width: border.width,
            height: border.height,
        },
        commands: Vec::new(),
        owners: Vec::new(),
        hits: Vec::new(),
        positioned: Vec::new(),
        boxes: vec![BoxGeometry {
            node,
            border,
            padding: border,
            content: border,
            fixed: false,
            inline: false,
            inline_prefix: 0.0,
            inline_basis: 0.0,
            inline_sides: 0,
        }],
        paint_slots: PaintSlots::default(),
        principal_box: 0,
    }
}
fn gradients(output: &LayoutResult) -> Vec<(Rect, Rect)> {
    output
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::LinearGradient { rect, tile, .. } => Some((*rect, *tile)),
            _ => None,
        })
        .collect()
}

#[test]
fn explicit_empty_cells_show_the_row_but_spacing_shows_the_table() {
    assert_eq!(
        render("#r{background:red}#s{background:blue}", TWO_ROWS, 14, 7),
        literal(&[
            "YYYYYYYYYYYYYY",
            "YYRRRRYYRRRRYY",
            "YYRRRRYYRRRRYY",
            "YYYYYYYYYYYYYY",
            "YYBBBBYYBBBBYY",
            "YYBBBBYYBBBBYY",
            "YYYYYYYYYYYYYY",
        ])
    );
}

#[test]
fn rowspans_and_colspans_cover_their_full_originating_cells() {
    let body = "<table id=t><tr id=r><td id=a rowspan=2></td><td></td></tr><tr id=s><td></td></tr></table>";
    assert_eq!(
        render(
            "#r{background:red}#s{background:blue}#a{height:5px}",
            body,
            14,
            7
        ),
        literal(&[
            "YYYYYYYYYYYYYY",
            "YYRRRRYYRRRRYY",
            "YYRRRRYYRRRRYY",
            "YYRRRRYYYYYYYY",
            "YYRRRRYYBBBBYY",
            "YYRRRRYYBBBBYY",
            "YYYYYYYYYYYYYY",
        ])
    );
    let body = "<table id=t><tr id=r><td id=a colspan=2></td><td></td></tr><tr id=s><td></td><td></td><td></td></tr></table>";
    assert_eq!(
        render(
            "table{width:20px}#a{width:10px}#r{background:red}#s{background:blue}",
            body,
            20,
            7
        ),
        literal(&[
            "YYYYYYYYYYYYYYYYYYYY",
            "YYRRRRRRRRRRYYRRRRYY",
            "YYRRRRRRRRRRYYRRRRYY",
            "YYYYYYYYYYYYYYYYYYYY",
            "YYBBBBYYBBBBYYBBBBYY",
            "YYBBBBYYBBBBYYBBBBYY",
            "YYYYYYYYYYYYYYYYYYYY",
        ])
    );
}

#[test]
fn missing_cells_and_empty_rows_do_not_invent_row_coverage() {
    for (body, middle) in [
        (
            "<table id=t><tr id=r><td></td><td></td></tr><tr id=s><td></td></tr></table>",
            "YYBBBBYYYYYYYY",
        ),
        (
            "<table id=t><tr id=r><td></td><td></td></tr><tr id=s></tr></table>",
            "YYYYYYYYYYYYYY",
        ),
    ] {
        assert_eq!(
            render("#r{background:red}#s{background:blue}", body, 14, 7),
            literal(&[
                "YYYYYYYYYYYYYY",
                "YYRRRRYYRRRRYY",
                "YYRRRRYYRRRRYY",
                "YYYYYYYYYYYYYY",
                middle,
                middle,
                "YYYYYYYYYYYYYY",
            ])
        );
    }
}

#[test]
fn gradient_phase_spans_the_row_and_fixed_track_overflow_preserves_old_hit_bounds() {
    let (doc, styles) = prepare(
        "#r{background-image:linear-gradient(to right,red 4px,blue 4px)}",
        ONE_ROW,
        14,
        4,
    );
    let output = layout(&doc, &styles, 14.0, 4.0, &Fonts::new());
    assert_eq!(
        gradients(&output),
        vec![
            (rect(2.0, 1.0, 4.0, 2.0), rect(2.0, 1.0, 10.0, 2.0)),
            (rect(8.0, 1.0, 4.0, 2.0), rect(2.0, 1.0, 10.0, 2.0)),
        ]
    );
    assert_eq!(
        pixels(&output, 14, 4),
        literal(&[
            "YYYYYYYYYYYYYY",
            "YYRRRRYYBBBBYY",
            "YYRRRRYYBBBBYY",
            "YYYYYYYYYYYYYY"
        ])
    );
    let (doc, styles) = prepare(
        "table{width:10px}td{width:6px}#r{background-image:linear-gradient(to right,red 50%,blue 50%)}",
        ONE_ROW,
        18,
        4,
    );
    let output = layout(&doc, &styles, 18.0, 4.0, &Fonts::new());
    assert_eq!(
        gradients(&output),
        vec![
            (rect(2.0, 1.0, 6.0, 2.0), rect(2.0, 1.0, 14.0, 2.0)),
            (rect(10.0, 1.0, 6.0, 2.0), rect(2.0, 1.0, 14.0, 2.0)),
        ]
    );
    let row = doc.query_selector("#r").unwrap();
    assert_eq!(
        output
            .hit_regions
            .iter()
            .find(|h| h.node == row)
            .unwrap()
            .rect,
        rect(2.0, 1.0, 6.0, 2.0)
    );
    assert_eq!(
        pixels(&output, 18, 4),
        literal(&[
            "YYYYYYYYYYWWWWWWWW",
            "YYRRRRRRYYBBBBBBWW",
            "YYRRRRRRYYBBBBBBWW",
            "YYYYYYYYYYWWWWWWWW"
        ])
    );
}

#[test]
fn url_tiles_keep_the_rows_loaded_binding_and_repeat_phase() {
    let (doc, styles) = prepare(
        "#r{background-image:url(tile);background-size:4px 2px;background-repeat:repeat-x}",
        ONE_ROW,
        14,
        4,
    );
    let ranges = row_bindings(&doc, 1);
    let slots = [loaded("rowk")];
    let output = layout_with_backgrounds(
        &doc,
        &styles,
        14.0,
        4.0,
        &Fonts::new(),
        BackgroundBindings {
            ranges: &ranges,
            slots: &slots,
        },
    );
    let images = output
        .commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::BackgroundImage {
                rect,
                tile,
                key,
                repeat,
                ..
            } => Some((*rect, *tile, key.as_str(), *repeat)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        images,
        vec![
            (
                rect(2.0, 1.0, 4.0, 2.0),
                rect(2.0, 1.0, 4.0, 2.0),
                "rowk",
                [true, false]
            ),
            (
                rect(8.0, 1.0, 4.0, 2.0),
                rect(2.0, 1.0, 4.0, 2.0),
                "rowk",
                [true, false]
            ),
        ]
    );
    assert_eq!(
        pixels(&output, 14, 4),
        literal(&[
            "YYYYYYYYYYYYYY",
            "YYRRBBYYBBRRYY",
            "YYRRBBYYBBRRYY",
            "YYYYYYYYYYYYYY"
        ])
    );
}

#[test]
fn spanning_coverage_does_not_stretch_the_row_tile_or_enable_no_repeat() {
    let body =
        "<table id=t><tr id=r><td id=a rowspan=2></td><td></td></tr><tr><td></td></tr></table>";
    for (repeat, lower) in [
        ("no-repeat", "YYYYYYYYYYYYYY"),
        ("repeat-y", "YYBBBBYYYYYYYY"),
    ] {
        let rules = format!(
            "#a{{height:5px}}#r{{background-image:linear-gradient(blue,blue);background-repeat:{repeat}}}"
        );
        let (doc, styles) = prepare(&rules, body, 14, 7);
        let output = layout(&doc, &styles, 14.0, 7.0, &Fonts::new());
        assert_eq!(
            gradients(&output),
            vec![
                (rect(2.0, 1.0, 4.0, 5.0), rect(2.0, 1.0, 10.0, 2.0)),
                (rect(8.0, 1.0, 4.0, 2.0), rect(2.0, 1.0, 10.0, 2.0)),
            ]
        );
        assert_eq!(
            pixels(&output, 14, 7),
            literal(&[
                "YYYYYYYYYYYYYY",
                "YYBBBBYYBBBBYY",
                "YYBBBBYYBBBBYY",
                lower,
                lower,
                lower,
                "YYYYYYYYYYYYYY"
            ])
        );
    }
}

#[test]
fn authored_layer_gaps_and_unavailable_urls_do_not_change_paint_order() {
    for middle in ["none", "url(missing)"] {
        let rules = format!(
            "#r{{background-image:linear-gradient(to right,red 2px,transparent 2px),{middle},linear-gradient(blue,blue);background-repeat:no-repeat}}"
        );
        assert_eq!(
            render(&rules, ONE_ROW, 14, 4),
            literal(&[
                "YYYYYYYYYYYYYY",
                "YYRRBBYYBBBBYY",
                "YYRRBBYYBBBBYY",
                "YYYYYYYYYYYYYY"
            ])
        );
    }
}

#[test]
fn replicated_row_paint_and_cell_descendants_share_one_opacity_scope() {
    let body = "<table id=t><tr id=r><td><i></i></td><td></td></tr></table>";
    let (doc, styles) = prepare(
        "table{background:white}#r{background:red;opacity:.5}i{display:block;width:4px;height:2px;background:blue}",
        body,
        14,
        4,
    );
    let output = layout(&doc, &styles, 14.0, 4.0, &Fonts::new());
    assert_eq!(
        pixels(&output, 14, 4),
        literal(&[
            "WWWWWWWWWWWWWW",
            "WWLLLLWWPPPPWW",
            "WWLLLLWWPPPPWW",
            "WWWWWWWWWWWWWW"
        ])
    );
    assert_eq!(
        output
            .commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::PushOpacity { .. }))
            .count(),
        1
    );
    assert_eq!(
        output
            .commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::PopOpacity))
            .count(),
        1
    );
}

#[test]
fn transparent_single_none_rows_do_not_spend_or_publish() {
    let (doc, styles) = prepare("", ONE_ROW, 14, 4);
    let fonts = Fonts::new();
    let row = doc.query_selector("#r").unwrap();
    let cell = doc.query_selector("#a").unwrap();
    let fragment = fragment(cell, rect(0.0, 0.0, 4.0, 2.0));
    let mut e = engine(&doc, &styles, &fonts, BackgroundBindings::default());
    e.position_work_left = 7;
    e.background_key_bytes_left = 0;
    e.commands_created = MAX_COMMANDS;
    assert!(!e.paint_table_cell_row_background(
        row,
        cell,
        &fragment,
        [2.0, 1.0],
        rect(2.0, 1.0, 10.0, 2.0)
    ));
    assert_eq!(e.position_work_left, 7);
    assert_eq!(e.background_key_bytes_left, 0);
    assert_eq!(e.commands_created, MAX_COMMANDS);
    assert!(e.commands.is_empty() && e.owners.is_empty());
}

#[test]
fn cell_group_slots_and_geometry_work_have_independent_exact_boundaries() {
    let (doc, styles) = prepare(
        "#r{background-image:linear-gradient(red,red),linear-gradient(blue,blue)}",
        ONE_ROW,
        14,
        4,
    );
    let fonts = Fonts::new();
    let row = doc.query_selector("#r").unwrap();
    let cell = doc.query_selector("#a").unwrap();
    let fragment = fragment(cell, rect(0.0, 0.0, 4.0, 2.0));
    for (room, work, success, remainder) in
        [(2, 96, false, 96), (3, 47, false, 47), (3, 48, true, 0)]
    {
        let mut e = engine(&doc, &styles, &fonts, BackgroundBindings::default());
        e.commands_created = MAX_COMMANDS - room;
        e.position_work_left = work;
        assert_eq!(
            e.paint_table_cell_row_background(
                row,
                cell,
                &fragment,
                [2.0, 1.0],
                rect(2.0, 1.0, 10.0, 2.0)
            ),
            success
        );
        assert_eq!(e.position_work_left, remainder);
        assert_eq!(e.commands.len(), if success { 3 } else { 0 });
        assert_eq!(e.owners.len(), e.commands.len());
        assert_eq!(e.commands_created, MAX_COMMANDS - room + e.commands.len());
        assert_eq!(e.open_clips, 0);
        assert_eq!(e.paint_owner.node, doc.root);
        assert!(!e.paint_owner.background);
        assert_eq!(e.background_key_bytes_left, MAX_BACKGROUND_KEY_BYTES);
        if success {
            assert!(e.owners.iter().all(|o| o.node == row && o.background));
            assert_eq!(
                e.owners
                    .iter()
                    .map(|o| o.background_image_index)
                    .collect::<Vec<_>>(),
                vec![0, 1, 0]
            );
        }
    }
}

#[test]
fn per_layer_key_refusal_keeps_geometry_phase_and_other_layers() {
    let (doc, styles) = prepare(
        "#r{background-image:url(tile),none,linear-gradient(blue,blue);background-size:4px 2px;background-repeat:repeat-x}",
        ONE_ROW,
        14,
        4,
    );
    let fonts = Fonts::new();
    let row = doc.query_selector("#r").unwrap();
    let cell = doc.query_selector("#a").unwrap();
    let fragment = fragment(cell, rect(0.0, 0.0, 4.0, 2.0));
    let ranges = row_bindings(&doc, 3);
    let slots = [loaded("rowk"), None, None];
    let mut e = engine(
        &doc,
        &styles,
        &fonts,
        BackgroundBindings {
            ranges: &ranges,
            slots: &slots,
        },
    );
    e.background_key_bytes_left = 4;
    e.position_work_left = 133; // two64-unit groups and one5-unit key clone
    for x in [2.0, 8.0] {
        assert!(e.paint_table_cell_row_background(
            row,
            cell,
            &fragment,
            [x, 1.0],
            rect(2.0, 1.0, 10.0, 2.0)
        ));
    }
    assert_eq!(e.position_work_left, 0);
    assert_eq!(e.background_key_bytes_left, 0);
    assert_eq!(e.commands.len(), 8);
    assert_eq!(
        e.commands
            .iter()
            .filter(|c| matches!(c, DrawCommand::BackgroundImage { .. }))
            .count(),
        1
    );
    let tiles = e
        .commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::LinearGradient { tile, .. } => Some(*tile),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(tiles, vec![rect(2.0, 1.0, 4.0, 2.0); 2]);
    assert_eq!(
        e.owners
            .iter()
            .map(|o| o.background_image_index)
            .collect::<Vec<_>>(),
        vec![0, 2, 1, 0, 0, 2, 1, 0]
    );
    assert_eq!(fragment.boxes[0].border, rect(0.0, 0.0, 4.0, 2.0));
}

#[test]
fn final_principal_coverage_is_checked_without_clamping_or_child_substitution() {
    let (doc, styles) = prepare("#r{background:red}", ONE_ROW, 14, 4);
    let fonts = Fonts::new();
    let row = doc.query_selector("#r").unwrap();
    let cell = doc.query_selector("#a").unwrap();
    for (x, dx, accepted, expected) in [
        (999_999.0, 1.0, true, 1_000_000.0),
        (999_999.0, 2.0, false, 0.0),
        (-999_999.0, -1.0, true, -1_000_000.0),
        (-999_999.0, -2.0, false, 0.0),
        (f32::NAN, 0.0, false, 0.0),
    ] {
        let f = fragment(
            cell,
            Rect {
                x,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
        );
        let mut e = engine(&doc, &styles, &fonts, BackgroundBindings::default());
        e.position_work_left = 16;
        assert_eq!(
            e.paint_table_cell_row_background(row, cell, &f, [dx, 0.0], rect(0.0, 0.0, 1.0, 1.0)),
            accepted
        );
        assert_eq!(e.position_work_left, 0);
        assert_eq!(e.commands.len(), usize::from(accepted));
        if accepted {
            assert!(matches!(e.commands[0], DrawCommand::Rect { rect: r, .. } if r.x == expected));
        }
    }
    for wrong in [false, true] {
        let mut f = fragment(if wrong { row } else { cell }, rect(0.0, 0.0, 4.0, 2.0));
        if !wrong {
            f.principal_box = u32::MAX;
        }
        let mut e = engine(&doc, &styles, &fonts, BackgroundBindings::default());
        e.position_work_left = 16;
        assert!(!e.paint_table_cell_row_background(
            row,
            cell,
            &f,
            [2.0, 1.0],
            rect(2.0, 1.0, 10.0, 2.0)
        ));
        assert!(e.commands.is_empty());
        assert_eq!(e.position_work_left, 0);
        assert_eq!(f.boxes.len(), 1);
    }
}

#[test]
fn row_paint_budget_refusal_does_not_change_cell_or_rowspan_geometry() {
    let body = "<table id=t><tr id=r><td id=a rowspan=2></td><td id=b></td></tr><tr id=s><td id=c></td></tr></table>";
    let (doc, styles) = prepare(
        "#r{background:red}#s{background:blue}#a{height:5px}",
        body,
        14,
        7,
    );
    let fonts = Fonts::new();
    let table = doc.query_selector("#t").unwrap();
    let children = &doc.nodes[table].children;
    for work in [0, 1, 2, 17, 18, MAX_POSITION_WORK] {
        let mut e = engine(&doc, &styles, &fonts, BackgroundBindings::default());
        e.position_work_left = work;
        assert_eq!(e.layout_table(children, 0.0, 0.0, 14.0, [2.0, 1.0], 0), 7.0);
        for (name, expected) in [
            ("#a", rect(2.0, 1.0, 4.0, 5.0)),
            ("#b", rect(8.0, 1.0, 4.0, 2.0)),
            ("#c", rect(8.0, 4.0, 4.0, 2.0)),
        ] {
            let node = doc.query_selector(name).unwrap();
            assert_eq!(
                e.hits.iter().find(|h| h.node == node).unwrap().rect,
                expected
            );
            assert_eq!(
                e.boxes
                    .iter()
                    .find(|g| g.node == node && !g.inline)
                    .unwrap()
                    .border,
                expected
            );
        }
        assert_eq!(e.open_clips, 0);
        assert!(e.commands_created <= MAX_COMMANDS);
        if work < 18 {
            assert!(
                !e.owners
                    .iter()
                    .any(|o| o.node == doc.query_selector("#r").unwrap() && o.background)
            );
        }
    }
}
