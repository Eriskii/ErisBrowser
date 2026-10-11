//! Independent literal/native agreement; the dedicated probe runs the same plans.
#![cfg(feature = "raster-bridge")]
#[path = "support/vulkan_backgrounds.rs"]
mod witness;
use eris::graphics::Fonts;

#[test]
fn vulkan_backgrounds_complete_literal_roster_matches_canvas_and_packed_native() {
    let fonts = Fonts::new();
    let fixtures = witness::fixtures();
    assert_eq!(
        fixtures.iter().map(|f| f.name).collect::<Vec<_>>(),
        witness::FIXTURE_NAMES
    );
    assert_eq!(
        fixtures.iter().map(|f| f.expected.len()).sum::<usize>(),
        witness::LITERAL_PIXELS
    );
    assert_eq!(fixtures.len() * 2, witness::CONVERTED_FRAMES);
    assert_eq!(witness::LITERAL_PIXELS as u64 * 8, witness::COMPARED_BYTES);
    for fixture in fixtures {
        assert_eq!(
            fixture.canvas_pixels(&fonts),
            fixture.expected,
            "{}",
            fixture.name
        );
        let native = fixture
            .native(&fonts)
            .expect("frozen square-corner background admits Native");
        assert_eq!(
            witness::packed_pixels(native.plan()),
            fixture.expected,
            "{}",
            fixture.name
        );
        if fixture.name == "background-alpha-nested-opacity" {
            assert!(native.plan().group_scratch_bytes() > 0);
            assert!(
                native
                    .plan()
                    .draws()
                    .iter()
                    .any(|draw| draw.target_is_group())
            );
        }
    }
}
