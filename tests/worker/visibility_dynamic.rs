use super::*;
use eris::{
    dom::NodeId,
    graphics::{Canvas, Color, Fonts},
    page::TaskState,
};
#[path = "../support/visibility_dynamic.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;

fn check(client: &mut WorkerClient, generation: u64, shown: bool, title: &str) -> NodeId {
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
        let toggle = snapshot.document.query_selector("#toggle").unwrap();
        target = Some(toggle);
        let expected = oracle::expected(width, 240, shown);
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
            oracle::assert_pixels(&witness::packed_pixels(scene.plan()), &expected, width);
        }
        let box_node = snapshot.document.query_selector("#box").unwrap();
        assert_eq!(snapshot.layout.hit_test(1.0, 1.0), Some(toggle));
        assert_eq!(
            snapshot.layout.hit_test(1.0, 9.0),
            snapshot.document.query_selector("#after")
        );
        assert_eq!(snapshot.layout.hit_test(5.0, 5.0) == Some(box_node), shown);
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
fn confined_visibility_dynamic_hiding_preserves_space_and_visible_child_event_bubbling() {
    let fixture = Fixture::new(oracle::HTML);
    for scripts in [false, true] {
        let generation = if scripts { 682 } else { 681 };
        let mut client = fixture.spawn(scripts, generation);
        load(&mut client, &fixture.navigation);
        let toggle = check(&mut client, generation, false, "hidden");
        exchange_empty(&mut client, WorkerCommand::Click { node: toggle });
        check(
            &mut client,
            generation,
            scripts,
            if scripts { "shownB" } else { "hidden" },
        );
        exchange_empty(&mut client, WorkerCommand::Click { node: toggle });
        check(
            &mut client,
            generation,
            false,
            if scripts { "hiddenB" } else { "hidden" },
        );
        exchange_empty(&mut client, WorkerCommand::RunTasks);
        check(
            &mut client,
            generation,
            false,
            if scripts { "hiddenB" } else { "hidden" },
        );
    }
}
