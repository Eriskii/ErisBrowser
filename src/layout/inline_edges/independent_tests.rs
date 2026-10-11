use super::*;

fn engine<'a>(doc: &'a Document, styles: &'a [ComputedStyle], fonts: &'a Fonts) -> Engine<'a> {
    Engine {
        doc,
        styles,
        fonts,
        backgrounds: BackgroundBindings::default(),
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
fn actual_inline_pipeline_retains_owner_flow_boxes_before_child_relative_motion() {
    for (owner_position, dx, dy) in [
        ("", 0.0, 0.0),
        ("position:relative;left:3px;top:6px", 3.0, 6.0),
    ] {
        let doc = Document::parse(&format!(
            "<style>body{{margin:0}}#owner{{font-size:20px;line-height:40px;padding-left:1px;padding-right:2px}}i{{display:inline-block;width:4px;height:4px;vertical-align:top}}</style><div id=row><span id=owner style='{owner_position}'><i id=a></i><i id=b style='position:relative;left:5px'></i></span><i id=c></i></div>"
        ));
        let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 64.0, 100.0);
        let fonts = Fonts::new();
        let mut e = engine(&doc, &styles, &fonts);
        let owner = doc.query_selector("#owner").unwrap();
        let row = doc.query_selector("#row").unwrap();
        assert_eq!(
            e.layout_inline(&doc.nodes[row].children, 0.0, 0.0, 64.0, "left", 0),
            4.0
        );
        let boxes: Vec<_> = e.boxes.iter().filter(|g| g.node == owner).collect();
        assert_eq!(boxes.len(), 1);
        let g = boxes[0];
        // Atomic baseline4 minus owner's nominal ascent19; no line-leading.
        assert_eq!(g.border, rect(dx, -15.0 + dy, 11.0, 26.0));
        assert_eq!(g.padding, rect(dx, -15.0 + dy, 11.0, 26.0));
        assert_eq!(g.content, rect(1.0 + dx, -15.0 + dy, 8.0, 26.0));
        assert!(g.inline && !g.fixed);
        for (selector, x, y) in [
            ("#a", 1.0 + dx, dy),
            ("#b", 10.0 + dx, dy),
            ("#c", 11.0, 0.0),
        ] {
            let node = doc.query_selector(selector).unwrap();
            let h = e.hits.iter().find(|h| h.node == node).unwrap();
            assert_eq!(h.rect, rect(x, y, 4.0, 4.0));
        }
    }
}

fn atomic(node: NodeId, color: Color) -> InlineItem {
    InlineItem {
        node,
        width: 4.0,
        height: 4.0,
        preserve: false,
        kind: InlineKind::Box(
            Fragment {
                size: Size {
                    width: 4.0,
                    height: 4.0,
                },
                commands: vec![DrawCommand::Rect {
                    rect: rect(0.0, 0.0, 4.0, 4.0),
                    color,
                    radius: 0.0,
                }],
                owners: vec![PaintOwner {
                    node,
                    background: true,
                    inline_layer: InlineLayer::None,
                    inline_prefix: 0.0,
                    inline_sides: 0,
                    background_image_index: 0,
                }],
                hits: vec![HitRegion {
                    node,
                    rect: rect(0.0, 0.0, 4.0, 4.0),
                    fixed: false,
                    action: HitAction::Node,
                }],
                boxes: Vec::new(),
                positioned: Vec::new(),
                paint_slots: PaintSlots::new(false, false),
                principal_box: 0,
            },
            Sides::default(),
        ),
    }
}

