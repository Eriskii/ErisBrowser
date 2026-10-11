//! Originating row paint survives confined CSSOM mutation and snapshot resize.
#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;
use super::*;
use eris::graphics::{Canvas, Color, Fonts};

fn equal_pixels(actual: &[u32], expected: &[u32], width: u32) {
    assert_eq!(actual.len(), expected.len());
    if let Some(index) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "pixel ({},{}) actual={:06x} expected={:06x}",
            index % width as usize,
            index / width as usize,
            actual[index],
            expected[index]
        );
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_originating_row_background_mutation_preserves_spans_gaps_and_hits() {
    let fixture = Fixture::new(
        r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}table{position:absolute;left:2px;top:2px;box-sizing:content-box;width:14px;margin:0;padding:0;border:0;border-spacing:2px 1px;background:yellow}tr{height:2px}td{box-sizing:content-box;height:2px;margin:0;padding:0;border:0;background:transparent;vertical-align:top}</style><table><tr id=row style='background:red' onclick="this.style.backgroundColor='lime'"><td id=a rowspan=2></td><td></td></tr><tr style='background:blue'><td></td></tr></table>"#,
    );
    let mut client = fixture.spawn(true, 319);
    load(&mut client, &fixture.navigation);
    let fonts = Fonts::new();
    let mut row = None;
    for (index, (changed, width)) in [(false, 320u32), (false, 300), (true, 320), (true, 300)]
        .into_iter()
        .enumerate()
    {
        if index == 2 {
            exchange_empty(&mut client, WorkerCommand::Click { node: row.unwrap() });
        }
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
        assert_eq!(snapshot.generation, 319);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        row = snapshot.document.query_selector("#row");
        assert!(row.is_some());
        let spanning = snapshot.document.query_selector("#a");
        assert!(spanning.is_some());
        assert_eq!(snapshot.layout.hit_test(5.0, 6.0), spanning);
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
        let color = if changed { 0x00ff00 } else { 0xff0000 };
        let mut expected = vec![0xffffff; width as usize * 240];
        for (x, y, w, h, color) in [
            (2, 2, 14, 7, 0xffff00),
            (4, 3, 4, 5, color),
            (10, 3, 4, 2, color),
            (10, 6, 4, 2, 0x0000ff),
        ] {
            for y in y..y + h {
                for x in x..x + w {
                    expected[(y * width + x) as usize] = color;
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
        equal_pixels(&canvas.pixels, &expected, width);
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
            equal_pixels(&witness::packed_pixels(scene.plan()), &expected, width);
        }
    }
}
