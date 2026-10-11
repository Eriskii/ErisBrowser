//! Literal table geometry frozen before spacing implementation; no measured oracles.
use super::*;
use crate::graphics::{Canvas, ImageStore};

fn prepare(rules: &str, body: &str) -> (Document, Vec<ComputedStyle>) {
    let source = format!(
        "<style>html,body{{margin:0;padding:0}}html{{background:white}}table{{box-sizing:content-box;width:20px;margin:0;padding:0;border:0}}td,th{{box-sizing:border-box;padding:0;border:0}}caption{{margin:0;padding:0;border:0}}{rules}</style>{body}"
    );
    let doc = Document::parse(&source);
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 64.0, 64.0);
    (doc, styles)
}
fn pair(styles: &mut [ComputedStyle], doc: &Document, value: [f32; 2]) {
    styles[doc.query_selector("#t").unwrap()].border_spacing = value;
}
fn result(doc: &Document, styles: &[ComputedStyle]) -> LayoutResult {
    layout(doc, styles, 64.0, 64.0, &Fonts::new())
}
fn hit(doc: &Document, result: &LayoutResult, selector: &str) -> Rect {
    let node = doc.query_selector(selector).unwrap();
    result
        .hit_regions
        .iter()
        .find(|hit| hit.node == node)
        .unwrap()
        .rect
}
fn board() -> &'static str {
    "<table id=t><tr id=r0><td id=a></td><td id=b></td></tr><tr id=r1><td id=c></td><td id=d></td></tr></table>"
}
fn board_rules() -> &'static str {
    "#a,#b{height:4px}#c,#d{height:6px}"
}
fn cells(doc: &Document, result: &LayoutResult) -> [Rect; 4] {
    ["#a", "#b", "#c", "#d"].map(|selector| hit(doc, result, selector))
}

#[test]
fn default_pair_and_each_axis_preserve_literal_tracks_and_outer_gaps() {
    let (doc, mut styles) = prepare(board_rules(), board());
    assert_eq!(
        styles[doc.query_selector("#t").unwrap()].border_spacing,
        [2.0, 2.0]
    );
    let initial = result(&doc, &styles);
    assert_eq!(
        cells(&doc, &initial),
        [
            rect(2.0, 2.0, 7.0, 4.0),
            rect(11.0, 2.0, 7.0, 4.0),
            rect(2.0, 8.0, 7.0, 6.0),
            rect(11.0, 8.0, 7.0, 6.0)
        ]
    );
    assert_eq!(hit(&doc, &initial, "#t"), rect(0.0, 0.0, 20.0, 16.0));
    for (spacing, width, expected, height) in [
        (
            [2.0, 3.0],
            20.0,
            [
                rect(2.0, 3.0, 7.0, 4.0),
                rect(11.0, 3.0, 7.0, 4.0),
                rect(2.0, 10.0, 7.0, 6.0),
                rect(11.0, 10.0, 7.0, 6.0),
            ],
            19.0,
        ),
        (
            [3.0, 2.0],
            21.0,
            [
                rect(3.0, 2.0, 6.0, 4.0),
                rect(12.0, 2.0, 6.0, 4.0),
                rect(3.0, 8.0, 6.0, 6.0),
                rect(12.0, 8.0, 6.0, 6.0),
            ],
            16.0,
        ),
        (
            [0.0, 3.0],
            20.0,
            [
                rect(0.0, 3.0, 10.0, 4.0),
                rect(10.0, 3.0, 10.0, 4.0),
                rect(0.0, 10.0, 10.0, 6.0),
                rect(10.0, 10.0, 10.0, 6.0),
            ],
            19.0,
        ),
        (
            [2.0, 0.0],
            20.0,
            [
                rect(2.0, 0.0, 7.0, 4.0),
                rect(11.0, 0.0, 7.0, 4.0),
                rect(2.0, 4.0, 7.0, 6.0),
                rect(11.0, 4.0, 7.0, 6.0),
            ],
            10.0,
        ),
        (
            [0.0, 0.0],
            20.0,
            [
                rect(0.0, 0.0, 10.0, 4.0),
                rect(10.0, 0.0, 10.0, 4.0),
                rect(0.0, 4.0, 10.0, 6.0),
                rect(10.0, 4.0, 10.0, 6.0),
            ],
            10.0,
        ),
        (
            [3.0, 3.0],
            21.0,
            [
                rect(3.0, 3.0, 6.0, 4.0),
                rect(12.0, 3.0, 6.0, 4.0),
                rect(3.0, 10.0, 6.0, 6.0),
                rect(12.0, 10.0, 6.0, 6.0),
            ],
            19.0,
        ),
    ] {
        pair(&mut styles, &doc, spacing);
        styles[doc.query_selector("#t").unwrap()].width = Length::Px(width);
        let output = result(&doc, &styles);
        assert_eq!(cells(&doc, &output), expected);
        assert_eq!(hit(&doc, &output, "#t"), rect(0.0, 0.0, width, height));
    }
}

