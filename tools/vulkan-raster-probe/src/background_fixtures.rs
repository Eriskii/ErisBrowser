//! Fixed literal raster-background inventory; no expected pixels come from rendering.
#[path = "../../../tests/support/vulkan_backgrounds.rs"]
pub mod witness;
use crate::{Plan, Result};
use eris::graphics::Fonts;

pub struct PreparedFixture {
    pub name: &'static str,
    pub expected: Vec<u32>,
    pub plan: Plan,
}
pub fn fixtures() -> Result<Vec<PreparedFixture>> {
    let fonts = Fonts::new();
    let fixtures = witness::fixtures();
    if fixtures.iter().map(|f| f.name).collect::<Vec<_>>() != witness::FIXTURE_NAMES
        || fixtures.iter().map(|f| f.expected.len()).sum::<usize>() != witness::LITERAL_PIXELS
    {
        return Err("background literal inventory".into());
    }
    fixtures
        .into_iter()
        .map(|fixture| {
            if fixture.canvas_pixels(&fonts) != fixture.expected {
                return Err(format!("{} Canvas/literal mismatch", fixture.name));
            }
            let native = fixture.native(&fonts)?;
            if witness::packed_pixels(native.plan()) != fixture.expected {
                return Err(format!("{} packed Native/literal mismatch", fixture.name));
            }
            Ok(PreparedFixture {
                name: fixture.name,
                expected: fixture.expected,
                plan: native.into_plan(),
            })
        })
        .collect()
}
