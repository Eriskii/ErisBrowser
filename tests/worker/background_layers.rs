//! Layer identities and cached pixels across real confined-worker renders.
use super::*;
use eris::graphics::{Canvas, Color, Fonts};

#[cfg(feature = "raster-bridge")]
#[allow(dead_code)]
#[path = "../support/vulkan_backgrounds.rs"]
mod witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_background_layers_keep_cached_images_and_none_slots_after_mutation() {
    let fixture = Fixture::new(
        r#"<!doctype html><style>html,body{margin:0;background:white}#board{position:absolute;left:0;top:0;width:6px;height:2px;background-color:#0f0;background-image:url(red.png),none,url(blue.png);background-size:2px 2px;background-position:0 0,2px 0,4px 0;background-repeat:no-repeat}</style><div id=board onclick="this.style.backgroundImage='url(blue.png),url(missing.png),url(red.png)'"></div>"#,
    );
    fs::write(
        fixture.directory.join("red.png"),
        include_bytes!("../reftests/url-background-assets/red.png"),
    )
    .unwrap();
    fs::write(
        fixture.directory.join("blue.png"),
        include_bytes!("../reftests/url-background-assets/blue.png"),
    )
    .unwrap();
    let mut client = fixture.spawn(true, 317);
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
        assert_eq!(snapshot.generation, 317);
        assert_eq!(snapshot.processed_edit_sequence, 0);
        board = snapshot.document.query_selector("#board");
        assert!(board.is_some());
        assert_eq!(snapshot.layout.hit_test(1.0, 1.0), board);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .any(|s| s.starts_with("Page process "))
        );
        assert!(
            snapshot
                .diagnostics
                .iter()
                .any(|s| s.starts_with("Resource broker "))
        );
        let missing = snapshot
            .diagnostics
            .iter()
            .filter(|s| s.starts_with("background image unavailable"))
            .count();
        assert_eq!(missing, usize::from(changed));
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|s| s.starts_with("Page process ")
                    || s.starts_with("Resource broker ")
                    || s.starts_with("background image unavailable"))
        );
        let row = if changed {
            [0x0000ff, 0x0000ff, 0x00ff00, 0x00ff00, 0xff0000, 0xff0000]
        } else {
            [0xff0000, 0xff0000, 0x00ff00, 0x00ff00, 0x0000ff, 0x0000ff]
        };
        let mut expected = vec![0xffffff; width as usize * 240];
        expected[..6].copy_from_slice(&row);
        expected[width as usize..width as usize + 6].copy_from_slice(&row);
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
            assert_eq!(native.stats().bridge.referenced_sources, 2);
            assert_eq!(witness::packed_pixels(native.plan()), expected);
        }
        if index == 0 {
            fs::remove_file(fixture.directory.join("red.png")).unwrap();
            fs::remove_file(fixture.directory.join("blue.png")).unwrap();
        }
    }
}