#[test]
fn unequal_gaps_show_the_table_fill_between_opaque_cells() {
    let (doc, mut styles) = prepare(
        "table{background:#ffff00}#a,#b{height:4px}#c,#d{height:6px}#a{background:red}#b{background:blue}#c{background:lime}#d{background:black}",
        board(),
    );
    pair(&mut styles, &doc, [2.0, 3.0]);
    let output = result(&doc, &styles);
    let mut canvas = Canvas::new(20, 19).unwrap();
    canvas.paint(
        &output.commands,
        &Fonts::new(),
        &ImageStore::new(),
        0.0,
        0.0,
    );
    assert!(!canvas.exhausted());
    let row = |text: &str| {
        text.bytes()
            .map(|c| match c {
                b'Y' => 0xffff00,
                b'R' => 0xff0000,
                b'B' => 0x0000ff,
                b'G' => 0x00ff00,
                b'K' => 0,
                _ => panic!("literal palette"),
            })
            .collect::<Vec<_>>()
    };
    let expected = [
        (3, "YYYYYYYYYYYYYYYYYYYY"),
        (4, "YYRRRRRRRYYBBBBBBBYY"),
        (3, "YYYYYYYYYYYYYYYYYYYY"),
        (6, "YYGGGGGGGYYKKKKKKKYY"),
        (3, "YYYYYYYYYYYYYYYYYYYY"),
    ]
    .into_iter()
    .flat_map(|(count, text)| row(text).repeat(count))
    .collect::<Vec<_>>();
    assert_eq!(canvas.pixels, expected);
}

#[test]
fn colspan_and_rowspan_use_distinct_axes_before_deficit_distribution() {
    let body = "<table id=t><tr><td id=a colspan=2 rowspan=2></td><td id=b></td></tr><tr><td id=c></td></tr></table>";
    let (doc, mut styles) = prepare(
        "table{width:26px}#a{height:13px}#b{height:4px}#c{height:6px}",
        body,
    );
    pair(&mut styles, &doc, [2.0, 3.0]);
    for (a_height, expected, height) in [
        (
            13.0,
            [
                rect(2.0, 3.0, 14.0, 13.0),
                rect(18.0, 3.0, 6.0, 4.0),
                rect(18.0, 10.0, 6.0, 6.0),
            ],
            19.0,
        ),
        (
            17.0,
            [
                rect(2.0, 3.0, 14.0, 17.0),
                rect(18.0, 3.0, 6.0, 6.0),
                rect(18.0, 12.0, 6.0, 8.0),
            ],
            23.0,
        ),
    ] {
        styles[doc.query_selector("#a").unwrap()].height = Length::Px(a_height);
        let output = result(&doc, &styles);
        assert_eq!(
            ["#a", "#b", "#c"].map(|selector| hit(&doc, &output, selector)),
            expected
        );
        assert_eq!(hit(&doc, &output, "#t").height, height);
    }
}

