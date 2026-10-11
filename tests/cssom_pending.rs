//! Literal pixels for persistent pending shorthand declarations and CSSOM edits.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::Page,
};
use url::Url;

#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

const WHITE: u32 = 0xffffff;
const RED: u32 = 0xff0000;
const BLUE: u32 = 0x0000ff;

fn page(extra: &str) -> Page {
    let source = include_str!("fixtures/cssom-pending.html")
        .replace("</style>", &format!("{extra}</style>"));
    let page = Page::from_html(
        Url::parse("https://pending.invalid/").unwrap(),
        &source,
        true,
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    page
}
fn execute(page: &mut Page, source: &str) {
    page.runtime.execute(source, &mut page.document).unwrap();
}
fn check(page: &Page, rectangles: &[(u32, u32, u32, u32, u32)]) {
    let fonts = Fonts::new();
    for width in [40u32, 48] {
        let height = 32u32;
        let layout = page.layout(width as f32, height as f32, &fonts);
        let mut expected = vec![WHITE; (width * height) as usize];
        for &(x, y, w, h, color) in rectangles {
            assert!(x + w <= width && y + h <= height);
            for row in y..y + h {
                for column in x..x + w {
                    expected[(row * width + column) as usize] = color;
                }
            }
        }
        let mut canvas = Canvas::new(width, height).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        if let Some(i) = canvas
            .pixels
            .iter()
            .zip(&expected)
            .position(|(a, b)| a != b)
        {
            panic!(
                "pixel ({},{}) actual={:06x} expected={:06x}",
                i % width as usize,
                i / width as usize,
                canvas.pixels[i],
                expected[i]
            );
        }
        #[cfg(feature = "raster-bridge")]
        {
            use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
            let frame = eris_raster_core::Frame::new(width, height, WHITE);
            let scene = plan_native_scene(
                frame,
                &[NativePhase {
                    frame,
                    commands: &layout.commands,
                }],
                &page.images,
                &fonts,
            )
            .unwrap();
            assert_eq!(witness::packed_pixels(scene.plan()), expected);
        }
    }
}
const INITIAL: &[(u32, u32, u32, u32, u32)] = &[(2, 2, 12, 8, RED), (6, 3, 6, 4, BLUE)];
const LOWERED: &[(u32, u32, u32, u32, u32)] = &[(2, 2, 17, 8, RED), (11, 3, 6, 4, BLUE)];
const LIVE_VARIABLE: &[(u32, u32, u32, u32, u32)] = &[(2, 2, 21, 16, RED), (11, 7, 6, 4, BLUE)];
const TEXT_REPARSED: &[(u32, u32, u32, u32, u32)] = &[(2, 2, 15, 4, RED), (11, 2, 6, 4, BLUE)];

#[test]
fn complete_pending_padding_control_has_literal_initial_geometry() {
    check(&page(""), INITIAL);
}
#[test]
fn lowering_one_pending_side_survives_separate_script_access_and_resize() {
    let mut page = page("");
    execute(
        &mut page,
        "document.getElementById('target').style.setProperty('padding-left','9px','');",
    );
    check(&page, LOWERED);
    execute(
        &mut page,
        "document.getElementById('target').style.color='green';",
    );
    check(&page, LOWERED);
    execute(
        &mut page,
        "document.getElementById('target').style.setProperty('--p','5px 6px 7px 8px','');",
    );
    check(&page, LIVE_VARIABLE);
}
#[test]
fn removing_an_empty_serialized_pending_side_changes_real_geometry() {
    let mut page = page("");
    execute(
        &mut page,
        "document.getElementById('target').style.removeProperty('padding-left');",
    );
    check(&page, &[(2, 2, 8, 8, RED), (2, 3, 6, 4, BLUE)]);
    execute(
        &mut page,
        "document.getElementById('target').style.setProperty('--p','5px 6px 7px 8px','');",
    );
    check(&page, &[(2, 2, 12, 16, RED), (2, 7, 6, 4, BLUE)]);
}
#[test]
fn stylesheet_important_wins_only_the_lowered_inline_component() {
    let mut page = page("#target{padding-left:2px!important}");
    check(&page, INITIAL);
    execute(
        &mut page,
        "document.getElementById('target').style.setProperty('padding-left','9px','');",
    );
    check(&page, &[(2, 2, 10, 8, RED), (4, 3, 6, 4, BLUE)]);
}
#[test]
fn equal_text_external_attribute_write_discards_hidden_pending_state() {
    let mut page = page("");
    execute(
        &mut page,
        "var target=document.getElementById('target');target.style.setProperty('padding-left','9px','');",
    );
    check(&page, LOWERED);
    let target = page.document.query_selector("#target").unwrap();
    let before = page.document.attr(target, "style").unwrap().to_owned();
    execute(
        &mut page,
        "target.setAttribute('style',target.getAttribute('style'));",
    );
    assert_eq!(page.document.attr(target, "style"), Some(before.as_str()));
    check(&page, TEXT_REPARSED);
    execute(
        &mut page,
        "target.style.setProperty('--p','5px 6px 7px 8px','');",
    );
    check(&page, TEXT_REPARSED);
}
#[test]
fn css_text_self_assignment_reparses_partial_pending_declarations() {
    let mut page = page("");
    execute(
        &mut page,
        "var target=document.getElementById('target');target.style.setProperty('padding-left','9px','');",
    );
    check(&page, LOWERED);
    execute(&mut page, "target.style.cssText=target.style.cssText;");
    check(&page, TEXT_REPARSED);
}
