//! Bounded native composition. Original page commands remain borrowed and each
//! phase has its own fixed-position caller clip. Preparation owns no GPU objects.
use super::*;
use eris::graphics::{
    ImageStore,
    raster_bridge::native::{self, NativePhase, NativeScenePlan},
};
use eris_raster_core::{Frame, Profile, Rect as CoreRect};
use std::borrow::Cow;
use winit::dpi::PhysicalSize;

const UI_COMMANDS: usize = 32;
const UI_TEXT_BYTES: usize = 16 * 1024;
const UI_RUN_BYTES: usize = 4096;
const PREPARATION_BYTES: usize = 16 * 1024 * 1024;

pub(super) struct Prepared {
    pub plan: NativeScenePlan,
    pub selection: Selection,
}

struct Commands {
    list: Vec<DrawCommand>,
    text_bytes: usize,
}
impl Commands {
    fn new() -> Result<Self, String> {
        let mut list = Vec::new();
        list.try_reserve_exact(UI_COMMANDS)
            .map_err(|_| "native UI allocation")?;
        Ok(Self {
            list,
            text_bytes: 0,
        })
    }
    fn push(&mut self, command: DrawCommand) -> Result<(), String> {
        if self.list.len() == UI_COMMANDS {
            return Err("native UI command budget".into());
        }
        self.list.push(command);
        Ok(())
    }
    fn rect(&mut self, rect: Rect, color: Color, radius: f32) -> Result<(), String> {
        self.push(DrawCommand::Rect {
            rect,
            color,
            radius,
        })
    }
    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        x: f32,
        y: f32,
        text: &str,
        size: f32,
        color: Color,
        bold: bool,
    ) -> Result<(), String> {
        if text.len() > UI_RUN_BYTES {
            return Err("native UI text run budget".into());
        }
        self.text_bytes = self
            .text_bytes
            .checked_add(text.len())
            .filter(|n| *n <= UI_TEXT_BYTES)
            .ok_or("native UI text budget")?;
        if self.list.len() == UI_COMMANDS {
            return Err("native UI command budget".into());
        }
        let mut owned = String::new();
        owned
            .try_reserve_exact(text.len())
            .map_err(|_| "native UI text allocation")?;
        owned.push_str(text);
        self.push(DrawCommand::Text {
            x,
            y,
            text: owned,
            size,
            color,
            bold,
            italic: false,
            monospace: false,
        })
    }
}

struct Scene<'a> {
    target: Frame,
    page_frame: Frame,
    overlay_frame: Frame,
    page: &'a [DrawCommand],
    overlays: Commands,
    chrome: Commands,
    selection: Selection,
}
impl Scene<'_> {
    fn phases(&self) -> [NativePhase<'_>; 3] {
        [
            NativePhase {
                frame: self.page_frame,
                commands: self.page,
            },
            NativePhase {
                frame: self.overlay_frame,
                commands: &self.overlays.list,
            },
            NativePhase {
                frame: self.target,
                commands: &self.chrome.list,
            },
        ]
    }
}

