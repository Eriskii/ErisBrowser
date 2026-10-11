//! Independent pending-decoration translation regression, frozen before its fix.
//! Intended child module layout::inline_backgrounds::translation_tests.
use super::*;

fn owner(layer: InlineLayer) -> PaintOwner {
    PaintOwner {
        node: 1,
        background: true,
        inline_layer: layer,
        inline_prefix: 8.0,
        inline_sides: LAST,
    }
}
fn pending(area: Rect, fixed: bool) -> Fragment {
    let first = Rect { width: 8.0, ..area };
    let geometry = |border, prefix, sides| BoxGeometry {
        node: 1,
        border,
        padding: border,
        content: border,
        fixed,
        inline: true,
        inline_prefix: prefix,
        inline_basis: 40.0,
        inline_sides: sides,
    };
    Fragment {
        size: Size {
            width: 12.0,
            height: 6.0,
        },
        commands: vec![
            DrawCommand::PushClip { rect: area },
            DrawCommand::Rect {
                rect: area,
                color: Color::TRANSPARENT,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: area,
                color: Color::TRANSPARENT,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: area,
                color: Color::TRANSPARENT,
                radius: 0.0,
            },
            DrawCommand::PopClip,
        ],
        owners: [
            InlineLayer::Clip,
            InlineLayer::Color,
            InlineLayer::Image,
            InlineLayer::Border,
            InlineLayer::None,
        ]
        .map(owner)
        .to_vec(),
        hits: vec![HitRegion {
            node: 1,
            action: HitAction::Node,
            rect: area,
            fixed,
        }],
        boxes: vec![geometry(first, 0.0, FIRST), geometry(area, 8.0, LAST)],
        positioned: Vec::new(),
        rounded_border: false,
        gradient: false,
    }
}
fn command_rect(command: &DrawCommand) -> Rect {
    match command {
        DrawCommand::Rect { rect, .. } | DrawCommand::PushClip { rect } => *rect,
        _ => panic!("expected an explicit pending rectangle or clip"),
    }
}
fn invariant_geometry(fragment: &Fragment) {
    assert_eq!(fragment.boxes.len(), 2);
    for (g, width, prefix, sides) in [
        (&fragment.boxes[0], 8.0, 0.0, FIRST),
        (&fragment.boxes[1], 4.0, 8.0, LAST),
    ] {
        assert_eq!(
            (
                g.node,
                g.inline,
                g.inline_basis,
                g.inline_prefix,
                g.inline_sides
            ),
            (1, true, 40.0, prefix, sides)
        );
        for area in [g.border, g.padding, g.content] {
            assert_eq!((area.width, area.height), (width, 6.0));
        }
    }
    assert_eq!((fragment.size.width, fragment.size.height), (12.0, 6.0));
    assert_eq!(fragment.hits.len(), 1);
    assert_eq!(
        (fragment.hits[0].node, fragment.hits[0].action),
        (1, HitAction::Node)
    );
    assert_eq!(
        (fragment.hits[0].rect.width, fragment.hits[0].rect.height),
        (4.0, 6.0)
    );
    for value in &fragment.owners {
        assert_eq!(
            (
                value.node,
                value.background,
                value.inline_prefix,
                value.inline_sides
            ),
            (1, true, 8.0, LAST)
        );
    }
}
fn assert_refused(fragment: &Fragment, start: usize) {
    assert!(
        matches!(fragment.commands[start], DrawCommand::PushClip { rect } if rect == Rect::default())
    );
    for index in start + 1..start + 4 {
        assert!(
            matches!(fragment.commands[index], DrawCommand::Rect { rect, color, .. }
            if rect == Rect::default() && color.a == 0)
        );
    }
    assert!(matches!(fragment.commands[start + 4], DrawCommand::PopClip));
    assert!(
        fragment.owners[start..start + 4]
            .iter()
            .all(|v| v.inline_layer == InlineLayer::None)
    );
}

