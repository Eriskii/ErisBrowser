#![forbid(unsafe_code)]

use eris::{
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore, RasterImage, Rect},
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
    fs,
    io::{Read, Write as _},
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    process::ExitCode,
    sync::Arc,
    time::{Duration, Instant},
};

const MAX_CASES: usize = 16;
const MAX_RETAINED_GPU_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RETAINED_CPU_BYTES: usize = 16 * 320 * 240 * 4;
const MAX_RETAINED_DRAWS: usize = 16 * 257;
const MAX_SNAPSHOT_DUMP: usize = 65_536;
const MAX_OWN_TASKS: usize = 64;
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(180);

fn reserve<T>(count: usize) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| "bounded checker allocation")?;
    Ok(result)
}

fn check_capture_deadline(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        Err("capture deadline".into())
    } else {
        Ok(())
    }
}

// This is a refusal check, not a reaper. The outer subreaper owns cleanup.
// Every task is checked because Linux children lists are per-thread.
fn ensure_no_children() -> Result<usize> {
    let mut tasks = 0usize;
    for entry in fs::read_dir("/proc/self/task").map_err(|e| format!("own tasks: {e}"))? {
        tasks += 1;
        if tasks > MAX_OWN_TASKS {
            return Err("own task inventory limit".into());
        }
        let entry = entry.map_err(|e| format!("own task entry: {e}"))?;
        let mut file = fs::File::open(entry.path().join("children"))
            .map_err(|e| format!("own task children: {e}"))?;
        let mut bytes = [0u8; 4097];
        let mut length = 0;
        loop {
            let count = file
                .read(&mut bytes[length..])
                .map_err(|e| format!("own child inventory: {e}"))?;
            if count == 0 {
                break;
            }
            length += count;
            if length > 4096 {
                return Err("own child inventory exceeds bound".into());
            }
        }
        if bytes[..length].iter().any(|b| !b.is_ascii_whitespace()) {
            return Err("capture has unresolved children".into());
        }
    }
    if tasks == 0 {
        return Err("missing own task inventory".into());
    }
    Ok(tasks)
}

fn file_address(path: &Path) -> Result<String> {
    let bytes = path.as_os_str().as_bytes();
    if !path.is_absolute() || bytes.len() > 4096 {
        return Err("fixture path bound".into());
    }
    let mut address = String::new();
    address
        .try_reserve_exact(7 + bytes.len() * 3)
        .map_err(|_| "fixture address allocation")?;
    address.push_str("file://");
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
            address.push(char::from(byte));
        } else {
            write!(address, "%{byte:02X}").map_err(|_| "fixture address formatting")?;
        }
    }
    Ok(address)
}

fn fixture_path(directory: &Path, name: &str) -> Result<PathBuf> {
    let relative = Path::new(name);
    if name.len() > 4096
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("fixture relative path".into());
    }
    let path = directory
        .join(relative)
        .canonicalize()
        .map_err(|e| format!("fixture path: {e}"))?;
    if !path.starts_with(directory) || !path.is_file() {
        return Err("fixture outside supplied directory".into());
    }
    Ok(path)
}

// Canonical capture format EWB1, all integers little-endian; strings are u32
// byte length + exact UTF-8. No field is normalized or removed.
// Header: magic, frame width/height u32, generation/edit sequence u64, task u8,
// content-height f32 bits, load-ms f64 bits, title string, URL string.
// Commands: u32 count, then tags 0 clip(rect), 1 pop-clip, 2 fixed, 3 pop-fixed,
// 4 opacity(f32), 5 pop-opacity, 6 rect(rect,RGBA,radius),
// 7 text(x,y,size,RGBA,bold,italic,mono,text), 8 image(rect,key),
// 9 line(x1,y1,x2,y2,width,RGBA). Rect is x/y/width/height f32 bits.
// Images: u32 key count, sorted (key string, u32 source ID); then u32 source
// count and each (width,height,RGBA byte count,exact bytes). Arc identity gets
// first sorted-key IDs, never pointer addresses. Finally u32 diagnostic count
// followed by each exact string. Snapshot DOM/hit-regions are not serialized.
struct Dump {
    bytes: Option<Vec<u8>>,
    len: usize,
}
impl Dump {
    fn raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .ok_or("capture size overflow")?;
        if self.len > MAX_SNAPSHOT_DUMP {
            return Err("worker capture record too large".into());
        }
        if let Some(out) = &mut self.bytes {
            if self.len > out.capacity() {
                return Err("capture reservation mismatch".into());
            }
            out.extend_from_slice(bytes);
        }
        Ok(())
    }
    fn count(&mut self, n: usize) -> Result<()> {
        self.raw(
            &u32::try_from(n)
                .map_err(|_| "capture count overflow")?
                .to_le_bytes(),
        )
    }
    fn string(&mut self, s: &str) -> Result<()> {
        self.count(s.len())?;
        self.raw(s.as_bytes())
    }
    fn scalar(&mut self, v: f32) -> Result<()> {
        self.raw(&v.to_bits().to_le_bytes())
    }
    fn rect(&mut self, rect: Rect) -> Result<()> {
        for v in [rect.x, rect.y, rect.width, rect.height] {
            self.scalar(v)?;
        }
        Ok(())
    }
    fn color(&mut self, color: Color) -> Result<()> {
        self.raw(&[color.r, color.g, color.b, color.a])
    }
}

