use super::*;

fn prepared(widths: &[f32], edges: bool) -> (Vec<InlineItem>, InlineEdges) {
    let mut collection = InlineCollection::default();
    collection.spans.push(InlineSpan {
        node: 0,
        parent: None,
        basis: 40.0,
        border: if edges {
            Sides {
                left: 1.0,
                right: 2.0,
                ..Sides::default()
            }
        } else {
            Sides::default()
        },
        padding: if edges {
            Sides {
                left: 2.0,
                right: 1.0,
                ..Sides::default()
            }
        } else {
            Sides::default()
        },
        margin: if edges { (-2.0, 5.0) } else { (0.0, 0.0) },
        first: None,
        last: None,
        font_size: 20.0,
    });
    collection.offsets.push((7.0, -3.0));
    for &width in widths {
        collection.items.push(InlineItem {
            node: 0,
            width,
            height: 26.0,
            preserve: false,
            kind: InlineKind::Box(
                Fragment {
                    size: Size {
                        width,
                        height: 26.0,
                    },
                    commands: Vec::new(),
                    owners: Vec::new(),
                    hits: Vec::new(),
                    boxes: Vec::new(),
                    positioned: Vec::new(),
                    rounded_border: false,
                    gradient: false,
                },
                Sides::default(),
            ),
        });
    }
    let mut work = MAX_POSITION_WORK;
    collection.mark(Some(0), &mut work);
    collection.finish(&mut work).unwrap()
}

#[test]
fn committed_border_phase_ignores_line_positions_margins_and_visual_offsets() {
    let (items, mut edges) = prepared(&[4.0, 4.0, 4.0, 4.0], true);
    let mut work = MAX_POSITION_WORK;
    for (range, indices, x, top, prefix, area, positions, sides) in [
        (
            0..1,
            vec![0],
            10.0,
            0.0,
            0.0,
            rect(8.0, 7.0, 7.0, 26.0),
            vec![11.0],
            inline_backgrounds::FIRST,
        ),
        (
            1..3,
            vec![1, 2],
            80.0,
            40.0,
            7.0,
            rect(80.0, 47.0, 8.0, 26.0),
            vec![80.0, 84.0],
            0,
        ),
        (
            3..4,
            vec![3],
            -5.0,
            80.0,
            15.0,
            rect(-5.0, 87.0, 7.0, 26.0),
            vec![-5.0],
            inline_backgrounds::LAST,
        ),
    ] {
        let plan = edges
            .line_plan(
                &items[range],
                &indices,
                x,
                26.0,
                (top, 26.0),
                0.0,
                &mut work,
            )
            .unwrap();
        assert_eq!(plan.positions, positions);
        assert_eq!(plan.boxes.len(), 1);
        let (geometry, offset) = plan.boxes[0];
        assert_eq!(geometry.border, area);
        assert_eq!(geometry.inline_prefix, prefix);
        assert_eq!(geometry.inline_basis, 40.0);
        assert_eq!(geometry.inline_sides, sides);
        assert_eq!(offset, (7.0, -3.0));
        edges.commit_line(&plan);
    }
    // Retain the cumulative sum in f64. Rounding each stored f32 prefix back
    // into the accumulator would lose these two1/32 fragments at524288.
    let (items, mut edges) = prepared(&[524_288.0, 0.03125, 0.03125, 4.0], false);
    for (index, prefix) in [0.0, 524_288.0, 524_288.0, f32::from_bits(0x4900_0001)]
        .into_iter()
        .enumerate()
    {
        let plan = edges
            .line_plan(
                &items[index..index + 1],
                &[index],
                0.0,
                26.0,
                (0.0, 26.0),
                0.0,
                &mut work,
            )
            .unwrap();
        assert_eq!(plan.boxes[0].0.inline_prefix, prefix);
        edges.commit_line(&plan);
    }
}

