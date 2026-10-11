//! Explicit variable longhands retain specified values without synthesizing margin.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::Page,
};
use url::Url;
#[path = "support/cssom_var_getter.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;

#[test]
fn explicit_margin_vars_have_empty_shorthand_results_and_literal_pixels() {
    let mut page = Page::from_html(
        Url::parse("https://var-getter.invalid/").unwrap(),
        oracle::HTML,
        true,
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let direct = page.document.query_selector("#direct").unwrap();
    let pending = page.document.query_selector("#pending").unwrap();
    let fonts = Fonts::new();
    for (stage, state) in oracle::STATES.iter().enumerate() {
        if stage > 0 {
            assert!(page.click(direct).is_none());
            assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        }
        for width in [40u32, 48] {
            let height = 16;
            let layout = page.layout(width as f32, height as f32, &fonts);
            let expected = oracle::expected(width, height, state);
            let mut canvas = Canvas::new(width, height).unwrap();
            canvas.clear(Color::WHITE);
            canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
            assert!(!canvas.exhausted());
            oracle::assert_pixels(&canvas.pixels, &expected, width, stage);
            #[cfg(feature = "raster-bridge")]
            {
                use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
                let frame = eris_raster_core::Frame::new(width, height, oracle::WHITE);
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
                oracle::assert_pixels(
                    &witness::packed_pixels(scene.plan()),
                    &expected,
                    width,
                    stage,
                );
            }
            for (node, &(x, y, _, _, _)) in [direct, pending].iter().zip(&state.rectangles) {
                assert_eq!(layout.hit_test(x as f32 + 1.0, y as f32 + 1.0), Some(*node));
            }
            // The unchanged-parent baseline must pass its pixel control before
            // the first empty-getter assertion can fail.
            assert_eq!(page.title(), state.title, "stage={stage}");
        }
    }
}
