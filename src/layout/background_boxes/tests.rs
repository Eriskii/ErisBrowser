use super::*;

fn sides(top: f32, right: f32, bottom: f32, left: f32) -> Sides {
    Sides {
        top,
        right,
        bottom,
        left,
    }
}
fn shape(area: Rect, radius: f32) -> RoundedBox {
    RoundedBox {
        rect: area,
        radii: [[radius; 2]; 4],
    }
}
fn boxes() -> BackgroundBoxes {
    BackgroundBoxes::new(
        shape(rect(10.0, 20.0, 30.0, 24.0), 6.0),
        sides(1.0, 2.0, 3.0, 4.0),
        sides(2.0, 1.0, 0.0, 1.0),
    )
    .unwrap()
}
#[test]
fn asymmetric_border_and_padding_have_three_distinct_shapes() {
    let b = boxes();
    assert_eq!(b.border, shape(rect(10.0, 20.0, 30.0, 24.0), 6.0));
    assert_eq!(
        b.padding,
        RoundedBox {
            rect: rect(14.0, 21.0, 24.0, 20.0),
            radii: [[2.0, 5.0], [4.0, 5.0], [4.0, 3.0], [2.0, 3.0]]
        }
    );
    assert_eq!(
        b.content,
        RoundedBox {
            rect: rect(15.0, 23.0, 22.0, 18.0),
            radii: [[1.0, 3.0], [3.0, 3.0], [3.0, 3.0], [1.0, 3.0]]
        }
    );
}
#[test]
fn content_insets_sum_against_original_normalized_outer() {
    // Original radius9 normalizes to4. The2px inner width must NOT rescale
    // the retained4px right arcs when the padding bottom is subtracted.
    let b = BackgroundBoxes::new(
        shape(rect(0.0, 0.0, 12.0, 8.0), 9.0),
        sides(0.0, 0.0, 0.0, 10.0),
        sides(0.0, 0.0, 1.0, 0.0),
    )
    .unwrap();
    assert_eq!(b.border, shape(rect(0.0, 0.0, 12.0, 8.0), 4.0));
    assert_eq!(
        b.padding,
        RoundedBox {
            rect: rect(10.0, 0.0, 2.0, 8.0),
            radii: [[0.0, 4.0], [4.0, 4.0], [4.0, 4.0], [0.0, 4.0]]
        }
    );
    assert_eq!(
        b.content,
        RoundedBox {
            rect: rect(10.0, 0.0, 2.0, 7.0),
            radii: [[0.0, 4.0], [4.0, 4.0], [4.0, 3.0], [0.0, 3.0]]
        }
    );
}
#[test]
fn every_origin_clip_pair_is_independent_for_gradient_and_raster_tiles() {
    let b = boxes();
    let cases = [
        (
            BackgroundBox::Border,
            rect(10.0, 20.0, 30.0, 24.0),
            rect(23.0, 31.0, 4.0, 2.0),
        ),
        (
            BackgroundBox::Padding,
            rect(14.0, 21.0, 24.0, 20.0),
            rect(24.0, 30.0, 4.0, 2.0),
        ),
        (
            BackgroundBox::Content,
            rect(15.0, 23.0, 22.0, 18.0),
            rect(24.0, 31.0, 4.0, 2.0),
        ),
    ];
    let center = BackgroundPosition {
        x: PositionComponent {
            from_end: false,
            offset: Length::Percent(50.0),
        },
        y: PositionComponent {
            from_end: false,
            offset: Length::Percent(50.0),
        },
    };
    for &(origin, area, raster) in &cases {
        for &(clip, paint, _) in &cases {
            assert_eq!(b.select(clip).rect, paint);
            assert_eq!(
                background_tile(
                    b.select(origin).rect,
                    BackgroundSize::default(),
                    BackgroundPosition::default()
                ),
                Some(area)
            );
            assert_eq!(
                background_tile_with_raster(
                    b.select(origin).rect,
                    BackgroundSize::default(),
                    center,
                    RasterNaturalSize {
                        width: 4,
                        height: 2
                    }
                ),
                Some(raster)
            );
        }
    }
}
fn source(origin: &str, clip: &str, image: &str) -> String {
    format!(
        "<style>html,body{{margin:0;padding:0}}#box{{box-sizing:border-box;width:16px;height:12px;border:2px solid transparent;padding:2px;background-image:{image};background-origin:{origin};background-clip:{clip};background-repeat:no-repeat}}</style><div id=box></div>"
    )
}
fn image_geometry(command: &DrawCommand) -> Option<(Rect, Rect)> {
    match command {
        DrawCommand::LinearGradient { rect, tile, .. }
        | DrawCommand::BackgroundImage { rect, tile, .. } => Some((*rect, *tile)),
        DrawCommand::BoxDecoration(d) => match &d.paint {
            BoxPaint::LinearGradient { tile, .. } | BoxPaint::Image { tile, .. } => {
                Some((d.shape.rect, *tile))
            }
            _ => None,
        },
        _ => None,
    }
}
#[test]
fn principal_gradient_uses_all_nine_box_pairs() {
    let cases = [
        ("border-box", rect(0.0, 0.0, 16.0, 12.0)),
        ("padding-box", rect(2.0, 2.0, 12.0, 8.0)),
        ("content-box", rect(4.0, 4.0, 8.0, 4.0)),
    ];
    for &(origin, area) in &cases {
        for &(clip, paint) in &cases {
            let doc = Document::parse(&source(origin, clip, "linear-gradient(red,blue)"));
            let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
            let result = layout(&doc, &styles, 32.0, 24.0, &Fonts::new());
            let images: Vec<_> = result.commands.iter().filter_map(image_geometry).collect();
            assert_eq!(images, vec![(paint, area)]);
        }
    }
}
#[test]
fn principal_loaded_url_uses_all_nine_box_pairs_without_resizing_natural_auto() {
    let cases = [
        ("border-box", rect(0.0, 0.0, 16.0, 12.0)),
        ("padding-box", rect(2.0, 2.0, 12.0, 8.0)),
        ("content-box", rect(4.0, 4.0, 8.0, 4.0)),
    ];
    for &(origin, area) in &cases {
        for &(clip, paint) in &cases {
            let doc = Document::parse(&source(origin, clip, "url(tile.png)"));
            let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
            let id = doc.query_selector("#box").unwrap();
            let mut loaded = vec![None; doc.nodes.len()];
            loaded[id] = Some(LoadedBackground {
                key: "private-tile",
                natural: RasterNaturalSize {
                    width: 4,
                    height: 2,
                },
            });
            let result = layout_with_backgrounds(&doc, &styles, 32.0, 24.0, &Fonts::new(), &loaded);
            let images: Vec<_> = result.commands.iter().filter_map(image_geometry).collect();
            assert_eq!(images, vec![(paint, rect(area.x, area.y, 4.0, 2.0))]);
        }
    }
}
#[test]
fn transparent_rounded_interior_is_a_ring_not_an_opaque_underlay() {
    let doc = Document::parse(
        "<style>body{margin:0;background:white}#box{box-sizing:border-box;width:8px;height:8px;border:1px solid rgba(255,0,0,.5);border-radius:3px;background:transparent}</style><div id=box></div>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 8.0, 8.0);
    let fonts = Fonts::new();
    let result = layout(&doc, &styles, 8.0, 8.0, &fonts);
    let ring = result
        .commands
        .iter()
        .find_map(|c| {
            if let DrawCommand::BoxDecoration(d) = c {
                d.cutout.is_some().then_some(d)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(ring.shape, shape(rect(0.0, 0.0, 8.0, 8.0), 3.0));
    assert_eq!(ring.cutout, Some(shape(rect(1.0, 1.0, 6.0, 6.0), 2.0)));
    let mut canvas = crate::graphics::Canvas::new(8, 8).unwrap();
    canvas.paint(
        &result.commands,
        &fonts,
        &crate::graphics::ImageStore::new(),
        0.0,
        0.0,
    );
    assert_eq!(canvas.pixels[0], 0xffffff);
    assert_eq!(canvas.pixels[3], 0xff7f7f);
    assert_eq!(canvas.pixels[3 * 8 + 3], 0xffffff);
}
#[test]
fn safe_default_opaque_rounded_path_preserves_legacy_commands() {
    let doc = Document::parse(
        "<style>body{margin:0}#box{box-sizing:border-box;width:8px;height:8px;border:1px solid black;border-radius:3px;background:red}</style><div id=box></div>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 8.0, 8.0);
    let result = layout(&doc, &styles, 8.0, 8.0, &Fonts::new());
    assert!(
        !result
            .commands
            .iter()
            .any(|c| matches!(c, DrawCommand::BoxDecoration(_)))
    );
    assert!(result.commands.iter().any(|c|matches!(c,DrawCommand::Rect{rect:r,color,radius} if *r==rect(0.0,0.0,8.0,8.0)&&*color==Color::rgb(0,0,0)&&*radius==3.0)));
    assert!(result.commands.iter().any(|c|matches!(c,DrawCommand::Rect{rect:r,color,radius} if *r==rect(1.0,1.0,6.0,6.0)&&*color==Color::rgb(255,0,0)&&*radius==2.0)));
}
#[test]
fn canvas_uses_root_selected_origin_and_ignores_selected_clip() {
    let doc = Document::parse(
        "<style>html{box-sizing:border-box;width:12px;height:8px;border:2px solid transparent;padding:1px;background:lime linear-gradient(red,blue) 0 0/4px 2px no-repeat content-box}body{margin:0}</style>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 16.0);
    let result = layout(&doc, &styles, 32.0, 16.0, &Fonts::new());
    let images: Vec<_> = result.commands.iter().filter_map(image_geometry).collect();
    assert_eq!(
        images,
        vec![(rect(0.0, 0.0, 32.0, 16.0), rect(3.0, 3.0, 4.0, 2.0))]
    );
    assert!(result.commands.iter().any(|c|matches!(c,DrawCommand::Rect{rect:r,color,..} if *r==rect(0.0,0.0,32.0,16.0)&&*color==Color::rgb(0,255,0))));
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
        background_key_bytes_left: MAX_BACKGROUND_KEY_BYTES,
        viewport: Size {
            width: 64.0,
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

fn stretch_source(image: &str) -> String {
    format!(
        "<style>body{{margin:0}}#box{{box-sizing:border-box;width:8px;height:4px;border:1px solid transparent;padding:1px;background-image:{image};background-origin:content-box;background-clip:content-box;background-size:100% 100%;background-repeat:no-repeat}}</style><div id=box></div>"
    )
}
#[test]
fn stretching_revives_zero_content_gradient_and_recomputes_selected_clip() {
    let doc = Document::parse(&stretch_source("linear-gradient(red,blue)"));
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
    let id = doc.query_selector("#box").unwrap();
    let fonts = Fonts::new();
    let mut e = engine(&doc, &styles, &fonts, &[], &[]);
    let mut f = e.fragment(id, 8.0, 8.0, 0);
    assert_eq!(f.size.height, 4.0);
    let before: Vec<_> = f.commands.iter().filter_map(image_geometry).collect();
    assert_eq!(before, vec![(rect(2.0, 2.0, 4.0, 0.0), Rect::default())]);
    let mut work = MAX_POSITION_WORK;
    assert!(stretch_fragment_background_boxes(
        &mut f,
        10.0,
        &styles[id],
        8.0,
        None,
        &mut work
    ));
    let after: Vec<_> = f.commands.iter().filter_map(image_geometry).collect();
    assert_eq!(
        after,
        vec![(rect(2.0, 2.0, 4.0, 6.0), rect(2.0, 2.0, 4.0, 6.0))]
    );
    assert_eq!(f.hits[0].rect, rect(0.0, 0.0, 8.0, 10.0));
    let g = f.boxes.iter().find(|g| g.node == id).unwrap();
    assert_eq!(g.border, rect(0.0, 0.0, 8.0, 10.0));
    assert_eq!(g.padding, rect(1.0, 1.0, 6.0, 8.0));
    assert_eq!(g.content, rect(2.0, 2.0, 4.0, 6.0));
}
#[test]
fn stretching_loaded_url_reuses_the_once_paid_key() {
    let doc = Document::parse(&stretch_source("url(tile.png)"));
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
    let id = doc.query_selector("#box").unwrap();
    let fonts = Fonts::new();
    let mut loaded = vec![None; doc.nodes.len()];
    let natural = RasterNaturalSize {
        width: 4,
        height: 2,
    };
    loaded[id] = Some(LoadedBackground {
        key: "abc",
        natural,
    });
    let background_ranges = singleton_ranges(loaded.len());
    let mut e = engine(&doc, &styles, &fonts, &loaded, &background_ranges);
    let mut f = e.fragment(id, 8.0, 8.0, 0);
    assert_eq!(e.background_key_bytes_left, MAX_BACKGROUND_KEY_BYTES - 3);
    let key_ptr = f
        .commands
        .iter()
        .find_map(|c| {
            if let DrawCommand::BoxDecoration(d) = c {
                if let BoxPaint::Image { key, .. } = &d.paint {
                    Some(key.as_ptr())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    let mut work = MAX_POSITION_WORK;
    assert!(stretch_fragment_background_boxes(
        &mut f,
        10.0,
        &styles[id],
        8.0,
        Some(natural),
        &mut work
    ));
    let after: Vec<_> = f.commands.iter().filter_map(image_geometry).collect();
    assert_eq!(
        after,
        vec![(rect(2.0, 2.0, 4.0, 6.0), rect(2.0, 2.0, 4.0, 6.0))]
    );
    let after_key = f
        .commands
        .iter()
        .find_map(|c| {
            if let DrawCommand::BoxDecoration(d) = c {
                if let BoxPaint::Image { key, .. } = &d.paint {
                    Some(key)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(after_key, "abc");
    assert_eq!(after_key.as_ptr(), key_ptr);
    assert_eq!(e.background_key_bytes_left, MAX_BACKGROUND_KEY_BYTES - 3);
}
#[test]
fn retile_paint_refusal_preserves_completed_geometry() {
    let doc = Document::parse(&stretch_source("linear-gradient(red,blue)"));
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
    let id = doc.query_selector("#box").unwrap();
    let fonts = Fonts::new();
    for budget in [16, 17] {
        let mut e = engine(&doc, &styles, &fonts, &[], &[]);
        let mut f = e.fragment(id, 8.0, 8.0, 0);
        assert_eq!(f.boxes.len(), 1);
        let mut work = budget;
        assert_eq!(
            stretch_fragment_background_boxes(&mut f, 10.0, &styles[id], 8.0, None, &mut work),
            budget == 17
        );
        if budget == 16 {
            assert_eq!(work, 16);
            assert_eq!(f.size.height, 10.0);
            assert_eq!(f.hits[0].rect.height, 10.0);
            assert_eq!(f.boxes[0].border.height, 10.0);
            assert_eq!(
                f.commands
                    .iter()
                    .filter_map(image_geometry)
                    .collect::<Vec<_>>(),
                Vec::new()
            );
        } else {
            assert_eq!(work, 0);
            assert_eq!(f.size.height, 10.0);
        }
    }
}

#[test]
fn translated_fractional_table_retile_uses_forward_insets() {
    // Literal f32 values: x=.1, width=.3, left=.1, right=0. Recovering
    // right from translated endpoints gives -2^-25 despite a zero side.
    let doc = Document::parse(
        "<style>body{margin:0}#box{position:relative;left:.1px;box-sizing:border-box;width:.3px;height:2px;border-left:.1px solid transparent;background-image:linear-gradient(red,blue);background-origin:content-box;background-clip:content-box;background-size:100% 100%}</style><div id=box></div>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
    let id = doc.query_selector("#box").unwrap();
    let fonts = Fonts::new();
    let mut e = engine(&doc, &styles, &fonts, &[], &[]);
    let mut f = e.fragment(id, 0.3, 0.3, 0);
    let g = f.boxes.iter().find(|g| g.node == id).unwrap();
    assert_eq!(g.border, rect(0.1, 0.0, 0.3, 2.0));
    assert_eq!(g.padding.x.to_bits(), 0x3e4ccccd);
    assert_eq!(g.padding.width.to_bits(), 0x3e4cccce);
    assert_eq!(
        ((g.border.x + g.border.width) - (g.padding.x + g.padding.width)).to_bits(),
        0xb3000000
    );
    let mut work = 17;
    assert!(stretch_fragment_background_boxes(
        &mut f,
        4.0,
        &styles[id],
        0.3,
        None,
        &mut work
    ));
    assert_eq!(work, 0);
    let area = rect(
        f32::from_bits(0x3e4ccccd),
        0.0,
        f32::from_bits(0x3e4cccce),
        4.0,
    );
    assert_eq!(
        f.commands
            .iter()
            .filter_map(image_geometry)
            .collect::<Vec<_>>(),
        vec![(area, area)]
    );
    assert_eq!(f.hits[0].rect, rect(0.1, 0.0, 0.3, 4.0));
}

#[test]
fn table_rows_keep_coincident_boxes_for_every_origin_and_clip() {
    for origin in ["border-box", "padding-box", "content-box"] {
        for clip in ["border-box", "padding-box", "content-box"] {
            for raster in [false, true] {
                let image = if raster {
                    "url(tile.png)"
                } else {
                    "linear-gradient(red,blue)"
                };
                let doc = Document::parse(&format!(
                    "<style>body{{margin:0}}table{{width:12px}}td{{height:4px;padding:0;border:0;font-size:0}}tr{{background-image:{image};background-origin:{origin};background-clip:{clip};background-repeat:no-repeat}}</style><table><tr id=row><td></td></tr></table>"
                ));
                let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
                let id = doc.query_selector("#row").unwrap();
                let mut loaded = vec![None; doc.nodes.len()];
                loaded[id] = raster.then_some(LoadedBackground {
                    key: "row-tile",
                    natural: RasterNaturalSize {
                        width: 4,
                        height: 2,
                    },
                });
                let result =
                    layout_with_backgrounds(&doc, &styles, 32.0, 24.0, &Fonts::new(), &loaded);
                let area = rect(2.0, 2.0, 8.0, 4.0);
                let tile = if raster {
                    rect(2.0, 2.0, 4.0, 2.0)
                } else {
                    area
                };
                assert_eq!(
                    result
                        .commands
                        .iter()
                        .filter_map(image_geometry)
                        .collect::<Vec<_>>(),
                    vec![(area, tile)]
                );
            }
        }
    }
}

#[test]
fn oversized_table_available_width_survives_the_forced_border_clamp() {
    let doc = Document::parse(
        "<style>body{margin:0}#box{box-sizing:border-box;height:2px;padding-left:1%;padding-right:1%;background-image:linear-gradient(red,blue);background-origin:content-box;background-clip:content-box;background-size:100% 100%}</style><div id=box></div>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
    let id = doc.query_selector("#box").unwrap();
    let fonts = Fonts::new();
    let mut e = engine(&doc, &styles, &fonts, &[], &[]);
    let mut f = e.fragment(id, 2_000_000.0, 2_000_000.0, 0);
    assert_eq!(f.size.width, 1_000_000.0);
    assert_eq!(
        f.commands
            .iter()
            .filter_map(image_geometry)
            .collect::<Vec<_>>(),
        vec![(
            rect(20_000.0, 0.0, 960_000.0, 2.0),
            rect(20_000.0, 0.0, 960_000.0, 2.0)
        )]
    );
    let mut work = 17;
    assert!(stretch_fragment_background_boxes(
        &mut f,
        6.0,
        &styles[id],
        2_000_000.0,
        None,
        &mut work
    ));
    assert_eq!(work, 0);
    assert_eq!(
        f.commands
            .iter()
            .filter_map(image_geometry)
            .collect::<Vec<_>>(),
        vec![(
            rect(20_000.0, 0.0, 960_000.0, 6.0),
            rect(20_000.0, 0.0, 960_000.0, 6.0)
        )]
    );
}
#[test]
fn spanning_table_cell_retains_original_track_available_width() {
    // Two999999 tracks and fixed2 spacing give the spanning cell2000000.
    // Its border width clamps1000000; both1% sides still resolve to20000.
    let doc = Document::parse(
        "<style>body{margin:0}table{width:32px}td{height:2px;padding:0;border:0;font-size:0}.track{width:999999px}#box{padding-left:1%;padding-right:1%;background-image:linear-gradient(red,blue);background-origin:content-box;background-clip:content-box;background-size:100% 100%}</style><table><tr><td class=track></td><td class=track></td></tr><tr><td id=box colspan=2></td></tr></table>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 24.0);
    let result = layout(&doc, &styles, 32.0, 24.0, &Fonts::new());
    assert_eq!(
        result
            .commands
            .iter()
            .filter_map(image_geometry)
            .collect::<Vec<_>>(),
        vec![(
            rect(20_002.0, 6.0, 960_000.0, 2.0),
            rect(20_002.0, 6.0, 960_000.0, 2.0)
        )]
    );
}

#[test]
fn translucent_or_transparent_rounded_border_keeps_opaque_background_under_ring() {
    for (border, top) in [
        ("rgba(255,0,0,.5)", 0x80007f),
        ("rgba(255,0,0,0)", 0x0000ff),
    ] {
        let doc = Document::parse(&format!(
            "<style>html,body{{margin:0;padding:0;background:white}}#box{{box-sizing:border-box;width:12px;height:12px;border:2px solid {border};border-radius:4px;background:blue}}</style><div id=box></div>"
        ));
        let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 12.0, 12.0);
        let fonts = Fonts::new();
        let result = layout(&doc, &styles, 12.0, 12.0, &fonts);
        let background = result.commands.iter().find_map(|command| match command {
            DrawCommand::BoxDecoration(decoration)
                if decoration.cutout.is_none()
                    && matches!(&decoration.paint, BoxPaint::Color(color) if *color == Color::rgb(0, 0, 255)) => Some(decoration),
            _ => None,
        }).expect("opaque background covers its outer border-box, including beneath transparent border pixels");
        assert_eq!(background.shape, shape(rect(0.0, 0.0, 12.0, 12.0), 4.0));
        if top == 0x80007f {
            let ring = result
                .commands
                .iter()
                .find_map(|command| match command {
                    DrawCommand::BoxDecoration(decoration) if decoration.cutout.is_some() => {
                        Some(decoration)
                    }
                    _ => None,
                })
                .unwrap();
            assert_eq!(ring.shape, shape(rect(0.0, 0.0, 12.0, 12.0), 4.0));
            assert_eq!(ring.cutout, Some(shape(rect(2.0, 2.0, 8.0, 8.0), 2.0)));
        }
        let mut canvas = crate::graphics::Canvas::new(12, 12).unwrap();
        canvas.paint(
            &result.commands,
            &fonts,
            &crate::graphics::ImageStore::new(),
            0.0,
            0.0,
        );
        assert!(!canvas.exhausted());
        assert_eq!(canvas.pixels[5], top, "straight top edge: {border}");
        assert_eq!(
            canvas.pixels[5 * 12 + 5],
            0x0000ff,
            "uncut interior: {border}"
        );
        assert_eq!(canvas.pixels[0], 0xffffff, "outside corner: {border}");
    }
}

// Append to layout/background_boxes/tests.rs; uses its existing engine helper.
#[test]
fn principal_extraction_preserves_raw_origin_until_each_original_rect_call() {
    let doc = Document::parse(
        "<style>#box{box-sizing:border-box;border-left:2px solid transparent;background-image:linear-gradient(red,blue)}</style><div id=box></div>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 10.0, 4.0);
    let id = doc.query_selector("#box").unwrap();
    assert_eq!(styles[id].backgrounds.origins()[0], BackgroundBox::Padding);
    assert_eq!(styles[id].backgrounds.clips()[0], BackgroundBox::Border);
    let fonts = Fonts::new();
    let mut e = engine(&doc, &styles, &fonts, &[], &[]);
    let size = e.layout_box_sized(id, -1_000_001.0, 0.0, 10.0, (Some(10.0), Some(4.0)), 0);
    assert_eq!((size.size.width, size.size.height), (10.0, 4.0));
    let gradients: Vec<_> = e
        .commands
        .iter()
        .filter_map(|command| {
            if let DrawCommand::LinearGradient {
                rect,
                tile,
                repeat,
                radius,
                stop_scale,
                ..
            } = command
            {
                assert_eq!(*repeat, [true, true]);
                assert_eq!(*radius, 0.0);
                assert_eq!(*stop_scale, 1.0);
                Some((*rect, *tile))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        gradients,
        vec![(
            Rect {
                x: -1_000_000.0,
                y: 0.0,
                width: 10.0,
                height: 4.0
            },
            Rect {
                x: -999_999.0,
                y: 0.0,
                width: 8.0,
                height: 4.0
            },
        )]
    );
    assert!(
        !e.commands
            .iter()
            .any(|command| matches!(command, DrawCommand::BoxDecoration(_)))
    );
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
