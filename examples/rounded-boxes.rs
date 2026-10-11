//! Render the rounded-box foundation directly; CSS layout integration is separate.
use eris::graphics::box_decoration::{BoxDecoration, BoxPaint, RoundedBox};
use eris::graphics::gradient::{GradientColor, GradientDirection, GradientStop, LinearGradient};
use eris::graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore, RasterImage, Rect};
use std::{process::ExitCode, sync::Arc};

fn render(path: &std::path::Path) -> Result<(), String> {
    let mut canvas = Canvas::new(480, 160)?;
    canvas.clear(Color::rgb(240, 242, 247));
    let mut images = ImageStore::new();
    images.insert(
        "checker".into(),
        Arc::new(RasterImage {
            width: 2,
            height: 2,
            rgba: vec![
                25, 100, 230, 255, 250, 190, 30, 255, 250, 190, 30, 255, 25, 100, 230, 255,
            ],
        }),
    );
    let gradient = Arc::new(LinearGradient {
        direction: GradientDirection::Angle(90.0),
        stops: vec![
            GradientStop {
                color: GradientColor::Color(Color::rgb(25, 100, 230)),
                position: None,
            },
            GradientStop {
                color: GradientColor::Color(Color::rgb(230, 45, 125)),
                position: None,
            },
        ]
        .into(),
    });
    let mut commands = Vec::new();
    for index in 0..4 {
        let x = 12.0 + index as f32 * 120.0;
        let shape = RoundedBox {
            rect: Rect {
                x,
                y: 24.0,
                width: 96.0,
                height: 112.0,
            },
            radii: [[32.0, 20.0], [16.0, 40.0], [36.0, 28.0], [8.0, 16.0]],
        }
        .normalized()
        .ok_or("invalid example shape")?;
        let tile = Rect {
            x,
            y: 24.0,
            width: 24.0,
            height: 24.0,
        };
        let paint = match index {
            0 | 1 => BoxPaint::Color(Color::rgb(25, 100, 230)),
            2 => BoxPaint::Image {
                tile,
                repeat: [true, true],
                key: "checker".into(),
            },
            _ => BoxPaint::LinearGradient {
                tile: shape.rect,
                repeat: [false, false],
                gradient: Arc::clone(&gradient),
                current_color: Color::BLACK,
                stop_scale: 1.0,
            },
        };
        commands.push(DrawCommand::BoxDecoration(Arc::new(BoxDecoration {
            shape,
            cutout: if index == 1 {
                shape.inset([8.0, 18.0, 24.0, 10.0])
            } else {
                None
            },
            paint,
        })));
    }
    canvas.paint(&commands, &Fonts::new(), &images, 0.0, 0.0);
    if canvas.exhausted() {
        return Err("example exceeded painter budget".into());
    }
    canvas.save(path)
}

fn main() -> ExitCode {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("usage: cargo run --example rounded-boxes -- OUTPUT.png");
        return ExitCode::FAILURE;
    };
    match render(std::path::Path::new(&path)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
