//! Literal pointer eligibility controls: pixels and retained geometry are independent of hit selection.
use eris::{
    dom::{NodeId, NodeKind},
    graphics::{Canvas, Color, Fonts},
    layout::{HitAction, LayoutResult},
    page::Page,
    script::Value,
};
use url::Url;

const WIDTH: u32 = 8;
const HEIGHT: u32 = 6;
type HitGeometry = (NodeId, HitAction, u32, u32, u32, u32, bool);

fn page(html: &str) -> Page {
    Page::from_html(
        Url::parse("https://pointer-events.invalid/").unwrap(),
        html,
        false,
    )
}

fn node(page: &Page, id: &str) -> NodeId {
    page.document.query_selector(&format!("#{id}")).unwrap()
}

fn execute(page: &mut Page, strict: bool, source: &str) -> Value {
    if strict {
        page.runtime.execute_strict(source, &mut page.document)
    } else {
        page.runtime.execute(source, &mut page.document)
    }
    .unwrap()
}

fn snapshot(page: &Page, fonts: &Fonts) -> (LayoutResult, Vec<u32>) {
    let layout = page.layout(WIDTH as f32, HEIGHT as f32, fonts);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels.len(), 48);
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    (layout, canvas.pixels)
}

fn geometry(layout: &LayoutResult) -> Vec<HitGeometry> {
    layout
        .hit_regions
        .iter()
        .map(|hit| {
            (
                hit.node,
                hit.action,
                hit.rect.x.to_bits(),
                hit.rect.y.to_bits(),
                hit.rect.width.to_bits(),
                hit.rect.height.to_bits(),
                hit.fixed,
            )
        })
        .collect()
}

fn preserved(before: &(LayoutResult, Vec<u32>), after: &(LayoutResult, Vec<u32>)) {
    assert_eq!(
        after.1, before.1,
        "pointer eligibility must not alter pixels"
    );
    assert_eq!(
        geometry(&after.0),
        geometry(&before.0),
        "all existing hit fields and ordering"
    );
    assert_eq!(
        after.0.content_height.to_bits(),
        before.0.content_height.to_bits()
    );
    assert_eq!(after.0.node_visibility, before.0.node_visibility);
}

fn check(html: &str, expected: &[u32; 48], points: &[(f32, f32, &str)]) {
    let page = page(html);
    let (layout, pixels) = snapshot(&page, &Fonts::new());
    assert_eq!(pixels.as_slice(), expected);
    for &(x, y, id) in points {
        assert_eq!(
            layout.hit_test(x, y),
            Some(node(&page, id)),
            "point ({x},{y}): {id}"
        );
    }
}

#[test]
fn pointer_events_auto_overlay() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:auto}</style><div id=under></div><div id=over></div>"###,
        &[
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
        ],
        &[(3.0, 3.0, "over")],
    );
}

#[test]
fn pointer_events_none_overlay() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:none}</style><div id=under></div><div id=over></div>"###,
        &[
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
        ],
        &[(3.0, 3.0, "under")],
    );
}

#[test]
fn pointer_events_inherited_none_child() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:none}#child{width:2px;height:2px;background:lime}</style><div id=under></div><div id=over><div id=child></div></div>"###,
        &[
            0x00ff00, 0x00ff00, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0x00ff00, 0x00ff00, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
        ],
        &[(1.0, 1.0, "under"), (3.0, 3.0, "under")],
    );
}

#[test]
fn pointer_events_reenabled_child() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:none}#child{pointer-events:auto;width:2px;height:2px;background:lime}</style><div id=under></div><div id=over><div id=child></div></div>"###,
        &[
            0x00ff00, 0x00ff00, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0x00ff00, 0x00ff00, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
        ],
        &[(1.0, 1.0, "child"), (3.0, 3.0, "under")],
    );
}

#[test]
fn pointer_events_visibility_hidden() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:auto;visibility:hidden}</style><div id=under></div><div id=over></div>"###,
        &[
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
        ],
        &[(3.0, 3.0, "under")],
    );
}

