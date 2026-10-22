use super::*;
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::TaskState,
};
#[path = "../support/pointer_events_dynamic.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;

fn check(
    client: &mut WorkerClient,
    generation: u64,
    main: &str,
    title: &str,
    baselines: &mut [Option<oracle::Geometry>; 2],
) -> oracle::Targets {
    let fonts = Fonts::new();
    let mut targets = None;
    for (index, width) in oracle::WIDTHS.into_iter().enumerate() {
        let reply = client
            .exchange(
                WorkerCommand::Render {
                    width: width as f32,
                    height: oracle::HEIGHT as f32,
                },
                || false,
            )
            .unwrap();
        assert!(reply.navigation.is_none());
        let snapshot = reply.snapshot.unwrap();
        assert_eq!(snapshot.generation, generation);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        let expected = oracle::expected(width);
        let mut canvas = Canvas::new(width, oracle::HEIGHT).unwrap();
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
            let frame = eris_raster_core::Frame::new(width, oracle::HEIGHT, 0xffffff);
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
        oracle::assert_geometry(&snapshot.layout, &mut baselines[index]);
        let current = oracle::assert_targets(&snapshot.document, &snapshot.layout, main);
        if let Some(previous) = targets {
            assert_eq!(current, previous, "target IDs changed with viewport width");
        }
        targets = Some(current);
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
    targets.unwrap()
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_pointer_events_dynamic_keeps_pixels_geometry_and_auto_child_event_ancestors() {
    let fixture = Fixture::new(oracle::HTML);
    for scripts in [false, true] {
        let generation = if scripts { 742 } else { 741 };
        let mut client = fixture.spawn(scripts, generation);
        load(&mut client, &fixture.navigation);
        let mut baselines = [None, None];
        let mut targets: Option<oracle::Targets> = None;
        for (action, main, title) in oracle::PHASES {
            let click = match action {
                oracle::Action::Initial => None,
                oracle::Action::ClickToggle => Some(targets.unwrap().toggle),
                oracle::Action::ClickMain => Some(targets.unwrap().main),
                oracle::Action::ClickChild => Some(targets.unwrap().child),
                oracle::Action::DrainTasks => {
                    exchange_empty(&mut client, WorkerCommand::RunTasks);
                    None
                }
            };
            if let Some(node) = click {
                exchange_empty(&mut client, WorkerCommand::Click { node });
            }
            targets = Some(check(
                &mut client,
                generation,
                if scripts { main } else { "over" },
                if scripts { title } else { "auto" },
                &mut baselines,
            ));
        }
    }
}
