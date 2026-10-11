//! Real Page inputs for the acquired-window fixture; no device or window is opened.
#![cfg(feature = "vulkan-raster")]

use eris::{
    graphics::{
        Canvas, Color, DrawCommand, Fonts, Rect,
        box_decoration::{BoxDecoration, BoxPaint, RoundedBox},
        gradient::{GradientColor, GradientDirection, GradientPosition, GradientStop},
        raster_bridge::{
            FallbackKind,
            native::{NativePhase, plan_native_scene},
        },
    },
    layout::LayoutResult,
    page::Page,
};
use eris_raster_core::{Frame, Profile, Rect as CoreRect};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use url::Url;

const ADMITTED: &str = include_str!("../tools/native-background-fixtures/admitted.html");
const ROUNDED: &str = include_str!("../tools/native-background-fixtures/rounded.html");
const PNG: &[u8] = include_bytes!("../tools/native-background-fixtures/two-pixels.png");
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 880;
const PAGE_HEIGHT: f32 = 779.0;
const PAGE_TOP: f32 = 76.0;
const KEY: &str = "eris-css-background:0";
const RGBA: [u8; 32] = [
    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 255, 255, 255, 255, 0, 255, 255, 255, 255,
    0, 255, 0, 0, 0, 255, 255, 128, 0, 255,
];
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    address: String,
}
impl Fixture {
    fn new(rounded: bool) -> Self {
        for _ in 0..16 {
            let directory = std::env::temp_dir().join(format!(
                "eris-native-background-window-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&directory) {
                Ok(()) => {
                    let address = Url::from_file_path(directory.join("admitted.html"))
                        .unwrap()
                        .to_string();
                    let fixture = Self { directory, address };
                    fs::write(
                        fixture.directory.join("admitted.html"),
                        if rounded { ROUNDED } else { ADMITTED },
                    )
                    .unwrap();
                    fs::write(fixture.directory.join("two-pixels.png"), PNG).unwrap();
                    return fixture;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("create owned fixture: {error}"),
            }
        }
        panic!("fixture directory collision bound");
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn decoration(command: &DrawCommand) -> &BoxDecoration {
    match command {
        DrawCommand::BoxDecoration(value) => value,
        other => panic!("expected selected background box, got {other:?}"),
    }
}

fn check_layout(page: &Page, layout: &LayoutResult, rounded: bool) {
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert_eq!(page.title(), "Native background window");
    assert_eq!(layout.content_height, PAGE_HEIGHT);
    assert_eq!(
        layout.commands.len(),
        4,
        "no unexpected page paint or scopes"
    );
    match &layout.commands[0] {
        DrawCommand::Rect {
            rect: value,
            color,
            radius,
        } => {
            assert_eq!(*value, rect(0.0, 0.0, 1280.0, 779.0));
            assert_eq!(*color, Color::WHITE);
            assert_eq!(*radius, 0.0);
        }
        other => panic!("expected the propagated white canvas, got {other:?}"),
    }
    for (index, x) in [(1, 10.0), (2, 26.0), (3, 42.0)] {
        let value = decoration(&layout.commands[index]);
        let radius = if rounded && index == 2 { 1.0 } else { 0.0 };
        assert_eq!(
            value.shape,
            RoundedBox {
                rect: rect(x, 10.0, 4.0, 2.0),
                radii: [[radius; 2]; 4],
            }
        );
        assert_eq!(value.cutout, None);
    }
    match &decoration(&layout.commands[1]).paint {
        BoxPaint::Color(color) => assert_eq!(*color, Color::rgb(0, 255, 0)),
        _ => panic!("Color board lost its selected content clip"),
    }
    match &decoration(&layout.commands[2]).paint {
        BoxPaint::Image { tile, repeat, key } => {
            assert_eq!(*tile, rect(27.0, 11.0, 4.0, 2.0));
            assert_eq!(*repeat, [false, false]);
            assert_eq!(key, KEY);
        }
        _ => panic!("Image board lost its independent padding-box origin"),
    }
    match &decoration(&layout.commands[3]).paint {
        BoxPaint::LinearGradient {
            tile,
            repeat,
            gradient,
            current_color,
            stop_scale,
        } => {
            assert_eq!(*tile, rect(43.0, 11.0, 4.0, 2.0));
            assert_eq!(*repeat, [false, false]);
            assert_eq!(*current_color, Color::rgb(255, 0, 0));
            assert_eq!(*stop_scale, 1.0);
            assert_eq!(gradient.direction, GradientDirection::Angle(90.0));
            assert_eq!(
                gradient.stops.as_ref(),
                &[
                    GradientStop {
                        color: GradientColor::CurrentColor,
                        position: Some(GradientPosition {
                            px: 0.0,
                            percent: 50.0
                        }),
                    },
                    GradientStop {
                        color: GradientColor::Color(Color::rgb(0, 0, 255)),
                        position: Some(GradientPosition {
                            px: 0.0,
                            percent: 50.0
                        }),
                    },
                ]
            );
        }
        _ => panic!("Gradient board lost its independent padding-box origin"),
    }
    assert_eq!(page.images.len(), 1);
    let image = page
        .images
        .get(KEY)
        .expect("authenticated CSS background key");
    assert_eq!((image.width, image.height), (4, 2));
    assert_eq!(image.rgba, RGBA);
}

fn page_frame() -> Frame {
    Frame {
        caller_clip: CoreRect::new(0.0, PAGE_TOP, WIDTH as f32, PAGE_HEIGHT),
        document_offset: (0.0, PAGE_TOP),
        viewport_offset: (0.0, PAGE_TOP),
        ..Frame::new(WIDTH, HEIGHT, 0xffffff)
    }
}

fn check_software(page: &Page, layout: &LayoutResult, fonts: &Fonts, rounded: bool) {
    // Literal physical pixels, derived before renderer execution. Browser chrome
    // is omitted in this Page-only test; the acquired check compares real chrome.
    let mut expected = vec![0xffffff; WIDTH as usize * HEIGHT as usize];
    for y in [86, 87] {
        for x in 10..14 {
            expected[y * WIDTH as usize + x] = 0x00ff00;
        }
    }
    for (x, color) in [
        (27, 0xff0000),
        (28, 0x00ff00),
        (29, 0x0000ff),
        (43, 0xff0000),
        (44, 0xff0000),
        (45, 0x0000ff),
    ] {
        expected[87 * WIDTH as usize + x] = color;
    }
    if rounded {
        // Radius1 at the image's bottom-right pixel:4+4+3+2=13/16
        // fixed samples, floor(255*13/16)=207. Blue over white gives48,48,255.
        expected[87 * WIDTH as usize + 29] = 0x3030ff;
    }
    let mut canvas = Canvas::new(WIDTH, HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.set_clip(rect(0.0, PAGE_TOP, WIDTH as f32, PAGE_HEIGHT));
    canvas.paint_with_viewport(
        &layout.commands,
        fonts,
        &page.images,
        (0.0, PAGE_TOP),
        (0.0, PAGE_TOP),
    );
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels.len(), expected.len());
    if let Some(index) = canvas
        .pixels
        .iter()
        .zip(&expected)
        .position(|(a, b)| a != b)
    {
        panic!(
            "literal pixel ({},{}): actual={:06x} expected={:06x}",
            index % WIDTH as usize,
            index / WIDTH as usize,
            canvas.pixels[index],
            expected[index]
        );
    }
}

#[test]
fn loaded_three_background_boxes_preserve_literal_geometry_pixels_and_native_routes() {
    let fixture = Fixture::new(false);
    let mut page = Page::load(&fixture.address, false).unwrap();
    let fonts = Fonts::new();
    let layout = page.prepare_layout(WIDTH as f32, PAGE_HEIGHT, &fonts);
    check_layout(&page, &layout, false);
    check_software(&page, &layout, &fonts, false);
    let native = plan_native_scene(
        Frame::new(WIDTH, HEIGHT, 0xffffff),
        &[NativePhase {
            frame: page_frame(),
            commands: &layout.commands,
        }],
        &page.images,
        &fonts,
    )
    .expect("the frozen page phase must admit without changing Native limits");
    assert_eq!(native.plan().profile(), Profile::Native);
    let stats = native.stats();
    assert_eq!(
        (
            stats.box_rectangles,
            stats.box_masks,
            stats.box_background_images,
            stats.box_gradients,
            stats.box_loop_work
        ),
        (1, 0, 1, 1, 159)
    );
    assert_eq!(
        (
            stats.generated_gradient_sources,
            stats.generated_gradient_bytes,
            stats.gradient_evaluation_work,
            stats.gradient_preparation_work
        ),
        (1, 12, 39, 3)
    );
    assert_eq!(
        (
            stats.bridge.original_commands,
            stats.bridge.image_store_entries,
            stats.bridge.unique_sources,
            stats.bridge.referenced_sources,
            stats.bridge.total_rgba_bytes,
            stats.bridge.referenced_rgba_bytes
        ),
        (4, 1, 1, 1, 32, 32)
    );
    assert_eq!(stats.mask_sources, 0);
    assert_eq!(stats.coverage_bytes, 0);
    assert_eq!(native.plan().group_scratch_bytes(), 0);
}

#[test]
fn loaded_rounded_image_refuses_whole_native_scene_and_keeps_literal_software_edge() {
    let fixture = Fixture::new(true);
    let mut page = Page::load(&fixture.address, false).unwrap();
    let fonts = Fonts::new();
    let layout = page.prepare_layout(WIDTH as f32, PAGE_HEIGHT, &fonts);
    check_layout(&page, &layout, true);
    check_software(&page, &layout, &fonts, true);
    let error = plan_native_scene(
        Frame::new(WIDTH, HEIGHT, 0xffffff),
        &[NativePhase {
            frame: page_frame(),
            commands: &layout.commands,
        }],
        &page.images,
        &fonts,
    )
    .unwrap_err();
    assert_eq!(error.kind, FallbackKind::UnsupportedBoxDecoration);
    assert_eq!((error.phase_index, error.command_index), (Some(0), Some(2)));
    assert_eq!(
        error.to_string(),
        "unsupported-box-decoration at phase 0 original command 2"
    );
}
