use super::*;

fn render(body: &str, width: f32) -> (Document, LayoutResult) {
    let doc = Document::parse(&format!(
        "<style>body{{margin:0;font-size:1px;line-height:4px}}i{{display:inline-block;width:4px;height:4px;vertical-align:top}}</style>{body}"
    ));
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), width, 100.0);
    let result = layout(&doc, &styles, width, 100.0, &Fonts::new());
    (doc, result)
}
fn hit(doc: &Document, result: &LayoutResult, selector: &str) -> Rect {
    let node = doc.query_selector(selector).unwrap();
    result
        .hit_regions
        .iter()
        .find(|h| h.node == node)
        .unwrap()
        .rect
}
fn span() -> InlineSpan {
    InlineSpan {
        node: 0,
        parent: None,
        border: Sides {
            top: 1.0,
            right: 2.0,
            bottom: 3.0,
            left: 1.0,
        },
        padding: Sides {
            top: 2.0,
            right: 1.0,
            bottom: 0.0,
            left: 2.0,
        },
        margin: (0.0, 0.0),
        first: Some(0),
        last: Some(2),
        font_size: 20.0,
    }
}
#[test]
fn physical_edges_affect_positions_and_wrap_only_once() {
    let source = "<span style='padding-left:2px;padding-right:3px'><i id=a></i><i id=b></i></span><i id=c></i>";
    for (width, expected) in [
        (32.0, [(2.0, 0.0), (6.0, 0.0), (13.0, 0.0)]),
        (10.0, [(2.0, 0.0), (0.0, 4.0), (0.0, 8.0)]),
    ] {
        let (d, r) = render(source, width);
        for (s, (x, y)) in ["#a", "#b", "#c"].into_iter().zip(expected) {
            assert_eq!(hit(&d, &r, s), rect(x, y, 4.0, 4.0));
        }
    }
}
#[test]
fn closing_edge_is_in_last_unit_fit_test() {
    let (d, r) = render(
        "<i></i><span style='padding-left:2px;padding-right:5px'><i id=a></i><i id=b></i></span><i id=c></i>",
        14.0,
    );
    assert_eq!(hit(&d, &r, "#a"), rect(6.0, 0.0, 4.0, 4.0));
    assert_eq!(hit(&d, &r, "#b"), rect(0.0, 4.0, 4.0, 4.0));
    assert_eq!(hit(&d, &r, "#c"), rect(9.0, 4.0, 4.0, 4.0));
}
#[test]
fn nested_edges_and_empty_siblings_preserve_source_order() {
    let (d, r) = render(
        "<span style='padding-left:1px;padding-right:2px'><b style='padding-left:3px;padding-right:4px'><i id=a></i></b></span><span style='padding-left:2px;padding-right:3px'></span><i id=b></i>",
        64.0,
    );
    assert_eq!(hit(&d, &r, "#a").x, 4.0);
    assert_eq!(hit(&d, &r, "#b").x, 19.0);
    assert!(
        !r.commands
            .iter()
            .any(|c| matches!(c,DrawCommand::Text{text,..} if text.is_empty()))
    );
}
#[test]
fn trimmed_spaces_keep_closing_edges_and_do_not_add_a_line() {
    for source in [
        "<span style='padding-left:1px;padding-right:3px'><i id=a></i></span>",
        "<span style='padding-left:1px;padding-right:3px'>  <i id=a></i>  </span>",
    ] {
        let (d, r) = render(
            &format!("<div style='width:8px'>{source}</div><i id=b></i>"),
            32.0,
        );
        assert_eq!(hit(&d, &r, "#a"), rect(1.0, 0.0, 4.0, 4.0));
        assert_eq!(hit(&d, &r, "#b").y, 4.0);
    }
    let (d, r) = render("<span>  </span><i id=a></i>", 32.0);
    assert_eq!(hit(&d, &r, "#a"), rect(0.0, 0.0, 4.0, 4.0));
}
#[test]
fn preserved_space_width_is_inside_physical_edges() {
    let (d, r) = render(
        "<span style='white-space:pre;font:1px monospace;padding-left:2px;padding-right:3px'><i id=a></i> </span><i id=b></i>",
        64.0,
    );
    let space = r
        .commands
        .iter()
        .find_map(|c| {
            if let DrawCommand::Text { x, text, .. } = c {
                (text == " ").then_some(*x)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(hit(&d, &r, "#a").x, 2.0);
    assert_eq!(space, 6.0);
    let text_hit = r
        .hit_regions
        .iter()
        .find(|h| matches!(&d.nodes[h.node].kind,NodeKind::Text(t) if t==" "))
        .unwrap();
    assert_eq!(hit(&d, &r, "#b").x, 9.0 + text_hit.rect.width);
}
#[test]
fn explicit_break_keeps_only_first_and_last_physical_edges() {
    let (d, r) = render(
        "<span style='padding-left:3px;padding-right:5px'><i id=a></i><br><i id=b></i></span><i id=c></i>",
        32.0,
    );
    assert_eq!(hit(&d, &r, "#a"), rect(3.0, 0.0, 4.0, 4.0));
    assert_eq!(hit(&d, &r, "#b"), rect(0.0, 4.0, 4.0, 4.0));
    assert_eq!(hit(&d, &r, "#c"), rect(9.0, 4.0, 4.0, 4.0));
}
#[test]
fn signed_margins_and_full_width_percentage_basis_are_separate() {
    let (d, r) = render(
        "<span style='margin-left:-2px;margin-right:3px;padding-left:4px;padding-right:5px'><i id=a></i></span><i id=b></i>",
        64.0,
    );
    assert_eq!(hit(&d, &r, "#a").x, 2.0);
    assert_eq!(hit(&d, &r, "#b").x, 14.0);
    let (d, r) = render(
        "<div style='width:40px'><b style='float:left;width:16px;height:4px'></b><span style='padding-left:10%;padding-right:10%'><i id=a></i></span><i id=b></i></div>",
        64.0,
    );
    assert_eq!(hit(&d, &r, "#a").x, 20.0);
    assert_eq!(hit(&d, &r, "#b").x, 28.0);
}
#[test]
fn nominal_owner_boxes_exclude_leading_and_slice_side_insets() {
    let s = span();
    let a = s.geometry(3.0, 7.0, 0.0, 26.0, true, false);
    let b = s.geometry(0.0, 4.0, 40.0, 26.0, false, false);
    let c = s.geometry(0.0, 4.0, 80.0, 26.0, false, true);
    assert_eq!(
        (a.border, a.padding, a.content),
        (
            rect(0.0, 4.0, 7.0, 32.0),
            rect(1.0, 5.0, 6.0, 28.0),
            rect(3.0, 7.0, 4.0, 26.0)
        )
    );
    assert_eq!(
        (b.border, b.padding, b.content),
        (
            rect(0.0, 44.0, 4.0, 32.0),
            rect(0.0, 45.0, 4.0, 28.0),
            rect(0.0, 47.0, 4.0, 26.0)
        )
    );
    assert_eq!(
        (c.border, c.padding, c.content),
        (
            rect(0.0, 84.0, 7.0, 32.0),
            rect(0.0, 85.0, 5.0, 28.0),
            rect(0.0, 87.0, 4.0, 26.0)
        )
    );
}
#[test]
fn separate_inline_box_unions_do_not_alias_padding() {
    let s = span();
    let first = s.geometry(3.0, 7.0, 0.0, 26.0, true, false);
    let last = s.geometry(0.0, 4.0, 40.0, 26.0, false, true);
    let mut out = [None];
    merge_geometry(&mut out, &[first, last]);
    let g = out[0].unwrap();
    assert_eq!(g.border, rect(0.0, 4.0, 7.0, 72.0));
    assert_eq!(g.padding, rect(0.0, 5.0, 7.0, 68.0));
    assert_eq!(g.content, rect(0.0, 7.0, 7.0, 66.0));
}
#[test]
fn child_relative_motion_keeps_sibling_flow_and_parent_advance() {
    let (d, r) = render(
        "<span style='padding-left:1px;padding-right:2px'><i id=a></i><i id=b style='position:relative;left:5px'></i></span><i id=c></i>",
        64.0,
    );
    assert_eq!(hit(&d, &r, "#a").x, 1.0);
    assert_eq!(hit(&d, &r, "#b").x, 10.0);
    assert_eq!(hit(&d, &r, "#c").x, 11.0);
}
#[test]
fn span_admission_exact_and_one_short_preserve_remaining_work() {
    let mut work = 1;
    let mut spans = Vec::new();
    assert!(push_span(&mut spans, span(), &mut work).is_some());
    assert_eq!(work, 0);
    assert_eq!(spans.len(), 1);
    let mut work = 0;
    assert!(push_span(&mut spans, span(), &mut work).is_none());
    assert_eq!(spans.len(), 1);
    assert_eq!(work, 0);
    let mut work = 7;
    assert!(!charge(&mut work, 8));
    assert_eq!(work, 7);
}

#[test]
fn positioned_only_inline_retains_zero_width_containing_anchor() {
    let doc = Document::parse(
        "<!doctype html><style>html,body{margin:0;padding:0;background:white;font-size:1px;line-height:4px}span{position:relative}i{position:absolute;left:0;top:0;width:4px;height:4px;background:red}b{display:block;margin-left:8px;width:4px;height:4px;background:lime}</style><span><i id=anchor></i></span><b></b>",
    );
    let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 32.0, 16.0);
    let fonts = Fonts::new();
    let result = layout(&doc, &styles, 32.0, 16.0, &fonts);
    assert_eq!(hit(&doc, &result, "#anchor"), rect(0.0, 0.0, 4.0, 4.0));
    let mut canvas = crate::graphics::Canvas::new(32, 16).unwrap();
    canvas.paint(
        &result.commands,
        &fonts,
        &crate::graphics::ImageStore::new(),
        0.0,
        0.0,
    );
    for y in 0..16 {
        for x in 0..32 {
            let expected = if y < 4 && x < 4 {
                0xff0000
            } else if y < 4 && (8..12).contains(&x) {
                0x00ff00
            } else {
                0xffffff
            };
            assert_eq!(canvas.pixels[y * 32 + x], expected, "at {x},{y}");
        }
    }
}

#[test]
fn translated_line_subtracts_nominal_ascent_before_adding_line_top() {
    let s = InlineSpan {
        node: 0,
        parent: None,
        border: Sides::default(),
        padding: Sides::default(),
        margin: (0.0, 0.0),
        first: Some(0),
        last: Some(0),
        font_size: 16.0,
    };
    // At line top55, adding local15.2 first and then subtracting the same
    // ascent loses one f32 ULP. The local baseline contract preserves55 exactly.
    let g = s.geometry(0.0, 4.0, 55.0, 15.2, true, true);
    assert_eq!(g.content.y.to_bits(), 55.0f32.to_bits());
    assert_eq!(g.padding.y.to_bits(), 55.0f32.to_bits());
    assert_eq!(g.border.y.to_bits(), 55.0f32.to_bits());
}
