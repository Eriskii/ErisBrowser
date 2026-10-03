//! Real-binary IPC tests: `cargo test --test worker -- --include-ignored`.
//! Confined worker tests require Linux with Landlock ABI 6; framing rejection
//! happens before sandbox initialization and is independently testable.
#![cfg(target_os = "linux")]

use eris::{
    page::Navigation,
    worker::{Command as WorkerCommand, Snapshot, WorkerClient},
};
use std::{
    cell::Cell,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
use url::Url;

const BINARY: &str = env!("CARGO_BIN_EXE_eris-browser");
static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    directory: PathBuf,
    navigation: Navigation,
}
impl Fixture {
    fn new(source: &str) -> Self {
        let directory = loop {
            let candidate = std::env::temp_dir().join(format!(
                "eris-worker-integration-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create fixture directory: {error}"),
            }
        };
        let path = directory.join("index.html");
        fs::write(&path, source).expect("write fixture");
        Self {
            directory,
            navigation: Navigation::get(Url::from_file_path(path).unwrap().to_string()),
        }
    }
    fn spawn(&self, scripts: bool, generation: u64) -> WorkerClient {
        WorkerClient::spawn_at(Path::new(BINARY), scripts, &self.navigation, generation)
            .expect("real page worker requires fully enforced Linux Landlock ABI 6")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn load(client: &mut WorkerClient, navigation: &Navigation) {
    let reply = client
        .exchange(
            WorkerCommand::Load {
                navigation: navigation.clone(),
            },
            || false,
        )
        .expect("load through IPC");
    assert!(reply.snapshot.is_none());
    assert!(reply.navigation.is_none());
}
fn render(client: &mut WorkerClient) -> Snapshot {
    let reply = client
        .exchange(
            WorkerCommand::Render {
                width: 320.0,
                height: 240.0,
            },
            || false,
        )
        .expect("render through IPC");
    assert!(reply.navigation.is_none());
    reply.snapshot.expect("render returns a snapshot")
}
fn exchange_empty(client: &mut WorkerClient, command: WorkerCommand) {
    let reply = client.exchange(command, || false).expect("IPC command");
    assert!(reply.snapshot.is_none());
    assert!(reply.navigation.is_none());
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_date_receives_host_rules_before_any_author_script() {
    const HELPER: &str = "ERIS_WORKER_DATE_CONTEXT_HELPER";
    if std::env::var_os(HELPER).is_none() {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "confined_date_receives_host_rules_before_any_author_script",
                "--ignored",
                "--nocapture",
            ])
            .env(HELPER, "1")
            .env("TZ", "EST5EDT,M3.2.0,M11.1.0")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let fixture = Fixture::new(
        "<title>before</title><script>document.title=new Date(0).getTimezoneOffset()+','+new Date(Date.UTC(2020,6,1)).getTimezoneOffset()+','+new Date(1970,0,1).getTime()+','+new Date(0).toISOString();</script>",
    );
    let mut client = fixture.spawn(true, 123);
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    assert_eq!(snapshot.title, "300,240,18000000,1970-01-01T00:00:00.000Z");
    assert!(
        snapshot
            .diagnostics
            .iter()
            .all(|message| message.starts_with("Page process ")
                || message.starts_with("Resource broker ")),
        "{:?}",
        snapshot.diagnostics
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn inline_style_mutations_preserve_the_same_pixels_through_the_confined_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/inline-style.html"), 117);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn default_parameters_and_event_callbacks_survive_the_confined_worker() {
    assert_six_scripted_samples_through_worker(
        include_str!("fixtures/default-parameters.html"),
        121,
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn rest_parameters_and_event_callbacks_survive_the_confined_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/rest-parameters.html"), 123);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn prototype_membership_and_mutations_survive_the_confined_worker() {
    assert_six_scripted_samples_through_worker(
        include_str!("fixtures/prototype-membership.html"),
        127,
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn replaceable_window_self_preserves_pixels_and_events_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/window-self.html"), 129);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn global_value_properties_preserve_pixels_and_events_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/global-values.html"), 133);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn stable_array_sort_preserves_pixels_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/array-sort.html"), 137);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn unicode_identifier_bindings_preserve_pixels_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/identifiers.html"), 139);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn array_reductions_preserve_pixels_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/array-reduce.html"), 143);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn numeric_conversion_preserves_pixels_and_hooks_through_the_worker() {
    assert_six_scripted_samples_through_worker(
        include_str!("fixtures/numeric-conversion.html"),
        151,
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn numeric_parsing_preserves_pixels_and_aliases_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/numeric-parsing.html"), 153);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn compound_bitwise_assignments_preserve_pixels_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(
        include_str!("fixtures/compound-assignment.html"),
        157,
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn labeled_jumps_preserve_finalizers_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/labels.html"), 179);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn equality_conversions_preserve_rules_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/equality.html"), 173);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn relational_conversions_preserve_order_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/relational.html"), 169);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn uri_transforms_preserve_unicode_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/uri.html"), 167);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn logical_assignment_preserves_short_circuit_and_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(
        include_str!("fixtures/logical-assignment.html"),
        163,
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn addition_preserves_pixels_and_conversion_callbacks_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/addition.html"), 161);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn number_statics_preserve_pixels_and_native_aliases_through_the_worker() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/number-statics.html"), 147);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn window_self_persists_across_fragments_and_resets_in_a_fresh_worker() {
    let fixture = Fixture::new(include_str!("fixtures/window-self.html"));
    let mut client = fixture.spawn(true, 130);
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    let button = snapshot.document.query_selector("#change").unwrap();
    let mut destination = Url::parse(&fixture.navigation.address).unwrap();
    destination.set_fragment(Some("replacement"));
    exchange_empty(
        &mut client,
        WorkerCommand::Fragment {
            address: destination.to_string(),
        },
    );
    exchange_empty(&mut client, WorkerCommand::Click { node: button });
    for (reload, state) in [(false, "clicked"), (true, "ready")] {
        if reload {
            drop(client);
            client = fixture.spawn(true, 131);
            load(&mut client, &fixture.navigation);
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, if reload { 131 } else { 130 });
        assert_eq!(
            snapshot.url,
            if reload {
                fixture.navigation.address.as_str()
            } else {
                destination.as_str()
            }
        );
        let body = snapshot.document.query_selector("body").unwrap();
        assert_eq!(snapshot.document.attr(body, "class"), Some(state));
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|line| line.starts_with("Page process ")
                    || line.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
    }
}

fn assert_six_scripted_samples_through_worker(source: &str, generation: u64) {
    assert_six_scripted_samples_through_worker_with_result_text(source, generation, None);
}

fn assert_six_scripted_samples_through_worker_with_result_text(
    source: &str,
    generation: u64,
    result_text: Option<[&str; 2]>,
) {
    assert_six_scripted_samples_through_worker_with_result_rendering(
        source,
        generation,
        result_text,
        false,
    );
}