#[test]
fn actual_paint_line_exact_and_one_short_preserve_published_state_and_caller_cursor() {
    let doc = Document::parse("<div><span id=owner><i id=a></i><i id=b></i></span></div>");
    let styles = crate::css::compute_styles(&doc, &[], 64.0, 100.0);
    let fonts = Fonts::new();
    let owner = doc.query_selector("#owner").unwrap();
    let a = doc.query_selector("#a").unwrap();
    let b = doc.query_selector("#b").unwrap();
    for budget in [0, 14, 15] {
        let mut collection = InlineCollection::default();
        collection.spans.push(InlineSpan {
            node: owner,
            parent: None,
            basis: 0.0,
            border: Sides::default(),
            padding: Sides {
                left: 1.0,
                right: 2.0,
                ..Sides::default()
            },
            margin: (0.0, 0.0),
            first: None,
            last: None,
            font_size: 20.0,
        });
        collection.offsets.push((0.0, 0.0));
        collection.items.push(atomic(a, Color::rgb(255, 0, 0)));
        collection.items.push(atomic(b, Color::rgb(0, 255, 0)));
        let mut setup = MAX_POSITION_WORK;
        collection.mark(Some(0), &mut setup);
        let (line, mut edges) = collection.finish(&mut setup).unwrap();
        let mut e = engine(&doc, &styles, &fonts);
        let sentinel = rect(50.0, 60.0, 2.0, 3.0);
        e.commands.push(DrawCommand::Rect {
            rect: sentinel,
            color: Color::rgb(0, 0, 0),
            radius: 0.0,
        });
        e.owners.push(e.paint_owner);
        e.boxes.push(BoxGeometry {
            node: doc.root,
            border: sentinel,
            padding: sentinel,
            content: sentinel,
            inline: false,
            inline_prefix: 0.0,
            inline_basis: 0.0,
            inline_sides: 0,
            fixed: false,
        });
        e.hits.push(HitRegion {
            node: doc.root,
            rect: sentinel,
            fixed: false,
            action: HitAction::Node,
        });
        // Atomic fragments were already admitted during collection. The only
        // new line tariff is 2 scan + 2*2 membership + 1*2*(1+2) order + 2 positions + 1 phase commit =15.
        e.commands_created = 3;
        e.hits_created = 3;
        e.position_work_left = budget;
        let mut cursor = 5.0;
        let result = e.paint_line(
            line,
            vec![0, 1],
            &mut edges,
            64.0,
            10.0,
            cursor,
            32.0,
            "left",
            false,
        );
        assert_eq!(result, if budget < 15 { None } else { Some(4.0) });
        cursor += result.unwrap_or(0.0);
        assert!(matches!(&e.commands[0], DrawCommand::Rect { rect, .. } if *rect == sentinel));
        assert_eq!(e.boxes[0].border, sentinel);
        assert_eq!(e.boxes[0].padding, sentinel);
        assert_eq!(e.boxes[0].content, sentinel);
        assert_eq!(e.hits[0].rect, sentinel);
        assert_eq!(e.paint_owner.node, doc.root);
        assert!(!e.paint_owner.background && e.paint_owner.inline_layer == InlineLayer::None);
        assert_eq!(e.commands_created, 3);
        assert_eq!(e.hits_created, 3);
        if budget < 15 {
            assert_eq!(cursor, 5.0);
            assert_eq!(
                (
                    e.commands.len(),
                    e.owners.len(),
                    e.boxes.len(),
                    e.hits.len()
                ),
                (1, 1, 1, 1)
            );
            assert!(e.positioned.is_empty());
            assert_eq!(e.position_work_left, if budget == 0 { 0 } else { 8 });
        } else {
            assert_eq!(cursor, 9.0);
            assert_eq!(e.position_work_left, 0);
            assert_eq!(
                (
                    e.commands.len(),
                    e.owners.len(),
                    e.boxes.len(),
                    e.hits.len()
                ),
                (3, 3, 2, 3)
            );
            assert_eq!(e.boxes[1].border, rect(10.0, -10.0, 11.0, 26.0));
            assert_eq!(e.boxes[1].padding, rect(10.0, -10.0, 11.0, 26.0));
            assert_eq!(e.boxes[1].content, rect(11.0, -10.0, 8.0, 26.0));
            for (i, x) in [(1, 11.0), (2, 15.0)] {
                assert_eq!(e.hits[i].rect, rect(x, 5.0, 4.0, 4.0));
                assert!(
                    matches!(&e.commands[i], DrawCommand::Rect { rect: area, .. } if *area == rect(x, 5.0, 4.0, 4.0))
                );
            }
        }
    }
}

#[test]
fn actual_forced_break_caller_does_not_advance_a_refused_nonempty_line() {
    let doc = Document::parse(
        "<style>body{margin:0}#owner{font-size:1px;line-height:4px;padding-left:1px;padding-right:1px}i{display:inline-block;width:4px;height:4px;vertical-align:top}</style><div id=row><span id=owner><i id=a></i><br></span></div>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 64.0, 100.0);
    let fonts = Fonts::new();
    let row = doc.query_selector("#row").unwrap();
    let owner = doc.query_selector("#owner").unwrap();
    let a = doc.query_selector("#a").unwrap();
    let (mut refusals, mut successes) = (0, 0);
    // Fixed bounded cuts exercise collection, line planning and the actual br
    // caller. No runtime-measured fee is used to select or adjust a cut.
    for budget in (0..=256).chain(std::iter::once(MAX_POSITION_WORK)) {
        let mut e = engine(&doc, &styles, &fonts);
        e.position_work_left = budget;
        let height = e.layout_inline(&doc.nodes[row].children, 0.0, 0.0, 64.0, "left", 0);
        if e.boxes.iter().any(|g| g.node == owner) {
            successes += 1;
            assert_eq!(
                height, 8.0,
                "successful original forced-break height, budget {budget}"
            );
            assert_eq!(
                e.hits.iter().find(|h| h.node == a).unwrap().rect,
                rect(1.0, 0.0, 4.0, 4.0)
            );
        } else {
            refusals += 1;
            assert_eq!(
                height, 0.0,
                "refusal must not become an empty successful br, budget {budget}"
            );
            assert!(
                e.commands.is_empty()
                    && e.owners.is_empty()
                    && e.hits.is_empty()
                    && e.boxes.is_empty()
            );
            assert!(e.positioned.is_empty());
        }
    }
    assert!(refusals > 0 && successes > 0);
}
