#![forbid(unsafe_code)]

use eris::graphics::{Canvas, Color, Fonts};
use eris_vulkan_raster_prototype::{
    Command, Frame, MAX_GPU_BUFFER_BYTES, Plan, Rect, Result, SourceImage, SourceMask,
    browser_adapter::{FontBridgePlan, plan_display_list_with_fonts},
    gpu,
};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::ExitCode,
};

#[path = "../glyph_fixtures/font_inventory.rs"]
mod font_inventory;
#[path = "../glyph_fixtures/literal_inventory.rs"]
mod literal_inventory;

const LITERAL_CASES: usize = 12;
const FONT_CASES: usize = 14;
const MAX_CASES: usize = LITERAL_CASES + FONT_CASES;
const MAX_PIXELS: usize = 320 * 240;

enum PreparedPlan {
    Literal(Plan),
    Font(FontBridgePlan),
}
impl PreparedPlan {
    fn plan(&self) -> &Plan {
        match self {
            Self::Literal(plan) => plan,
            Self::Font(plan) => plan.plan(),
        }
    }
}
struct Prepared {
    name: &'static str,
    population: &'static str,
    plan: PreparedPlan,
    expected: Vec<u32>,
}

fn reserved<T>(count: usize) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| "checker allocation")?;
    Ok(result)
}

fn parent_pixels(directory: &Path, name: &str, pixels: usize) -> Result<Vec<u32>> {
    if pixels == 0
        || pixels > MAX_PIXELS
        || name.len() > 128
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("parent baseline input bound".into());
    }
    let bytes = pixels * 4;
    let file = File::open(directory.join(format!("{name}.rgb")))
        .map_err(|e| format!("{name}: parent baseline open: {e}"))?;
    if file.metadata().map_err(|e| e.to_string())?.len() != bytes as u64 {
        return Err(format!("{name}: parent baseline length"));
    }
    let mut data = reserved(bytes + 1)?;
    file.take(bytes as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() != bytes {
        return Err(format!("{name}: changed baseline length"));
    }
    let mut result = reserved(pixels)?;
    for chunk in data.as_chunks::<4>().0 {
        let pixel = u32::from_le_bytes(*chunk);
        if pixel > 0xffffff {
            return Err(format!("{name}: baseline high byte"));
        }
        result.push(pixel);
    }
    Ok(result)
}

fn font_frame(frame: &font_inventory::FontFrame) -> Frame {
    Frame {
        width: frame.width,
        height: frame.height,
        clear: frame.clear,
        caller_clip: eris_vulkan_raster_prototype::Rect::new(
            frame.caller_clip.x,
            frame.caller_clip.y,
            frame.caller_clip.width,
            frame.caller_clip.height,
        ),
        document_offset: frame.document_offset,
        viewport_offset: frame.viewport_offset,
    }
}

fn check_cpu(
    fixture: &font_inventory::FontFixture,
    fonts: &Fonts,
    expected: &[u32],
    pass: &str,
) -> Result<()> {
    let f = &fixture.frame;
    let mut canvas = Canvas::new(f.width, f.height)?;
    canvas.clear(Color::rgb(
        (f.clear >> 16) as u8,
        (f.clear >> 8) as u8,
        f.clear as u8,
    ));
    canvas.set_clip(f.caller_clip);
    canvas.paint_with_viewport(
        &fixture.commands,
        fonts,
        &fixture.images,
        f.document_offset,
        f.viewport_offset,
    );
    if canvas.exhausted() {
        return Err(format!("{}: {pass} CPU paint exhausted", fixture.name));
    }
    if canvas.pixels != expected {
        // A failed comparison retains the complete bounded current CPU target.
        // This diagnostic is not accepted by the success protocol parser.
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        write!(
            out,
            "CPU_MISMATCH {} pass={pass} rgba_le_hex=",
            fixture.name
        )
        .map_err(|e| e.to_string())?;
        for pixel in canvas.pixels {
            for byte in pixel.to_le_bytes() {
                write!(out, "{byte:02x}").map_err(|e| e.to_string())?;
            }
        }
        writeln!(out).map_err(|e| e.to_string())?;
        return Err(format!(
            "{}: {pass} differs from preserved parent CPU",
            fixture.name
        ));
    }
    Ok(())
}