#[test]
fn explicit_colspan_and_percentage_bases_use_the_owning_table_pair() {
    let (doc, mut styles) = prepare(
        "table{width:32px}#a{width:17px;height:4px}#b{width:6px;height:4px}",
        "<table id=t><tr id=r><td id=a colspan=2></td><td id=b></td></tr></table>",
    );
    pair(&mut styles, &doc, [3.0, 5.0]);
    styles[doc.query_selector("#r").unwrap()].border_spacing = [9.0, 11.0];
    for selector in ["#a", "#b"] {
        styles[doc.query_selector(selector).unwrap()].border_spacing = [0.0, 0.0];
    }
    let output = result(&doc, &styles);
    assert_eq!(hit(&doc, &output, "#a"), rect(3.0, 5.0, 17.0, 4.0));
    assert_eq!(hit(&doc, &output, "#b"), rect(23.0, 5.0, 6.0, 4.0));
    assert_eq!(hit(&doc, &output, "#r"), rect(3.0, 5.0, 26.0, 4.0));
    assert_eq!(hit(&doc, &output, "#t").height, 14.0);
    let (doc, mut styles) = prepare(
        "table{width:40px}td{height:4px}#a{width:50%;padding-left:10%;background:linear-gradient(red);background-origin:content-box;background-clip:content-box}",
        "<table id=t><tr><td id=a></td><td id=b></td></tr></table>",
    );
    pair(&mut styles, &doc, [4.0, 2.0]);
    let output = result(&doc, &styles);
    assert_eq!(hit(&doc, &output, "#a"), rect(4.0, 2.0, 20.0, 4.0));
    assert_eq!(hit(&doc, &output, "#b"), rect(28.0, 2.0, 8.0, 4.0));
    let selected = output.commands.iter().find_map(|command| match command {
        DrawCommand::BoxDecoration(decoration)
            if matches!(
                decoration.paint,
                crate::graphics::box_decoration::BoxPaint::LinearGradient { .. }
            ) =>
        {
            Some(decoration.shape.rect)
        }
        _ => None,
    });
    assert_eq!(selected, Some(rect(6.0, 2.0, 18.0, 4.0)));
    assert_eq!(hit(&doc, &output, "#t").height, 8.0);
}

#[test]
fn captions_insets_and_empty_rows_keep_their_existing_outer_box_policy() {
    let (doc, mut styles) = prepare(
        "table{position:relative;left:10px;top:10px;padding:1px 2px 3px 4px;border:1px solid black}#a,#b{height:4px}#c,#d{height:6px}",
        board(),
    );
    pair(&mut styles, &doc, [2.0, 3.0]);
    let output = result(&doc, &styles);
    assert_eq!(hit(&doc, &output, "#t"), rect(10.0, 10.0, 28.0, 25.0));
    assert_eq!(
        cells(&doc, &output),
        [
            rect(17.0, 15.0, 7.0, 4.0),
            rect(26.0, 15.0, 7.0, 4.0),
            rect(17.0, 22.0, 7.0, 6.0),
            rect(26.0, 22.0, 7.0, 6.0)
        ]
    );
    let (doc, mut styles) = prepare(
        "caption{height:3px}#a,#b{height:4px}#c,#d{height:6px}",
        "<table id=t><caption id=cap></caption><tr><td id=a></td><td id=b></td></tr><tr><td id=c></td><td id=d></td></tr></table>",
    );
    pair(&mut styles, &doc, [2.0, 3.0]);
    let output = result(&doc, &styles);
    assert_eq!(hit(&doc, &output, "#cap"), rect(0.0, 0.0, 20.0, 3.0));
    assert_eq!(
        cells(&doc, &output),
        [
            rect(2.0, 6.0, 7.0, 4.0),
            rect(11.0, 6.0, 7.0, 4.0),
            rect(2.0, 13.0, 7.0, 6.0),
            rect(11.0, 13.0, 7.0, 6.0)
        ]
    );
    assert_eq!(hit(&doc, &output, "#t").height, 22.0);
    for (body, height) in [
        ("<table id=t></table>", 0.0),
        ("<table id=t><caption></caption></table>", 3.0),
        ("<table id=t><tr></tr></table>", 6.0),
    ] {
        let (doc, mut styles) = prepare("caption{height:3px}", body);
        pair(&mut styles, &doc, [2.0, 3.0]);
        assert_eq!(hit(&doc, &result(&doc, &styles), "#t").height, height);
    }
}

