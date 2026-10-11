use super::*;

fn command(x: f32, y: f32) -> DrawCommand {
    DrawCommand::BackgroundImage {
        rect: Rect {
            x: -1.5,
            y: 2.25,
            width: 8.0,
            height: 6.0,
        },
        tile: Rect {
            x,
            y,
            width: 4.0,
            height: 2.0,
        },
        repeat: [false, true],
        radius: 1.25,
        key: "literal".into(),
    }
}

#[test]
fn tiled_translation_preserves_phase_and_omits_unrepresentable_tile_origins() {
    let mut ordinary = command(-3.0, 0.5);
    translate(&mut ordinary, 1.25, -2.5);
    let DrawCommand::BackgroundImage {
        rect,
        tile,
        repeat,
        radius,
        key,
    } = ordinary
    else {
        panic!()
    };
    assert_eq!(
        rect,
        Rect {
            x: -0.25,
            y: -0.25,
            width: 8.0,
            height: 6.0
        }
    );
    assert_eq!(
        tile,
        Rect {
            x: -1.75,
            y: -2.0,
            width: 4.0,
            height: 2.0
        }
    );
    assert_eq!(
        (repeat, radius, key.as_str()),
        ([false, true], 1.25, "literal")
    );
    for (x, y, dx, dy, expected) in [
        (
            999998.0,
            -999998.0,
            2.0,
            -2.0,
            Some((1000000.0, -1000000.0)),
        ),
        (1000000.0, 0.0, 1.0, 0.0, None),
        (0.0, -1000000.0, 0.0, -1.0, None),
        (f32::INFINITY, 0.0, 0.0, 0.0, None),
        (0.0, f32::NAN, 0.0, 0.0, None),
    ] {
        let mut value = command(x, y);
        translate(&mut value, dx, dy);
        let DrawCommand::BackgroundImage {
            tile,
            key,
            repeat,
            radius,
            ..
        } = value
        else {
            panic!()
        };
        if let Some((x, y)) = expected {
            assert_eq!(
                tile,
                Rect {
                    x,
                    y,
                    width: 4.0,
                    height: 2.0
                }
            );
        } else {
            assert_eq!(tile, Rect::default());
        }
        assert_eq!(
            (key.as_str(), repeat, radius),
            ("literal", [false, true], 1.25)
        );
    }
}

#[test]
fn tiled_fragment_translation_leaves_fixed_scope_untouched_and_restores_document_space() {
    let mut fragment = Fragment {
        size: Size {
            width: 8.0,
            height: 6.0,
        },
        commands: vec![
            command(-3.0, 0.5),
            DrawCommand::PushFixed,
            command(-3.0, 0.5),
            DrawCommand::PopFixed,
            command(-3.0, 0.5),
        ],
        owners: Vec::new(),
        hits: Vec::new(),
        boxes: Vec::new(),
        positioned: Vec::new(),
        rounded_border: false,
        gradient: false,
    };
    translate_fragment(&mut fragment, 3.0, 4.0);
    for (index, expected_rect, expected_tile) in [
        (0, (1.5, 6.25), (0.0, 4.5)),
        (2, (-1.5, 2.25), (-3.0, 0.5)),
        (4, (1.5, 6.25), (0.0, 4.5)),
    ] {
        let DrawCommand::BackgroundImage {
            rect,
            tile,
            repeat,
            radius,
            key,
        } = &fragment.commands[index]
        else {
            panic!()
        };
        assert_eq!((rect.x, rect.y), expected_rect);
        assert_eq!((tile.x, tile.y), expected_tile);
        assert_eq!(
            (rect.width, rect.height, tile.width, tile.height, *radius),
            (8.0, 6.0, 4.0, 2.0, 1.25)
        );
        assert_eq!((*repeat, key.as_str()), ([false, true], "literal"));
    }
    assert!(matches!(fragment.commands[1], DrawCommand::PushFixed));
    assert!(matches!(fragment.commands[3], DrawCommand::PopFixed));
}

#[test]
fn tiled_restack_keeps_only_positive_paint_and_tile_boxes() {
    let doc = Document::parse("<div></div>");
    let styles = vec![ComputedStyle::default(); doc.nodes.len()];
    let fonts = Fonts::new();
    let mut engine = Engine {
        doc: &doc,
        styles: &styles,
        fonts: &fonts,
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
            background: true,
            inline_raster: false,
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
    };
    assert!(engine.push(command(-3.0, 0.5)));
    for field in 0..4 {
        let mut value = command(-3.0, 0.5);
        let DrawCommand::BackgroundImage { rect, tile, .. } = &mut value else {
            panic!()
        };
        match field {
            0 => rect.width = 0.0,
            1 => rect.height = 0.0,
            2 => tile.width = 0.0,
            3 => tile.height = 0.0,
            _ => unreachable!(),
        }
        assert!(engine.push(value));
    }
    assert!(engine.push(DrawCommand::Rect {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 2.0
        },
        color: Color::WHITE,
        radius: 0.0,
    }));
    engine.restack();
    assert_eq!(engine.commands.len(), 2);
    assert!(
        matches!(&engine.commands[0], DrawCommand::BackgroundImage { key, .. } if key == "literal")
    );
    assert!(
        matches!(&engine.commands[1], DrawCommand::Rect { color, .. } if *color == Color::WHITE)
    );
}