#[test]
fn pointer_events_transparent_auto() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:auto;opacity:0}</style><div id=under></div><div id=over></div>"###,
        &[
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
        ],
        &[(3.0, 3.0, "over")],
    );
}

#[test]
fn pointer_events_transparent_none() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:none;opacity:0}</style><div id=under></div><div id=over></div>"###,
        &[
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
        ],
        &[(3.0, 3.0, "under")],
    );
}

#[test]
fn pointer_events_clipped_reenabled_child() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:none;width:4px;height:4px;overflow:hidden}#child{pointer-events:auto;width:8px;height:2px;background:lime}</style><div id=under></div><div id=over><div id=child></div></div>"###,
        &[
            0x00ff00, 0x00ff00, 0x00ff00, 0x00ff00, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x00ff00, 0x00ff00, 0x00ff00, 0x00ff00, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
        ],
        &[
            (1.0, 1.0, "child"),
            (5.0, 1.0, "under"),
            (1.0, 3.0, "under"),
        ],
    );
}

#[test]
fn pointer_events_fixed_none() {
    check(
        r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{position:fixed;pointer-events:none}</style><div id=under></div><div id=over></div>"###,
        &[
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
        ],
        &[(3.0, 3.0, "under")],
    );
}

#[test]
fn pointer_events_cssom_auto_none_auto_preserves_pixels_and_geometry() {
    for strict in [false, true] {
        let mut page = page(
            r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:auto}</style><div id=under></div><div id=over></div>"###,
        );
        let fonts = Fonts::new();
        let before = snapshot(&page, &fonts);
        assert_eq!(
            before.1.as_slice(),
            &[
                0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
                0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
                0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
                0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
                0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
                0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000, 0xff0000,
            ]
        );
        for (source, target) in [
            (
                r###"document.getElementById('over').style.pointerEvents = 'auto';"###,
                "over",
            ),
            (
                r###"document.getElementById('over').style.pointerEvents = 'none';"###,
                "under",
            ),
            (
                r###"document.getElementById('over').style.pointerEvents = 'auto';"###,
                "over",
            ),
        ] {
            execute(&mut page, strict, source);
            let after = snapshot(&page, &fonts);
            preserved(&before, &after);
            assert_eq!(
                after.0.hit_test(3.0, 3.0),
                Some(node(&page, target)),
                "strict={strict}, target={target}"
            );
        }
    }
}

