//! Bounded native composition. Unit-zoom page commands remain borrowed; zoomed
//! pages own a bounded scaled copy. Each phase has its own fixed-position caller
//! clip. Preparation owns no GPU objects.
use super::*;
use eris::graphics::{
    ImageStore,
    raster_bridge::native::{self, NativePhase, NativeScenePlan},
    raster_bridge::{MAX_COMMAND_KEY_BYTES, MAX_KEY_BYTES},
    text_masks::{MAX_FRAME_SCALARS, MAX_FRAME_TEXT_BYTES, MAX_RUN_TEXT_BYTES},
};
use eris_raster_core::{Frame, Profile, Rect as CoreRect};
use std::borrow::Cow;
use winit::dpi::PhysicalSize;

const UI_COMMANDS: usize = 32;
const UI_TEXT_BYTES: usize = 16 * 1024;
const UI_RUN_BYTES: usize = 4096;
const PREPARATION_BYTES: usize = 16 * 1024 * 1024;
// Includes the owned page's vector header, all command slots and both bounded
// string arenas. Borrowed snapshot/image/font storage and allocator overhead
// remain excluded, as in the adapter's preparation accounting.
const ZOOM_COPY_BYTES: usize = size_of::<Vec<DrawCommand>>()
    + eris_raster_core::MAX_COMMANDS * size_of::<DrawCommand>()
    + MAX_FRAME_TEXT_BYTES
    + MAX_COMMAND_KEY_BYTES;

fn needs_scaled_page(zoom: f32) -> Result<bool, String> {
    if !zoom.is_finite() || !(0.5..=3.0).contains(&zoom) {
        return Err("native scene zoom outside supported range".into());
    }
    // Keep the CPU painter's near-unit branch, including its f32 subtraction.
    Ok((zoom - 1.0).abs() >= 0.001)
}

fn admit_preparation(peak: usize, scaled: bool) -> Result<(), String> {
    peak.checked_add(if scaled { ZOOM_COPY_BYTES } else { 0 })
        .filter(|&bytes| bytes <= PREPARATION_BYTES)
        .ok_or_else(|| "native scene preparation capacity budget".to_string())?;
    Ok(())
}

fn copy_payload(text: &str, capacity: &mut usize, limit: usize) -> Result<String, String> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(text.len())
        .map_err(|_| "native zoom payload allocation")?;
    *capacity = capacity
        .checked_add(owned.capacity())
        .filter(|&bytes| bytes <= limit)
        .ok_or("native zoom payload capacity budget")?;
    owned.push_str(text);
    Ok(owned)
}

