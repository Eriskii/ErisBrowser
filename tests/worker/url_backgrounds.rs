use super::*;
use eris::graphics::{Canvas, Color, DrawCommand, Fonts};

fn image_keys(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::BackgroundImage { key, .. } => Some(key.as_str()),
            _ => None,
        })
        .collect()
}

fn top_pixels(snapshot: &Snapshot) -> Vec<u32> {
    let mut canvas = Canvas::new(320, 240).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(
        &snapshot.layout.commands,
        &Fonts::new(),
        &snapshot.images,
        0.0,
        0.0,
    );
    assert!(!canvas.exhausted());
    canvas.pixels[..8].to_vec()
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn background_preparation_pixels_and_failure_diagnostics_arrive_in_same_snapshot() {
    let fixture = Fixture::new(
        r#"<!doctype html><style>html,body{margin:0}button{position:absolute;left:40px;top:40px}#box{position:absolute;left:0;top:0;width:8px;height:4px;background-color:yellow;background-image:url(red.png)}</style><div id=box></div><button id=green onclick="document.getElementById('box').style.backgroundImage='url(green.png)'">green</button><button id=missing onclick="document.getElementById('box').style.backgroundImage='url(missing.png)'">missing</button>"#,
    );
    fs::write(
        fixture.directory.join("red.png"),
        include_bytes!("../reftests/url-background-assets/red.png"),
    )
    .unwrap();
    fs::write(
        fixture.directory.join("green.png"),
        include_bytes!("../reftests/url-background-assets/green.png"),
    )
    .unwrap();
    let mut client = fixture.spawn(true, 217);
    load(&mut client, &fixture.navigation);
    let first = render(&mut client);
    assert_eq!(image_keys(&first).len(), 1);
    assert!(first.images.contains_key(image_keys(&first)[0]));
    assert_eq!(top_pixels(&first), [0xff0000; 8]);
    fs::remove_file(fixture.directory.join("red.png")).unwrap();
    let warm = render(&mut client);
    assert_eq!(top_pixels(&warm), [0xff0000; 8]);
    let green = first.document.query_selector("#green").unwrap();
    exchange_empty(&mut client, WorkerCommand::Click { node: green });
    let changed = render(&mut client);
    assert_eq!(image_keys(&changed).len(), 1);
    assert_ne!(image_keys(&first), image_keys(&changed));
    assert_eq!(top_pixels(&changed), [0x00ff00; 8]);
    let missing = first.document.query_selector("#missing").unwrap();
    exchange_empty(&mut client, WorkerCommand::Click { node: missing });
    let failed = render(&mut client);
    assert!(image_keys(&failed).is_empty());
    assert_eq!(top_pixels(&failed), [0xffff00; 8]);
    assert!(
        failed
            .diagnostics
            .iter()
            .any(|s| s.starts_with("background image unavailable"))
    );
    let again = render(&mut client);
    assert_eq!(again.diagnostics, failed.diagnostics);
    assert_eq!(again.generation, 217);
}