fn build(browser: &Browser, size: PhysicalSize<u32>) -> Result<Scene<'_>, String> {
    Profile::Native.validate_viewport(size.width, size.height)?;
    if !browser.zoom.is_finite() || (browser.zoom - 1.0).abs() >= 0.001 {
        return Err("native scene currently requires unit zoom".into());
    }
    if browser.worker_error.is_some() {
        return Err("native process-error overlay uses CPU painting".into());
    }
    let page = browser
        .snapshot
        .as_ref()
        .map_or(&[][..], |s| s.layout.commands.as_slice());
    if page.len() > eris_raster_core::MAX_COMMANDS {
        return Err("native page command budget".into());
    }
    let title = browser
        .snapshot
        .as_ref()
        .map(|s| s.title.as_str())
        .unwrap_or("A browser of its own");
    if [
        title,
        browser.address.as_str(),
        browser.status.as_str(),
        browser.input_value.as_str(),
    ]
    .iter()
    .any(|text| text.len() > UI_RUN_BYTES)
    {
        return Err("native UI input text budget".into());
    }
    let target = Frame::new(size.width, size.height, 0xffffff);
    let viewport = Rect {
        x: 0.0,
        y: TOOLBAR,
        width: size.width as f32,
        height: (size.height as f32 - TOOLBAR - STATUS).max(0.0),
    };
    let clip = CoreRect::new(viewport.x, viewport.y, viewport.width, viewport.height);
    let page_frame = Frame {
        caller_clip: clip,
        document_offset: (0.0, TOOLBAR - browser.scroll),
        viewport_offset: (0.0, TOOLBAR),
        ..target
    };
    let overlay_frame = Frame {
        caller_clip: clip,
        ..target
    };
    let mut overlays = Commands::new()?;
    if let Some(snapshot) = &browser.snapshot {
        if let Some(node) = browser.focused
            && let Some(hit) = snapshot
                .layout
                .hit_regions
                .iter()
                .find(|hit| hit.node == node)
        {
            // Preserve the existing scale-before-scroll order, including the
            // CPU near-unit branch's overlay multiplication.
            let r = Rect {
                x: hit.rect.x * browser.zoom,
                y: hit.rect.y * browser.zoom + TOOLBAR
                    - if hit.fixed { 0.0 } else { browser.scroll },
                width: hit.rect.width * browser.zoom,
                height: hit.rect.height * browser.zoom,
            };
            let blue = Color::rgb(67, 134, 231);
            overlays.rect(Rect { height: 2.0, ..r }, blue, 0.0)?;
            overlays.rect(
                Rect {
                    y: r.y + r.height - 2.0,
                    height: 2.0,
                    ..r
                },
                blue,
                0.0,
            )?;
            overlays.rect(Rect { width: 2.0, ..r }, blue, 0.0)?;
            overlays.rect(
                Rect {
                    x: r.x + r.width - 2.0,
                    width: 2.0,
                    ..r
                },
                blue,
                0.0,
            )?;
        }
        let total = snapshot.layout.content_height * browser.zoom;
        if total > viewport.height {
            let h = (viewport.height * viewport.height / total).max(30.0);
            let y = TOOLBAR
                + (viewport.height - h) * browser.scroll / (total - viewport.height).max(1.0);
            overlays.rect(
                Rect {
                    x: size.width as f32 - 7.0,
                    y,
                    width: 5.0,
                    height: h,
                },
                Color::rgba(123, 133, 154, 180),
                3.0,
            )?;
        }
    }
    let mut chrome = Commands::new()?;
    chrome.rect(
        Rect {
            x: 0.0,
            y: 0.0,
            width: size.width as f32,
            height: TOOLBAR,
        },
        Color::rgb(22, 26, 36),
        0.0,
    )?;
    chrome.text(17.0, 9.0, "ERIS", 11.0, Color::rgb(241, 191, 101), true)?;
    chrome.text(66.0, 8.0, title, 12.0, Color::rgb(167, 180, 203), false)?;
    for (x, label) in [(20.0, "←"), (60.0, "→"), (101.0, "↻"), (143.0, "⌂")] {
        chrome.text(x, 35.0, label, 23.0, Color::rgb(209, 216, 232), false)?;
    }
    let bar = Rect {
        x: 181.0,
        y: 31.0,
        width: (size.width as f32 - 196.0).max(20.0),
        height: 34.0,
    };
    chrome.rect(
        bar,
        if browser.address_focused {
            Color::rgb(57, 65, 87)
        } else {
            Color::rgb(38, 45, 61)
        },
        7.0,
    )?;
    chrome.push(DrawCommand::PushClip {
        rect: Rect {
            x: bar.x + 9.0,
            y: bar.y,
            width: bar.width - 18.0,
            height: bar.height,
        },
    })?;
    let mut selection = Selection {
        caret: browser.selection.caret,
        anchor: browser.selection.anchor,
    };
    selection.normalize(if browser.address_focused {
        &browser.address
    } else {
        &browser.input_value
    });
    let caret_width = if browser.address_focused {
        browser.fonts.measure(
            &browser.address[..selection.caret],
            14.0,
            false,
            false,
            false,
        )
    } else {
        0.0
    };
    let offset = if browser.address_focused {
        (caret_width - (bar.width - 28.0)).max(0.0)
    } else {
        0.0
    };
    let text_x = bar.x + 11.0 - offset;
    if browser.address_focused && !selection.collapsed() {
        let selected = selection.range();
        let before = browser.fonts.measure(
            &browser.address[..selected.start],
            14.0,
            false,
            false,
            false,
        );
        let selected_width =
            browser
                .fonts
                .measure(&browser.address[selected], 14.0, false, false, false);
        chrome.rect(
            Rect {
                x: text_x + before,
                y: bar.y + 7.0,
                width: selected_width,
                height: 20.0,
            },
            Color::rgb(55, 93, 155),
            0.0,
        )?;
    }
    chrome.text(
        text_x,
        bar.y + 7.0,
        &browser.address,
        14.0,
        Color::rgb(223, 230, 242),
        false,
    )?;
    if browser.address_focused && selection.collapsed() {
        chrome.rect(
            Rect {
                x: text_x + caret_width,
                y: bar.y + 8.0,
                width: 1.0,
                height: 18.0,
            },
            Color::WHITE,
            0.0,
        )?;
    }
    chrome.push(DrawCommand::PopClip)?;
    let status_y = size.height as f32 - STATUS;
    chrome.rect(
        Rect {
            x: 0.0,
            y: status_y,
            width: size.width as f32,
            height: STATUS,
        },
        Color::rgb(22, 26, 36),
        0.0,
    )?;
    let status = if browser.loading {
        Cow::Borrowed("Loading…")
    } else if !browser.status.is_empty() {
        Cow::Borrowed(browser.status.as_str())
    } else if let Some(snapshot) = &browser.snapshot {
        Cow::Owned(format!(
            "{} nodes · {} draw commands · {:.1} ms load · {} notices · {:.0}%",
            snapshot.document.nodes.len(),
            snapshot.layout.commands.len(),
            snapshot.load_ms,
            snapshot.diagnostics.len(),
            browser.zoom * 100.0
        ))
    } else {
        Cow::Borrowed("Ready")
    };
    chrome.text(
        12.0,
        status_y + 5.0,
        &status,
        11.0,
        Color::rgb(157, 173, 197),
        false,
    )?;
    Ok(Scene {
        target,
        page_frame,
        overlay_frame,
        page,
        overlays,
        chrome,
        selection,
    })
}

