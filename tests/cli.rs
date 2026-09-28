use std::{fs, process::Command};

#[test]
fn worker_benchmark_rejects_ambiguous_modes_and_invalid_counts_before_loading() {
    let cases: &[(&[&str], &str)] = &[
        (&["--benchmark-worker"], "needs an iteration count"),
        (
            &["--benchmark-worker", "abc"],
            "invalid worker iteration count",
        ),
        (&["--benchmark-worker", "0"], "between 1 and 10000"),
        (&["--benchmark-worker", "10001"], "between 1 and 10000"),
        (
            &["--benchmark-worker", "1", "--benchmark", "1"],
            "cannot be combined",
        ),
        (
            &["--benchmark", "1", "--benchmark-worker", "1"],
            "cannot be combined",
        ),
        (
            &["--benchmark-worker", "1", "--dump-dom"],
            "cannot be combined",
        ),
        (
            &["--benchmark-worker", "1", "--click", "#button"],
            "cannot be combined",
        ),
        (
            &["--benchmark-worker", "1", "--exit-after", "1"],
            "cannot be combined",
        ),
        (
            &[
                "--benchmark-worker",
                "1",
                "--window-screenshot",
                "unused.png",
            ],
            "cannot be combined",
        ),
        (
            &["--benchmark-worker", "1", "--width", "0"],
            "viewport exceeds",
        ),
        (
            &["--benchmark-worker", "1", "--height", "8193"],
            "viewport exceeds",
        ),
    ];
    for (arguments, message) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_eris-browser"))
            .arg("/nonexistent-eris-benchmark-document")
            .args(*arguments)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{arguments:?}");
        assert!(output.stdout.is_empty(), "{arguments:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{arguments:?}: {stderr}");
    }
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn worker_benchmark_preserves_scripts_fragments_and_headless_pixels() {
    let directory = std::env::temp_dir().join(format!("eris-cli-benchmark-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let document = directory.join("index.html");
    fs::write(&document, "<style>body{margin:0}#fixed{position:fixed;top:0;left:0;width:10px;height:10px;background:red}#space{height:300px}#target{height:100px;background:blue}</style><div id=fixed></div><div id=space></div><div id=target></div><script>document.getElementById('target').style.background='green';</script>").unwrap();
    let address = format!("{}#target", url::Url::from_file_path(&document).unwrap());
    for scripts in [true, false] {
        let mut pixels = Vec::new();
        for mode in ["--render", "--benchmark-worker"] {
            let output_path = directory.join(format!("{scripts}-{mode}.png"));
            let mut command = Command::new(env!("CARGO_BIN_EXE_eris-browser"));
            command.arg(&address).arg(mode);
            if mode == "--benchmark-worker" {
                command.arg("3");
            }
            if !scripts {
                command.arg("--no-scripts");
            }
            let output = command
                .args(["--width", "320", "--height", "160", "--output"])
                .arg(&output_path)
                .output()
                .unwrap();
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(output.status.success(), "{stderr}");
            if mode == "--benchmark-worker" {
                let stdout = String::from_utf8(output.stdout).unwrap();
                assert!(stdout.starts_with("{\"schema\":1,"), "{stdout}");
                assert!(stdout.ends_with("]}\n"), "{stdout}");
                assert!(stdout.contains(&format!("\"scripts\":{scripts}")));
                assert!(stdout.contains("\"viewport\":[320,160]"));
                assert_eq!(stdout.matches("\"render_exchange_ms\":").count(), 3);
                for key in [
                    "startup_ms",
                    "load_ms",
                    "cold_render_exchange_ms",
                    "cold_clear_paint_ms",
                    "teardown_ms",
                ] {
                    let prefix = format!("\"{key}\":");
                    let value: f64 = stdout
                        .split_once(&prefix)
                        .unwrap()
                        .1
                        .split(',')
                        .next()
                        .unwrap()
                        .parse()
                        .unwrap();
                    assert!(value.is_finite() && value >= 0.0, "{key}: {value}");
                }
                assert!(stderr.contains("Landlock ABI 6 and seccomp"), "{stderr}");
                assert!(
                    stderr
                        .lines()
                        .all(|line| line.starts_with("[page] Page process ")
                            || line.starts_with("[page] Resource broker ")),
                    "{stderr}"
                );
            } else {
                assert!(stderr.is_empty(), "{stderr}");
            }
            let image = image::open(output_path).unwrap().to_rgb8();
            assert_eq!(image.dimensions(), (320, 160));
            assert_eq!(image.get_pixel(5, 5).0, [255, 0, 0]);
            assert_eq!(
                image.get_pixel(300, 150).0,
                if scripts { [0, 128, 0] } else { [0, 0, 255] }
            );
            pixels.push(image);
        }
        assert_eq!(pixels[0], pixels[1]);
    }
    fs::remove_dir_all(directory).unwrap();
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn worker_benchmark_does_not_measure_an_error_page_after_rejected_document_load() {
    let directory =
        std::env::temp_dir().join(format!("eris-cli-benchmark-error-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let document = directory.join("oversized.html");
    let file = fs::File::create(&document).unwrap();
    file.set_len(8 * 1024 * 1024 + 1).unwrap();
    drop(file);
    let image = directory.join("must-not-exist.png");
    let output = Command::new(env!("CARGO_BIN_EXE_eris-browser"))
        .arg(document)
        .args(["--benchmark-worker", "1", "--output"])
        .arg(&image)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("renderer returned its error page"));
    assert!(!image.exists());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn headless_fragment_navigation_preserves_script_state_and_updates_document_url() {
    let directory = std::env::temp_dir().join(format!("eris-cli-fragments-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let document = directory.join("index.html");
    fs::write(
        &document,
        "<style>body{margin:0}#fixed{position:fixed;top:0;left:0;width:10px;height:10px;background:red}#spacer{height:300px}#destination{height:100px;background:blue}</style><div id=fixed></div><button id=read>Read URL</button><a id=first href='#caf%C3%A9'>Jump</a><a id=second href='#destination'>Again</a><p id=state>before</p><div id=spacer></div><div id='café'>Anchor</div><div id=destination>Final</div><script>document.getElementById('read').addEventListener('click',function(){document.getElementById('state').setAttribute('data-url',document.URL);document.getElementById('state').setAttribute('data-base',document.baseURI);});document.getElementById('first').addEventListener('click',function(){document.getElementById('state').textContent='preserved';});</script>",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_eris-browser"))
        .arg(&document)
        .args([
            "--click",
            "#first",
            "--click",
            "#second",
            "--click",
            "#read",
            "--dump-dom",
        ])
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
    let target = format!(
        "{}#destination",
        url::Url::from_file_path(&document).unwrap()
    );
    for name in ["data-url", "data-base"] {
        assert!(
            stdout.contains(&format!("\"{name}\": \"{target}\"")),
            "{stdout}"
        );
    }
    let image = image::open(directory.join("result.png")).unwrap().to_rgb8();
    assert_eq!(image.get_pixel(5, 5).0, [255, 0, 0]);
    assert_eq!(image.dimensions(), (320, 160));
    assert_eq!(image.get_pixel(300, 150).0, [0, 0, 255]);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn presenter_options_reject_invalid_and_headless_modes_before_opening_a_window() {
    let cases: &[(&[&str], &str)] = &[
        (&["--presenter"], "needs a mode"),
        (&["--presenter=metal"], "must be software or vulkan"),
        (&["--presenter", "auto"], "must be software or vulkan"),
        (&["--vulkan-verify-frames"], "needs a count"),
        (&["--vulkan-verify-frames", "0"], "between 1 and 8"),
        (&["--vulkan-verify-frames", "9"], "between 1 and 8"),
        (&["--vulkan-verify-frames", "256"], "between 1 and 8"),
        (
            &["--vulkan-verify-frames", "1"],
            "requires --presenter=vulkan",
        ),
        (
            &["--presenter=vulkan", "--render"],
            "requires a desktop window",
        ),
        (
            &["--headless", "--presenter", "vulkan"],
            "requires a desktop window",
        ),
        (
            &["--presenter=vulkan", "--dump-dom"],
            "requires a desktop window",
        ),
        (
            &["--presenter=vulkan", "--benchmark", "1"],
            "requires a desktop window",
        ),
        (
            &["--presenter=vulkan", "--benchmark-worker", "1"],
            "requires a desktop window",
        ),
        (
            &["--presenter=vulkan", "--click", "body"],
            "requires a desktop window",
        ),
    ];
    for (arguments, expected) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_eris-browser"))
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .arg("/nonexistent-eris-presenter-document")
            .args(*arguments)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{arguments:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(expected), "{arguments:?}: {stderr}");
        assert!(
            !stderr.contains("Unable to open browser window"),
            "{stderr}"
        );
    }
}

#[test]
#[cfg(not(all(target_os = "linux", feature = "vulkan-presenter")))]
fn unavailable_vulkan_feature_is_an_explicit_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_eris-browser"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .arg("--presenter=vulkan")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("requires Linux and a build with --features vulkan-presenter")
    );
}

#[test]
#[cfg(all(target_os = "linux", feature = "vulkan-presenter"))]
fn clean_worker_launcher_accepts_only_exact_internal_roles() {
    for args in [
        vec!["--clean-worker-launch"],
        vec!["--clean-worker-launch", "--help"],
        vec!["--clean-worker-launch", "--page-worker", "extra"],
        vec!["--clean-worker-launch", "/bin/sh"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_eris-browser"))
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("worker"));
    }
}
