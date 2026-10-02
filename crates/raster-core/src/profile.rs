//! Closed admission policies. Callers cannot supply arbitrary limits.
use crate::{
    MAX_COMMANDS, MAX_GPU_BUFFER_BYTES, MAX_HEIGHT, MAX_INVOCATIONS, MAX_LUT_ENTRIES, MAX_WIDTH,
    Result,
};

pub const NATIVE_MAX_WIDTH: u32 = 1280;
pub const NATIVE_MAX_HEIGHT: u32 = 1024;
pub const NATIVE_MAX_GPU_BUFFER_BYTES: u64 = 16 * 1024 * 1024;
// Constant evaluation checks this derived structural bound for overflow.
pub const NATIVE_MAX_LUT_ENTRIES: usize =
    MAX_COMMANDS * (NATIVE_MAX_WIDTH + NATIVE_MAX_HEIGHT) as usize;
pub const CONVERSION_ROW_ALIGNMENT: u64 = 256;
pub const CONVERSION_UNIFORM_BYTES: u64 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Probe,
    Native,
}
impl Profile {
    pub const fn max_width(self) -> u32 {
        match self {
            Self::Probe => MAX_WIDTH,
            Self::Native => NATIVE_MAX_WIDTH,
        }
    }
    pub const fn max_height(self) -> u32 {
        match self {
            Self::Probe => MAX_HEIGHT,
            Self::Native => NATIVE_MAX_HEIGHT,
        }
    }
    pub const fn max_gpu_buffer_bytes(self) -> u64 {
        match self {
            Self::Probe => MAX_GPU_BUFFER_BYTES,
            Self::Native => NATIVE_MAX_GPU_BUFFER_BYTES,
        }
    }
    pub const fn max_lut_entries(self) -> usize {
        match self {
            Self::Probe => MAX_LUT_ENTRIES,
            Self::Native => NATIVE_MAX_LUT_ENTRIES,
        }
    }
    pub fn validate_viewport(self, width: u32, height: u32) -> Result<()> {
        if width == 0 || height == 0 || width > self.max_width() || height > self.max_height() {
            Err("viewport budget".into())
        } else {
            Ok(())
        }
    }
    /// Native plans reserve a full padded 8x8 conversion dispatch. The core
    /// raster encoder does not execute that separately owned conversion pass.
    pub fn conversion_invocations(self, width: u32, height: u32) -> Result<u64> {
        self.validate_viewport(width, height)?;
        match self {
            Self::Probe => Ok(0),
            Self::Native => padded_invocations(width, height),
        }
    }
    /// Native conversion destination (256-byte rows) plus its 16-byte uniform.
    /// Probe conversion is absent; its original packed readback is charged by
    /// the planner separately. This reserve does not include native readback.
    pub fn conversion_buffer_bytes(self, width: u32, height: u32) -> Result<u64> {
        self.validate_viewport(width, height)?;
        match self {
            Self::Probe => Ok(0),
            Self::Native => padded_conversion_bytes(width, height),
        }
    }
    pub(crate) fn validate_buffer_bytes(self, bytes: u64) -> Result<()> {
        if bytes > self.max_gpu_buffer_bytes() {
            Err("GPU buffer budget".into())
        } else {
            Ok(())
        }
    }
}

pub(crate) fn padded_invocations(width: u32, height: u32) -> Result<u64> {
    u64::from(width.div_ceil(8))
        .checked_mul(u64::from(height.div_ceil(8)))
        .and_then(|n| n.checked_mul(64))
        .ok_or_else(|| "invocation overflow".into())
}
pub(crate) fn padded_conversion_bytes(width: u32, height: u32) -> Result<u64> {
    let row = u64::from(width)
        .checked_mul(4)
        .and_then(|n| n.checked_add(CONVERSION_ROW_ALIGNMENT - 1))
        .ok_or("GPU buffer overflow")?
        / CONVERSION_ROW_ALIGNMENT
        * CONVERSION_ROW_ALIGNMENT;
    row.checked_mul(u64::from(height))
        .and_then(|n| n.checked_add(CONVERSION_UNIFORM_BYTES))
        .ok_or_else(|| "GPU buffer overflow".into())
}
pub(crate) fn add_work(total: u64, added: u64) -> Result<u64> {
    let total = total.checked_add(added).ok_or("invocation overflow")?;
    if total > MAX_INVOCATIONS {
        Err("GPU invocation budget".into())
    } else {
        Ok(total)
    }
}
