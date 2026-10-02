#![forbid(unsafe_code)]

use eris::{
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore, Rect},
    page::{Navigation, TaskState},
    worker::{Command as WorkerCommand, Snapshot, WorkerClient},
};
use eris_vulkan_raster_prototype::{
    Frame, MAX_HEIGHT, MAX_WIDTH, Plan, Result,
    browser_adapter::{self, BridgePlan, FallbackReason},
    browser_fixtures::{self, BrowserFixture, FixtureInput, WorkerKind},
    gpu,
};
use std::{
    fmt::Write as _,
    io::Write as _,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

mod checker_support;
use checker_support::{
    check_capture_deadline, dump_snapshot, ensure_no_children, file_address, fixture_path,
    print_snapshot, read_gpu_grant, reserve,
};

const MAX_CASES: usize = 16;
const MAX_RETAINED_GPU_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RETAINED_CPU_BYTES: usize = 16 * 320 * 240 * 4;
const MAX_RETAINED_DRAWS: usize = 16 * 257;
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(180);

fn rect_eq(rect: Rect, values: [f32; 4]) -> bool {
    [rect.x, rect.y, rect.width, rect.height] == values
}

#[derive(Clone, Copy)]
enum ExpectedPrimitive {
    Rect([f32; 4], Color, f32),
    Image([f32; 4], &'static str),
}

fn verify_worker(snapshot: &Snapshot, frame: Frame, kind: WorkerKind) -> Result<()> {
    let white = ExpectedPrimitive::Rect(
        [0.0, 0.0, frame.width as f32, frame.height as f32],
        Color::WHITE,
        0.0,
    );
    let red = Color::rgb(255, 0, 0);
    let blue = Color::rgb(0, 0, 255);
    let green = Color::rgb(0, 255, 0);
    let (expected, expected_count) = match kind {
        WorkerKind::OrderedRectangles => (
            [
                white,
                ExpectedPrimitive::Rect([1.0, 1.0, 4.0, 2.0], red, 0.0),
                ExpectedPrimitive::Rect([3.0, 0.0, 2.0, 3.0], blue, 0.0),
                white,
            ],
            3,
        ),
        WorkerKind::FixedChild => (
            [
                white,
                ExpectedPrimitive::Rect([1.0, 1.0, 2.0, 2.0], red, 0.0),
                ExpectedPrimitive::Rect([2.0, 2.0, 2.0, 2.0], green, 0.0),
                ExpectedPrimitive::Rect([5.0, 0.0, 2.0, 1.0], blue, 0.0),
            ],
            4,
        ),
        WorkerKind::RepeatedImage => (
            [
                white,
                ExpectedPrimitive::Image([0.0, 0.0, 4.0, 1.0], "two-pixels.png"),
                ExpectedPrimitive::Image([2.0, 1.0, 4.0, 1.0], "two-pixels.png"),
                white,
            ],
            3,
        ),
        WorkerKind::MissingImage => (
            [
                white,
                ExpectedPrimitive::Rect([1.0, 0.0, 2.0, 1.0], Color::rgb(236, 238, 242), 0.0),
                ExpectedPrimitive::Image([1.0, 0.0, 2.0, 1.0], "deliberately-absent.png"),
                ExpectedPrimitive::Rect([3.0, 1.0, 1.0, 1.0], blue, 0.0),
            ],
            4,
        ),
        WorkerKind::Rounded => (
            [
                white,
                ExpectedPrimitive::Rect([1.0, 0.0, 2.0, 2.0], blue, 1.0),
                ExpectedPrimitive::Rect([0.0, 2.0, 1.0, 1.0], red, 0.0),
                white,
            ],
            3,
        ),
    };
    #[derive(Clone, Copy)]
    enum Scope {
        Clip(Rect),
        Fixed,
    }
    let mut scopes = [None; 32];
    let mut depth = 0usize;
    let mut fixed = 0usize;
    let mut seen = 0usize;
    for command in &snapshot.layout.commands {
        match command {
            DrawCommand::PushClip { rect } => {
                if depth == scopes.len() {
                    return Err("worker scope assumption".into());
                }
                scopes[depth] = Some(Scope::Clip(*rect));
                depth += 1;
                continue;
            }
            DrawCommand::PushFixed => {
                if depth == scopes.len() {
                    return Err("worker fixed assumption".into());
                }
                scopes[depth] = Some(Scope::Fixed);
                depth += 1;
                fixed += 1;
                continue;
            }
            DrawCommand::PopClip => {
                if depth == 0 || !matches!(scopes[depth - 1], Some(Scope::Clip(_))) {
                    return Err("worker clip balance".into());
                }
                depth -= 1;
                scopes[depth] = None;
                continue;
            }
            DrawCommand::PopFixed => {
                if depth == 0 || !matches!(scopes[depth - 1], Some(Scope::Fixed)) {
                    return Err("worker fixed balance".into());
                }
                depth -= 1;
                fixed -= 1;
                scopes[depth] = None;
                continue;
            }
            // These benign entries remain in the complete dump, CPU list and
            // adapter input. They are not removed or used to hide unsupported kinds.
            DrawCommand::Rect {
                rect,
                color,
                radius,
            } if (color.a == 0 || rect.width == 0.0 || rect.height == 0.0) && *radius == 0.0 => {
                continue;
            }
            _ => {}
        }
        let wanted = expected[..expected_count]
            .get(seen)
            .ok_or("unexpected worker drawing command")?;
        match (command, wanted) {
            (
                DrawCommand::Rect {
                    rect,
                    color,
                    radius,
                },
                ExpectedPrimitive::Rect(r, c, k),
            ) if rect_eq(*rect, *r) && color == c && radius == k => {
                if matches!(kind, WorkerKind::FixedChild) {
                    if *color == blue && fixed == 0 {
                        return Err("fixed child missing fixed scope".into());
                    }
                    if *color != blue && fixed != 0 {
                        return Err("document rectangle moved to fixed scope".into());
                    }
                    if *color == green
                        && !scopes[..depth].iter().any(
                            |s| matches!(s, Some(Scope::Clip(r)) if rect_eq(*r,[1.0,1.0,2.0,2.0])),
                        )
                    {
                        return Err("green child missing document clip".into());
                    }
                }
            }
            (DrawCommand::Image { rect, key }, ExpectedPrimitive::Image(r, k))
                if rect_eq(*rect, *r) && key.as_str() == *k => {}
            _ => return Err("worker command differs from frozen input assumption".into()),
        }
        seen += 1;
    }
    if depth != 0 || seen != expected_count {
        return Err("worker command inventory mismatch".into());
    }
    match kind {
        WorkerKind::RepeatedImage => {
            if snapshot.images.len() != 1 {
                return Err("worker source inventory mismatch".into());
            }
            let image = snapshot
                .images
                .get("two-pixels.png")
                .ok_or("worker image key mismatch")?;
            if image.width != 2
                || image.height != 1
                || image.rgba != [255, 0, 0, 255, 0, 0, 255, 128]
            {
                return Err("worker decoded bytes differ from literal source".into());
            }
        }
        _ if !snapshot.images.is_empty() => return Err("unexpected worker source".into()),
        _ => {}
    }
    if matches!(kind, WorkerKind::MissingImage)
        && !snapshot
            .diagnostics
            .iter()
            .any(|d| d.contains("deliberately-absent.png"))
    {
        return Err("missing-image diagnostic was not retained".into());
    }
    Ok(())
}

struct Prepared {
    name: &'static str,
    frame: Frame,
    expected: Vec<u32>,
    plan: Option<BridgePlan>,
    fallback: Option<FallbackReason>,
    original_commands: usize,
    image_entries: usize,
}

struct Painted {
    plan: Option<BridgePlan>,
    fallback: Option<FallbackReason>,
}

fn prepare_frame(
    fixture: &BrowserFixture,
    commands: &[DrawCommand],
    images: &ImageStore,
    fonts: &Fonts,
    admission: std::result::Result<BridgePlan, FallbackReason>,
) -> Result<Painted> {
    let (plan, fallback) = match admission {
        Ok(plan) if fixture.expected_fallback.is_none() => (Some(plan), None),
        Err(reason) if fixture.expected_fallback == Some(reason.category()) => (None, Some(reason)),
        Ok(_) => return Err(format!("{}: expected whole-frame fallback", fixture.name)),
        Err(reason) => {
            return Err(format!(
                "{}: unexpected fallback {}",
                fixture.name,
                reason.category()
            ));
        }
    };
    // Only these four original oracle rows specify an exact rejection index.
    // Worker indices and the two limit cases are deliberately not invented.
    let expected_index = match fixture.name {
        "direct-hidden-text-refuses-whole-frame" => Some(2),
        "direct-rounded-refuses-whole-frame" => Some(0),
        "direct-unit-opacity-refuses-whole-frame" => Some(1),
        "direct-transparent-hidden-rounded-still-refuses" => Some(2),
        _ => None,
    };
    if let Some(index) = expected_index
        && fallback.as_ref().and_then(|reason| reason.command_index) != Some(index)
    {
        return Err(format!(
            "{}: fallback command index differs from oracle",
            fixture.name
        ));
    }
    let frame = fixture.frame;
    let mut canvas = Canvas::new(frame.width, frame.height)?;
    canvas.clear(Color::rgb(
        (frame.clear >> 16) as u8,
        (frame.clear >> 8) as u8,
        frame.clear as u8,
    ));
    canvas.set_clip(Rect {
        x: frame.caller_clip.x,
        y: frame.caller_clip.y,
        width: frame.caller_clip.width,
        height: frame.caller_clip.height,
    });
    canvas.paint_with_viewport(
        commands,
        fonts,
        images,
        frame.document_offset,
        frame.viewport_offset,
    );
    if canvas.exhausted() {
        return Err(format!("{}: CPU paint exhausted", fixture.name));
    }
    if canvas.pixels != fixture.expected {
        return Err(format!(
            "{}: CPU output differs from frozen literal target",
            fixture.name
        ));
    }
    // Retain only the independent literal target after the full comparison.
    // Canvas and its pixel allocation are released before the next fixture.
    Ok(Painted { plan, fallback })
}

fn capture(
    fixtures: Vec<BrowserFixture>,
    browser: &Path,
    directory: &Path,
) -> Result<Vec<Prepared>> {
    if fixtures.len() != MAX_CASES {
        return Err("checker case inventory".into());
    }
    // Preflight worst-case cumulative plan storage/draws before any plan can
    // grow. Each private Plan independently retains at most the unchanged
    // explicit per-frame GPU bound; no more than 257 draws can be generated.
    if fixtures
        .len()
        .checked_mul(1_048_576)
        .is_none_or(|n| n as u64 > MAX_RETAINED_GPU_BYTES)
        || fixtures
            .len()
            .checked_mul(257)
            .is_none_or(|n| n > MAX_RETAINED_DRAWS)
    {
        return Err("checker aggregate plan preflight".into());
    }
    let deadline = Instant::now() + CAPTURE_TIMEOUT;
    let fonts = Fonts::new();
    let mut prepared = reserve(MAX_CASES)?;
    let mut cpu_bytes = 0usize;
    let mut gpu_bytes = 0u64;
    let mut draws = 0usize;
    let mut worker_cases = 0usize;
    for (index, fixture) in fixtures.into_iter().enumerate() {
        check_capture_deadline(deadline)?;
        let frame = fixture.frame;
        if frame.width == 0
            || frame.height == 0
            || frame.width > MAX_WIDTH
            || frame.height > MAX_HEIGHT
        {
            return Err("checker viewport bound".into());
        }
        let bytes = (frame.width as usize)
            .checked_mul(frame.height as usize)
            .and_then(|n| n.checked_mul(4))
            .ok_or("checker pixel overflow")?;
        cpu_bytes = cpu_bytes
            .checked_add(bytes)
            .ok_or("checker CPU total overflow")?;
        if cpu_bytes > MAX_RETAINED_CPU_BYTES
            || fixture.expected.len().checked_mul(4) != Some(bytes)
        {
            return Err("checker retained CPU pixel bound".into());
        }
        let (plan, fallback, original_commands, image_entries) = match &fixture.input {
            FixtureInput::Direct { commands, images } => {
                let painted = prepare_frame(
                    &fixture,
                    commands,
                    images,
                    &fonts,
                    browser_adapter::plan_display_list(commands, images, frame),
                )?;
                (painted.plan, painted.fallback, commands.len(), images.len())
            }
            FixtureInput::Worker { html_file, kind } => {
                worker_cases += 1;
                ensure_no_children()?;
                let address = file_address(&fixture_path(directory, html_file)?)?;
                let generation = 2_000 + index as u64;
                let navigation = Navigation::get(address.clone());
                let mut client = WorkerClient::spawn_at_cancellable(
                    browser,
                    false,
                    &navigation,
                    generation,
                    || Instant::now() >= deadline,
                )?;
                let load = client.exchange(WorkerCommand::Load { navigation }, || {
                    Instant::now() >= deadline
                })?;
                if load.snapshot.is_some() || load.navigation.is_some() {
                    return Err("unexpected worker Load reply".into());
                }
                let mut render = client.exchange(
                    WorkerCommand::Render {
                        width: frame.width as f32,
                        height: frame.height as f32,
                    },
                    || Instant::now() >= deadline,
                )?;
                if render.navigation.is_some() {
                    return Err("unexpected worker Render navigation".into());
                }
                let snapshot = render.snapshot.take().ok_or("missing worker snapshot")?;
                print_snapshot(fixture.name, &dump_snapshot(&snapshot, frame)?)?;
                if snapshot.generation != generation
                    || snapshot.processed_edit_sequence != 0
                    || snapshot.task_state != TaskState::Idle
                    || snapshot.url != address
                {
                    return Err("worker metadata differs from capture request".into());
                }
                verify_worker(&snapshot, frame, *kind)?;
                let painted = prepare_frame(
                    &fixture,
                    &snapshot.layout.commands,
                    &snapshot.images,
                    &fonts,
                    browser_adapter::plan_snapshot(&snapshot, frame),
                )?;
                let counts = (snapshot.layout.commands.len(), snapshot.images.len());
                drop(snapshot);
                // Existing Channel::Drop may block after kill. The outer
                // dedicated subreaper handles a stalled capture; no GPU exists.
                drop(client);
                check_capture_deadline(deadline)?;
                ensure_no_children()?;
                (painted.plan, painted.fallback, counts.0, counts.1)
            }
        };
        if let Some(p) = &plan {
            gpu_bytes = gpu_bytes
                .checked_add(p.plan().gpu_buffer_bytes())
                .ok_or("checker GPU total overflow")?;
            draws = draws
                .checked_add(p.plan().draws().len())
                .ok_or("checker draw total overflow")?;
            if gpu_bytes > MAX_RETAINED_GPU_BYTES || draws > MAX_RETAINED_DRAWS {
                return Err("checker retained plan bound".into());
            }
        }
        prepared.push(Prepared {
            name: fixture.name,
            frame,
            expected: fixture.expected,
            plan,
            fallback,
            original_commands,
            image_entries,
        });
    }
    let gpu_count = prepared.iter().filter(|p| p.plan.is_some()).count();
    let pixels: usize = prepared.iter().map(|p| p.expected.len()).sum();
    if worker_cases != 5 || gpu_count != 9 || prepared.len() - gpu_count != 7 || pixels != 197 {
        return Err("frozen checker totals differ".into());
    }
    check_capture_deadline(deadline)?;
    let tasks = ensure_no_children()?;
    println!("CAPTURE_COMPLETE cases=16 worker_cases=5 own_tasks={tasks} owned_children=0");
    Ok(prepared)
}

fn print_case(case: &Prepared, gpu_checked: bool) -> Result<()> {
    let (path, reason, command_index, draws, invocations, gpu_bytes) =
        match (&case.plan, &case.fallback) {
            (Some(plan), None) if gpu_checked => (
                "gpu",
                "none",
                None,
                plan.plan().draws().len(),
                plan.plan().invocations(),
                plan.plan().gpu_buffer_bytes(),
            ),
            (None, Some(reason)) if !gpu_checked => (
                "cpu-fallback",
                reason.category(),
                reason.command_index,
                0,
                0,
                0,
            ),
            _ => return Err("checker output path mismatch".into()),
        };
    let mut packed = String::new();
    packed
        .try_reserve_exact(
            case.expected
                .len()
                .checked_mul(9)
                .ok_or("pixel output overflow")?,
        )
        .map_err(|_| "pixel output allocation")?;
    for (i, pixel) in case.expected.iter().enumerate() {
        if i != 0 {
            packed.push(',');
        }
        write!(packed, "{pixel:08x}").map_err(|_| "pixel output formatting")?;
    }
    let command_index = OptionalCount(command_index);
    let stats = case.plan.as_ref().map(BridgePlan::stats);
    let lowered_commands = OptionalCount(stats.map(|s| s.lowered_commands));
    let unique_sources = OptionalCount(stats.map(|s| s.unique_sources));
    let referenced_sources = OptionalCount(stats.map(|s| s.referenced_sources));
    let total_rgba_bytes = OptionalCount(stats.map(|s| s.total_rgba_bytes));
    let referenced_rgba_bytes = OptionalCount(stats.map(|s| s.referenced_rgba_bytes));
    let missing_images = OptionalCount(stats.map(|s| s.missing_images));
    println!(
        "CASE {} path={path} reason={reason} command_index={command_index} width={} height={} commands={} image_entries={} lowered_commands={lowered_commands} unique_sources={unique_sources} referenced_sources={referenced_sources} total_rgba_bytes={total_rgba_bytes} referenced_rgba_bytes={referenced_rgba_bytes} missing_images={missing_images} draws={draws} invocations={invocations} gpu_buffers={gpu_bytes} compared_bytes={} packed={packed} exact=true",
        case.name,
        case.frame.width,
        case.frame.height,
        case.original_commands,
        case.image_entries,
        case.expected.len() * 4
    );
    Ok(())
}

struct OptionalCount(Option<usize>);
impl std::fmt::Display for OptionalCount {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(count) => write!(out, "{count}"),
            None => out.write_str("none"),
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).take(8).collect();
    if args.as_slice() == ["--list"] {
        ensure_no_children()?;
        return gpu::run_plans(None, &[], |_, _, _| Ok(()));
    }
    let (index, browser, directory) = match args.as_slice() {
        [mode, index, browser_flag, browser, fixtures_flag, fixtures]
            if mode == "--adapter"
                && browser_flag == "--browser"
                && fixtures_flag == "--fixtures" =>
        {
            (
                index.parse::<usize>().map_err(|_| "adapter index")?,
                PathBuf::from(browser)
                    .canonicalize()
                    .map_err(|e| format!("browser path: {e}"))?,
                PathBuf::from(fixtures)
                    .canonicalize()
                    .map_err(|e| format!("fixture directory: {e}"))?,
            )
        }
        _ => {
            return Err(
                "usage: browser-check --list | --adapter INDEX --browser PATH --fixtures DIR"
                    .into(),
            );
        }
    };
    if !browser.is_file() || !directory.is_dir() {
        return Err("checker input paths".into());
    }
    let prepared = capture(browser_fixtures::fixtures()?, &browser, &directory)?;
    for case in &prepared {
        if case.fallback.is_some() {
            print_case(case, false)?;
        }
    }
    let mut plans: Vec<(&str, &[u32], &Plan)> = reserve(9)?;
    for case in &prepared {
        if let Some(plan) = &case.plan {
            plans.push((case.name, &case.expected, plan.plan()));
        }
    }
    // No worker handles or snapshots exist beyond this point; the child check
    // is repeated immediately before the first Vulkan operation. The outer
    // subreaper must also confirm that no capture descendant was adopted by
    // it before sending the grant. Its deadline owns a stalled stdin wait.
    ensure_no_children()?;
    std::io::stdout()
        .flush()
        .map_err(|e| format!("capture phase flush: {e}"))?;
    read_gpu_grant(&mut std::io::stdin().lock())?;
    let mut checked = 0usize;
    gpu::run_plans(Some(index), &plans, |name, _, _| {
        let case = prepared
            .iter()
            .find(|c| c.name == name)
            .ok_or("unknown GPU callback")?;
        print_case(case, true)?;
        checked += 1;
        Ok(())
    })?;
    if checked != 9 {
        return Err("GPU comparison count mismatch".into());
    }
    println!(
        "BROWSER_COMPLETE adapter={index} cases=16 gpu=9 fallback=7 pixels=197 compared_bytes=788 exact=true custom_wgsl=true"
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("FAIL {error}");
            ExitCode::FAILURE
        }
    }
}