#[test]
fn pointer_events_inline_text_inheritance_and_explicit_override() {
    for strict in [false, true] {
        let mut page = page(
            r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{pointer-events:none;font-size:4px;line-height:6px}</style><div id=under></div><div id=over><span id=inline>X</span></div>"###,
        );
        let inline = node(&page, "inline");
        assert_eq!(page.document.nodes[inline].children.len(), 1);
        let text = page.document.nodes[inline].children[0];
        assert!(matches!(&page.document.nodes[text].kind, NodeKind::Text(value) if value == "X"));
        let under = node(&page, "under");
        let fonts = Fonts::new();
        let before = snapshot(&page, &fonts);
        assert!(before.0.hit_regions.iter().any(|hit| hit.node == text));
        assert!(before.1.iter().any(|&pixel| pixel != 0x0000ff));
        for (source, expected) in [
            (r###""###, under),
            (
                r###"document.getElementById('inline').style.pointerEvents = 'auto';"###,
                text,
            ),
            (
                r###"document.getElementById('inline').style.pointerEvents = 'none';"###,
                under,
            ),
        ] {
            if !source.is_empty() {
                execute(&mut page, strict, source);
            }
            let after = snapshot(&page, &fonts);
            preserved(&before, &after);
            assert_eq!(
                after.0.hit_test(1.0, 1.0),
                Some(expected),
                "strict={strict}"
            );
        }
    }
}

#[test]
fn pointer_events_table_row_and_cell_have_independent_owners() {
    for strict in [false, true] {
        let mut page = page(
            r###"<!doctype html><style>html,body{margin:0;padding:0}*{box-sizing:content-box}#under{position:absolute;left:0;top:0;width:8px;height:6px;background:blue}table{position:absolute;left:0;top:0;width:7px;margin:0;padding:0;border:0;border-spacing:1px;pointer-events:none;background:yellow}tr{height:2px;background:red}td{width:2px;height:2px;margin:0;padding:0;border:0;background:lime;vertical-align:top}</style><div id=under></div><table><tr id=row><td id=cell></td><td id=other></td></tr></table>"###,
        );
        let fonts = Fonts::new();
        let before = snapshot(&page, &fonts);
        for id in ["under", "row", "cell", "other"] {
            assert!(
                before
                    .0
                    .hit_regions
                    .iter()
                    .any(|hit| hit.node == node(&page, id)),
                "retained {id}"
            );
        }
        for (source, targets) in [
            (r###""###, ["under", "under", "under"]),
            (
                r###"document.getElementById('row').style.pointerEvents = 'auto'; document.getElementById('cell').style.pointerEvents = 'none'; document.getElementById('other').style.pointerEvents = 'none';"###,
                ["row", "row", "row"],
            ),
            (
                r###"document.getElementById('row').style.pointerEvents = 'none'; document.getElementById('cell').style.pointerEvents = 'auto';"###,
                ["cell", "under", "under"],
            ),
            (
                r###"document.getElementById('cell').style.pointerEvents = 'none';"###,
                ["under", "under", "under"],
            ),
        ] {
            if !source.is_empty() {
                execute(&mut page, strict, source);
            }
            let after = snapshot(&page, &fonts);
            preserved(&before, &after);
            for (x, id) in [1.5, 3.5, 4.5].into_iter().zip(targets) {
                assert_eq!(
                    after.0.hit_test(x, 1.5),
                    Some(node(&page, id)),
                    "strict={strict}, x={x}, owner={id}"
                );
            }
        }
    }
}

#[test]
fn pointer_events_reenabled_child_keeps_capture_and_bubble_without_skipped_sibling() {
    for strict in [false, true] {
        let mut page = page(
            r###"<!doctype html><style>html,body{margin:0;padding:0}#under,#over{position:absolute;left:0;top:0;width:8px;height:6px}#under{background:blue}#over{background:red}#over{pointer-events:none}#child{pointer-events:auto;width:2px;height:2px;background:lime}</style><div id=under></div><div id=over><div id=child></div></div><style>#skip{position:absolute;left:0;top:0;width:8px;height:6px;pointer-events:none}</style><div id=skip></div>"###,
        );
        execute(
            &mut page,
            strict,
            r###"var trace = '';
var over = document.getElementById('over');
var child = document.getElementById('child');
var skip = document.getElementById('skip');
over.addEventListener('click', function(e) { trace += 'C' + e.eventPhase + ':' + e.target.id + ':' + e.currentTarget.id + ';'; }, true);
child.addEventListener('click', function(e) { trace += 'T' + e.eventPhase + ':' + e.target.id + ':' + e.currentTarget.id + ';'; });
over.addEventListener('click', function(e) { trace += 'B' + e.eventPhase + ':' + e.target.id + ':' + e.currentTarget.id + ';'; });
skip.addEventListener('click', function() { trace += 'SKIP;'; });"###,
        );
        let child = node(&page, "child");
        let (layout, _) = snapshot(&page, &Fonts::new());
        // The expected DOM owner is fixed independently before event dispatch.
        assert_eq!(layout.hit_test(1.0, 1.0), Some(child));
        page.runtime
            .dispatch_click(child, &mut page.document)
            .unwrap();
        assert_eq!(
            execute(&mut page, strict, r###"trace"###),
            Value::String("C1:child:over;T2:child:child;B3:child:over;".into()),
            "strict={strict}"
        );
    }
}
