//! Same frozen literal frames through the real confined worker snapshot codec.
use super::*;
#[path = "../support/inline_edges.rs"]
mod witness;

fn render_at(client: &mut WorkerClient, width: u32, generation: u64) -> Snapshot {
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
    snapshot
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_inline_padding_mutation_and_resize_match_direct_literal_frames() {
    let fixture = Fixture::new(witness::MUTATION_HTML);
    let mut client = fixture.spawn(true, 313);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous: Option<witness::Identity> = None;
    for changed in [false, true] {
        if changed {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: previous.as_ref().unwrap().red,
                },
            );
        }
        for width in [32, 10, 32] {
            let snapshot = render_at(&mut client, width, 313);
            previous = Some(witness::mutation(
                &snapshot.document,
                &snapshot.layout,
                &snapshot.images,
                &fonts,
                width,
                changed,
                previous.as_ref(),
            ));
        }
    }
    drop(client);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_inline_nested_and_trimmed_edges_match_direct_literal_frames() {
    let fixture = Fixture::new(witness::BOUNDARIES_HTML);
    let mut client = fixture.spawn(false, 314);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    for _ in 0..2 {
        let snapshot = render_at(&mut client, 24, 314);
        previous = Some(witness::boundaries(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            previous.as_ref(),
        ));
    }
    drop(client);
}
