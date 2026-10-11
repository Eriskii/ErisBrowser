//! The host checkpoint contract through real confined Page/broker IPC.
use super::*;
use eris::{
    dom::NodeId,
    graphics::{Canvas, Color, Fonts},
    page::TaskState,
};
#[path = "../support/microtasks.rs"]
mod oracle;
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;

fn check(
    client: &mut WorkerClient,
    generation: u64,
    red_x: u32,
    title: &str,
    state: TaskState,
    failed: bool,
) -> (NodeId, NodeId) {
    let fonts = Fonts::new();
    let mut nodes = None;
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
        let target = snapshot.document.query_selector("#box").unwrap();
        let link = snapshot.document.query_selector("#link").unwrap();
        nodes = Some((target, link));
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
            Some(target)
        );
        assert_eq!(snapshot.layout.hit_test(21.0, 3.0), Some(link));
        assert_eq!(snapshot.title, title);
        assert_eq!(snapshot.task_state, state);
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
        let errors: Vec<_> = snapshot
            .diagnostics
            .iter()
            .filter(|s| !s.starts_with("Page process ") && !s.starts_with("Resource broker "))
            .collect();
        if failed {
            assert_eq!(errors.len(), 1, "{:?}", snapshot.diagnostics);
            assert!(errors[0].starts_with("script:"));
            assert!(errors[0].contains("script instruction limit exceeded"));
        } else {
            assert!(errors.is_empty(), "{:?}", snapshot.diagnostics);
        }
    }
    nodes.unwrap()
}
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_microtask_script_listener_and_default_action_order_has_literal_pixels() {
    let fixture = Fixture::new(oracle::NORMAL);
    // Both destinations exist so broker validation cannot mask the href oracle.
    for target in ["wrong.html", "chosen.html"] {
        fs::write(fixture.directory.join(target), b"").unwrap();
    }
    let mut client = fixture.spawn(true, 401);
    load(&mut client, &fixture.navigation);
    let (target, link) = check(&mut client, 401, 6, oracle::INITIAL, TaskState::Idle, false);
    exchange_empty(&mut client, WorkerCommand::Click { node: target });
    check(
        &mut client,
        401,
        10,
        oracle::CLICKED,
        TaskState::Idle,
        false,
    );
    let reply = client
        .exchange(WorkerCommand::Click { node: link }, || false)
        .unwrap();
    assert!(reply.snapshot.is_none());
    let navigation = reply.navigation.unwrap();
    let expected = Url::parse(&fixture.navigation.address)
        .unwrap()
        .join("chosen.html")
        .unwrap();
    assert_eq!(navigation.address, expected.to_string());
    assert!(navigation.form_body.is_none());
    check(&mut client, 401, 10, oracle::LINK, TaskState::Idle, false);
}
#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_microtask_failure_never_resumes_in_render_and_fresh_or_disabled_realms_are_independent()
{
    let fixture = Fixture::new(oracle::FAILURE);
    let mut client = fixture.spawn(true, 402);
    load(&mut client, &fixture.navigation);
    let (target, _) = check(&mut client, 402, 6, "queued", TaskState::Suspended, true);
    for _ in 0..2 {
        exchange_empty(&mut client, WorkerCommand::RunTasks);
        check(&mut client, 402, 6, "queued", TaskState::Suspended, true);
    }
    exchange_empty(&mut client, WorkerCommand::Click { node: target });
    exchange_empty(&mut client, WorkerCommand::RunTasks);
    check(&mut client, 402, 6, "clicked", TaskState::Suspended, true);
    drop(client);
    let fresh_fixture = Fixture::new(oracle::NORMAL);
    let mut fresh = fresh_fixture.spawn(true, 403);
    load(&mut fresh, &fresh_fixture.navigation);
    check(&mut fresh, 403, 6, oracle::INITIAL, TaskState::Idle, false);
    drop(fresh);
    let mut disabled = fixture.spawn(false, 404);
    load(&mut disabled, &fixture.navigation);
    let (target, _) = check(&mut disabled, 404, 2, "before", TaskState::Idle, false);
    exchange_empty(&mut disabled, WorkerCommand::Click { node: target });
    exchange_empty(&mut disabled, WorkerCommand::RunTasks);
    check(&mut disabled, 404, 2, "before", TaskState::Idle, false);
}
