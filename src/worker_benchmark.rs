//! Repeatable measurements of the real confined worker and validated snapshot path.
//! This does not open a window or measure native presentation.
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::{Navigation, find_fragment},
    worker::{Command, Snapshot, WorkerClient},
};
use std::{fmt::Write, path::Path, time::Instant};

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

fn render(client: &mut WorkerClient, width: u32, height: u32) -> Result<Snapshot, String> {
    let reply = client.exchange(
        Command::Render {
            width: width as f32,
            height: height as f32,
        },
        || false,
    )?;
    if reply.navigation.is_some() {
        return Err("unexpected navigation during worker benchmark".into());
    }
    let snapshot = reply.snapshot.ok_or("missing benchmark snapshot")?;
    if url::Url::parse(&snapshot.url)
        .is_ok_and(|url| url.scheme() == "eris" && url.path() == "error")
    {
        return Err("worker benchmark load failed; renderer returned its error page".into());
    }
    Ok(snapshot)
}

fn scroll_offset(snapshot: &Snapshot, height: u32) -> f32 {
    url::Url::parse(&snapshot.url)
        .ok()
        .and_then(|url| find_fragment(&snapshot.document, url.fragment()?))
        .and_then(|node| {
            snapshot
                .layout
                .hit_regions
                .iter()
                .find(|hit| hit.node == node)
        })
        .filter(|hit| !hit.fixed)
        .map(|hit| {
            hit.rect.y.clamp(
                0.0,
                (snapshot.layout.content_height - height as f32).max(0.0),
            )
        })
        .unwrap_or(0.0)
}

fn paint(
    canvas: &mut Canvas,
    fonts: &Fonts,
    snapshot: &Snapshot,
    scroll: f32,
) -> Result<(), String> {
    canvas.clear(Color::WHITE);
    canvas.paint_with_viewport(
        &snapshot.layout.commands,
        fonts,
        &snapshot.images,
        (0.0, -scroll),
        (0.0, 0.0),
    );
    if canvas.exhausted() {
        return Err("worker benchmark page exceeds the raster work budget".into());
    }
    std::hint::black_box(&canvas.pixels);
    Ok(())
}

pub fn run(
    address: &str,
    scripts: bool,
    width: u32,
    height: u32,
    iterations: usize,
    output: &Path,
) -> Result<(), String> {
    // Validate dimensions before launching a process. Font/canvas setup is outside
    // the measured phases, and each CLI invocation starts with a fresh glyph cache.
    let mut canvas = Canvas::new(width, height)?;
    let fonts = Fonts::new();
    let navigation = Navigation::get(address);
    let start = Instant::now();
    let mut client = WorkerClient::spawn(scripts, &navigation, 1)?;
    let startup = elapsed_ms(start);
    let start = Instant::now();
    let reply = client.exchange(Command::Load { navigation }, || false)?;
    let load = elapsed_ms(start);
    if reply.snapshot.is_some() || reply.navigation.is_some() {
        return Err("unexpected worker benchmark load reply".into());
    }
    let start = Instant::now();
    let initial = render(&mut client, width, height)?;
    let cold_exchange = elapsed_ms(start);
    let nodes = initial.document.nodes.len();
    let commands = initial.layout.commands.len();
    for line in &initial.diagnostics {
        eprintln!("[page] {line}");
    }
    let scroll = scroll_offset(&initial, height);
    let start = Instant::now();
    paint(&mut canvas, &fonts, &initial, scroll)?;
    let cold_paint = elapsed_ms(start);
    drop(initial);

    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let snapshot = render(&mut client, width, height)?;
        let exchange = elapsed_ms(start);
        let scroll = scroll_offset(&snapshot, height);
        let paint_start = Instant::now();
        paint(&mut canvas, &fonts, &snapshot, scroll)?;
        let paint = elapsed_ms(paint_start);
        // The total additionally covers fragment lookup and snapshot destruction.
        drop(snapshot);
        samples.push((exchange, paint, elapsed_ms(start)));
    }
    let start = Instant::now();
    drop(client);
    let teardown = elapsed_ms(start);
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    canvas.save(output)?;

    // Only fixed keys, booleans and finite measured numbers enter this JSON. Page
    // text stays on stderr; no escaping of an untrusted URL/title is needed here.
    let mut report = format!(
        "{{\"schema\":1,\"viewport\":[{width},{height}],\"scripts\":{scripts},\"iterations\":{iterations},\"nodes\":{nodes},\"commands\":{commands},\"startup_ms\":{startup:.6},\"load_ms\":{load:.6},\"cold_render_exchange_ms\":{cold_exchange:.6},\"cold_clear_paint_ms\":{cold_paint:.6},\"teardown_ms\":{teardown:.6},\"warm_samples\":["
    );
    for (index, (exchange, paint, total)) in samples.into_iter().enumerate() {
        if index > 0 {
            report.push(',');
        }
        write!(report, "{{\"render_exchange_ms\":{exchange:.6},\"clear_paint_ms\":{paint:.6},\"total_ms\":{total:.6}}}").unwrap();
    }
    report.push_str("]}");
    println!("{report}");
    Ok(())
}
