#![forbid(unsafe_code)]

use eris::{
    graphics::{Canvas, Color, DrawCommand, Fonts, Rect},
    page::{Navigation, TaskState},
    worker::{Command as WorkerCommand, Snapshot, WorkerClient},
};
use eris_vulkan_raster_prototype::{
    Frame, Plan, Result,
    browser_adapter::{FallbackReason, FontBridgePlan, plan_snapshot_with_fonts},
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
#[path = "../../worker-text-fixtures/fixture_inventory.rs"]
mod fixture_inventory;
use fixture_inventory::{DrawingKind, WorkerTextFixture};

const CASES: usize = 7;
const GPU_CASES: usize = 6;
const FALLBACK_CASES: usize = 1;
const MAX_WIDTH: u32 = 160;
const MAX_HEIGHT: u32 = 80;
const MAX_REFERENCE_BYTES: usize = CASES * MAX_WIDTH as usize * MAX_HEIGHT as usize * 4;
const MAX_GPU_BYTES: u64 = CASES as u64 * 1_048_576;
const MAX_DRAWS: usize = CASES * 257;
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Clone, Copy)]
enum Scope {
    Clip(Rect),
    Fixed { clip: Rect, offset: (f32, f32) },
}

#[derive(Clone, Copy)]
struct TextInput {
    runs: usize,
    bytes: usize,
    scalars: usize,
}