fn dump_snapshot(snapshot: &Snapshot, frame: Frame) -> Result<Vec<u8>> {
    if snapshot.layout.commands.len() > 256
        || snapshot.images.len() > 256
        || snapshot.images.capacity() > 512
        || snapshot.diagnostics.len() > 256
    {
        return Err("worker capture inventory exceeds fixture contract".into());
    }
    let mut key_bytes = 0usize;
    for key in snapshot.images.keys() {
        key_bytes = key_bytes
            .checked_add(key.len())
            .ok_or("capture key overflow")?;
        if key.len() > 4096 || key_bytes > 65_536 {
            return Err("worker capture key bound".into());
        }
    }
    let mut entries = reserve(snapshot.images.len())?;
    entries.extend(snapshot.images.iter());
    // At most 256 entries and 64 KiB of key bytes, bounded before sorting.
    for i in 1..entries.len() {
        let mut j = i;
        while j > 0 && entries[j].0 < entries[j - 1].0 {
            entries.swap(j, j - 1);
            j -= 1;
        }
    }
    let mut sources: Vec<&Arc<RasterImage>> = reserve(entries.len())?;
    let mut ids = reserve(entries.len())?;
    for (_, image) in &entries {
        let id = if let Some(id) = sources.iter().position(|old| Arc::ptr_eq(old, image)) {
            id
        } else {
            sources.push(image);
            sources.len() - 1
        };
        ids.push(id);
    }
    let serialize = |out: &mut Dump| -> Result<()> {
        out.raw(b"EWB1")?;
        out.raw(&frame.width.to_le_bytes())?;
        out.raw(&frame.height.to_le_bytes())?;
        out.raw(&snapshot.generation.to_le_bytes())?;
        out.raw(&snapshot.processed_edit_sequence.to_le_bytes())?;
        out.raw(&[match snapshot.task_state {
            TaskState::Idle => 0,
            TaskState::Pending => 1,
            TaskState::Suspended => 2,
        }])?;
        out.scalar(snapshot.layout.content_height)?;
        out.raw(&snapshot.load_ms.to_bits().to_le_bytes())?;
        out.string(&snapshot.title)?;
        out.string(&snapshot.url)?;
        out.count(snapshot.layout.commands.len())?;
        for command in &snapshot.layout.commands {
            match command {
                DrawCommand::PushClip { rect } => {
                    out.raw(&[0])?;
                    out.rect(*rect)?;
                }
                DrawCommand::PopClip => out.raw(&[1])?,
                DrawCommand::PushFixed => out.raw(&[2])?,
                DrawCommand::PopFixed => out.raw(&[3])?,
                DrawCommand::PushOpacity { opacity } => {
                    out.raw(&[4])?;
                    out.scalar(*opacity)?;
                }
                DrawCommand::PopOpacity => out.raw(&[5])?,
                DrawCommand::Rect {
                    rect,
                    color,
                    radius,
                } => {
                    out.raw(&[6])?;
                    out.rect(*rect)?;
                    out.color(*color)?;
                    out.scalar(*radius)?;
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
                    out.raw(&[7])?;
                    for v in [*x, *y, *size] {
                        out.scalar(v)?;
                    }
                    out.color(*color)?;
                    out.raw(&[u8::from(*bold), u8::from(*italic), u8::from(*monospace)])?;
                    out.string(text)?;
                }
                DrawCommand::Image { rect, key } => {
                    out.raw(&[8])?;
                    out.rect(*rect)?;
                    out.string(key)?;
                }
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                    width,
                } => {
                    out.raw(&[9])?;
                    for v in [*x1, *y1, *x2, *y2, *width] {
                        out.scalar(v)?;
                    }
                    out.color(*color)?;
                }
            }
        }
        out.count(entries.len())?;
        for ((key, _), id) in entries.iter().zip(&ids) {
            out.string(key)?;
            out.count(*id)?;
        }
        out.count(sources.len())?;
        for image in &sources {
            out.raw(&image.width.to_le_bytes())?;
            out.raw(&image.height.to_le_bytes())?;
            out.count(image.rgba.len())?;
            out.raw(&image.rgba)?;
        }
        out.count(snapshot.diagnostics.len())?;
        for diagnostic in &snapshot.diagnostics {
            out.string(diagnostic)?;
        }
        Ok(())
    };
    let mut measure = Dump {
        bytes: None,
        len: 0,
    };
    serialize(&mut measure)?;
    let mut output = Dump {
        bytes: Some(reserve(measure.len)?),
        len: 0,
    };
    serialize(&mut output)?;
    if output.len != measure.len {
        return Err("capture length changed".into());
    }
    output.bytes.ok_or_else(|| "missing capture bytes".into())
}

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