#[test]
fn pending_fragment_exact_signed_origin_limit_preserves_common_phase() {
    // Literal origin-only bounds agree with completed BoxDecoration placement;
    // endpoints stay within the separate16M shape bound.
    for (x, y, dx, dy, expected_x, expected_y) in [
        (999_999.0, 2.0, 1.0, 0.0, 1_000_000.0, 2.0),
        (-999_999.0, 2.0, -1.0, 0.0, -1_000_000.0, 2.0),
        (2.0, 999_999.0, 0.0, 1.0, 2.0, 1_000_000.0),
        (2.0, -999_999.0, 0.0, -1.0, 2.0, -1_000_000.0),
    ] {
        let mut fragment = pending(
            Rect {
                x,
                y,
                width: 4.0,
                height: 6.0,
            },
            false,
        );
        translate_fragment(&mut fragment, dx, dy);
        let expected = Rect {
            x: expected_x,
            y: expected_y,
            width: 4.0,
            height: 6.0,
        };
        for command in &fragment.commands[..4] {
            assert_eq!(command_rect(command), expected);
        }
        assert!(matches!(fragment.commands[4], DrawCommand::PopClip));
        for (value, layer) in fragment.owners.iter().zip([
            InlineLayer::Clip,
            InlineLayer::Color,
            InlineLayer::Image,
            InlineLayer::Border,
            InlineLayer::None,
        ]) {
            assert!(value.inline_layer == layer);
        }
        invariant_geometry(&fragment);
        let mut metrics = Metrics::default();
        for &g in &fragment.boxes {
            metrics.add(g, Sides::default(), Sides::default());
        }
        let full = metrics.boxes(expected, 8.0, 0.0).unwrap();
        assert_eq!(
            full.border.rect,
            Rect {
                x: expected_x - 8.0,
                y: expected_y,
                width: 12.0,
                height: 6.0
            }
        );
        assert_eq!(full.padding.rect, full.border.rect);
        assert_eq!(full.content.rect, full.border.rect);
    }
}

#[test]
fn one_past_pending_translation_omits_balanced_paint_without_erasing_strip_geometry() {
    for (x, y, dx, dy) in [
        (999_999.0, 2.0, 2.0, 0.0),
        (-999_999.0, 2.0, -2.0, 0.0),
        (2.0, 999_999.0, 0.0, 2.0),
        (2.0, -999_999.0, 0.0, -2.0),
        (0.0, 2.0, f32::INFINITY, 0.0),
        (0.0, 2.0, f32::NAN, 0.0),
        (1_000_001.0, 2.0, 0.0, 0.0),
    ] {
        let mut fragment = pending(
            Rect {
                x,
                y,
                width: 4.0,
                height: 6.0,
            },
            false,
        );
        translate_fragment(&mut fragment, dx, dy);
        assert_refused(&fragment, 0);
        invariant_geometry(&fragment);
        // All completed widths and physical endpoints remain available to a
        // different, admitted fragment; rejected paint never shortens its strip.
        let mut metrics = Metrics::default();
        for &g in &fragment.boxes {
            metrics.add(g, Sides::default(), Sides::default());
        }
        let full = metrics
            .boxes(
                Rect {
                    x: 20.0,
                    y: 2.0,
                    width: 4.0,
                    height: 6.0,
                },
                8.0,
                0.0,
            )
            .unwrap();
        assert_eq!(
            full.border.rect,
            Rect {
                x: 12.0,
                y: 2.0,
                width: 12.0,
                height: 6.0
            }
        );
        assert_eq!(metrics.sides, FIRST | LAST);
        // Cleared tags are the no-resurrection condition after later fragment moves.
        translate_fragment(&mut fragment, -10.0, -10.0);
        assert!(
            fragment.owners[..4]
                .iter()
                .all(|v| v.inline_layer == InlineLayer::None)
        );
        for command in &fragment.commands[..4] {
            let area = command_rect(command);
            assert_eq!((area.width, area.height), (0.0, 0.0));
        }
        assert!(matches!(fragment.commands[4], DrawCommand::PopClip));
    }
}

