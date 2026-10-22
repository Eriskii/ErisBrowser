use super::*;
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::TaskState,
};
#[path = "../support/supports_plain.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_supports_plain_condition_matches_literal_pixels() {
    const GENERATION: u64 = 744;
    let fixture = Fixture::new(oracle::HTML);
    let mut client = fixture.spawn(false, GENERATION);
    load(&mut client, &fixture.navigation);
    let reply = client
        .exchange(
            WorkerCommand::Render {
                width: oracle::WIDTH as f32,
                height: oracle::HEIGHT as f32,
            },
            || false,
        )
        .unwrap();
    assert!(reply.navigation.is_none());
    let snapshot = reply.snapshot.expect("render returns a snapshot");
    assert_eq!(snapshot.generation, GENERATION);
    assert_eq!(snapshot.processed_edit_sequence, 0);
    assert_eq!(snapshot.task_state, TaskState::Idle);
    for prefix in ["Page process ", "Resource broker "] {
        assert_eq!(
            snapshot
                .diagnostics
                .iter()
                .filter(|message| message.starts_with(prefix))
                .count(),
            1
        );
    }
    assert!(
        snapshot
            .diagnostics
            .iter()
            .all(|message| message.starts_with("Page process ")
                || message.starts_with("Resource broker ")),
        "{:?}",
        snapshot.diagnostics
    );
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(oracle::WIDTH, oracle::HEIGHT).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(
        &snapshot.layout.commands,
        &fonts,
        &snapshot.images,
        0.0,
        0.0,
    );
    assert!(!canvas.exhausted());
    oracle::assert_pixels(&canvas.pixels);
    #[cfg(feature = "raster-bridge")]
    {
        use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
        let frame = eris_raster_core::Frame::new(oracle::WIDTH, oracle::HEIGHT, 0xffffff);
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
        oracle::assert_pixels(&witness::packed_pixels(scene.plan()));
    }
}