fn print_snapshot(name: &str, bytes: &[u8]) -> Result<()> {
    let mut hex = String::new();
    hex.try_reserve_exact(bytes.len().checked_mul(2).ok_or("capture hex overflow")?)
        .map_err(|_| "capture hex allocation")?;
    for byte in bytes {
        write!(hex, "{byte:02x}").map_err(|_| "capture hex formatting")?;
    }
    println!("SNAPSHOT {name} bytes={} hex={hex}", bytes.len());
    Ok(())
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

fn read_gpu_grant(input: &mut impl Read) -> Result<()> {
    const EXPECTED: &[u8] = b"GPU_READY\n";
    let mut token = [0u8; EXPECTED.len()];
    input
        .read_exact(&mut token)
        .map_err(|e| format!("GPU capture grant: {e}"))?;
    if token != EXPECTED {
        return Err("invalid GPU capture grant".into());
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_grant_requires_exact_token_and_complete_input() {
        assert!(read_gpu_grant(&mut &b"GPU_READY\n"[..]).is_ok());
        assert!(read_gpu_grant(&mut &b"GPU_READY"[..]).is_err());
        assert!(read_gpu_grant(&mut &b"BAD_READY\n"[..]).is_err());
        assert!(read_gpu_grant(&mut &b""[..]).is_err());
    }

    #[test]
    fn fixture_file_address_encodes_reserved_and_utf8_bytes() {
        assert_eq!(
            file_address(Path::new("/tmp/a b#c%/é.html")).unwrap(),
            "file:///tmp/a%20b%23c%25/%C3%A9.html"
        );
        assert!(file_address(Path::new("relative.html")).is_err());
    }

    #[test]
    fn capture_measure_rejects_oversized_payload_before_output_allocation() {
        let source = [0u8; MAX_SNAPSHOT_DUMP + 1];
        let mut measure = Dump {
            bytes: None,
            len: 0,
        };
        assert!(measure.raw(&source).is_err());
        assert!(measure.bytes.is_none());
    }

    #[test]
    fn canonical_record_keeps_utf8_and_negative_zero_bits() {
        let serialize = |out: &mut Dump| -> Result<()> {
            out.string("\0é")?;
            out.scalar(-0.0)
        };
        let mut measure = Dump {
            bytes: None,
            len: 0,
        };
        serialize(&mut measure).unwrap();
        assert_eq!(measure.len, 11);
        let mut out = Dump {
            bytes: Some(reserve(measure.len).unwrap()),
            len: 0,
        };
        serialize(&mut out).unwrap();
        assert_eq!(
            out.bytes.unwrap(),
            [3, 0, 0, 0, 0, 0xc3, 0xa9, 0, 0, 0, 0x80]
        );
    }
}
