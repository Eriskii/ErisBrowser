//! Mixed CSS-wide shorthand behavior through a real confined Page worker.
use super::*;
use eris::graphics::{Canvas, Color, Fonts};
#[path = "../support/cssom_wide_getter.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_mixed_wide_margin_preserves_uniform_control_and_empty_remove_result() {
    let fixture = Fixture::new(oracle::HTML);
    let mut client = fixture.spawn(true, 322);
    load(&mut client, &fixture.navigation);
    let fonts = Fonts::new();
    let mut mixed = None;
    for (stage, state) in oracle::STATES.iter().enumerate() {
        if stage > 0 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: mixed.unwrap(),
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
            assert_eq!(snapshot.generation, 322);
            assert_eq!(snapshot.processed_edit_sequence, 0);
            mixed = snapshot.document.query_selector("#mixed");
            let uniform = snapshot.document.query_selector("#uniform");
            assert!(mixed.is_some() && uniform.is_some());
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
            let expected = oracle::expected(width, 240, state);
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
            oracle::assert_pixels(&canvas.pixels, &expected, width, stage);
            #[cfg(feature = "raster-bridge")]
            {
                use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
                let frame = eris_raster_core::Frame::new(width, 240, oracle::WHITE);
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
                oracle::assert_pixels(
                    &witness::packed_pixels(scene.plan()),
                    &expected,
                    width,
                    stage,
                );
            }
            for (node, &(x, y, _, _, _)) in [mixed, uniform].iter().zip(&state.rectangles) {
                assert_eq!(
                    snapshot.layout.hit_test(x as f32 + 1.0, y as f32 + 1.0),
                    *node
                );
            }
            // Pixel/IPC controls precede the observable getter/title assertion.
            assert_eq!(snapshot.title, state.title, "stage={stage}");
        }
    }
}