#[test]
fn uncommitted_or_refused_line_never_advances_later_border_phase() {
    let (items, mut edges) = prepared(&[4.0, 4.0, 4.0, 4.0], true);
    let mut work = 11;
    let first = edges
        .line_plan(&items[..1], &[0], 0.0, 26.0, (0.0, 26.0), 0.0, &mut work)
        .unwrap();
    assert_eq!(work, 0);
    assert_eq!(first.boxes[0].0.inline_prefix, 0.0);
    assert_eq!(first.boxes[0].0.border.width, 7.0);
    edges.commit_line(&first);
    // A successful plan is still private until the caller has reserved its
    // publication destination. Dropping it models that caller-side refusal.
    let mut work = 15;
    let uncommitted = edges
        .line_plan(
            &items[1..3],
            &[1, 2],
            0.0,
            26.0,
            (26.0, 26.0),
            0.0,
            &mut work,
        )
        .unwrap();
    assert_eq!(work, 0);
    assert_eq!(uncommitted.boxes[0].0.inline_prefix, 7.0);
    drop(uncommitted);
    let mut work = 14;
    assert!(
        edges
            .line_plan(
                &items[1..3],
                &[1, 2],
                0.0,
                26.0,
                (26.0, 26.0),
                0.0,
                &mut work
            )
            .is_none()
    );
    assert_eq!(work, 8);
    // Full tariff was admitted, but a later cursor check refuses this line.
    let mut work = 15;
    assert!(
        edges
            .line_plan(
                &items[1..3],
                &[1, 2],
                MAX_EXTENT - 1.0,
                26.0,
                (26.0, 26.0),
                0.0,
                &mut work
            )
            .is_none()
    );
    assert_eq!(work, 0);
    let mut work = 15;
    let middle = edges
        .line_plan(
            &items[1..3],
            &[1, 2],
            0.0,
            26.0,
            (26.0, 26.0),
            0.0,
            &mut work,
        )
        .unwrap();
    assert_eq!(work, 0);
    assert_eq!(middle.boxes[0].0.inline_prefix, 7.0);
    assert_eq!(middle.boxes[0].0.border.width, 8.0);
    edges.commit_line(&middle);
    let mut work = 11;
    let last = edges
        .line_plan(&items[3..], &[3], 0.0, 26.0, (52.0, 26.0), 0.0, &mut work)
        .unwrap();
    assert_eq!(work, 0);
    assert_eq!(last.boxes[0].0.inline_prefix, 15.0);
    assert_eq!(last.boxes[0].0.border.width, 7.0);
    edges.commit_line(&last);
}

