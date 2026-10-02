//! Literal transcription of the independently frozen, unexecuted browser oracle.
//! Original definitions, assumptions and byte bindings live in browser-fixtures/oracle.
use crate::{Frame, Result};
use eris::graphics::{Color, DrawCommand, ImageStore, RasterImage, Rect};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub enum WorkerKind {
    OrderedRectangles,
    FixedChild,
    RepeatedImage,
    MissingImage,
    Rounded,
}

pub enum FixtureInput {
    Direct {
        commands: Vec<DrawCommand>,
        images: ImageStore,
    },
    /// Relative to the explicit, supervisor-bound --fixtures directory.
    Worker {
        html_file: &'static str,
        kind: WorkerKind,
    },
}

pub struct BrowserFixture {
    pub name: &'static str,
    pub frame: Frame,
    pub input: FixtureInput,
    pub expected: Vec<u32>,
    pub expected_fallback: Option<&'static str>,
}

// All callers below have literal counts bounded by the frozen oracle.
fn reserved<T>(count: usize) -> Result<Vec<T>> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| "fixture vector allocation")?;
    Ok(values)
}
fn owned(text: &str) -> Result<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(text.len())
        .map_err(|_| "fixture string allocation")?;
    value.push_str(text);
    Ok(value)
}
fn bytes(source: &[u8]) -> Result<Vec<u8>> {
    let mut value = reserved(source.len())?;
    value.extend_from_slice(source);
    Ok(value)
}
fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn color(rgba: [u8; 4]) -> Color {
    Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3])
}
fn expected(rgb: &[u8], width: u32, height: u32) -> Result<Vec<u32>> {
    let count =
        usize::try_from(u64::from(width) * u64::from(height)).map_err(|_| "fixture dimensions")?;
    if count.checked_mul(3) != Some(rgb.len()) {
        return Err("fixture literal RGB length".into());
    }
    let mut pixels = reserved(count)?;
    for pixel in rgb.as_chunks::<3>().0 {
        pixels.push((u32::from(pixel[0]) << 16) | (u32::from(pixel[1]) << 8) | u32::from(pixel[2]));
    }
    Ok(pixels)
}