fn scaled_page(commands: &[DrawCommand], zoom: f32) -> Result<Cow<'_, [DrawCommand]>, String> {
    let scaled = needs_scaled_page(zoom)?;
    if commands.len() > eris_raster_core::MAX_COMMANDS {
        return Err("native page command budget".into());
    }
    if !scaled {
        return Ok(Cow::Borrowed(commands));
    }
    // Check all borrowed payloads before allocating their copies. These are the
    // existing adapter limits; scaling cannot hide an unsupported source.
    let (mut text_bytes, mut scalars, mut key_bytes) = (0usize, 0usize, 0usize);
    for command in commands {
        match command {
            DrawCommand::Text { text, .. } => {
                text_bytes = text_bytes
                    .checked_add(text.len())
                    .filter(|&n| n <= MAX_FRAME_TEXT_BYTES && text.len() <= MAX_RUN_TEXT_BYTES)
                    .ok_or("native zoom text budget")?;
                scalars = scalars
                    .checked_add(text.chars().count())
                    .filter(|&n| n <= MAX_FRAME_SCALARS)
                    .ok_or("native zoom scalar budget")?;
            }
            DrawCommand::Image { key, .. } => {
                key_bytes = key_bytes
                    .checked_add(key.len())
                    .filter(|&n| n <= MAX_COMMAND_KEY_BYTES && key.len() <= MAX_KEY_BYTES)
                    .ok_or("native zoom image key budget")?;
            }
            _ => {}
        }
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(commands.len())
        .map_err(|_| "native zoom command allocation")?;
    if result.capacity() > eris_raster_core::MAX_COMMANDS {
        return Err("native zoom command capacity budget".into());
    }
    let (mut text_capacity, mut key_capacity) = (0usize, 0usize);
    let rect = |r: &Rect| Rect {
        x: r.x * zoom,
        y: r.y * zoom,
        width: r.width * zoom,
        height: r.height * zoom,
    };
    for command in commands {
        // Match scaled_command's arithmetic; only payload cloning differs.
        let scaled = match command {
            DrawCommand::PushClip { rect: r } => DrawCommand::PushClip { rect: rect(r) },
            DrawCommand::PopClip => DrawCommand::PopClip,
            DrawCommand::PushFixed => DrawCommand::PushFixed,
            DrawCommand::PopFixed => DrawCommand::PopFixed,
            DrawCommand::PushOpacity { opacity } => DrawCommand::PushOpacity { opacity: *opacity },
            DrawCommand::PopOpacity => DrawCommand::PopOpacity,
            DrawCommand::Rect {
                rect: r,
                color,
                radius,
            } => DrawCommand::Rect {
                rect: rect(r),
                color: *color,
                radius: radius * zoom,
            },
            DrawCommand::Text {
                x,
                y,
                text,
                size,
                color,
                bold,
                italic,
                monospace,
            } => DrawCommand::Text {
                x: x * zoom,
                y: y * zoom,
                text: copy_payload(text, &mut text_capacity, MAX_FRAME_TEXT_BYTES)?,
                size: size * zoom,
                color: *color,
                bold: *bold,
                italic: *italic,
                monospace: *monospace,
            },
            DrawCommand::Image { rect: r, key } => DrawCommand::Image {
                rect: rect(r),
                key: copy_payload(key, &mut key_capacity, MAX_COMMAND_KEY_BYTES)?,
            },
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => DrawCommand::Line {
                x1: x1 * zoom,
                y1: y1 * zoom,
                x2: x2 * zoom,
                y2: y2 * zoom,
                color: *color,
                width: width * zoom,
            },
        };
        result.push(scaled);
    }
    Ok(Cow::Owned(result))
}

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
    page: Cow<'a, [DrawCommand]>,
    overlays: Commands,
    chrome: Commands,
    selection: Selection,
}
impl Scene<'_> {
    fn phases(&self) -> [NativePhase<'_>; 3] {
        [
            NativePhase {
                frame: self.page_frame,
                commands: &self.page,
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
    needs_scaled_page(browser.zoom)?;
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
    let page = scaled_page(page, browser.zoom)?;
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
        return Err(if browser.zoom == 1.0 {
            "native scene awaits a current loaded page".into()
        } else {
            format!(
                "native scene awaits a current loaded page; zoom_bits={}",
                browser.zoom.to_bits()
            )
        });
    }
    let target = Frame::new(size.width, size.height, 0xffffff);
    let peak =
        native::preparation_peak_bytes(target, include_reference).map_err(|e| e.to_string())?;
    // The scaled scene coexists with adapter preparation. It is dropped before
    // paint_canvas constructs the optional CPU reference's own scaled commands,
    // so one copy allowance covers both stages (the reference canvas is already
    // charged by preparation_peak_bytes). The submitted packet retains neither.
    admit_preparation(peak, needs_scaled_page(browser.zoom)?)?;
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
        assert!(matches!(scene.page, Cow::Borrowed(_)));
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
    fn zoomed_complete_scenes_match_cpu_scale_before_scroll_and_fixed_clips() {
        for (zoom, fixed_focus) in [(0.75, false), (1.25, true)] {
            let mut browser = super::super::tests::editing_browser("<input id=focus>");
            browser.zoom = zoom;
            browser.scroll = 37.25;
            browser.address = "eris:zoom".into();
            browser.status = "Zoom".into();
            browser.address_focused = true;
            browser.selection = Selection {
                anchor: 2,
                caret: 6,
            };
            let snapshot = browser.snapshot.as_mut().unwrap();
            snapshot.title = "Z".into();
            let node = snapshot.document.query_selector("#focus").unwrap();
            snapshot.layout.hit_regions = vec![eris::layout::HitRegion {
                node,
                action: HitAction::Node,
                rect: Rect {
                    x: 27.5,
                    y: 83.25,
                    width: 66.5,
                    height: 23.75,
                },
                fixed: fixed_focus,
            }];
            browser.focused = Some(node);
            snapshot.layout.content_height = 1600.0;
            snapshot.images.insert(
                "tile".into(),
                Arc::new(eris::graphics::RasterImage {
                    width: 2,
                    height: 2,
                    rgba: vec![
                        255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 0, 33, 66, 99, 255,
                    ],
                }),
            );
            snapshot.layout.commands = vec![
                DrawCommand::Rect {
                    rect: Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 250.0,
                        height: 220.0,
                    },
                    color: Color::rgb(240, 225, 210),
                    radius: 0.0,
                },
                DrawCommand::PushClip {
                    rect: Rect {
                        x: 13.25,
                        y: 45.5,
                        width: 170.5,
                        height: 95.75,
                    },
                },
                DrawCommand::Rect {
                    rect: Rect {
                        x: 10.25,
                        y: 43.5,
                        width: 180.5,
                        height: 75.5,
                    },
                    color: Color::rgba(31, 71, 111, 128),
                    radius: 7.25,
                },
                DrawCommand::Text {
                    x: 20.25,
                    y: 55.75,
                    text: "Zoom".into(),
                    size: 14.5,
                    color: Color::rgb(13, 23, 43),
                    bold: true,
                    italic: true,
                    monospace: false,
                },
                DrawCommand::Image {
                    rect: Rect {
                        x: 25.5,
                        y: 80.25,
                        width: 16.5,
                        height: 11.75,
                    },
                    key: "tile".into(),
                },
                DrawCommand::Line {
                    x1: 120.25,
                    y1: 92.5,
                    x2: 157.75,
                    y2: 104.0,
                    color: Color::rgb(91, 19, 141),
                    width: 1.75,
                },
                DrawCommand::PushFixed,
                // Fixed content escapes the document clip and scroll offset.
                DrawCommand::Rect {
                    rect: Rect {
                        x: 225.5,
                        y: 5.75,
                        width: 21.5,
                        height: 17.25,
                    },
                    color: Color::rgb(12, 160, 80),
                    radius: 2.75,
                },
                DrawCommand::PopFixed,
                DrawCommand::Rect {
                    rect: Rect {
                        x: 45.25,
                        y: 115.5,
                        width: 190.5,
                        height: 18.25,
                    },
                    color: Color::rgb(140, 70, 10),
                    radius: 0.0,
                },
                DrawCommand::PopClip,
            ];
            let size = PhysicalSize::new(1180, 880);
            let scene = build(&browser, size).unwrap();
            assert!(matches!(scene.page, Cow::Owned(_)));
            assert_eq!(scene.page_frame.document_offset, (0.0, TOOLBAR - 37.25));
            assert_eq!(scene.page_frame.viewport_offset, (0.0, TOOLBAR));
            let actual = paint_scene(&browser, &scene);
            let expected = browser.paint_canvas(size).unwrap();
            assert_eq!(actual.pixels, expected.pixels, "zoom={zoom}");
            // The complete comparison deliberately exercises many overlapping
            // operations. Its conservative CPU-work admission must still refuse
            // the whole scene under the unchanged adapter budget.
            assert_eq!(
                prepare(&browser, size, true).err().unwrap(),
                "cpu-paint-budget"
            );
        }
    }

    #[test]
    fn minimal_loaded_page_admits_native_zoom_with_cpu_reference() {
        for zoom in [0.75, 1.25] {
            let mut browser =
                super::super::tests::editing_browser("<title>Zoom</title><p>Page</p>");
            browser.zoom = zoom;
            browser.address = "eris:zoom".into();
            browser.status = "Zoom".into();
            let size = PhysicalSize::new(1180, 880);
            let scene = build(&browser, size).unwrap();
            assert!(matches!(scene.page, Cow::Owned(_)));
            let actual = paint_scene(&browser, &scene);
            let expected = browser.paint_canvas(size).unwrap();
            assert_eq!(actual.pixels, expected.pixels, "zoom={zoom}");
            let prepared = prepare(&browser, size, true).unwrap();
            assert_eq!(prepared.plan.stats().phases, 3);
            assert!(prepared.plan.plan().has_glyphs());
        }
    }

    #[test]
    fn zoom_range_and_near_unit_copy_boundary_preserve_source() {
        let commands = [DrawCommand::Text {
            x: 7.25,
            y: 13.5,
            text: "source".into(),
            size: 12.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            monospace: false,
        }];
        for zoom in [0.9995, 1.0, 1.0005] {
            let page = scaled_page(&commands, zoom).unwrap();
            assert!(matches!(page, Cow::Borrowed(_)));
            assert!(std::ptr::eq(page.as_ref(), commands.as_slice()));
        }
        for zoom in [0.5, 0.9985, 1.0015, 3.0] {
            let page = scaled_page(&commands, zoom).unwrap();
            assert!(matches!(page, Cow::Owned(_)));
            let DrawCommand::Text { text, .. } = &page[0] else {
                panic!()
            };
            assert_eq!(text, "source");
        }
        let DrawCommand::Text {
            x, y, text, size, ..
        } = &commands[0]
        else {
            panic!()
        };
        assert_eq!((*x, *y, text.as_str(), *size), (7.25, 13.5, "source", 12.0));
        for zoom in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            -1.0,
            0.0,
            0.4999,
            3.0001,
        ] {
            assert!(
                scaled_page(&commands, zoom)
                    .unwrap_err()
                    .contains("zoom outside")
            );
        }
    }

    #[test]
    fn zoom_copy_obeys_existing_command_text_and_image_key_caps() {
        let image = |key| DrawCommand::Image {
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            key,
        };
        let mut keys =
            vec![image("k".repeat(MAX_KEY_BYTES)); MAX_COMMAND_KEY_BYTES / MAX_KEY_BYTES];
        assert!(scaled_page(&keys, 1.25).is_ok());
        keys.push(image("k".into()));
        assert!(
            scaled_page(&keys, 1.25)
                .unwrap_err()
                .contains("image key budget")
        );
        assert!(scaled_page(&[image("k".repeat(MAX_KEY_BYTES + 1))], 1.25).is_err());
        let text = |value| DrawCommand::Text {
            x: 0.0,
            y: 0.0,
            text: value,
            size: 12.0,
            color: Color::BLACK,
            bold: false,
            italic: false,
            monospace: false,
        };
        assert!(scaled_page(&[text("é".repeat(MAX_FRAME_SCALARS))], 1.25).is_ok());
        assert!(
            scaled_page(&[text("é".repeat(MAX_FRAME_SCALARS + 1))], 1.25)
                .unwrap_err()
                .contains("scalar budget")
        );
        assert!(
            scaled_page(&[text("a".repeat(MAX_RUN_TEXT_BYTES + 1))], 1.25)
                .unwrap_err()
                .contains("text budget")
        );
        assert!(
            scaled_page(
                &vec![DrawCommand::PopClip; eris_raster_core::MAX_COMMANDS],
                1.25
            )
            .is_ok()
        );
        assert!(
            scaled_page(
                &vec![DrawCommand::PopClip; eris_raster_core::MAX_COMMANDS + 1],
                1.25
            )
            .unwrap_err()
            .contains("command budget")
        );
    }

    #[test]
    fn zoom_copy_allowance_is_checked_with_the_complete_preparation_peak() {
        assert!(admit_preparation(PREPARATION_BYTES, false).is_ok());
        assert!(admit_preparation(PREPARATION_BYTES + 1, false).is_err());
        assert!(admit_preparation(PREPARATION_BYTES - ZOOM_COPY_BYTES, true).is_ok());
        assert!(admit_preparation(PREPARATION_BYTES - ZOOM_COPY_BYTES + 1, true).is_err());
        assert!(admit_preparation(usize::MAX, true).is_err());
        for reference in [false, true] {
            let peak = native::preparation_peak_bytes(Frame::new(1280, 1024, 0xffffff), reference)
                .unwrap();
            assert!(admit_preparation(peak, true).is_ok());
        }
    }

    #[test]
    fn zoom_keeps_scope_and_geometry_refusals_and_reports_waiting_scale() {
        let mut browser = super::super::tests::editing_browser("Page");
        browser.zoom = 1.25;
        let size = PhysicalSize::new(1180, 880);
        browser.snapshot.as_mut().unwrap().layout.commands = vec![
            DrawCommand::PushOpacity { opacity: 0.5 },
            DrawCommand::PopOpacity,
        ];
        assert!(
            prepare(&browser, size, false)
                .err()
                .unwrap()
                .contains("unsupported-opacity")
        );
        browser.snapshot.as_mut().unwrap().layout.commands = vec![DrawCommand::Rect {
            rect: Rect {
                x: f32::MAX,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            color: Color::BLACK,
            radius: 0.0,
        }];
        assert!(prepare(&browser, size, false).is_err());
        browser.loading = true;
        browser.zoom = 1.1;
        assert_eq!(
            prepare(&browser, size, false).err().unwrap(),
            "native scene awaits a current loaded page; zoom_bits=1066192077"
        );
        browser.zoom = 1.0;
        assert_eq!(
            prepare(&browser, size, false).err().unwrap(),
            "native scene awaits a current loaded page"
        );
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
        browser.zoom = f32::NAN;
        assert!(
            build(&browser, size)
                .err()
                .unwrap()
                .contains("zoom outside")
        );
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
