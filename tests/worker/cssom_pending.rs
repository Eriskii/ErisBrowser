//! Authoritative pending style survives confined rendering and later edits.
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;
use super::*;
use eris::graphics::{Canvas, Color, Fonts};

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_pending_shorthand_state_survives_clicks_and_ui_snapshot_projection() {
    let fixture = Fixture::new(include_str!("../fixtures/cssom-pending.html"));
    let mut client = fixture.spawn(true, 320);
    load(&mut client, &fixture.navigation);
    let fonts = Fonts::new();
    let states = [
        (12, 8, 6, 3, "ready"),
        (17, 8, 11, 3, "9px|important|important|5"),
        (21, 16, 11, 7, "9px|important|important|5"),
        (15, 16, 11, 7, "9px|important||4"),
        (15, 4, 11, 2, "9px|||2"),
    ];
    let mut target = None;
    for (index, (outer_width, outer_height, ink_x, ink_y, title)) in states.into_iter().enumerate()
    {
        if index > 0 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: target.unwrap(),
                },
            );
        }
        for width in [320u32, 300] {
            let reply = client
                .exchange(
                    WorkerCommand::Render {
                        width: width as f32,
                        height: 240.0,
                    },
                    || false,
                )
                .unwrap();
            assert!(reply.navigation.is_none());
            let snapshot = reply.snapshot.unwrap();
            assert_eq!(snapshot.generation, 320);
            assert_eq!(snapshot.processed_edit_sequence, 0);
            assert_eq!(snapshot.title, title, "stage={index}");
            target = snapshot.document.query_selector("#target");
            let ink = snapshot.document.query_selector("#ink");
            assert!(target.is_some() && ink.is_some());
            assert_eq!(
                snapshot
                    .layout
                    .hit_test(ink_x as f32 + 1.0, ink_y as f32 + 1.0),
                ink
            );
            for prefix in ["Page process ", "Resource broker "] {
                assert!(snapshot.diagnostics.iter().any(|s| s.starts_with(prefix)));
            }
            assert!(
                snapshot
                    .diagnostics
                    .iter()
                    .all(|s| s.starts_with("Page process ") || s.starts_with("Resource broker ")),
                "{:?}",
                snapshot.diagnostics
            );
            let mut expected = vec![0xffffff; width as usize * 240];
            for (x, y, w, h, color) in [
                (2, 2, outer_width, outer_height, 0xff0000),
                (ink_x, ink_y, 6, 4, 0x0000ff),
            ] {
                for row in y..y + h {
                    for column in x..x + w {
                        expected[(row * width + column) as usize] = color;
                    }
                }
            }
            let mut canvas = Canvas::new(width, 240).unwrap();
            canvas.clear(Color::WHITE);
            canvas.paint(
                &snapshot.layout.commands,
                &fonts,
                &snapshot.images,
                0.0,
                0.0,
            );
            assert!(!canvas.exhausted());
            if let Some(i) = canvas
                .pixels
                .iter()
                .zip(&expected)
                .position(|(a, b)| a != b)
            {
                panic!(
                    "stage={index} pixel ({},{}) actual={:06x} expected={:06x}",
                    i % width as usize,
                    i / width as usize,
                    canvas.pixels[i],
                    expected[i]
                );
            }
            #[cfg(feature = "raster-bridge")]
            {
                use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
                let frame = eris_raster_core::Frame::new(width, 240, 0xffffff);
                let scene = plan_native_scene(
                    frame,
                    &[NativePhase {
                        frame,
                        commands: &snapshot.layout.commands,
                    }],
                    &snapshot.images,
                    &fonts,
                )
                .unwrap();
                assert_eq!(witness::packed_pixels(scene.plan()), expected);
            }
        }
    }
}