fn assert_six_scripted_samples_through_worker_with_result_rendering(
    source: &str,
    generation: u64,
    result_text: Option<[&str; 2]>,
    check_text_commands: bool,
) {
    use eris::graphics::{Canvas, Color, DrawCommand, Fonts};
    let fixture = Fixture::new(source);
    let mut client = fixture.spawn(true, generation);
    load(&mut client, &fixture.navigation);
    let mut direct = eris::page::Page::from_html(
        Url::parse(&fixture.navigation.address).unwrap(),
        source,
        true,
    );
    let fonts = Fonts::new();
    for (index, (state, color)) in [("ready", 0x008000), ("clicked", 0x0000ff)]
        .into_iter()
        .enumerate()
    {
        if state == "clicked" {
            let node = direct.document.query_selector("#change").unwrap();
            assert!(direct.click(node).is_none());
            exchange_empty(&mut client, WorkerCommand::Click { node });
        }
        let snapshot = render(&mut client);
        let body = snapshot.document.query_selector("body").unwrap();
        assert_eq!(snapshot.document.attr(body, "class"), Some(state));
        if let Some(expected) = result_text {
            let result = direct.document.query_selector("#result").unwrap();
            assert_eq!(
                direct.document.text_content(result),
                expected[index],
                "direct {state}"
            );
            let result = snapshot.document.query_selector("#result").unwrap();
            assert_eq!(
                snapshot.document.text_content(result),
                expected[index],
                "worker {state}"
            );
        }
        assert!(direct.diagnostics.is_empty(), "{:?}", direct.diagnostics);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|line| line.starts_with("Page process ")
                    || line.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        let direct_layout = direct.layout(320.0, 240.0, &fonts);
        if check_text_commands {
            let expected = result_text.expect("text commands require expected result text")[index];
            for (route, commands) in [
                ("direct", &direct_layout.commands),
                ("worker", &snapshot.layout.commands),
            ] {
                assert!(
                    commands.iter().any(
                        |command| matches!(command, DrawCommand::Text { text, .. } if text == expected)
                    ),
                    "{route} result text must reach a draw command in {state}"
                );
            }
        }
        let mut expected = Canvas::new(320, 240).unwrap();
        expected.clear(Color::WHITE);
        expected.paint(&direct_layout.commands, &fonts, &direct.images, 0.0, 0.0);
        let mut actual = Canvas::new(320, 240).unwrap();
        actual.clear(Color::WHITE);
        actual.paint(
            &snapshot.layout.commands,
            &fonts,
            &snapshot.images,
            0.0,
            0.0,
        );
        assert!(!actual.exhausted() && !expected.exhausted());
        assert_eq!(actual.pixels, expected.pixels);
        for x in [10, 60, 110, 160, 210, 260] {
            assert_eq!(actual.pixels[10 * 320 + x], color, "{state}, x={x}");
        }
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_for_of_protocols_bindings_and_closing_survive_document_callbacks() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/for-of.html"), 181);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_array_from_mapping_construction_and_closing_survive_document_callbacks() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/array-from.html"), 183);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_array_splice_preserves_species_live_properties_and_partial_callback_effects() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/array-splice.html"),
        189,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_array_concat_preserves_spreadability_aliases_and_partial_callback_effects() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/array-concat.html"),
        191,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_array_buffer_resizing_transfer_and_species_survive_document_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/array-buffer.html"),
        193,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_data_view_bytes_resize_and_transfer_survive_document_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/data-view.html"),
        195,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_object_is_same_value_and_saved_identity_survive_document_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/object-is.html"),
        197,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_dom_defining_interface_identity_and_brands_survive_document_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/dom-method-identity.html"),
        198,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_own_key_order_and_live_enumeration_survive_document_callbacks() {
    assert_six_scripted_samples_through_worker(include_str!("fixtures/own-keys.html"), 187);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn mixed_calculations_resize_and_mutate_through_the_confined_worker() {
    use eris::graphics::{Canvas, Color, Fonts};
    let source = include_str!("fixtures/calc-resize.html");
    let fixture = Fixture::new(source);
    let mut client = fixture.spawn(true, 125);
    load(&mut client, &fixture.navigation);
    let mut direct = eris::page::Page::from_html(
        Url::parse(&fixture.navigation.address).unwrap(),
        source,
        true,
    );
    let fonts = Fonts::new();
    for (width, click, x, box_width, color) in [
        (320, false, 45.0, 140.0, 0x008000),
        (520, false, 65.0, 240.0, 0x008000),
        (520, true, 30.0, 130.0, 0x0000ff),
        (320, false, 20.0, 80.0, 0x0000ff),
    ] {
        if click {
            let node = direct.document.query_selector("#change").unwrap();
            assert!(direct.click(node).is_none());
            exchange_empty(&mut client, WorkerCommand::Click { node });
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
        assert!(direct.diagnostics.is_empty(), "{:?}", direct.diagnostics);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|line| line.starts_with("Page process ")
                    || line.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        let sample = snapshot.document.query_selector("#sample").unwrap();
        let rect = snapshot
            .layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == sample)
            .unwrap()
            .rect;
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (x, 10.0, box_width, 40.0)
        );
        let mut expected = Canvas::new(width, 240).unwrap();
        expected.clear(Color::WHITE);
        expected.paint(
            &direct.layout(width as f32, 240.0, &fonts).commands,
            &fonts,
            &direct.images,
            0.0,
            0.0,
        );
        let mut actual = Canvas::new(width, 240).unwrap();
        actual.clear(Color::WHITE);
        actual.paint(
            &snapshot.layout.commands,
            &fonts,
            &snapshot.images,
            0.0,
            0.0,
        );
        assert!(!actual.exhausted() && !expected.exhausted());
        assert_eq!(actual.pixels, expected.pixels);
        let row = 20 * width as usize;
        assert_eq!(actual.pixels[row + x as usize], color);
        assert_eq!(actual.pixels[row + (x + box_width) as usize - 1], color);
        assert_eq!(actual.pixels[row + (x + box_width) as usize], 0xeeeeee);
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn custom_properties_recompute_inherited_aliases_through_the_confined_worker() {
    use eris::graphics::{Canvas, Color, Fonts};
    let source = include_str!("fixtures/custom-properties.html");
    let fixture = Fixture::new(source);
    let mut client = fixture.spawn(true, 113);
    load(&mut client, &fixture.navigation);
    let mut direct = eris::page::Page::from_html(
        Url::parse(&fixture.navigation.address).unwrap(),
        source,
        true,
    );
    let fonts = Fonts::new();
    for (click, color) in [(false, 0x008000), (true, 0x0000ff)] {
        if click {
            let node = direct.document.query_selector("#change").unwrap();
            assert!(direct.click(node).is_none());
            exchange_empty(&mut client, WorkerCommand::Click { node });
        }
        let snapshot = render(&mut client);
        assert!(direct.diagnostics.is_empty(), "{:?}", direct.diagnostics);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|line| line.starts_with("Page process ")
                    || line.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        let mut expected = Canvas::new(320, 240).unwrap();
        expected.clear(Color::WHITE);
        expected.paint(
            &direct.layout(320.0, 240.0, &fonts).commands,
            &fonts,
            &direct.images,
            0.0,
            0.0,
        );
        let mut actual = Canvas::new(320, 240).unwrap();
        actual.clear(Color::WHITE);
        actual.paint(
            &snapshot.layout.commands,
            &fonts,
            &snapshot.images,
            0.0,
            0.0,
        );
        assert!(!actual.exhausted() && !expected.exhausted());
        assert_eq!(actual.pixels, expected.pixels);
        assert_eq!(actual.pixels[10 * 320 + 10], color);
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn script_feature_queries_drive_the_same_pixels_through_the_confined_worker() {
    use eris::graphics::{Canvas, Color, Fonts};
    let source = include_str!("fixtures/css-supports.html");
    let fixture = Fixture::new(source);
    let mut client = fixture.spawn(true, 111);
    load(&mut client, &fixture.navigation);
    let mut direct = eris::page::Page::from_html(
        Url::parse(&fixture.navigation.address).unwrap(),
        source,
        true,
    );
    let fonts = Fonts::new();
    for (state, color) in [("ready", 0x008000), ("clicked", 0x0000ff)] {
        if state == "clicked" {
            let node = direct.document.query_selector("#check").unwrap();
            assert!(direct.click(node).is_none());
            exchange_empty(&mut client, WorkerCommand::Click { node });
        }
        let snapshot = render(&mut client);
        let body = snapshot.document.query_selector("body").unwrap();
        assert_eq!(snapshot.document.attr(body, "class"), Some(state));
        assert!(direct.diagnostics.is_empty(), "{:?}", direct.diagnostics);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|line| line.starts_with("Page process ")
                    || line.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        let mut expected = Canvas::new(320, 240).unwrap();
        expected.clear(Color::WHITE);
        expected.paint(
            &direct.layout(320.0, 240.0, &fonts).commands,
            &fonts,
            &direct.images,
            0.0,
            0.0,
        );
        let mut actual = Canvas::new(320, 240).unwrap();
        actual.clear(Color::WHITE);
        actual.paint(
            &snapshot.layout.commands,
            &fonts,
            &snapshot.images,
            0.0,
            0.0,
        );
        assert!(!actual.exhausted() && !expected.exhausted());
        assert_eq!(actual.pixels, expected.pixels);
        assert_eq!(actual.pixels[10 * 320 + 10], color);
    }
    drop(client);
    let mut disabled = fixture.spawn(false, 112);
    load(&mut disabled, &fixture.navigation);
    let snapshot = render(&mut disabled);
    assert!(
        snapshot
            .document
            .attr(snapshot.document.query_selector("body").unwrap(), "class")
            .is_none()
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn idle_task_batches_cross_ipc_without_render_running_scripts() {
    use eris::page::TaskState;
    let fixture = Fixture::new(&format!(
        "<!doctype html><input id=n value=0 readonly>{}<script>let count=0;const n=document.getElementById('n');document.addEventListener('toggle',()=>{{count++;n.value=count;}},true);</script>",
        "<details open></details>".repeat(150)
    ));
    let mut client = fixture.spawn(true, 101);
    load(&mut client, &fixture.navigation);
    for (count, state) in [
        ("64", TaskState::Pending),
        ("128", TaskState::Pending),
        ("150", TaskState::Idle),
        ("150", TaskState::Idle),
    ] {
        for _ in 0..2 {
            let snapshot = render(&mut client);
            assert_eq!(snapshot.task_state, state);
            assert_eq!(snapshot.generation, 101);
            assert_eq!(snapshot.processed_edit_sequence, 0);
            assert_eq!(
                snapshot
                    .document
                    .attr(snapshot.document.query_selector("#n").unwrap(), "value")
                    .unwrap(),
                count
            );
        }
        exchange_empty(&mut client, WorkerCommand::RunTasks);
    }
    drop(client);
    let mut disabled = fixture.spawn(false, 102);
    load(&mut disabled, &fixture.navigation);
    exchange_empty(&mut disabled, WorkerCommand::RunTasks);
    let snapshot = render(&mut disabled);
    assert_eq!(snapshot.task_state, TaskState::Idle);
    assert_eq!(
        snapshot
            .document
            .attr(snapshot.document.query_selector("#n").unwrap(), "value")
            .unwrap(),
        "0"
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn idle_task_quota_failure_suspends_automatic_retries_across_ipc() {
    use eris::page::TaskState;
    let fixture = Fixture::new(&format!(
        "<!doctype html><input id=n value=0 readonly>{}<script>let count=0;const n=document.getElementById('n');document.addEventListener('toggle',e=>{{count++;n.value=count;if(count===65){{e.target.open=false;while(true){{}}}}}},true);</script>",
        "<details open></details>".repeat(150)
    ));
    let mut client = fixture.spawn(true, 103);
    load(&mut client, &fixture.navigation);
    assert_eq!(render(&mut client).task_state, TaskState::Pending);
    for _ in 0..3 {
        exchange_empty(&mut client, WorkerCommand::RunTasks);
        let snapshot = render(&mut client);
        assert_eq!(snapshot.task_state, TaskState::Suspended);
        assert_eq!(
            snapshot
                .document
                .attr(snapshot.document.query_selector("#n").unwrap(), "value")
                .unwrap(),
            "65"
        );
        assert_eq!(
            snapshot
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.contains("suspended until reload"))
                .count(),
            1
        );
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn generated_summary_actions_survive_ipc_and_cannot_activate_ordinary_nodes() {
    use eris::layout::HitAction;
    let fixture = Fixture::new(
        "<!doctype html><details id=fallback><p id=content>Disclosure content</p></details><details id=real><summary id=summary>Authored summary</summary><p>More content</p></details><p id=ordinary>Ordinary text</p>",
    );
    let mut client = fixture.spawn(false, 95);
    load(&mut client, &fixture.navigation);
    let before = render(&mut client);
    let fallback = before.document.query_selector("#fallback").unwrap();
    let content = before.document.query_selector("#content").unwrap();
    assert!(
        before
            .layout
            .hit_regions
            .iter()
            .any(|hit| hit.node == fallback && hit.action == HitAction::DefaultSummary)
    );
    assert!(
        !before
            .layout
            .hit_regions
            .iter()
            .any(|hit| hit.node == content)
    );
    exchange_empty(&mut client, WorkerCommand::Click { node: fallback });
    assert!(
        render(&mut client)
            .document
            .attr(fallback, "open")
            .is_none()
    );
    exchange_empty(
        &mut client,
        WorkerCommand::DefaultSummary { node: fallback },
    );
    let opened = render(&mut client);
    assert!(opened.document.attr(fallback, "open").is_some());
    assert!(
        opened
            .layout
            .hit_regions
            .iter()
            .any(|hit| hit.node == content)
    );
    exchange_empty(
        &mut client,
        WorkerCommand::DefaultSummary { node: fallback },
    );
    let closed = render(&mut client);
    assert!(closed.document.attr(fallback, "open").is_none());
    assert!(
        !closed
            .layout
            .hit_regions
            .iter()
            .any(|hit| hit.node == content)
    );
    drop(client);
    // Invalid action hints fail the worker boundary closed. Each forged
    // command gets a fresh worker because rejection terminates that channel.
    for selector in ["#real", "#summary", "#ordinary"] {
        let mut client = fixture.spawn(false, 96);
        load(&mut client, &fixture.navigation);
        let snapshot = render(&mut client);
        let node = snapshot.document.query_selector(selector).unwrap();
        assert!(
            client
                .exchange(WorkerCommand::DefaultSummary { node }, || false)
                .is_err()
        );
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn disclosure_group_clicks_and_toggle_handlers_match_direct_page_pixels_through_ipc() {
    use eris::{
        graphics::{Canvas, Fonts},
        page::Page,
    };
    let source = include_str!("../examples/disclosures.html");
    let fixture = Fixture::new(source);
    let mut expected = Page::from_html(
        Url::parse(&fixture.navigation.address).unwrap(),
        source,
        true,
    );
    let mut client = fixture.spawn(true, 94);
    load(&mut client, &fixture.navigation);
    let fonts = Fonts::new();
    for action in [
        None,
        Some("#next-note"),
        Some("#summary-3"),
        Some("#summary-3"),
        Some("#next-note"),
    ] {
        if let Some(selector) = action {
            let before = render(&mut client);
            let node = before.document.query_selector(selector).unwrap();
            exchange_empty(&mut client, WorkerCommand::Click { node });
            assert!(
                expected
                    .click(expected.document.query_selector(selector).unwrap())
                    .is_none()
            );
        }
        assert!(
            expected.diagnostics.is_empty(),
            "{:?}",
            expected.diagnostics
        );
        for width in [960, 390] {
            let reply = client
                .exchange(
                    WorkerCommand::Render {
                        width: width as f32,
                        height: 1300.0,
                    },
                    || false,
                )
                .unwrap();
            assert!(reply.navigation.is_none());
            let snapshot = reply.snapshot.unwrap();
            assert!(
                snapshot
                    .diagnostics
                    .iter()
                    .all(|line| line.starts_with("Page process ")
                        || line.starts_with("Resource broker ")),
                "{:?}",
                snapshot.diagnostics
            );
            for selector in ["#status", "#note-1", "#note-2", "#note-3"] {
                let actual = snapshot.document.query_selector(selector).unwrap();
                let target = expected.document.query_selector(selector).unwrap();
                assert_eq!(
                    snapshot.document.text_content(actual),
                    expected.document.text_content(target)
                );
                assert_eq!(
                    snapshot.document.attr(actual, "open"),
                    expected.document.attr(target, "open")
                );
            }
            let mut actual = Canvas::new(width, 1300).unwrap();
            actual.paint(
                &snapshot.layout.commands,
                &fonts,
                &snapshot.images,
                0.0,
                0.0,
            );
            let layout = expected.layout(width as f32, 1300.0, &fonts);
            let mut direct = Canvas::new(width, 1300).unwrap();
            direct.paint(&layout.commands, &fonts, &expected.images, 0.0, 0.0);
            assert!(!actual.exhausted() && !direct.exhausted());
            assert_eq!(
                actual.pixels, direct.pixels,
                "action={action:?}, width={width}"
            );
        }
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn responsive_template_updates_and_flex_resize_survive_worker_snapshots() {
    let fixture = Fixture::new(include_str!("../examples/responsive.html"));
    let mut client = fixture.spawn(true, 93);
    load(&mut client, &fixture.navigation);
    let initial = render(&mut client);
    let button = initial.document.query_selector("#advance").unwrap();
    exchange_empty(&mut client, WorkerCommand::Click { node: button });
    for (width, visible) in [
        (960.0, "#wide"),
        (390.0, "#compact"),
        (800.0, "#medium"),
        (960.0, "#wide"),
    ] {
        let reply = client
            .exchange(
                WorkerCommand::Render {
                    width,
                    height: 900.0,
                },
                || false,
            )
            .unwrap();
        assert!(reply.navigation.is_none());
        let snapshot = reply.snapshot.unwrap();
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|m| m.starts_with("Page process ") || m.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
        let status = snapshot.document.query_selector("#status").unwrap();
        assert_eq!(snapshot.document.text_content(status), "Reading 2 · midday");
        for selector in ["#wide", "#medium", "#compact"] {
            let node = snapshot.document.query_selector(selector).unwrap();
            assert_eq!(
                snapshot
                    .layout
                    .hit_regions
                    .iter()
                    .any(|hit| hit.node == node),
                selector == visible
            );
        }
        let cards = snapshot.document.query_selector_all(".sample");
        let boxes = cards
            .iter()
            .map(|node| {
                snapshot
                    .layout
                    .hit_regions
                    .iter()
                    .find(|hit| hit.node == *node)
                    .unwrap()
                    .rect
            })
            .collect::<Vec<_>>();
        assert_eq!(boxes.len(), 6);
        if width >= 640.0 {
            assert_eq!(boxes[0].x, boxes[1].x);
            assert_eq!(boxes[0].y, boxes[2].y);
            assert!(boxes[2].x > boxes[0].x);
        } else {
            assert!(
                boxes
                    .windows(2)
                    .all(|pair| pair[1].x == pair[0].x && pair[1].y > pair[0].y)
            );
        }
        let mut canvas = eris::graphics::Canvas::new(width as u32, 900).unwrap();
        canvas.paint(
            &snapshot.layout.commands,
            &eris::graphics::Fonts::new(),
            &snapshot.images,
            0.0,
            0.0,
        );
        assert!(!canvas.exhausted());
        let x = (boxes[0].x + 2.0) as usize;
        let y = (boxes[0].y + 40.0) as usize;
        assert_eq!(canvas.pixels[y * width as usize + x], 0x203441);
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_stylesheet_imports_and_fixed_scopes_survive_snapshot_transfer() {
    let fixture = Fixture::new(
        "<!doctype html><base href='assets/'><link rel=stylesheet href='root.css'><div id=f>fixed</div><div id=body>body</div>",
    );
    fs::create_dir(fixture.directory.join("assets")).unwrap();
    fs::write(fixture.directory.join("assets/root.css"), "@import 'palette.css'; body{margin:0} #f{position:fixed;left:10px;top:12px;width:40px;height:20px} #body{height:900px}").unwrap();
    fs::write(
        fixture.directory.join("assets/palette.css"),
        "#f{background:red}",
    )
    .unwrap();
    let mut client = fixture.spawn(false, 90);
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    assert!(
        snapshot
            .diagnostics
            .iter()
            .all(|message| message.starts_with("Page process ")
                || message.starts_with("Resource broker ")),
        "{:?}",
        snapshot.diagnostics
    );
    assert_eq!(snapshot.document.url().as_str(), snapshot.url);
    let fixed = snapshot.document.query_selector("#f").unwrap();
    assert!(
        snapshot
            .layout
            .hit_regions
            .iter()
            .any(|h| h.node == fixed && h.fixed && h.rect.x == 10.0 && h.rect.y == 12.0)
    );
    assert!(
        snapshot
            .layout
            .commands
            .iter()
            .any(|c| matches!(c, eris::graphics::DrawCommand::PushFixed))
    );
    let mut canvas = eris::graphics::Canvas::new(320, 240).unwrap();
    canvas.paint_with_viewport(
        &snapshot.layout.commands,
        &eris::graphics::Fonts::new(),
        &snapshot.images,
        (0.0, -400.0),
        (0.0, 0.0),
    );
    assert_eq!(canvas.pixels[13 * 320 + 11], 0xff0000);
}
fn process_exists(pid: u32) -> bool {
    Path::new("/proc").join(pid.to_string()).exists()
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn layered_imports_opacity_and_event_capture_cross_real_process_boundary() {
    let fixture = Fixture::new(
        r#"<!doctype html><link rel=stylesheet href=main.css>
      <div id=group><div id=back></div><div id=front></div></div>
      <div id=panel><a id=go href='/must-not-navigate'>Send</a></div><p id=out>ready</p>
      <script>
      var trace=[];var go=document.getElementById('go');var panel=document.getElementById('panel');
      document.addEventListener('click',function(e){trace=['document capture'];},true);
      panel.addEventListener('click',function(e){trace.push('panel capture');}, {capture:true});
      go.addEventListener('click',function(e){trace.push('target');e.preventDefault();go.dispatchEvent(new CustomEvent('signal',{bubbles:true,detail:7}));});
      panel.addEventListener('signal',function(e){trace.push('signal '+e.detail);},{once:true});
      panel.addEventListener('click',function(e){trace.push('panel bubble');});
      document.addEventListener('click',function(e){trace.push('document bubble');document.getElementById('out').textContent=trace.join('/');});
      </script>"#,
    );
    fs::write(fixture.directory.join("main.css"), "@layer base,theme;@import 'theme.css' layer(theme);@import 'base.css' layer(base);body{margin:0}#group{position:fixed;left:0;top:0;width:70px;height:30px;opacity:0.5}#back,#front{position:absolute;top:0;width:40px;height:30px}#back{left:0}#front{left:20px}#panel{margin-top:80px}").unwrap();
    fs::write(
        fixture.directory.join("base.css"),
        "#back,#front{background:blue}",
    )
    .unwrap();
    fs::write(
        fixture.directory.join("theme.css"),
        "#back,#front{background:red}",
    )
    .unwrap();
    let mut client = fixture.spawn(true, 91);
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    assert!(
        snapshot
            .diagnostics
            .iter()
            .all(|m| m.starts_with("Page process ") || m.starts_with("Resource broker ")),
        "{:?}",
        snapshot.diagnostics
    );
    assert!(snapshot.layout.commands.iter().any(
        |c| matches!(c, eris::graphics::DrawCommand::PushOpacity { opacity } if *opacity == 0.5)
    ));
    let mut canvas = eris::graphics::Canvas::new(320, 240).unwrap();
    canvas.paint_with_viewport(
        &snapshot.layout.commands,
        &eris::graphics::Fonts::new(),
        &snapshot.images,
        (0.0, -100.0),
        (0.0, 0.0),
    );
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels[5 * 320 + 5], 0xff8080);
    assert_eq!(
        canvas.pixels[5 * 320 + 25],
        canvas.pixels[5 * 320 + 5],
        "overlap must be composited once"
    );
    let go = snapshot.document.query_selector("#go").unwrap();
    for expected in [
        "document capture/panel capture/target/signal 7/panel bubble/document bubble",
        "document capture/panel capture/target/panel bubble/document bubble",
    ] {
        exchange_empty(&mut client, WorkerCommand::Click { node: go });
        let snapshot = render(&mut client);
        let out = snapshot.document.query_selector("#out").unwrap();
        assert_eq!(snapshot.document.text_content(out), expected);
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn aborting_a_listener_restores_native_link_navigation_across_ipc() {
    let fixture = Fixture::new(
        r#"<!doctype html><a id=go href='#next'>Continue</a>
        <button id=stop type=button>Release listener</button><p id=out>ready</p>
        <section id=next>Destination</section>
        <script>
        var controller = new AbortController();
        var signal = controller.signal;
        var out = document.getElementById('out');
        var count = 0;
        document.getElementById('go').addEventListener('click', function(event) {
            event.preventDefault(); count++; out.textContent = 'held ' + count;
        }, {signal:signal});
        signal.addEventListener('abort', function(event) {
            out.textContent = 'released ' + signal.reason + ' ' + event.isTrusted;
        });
        document.getElementById('stop').addEventListener('click', function() {
            controller.abort('closed');
        });
        </script>"#,
    );
    let mut client = fixture.spawn(true, 92);
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    let go = snapshot.document.query_selector("#go").unwrap();
    let stop = snapshot.document.query_selector("#stop").unwrap();
    let output = snapshot.document.query_selector("#out").unwrap();
    for (node, expected) in [(go, "held 1"), (stop, "released closed true")] {
        exchange_empty(&mut client, WorkerCommand::Click { node });
        let snapshot = render(&mut client);
        assert_eq!(snapshot.document.text_content(output), expected);
        assert!(
            snapshot
                .diagnostics
                .iter()
                .all(|message| message.starts_with("Page process ")
                    || message.starts_with("Resource broker ")),
            "{:?}",
            snapshot.diagnostics
        );
    }
    let reply = client
        .exchange(WorkerCommand::Click { node: go }, || false)
        .unwrap();
    let mut destination = Url::parse(&fixture.navigation.address).unwrap();
    destination.set_fragment(Some("next"));
    let navigation = reply
        .navigation
        .expect("the aborted listener must no longer cancel navigation");
    assert_eq!(navigation.address, destination.as_str());
    assert!(navigation.form_body.is_none());
    assert_eq!(
        render(&mut client).document.text_content(output),
        "released closed true"
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn isolated_load_render_returns_valid_snapshot_from_distinct_process() {
    let fixture = Fixture::new(
        "<!doctype html><title>Worker fixture</title><h1 id=heading>Hello worker</h1><svg width=12 height=12><rect width=12 height=12 fill=red /></svg>",
    );
    let mut client = fixture.spawn(false, 42);
    let pid = client.pid();
    assert_ne!(pid, std::process::id());
    assert!(process_exists(pid));
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    assert_eq!(snapshot.generation, 42);
    assert_eq!(snapshot.processed_edit_sequence, 0);
    assert_eq!(snapshot.title, "Worker fixture");
    assert_eq!(snapshot.url, fixture.navigation.address);
    assert_eq!(
        snapshot
            .document
            .text_content(snapshot.document.query_selector("#heading").unwrap()),
        "Hello worker"
    );
    assert!(!snapshot.layout.commands.is_empty());
    assert!(snapshot.layout.content_height.is_finite());
    assert!(!snapshot.images.is_empty());
    let broker_pid = client
        .broker_pid()
        .expect("local document uses the resource broker");
    assert_ne!(broker_pid, pid);
    assert!(process_exists(broker_pid));
    assert!(snapshot.diagnostics.iter().any(|message| {
        message.starts_with(&format!("Page process {pid}:")) && message.contains("Landlock ABI 6")
    }));
    drop(client);
    assert!(!process_exists(pid), "Drop must reap the worker process");
    assert!(
        !process_exists(broker_pid),
        "Drop must reap the resource broker"
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn template_fragments_strict_callbacks_and_encoding_survive_native_snapshots() {
    let fixture = Fixture::new(include_str!("../examples/templates.html"));
    let mut client = fixture.spawn(true, 76);
    let pid = client.pid();
    load(&mut client, &fixture.navigation);
    let initial = render(&mut client);
    assert_eq!(initial.document.mode(), eris::dom::DocumentMode::NoQuirks);
    assert_eq!(initial.document.character_set(), "UTF-8");
    assert_eq!(initial.title, "Eris · A collection of small things");
    let collection = initial.document.query_selector("#collection").unwrap();
    let template = initial.document.query_selector("#note").unwrap();
    let contents = initial.document.template_contents(template).unwrap();
    assert!(initial.document.nodes[template].children.is_empty());
    assert_eq!(initial.document.nodes[contents].children.len(), 1);
    assert_eq!(initial.document.nodes[collection].children.len(), 5);
    for selector in ["#arrange", "#add"] {
        exchange_empty(
            &mut client,
            WorkerCommand::Click {
                node: initial.document.query_selector(selector).unwrap(),
            },
        );
    }
    let changed = render(&mut client);
    assert_eq!(
        changed.document.attr(collection, "class"),
        Some("collection compact")
    );
    assert_eq!(changed.document.nodes[collection].children.len(), 6);
    assert_eq!(changed.document.nodes[contents].children.len(), 1);
    assert_eq!(
        changed
            .document
            .text_content(changed.document.query_selector("#status").unwrap()),
        "6 notes in the collection"
    );
    assert!(
        changed
            .diagnostics
            .iter()
            .all(|message| message.starts_with("Page process ")
                || message.starts_with("Resource broker ")),
        "{:?}",
        changed.diagnostics
    );
    let broker_pid = client.broker_pid().unwrap();
    drop(client);
    assert!(!process_exists(pid));
    assert!(!process_exists(broker_pid));
}

fn accept_request(listener: &TcpListener, expected_path: &str) -> std::net::TcpStream {
    accept_form_request(listener, "GET", expected_path, None)
}
fn accept_form_request(
    listener: &TcpListener,
    method: &str,
    expected_path: &str,
    expected_body: Option<&str>,
) -> std::net::TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(
                    Instant::now() < deadline,
                    "fixture received no request for {expected_path}"
                );
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("accept fixture request: {error}"),
        }
    };
    stream
        .set_write_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut request = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !request.ends_with(b"\r\n\r\n") {
        assert!(request.len() < 16_384);
        let remaining = deadline.saturating_duration_since(Instant::now());
        assert!(!remaining.is_zero(), "fixture header deadline");
        stream.set_read_timeout(Some(remaining)).unwrap();
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        request.push(byte[0]);
    }
    let headers = String::from_utf8(request).unwrap();
    assert!(headers.starts_with(&format!("{method} {expected_path} HTTP/1.1\r\n")));
    if let Some(body) = expected_body {
        let lower = headers.to_ascii_lowercase();
        assert!(lower.contains("content-type: application/x-www-form-urlencoded\r\n"));
        assert!(lower.contains(&format!("content-length: {}\r\n", body.len())));
        let mut received = vec![0; body.len()];
        stream.read_exact(&mut received).unwrap();
        assert_eq!(received, body.as_bytes());
    }
    stream
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and a bounded HTTP form fixture"]
fn broker_submits_only_the_authorized_form_body_and_commits_redirect() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let body = "q=hello+world&symbol=%F0%9F%A6%80";
    let navigation = Navigation {
        address: format!("{base}/submit?fixed=1"),
        form_body: Some(body.into()),
    };
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), true, &navigation, 107).unwrap();
    let mut wrong = navigation.clone();
    wrong.form_body = Some("q=unauthorized".into());
    assert!(
        client
            .exchange(WorkerCommand::Load { navigation: wrong }, || false)
            .err()
            .expect("unauthorized form body must fail")
            .contains("authorized")
    );
    assert!(client.broker_pid().is_none());
    let server = thread::spawn(move || {
        let mut stream = accept_form_request(&listener, "POST", "/submit?fixed=1", Some(body));
        write!(stream, "HTTP/1.1 303 See Other\r\nLocation: /complete\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = accept_request(&listener, "/complete");
        let html = "<!doctype html><title>Authorized form</title><p>Received</p>";
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}", html.len()).unwrap();
    });
    load(&mut client, &navigation);
    let snapshot = render(&mut client);
    assert_eq!(snapshot.title, "Authorized form");
    assert_eq!(snapshot.url, format!("{base}/complete"));
    server.join().unwrap();
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and two bounded loopback HTTP origins"]
fn broker_commits_cross_origin_redirect_and_enforces_the_final_origin() {
    let initial = TcpListener::bind("127.0.0.1:0").unwrap();
    let destination = TcpListener::bind("127.0.0.1:0").unwrap();
    initial.set_nonblocking(true).unwrap();
    destination.set_nonblocking(true).unwrap();
    let initial_address = initial.local_addr().unwrap();
    let final_address = destination.local_addr().unwrap();
    let navigation = Navigation::get(format!("http://{initial_address}/start"));
    let final_url = format!("http://{final_address}/final/page");
    let redirect_url = final_url.clone();
    let server = thread::spawn(move || {
        let mut stream = accept_request(&initial, "/start");
        write!(stream, "HTTP/1.1 302 Found\r\nLocation: {redirect_url}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = accept_request(&destination, "/final/page");
        let body = format!(
            "<!doctype html><title>Broker redirect</title><p id=result>initial</p><script src=after.js></script><script src='http://{initial_address}/forbidden.js'></script><img src='/redirect-image'><img src='http://{initial_address}/direct-image'><img src='http://{initial_address}/direct-svg'><img src='http://{initial_address}/denied'><img src='/redirect-denied'>"
        );
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        drop(stream);
        let mut stream = accept_request(&destination, "/final/after.js");
        let body = "document.getElementById('result').textContent='final origin script';";
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/javascript\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        drop(stream);
        let mut stream = accept_request(&destination, "/redirect-image");
        write!(stream, "HTTP/1.1 302 Found\r\nLocation: http://{initial_address}/private.json\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = accept_request(&initial, "/private.json");
        let private = br#"{"secret":"cross-origin body must not reach renderer"}"#;
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Private: hidden\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", private.len()).unwrap();
        stream.write_all(private).unwrap();
        drop(stream);
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_pixel(3, 2, image::Rgba([11, 22, 33, 255]))
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let png = png.into_inner();
        let mut stream = accept_request(&initial, "/direct-image");
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nX-Private: hidden\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", png.len()).unwrap();
        stream.write_all(&png).unwrap();
        drop(stream);
        let mut stream = accept_request(&initial, "/direct-svg");
        let svg = "<svg width='2' height='3'><rect width='2' height='3' fill='#123456'/></svg>";
        write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: image/svg+xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{svg}", svg.len()).unwrap();
        drop(stream);
        let mut stream = accept_request(&initial, "/denied");
        write!(stream, "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = accept_request(&destination, "/redirect-denied");
        write!(stream, "HTTP/1.1 302 Found\r\nLocation: http://{initial_address}/redirect-target-denied\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        drop(stream);
        let mut stream = accept_request(&initial, "/redirect-target-denied");
        write!(stream, "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        initial
    });
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), true, &navigation, 105).unwrap();
    load(&mut client, &navigation);
    let snapshot = render(&mut client);
    let initial = server.join().unwrap();
    assert_eq!(snapshot.url, final_url);
    assert_eq!(snapshot.title, "Broker redirect");
    assert_eq!(
        snapshot
            .document
            .text_content(snapshot.document.query_selector("#result").unwrap()),
        "final origin script"
    );
    assert!(
        snapshot
            .diagnostics
            .iter()
            .any(|d| d.contains("cross-origin")),
        "{:?}",
        snapshot.diagnostics
    );
    assert_eq!(snapshot.images.len(), 2);
    let png = snapshot
        .images
        .get(&format!("http://{initial_address}/direct-image"))
        .unwrap();
    assert_eq!((png.width, png.height), (3, 2));
    assert!(
        png.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [11, 22, 33, 255])
    );
    let svg = snapshot
        .images
        .get(&format!("http://{initial_address}/direct-svg"))
        .unwrap();
    assert_eq!((svg.width, svg.height), (2, 3));
    assert_eq!(&svg.rgba[..4], &[0x12, 0x34, 0x56, 255]);
    assert!(!snapshot.images.contains_key("/redirect-image"));
    assert!(!snapshot.diagnostics.iter().any(|d| d.contains("HTTP 403")
        || d.contains("HTTP 404")
        || d.contains("redirect-target-denied")));
    assert_eq!(
        snapshot
            .diagnostics
            .iter()
            .filter(|d| d.contains("image loading or decoding failed"))
            .count(),
        3
    );
    assert!(
        snapshot
            .diagnostics
            .iter()
            .any(|d| d.contains("image loading or decoding failed"))
    );
    assert!(
        !snapshot
            .diagnostics
            .iter()
            .any(|d| d.contains("cross-origin body") || d.contains("X-Private"))
    );
    assert_eq!(
        initial.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock,
        "active cross-origin scripts must not reach the previous origin"
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and a bounded stalled HTTP fixture"]
fn cancellation_during_broker_io_reaps_both_processes() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let navigation = Navigation::get(format!("http://{}/stall", listener.local_addr().unwrap()));
    let (accepted_tx, accepted_rx) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        let mut stream = accept_request(&listener, "/stall");
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        accepted_tx.send(()).unwrap();
        let mut byte = [0];
        assert_eq!(
            stream.read(&mut byte).unwrap(),
            0,
            "broker cancellation closes its HTTP socket"
        );
    });
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 106).unwrap();
    let renderer_pid = client.pid();
    let broker_pid = Cell::new(None);
    let started = Instant::now();
    let result = client.exchange(WorkerCommand::Load { navigation }, || {
        // Children belong to the task which spawned them, so this remains
        // deterministic even while other integration tests run in parallel.
        for pid in fs::read_to_string("/proc/thread-self/children")
            .unwrap()
            .split_whitespace()
        {
            if let Ok(arguments) = fs::read(format!("/proc/{pid}/cmdline"))
                && arguments
                    .split(|&b| b == 0)
                    .any(|arg| arg == b"--resource-broker")
            {
                broker_pid.set(Some(pid.parse::<u32>().unwrap()));
            }
        }
        // Cancellation is intentionally a one-shot notification. Observing it
        // inside a broker exchange must also cancel the outer renderer request.
        accepted_rx.try_recv().is_ok() || started.elapsed() > Duration::from_secs(4)
    });
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(3));
    let broker_pid = broker_pid
        .get()
        .expect("the request reached a resource broker");
    assert!(!process_exists(renderer_pid));
    assert!(!process_exists(broker_pid));
    assert!(client.broker_pid().is_none());
    server.join().unwrap();
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and the confined broker's permitted loopback TCP client"]
fn broker_http_load_resolves_and_fetches_without_spawning_threads() {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let navigation = Navigation::get(format!("http://{address}/fixture"));
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 104)
        .expect("confined HTTP worker");
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(
                        Instant::now() < deadline,
                        "worker did not connect to HTTP fixture"
                    );
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("HTTP fixture accept failed: {error}"),
            }
        };
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let mut request = Vec::new();
        let read_deadline = Instant::now() + Duration::from_secs(2);
        while !request.ends_with(b"\r\n\r\n") {
            assert!(request.len() < 16_384, "oversized fixture request");
            let remaining = read_deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "HTTP fixture request timed out");
            stream.set_read_timeout(Some(remaining)).unwrap();
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        let request = String::from_utf8(request).unwrap();
        assert!(
            request.starts_with("GET /fixture HTTP/1.1\r\n"),
            "{request}"
        );
        assert!(
            request
                .to_ascii_lowercase()
                .contains(&format!("host: {address}\r\n"))
        );
        let body = "<!doctype html><title>Confined HTTP</title><h1>TCP client works</h1>";
        write!(stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ).unwrap();
    });
    load(&mut client, &navigation);
    let snapshot = render(&mut client);
    server.join().expect("bounded HTTP fixture server");
    assert_eq!(snapshot.title, "Confined HTTP");
    assert_eq!(snapshot.url, navigation.address);
    assert_eq!(
        snapshot
            .document
            .text_content(snapshot.document.query_selector("h1").unwrap()),
        "TCP client works"
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn edits_events_clicks_forms_and_fragments_round_trip_without_reloading() {
    let fixture = Fixture::new(
        r##"<!doctype html><title>Interactive worker</title>
        <form id=form action="https://example.invalid/submit" method=post>
        <input id=field name=q value=initial>
        <textarea id=notes name=notes>old</textarea>
        <input id=locked readonly value=locked>
        <button id=submit name=submit value=send>Send</button></form>
        <button id=click type=button onclick="document.getElementById('status').textContent='clicked'">Click</button>
        <a id=jump href="#target">Jump</a><h2 id=target>Target</h2><p id=status>ready</p>
        <script>document.getElementById('field').addEventListener('input', function(event) {
          document.getElementById('status').textContent = event.target.value;
        });</script>"##,
    );
    let mut client = fixture.spawn(true, 99);
    load(&mut client, &fixture.navigation);
    let first = render(&mut client);
    let node = |selector| first.document.query_selector(selector).unwrap();
    let field = node("#field");
    let status = node("#status");
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 2,
            node: field,
            value: "edited 🦀".into(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 1,
            node: field,
            value: "stale edit".into(),
        },
    );
    let edited = render(&mut client);
    assert_eq!(edited.processed_edit_sequence, 2);
    assert_eq!(edited.document.attr(field, "value"), Some("edited 🦀"));
    assert_eq!(edited.document.text_content(status), "edited 🦀");
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 3,
            node: node("#locked"),
            value: "blocked".into(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Edit {
            sequence: 4,
            node: node("#notes"),
            value: "line1\nline2".into(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Click {
            node: node("#click"),
        },
    );
    let clicked = render(&mut client);
    assert_eq!(clicked.document.text_content(status), "clicked");
    assert_eq!(
        clicked.document.attr(node("#locked"), "value"),
        Some("locked")
    );
    assert_eq!(clicked.processed_edit_sequence, 4);
    let navigation = client
        .exchange(
            WorkerCommand::Click {
                node: node("#submit"),
            },
            || false,
        )
        .unwrap()
        .navigation
        .expect("form returns navigation without sending a public request");
    assert_eq!(navigation.address, "https://example.invalid/submit");
    let controls: Vec<_> =
        url::form_urlencoded::parse(navigation.form_body.as_ref().unwrap().as_bytes())
            .into_owned()
            .collect();
    assert_eq!(
        controls,
        [
            ("q".into(), "edited 🦀".into()),
            ("notes".into(), "line1\r\nline2".into()),
            ("submit".into(), "send".into())
        ]
    );
    let fragment = client
        .exchange(
            WorkerCommand::Click {
                node: node("#jump"),
            },
            || false,
        )
        .unwrap()
        .navigation
        .unwrap();
    assert_eq!(
        fragment.address,
        format!("{}#target", fixture.navigation.address)
    );
    assert!(fragment.form_body.is_none());
    exchange_empty(
        &mut client,
        WorkerCommand::Fragment {
            address: fragment.address.clone(),
        },
    );
    exchange_empty(
        &mut client,
        WorkerCommand::Fragment {
            address: "https://example.invalid/other".into(),
        },
    );
    let final_snapshot = render(&mut client);
    assert_eq!(final_snapshot.url, fragment.address);
    assert_eq!(final_snapshot.document.text_content(status), "clicked");
    assert_eq!(final_snapshot.generation, 99);
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn cancellation_reaps_child_and_makes_old_client_unavailable() {
    let navigation = Navigation::get("about:blank");
    let mut client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 7).unwrap();
    let pid = client.pid();
    assert!(process_exists(pid));
    let started = Instant::now();
    let cancellation_checks = Cell::new(0);
    let error = client
        .exchange(WorkerCommand::Load { navigation }, || {
            let checks = cancellation_checks.get() + 1;
            cancellation_checks.set(checks);
            // Let the first transaction iteration send the request, then cancel
            // while the reply is still pending.
            checks >= 2
        })
        .err()
        .expect("cancelled exchange must fail");
    assert!(error.contains("cancelled"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!process_exists(pid), "cancellation must reap the worker");
    let error = client
        .exchange(
            WorkerCommand::Render {
                width: 320.0,
                height: 240.0,
            },
            || false,
        )
        .err()
        .expect("cancelled clients cannot be reused");
    assert!(error.contains("unavailable"), "{error}");
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn cancellation_after_sending_run_tasks_reaps_renderer_and_broker() {
    use eris::page::TaskState;
    let fixture = Fixture::new(&format!(
        "<!doctype html><input id=n value=0 readonly>{}<script>let count=0;const n=document.getElementById('n');document.addEventListener('toggle',()=>{{count++;n.value=count;}},true);</script>",
        "<details open></details>".repeat(150)
    ));
    let mut client = fixture.spawn(true, 104);
    load(&mut client, &fixture.navigation);
    let snapshot = render(&mut client);
    assert_eq!(snapshot.task_state, TaskState::Pending);
    let output = snapshot.document.query_selector("#n").unwrap();
    assert_eq!(snapshot.document.attr(output, "value"), Some("64"));
    let renderer_pid = client.pid();
    let broker_pid = client.broker_pid().expect("local document uses a broker");
    assert!(process_exists(renderer_pid));
    assert!(process_exists(broker_pid));

    let started = Instant::now();
    let cancellation_checks = Cell::new(0);
    let error = client
        .exchange(WorkerCommand::RunTasks, || {
            let checks = cancellation_checks.get() + 1;
            cancellation_checks.set(checks);
            // The preceding Render completed and drained the request pipe.
            // The first channel iteration writes this ten-byte framed command;
            // cancel on the next iteration, before reading its response.
            checks >= 2
        })
        .err()
        .expect("an already-sent task transaction must be cancellable");
    assert_eq!(cancellation_checks.get(), 2);
    assert!(error.contains("cancelled"), "{error}");
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(
        !process_exists(renderer_pid),
        "task cancellation must reap the renderer"
    );
    assert!(
        !process_exists(broker_pid),
        "task cancellation must reap the renderer's broker"
    );
    let error = client
        .exchange(WorkerCommand::RunTasks, || false)
        .err()
        .expect("cancelled task clients cannot be reused");
    assert!(error.contains("unavailable"), "{error}");
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches the real confined browser worker"]
fn dropping_an_idle_client_leaves_no_child_process() {
    let navigation = Navigation::get("about:blank");
    let client = WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 8).unwrap();
    let pid = client.pid();
    assert!(process_exists(pid));
    drop(client);
    assert!(
        !process_exists(pid),
        "Drop must kill and reap an idle worker"
    );
}

#[test]
#[ignore = "launches a single-test subprocess and the real worker; Linux procfs required"]
fn unexpected_inherited_descriptor_is_rejected() {
    const HELPER: &str = "ERIS_WORKER_INHERITED_FD_HELPER";
    if std::env::var_os(HELPER).is_some() {
        // The dedicated, single-test subprocess avoids racing parallel tests
        // while the otherwise close-on-exec file temporarily permits inheritance.
        let file = fs::File::open("/dev/null").unwrap();
        let original = rustix::io::fcntl_getfd(&file).unwrap();
        rustix::io::fcntl_setfd(&file, rustix::io::FdFlags::empty()).unwrap();
        // Exercise the worker entry point directly: no launch sanitizer may
        // turn this negative test into a false pass.
        let mut direct = Command::new(BINARY)
            .arg("--page-worker")
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut init = b"ERWA\x00\x00".to_vec();
        init.extend_from_slice(&103u64.to_le_bytes());
        init.push(0); // Scripts disabled: no timezone capability is needed.
        let mut input = direct.stdin.take().unwrap();
        input.write_all(&(init.len() as u32).to_le_bytes()).unwrap();
        input.write_all(&init).unwrap();
        drop(input);
        let output = direct.wait_with_output().unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stdout).contains("unexpected open descriptor"));
        #[cfg(feature = "vulkan-presenter")]
        {
            // The normal parent API deliberately sanitizes before exec. Its
            // worker starts clean without changing the parent's owned file.
            let navigation = Navigation::get("about:blank");
            let mut client =
                WorkerClient::spawn_at(Path::new(BINARY), false, &navigation, 104).unwrap();
            load(&mut client, &navigation);
            let snapshot = render(&mut client);
            assert_eq!(snapshot.generation, 104);
            assert_eq!(
                rustix::io::fcntl_getfd(&file).unwrap(),
                rustix::io::FdFlags::empty()
            );
            assert!(file.metadata().is_ok());
            drop(client);
        }
        #[cfg(not(feature = "vulkan-presenter"))]
        {
            let error = WorkerClient::spawn_at(
                Path::new(BINARY),
                false,
                &Navigation::get("about:blank"),
                103,
            )
            .err()
            .expect("default launches still reject unexpected descriptors");
            assert!(error.contains("unexpected open descriptor"), "{error}");
        }
        rustix::io::fcntl_setfd(&file, original).unwrap();
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "unexpected_inherited_descriptor_is_rejected",
            "--include-ignored",
            "--test-threads=1",
            "--nocapture",
        ])
        .env(HELPER, "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn malformed_initial_frames_and_eof_exit_without_waiting_for_payload() {
    for (input, keep_input_open) in [
        (u32::MAX.to_le_bytes().to_vec(), true),
        ([5u32.to_le_bytes().as_slice(), b"ERW"].concat(), false),
        (Vec::new(), false),
    ] {
        let mut child = Command::new(BINARY)
            .arg("--page-worker")
            .env_clear()
            .current_dir("/")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&input).unwrap();
        stdin.flush().unwrap();
        let _open_input = if keep_input_open {
            Some(stdin)
        } else {
            drop(stdin);
            None
        };
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > Duration::from_secs(2) {
                let _ = child.kill();
                let _ = child.wait();
                panic!("malformed initial IPC did not exit promptly");
            }
            thread::sleep(Duration::from_millis(5));
        };
        assert!(!status.success(), "invalid frame must fail closed");
        let mut output = Vec::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut output)
            .unwrap();
        assert!(
            output.is_empty(),
            "invalid initialization cannot return a ready handshake"
        );
        assert!(!process_exists(child.id()));
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6 and launches fresh image decoders"]
fn image_decoder_confines_decodes_once_and_exits_without_response_body_leaks() {
    struct OwnedChild(std::process::Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn send(input: &mut impl Write, bytes: &[u8]) {
        input
            .write_all(&(bytes.len() as u32).to_le_bytes())
            .unwrap();
        input.write_all(bytes).unwrap();
        input.flush().unwrap();
    }
    fn receive(output: &mut impl Read) -> Vec<u8> {
        let started = Instant::now();
        let mut bytes = Vec::new();
        let mut length = None;
        loop {
            assert!(
                started.elapsed() < Duration::from_secs(5),
                "decoder reply deadline"
            );
            let mut chunk = [0; 128];
            match output.read(&mut chunk) {
                Ok(0) => panic!("decoder closed output before a complete reply"),
                Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) =>
                {
                    thread::sleep(Duration::from_millis(2))
                }
                Err(e) => panic!("decoder pipe: {e}"),
            }
            if bytes.len() >= 4 && length.is_none() {
                let n = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
                assert!((5..=8192).contains(&n));
                length = Some(n);
            }
            if let Some(length) = length
                && bytes.len() >= length + 4
            {
                assert_eq!(bytes.len(), length + 4);
                return bytes[4..].to_vec();
            }
        }
    }
    let svg = b"<svg width='2' height='1'><rect width='2' height='1' fill='#123456'/></svg>";
    for (mime, body, budget, success) in [
        ("image/svg+xml", svg.as_slice(), 8u32, true),
        ("image/svg+xml", svg.as_slice(), 7u32, false),
        (
            "application/json",
            b"{\"private\":\"secret-response\"}".as_slice(),
            64u32,
            false,
        ),
    ] {
        let mut child = OwnedChild(
            Command::new(BINARY)
                .arg("--image-decoder")
                .env_clear()
                .current_dir("/")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let pid = child.0.id();
        let mut input = child.0.stdin.take().unwrap();
        let mut output = child.0.stdout.take().unwrap();
        let flags = rustix::fs::fcntl_getfl(&output).unwrap();
        rustix::fs::fcntl_setfl(&output, flags | rustix::fs::OFlags::NONBLOCK).unwrap();
        send(&mut input, b"ERWA\x06");
        assert_eq!(receive(&mut output), b"ERWA\x02\x01\x00\x00");
        let status = fs::read_to_string(format!("/proc/{pid}/status")).unwrap();
        assert!(status.contains("NoNewPrivs:\t1"));
        assert!(status.contains("Seccomp:\t2"));
        let mut request = b"ERWA\x07".to_vec();
        request.extend_from_slice(&(mime.len() as u32).to_le_bytes());
        request.extend_from_slice(mime.as_bytes());
        request.extend_from_slice(&budget.to_le_bytes());
        request.extend_from_slice(&(body.len() as u32).to_le_bytes());
        request.extend_from_slice(body);
        send(&mut input, &request);
        let response = receive(&mut output);
        assert_eq!(&response[..5], b"ERWA\x08");
        assert_eq!(response[5], u8::from(success));
        if success {
            assert_eq!(u32::from_le_bytes(response[6..10].try_into().unwrap()), 2);
            assert_eq!(u32::from_le_bytes(response[10..14].try_into().unwrap()), 1);
            assert_eq!(
                &response[14..],
                &[0x12, 0x34, 0x56, 255, 0x12, 0x34, 0x56, 255]
            );
        } else {
            assert_eq!(&response[10..], b"image decoding failed");
            assert!(!String::from_utf8_lossy(&response).contains("secret-response"));
        }
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                break status;
            }
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "decoder must exit after one response"
            );
            thread::sleep(Duration::from_millis(2));
        };
        assert!(status.success());
        assert!(
            !process_exists(pid),
            "decoder reaped after exactly one request"
        );
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_document_append_failure_prefixes_and_root_restoration_survive_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/document-append.html"),
        199,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_dom_own_properties_and_native_fallback_survive_page_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/dom-own-properties.html"),
        200,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_dom_prototypes_and_native_constructors_survive_page_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/dom-prototypes.html"),
        201,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_processing_instruction_and_character_data_survive_page_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_text(
        include_str!("fixtures/processing-instruction.html"),
        202,
        Some(["ready", "6"]),
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_character_data_methods_update_connected_text_and_pixels_through_callbacks() {
    assert_six_scripted_samples_through_worker_with_result_rendering(
        include_str!("fixtures/character-data.html"),
        203,
        Some(["ready", "6"]),
        true,
    );
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_exact_dom_production_roundtrips_units_and_literal_replacement_pixels() {
    use eris::{
        dom::NodeKind,
        graphics::{Canvas, Color, DrawCommand, Fonts},
        page::Page,
    };
    let source = include_str!("fixtures/dom-production.html");
    let fixture = Fixture::new(source);
    let mut client = fixture.spawn(true, 204);
    load(&mut client, &fixture.navigation);
    let fonts = Fonts::new();
    let mut retained = None;
    let mut button = None;
    for (index, (state, color, units, projected, result)) in [
        ("ready", 0x008000, [65, 55296, 66, 56320], "A�B�", "ready"),
        ("clicked", 0x0000ff, [67, 56320, 68, 55296], "C�D�", "6"),
    ]
    .into_iter()
    .enumerate()
    {
        if index == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 204);
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
        let body = snapshot.document.query_selector("body").unwrap();
        assert_eq!(snapshot.document.attr(body, "class"), Some(state));
        button = Some(snapshot.document.query_selector("#change").unwrap());
        let host = snapshot.document.query_selector("#exact").unwrap();
        let status = snapshot.document.query_selector("#result").unwrap();
        assert_eq!(snapshot.document.nodes[host].children.len(), 1);
        assert_eq!(snapshot.document.nodes[status].children.len(), 1);
        let text = snapshot.document.nodes[host].children[0];
        let status_text = snapshot.document.nodes[status].children[0];
        if let Some(previous) = retained {
            assert_eq!(
                (text, status_text),
                previous,
                "wire snapshots retain Text IDs"
            );
        } else {
            retained = Some((text, status_text));
        }
        assert_eq!(snapshot.document.nodes[text].parent, Some(host));
        let NodeKind::Text(data) = &snapshot.document.nodes[text].kind else {
            panic!("wire snapshot must contain the real produced Text");
        };
        assert_eq!(data.raw_units(), Some(units.as_slice()));
        assert_eq!(snapshot.document.text_content(status), result);
        for expected in [projected, result] {
            assert!(snapshot.layout.commands.iter().any(
                |command| matches!(command, DrawCommand::Text { text, .. } if text == expected)
            ));
        }

        // A scalar literal reference isolates presentation from exact wire data.
        // No fixture JavaScript is executed by this reference Page.
        let mut reference = Page::from_html(
            Url::parse(&fixture.navigation.address).unwrap(),
            source,
            false,
        );
        reference.document = snapshot.document.clone();
        // This public scalar mutator performs replacement-aware DOM admission.
        // Verify its complete result so parser-style clamping cannot hide a
        // failed reference setup; both old and literal payloads occupy 8 bytes.
        reference.document.set_text_content(text, projected);
        let NodeKind::Text(reference_data) = &reference.document.nodes[text].kind else {
            panic!("reference must retain a Text node");
        };
        assert_eq!(reference_data.scalar(), Some(projected));
        assert_eq!(reference.document.nodes[text].parent, Some(host));
        assert_eq!(
            reference.document.nodes.len(),
            snapshot.document.nodes.len()
        );
        assert_eq!(
            reference.document.retained_bytes(),
            snapshot.document.retained_bytes()
        );
        let mut expected = Canvas::new(320, 240).unwrap();
        expected.clear(Color::WHITE);
        expected.paint(
            &reference.layout(320.0, 240.0, &fonts).commands,
            &fonts,
            &reference.images,
            0.0,
            0.0,
        );
        let mut actual = Canvas::new(320, 240).unwrap();
        actual.clear(Color::WHITE);
        actual.paint(
            &snapshot.layout.commands,
            &fonts,
            &snapshot.images,
            0.0,
            0.0,
        );
        assert!(!actual.exhausted() && !expected.exhausted());
        assert_eq!(actual.pixels, expected.pixels, "{state} literal reference");
        assert!(
            actual.pixels[100 * 320..135 * 320]
                .iter()
                .any(|pixel| *pixel != 0xffffff)
        );
        for x in [10, 60, 110, 160, 210, 260] {
            assert_eq!(actual.pixels[10 * 320 + x], color, "{state}, x={x}");
        }
        let NodeKind::Text(data) = &snapshot.document.nodes[text].kind else {
            unreachable!()
        };
        assert_eq!(
            data.raw_units(),
            Some(units.as_slice()),
            "decoded payload remains exact"
        );
    }
}

#[path = "support/node_data.rs"]
mod node_data_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_node_data_accessors_preserve_replacement_identity_and_exact_snapshots() {
    let fixture = Fixture::new(node_data_witness::EXACT_HTML);
    let mut client = fixture.spawn(true, 205);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 205);
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
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(node_data_witness::check_exact(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_node_data_replacement_updates_connected_metadata_and_style() {
    let fixture = Fixture::new(node_data_witness::EFFECTS_HTML);
    let mut client = fixture.spawn(true, 206);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 206);
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
        assert_eq!(snapshot.title, ["ready", "changed"][phase]);
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(node_data_witness::check_effects(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[path = "support/append_domstrings.rs"]
mod append_domstrings_witness;

#[path = "support/document_title.rs"]
mod document_title_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_document_title_preserves_exact_snapshots_and_scalar_metadata_cap() {
    let fixture = Fixture::new(document_title_witness::HTML);
    let mut client = fixture.spawn(true, 208);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..3 {
        if phase > 0 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 208);
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
        assert_eq!(
            snapshot.title,
            document_title_witness::metadata(phase, snapshot.document.url().as_str(), true)
        );
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(document_title_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_append_exact_strings_preserve_nodes_units_and_restored_root() {
    let fixture = Fixture::new(append_domstrings_witness::HTML);
    let mut client = fixture.spawn(true, 207);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 207);
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
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(append_domstrings_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[path = "support/text_operations.rs"]
mod text_operations_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_text_split_preserves_exact_nodes_and_refreshes_metadata_and_pixels() {
    let fixture = Fixture::new(text_operations_witness::HTML);
    let mut client = fixture.spawn(true, 209);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 209);
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
        assert_eq!(snapshot.title, text_operations_witness::metadata(phase));
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(text_operations_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[path = "support/node_normalize.rs"]
mod node_normalize_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_normalize_preserves_survivor_and_removed_node_snapshots() {
    let fixture = Fixture::new(node_normalize_witness::HTML);
    let mut client = fixture.spawn(true, 210);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 210);
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
        assert_eq!(snapshot.title, node_normalize_witness::metadata(phase));
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(node_normalize_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[path = "support/node_predicates.rs"]
mod node_predicates_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_node_predicates_follow_live_moves_and_detached_snapshots() {
    let fixture = Fixture::new(node_predicates_witness::HTML);
    let mut client = fixture.spawn(true, 211);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 211);
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
        assert_eq!(snapshot.title, node_predicates_witness::metadata(phase));
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(node_predicates_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[path = "support/node_root.rs"]
mod node_root_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_node_root_reads_fresh_links_after_options_callbacks() {
    let fixture = Fixture::new(node_root_witness::HTML);
    let mut client = fixture.spawn(true, 212);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 212);
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
        assert_eq!(snapshot.title, node_root_witness::metadata(phase));
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(node_root_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[path = "support/node_equality.rs"]
mod node_equality_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_node_equality_tracks_exact_structural_changes() {
    let fixture = Fixture::new(node_equality_witness::HTML);
    let mut client = fixture.spawn(true, 213);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 213);
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
        assert_eq!(snapshot.title, node_equality_witness::metadata(phase));
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(node_equality_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}

#[path = "support/node_connected.rs"]
mod node_connected_witness;

#[test]
#[ignore = "requires Linux Landlock ABI 6; launches a confined renderer and broker"]
fn confined_node_connected_tracks_live_tree_membership() {
    let fixture = Fixture::new(node_connected_witness::HTML);
    let mut client = fixture.spawn(true, 214);
    load(&mut client, &fixture.navigation);
    let fonts = eris::graphics::Fonts::new();
    let mut previous = None;
    let mut button = None;
    for phase in 0..2 {
        if phase == 1 {
            exchange_empty(
                &mut client,
                WorkerCommand::Click {
                    node: button.unwrap(),
                },
            );
        }
        let snapshot = render(&mut client);
        assert_eq!(snapshot.generation, 214);
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
        assert_eq!(snapshot.title, node_connected_witness::metadata(phase));
        button = Some(snapshot.document.query_selector("#change").unwrap());
        previous = Some(node_connected_witness::check(
            &snapshot.document,
            &snapshot.layout,
            &snapshot.images,
            &fonts,
            phase,
            previous.as_ref(),
        ));
    }
}
