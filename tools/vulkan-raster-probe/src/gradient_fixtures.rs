//! Native adapter gradient fixtures; the expectations contain no rendered data.
#[path = "../../../tests/support/vulkan_gradients.rs"]
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
    if fixtures.len() != 3 {
        return Err("gradient literal inventory".into());
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
