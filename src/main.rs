#![forbid(unsafe_code)]
mod browser;
mod edit;
mod presenter;
mod worker_benchmark;
use eris::{
    graphics::{Canvas, Color, Fonts},
    page::Page,
};
use std::{path::PathBuf, time::Instant};

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--image-decoder") {
        if eris::worker::serve_image_decoder().is_err() {
            std::process::exit(1);
        }
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("--resource-broker") {
        if eris::worker::serve_resource_broker().is_err() {
            std::process::exit(1);
        }
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("--page-worker") {
        if eris::worker::serve().is_err() {
            std::process::exit(1);
        }
        return;
    }
    if let Err(error) = run() {
        eprintln!("eris-browser: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut address = "eris:home".to_owned();
    let mut headless = false;
    let mut output = PathBuf::from("render.png");
    let mut width = 1180u32;
    let mut height = 880u32;
    let mut scripts = true;
    let mut dump = false;
    let mut iterations = 0usize;
    let mut worker_iterations = None;
    let mut clicks = Vec::new();
    let mut exit_after = None;
    let mut capture = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                println!(
                    "Eris Browser — independent experimental Rust web engine\n\nUsage: eris-browser [ADDRESS] [OPTIONS]\n\n  --render                 Render without a desktop window\n  --output PATH            PNG output (default: render.png)\n  --width N --height N     Viewport in CSS pixels (1180 × 880)\n  --no-scripts             Disable page scripting\n  --click SELECTOR         Activate a matched node before rendering; repeatable\n  --dump-dom               Print the resulting document tree\n  --benchmark N            Measure N style/layout/paint iterations\n  --benchmark-worker N     Measure confined load/render/paint phases; JSON stdout\n  --exit-after SECONDS     Close desktop window after a smoke-test interval\n  --window-screenshot PATH Capture the native browser framebuffer\n\nAddresses: https://example.com, ./examples/forms.html, eris:home, about:blank\nDesktop keys: Ctrl+L address, Ctrl+R reload, Alt+Left/Right history, Ctrl +/- zoom\n\nThis is an early implementation with partial HTML/CSS/JavaScript support.\nFull web compatibility, production security and Chromium performance are unverified."
                );
                return Ok(());
            }
            "--render" | "--headless" => headless = true,
            "--output" | "-o" => {
                output = PathBuf::from(args.next().ok_or("--output needs a path")?)
            }
            "--width" => {
                width = args
                    .next()
                    .ok_or("--width needs a number")?
                    .parse()
                    .map_err(|_| "invalid width")?
            }
            "--height" => {
                height = args
                    .next()
                    .ok_or("--height needs a number")?
                    .parse()
                    .map_err(|_| "invalid height")?
            }
            "--no-scripts" => scripts = false,
            "--dump-dom" => {
                dump = true;
                headless = true;
            }
            "--benchmark" => {
                iterations = args
                    .next()
                    .ok_or("--benchmark needs an iteration count")?
                    .parse()
                    .map_err(|_| "invalid iteration count")?;
                if !(1..=10_000).contains(&iterations) {
                    return Err("benchmark iterations must be between 1 and 10000".into());
                }
                headless = true;
            }
            "--click" => {
                clicks.push(args.next().ok_or("--click needs a selector")?);
                headless = true;
            }
            "--benchmark-worker" => {
                let count = args
                    .next()
                    .ok_or("--benchmark-worker needs an iteration count")?
                    .parse::<usize>()
                    .map_err(|_| "invalid worker iteration count")?;
                if !(1..=10_000).contains(&count) {
                    return Err("worker benchmark iterations must be between 1 and 10000".into());
                }
                worker_iterations = Some(count);
                headless = true;
            }
            "--exit-after" => {
                let seconds: f64 = args
                    .next()
                    .ok_or("--exit-after needs seconds")?
                    .parse()
                    .map_err(|_| "invalid seconds")?;
                if !seconds.is_finite() || seconds <= 0.0 {
                    return Err("--exit-after must be positive and finite".into());
                }
                exit_after = Some(seconds);
            }
            "--window-screenshot" => {
                capture = Some(PathBuf::from(
                    args.next().ok_or("--window-screenshot needs a path")?,
                ));
            }
            flag if flag.starts_with('-') => return Err(format!("unknown option: {flag}")),
            value => address = value.into(),
        }
    }
    if let Some(count) = worker_iterations {
        if iterations > 0 || dump || !clicks.is_empty() || exit_after.is_some() || capture.is_some()
        {
            return Err("--benchmark-worker cannot be combined with --benchmark, --dump-dom, --click, --exit-after or --window-screenshot".into());
        }
        return worker_benchmark::run(&address, scripts, width, height, count, &output);
    }
    if !headless {
        return browser::run(address, scripts, exit_after, capture);
    }
    let mut canvas = Canvas::new(width, height)?;
    let fonts = Fonts::new();
    let start = Instant::now();
    let mut page = Page::load(&address, scripts)?;
    let mut fragment = page.url.fragment().map(str::to_owned);
    for selector in clicks {
        let node = page
            .document
            .query_selector(&selector)
            .ok_or_else(|| format!("click selector matched no node: {selector}"))?;
        if let Some(target) = page.click(node) {
            if target.form_body.is_none()
                && let Ok(url) = url::Url::parse(&target.address)
                && (url.fragment().is_some() || page.url.fragment().is_some())
                && page.navigate_fragment(url.clone())
            {
                fragment = url.fragment().map(str::to_owned);
                continue;
            }
            page = Page::load_navigation(&target, scripts)?;
            fragment = page.url.fragment().map(str::to_owned);
        }
    }
    let layout = page.layout(width as f32, height as f32, &fonts);
    let scroll = fragment
        .and_then(|fragment| eris::page::find_fragment(&page.document, &fragment))
        .and_then(|node| layout.hit_regions.iter().find(|hit| hit.node == node))
        .filter(|hit| !hit.fixed)
        .map(|hit| {
            hit.rect
                .y
                .clamp(0.0, (layout.content_height - height as f32).max(0.0))
        })
        .unwrap_or(0.0);
    canvas.clear(Color::WHITE);
    canvas.paint_with_viewport(
        &layout.commands,
        &fonts,
        &page.images,
        (0.0, -scroll),
        (0.0, 0.0),
    );
    if canvas.exhausted() {
        return Err("page exceeds the raster work budget; render is incomplete".into());
    }
    if let Some(parent) = output.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    canvas.save(&output)?;
    println!(
        "Rendered {} → {} ({}×{}, {} nodes, {} commands, {:.2} ms total)",
        page.url,
        output.display(),
        width,
        height,
        page.document.nodes.len(),
        layout.commands.len(),
        start.elapsed().as_secs_f64() * 1000.0
    );
    for line in &page.runtime.console {
        println!("console: {line}");
    }
    for line in &page.diagnostics {
        eprintln!("[page] {line}");
    }
    if dump {
        let mut stack = vec![(page.document.root, 0)];
        while let Some((id, depth)) = stack.pop() {
            let node = &page.document.nodes[id];
            println!("{}{} {:?}", "  ".repeat(depth.min(32)), id, node.kind);
            for child in node.children.iter().rev() {
                stack.push((*child, depth + 1));
            }
        }
    }
    if iterations > 0 {
        let stylesheets = page.stylesheets();
        let mut samples = Vec::with_capacity(iterations);
        for _ in 0..iterations {
            let start = Instant::now();
            let styles = eris::css::compute_styles_from_sources(
                &page.document,
                &stylesheets,
                width as f32,
                height as f32,
            );
            let layout =
                eris::layout::layout(&page.document, &styles, width as f32, height as f32, &fonts);
            canvas.clear(Color::WHITE);
            canvas.paint_with_viewport(
                &layout.commands,
                &fonts,
                &page.images,
                (0.0, -scroll),
                (0.0, 0.0),
            );
            if canvas.exhausted() {
                return Err("benchmark page exceeds the raster work budget".into());
            }
            std::hint::black_box(&canvas.pixels);
            samples.push(start.elapsed().as_secs_f64() * 1000.0);
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "Benchmark: {iterations} iterations; style+layout+paint median {:.3} ms, p95 {:.3} ms (warm glyph cache; excludes network, parse, scripts, PNG encoding; no Chromium comparison)",
            samples[iterations / 2],
            samples[((iterations as f64 * 0.95).ceil() as usize).saturating_sub(1)]
        );
    }
    Ok(())
}
