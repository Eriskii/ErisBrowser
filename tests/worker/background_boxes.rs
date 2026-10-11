//! The same literal principal boxes after real confined-worker tag13 transport.
use super::*;
#[path = "../support/background_boxes.rs"]
mod witness;

fn run(paint: witness::Paint, generation: u64) {
    let fixture = Fixture::new(&paint.html());
    if paint == witness::Paint::Image {
        fs::write(fixture.directory.join("pattern.png"), witness::PNG).unwrap();
    }
    let mut client = fixture.spawn(true, generation);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous: Option<witness::Identity> = None;
    let mut removed = false;
    for changed in [false, true] {
        if changed {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: previous.as_ref().unwrap().click,
                },
            );
        }
        for width in [32, 20, 32] {
            let reply = client
                .exchange(
                    WorkerCommand::Render {
                        width: width as f32,
                        height: witness::HEIGHT as f32,
                    },
                    || false,
                )
                .unwrap();
            assert!(reply.navigation.is_none());
            let snapshot = reply.snapshot.unwrap();
            assert_eq!(snapshot.generation, generation);
            assert_eq!(snapshot.processed_edit_sequence, 0);
            assert!(
                snapshot
                    .diagnostics
                    .iter()
                    .all(|message| message.starts_with("Page process ")
                        || message.starts_with("Resource broker ")),
                "{:?}",
                snapshot.diagnostics
            );
            previous = Some(witness::check(
                &snapshot.document,
                &snapshot.layout,
                &snapshot.images,
                &fonts,
                witness::Case {
                    paint,
                    width,
                    changed,
                },
                previous.as_ref(),
            ));
            if paint == witness::Paint::Image && !removed {
                fs::remove_file(fixture.directory.join("pattern.png")).unwrap();
                removed = true;
            }
        }
    }
    drop(client);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_gradient_origin_clip_mutation_and_resize_match_literal_frames() {
    run(witness::Paint::Gradient, 315);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_prepared_url_origin_clip_mutation_and_resize_match_literal_frames() {
    run(witness::Paint::Image, 316);
}