fn engine<'a>(doc: &'a Document, styles: &'a [ComputedStyle], fonts: &'a Fonts) -> Engine<'a> {
    Engine {
        doc,
        styles,
        fonts,
        backgrounds: &[],
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
fn refused_decoration_slots_preserve_completed_fragment_phase() {
    let doc = Document::parse(
        "<style>body{margin:0}#owner{font-size:20px;line-height:26px;border-left:1px solid transparent;border-right:2px solid transparent;padding-left:2px;padding-right:1px;background-image:linear-gradient(red,blue)}i{display:inline-block;width:4px;height:26px;vertical-align:top}</style><div id=row><span id=owner><i></i><br><i></i><br><i></i></span></div>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 64.0, 100.0);
    let fonts = Fonts::new();
    let row = doc.query_selector("#row").unwrap();
    let owner = doc.query_selector("#owner").unwrap();
    for full in [false, true] {
        let mut e = engine(&doc, &styles, &fonts);
        // Collection has its own earlier8-command guard. Prepare the real
        // fragments first so this witness reaches decoration-slot admission.
        let mut collection = InlineCollection::default();
        for &node in &doc.nodes[row].children {
            e.collect_inline(node, 64.0, 0, &mut collection, None);
        }
        let (tokens, mut edges) = collection.finish(&mut e.position_work_left).unwrap();
        assert_eq!(tokens.len(), 5);
        if full {
            e.commands_created = MAX_COMMANDS;
        }
        let mut cursor = 0.0;
        let mut painted = Vec::new();
        for (index, token) in tokens.into_iter().enumerate() {
            if matches!(token.kind, InlineKind::Break) {
                continue;
            }
            assert!(matches!(token.kind, InlineKind::Box(..)));
            let height = e.paint_line(
                vec![token],
                vec![index],
                &mut edges,
                64.0,
                0.0,
                cursor,
                64.0,
                "left",
                false,
            );
            assert_eq!(height, Some(26.0));
            cursor += height.unwrap();
            painted.push(index);
        }
        assert_eq!(painted, vec![0, 2, 4]);
        assert_eq!(cursor, 78.0);
        let geometries: Vec<_> = e
            .boxes
            .iter()
            .filter(|g| g.node == owner && g.inline)
            .collect();
        assert_eq!(geometries.len(), 3);
        for (g, (area, prefix, sides)) in geometries.into_iter().zip([
            (rect(0.0, 7.0, 7.0, 26.0), 0.0, inline_backgrounds::FIRST),
            (rect(0.0, 33.0, 4.0, 26.0), 7.0, 0),
            (rect(0.0, 59.0, 7.0, 26.0), 11.0, inline_backgrounds::LAST),
        ]) {
            assert_eq!(g.border, area);
            assert_eq!(g.inline_prefix, prefix);
            assert_eq!(g.inline_sides, sides);
            assert_eq!(g.inline_basis, 64.0);
        }
        let pending = e
            .owners
            .iter()
            .filter(|o| o.node == owner && o.inline_layer != InlineLayer::None)
            .count();
        assert_eq!(pending, if full { 0 } else { 12 });
    }
}

#[test]
fn float_narrowing_retains_full_percentage_basis_in_final_background_geometry() {
    for float in [false, true] {
        let float_html = if float {
            "<b style='float:left;width:16px;height:26px'></b>"
        } else {
            ""
        };
        let doc = Document::parse(&format!(
            "<style>body{{margin:0}}#row{{width:40px}}#owner{{font-size:20px;line-height:26px;padding-left:10%;padding-right:10%;background-image:linear-gradient(red 4px,blue 4px);background-origin:padding-box;background-clip:content-box;background-size:100% 100%}}i{{display:inline-block;width:4px;height:26px;vertical-align:top}}</style><main id=row>{float_html}<span id=owner><i></i></span></main>"
        ));
        let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 64.0, 100.0);
        let fonts = Fonts::new();
        let row = doc.query_selector("#row").unwrap();
        let owner = doc.query_selector("#owner").unwrap();
        let mut e = engine(&doc, &styles, &fonts);
        e.layout_box(row, 0.0, 0.0, 40.0, Some(40.0), 0);
        let x = if float { 16.0 } else { 0.0 };
        let geometries: Vec<_> = e
            .boxes
            .iter()
            .filter(|g| g.node == owner && g.inline)
            .collect();
        assert_eq!(geometries.len(), 1);
        assert_eq!(geometries[0].inline_basis, 40.0);
        assert_eq!(geometries[0].border, rect(x, 7.0, 12.0, 26.0));
        assert_eq!(geometries[0].content, rect(x + 4.0, 7.0, 4.0, 26.0));
        e.finish_inline_backgrounds();
        let images: Vec<_> = e
            .commands
            .iter()
            .zip(&e.owners)
            .filter_map(|(command, paint_owner)| {
                if paint_owner.node != owner {
                    return None;
                }
                if let DrawCommand::BoxDecoration(d) = command
                    && let crate::graphics::box_decoration::BoxPaint::LinearGradient {
                        tile, ..
                    } = &d.paint
                {
                    return Some((d.shape.rect, *tile));
                }
                None
            })
            .collect();
        assert_eq!(
            images,
            vec![(rect(x + 4.0, 7.0, 4.0, 26.0), rect(x, 7.0, 12.0, 26.0))]
        );
    }
}
