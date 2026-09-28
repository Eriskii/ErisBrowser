use std::{fs, process::Command};

#[test]
fn headless_fragment_navigation_preserves_script_state_and_updates_document_url() {
    let directory = std::env::temp_dir().join(format!("eris-cli-fragments-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let document = directory.join("index.html");
    fs::write(
        &document,
        "<style>body{margin:0}#spacer{height:300px}#destination{height:100px;background:blue}</style><a id=first href='#caf%C3%A9'>Jump</a><a id=second href='#destination'>Again</a><p id=state>before</p><div id=spacer></div><div id='café'>Anchor</div><div id=destination>Final</div><script>document.getElementById('first').addEventListener('click',function(){document.getElementById('state').textContent='preserved';});</script>",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_eris-browser"))
        .arg(&document)
        .args(["--click", "#first", "--click", "#second", "--dump-dom"])
        .args(["--width", "320", "--height", "160", "--output"])
        .arg(directory.join("result.png"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("index.html#destination →"), "{stdout}");
    assert!(stdout.contains("Text(\"preserved\")"), "{stdout}");
    let image = image::open(directory.join("result.png")).unwrap().to_rgb8();
    assert_eq!(image.dimensions(), (320, 160));
    assert_eq!(image.get_pixel(300, 150).0, [0, 0, 255]);
    fs::remove_dir_all(directory).unwrap();
}
