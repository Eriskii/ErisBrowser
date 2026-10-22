//! Source-derived visibility pixels and geometry; references never use visibility.
//! Table row-layer cases follow the selected CSS Tables 3 draft interpretation,
//! which has known historical interoperability differences.
use eris::{
    dom::Document,
    graphics::{Canvas, Color, Fonts},
    page::Page,
    script::{Runtime, Value},
};
use url::Url;

const WIDTH: u32 = 40;
const HEIGHT: u32 = 32;

type Rect = (u32, u32, u32, u32, u32);
type Hit<'a> = (&'a str, u32, u32, u32, u32, bool);

fn pixels(page: &Page, fonts: &Fonts) -> Vec<u32> {
    let layout = page.layout(WIDTH as f32, HEIGHT as f32, fonts);
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    canvas.pixels
}

fn compare(actual: &[u32], expected: &[u32]) {
    assert_eq!(actual.len(), expected.len());
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "pixel ({},{}) actual={:06x} expected={:06x}",
            index % WIDTH as usize,
            index / WIDTH as usize,
            actual[index],
            expected[index]
        );
    }
}

struct Scene<'a> {
    html: &'a str,
    reference: &'a str,
    background: u32,
    rectangles: Option<&'a [Rect]>,
    hidden: &'a [&'a str],
    hits: &'a [Hit<'a>],
    points: &'a [(&'a str, f32, f32)],
    content_height: Option<f32>,
}

fn check(scene: Scene<'_>) {
    assert!(!scene.reference.contains("visibility"));
    let url = Url::parse("https://visibility.invalid/").unwrap();
    let page = Page::from_html(url.clone(), scene.html, false);
    let reference = Page::from_html(url, scene.reference, false);
    let fonts = Fonts::new();
    let actual = pixels(&page, &fonts);
    let expected_reference = pixels(&reference, &fonts);
    compare(&actual, &expected_reference);
    if let Some(rectangles) = scene.rectangles {
        let mut expected = vec![scene.background; (WIDTH * HEIGHT) as usize];
        for &(x, y, width, height, color) in rectangles {
            assert!(x + width <= WIDTH && y + height <= HEIGHT);
            for row in y..y + height {
                expected[(row * WIDTH + x) as usize..(row * WIDTH + x + width) as usize]
                    .fill(color);
            }
        }
        compare(&expected_reference, &expected);
        compare(&actual, &expected);
    } else {
        assert!(
            expected_reference
                .iter()
                .any(|&pixel| pixel != scene.background)
        );
    }
    let layout = page.layout(WIDTH as f32, HEIGHT as f32, &fonts);
    for id in scene.hidden {
        let node = page.document.query_selector(&format!("#{id}")).unwrap();
        assert!(
            !layout.hit_regions.iter().any(|hit| hit.node == node),
            "hidden hit: {id}"
        );
    }
    for &(id, x, y, width, height, fixed) in scene.hits {
        let node = page.document.query_selector(&format!("#{id}")).unwrap();
        let hit = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == node)
            .unwrap();
        assert_eq!(
            (
                hit.rect.x,
                hit.rect.y,
                hit.rect.width,
                hit.rect.height,
                hit.fixed
            ),
            (x as f32, y as f32, width as f32, height as f32, fixed),
            "{id}"
        );
        assert_eq!(
            layout.hit_test(x as f32 + 0.5, y as f32 + 0.5),
            Some(node),
            "{id}"
        );
    }
    for &(id, x, y) in scene.points {
        let node = page.document.query_selector(&format!("#{id}")).unwrap();
        assert_eq!(layout.hit_test(x, y), Some(node), "{id}");
    }
    if let Some(height) = scene.content_height {
        assert_eq!(layout.content_height, height);
    }
}

