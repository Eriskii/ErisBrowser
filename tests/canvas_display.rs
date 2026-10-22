//! Literal 8x6 canvas backgrounds; white is an explicit backing-surface clear.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::Page,
    script::Value,
};
use url::Url;

fn page(source: &str) -> Page {
    Page::from_html(
        Url::parse("https://canvas-display.invalid/").unwrap(),
        source,
        false,
    )
}

fn assert_pixels(page: &Page, fonts: &Fonts, expected: &[u32; 48]) {
    let layout = page.layout(8.0, 6.0, fonts);
    let mut canvas = Canvas::new(8, 6).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels.as_slice(), expected);
}

fn check_static(source: &str, expected: [u32; 48]) {
    assert_pixels(&page(source), &Fonts::new(), &expected);
}

fn check_dynamic(source: &str, phases: &[(&str, [u32; 48])]) {
    for strict in [false, true] {
        let mut page = page(source);
        let fonts = Fonts::new();
        for (source, expected) in phases {
            let result = if strict {
                page.runtime.execute_strict(source, &mut page.document)
            } else {
                page.runtime.execute(source, &mut page.document)
            };
            assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
            assert_pixels(&page, &fonts, expected);
        }
    }
}

#[test]
fn canvas_display_root_none_suppresses_root_color_and_body_fallback() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:none;background:red}body{background:lime}</style></head><body id=body></body></html>"#,
        [0xffffff; 48],
    );
}

#[test]
fn canvas_display_root_none_suppresses_propagated_body_color() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:none;background:transparent}body{background:red}</style></head><body id=body></body></html>"#,
        [0xffffff; 48],
    );
}

#[test]
fn canvas_display_body_none_keeps_transparent_canvas() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:block;background:transparent}body{display:none;background:red}</style></head><body id=body></body></html>"#,
        [0xffffff; 48],
    );
}

#[test]
fn canvas_display_displayed_root_color_survives_body_none() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:block;background:blue}body{display:none;background:red}</style></head><body id=body></body></html>"#,
        [0x0000ff; 48],
    );
}

#[test]
fn canvas_display_visibility_hidden_root_keeps_canvas_color() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{visibility:hidden;background:yellow}body{background:transparent}</style></head><body id=body></body></html>"#,
        [0xffff00; 48],
    );
}

#[test]
fn canvas_display_visibility_hidden_body_keeps_propagated_color() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{background:transparent}body{visibility:hidden;background:yellow}</style></head><body id=body></body></html>"#,
        [0xffff00; 48],
    );
}

#[test]
fn canvas_display_root_precedence_keeps_ordinary_body_paint() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{background:blue}body{width:2px;height:1px;background:red}</style></head><body id=body></body></html>"#,
        [
            0xff0000, 0xff0000, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
            0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff, 0x0000ff,
        ],
    );
}

#[test]
fn canvas_display_multiple_none_does_not_enable_body_propagation() {
    check_static(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{background:transparent;background-image:none,none}body{width:2px;height:1px;background:red}</style></head><body id=body></body></html>"#,
        [
            0xff0000, 0xff0000, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
            0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff, 0xffffff,
        ],
    );
}

#[test]
fn canvas_display_cssom_root_owned_block_none_block() {
    check_dynamic(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:block;background:red}body{background:transparent}</style></head><body id=body></body></html>"#,
        &[
            (
                r#"var canvasStyle = document.getElementById('root').style; canvasStyle.setProperty('display', 'block'); canvasStyle.display === 'block';"#,
                [0xff0000; 48],
            ),
            (
                r#"canvasStyle.display = 'none'; canvasStyle.getPropertyValue('display') === 'none';"#,
                [0xffffff; 48],
            ),
            (
                r#"canvasStyle.setProperty('display', 'block'); canvasStyle.display === 'block';"#,
                [0xff0000; 48],
            ),
        ],
    );
}

#[test]
fn canvas_display_cssom_body_propagated_root_block_none_block() {
    check_dynamic(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:block;background:transparent}body{width:2px;height:1px;background:red}</style></head><body id=body></body></html>"#,
        &[
            (
                r#"var canvasStyle = document.getElementById('root').style; canvasStyle.setProperty('display', 'block'); canvasStyle.display === 'block';"#,
                [0xff0000; 48],
            ),
            (
                r#"canvasStyle.display = 'none'; canvasStyle.getPropertyValue('display') === 'none';"#,
                [0xffffff; 48],
            ),
            (
                r#"canvasStyle.setProperty('display', 'block'); canvasStyle.display === 'block';"#,
                [0xff0000; 48],
            ),
        ],
    );
}

#[test]
fn canvas_display_cssom_body_block_none_block() {
    check_dynamic(
        r#"<!doctype html><html id=root><head><style>html,body{margin:0;padding:0}html{display:block;background:transparent}body{display:block;width:2px;height:1px;background:red}</style></head><body id=body></body></html>"#,
        &[
            (
                r#"var canvasStyle = document.getElementById('body').style; canvasStyle.setProperty('display', 'block'); canvasStyle.display === 'block';"#,
                [0xff0000; 48],
            ),
            (
                r#"canvasStyle.display = 'none'; canvasStyle.getPropertyValue('display') === 'none';"#,
                [0xffffff; 48],
            ),
            (
                r#"canvasStyle.setProperty('display', 'block'); canvasStyle.display === 'block';"#,
                [0xff0000; 48],
            ),
        ],
    );
}