pub(super) fn prepare(
    browser: &Browser,
    size: PhysicalSize<u32>,
    include_reference: bool,
) -> Result<Prepared, String> {
    // Startup and navigation keep their complete CPU scene. In particular, a
    // verification quota cannot be consumed by chrome without a loaded page.
    if browser.loading
        || browser
            .snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot.generation != browser.generation())
    {
        return Err("native scene awaits a current loaded page".into());
    }
    let target = Frame::new(size.width, size.height, 0xffffff);
    let peak =
        native::preparation_peak_bytes(target, include_reference).map_err(|e| e.to_string())?;
    if peak > PREPARATION_BYTES {
        return Err("native scene preparation capacity budget".into());
    }
    let scene = build(browser, size)?;
    let empty = ImageStore::new();
    let images = browser
        .snapshot
        .as_ref()
        .map_or(&empty, |snapshot| &snapshot.images);
    let plan = native::plan_native_scene(scene.target, &scene.phases(), images, &browser.fonts)
        .map_err(|e| e.to_string())?;
    Ok(Prepared {
        plan,
        selection: scene.selection,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paint_scene(browser: &Browser, scene: &Scene<'_>) -> Canvas {
        let mut canvas = Canvas::new(scene.target.width, scene.target.height).unwrap();
        canvas.clear(Color::WHITE);
        let empty = ImageStore::new();
        let images = browser.snapshot.as_ref().map_or(&empty, |s| &s.images);
        for phase in scene.phases() {
            let c = phase.frame.caller_clip;
            canvas.set_clip(Rect {
                x: c.x,
                y: c.y,
                width: c.width,
                height: c.height,
            });
            canvas.paint_with_viewport(
                phase.commands,
                &browser.fonts,
                images,
                phase.frame.document_offset,
                phase.frame.viewport_offset,
            );
            assert!(!canvas.exhausted());
        }
        canvas
    }

    #[test]
    fn composed_page_fixed_overlay_and_selected_chrome_match_original_painter() {
        let mut browser = super::super::tests::editing_browser(
            "<style>body{margin:0}div{height:1200px;background:#abc}b{position:fixed;top:0;color:red}</style><div>Page</div><b>Fixed</b>",
        );
        browser.scroll = 37.25;
        browser.address_focused = true;
        browser.selection = Selection {
            anchor: 2,
            caret: 10,
        };
        browser.focused = browser
            .snapshot
            .as_ref()
            .and_then(|s| s.layout.hit_regions.first().map(|hit| hit.node));
        let size = PhysicalSize::new(1180, 880);
        let scene = build(&browser, size).unwrap();
        assert_eq!(scene.page_frame.caller_clip.y, TOOLBAR);
        assert_eq!(scene.page_frame.viewport_offset, (0.0, TOOLBAR));
        assert_eq!(scene.overlay_frame.document_offset, (0.0, 0.0));
        let actual = paint_scene(&browser, &scene);
        let expected = browser.paint_canvas(size).unwrap();
        assert_eq!(actual.pixels, expected.pixels);
    }

    #[test]
    fn loading_caret_and_near_unit_overlay_math_match_original_painter() {
        let mut browser = super::super::tests::editing_browser("<input value='hi'>");
        browser.loading = true;
        browser.zoom = 1.0005;
        browser.address_focused = true;
        browser.selection = Selection {
            anchor: usize::MAX,
            caret: usize::MAX,
        };
        let size = PhysicalSize::new(1180, 880);
        let scene = build(&browser, size).unwrap();
        assert_eq!(
            browser.selection.caret,
            usize::MAX,
            "building a candidate cannot mutate UI state"
        );
        assert_eq!(scene.selection.caret, browser.address.len());
        let actual = paint_scene(&browser, &scene);
        let expected = browser.paint_canvas(size).unwrap();
        assert_eq!(actual.pixels, expected.pixels);
    }

    #[test]
    fn preparation_requires_a_current_page_then_admits_complete_desktop_scene() {
        let mut browser = super::super::tests::editing_browser("<title>GPU</title><p>Page</p>");
        let size = PhysicalSize::new(1180, 880);
        browser.loading = true;
        assert!(prepare(&browser, size, true).is_err());
        browser.loading = false;
        browser.snapshot.as_mut().unwrap().generation = 2;
        assert!(prepare(&browser, size, true).is_err());
        browser.snapshot.as_mut().unwrap().generation = 1;
        let prepared = prepare(&browser, size, true).unwrap();
        assert_eq!(prepared.plan.stats().phases, 3);
        assert!(prepared.plan.stats().rounded_masks > 0);
        assert!(prepared.plan.plan().has_glyphs());
        browser.snapshot = None;
        assert!(prepare(&browser, size, false).is_err());
    }

    #[test]
    fn unsupported_ui_conditions_refuse_without_mutating_selection() {
        let mut browser = super::super::tests::editing_browser("Page");
        let size = PhysicalSize::new(1180, 880);
        browser.selection.caret = usize::MAX;
        browser.zoom = 1.25;
        assert!(build(&browser, size).err().unwrap().contains("unit zoom"));
        browser.zoom = 1.0;
        browser.worker_error = Some("stopped".into());
        assert!(
            build(&browser, size)
                .err()
                .unwrap()
                .contains("process-error")
        );
        browser.worker_error = None;
        browser.address = "a".repeat(UI_RUN_BYTES + 1);
        assert!(build(&browser, size).err().unwrap().contains("text budget"));
        assert_eq!(browser.selection.caret, usize::MAX);
    }
}