pub fn fixtures() -> Result<Vec<BrowserFixture>> {
    let mut fixtures = reserved(16)?;
    // fixture: direct-fixed-escape-and-nested-restoration
    {
        let mut commands = reserved(15)?;
        commands.push(DrawCommand::Rect {
            rect: rect(1.0, 1.0, 4.0, 3.0),
            color: color([255, 0, 0, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PushClip {
            rect: rect(2.0, 1.0, 1.0, 2.0),
        });
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 6.0, 6.0),
            color: color([0, 255, 0, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PushFixed);
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 2.0, 1.0),
            color: color([0, 0, 255, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PushClip {
            rect: rect(1.0, 0.0, 1.0, 1.0),
        });
        commands.push(DrawCommand::PushFixed);
        commands.push(DrawCommand::Rect {
            rect: rect(2.0, 1.0, 2.0, 1.0),
            color: color([255, 255, 0, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PopFixed);
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 4.0, 4.0),
            color: color([255, 0, 255, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PopClip);
        commands.push(DrawCommand::PopFixed);
        commands.push(DrawCommand::Rect {
            rect: rect(2.0, 1.0, 1.0, 1.0),
            color: color([0, 255, 255, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PopClip);
        commands.push(DrawCommand::Rect {
            rect: rect(4.0, 3.0, 1.0, 1.0),
            color: color([0, 0, 0, 255]),
            radius: 0.0,
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-fixed-escape-and-nested-restoration",
            frame: Frame { width: 6, height: 4, clear: 0xffffff, caller_clip: crate::Rect::new(1.0, 0.0, 4.0, 4.0), document_offset: (0.0, -1.0), viewport_offset: (1.0, 1.0) },
            input: FixtureInput::Direct { commands, images },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/direct-fixed-escape-and-nested-restoration.rgb"), 6, 4)?,
            expected_fallback: None,
        });
    }
    // fixture: direct-translucent-fractional-clip-edges
    {
        let mut commands = reserved(1)?;
        commands.push(DrawCommand::Rect {
            rect: rect(-0.25, -0.25, 5.0, 3.0),
            color: color([255, 0, 0, 128]),
            radius: 0.0,
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-translucent-fractional-clip-edges",
            frame: Frame { width: 5, height: 3, clear: 0xffffff, caller_clip: crate::Rect::new(0.5, 0.5, 3.0, 1.5), document_offset: (0.0, 0.0), viewport_offset: (0.0, 0.0) },
            input: FixtureInput::Direct { commands, images },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/direct-translucent-fractional-clip-edges.rgb"), 5, 3)?,
            expected_fallback: None,
        });
    }
    // fixture: direct-repeated-key-alias-and-alpha
    {
        let mut commands = reserved(3)?;
        for _ in 0..2 {
            commands.push(DrawCommand::Image {
                rect: rect(0.0, 0.0, 4.0, 1.0),
                key: owned("shared.png")?,
            });
        }
        commands.push(DrawCommand::Image {
            rect: rect(0.0, 1.0, 4.0, 1.0),
            key: owned("alias.png")?,
        });
        let mut images = ImageStore::new();
        images
            .try_reserve(2)
            .map_err(|_| "fixture image map allocation")?;
        // backing_id: shared-0
        let raster0 = Arc::new(RasterImage {
            width: 2,
            height: 1,
            rgba: bytes(&[200, 20, 40, 128, 99, 77, 55, 0])?,
        });
        images.insert(owned("shared.png")?, Arc::clone(&raster0));
        images.insert(owned("alias.png")?, Arc::clone(&raster0));
        fixtures.push(BrowserFixture {
            name: "direct-repeated-key-alias-and-alpha",
            frame: Frame {
                width: 4,
                height: 2,
                clear: 0x204060,
                caller_clip: crate::Rect::new(0.0, 0.0, 4.0, 2.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Direct { commands, images },
            expected: expected(
                include_bytes!(
                    "../browser-fixtures/oracle/expected/direct-repeated-key-alias-and-alpha.rgb"
                ),
                4,
                2,
            )?,
            expected_fallback: None,
        });
    }
    // fixture: direct-missing-image-is-a-legal-noop
    {
        let mut commands = reserved(3)?;
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 2.0, 2.0),
            color: color([255, 0, 0, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::Image {
            rect: rect(0.0, 0.0, 4.0, 2.0),
            key: owned("missing.png")?,
        });
        commands.push(DrawCommand::Rect {
            rect: rect(3.0, 1.0, 1.0, 1.0),
            color: color([0, 0, 255, 255]),
            radius: 0.0,
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-missing-image-is-a-legal-noop",
            frame: Frame {
                width: 4,
                height: 2,
                clear: 0xffffff,
                caller_clip: crate::Rect::new(0.0, 0.0, 4.0, 2.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Direct { commands, images },
            expected: expected(
                include_bytes!(
                    "../browser-fixtures/oracle/expected/direct-missing-image-is-a-legal-noop.rgb"
                ),
                4,
                2,
            )?,
            expected_fallback: None,
        });
    }
    // fixture: direct-lines-are-bounding-rectangles
    {
        let mut commands = reserved(3)?;
        commands.push(DrawCommand::Line {
            x1: 4.0,
            y1: 2.0,
            x2: 1.0,
            y2: 0.0,
            width: 1.0,
            color: color([0, 0, 255, 255]),
        });
        commands.push(DrawCommand::Line {
            x1: 0.0,
            y1: 3.0,
            x2: 4.0,
            y2: 3.0,
            width: 1.0,
            color: color([255, 0, 0, 255]),
        });
        commands.push(DrawCommand::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            width: 2.0,
            color: color([0, 255, 0, 255]),
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-lines-are-bounding-rectangles",
            frame: Frame {
                width: 5,
                height: 4,
                clear: 0xffffff,
                caller_clip: crate::Rect::new(0.0, 0.0, 5.0, 4.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Direct { commands, images },
            expected: expected(
                include_bytes!(
                    "../browser-fixtures/oracle/expected/direct-lines-are-bounding-rectangles.rgb"
                ),
                5,
                4,
            )?,
            expected_fallback: None,
        });
    }
    // fixture: direct-hidden-text-refuses-whole-frame
    {
        let mut commands = reserved(5)?;
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 2.0, 2.0),
            color: color([255, 0, 0, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 0.0, 0.0),
        });
        commands.push(DrawCommand::Text {
            x: 0.0,
            y: 0.0,
            text: owned("hidden")?,
            size: 12.0,
            color: color([0, 0, 0, 255]),
            bold: false,
            italic: false,
            monospace: false,
        });
        commands.push(DrawCommand::PopClip);
        commands.push(DrawCommand::Rect {
            rect: rect(3.0, 1.0, 1.0, 1.0),
            color: color([0, 0, 255, 255]),
            radius: 0.0,
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-hidden-text-refuses-whole-frame",
            frame: Frame {
                width: 4,
                height: 2,
                clear: 0xffffff,
                caller_clip: crate::Rect::new(0.0, 0.0, 4.0, 2.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Direct { commands, images },
            expected: expected(
                include_bytes!(
                    "../browser-fixtures/oracle/expected/direct-hidden-text-refuses-whole-frame.rgb"
                ),
                4,
                2,
            )?,
            expected_fallback: Some("unsupported-text"),
        });
    }
    // fixture: direct-rounded-refuses-whole-frame
    {
        let mut commands = reserved(2)?;
        commands.push(DrawCommand::Rect {
            rect: rect(1.0, 0.0, 2.0, 2.0),
            color: color([0, 0, 255, 255]),
            radius: 1.0,
        });
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 2.0, 1.0, 1.0),
            color: color([255, 0, 0, 255]),
            radius: 0.0,
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-rounded-refuses-whole-frame",
            frame: Frame {
                width: 4,
                height: 3,
                clear: 0xffffff,
                caller_clip: crate::Rect::new(0.0, 0.0, 4.0, 3.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Direct { commands, images },
            expected: expected(
                include_bytes!(
                    "../browser-fixtures/oracle/expected/direct-rounded-refuses-whole-frame.rgb"
                ),
                4,
                3,
            )?,
            expected_fallback: Some("unsupported-rounded-rectangle"),
        });
    }
    // fixture: direct-unit-opacity-refuses-whole-frame
    {
        let mut commands = reserved(5)?;
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 3.0, 2.0),
            color: color([0, 0, 255, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PushOpacity { opacity: 1.0 });
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 2.0, 1.0),
            color: color([255, 0, 0, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PopOpacity);
        commands.push(DrawCommand::Rect {
            rect: rect(2.0, 1.0, 1.0, 1.0),
            color: color([0, 255, 0, 255]),
            radius: 0.0,
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-unit-opacity-refuses-whole-frame",
            frame: Frame { width: 3, height: 2, clear: 0xffffff, caller_clip: crate::Rect::new(0.0, 0.0, 3.0, 2.0), document_offset: (0.0, 0.0), viewport_offset: (0.0, 0.0) },
            input: FixtureInput::Direct { commands, images },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/direct-unit-opacity-refuses-whole-frame.rgb"), 3, 2)?,
            expected_fallback: Some("unsupported-opacity"),
        });
    }
    // fixture: direct-transparent-hidden-rounded-still-refuses
    {
        let mut commands = reserved(5)?;
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 1.0, 1.0),
            color: color([255, 0, 0, 255]),
            radius: 0.0,
        });
        commands.push(DrawCommand::PushClip {
            rect: rect(0.0, 0.0, 0.0, 0.0),
        });
        commands.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, 1.0, 1.0),
            color: color([99, 77, 55, 0]),
            radius: 3.0,
        });
        commands.push(DrawCommand::PopClip);
        commands.push(DrawCommand::Rect {
            rect: rect(2.0, 1.0, 1.0, 1.0),
            color: color([0, 0, 255, 255]),
            radius: 0.0,
        });
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-transparent-hidden-rounded-still-refuses",
            frame: Frame { width: 3, height: 2, clear: 0xffffff, caller_clip: crate::Rect::new(0.0, 0.0, 3.0, 2.0), document_offset: (0.0, 0.0), viewport_offset: (0.0, 0.0) },
            input: FixtureInput::Direct { commands, images },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/direct-transparent-hidden-rounded-still-refuses.rgb"), 3, 2)?,
            expected_fallback: Some("unsupported-rounded-rectangle"),
        });
    }
    // fixture: html-ordered-rectangles
    {
        fixtures.push(BrowserFixture {
            name: "html-ordered-rectangles",
            frame: Frame {
                width: 6,
                height: 4,
                clear: 0xffffff,
                caller_clip: crate::Rect::new(0.0, 0.0, 6.0, 4.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Worker {
                html_file: "html/html-ordered-rectangles.html",
                kind: WorkerKind::OrderedRectangles,
            },
            expected: expected(
                include_bytes!("../browser-fixtures/oracle/expected/html-ordered-rectangles.rgb"),
                6,
                4,
            )?,
            expected_fallback: None,
        });
    }
    // fixture: html-fixed-child-escapes-scrolling-clip
    {
        fixtures.push(BrowserFixture {
            name: "html-fixed-child-escapes-scrolling-clip",
            frame: Frame { width: 8, height: 4, clear: 0xffffff, caller_clip: crate::Rect::new(0.0, 0.0, 8.0, 4.0), document_offset: (0.0, -1.0), viewport_offset: (0.0, 0.0) },
            input: FixtureInput::Worker { html_file: "html/html-fixed-child-escapes-scrolling-clip.html", kind: WorkerKind::FixedChild },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/html-fixed-child-escapes-scrolling-clip.rgb"), 8, 4)?,
            expected_fallback: None,
        });
    }
    // fixture: html-repeated-decoded-image-key
    {
        fixtures.push(BrowserFixture {
            name: "html-repeated-decoded-image-key",
            frame: Frame {
                width: 6,
                height: 2,
                clear: 0xffffff,
                caller_clip: crate::Rect::new(0.0, 0.0, 6.0, 2.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Worker {
                html_file: "html/html-repeated-decoded-image-key.html",
                kind: WorkerKind::RepeatedImage,
            },
            expected: expected(
                include_bytes!(
                    "../browser-fixtures/oracle/expected/html-repeated-decoded-image-key.rgb"
                ),
                6,
                2,
            )?,
            expected_fallback: None,
        });
    }
    // fixture: html-unavailable-image-keeps-placeholder
    {
        fixtures.push(BrowserFixture {
            name: "html-unavailable-image-keeps-placeholder",
            frame: Frame { width: 4, height: 2, clear: 0xffffff, caller_clip: crate::Rect::new(0.0, 0.0, 4.0, 2.0), document_offset: (0.0, 0.0), viewport_offset: (0.0, 0.0) },
            input: FixtureInput::Worker { html_file: "html/html-unavailable-image-keeps-placeholder.html", kind: WorkerKind::MissingImage },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/html-unavailable-image-keeps-placeholder.rgb"), 4, 2)?,
            expected_fallback: None,
        });
    }
    // fixture: html-rounded-page-requires-full-cpu-fallback
    {
        fixtures.push(BrowserFixture {
            name: "html-rounded-page-requires-full-cpu-fallback",
            frame: Frame { width: 4, height: 3, clear: 0xffffff, caller_clip: crate::Rect::new(0.0, 0.0, 4.0, 3.0), document_offset: (0.0, 0.0), viewport_offset: (0.0, 0.0) },
            input: FixtureInput::Worker { html_file: "html/html-rounded-page-requires-full-cpu-fallback.html", kind: WorkerKind::Rounded },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/html-rounded-page-requires-full-cpu-fallback.rgb"), 4, 3)?,
            expected_fallback: Some("unsupported-rounded-rectangle"),
        });
    }
    // fixture: direct-command-cap-refuses-whole-frame
    {
        let mut commands = reserved(257)?;
        for _ in 0..257 {
            commands.push(DrawCommand::Rect {
                rect: rect(0.0, 0.0, 0.0, 0.0),
                color: color([0, 0, 0, 0]),
                radius: 0.0,
            });
        }
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-command-cap-refuses-whole-frame",
            frame: Frame {
                width: 1,
                height: 1,
                clear: 0xffffff,
                caller_clip: crate::Rect::new(0.0, 0.0, 1.0, 1.0),
                document_offset: (0.0, 0.0),
                viewport_offset: (0.0, 0.0),
            },
            input: FixtureInput::Direct { commands, images },
            expected: expected(
                include_bytes!(
                    "../browser-fixtures/oracle/expected/direct-command-cap-refuses-whole-frame.rgb"
                ),
                1,
                1,
            )?,
            expected_fallback: Some("command-budget"),
        });
    }
    // fixture: direct-scope-cap-refuses-balanced-whole-frame
    {
        let mut commands = reserved(66)?;
        for _ in 0..33 {
            commands.push(DrawCommand::PushFixed);
        }
        for _ in 0..33 {
            commands.push(DrawCommand::PopFixed);
        }
        let images = ImageStore::new();
        fixtures.push(BrowserFixture {
            name: "direct-scope-cap-refuses-balanced-whole-frame",
            frame: Frame { width: 1, height: 1, clear: 0xffffff, caller_clip: crate::Rect::new(0.0, 0.0, 1.0, 1.0), document_offset: (0.0, 0.0), viewport_offset: (0.0, 0.0) },
            input: FixtureInput::Direct { commands, images },
            expected: expected(include_bytes!("../browser-fixtures/oracle/expected/direct-scope-cap-refuses-balanced-whole-frame.rgb"), 1, 1)?,
            expected_fallback: Some("scope-budget"),
        });
    }
    Ok(fixtures)
}
