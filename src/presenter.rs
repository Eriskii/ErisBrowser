//! Native presentation of a completed CPU frame. Painting stays in `Canvas`.
use eris::graphics::Canvas;
use std::{num::NonZeroU32, sync::Arc};
use winit::window::Window;

/// Immutable, tightly packed native pixel words, in row-major `0x00RRGGBB`
/// order. The high byte is unused, not transparent alpha. No pixel conversion
/// or retained framebuffer is needed for the current software presenter.
pub(crate) struct CpuFrame<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    pixels: &'a [u32],
}

impl<'a> CpuFrame<'a> {
    pub(crate) fn from_canvas(canvas: &'a Canvas) -> Result<Self, String> {
        // Canvas fields are public; validate at this boundary even though its
        // constructor normally enforces the same 16-Mpixel/8192-axis limits.
        let width = NonZeroU32::new(canvas.width).ok_or("frame width is zero")?;
        let height = NonZeroU32::new(canvas.height).ok_or("frame height is zero")?;
        let count = u64::from(width.get()) * u64::from(height.get());
        if width.get() > 8192 || height.get() > 8192 || count > 16_777_216 {
            return Err("frame exceeds 16 megapixels or 8192 pixels per axis".into());
        }
        let count = usize::try_from(count).map_err(|_| "frame pixel count overflow")?;
        if canvas.pixels.len() != count {
            return Err("frame dimensions do not match pixel length".into());
        }
        Ok(Self {
            width,
            height,
            pixels: &canvas.pixels,
        })
    }

    fn copy_to(&self, destination: &mut [u32]) -> Result<(), String> {
        if destination.len() != self.pixels.len() {
            return Err("presentation buffer length does not match completed frame".into());
        }
        destination.copy_from_slice(self.pixels);
        Ok(())
    }
}

/// The sole native surface owner. All surface creation, resizing, buffer
/// acquisition and submission stay together on the existing UI thread.
pub(crate) struct SoftwarePresenter {
    surface: softbuffer::Surface<Arc<Window>, Arc<Window>>,
}

impl SoftwarePresenter {
    pub(crate) fn new(window: Arc<Window>) -> Result<Self, String> {
        let context = softbuffer::Context::new(window.clone()).map_err(|e| e.to_string())?;
        let surface = softbuffer::Surface::new(&context, window).map_err(|e| e.to_string())?;
        Ok(Self { surface })
    }

    pub(crate) fn present(&mut self, frame: CpuFrame<'_>) -> Result<(), String> {
        self.surface
            .resize(frame.width, frame.height)
            .map_err(|e| e.to_string())?;
        let mut buffer = self.surface.buffer_mut().map_err(|e| e.to_string())?;
        frame.copy_to(&mut buffer)?;
        buffer.present().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_canvas_dimensions_and_lengths_are_rejected() {
        let mut canvas = Canvas::new(3, 2).unwrap();
        canvas.width = 0;
        assert!(CpuFrame::from_canvas(&canvas).is_err());
        canvas.width = 3;
        canvas.height = 0;
        assert!(CpuFrame::from_canvas(&canvas).is_err());
        canvas.height = 2;
        canvas.pixels.pop();
        assert!(CpuFrame::from_canvas(&canvas).is_err());
        canvas.pixels.extend([0, 0]);
        assert!(CpuFrame::from_canvas(&canvas).is_err());
    }

    #[test]
    fn oversized_frame_metadata_is_rejected_without_allocating_pixels() {
        let mut canvas = Canvas::new(1, 1).unwrap();
        for (width, height) in [(8193, 1), (1, 8193), (4097, 4096), (u32::MAX, u32::MAX)] {
            canvas.width = width;
            canvas.height = height;
            assert!(
                CpuFrame::from_canvas(&canvas)
                    .err()
                    .unwrap()
                    .contains("exceeds")
            );
        }
    }

    #[test]
    fn odd_width_completed_frame_is_borrowed_and_copied_without_conversion() {
        let mut canvas = Canvas::new(3, 2).unwrap();
        canvas.pixels = vec![0xff0000, 0x00ff00, 0x0000ff, 0, 0xffffff, 0xa5123456];
        let frame = CpuFrame::from_canvas(&canvas).unwrap();
        assert_eq!(frame.pixels.as_ptr(), canvas.pixels.as_ptr());
        assert_eq!((frame.width.get(), frame.height.get()), (3, 2));
        let mut destination = [0; 6];
        frame.copy_to(&mut destination).unwrap();
        assert_eq!(destination, canvas.pixels.as_slice());
    }

    #[test]
    fn mismatched_presentation_buffers_are_rejected_without_partial_copy() {
        let canvas = Canvas::new(3, 2).unwrap();
        let frame = CpuFrame::from_canvas(&canvas).unwrap();
        for length in [0, 5, 7] {
            let mut destination = vec![0x123456; length];
            assert!(frame.copy_to(&mut destination).is_err());
            assert!(destination.iter().all(|pixel| *pixel == 0x123456));
        }
    }
}