fn prepare(directory: &Path) -> Result<Vec<Prepared>> {
    // These are fixed compiled fixture constructors, never author-controlled
    // deserialization. No worker process is launched by this checker.
    let literals = literal_inventory::fixtures();
    let fonts = font_inventory::fixtures();
    if literals.len() != LITERAL_CASES || fonts.len() != FONT_CASES {
        return Err("glyph fixture inventory".into());
    }
    let mut prepared = reserved(MAX_CASES)?;
    for fixture in literals {
        let mut images = reserved(fixture.images.len())?;
        for image in &fixture.images {
            images.push(SourceImage {
                width: image.width,
                height: image.height,
                rgba: &image.rgba,
            });
        }
        let mut masks = reserved(fixture.masks.len())?;
        for mask in &fixture.masks {
            masks.push(SourceMask {
                width: mask.width,
                height: mask.height,
                coverage: &mask.coverage,
            });
        }
        let mut rows = reserved(fixture.rows.len())?;
        for row in &fixture.rows {
            rows.push(row.as_slice());
        }
        let plan = eris_vulkan_raster_prototype::plan_with_masks(
            fixture.frame,
            &fixture.commands,
            &images,
            &masks,
            &rows,
        )?;
        if fixture.expected.len() != fixture.frame.width as usize * fixture.frame.height as usize {
            return Err(format!("{}: literal pixel count", fixture.name));
        }
        prepared.push(Prepared {
            name: fixture.name,
            population: "literal-mask",
            plan: PreparedPlan::Literal(plan),
            expected: fixture.expected,
        });
    }
    for fixture in fonts {
        let frame = font_frame(&fixture.frame);
        let expected = parent_pixels(
            directory,
            fixture.name,
            frame.width as usize * frame.height as usize,
        )?;
        let fonts = Fonts::new();
        let cold = plan_display_list_with_fonts(&fixture.commands, &fixture.images, frame, &fonts)
            .map_err(|e| format!("{}: cold admission: {e}", fixture.name))?;
        check_cpu(&fixture, &fonts, &expected, "cold")?;
        check_cpu(&fixture, &fonts, &expected, "warm")?;
        let warm = plan_display_list_with_fonts(&fixture.commands, &fixture.images, frame, &fonts)
            .map_err(|e| format!("{}: warm admission: {e}", fixture.name))?;
        if cold.stats() != warm.stats()
            || cold.text_stats() != warm.text_stats()
            || cold.plan().draws() != warm.plan().draws()
            || cold.plan().invocations() != warm.plan().invocations()
            || cold.plan().gpu_buffer_bytes() != warm.plan().gpu_buffer_bytes()
            || cold.plan().parameters() != warm.plan().parameters()
            || cold.plan().input_bytes() != warm.plan().input_bytes()
        {
            return Err(format!(
                "{}: CPU cache warmth changed admission or plan",
                fixture.name
            ));
        }
        let s = cold.text_stats();
        let p = s.preparation;
        println!(
            "FONT_META {} bytes={} scalars={} visited={} occurrences={} masks={} coverage={} cold_work={} scratch={} rows={} cpu_upper={} gpu_upper={} cold_warm=true",
            fixture.name,
            s.input_bytes,
            s.input_scalars,
            p.visited_scalars,
            p.occurrences,
            p.cold_requests,
            p.unique_coverage_bytes,
            p.cold_mask_work,
            p.max_raster_scratch_bytes,
            s.row_entries,
            s.cpu_pixel_upper_bound,
            s.gpu_buffer_upper_bound
        );
        prepared.push(Prepared {
            name: fixture.name,
            population: "font-reference",
            plan: PreparedPlan::Font(cold),
            expected,
        });
        // Fonts, original inputs and the warm comparison plan drop each iteration.
    }
    let mut gpu_bytes = 0u64;
    let mut reference_bytes = 0usize;
    for case in &prepared {
        let p = case.plan.plan();
        gpu_bytes = gpu_bytes
            .checked_add(p.gpu_buffer_bytes())
            .ok_or("retained GPU bytes overflow")?;
        reference_bytes = reference_bytes
            .checked_add(case.expected.len() * 4)
            .ok_or("retained reference overflow")?;
        if gpu_bytes > MAX_CASES as u64 * MAX_GPU_BUFFER_BYTES
            || reference_bytes > MAX_CASES * MAX_PIXELS * 4
        {
            return Err("checker aggregate plan/reference bound".into());
        }
        let f = p.frame();
        println!(
            "GLYPH_READY {} population={} width={} height={} compared_bytes={} draws={} invocations={} gpu_buffers={}",
            case.name,
            case.population,
            f.width,
            f.height,
            case.expected.len() * 4,
            p.draws().len(),
            p.invocations(),
            p.gpu_buffer_bytes()
        );
    }
    println!(
        "GLYPH_PREPARED cases={MAX_CASES} literal={LITERAL_CASES} font={FONT_CASES} retained_gpu_bytes={gpu_bytes} retained_reference_bytes={reference_bytes} worker_cases=0"
    );
    Ok(prepared)
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).take(6).collect();
    if args.as_slice() == ["--list"] {
        return gpu::run_plans(None, &[], |_, _, _| Ok(()));
    }
    let (adapter, directory) = match args.as_slice() {
        [mode, flag, directory] if mode == "--cpu-check" && flag == "--baselines" => {
            (None, directory)
        }
        [mode, index, flag, directory] if mode == "--adapter" && flag == "--baselines" => (
            Some(index.parse::<usize>().map_err(|_| "adapter index")?),
            directory,
        ),
        _ => return Err(
            "usage: glyph-check --list | --cpu-check --baselines DIR | --adapter N --baselines DIR"
                .into(),
        ),
    };
    let directory = Path::new(directory);
    if !directory.is_dir() {
        return Err("parent baseline directory".into());
    }
    let prepared = prepare(directory)?;
    let Some(adapter) = adapter else {
        println!(
            "GLYPH_CPU_COMPLETE cases={MAX_CASES} literal={LITERAL_CASES} font={FONT_CASES} cpu_passes={} exact=true",
            FONT_CASES * 2
        );
        return Ok(());
    };
    let mut plans = reserved(MAX_CASES)?;
    for case in &prepared {
        plans.push((case.name, case.expected.as_slice(), case.plan.plan()));
    }
    let mut checked = 0;
    gpu::run_plans(Some(adapter), &plans, |name, plan, expected| {
        let case = prepared.get(checked).ok_or("extra GPU glyph callback")?;
        if case.name != name {
            return Err("GPU glyph callback order".into());
        }
        let f = plan.frame();
        println!(
            "GLYPH_PASS {name} population={} width={} height={} compared_bytes={} exact=true",
            case.population,
            f.width,
            f.height,
            expected.len() * 4
        );
        checked += 1;
        Ok(())
    })?;
    if checked != MAX_CASES {
        return Err("missing GPU glyph comparisons".into());
    }
    println!(
        "GLYPH_COMPLETE adapter={adapter} cases={MAX_CASES} literal={LITERAL_CASES} font={FONT_CASES} exact=true custom_wgsl=true"
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