#[test]
fn visibility_block_geometry_border_and_visible_descendant() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div id=hidden style="visibility:hidden;width:12px;height:8px;padding:2px;border:2px solid red;background:red"><div id=child style="visibility:visible;width:4px;height:4px;background:blue"></div></div><div id=after style="width:5px;height:3px;background:lime"></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:4px;top:4px;width:4px;height:4px;background:#0000ff"></i><i style="left:0px;top:16px;width:5px;height:3px;background:#00ff00"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(4, 4, 4, 4, 0x0000ff), (0, 16, 5, 3, 0x00ff00)]),
        hidden: &["hidden"],
        hits: &[("child", 4, 4, 4, 4, false), ("after", 0, 16, 5, 3, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_inline_fragments_preserve_width_and_child_overrides() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div style="font-size:1px;line-height:1px"><span id=hidden style="visibility:hidden;background:red"><i id=gone style="display:inline-block;width:6px;height:4px;background:red;vertical-align:top"></i><i id=child style="display:inline-block;visibility:visible;width:6px;height:4px;background:blue;vertical-align:top"></i></span><i id=after style="display:inline-block;width:6px;height:4px;background:lime;vertical-align:top"></i></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:6px;top:0px;width:6px;height:4px;background:#0000ff"></i><i style="left:12px;top:0px;width:6px;height:4px;background:#00ff00"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(6, 0, 6, 4, 0x0000ff), (12, 0, 6, 4, 0x00ff00)]),
        hidden: &["hidden", "gone"],
        hits: &[("child", 6, 0, 6, 4, false), ("after", 12, 0, 6, 4, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_flex_hidden_item_reserves_slot_and_gap() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div style="display:flex;width:20px;gap:2px;align-items:flex-start"><div id=hidden style="visibility:hidden;flex-shrink:0;width:6px;height:4px;background:red"><div id=child style="visibility:visible;width:3px;height:2px;background:blue"></div></div><div id=after style="flex-shrink:0;width:6px;height:4px;background:lime"></div></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:0px;top:0px;width:3px;height:2px;background:#0000ff"></i><i style="left:8px;top:0px;width:6px;height:4px;background:#00ff00"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(0, 0, 3, 2, 0x0000ff), (8, 0, 6, 4, 0x00ff00)]),
        hidden: &["hidden"],
        hits: &[("child", 0, 0, 3, 2, false), ("after", 8, 0, 6, 4, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_grid_hidden_item_preserves_tracks() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div style="display:grid;width:16px;grid-template-columns:6px 8px;grid-template-rows:4px;gap:2px"><div id=hidden style="visibility:hidden;background:red"><div id=child style="visibility:visible;width:3px;height:2px;background:blue"></div></div><div id=after style="background:lime"></div></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:0px;top:0px;width:3px;height:2px;background:#0000ff"></i><i style="left:8px;top:0px;width:8px;height:4px;background:#00ff00"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(0, 0, 3, 2, 0x0000ff), (8, 0, 8, 4, 0x00ff00)]),
        hidden: &["hidden"],
        hits: &[("child", 0, 0, 3, 2, false), ("after", 8, 0, 8, 4, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_row_visible_cell_owns_row_background() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><style>table{position:absolute;left:2px;top:2px;box-sizing:content-box;width:14px;margin:0;padding:0;border:0;border-spacing:2px 1px;background:yellow}tr{height:2px}td{box-sizing:content-box;width:4px;height:2px;margin:0;padding:0;border:0;background:transparent;vertical-align:top}</style><table><tr id=row style="visibility:hidden;background:red"><td id=cell style="visibility:visible"></td><td id=hidden></td></tr></table>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:2px;top:2px;width:14px;height:4px;background:#ffff00"></i><i style="left:4px;top:3px;width:4px;height:2px;background:#ff0000"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(2, 2, 14, 4, 0xffff00), (4, 3, 4, 2, 0xff0000)]),
        hidden: &["row", "hidden"],
        hits: &[("cell", 4, 3, 4, 2, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_visible_row_hidden_cell_suppresses_row_background() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><style>table{position:absolute;left:2px;top:2px;box-sizing:content-box;width:14px;margin:0;padding:0;border:0;border-spacing:2px 1px;background:yellow}tr{height:2px}td{box-sizing:content-box;width:4px;height:2px;margin:0;padding:0;border:0;background:transparent;vertical-align:top}</style><table><tr style="background:red"><td id=hidden style="visibility:hidden"></td><td id=cell></td></tr></table>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:2px;top:2px;width:14px;height:4px;background:#ffff00"></i><i style="left:10px;top:3px;width:4px;height:2px;background:#ff0000"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(2, 2, 14, 4, 0xffff00), (10, 3, 4, 2, 0xff0000)]),
        hidden: &["hidden"],
        hits: &[("cell", 10, 3, 4, 2, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_visible_descendant_does_not_revive_hidden_cell_layer() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><style>table{position:absolute;left:2px;top:2px;box-sizing:content-box;width:14px;margin:0;padding:0;border:0;border-spacing:2px 1px;background:yellow}tr{height:2px}td{box-sizing:content-box;width:4px;height:2px;margin:0;padding:0;border:0;background:transparent;vertical-align:top}</style><table><tr style="background:red"><td id=hidden style="visibility:hidden"><div id=child style="visibility:visible;width:2px;height:2px;background:blue"></div></td><td id=cell></td></tr></table>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:2px;top:2px;width:14px;height:4px;background:#ffff00"></i><i style="left:4px;top:3px;width:2px;height:2px;background:#0000ff"></i><i style="left:10px;top:3px;width:4px;height:2px;background:#ff0000"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[
            (2, 2, 14, 4, 0xffff00),
            (4, 3, 2, 2, 0x0000ff),
            (10, 3, 4, 2, 0xff0000),
        ]),
        hidden: &["hidden"],
        hits: &[("child", 4, 3, 2, 2, false), ("cell", 10, 3, 4, 2, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_table_visible_row_keeps_only_cell_layers() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><style>table{position:absolute;left:2px;top:2px;box-sizing:content-box;width:14px;margin:0;padding:0;border:0;border-spacing:2px 1px;background:yellow}tr{height:2px}td{box-sizing:content-box;width:4px;height:2px;margin:0;padding:0;border:0;background:transparent;vertical-align:top}</style><table id=hidden style="visibility:hidden"><tr style="visibility:visible;background:red"><td id=cell></td><td></td></tr></table>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:4px;top:3px;width:4px;height:2px;background:#ff0000"></i><i style="left:10px;top:3px;width:4px;height:2px;background:#ff0000"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(4, 3, 4, 2, 0xff0000), (10, 3, 4, 2, 0xff0000)]),
        hidden: &["hidden"],
        hits: &[("cell", 4, 3, 4, 2, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_root_preserves_canvas_background() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0}html{background:yellow;visibility:hidden;border:2px solid red}body{background:transparent}#child{visibility:visible;width:4px;height:3px;background:blue}</style><html id=root><body id=body><div id=child></div></body></html>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffff00}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:2px;top:2px;width:4px;height:3px;background:#0000ff"></i>"#,
        background: 0xffff00,
        rectangles: Some(&[(2, 2, 4, 3, 0x0000ff)]),
        hidden: &["root", "body"],
        hits: &[("child", 2, 2, 4, 3, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_body_preserves_propagated_canvas_background() {
    check(Scene {
        html: r#"<!doctype html><html><style>html,body{margin:0;padding:0}html{background:transparent}body{background:yellow;visibility:hidden;border:2px solid red}#child{visibility:visible;width:4px;height:3px;background:blue}</style><body id=body><div id=child></div></body></html>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffff00}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:2px;top:2px;width:4px;height:3px;background:#0000ff"></i>"#,
        background: 0xffff00,
        rectangles: Some(&[(2, 2, 4, 3, 0x0000ff)]),
        hidden: &["body"],
        hits: &[("child", 2, 2, 4, 3, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_clip_opacity_scope_survives_for_visible_child() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div id=hidden style="position:absolute;left:2px;top:2px;width:8px;height:6px;visibility:hidden;background:red;overflow:hidden;opacity:.5"><div id=child style="visibility:visible;width:12px;height:4px;background:blue"></div></div><div id=after style="position:absolute;left:0;top:12px;width:4px;height:3px;background:lime"></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:2px;top:2px;width:8px;height:4px;background:#8080ff"></i><i style="left:0px;top:12px;width:4px;height:3px;background:#00ff00"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(2, 2, 8, 4, 0x8080ff), (0, 12, 4, 3, 0x00ff00)]),
        hidden: &["hidden"],
        hits: &[("after", 0, 12, 4, 3, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_fixed_scope_preserves_child_coordinates() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div id=hidden style="position:fixed;left:4px;top:5px;width:10px;height:7px;visibility:hidden;background:red"><div id=child style="visibility:visible;width:3px;height:2px;background:blue"></div></div><div id=after style="width:4px;height:3px;background:lime"></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:4px;top:5px;width:3px;height:2px;background:#0000ff"></i><i style="left:0px;top:0px;width:4px;height:3px;background:#00ff00"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(4, 5, 3, 2, 0x0000ff), (0, 0, 4, 3, 0x00ff00)]),
        hidden: &["hidden"],
        hits: &[("child", 4, 5, 3, 2, true), ("after", 0, 0, 4, 3, false)],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_extent_remains_scrollable() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div id=hidden style="visibility:hidden;width:5px;height:48px;background:red"></div><div id=after style="width:5px;height:3px;background:lime"></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style>"#,
        background: 0xffffff,
        rectangles: Some(&[]),
        hidden: &["hidden"],
        hits: &[("after", 0, 48, 5, 3, false)],
        points: &[],
        content_height: Some(51.0),
    });
}

#[test]
fn visibility_hidden_front_does_not_occlude_hits_or_visible_child() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div id=back style="position:absolute;left:2px;top:2px;width:12px;height:8px;background:red"></div><div id=hidden style="position:absolute;left:2px;top:2px;width:12px;height:8px;background:blue;visibility:hidden"><div id=child style="visibility:visible;width:3px;height:2px;background:lime"></div></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:#ffffff}i{position:absolute;display:block;margin:0;padding:0;border:0}</style><i style="left:2px;top:2px;width:12px;height:8px;background:#ff0000"></i><i style="left:2px;top:2px;width:3px;height:2px;background:#00ff00"></i>"#,
        background: 0xffffff,
        rectangles: Some(&[(2, 2, 12, 8, 0xff0000), (2, 2, 3, 2, 0x00ff00)]),
        hidden: &["hidden"],
        hits: &[("child", 2, 2, 3, 2, false)],
        points: &[("back", 10.0, 7.0)],
        content_height: None,
    });
}

#[test]
fn visibility_hidden_text_inherits_and_visible_text_override_paints() {
    check(Scene {
        html: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div style="font-size:12px;line-height:16px"><span id=hidden style="visibility:hidden;color:red">HIDDEN<b id=child style="visibility:visible;color:blue">OK</b></span></div>"#,
        reference: r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}*{box-sizing:content-box}</style><div style="font-size:12px;line-height:16px"><span id=hidden style="color:transparent">HIDDEN<b id=child style="color:blue">OK</b></span></div>"#,
        background: 0xffffff,
        rectangles: None,
        hidden: &["hidden"],
        hits: &[],
        points: &[],
        content_height: None,
    });
}

#[test]
fn visibility_cssom_normalization_invalid_noop_and_supports() {
    for strict in [false, true] {
        let mut doc = Document::parse(r#"<body><div id=target></div></body>"#);
        let mut runtime = Runtime::new();
        let source = r#"var s = document.getElementById('target').style;
var checks = [];
checks.push('visibility' in s && s.visibility === '');
s.visibility = 'HIDDEN';
checks.push(s.visibility === 'hidden' && s.cssText === 'visibility: hidden;');
s.setProperty('visibility', '\\76 isible', 'important');
checks.push(s.visibility === 'visible' && s.getPropertyPriority('VISIBILITY') === 'important');
s.setProperty('visibility', 'hi/**/dden');
s.setProperty('visibility', 'collapse');
s.setProperty('visibility', 'visible hidden');
s.setProperty('visibility', 'hidden;color:red');
checks.push(s.visibility === 'visible' && s.getPropertyPriority('visibility') === 'important' && s.color === '');
s.setProperty('visibility', 'hidden/**/');
checks.push(s.visibility === 'hidden' && s.getPropertyPriority('visibility') === '');
checks.push(s.removeProperty('visibility') === 'hidden' && s.visibility === '');
s.cssText = 'visibility:visible;visibility:collapse;visibility:\\68 idden !important';
checks.push(s.visibility === 'hidden' && s.cssText === 'visibility: hidden !important;');
s.visibility = 'initial';
checks.push(s.visibility === 'initial' && s.getPropertyPriority('visibility') === '');
s.setProperty('--v', 'visible');
s.setProperty('visibility', 'var(--v)');
checks.push(s.getPropertyValue('visibility') === 'var(--v)');
checks.push(CSS.supports('visibility', '\\68 idden') && !CSS.supports('vi\\73 ibility', 'hidden'));
checks.push(CSS.supports('(vi\\73 ibility:\\68 idden)') && !CSS.supports('visibility', 'hi/**/dden'));
checks.every(function(v){return v;});"#;
        let result = if strict {
            runtime.execute_strict(source, &mut doc)
        } else {
            runtime.execute(source, &mut doc)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
    }
}