#[test]
fn narrow_tracks_and_fixed_overflow_preserve_the_current_row_rectangle() {
    let (doc, mut styles) = prepare(
        "table{width:10px}#a,#b{height:4px}",
        "<table id=t><tr id=r><td id=a></td><td id=b></td></tr></table>",
    );
    pair(&mut styles, &doc, [4.0, 3.0]);
    let output = result(&doc, &styles);
    assert_eq!(hit(&doc, &output, "#a"), rect(4.0, 3.0, 0.0, 4.0));
    assert_eq!(hit(&doc, &output, "#b"), rect(8.0, 3.0, 0.0, 4.0));
    assert_eq!(hit(&doc, &output, "#r"), rect(4.0, 3.0, 4.0, 4.0));
    assert_eq!(hit(&doc, &output, "#t").height, 10.0);
    pair(&mut styles, &doc, [2.0, 3.0]);
    for selector in ["#a", "#b"] {
        styles[doc.query_selector(selector).unwrap()].width = Length::Px(6.0);
    }
    let output = result(&doc, &styles);
    assert_eq!(hit(&doc, &output, "#a"), rect(2.0, 3.0, 6.0, 4.0));
    assert_eq!(hit(&doc, &output, "#b"), rect(10.0, 3.0, 6.0, 4.0));
    assert_eq!(hit(&doc, &output, "#r"), rect(2.0, 3.0, 6.0, 4.0));
}

#[test]
fn spacing_driven_rowspan_retiles_url_and_gradient_without_replacing_the_key() {
    let (doc, mut styles) = prepare(
        "table{width:26px}#a{height:0;background-image:url(tile.png),linear-gradient(red,blue);background-size:auto 100%;background-position:center;background-origin:content-box;background-clip:content-box;background-repeat:no-repeat}#b{height:4px}#c{height:6px}",
        "<table id=t><tr><td id=a colspan=2 rowspan=2></td><td id=b></td></tr><tr><td id=c></td></tr></table>",
    );
    pair(&mut styles, &doc, [2.0, 3.0]);
    let a = doc.query_selector("#a").unwrap();
    let mut ranges = vec![BackgroundBindingRange::default(); doc.nodes.len()];
    ranges[a] = BackgroundBindingRange { start: 0, len: 2 };
    let slots = [
        Some(LoadedBackground {
            key: "literal-key",
            natural: RasterNaturalSize {
                width: 2,
                height: 1,
            },
        }),
        None,
    ];
    let output = layout_with_backgrounds(
        &doc,
        &styles,
        64.0,
        64.0,
        &Fonts::new(),
        BackgroundBindings {
            ranges: &ranges,
            slots: &slots,
        },
    );
    assert_eq!(hit(&doc, &output, "#a"), rect(2.0, 3.0, 14.0, 13.0));
    let paints = output
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::BoxDecoration(decoration) => match &decoration.paint {
                crate::graphics::box_decoration::BoxPaint::LinearGradient { tile, .. } => {
                    Some((decoration.shape.rect, *tile, None))
                }
                crate::graphics::box_decoration::BoxPaint::Image { tile, key, .. } => {
                    Some((decoration.shape.rect, *tile, Some(key.as_str())))
                }
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        paints,
        vec![
            (rect(2.0, 3.0, 14.0, 13.0), rect(2.0, 3.0, 14.0, 13.0), None),
            (
                rect(2.0, 3.0, 14.0, 13.0),
                rect(-4.0, 3.0, 26.0, 13.0),
                Some("literal-key")
            )
        ]
    );
}

#[test]
fn synthetic_nonfinite_negative_and_oversized_pairs_use_existing_extent_bounds() {
    let (doc, mut styles) = prepare(board_rules(), board());
    for (input, normalized) in [
        ([f32::NAN, -1.0], [0.0, 0.0]),
        ([f32::INFINITY, 3.0], [0.0, 3.0]),
        ([2.0, f32::NEG_INFINITY], [2.0, 0.0]),
        ([1_000_001.0, 2.0], [1_000_000.0, 2.0]),
    ] {
        pair(&mut styles, &doc, input);
        let actual = result(&doc, &styles);
        pair(&mut styles, &doc, normalized);
        let expected = result(&doc, &styles);
        assert_eq!(cells(&doc, &actual), cells(&doc, &expected));
        assert_eq!(hit(&doc, &actual, "#t"), hit(&doc, &expected, "#t"));
        assert!(actual.hit_regions.iter().all(|hit| {
            [hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height]
                .into_iter()
                .all(f32::is_finite)
        }));
        assert!(actual.commands.len() <= MAX_COMMANDS);
    }
}