fn graphics_rect(rect: eris_vulkan_raster_prototype::Rect) -> Rect {
    Rect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

// These checks verify the separately frozen input assumptions. Every original
// command, including benign invisible rectangles, remains in the capture dump,
// original Canvas input and fonts-aware adapter input.
fn verify_worker(snapshot: &Snapshot, fixture: &WorkerTextFixture) -> Result<TextInput> {
    if snapshot.title != fixture.name
        || fixture.texts.is_empty()
        || fixture.texts.len() > 256
        || fixture.drawing_order.len() > 256
        || fixture.images.len() > 256
        || snapshot.images.len() != fixture.images.len()
    {
        return Err("worker text fixture inventory/title".into());
    }
    let frame = fixture.frame;
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width: frame.width as f32,
        height: frame.height as f32,
    };
    let caller = graphics_rect(frame.caller_clip).intersect(viewport);
    let mut clip = caller;
    let mut offset = frame.document_offset;
    let mut scopes = [None; 32];
    let (mut depth, mut fixed, mut text_index, mut drawing_index) = (0, 0, 0, 0);
    let mut input = TextInput {
        runs: 0,
        bytes: 0,
        scalars: 0,
    };
    let mut references = [0usize; 256];
    for command in &snapshot.layout.commands {
        let kind = match command {
            DrawCommand::PushClip { rect } => {
                if depth == scopes.len()
                    || ![rect.x, rect.y, rect.width, rect.height]
                        .into_iter()
                        .all(f32::is_finite)
                {
                    return Err("worker clip assumption".into());
                }
                scopes[depth] = Some(Scope::Clip(clip));
                depth += 1;
                clip = clip
                    .intersect(rect.translated(offset.0, offset.1))
                    .intersect(viewport);
                continue;
            }
            DrawCommand::PushFixed => {
                if depth == scopes.len() {
                    return Err("worker fixed depth".into());
                }
                scopes[depth] = Some(Scope::Fixed { clip, offset });
                depth += 1;
                fixed += 1;
                clip = caller;
                offset = frame.viewport_offset;
                continue;
            }
            DrawCommand::PopClip => {
                if depth == 0 {
                    return Err("worker clip underflow".into());
                }
                depth -= 1;
                let Some(Scope::Clip(old)) = scopes[depth].take() else {
                    return Err("worker clip scope mismatch".into());
                };
                clip = old;
                continue;
            }
            DrawCommand::PopFixed => {
                if depth == 0 {
                    return Err("worker fixed underflow".into());
                }
                depth -= 1;
                let Some(Scope::Fixed {
                    clip: old,
                    offset: old_offset,
                }) = scopes[depth].take()
                else {
                    return Err("worker fixed scope mismatch".into());
                };
                clip = old;
                offset = old_offset;
                fixed -= 1;
                continue;
            }
            DrawCommand::PushOpacity { .. } | DrawCommand::PopOpacity => {
                return Err("unexpected worker opacity scope".into());
            }
            DrawCommand::Text {
                x,
                y,
                text,
                size,
                color,
                bold,
                italic,
                monospace,
            } => {
                let wanted = fixture
                    .texts
                    .get(text_index)
                    .ok_or("unexpected worker text run")?;
                if ![*x, *y, *size].into_iter().all(f32::is_finite)
                    || text.as_str() != wanted.text
                    || size.to_bits() != wanted.size.to_bits()
                    || [color.r, color.g, color.b, color.a] != wanted.rgba
                    || *bold != wanted.bold
                    || *italic != wanted.italic
                    || *monospace != wanted.monospace
                    || (fixed != 0) != wanted.fixed
                    || (clip != caller) != wanted.clipped
                {
                    return Err(format!(
                        "{}: worker text/style/scope assumption at run {text_index}",
                        fixture.name
                    ));
                }
                input.bytes = input
                    .bytes
                    .checked_add(text.len())
                    .filter(|n| *n <= 65536)
                    .ok_or("worker text bytes")?;
                input.scalars += text.chars().count();
                if input.scalars > 4096 {
                    return Err("worker text scalar bound".into());
                }
                input.runs += 1;
                text_index += 1;
                DrawingKind::Text
            }
            DrawCommand::Rect {
                rect,
                color,
                radius,
            } => {
                if ![rect.x, rect.y, rect.width, rect.height, *radius]
                    .into_iter()
                    .all(f32::is_finite)
                {
                    return Err("worker rectangle geometry".into());
                }
                if (color.a == 0 || rect.width == 0.0 || rect.height == 0.0) && *radius == 0.0 {
                    continue;
                }
                DrawingKind::Rect
            }
            DrawCommand::Image { rect, key } => {
                if ![rect.x, rect.y, rect.width, rect.height]
                    .into_iter()
                    .all(f32::is_finite)
                {
                    return Err("worker image geometry".into());
                }
                let id = fixture
                    .images
                    .iter()
                    .position(|image| image.key == key.as_str())
                    .ok_or("unexpected worker image key")?;
                references[id] += 1;
                DrawingKind::Image
            }
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                ..
            } => {
                if ![*x1, *y1, *x2, *y2, *width].into_iter().all(f32::is_finite) {
                    return Err("worker line geometry".into());
                }
                DrawingKind::Line
            }
        };
        if fixture.drawing_order.get(drawing_index) != Some(&kind) {
            return Err(format!(
                "{}: worker drawing order at {drawing_index}",
                fixture.name
            ));
        }
        drawing_index += 1;
    }
    if depth != 0
        || fixed != 0
        || text_index != fixture.texts.len()
        || drawing_index != fixture.drawing_order.len()
    {
        return Err("worker text/scope/drawing inventory mismatch".into());
    }
    for (index, expected) in fixture.images.iter().enumerate() {
        let image = snapshot
            .images
            .get(expected.key)
            .ok_or("missing worker source image")?;
        if image.width != expected.width
            || image.height != expected.height
            || image.rgba != expected.rgba
            || references[index] != expected.references
        {
            return Err("worker decoded image or reference count mismatch".into());
        }
    }
    Ok(input)
}

struct Prepared {
    name: &'static str,
    frame: Frame,
    pixels: Vec<u32>,
    plan: Option<FontBridgePlan>,
    fallback: Option<FallbackReason>,
    commands: usize,
    image_entries: usize,
    text: TextInput,
}

fn paint(snapshot: &Snapshot, frame: Frame, fonts: &Fonts) -> Result<Vec<u32>> {
    let mut canvas = Canvas::new(frame.width, frame.height)?;
    canvas.clear(Color::rgb(
        (frame.clear >> 16) as u8,
        (frame.clear >> 8) as u8,
        frame.clear as u8,
    ));
    canvas.set_clip(graphics_rect(frame.caller_clip));
    canvas.paint_with_viewport(
        &snapshot.layout.commands,
        fonts,
        &snapshot.images,
        frame.document_offset,
        frame.viewport_offset,
    );
    if canvas.exhausted() {
        return Err("worker text CPU paint exhausted".into());
    }
    Ok(canvas.pixels)
}

