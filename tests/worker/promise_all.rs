//! Promise.all ordering, literal pixels and host callbacks through a confined worker.
use super::*;
use eris::{
    dom::NodeId,
    graphics::{Canvas, Color, Fonts},
    page::TaskState,
};
#[path = "../support/promise_all.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;

fn check(client: &mut WorkerClient, generation: u64, red_x: u32, title: &str) -> NodeId {
    let fonts = Fonts::new();
    let mut target = None;
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
        assert_eq!(snapshot.generation, generation);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        let node = snapshot.document.query_selector("#red").unwrap();
        target = Some(node);
        let expected = oracle::expected(width, 240, red_x);
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
        oracle::assert_pixels(&canvas.pixels, &expected, width);
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
            oracle::assert_pixels(&witness::packed_pixels(scene.plan()), &expected, width);
        }
        assert_eq!(
            snapshot.layout.hit_test(red_x as f32 + 1.0, 3.0),
            Some(node)
        );
        assert_eq!(
            snapshot.layout.hit_test(21.0, 3.0),
            snapshot.document.query_selector("#blue")
        );
        assert_eq!(snapshot.title, title);
        assert_eq!(snapshot.task_state, TaskState::Idle);
        for prefix in ["Page process ", "Resource broker "] {
            assert_eq!(
                snapshot
                    .diagnostics
                    .iter()
                    .filter(|s| s.starts_with(prefix))
                    .count(),
                1
            );
        }
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|s| s.starts_with("Page process ") || s.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
    }
    target.unwrap()
}
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_promise_all_collects_in_order_and_drains_before_the_next_host_listener() {
    let fixture = Fixture::new(oracle::HTML);
    let mut disabled = fixture.spawn(false, 641);
    load(&mut disabled, &fixture.navigation);
    let target = check(&mut disabled, 641, 2, "base");
    exchange_empty(&mut disabled, WorkerCommand::Click { node: target });
    exchange_empty(&mut disabled, WorkerCommand::RunTasks);
    check(&mut disabled, 641, 2, "base");
    drop(disabled);

    let mut client = fixture.spawn(true, 642);
    load(&mut client, &fixture.navigation);
    let target = check(&mut client, 642, 12, oracle::INITIAL);
    exchange_empty(&mut client, WorkerCommand::RunTasks);
    check(&mut client, 642, 12, oracle::INITIAL);
    exchange_empty(&mut client, WorkerCommand::Click { node: target });
    check(&mut client, 642, 16, oracle::CLICKED);
    exchange_empty(&mut client, WorkerCommand::RunTasks);
    check(&mut client, 642, 16, oracle::CLICKED);
}