#[test]
fn nested_fixed_pending_groups_skip_fragment_translation_and_keep_geometry() {
    let area = Rect {
        x: 999_999.0,
        y: 999_999.0,
        width: 4.0,
        height: 6.0,
    };
    let mut fragment = pending(area, true);
    fragment
        .commands
        .splice(0..0, [DrawCommand::PushFixed, DrawCommand::PushFixed]);
    fragment.owners.splice(0..0, [owner(InlineLayer::None); 2]);
    fragment
        .commands
        .extend([DrawCommand::PopFixed, DrawCommand::PopFixed]);
    fragment.owners.extend([owner(InlineLayer::None); 2]);
    translate_fragment(&mut fragment, 2.0, 2.0);
    for command in &fragment.commands[2..6] {
        assert_eq!(command_rect(command), area);
    }
    for geometry in &fragment.boxes {
        assert_eq!(
            (geometry.border.x, geometry.border.y),
            (999_999.0, 999_999.0)
        );
        assert!(geometry.fixed);
    }
    assert_eq!(fragment.hits[0].rect, area);
    assert!(fragment.hits[0].fixed);
    assert!(fragment.owners[2].inline_layer == InlineLayer::Clip);
    assert!(fragment.owners[4].inline_layer == InlineLayer::Image);
    assert!(matches!(fragment.commands[6], DrawCommand::PopClip));
    assert!(matches!(fragment.commands[7], DrawCommand::PopFixed));
    assert!(matches!(fragment.commands[8], DrawCommand::PopFixed));
}

#[test]
fn table_content_range_uses_same_pending_policy_and_respects_fixed_scope_at_start() {
    let area = Rect {
        x: 2.0,
        y: 999_999.0,
        width: 4.0,
        height: 6.0,
    };
    for (dy, refused) in [(1.0, false), (2.0, true)] {
        let mut fragment = pending(area, false);
        let header = DrawCommand::Rect {
            rect: Rect {
                x: 3.0,
                y: 4.0,
                width: 9.0,
                height: 8.0,
            },
            color: Color::rgb(0, 255, 0),
            radius: 0.0,
        };
        fragment.commands.insert(0, header);
        fragment.owners.insert(0, owner(InlineLayer::None));
        // Exact table call seam: a principal decoration preceding content_start
        // remains fixed while the child pending group receives vertical alignment.
        translate_fragment_commands(&mut fragment, 1, 0.0, dy);
        assert_eq!(
            command_rect(&fragment.commands[0]),
            Rect {
                x: 3.0,
                y: 4.0,
                width: 9.0,
                height: 8.0
            }
        );
        if refused {
            assert_refused(&fragment, 1);
        } else {
            for command in &fragment.commands[1..5] {
                assert_eq!(
                    command_rect(command),
                    Rect {
                        x: 2.0,
                        y: 1_000_000.0,
                        width: 4.0,
                        height: 6.0
                    }
                );
            }
        }
        // Command-only alignment does not mutate retained strip bookkeeping.
        assert_eq!(
            (fragment.boxes[0].border.x, fragment.boxes[0].border.y),
            (2.0, 999_999.0)
        );
    }
    let mut fragment = pending(area, true);
    fragment.commands.insert(0, DrawCommand::PushFixed);
    fragment.owners.insert(0, owner(InlineLayer::None));
    fragment.commands.push(DrawCommand::PopFixed);
    fragment.owners.push(owner(InlineLayer::None));
    // start is inside the fixed scope: the preceding opener must still count.
    translate_fragment_commands(&mut fragment, 1, 0.0, 2.0);
    for command in &fragment.commands[1..5] {
        assert_eq!(command_rect(command), area);
    }
    assert!(fragment.owners[1].inline_layer == InlineLayer::Clip);
    assert!(matches!(fragment.commands[5], DrawCommand::PopClip));
    assert!(matches!(fragment.commands[6], DrawCommand::PopFixed));
}
