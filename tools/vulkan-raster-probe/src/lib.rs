#![forbid(unsafe_code)]

// Preserve the probe's public planner paths with one shared implementation.
pub use eris_raster_core::{
    Command, Draw, DrawKind, Frame, MAX_COMMANDS, MAX_COORDINATE, MAX_GPU_BUFFER_BYTES, MAX_HEIGHT,
    MAX_INVOCATIONS, MAX_LUT_ENTRIES, MAX_MASK_AXIS, MAX_MASK_COVERAGE_BYTES, MAX_MASK_PIXELS,
    MAX_ROW_ENTRIES, MAX_SCOPES, MAX_SOURCE_ENTRIES, MAX_SOURCE_RGBA_BYTES, MAX_WIDTH,
    PARAM_STRIDE, Plan, Rect, Result, SourceImage, SourceMask, plan, plan_with_images,
    plan_with_masks, rect,
};

pub mod alpha_fixtures;
#[cfg(feature = "browser-bridge")]
pub mod browser_adapter;
#[cfg(feature = "browser-bridge")]
pub mod browser_fixtures;
pub mod fixtures;
pub mod gpu;
pub mod image_fixtures;