fn same_plans(a: &FontBridgePlan, b: &FontBridgePlan) -> bool {
    a.stats() == b.stats()
        && a.text_stats() == b.text_stats()
        && a.plan().draws() == b.plan().draws()
        && a.plan().invocations() == b.plan().invocations()
        && a.plan().gpu_buffer_bytes() == b.plan().gpu_buffer_bytes()
        && a.plan().parameters() == b.plan().parameters()
        && a.plan().input_bytes() == b.plan().input_bytes()
}

fn pixel_hex(pixels: &[u32]) -> Result<String> {
    if pixels.len() > MAX_WIDTH as usize * MAX_HEIGHT as usize
        || pixels.iter().any(|p| *p > 0xffffff)
    {
        return Err("worker text reference pixels".into());
    }
    let mut text = String::new();
    text.try_reserve_exact(pixels.len() * 8)
        .map_err(|_| "pixel hex allocation")?;
    for pixel in pixels {
        for byte in pixel.to_le_bytes() {
            write!(text, "{byte:02x}").map_err(|_| "pixel hex formatting")?;
        }
    }
    Ok(text)
}

struct Count(Option<u64>);
impl std::fmt::Display for Count {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(n) => write!(out, "{n}"),
            None => out.write_str("none"),
        }
    }
}
fn count(value: Option<usize>) -> Count {
    Count(value.map(|n| n as u64))
}

fn print_ready(case: &Prepared) -> Result<()> {
    let s = case.plan.as_ref().map(FontBridgePlan::stats);
    let t = case.plan.as_ref().map(FontBridgePlan::text_stats);
    let p = case.plan.as_ref().map(FontBridgePlan::plan);
    let path = if p.is_some() { "gpu" } else { "cpu-fallback" };
    let reason = case.fallback.map_or("none", |f| f.category());
    let index = count(case.fallback.and_then(|f| f.command_index));
    println!(
        "TEXT_READY {} path={path} reason={reason} command_index={index} width={} height={} commands={} image_entries={} text_runs={} bytes={} scalars={} lowered_commands={} unique_sources={} referenced_sources={} total_rgba_bytes={} referenced_rgba_bytes={} missing_images={} occurrences={} masks={} coverage={} cold_work={} scratch={} rows={} cpu_upper={} gpu_upper={} draws={} invocations={} gpu_buffers={} reference_bytes={} cpu_passes=2 cold_warm=true reference=canvas-snapshot pixels_le_hex={}",
        case.name,
        case.frame.width,
        case.frame.height,
        case.commands,
        case.image_entries,
        case.text.runs,
        case.text.bytes,
        case.text.scalars,
        count(s.map(|s| s.lowered_commands)),
        count(s.map(|s| s.unique_sources)),
        count(s.map(|s| s.referenced_sources)),
        count(s.map(|s| s.total_rgba_bytes)),
        count(s.map(|s| s.referenced_rgba_bytes)),
        count(s.map(|s| s.missing_images)),
        count(t.map(|t| t.preparation.occurrences)),
        count(t.map(|t| t.preparation.cold_requests)),
        count(t.map(|t| t.preparation.unique_coverage_bytes)),
        Count(t.map(|t| t.preparation.cold_mask_work)),
        count(t.map(|t| t.preparation.max_raster_scratch_bytes)),
        count(t.map(|t| t.row_entries)),
        Count(t.map(|t| t.cpu_pixel_upper_bound)),
        Count(t.map(|t| t.gpu_buffer_upper_bound)),
        count(p.map(|p| p.draws().len())),
        Count(p.map(|p| p.invocations())),
        Count(p.map(|p| p.gpu_buffer_bytes())),
        case.pixels.len() * 4,
        pixel_hex(&case.pixels)?
    );
    Ok(())
}

