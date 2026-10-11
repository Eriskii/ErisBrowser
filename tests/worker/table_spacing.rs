//! Literal table pixels survive CSSOM mutation and confined-worker resizing.
use super::*;
use eris::graphics::{Canvas, Color, Fonts};

#[cfg(feature = "raster-bridge")]
use super::raster_witness as witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_table_spacing_mutation_updates_pixels_and_hit_regions_after_resize() {
    let fixture = Fixture::new(
        r#"<!doctype html><style>html,body{margin:0;padding:0;background:white}table{position:absolute;left:2px;top:2px;box-sizing:content-box;width:20px;margin:0;padding:0;border:0;background:yellow;border-spacing:2px 3px}tr{background:transparent}td{margin:0;padding:0;border:0;height:4px}#c,#d{height:6px}#a{background:red}#b{background:blue}#c{background:lime}#d{background:black}</style><table id=board onclick="this.style.borderSpacing='0'"><tr><td id=a></td><td id=b></td></tr><tr><td id=c></td><td id=d></td></tr></table>"#,
    );
    let mut client = fixture.spawn(true, 318);
    load(&mut client, &fixture.navigation);
    let fonts = Fonts::new();
    let mut board = None;
    for (index, (changed, width)) in [(false, 320u32), (false, 300), (true, 320), (true, 300)]
        .into_iter()
        .enumerate()
    {
        if index == 2 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: board.unwrap(),
                },
            );
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
        assert_eq!(snapshot.generation, 318);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        board = snapshot.document.query_selector("#board");
        assert!(board.is_some());
        let hit = snapshot
            .document
            .query_selector(if changed { "#c" } else { "#a" });
        assert!(hit.is_some());
        assert_eq!(snapshot.layout.hit_test(5.0, 6.0), hit);
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
        let rectangles = if changed {
            [
                (2, 2, 20, 10, 0xffff00),
                (2, 2, 10, 4, 0xff0000),
                (12, 2, 10, 4, 0x0000ff),
                (2, 6, 10, 6, 0x00ff00),
                (12, 6, 10, 6, 0),
            ]
        } else {
            [
                (2, 2, 20, 19, 0xffff00),
                (4, 5, 7, 4, 0xff0000),
                (13, 5, 7, 4, 0x0000ff),
                (4, 12, 7, 6, 0x00ff00),
                (13, 12, 7, 6, 0),
            ]
        };
        let mut expected = vec![0xffffff; width as usize * 240];
        for (x, y, w, h, color) in rectangles {
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
        assert_eq!(canvas.pixels, expected, "changed={changed} width={width}");
        #[cfg(feature = "raster-bridge")]
        {
            use eris::graphics::raster_bridge::native::{NativePhase, plan_native_scene};
            let frame = eris_raster_core::Frame::new(width, 240, 0xffffff);
            let native = plan_native_scene(
                frame,
                &[NativePhase {
                    frame,
                    commands: &snapshot.layout.commands,
                }],
                &snapshot.images,
                &fonts,
            )
            .unwrap();
            assert_eq!(witness::packed_pixels(native.plan()), expected);
        }
    }
}