fn capture(browser: &Path, directory: &Path) -> Result<Vec<Prepared>> {
    let fixtures = fixture_inventory::fixtures();
    if fixtures.len() != CASES {
        return Err("worker text fixture count".into());
    }
    // Finite inventories precharge worst-case retained storage before any plan.
    if fixtures.len() * 1_048_576 > MAX_GPU_BYTES as usize || fixtures.len() * 257 > MAX_DRAWS {
        return Err("worker text aggregate plan preflight".into());
    }
    let mut reference_bytes = 0usize;
    for (index, fixture) in fixtures.iter().enumerate() {
        let f = fixture.frame;
        if f.width == 0
            || f.height == 0
            || f.width > MAX_WIDTH
            || f.height > MAX_HEIGHT
            || fixture.name.is_empty()
            || fixture.name.len() > 128
            || !fixture
                .name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || fixtures[..index].iter().any(|old| old.name == fixture.name)
        {
            return Err("worker text fixture frame/name bound".into());
        }
        reference_bytes += f.width as usize * f.height as usize * 4;
        if reference_bytes > MAX_REFERENCE_BYTES {
            return Err("worker text reference total".into());
        }
        fixture_path(directory, fixture.html_file)?;
    }
    let deadline = Instant::now() + CAPTURE_TIMEOUT;
    let mut prepared = reserve(CASES)?;
    let (mut gpu_bytes, mut draws) = (0u64, 0usize);
    for (index, fixture) in fixtures.into_iter().enumerate() {
        check_capture_deadline(deadline)?;
        ensure_no_children()?;
        let frame = fixture.frame;
        let address = file_address(&fixture_path(directory, fixture.html_file)?)?;
        let generation = 3000 + index as u64;
        let navigation = Navigation::get(address.clone());
        let mut client =
            WorkerClient::spawn_at_cancellable(browser, false, &navigation, generation, || {
                Instant::now() >= deadline
            })?;
        let load = client.exchange(WorkerCommand::Load { navigation }, || {
            Instant::now() >= deadline
        })?;
        if load.snapshot.is_some() || load.navigation.is_some() {
            return Err("unexpected text worker Load reply".into());
        }
        let mut render = client.exchange(
            WorkerCommand::Render {
                width: frame.width as f32,
                height: frame.height as f32,
            },
            || Instant::now() >= deadline,
        )?;
        if render.navigation.is_some() {
            return Err("unexpected text worker navigation".into());
        }
        let snapshot = render
            .snapshot
            .take()
            .ok_or("missing text worker snapshot")?;
        print_snapshot(fixture.name, &dump_snapshot(&snapshot, frame)?)?;
        if snapshot.generation != generation
            || snapshot.processed_edit_sequence != 0
            || snapshot.task_state != TaskState::Idle
            || snapshot.url != address
        {
            return Err("worker text capture metadata mismatch".into());
        }
        let text = verify_worker(&snapshot, &fixture)?;
        let fonts = Fonts::new();
        let cold = plan_snapshot_with_fonts(&snapshot, frame, &fonts);
        let pixels = paint(&snapshot, frame, &fonts)?;
        let warm_pixels = paint(&snapshot, frame, &fonts)?;
        if warm_pixels != pixels {
            println!(
                "TEXT_CPU_MISMATCH {} cold={} warm={}",
                fixture.name,
                pixel_hex(&pixels)?,
                pixel_hex(&warm_pixels)?
            );
            return Err("worker text cold/warm CPU mismatch".into());
        }
        drop(warm_pixels);
        let warm = plan_snapshot_with_fonts(&snapshot, frame, &fonts);
        let (plan, fallback) = match (cold, warm) {
            (Ok(cold), Ok(warm))
                if fixture.expected_fallback.is_none() && same_plans(&cold, &warm) =>
            {
                (Some(cold), None)
            }
            (Err(cold), Err(warm))
                if cold == warm && fixture.expected_fallback == Some(cold.category()) =>
            {
                (None, Some(cold))
            }
            _ => {
                return Err(format!(
                    "{}: unexpected route or CPU warmth changed admission/plan",
                    fixture.name
                ));
            }
        };
        let case = Prepared {
            name: fixture.name,
            frame,
            pixels,
            plan,
            fallback,
            commands: snapshot.layout.commands.len(),
            image_entries: snapshot.images.len(),
            text,
        };
        if let Some(p) = &case.plan {
            gpu_bytes += p.plan().gpu_buffer_bytes();
            draws += p.plan().draws().len();
            if gpu_bytes > MAX_GPU_BYTES || draws > MAX_DRAWS {
                return Err("worker text retained plan bound".into());
            }
        }
        drop(fonts);
        drop(snapshot);
        // Channel::Drop can block; the outer dedicated subreaper retains the
        // deadline and adopted-child cleanup responsibility before any GPU.
        drop(client);
        check_capture_deadline(deadline)?;
        ensure_no_children()?;
        print_ready(&case)?;
        prepared.push(case);
    }
    if prepared.iter().filter(|p| p.plan.is_some()).count() != GPU_CASES
        || prepared.iter().filter(|p| p.fallback.is_some()).count() != FALLBACK_CASES
    {
        return Err("worker text route totals".into());
    }
    let tasks = ensure_no_children()?;
    println!("TEXT_CAPTURE_COMPLETE cases=7 worker_cases=7 own_tasks={tasks} owned_children=0");
    Ok(prepared)
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).take(8).collect();
    if args.as_slice() == ["--list"] {
        ensure_no_children()?;
        return gpu::run_plans(None, &[], |_, _, _| Ok(()));
    }
    let (adapter,browser,fixtures)=match args.as_slice() {
        [mode,browser_flag,browser,fixtures_flag,fixtures]
            if mode=="--cpu-check" && browser_flag=="--browser" && fixtures_flag=="--fixtures" => (None,browser,fixtures),
        [mode,index,browser_flag,browser,fixtures_flag,fixtures]
            if mode=="--adapter" && browser_flag=="--browser" && fixtures_flag=="--fixtures" =>
                (Some(index.parse::<usize>().map_err(|_|"adapter index")?),browser,fixtures),
        _=>return Err("usage: worker-text-check --list | --cpu-check --browser PATH --fixtures DIR | --adapter N --browser PATH --fixtures DIR".into()),
    };
    let browser = PathBuf::from(browser)
        .canonicalize()
        .map_err(|e| format!("browser path: {e}"))?;
    let fixtures = PathBuf::from(fixtures)
        .canonicalize()
        .map_err(|e| format!("fixture directory: {e}"))?;
    if !browser.is_file() || !fixtures.is_dir() {
        return Err("worker text input paths".into());
    }
    let prepared = capture(&browser, &fixtures)?;
    let reference_bytes: usize = prepared.iter().map(|p| p.pixels.len() * 4).sum();
    let Some(adapter) = adapter else {
        println!(
            "TEXT_CPU_COMPLETE cases=7 gpu_admitted=6 fallback=1 cpu_passes=14 reference_bytes={reference_bytes} gpu_comparisons=0 differential=true"
        );
        return Ok(());
    };
    let mut plans: Vec<(&str, &[u32], &Plan)> = reserve(GPU_CASES)?;
    for case in &prepared {
        if let Some(plan) = &case.plan {
            plans.push((case.name, &case.pixels, plan.plan()));
        }
    }
    ensure_no_children()?;
    std::io::stdout()
        .flush()
        .map_err(|e| format!("text capture flush: {e}"))?;
    read_gpu_grant(&mut std::io::stdin().lock())?;
    let mut checked = 0usize;
    let mut compared_bytes = 0usize;
    gpu::run_plans(Some(adapter), &plans, |name, plan, pixels| {
        if plans.get(checked).map(|p| p.0) != Some(name) {
            return Err("worker text GPU callback order".into());
        }
        let frame = plan.frame();
        compared_bytes += pixels.len() * 4;
        checked += 1;
        println!(
            "TEXT_PASS {name} width={} height={} compared_bytes={} exact=true",
            frame.width,
            frame.height,
            pixels.len() * 4
        );
        Ok(())
    })?;
    if checked != GPU_CASES {
        return Err("missing worker text GPU comparison".into());
    }
    let fallback_bytes = reference_bytes - compared_bytes;
    println!(
        "WORKER_TEXT_COMPLETE adapter={adapter} cases=7 gpu=6 fallback=1 gpu_compared_bytes={compared_bytes} fallback_reference_bytes={fallback_bytes} differential=true custom_wgsl=true"
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
