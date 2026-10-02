#![forbid(unsafe_code)]
//! Offscreen prerequisites only: this binary never acquires a window surface.
use eris::graphics::{Canvas, Color, Rect as CanvasRect};
use eris_raster_core::{
    Command, Frame, Plan, Profile, Rect, Result, plan_with_masks_for_profile, rect,
    rounded::{RoundedDisposition, RoundedShape},
};
use eris_vulkan_raster_prototype::surface_gpu;
use std::process::ExitCode;

struct Case {
    name: &'static str,
    plan: Plan,
    expected: Vec<u32>,
    reference: &'static str,
}

fn simple(
    name: &'static str,
    width: u32,
    height: u32,
    clear: u32,
    commands: &[Command],
    expected: Vec<u32>,
) -> Result<Case> {
    Ok(Case {
        name,
        plan: plan_with_masks_for_profile(
            Profile::Native,
            Frame::new(width, height, clear),
            commands,
            &[],
            &[],
            &[],
        )?,
        expected,
        reference: "literal-geometry",
    })
}

#[allow(clippy::too_many_arguments)]
fn rounded(
    name: &'static str,
    width: u32,
    height: u32,
    geometry: Rect,
    radius: f32,
    clip: Rect,
    rgba: [u8; 4],
    repeats: usize,
    expected: Vec<u32>,
    reference: &'static str,
) -> Result<Case> {
    let mut frame = Frame::new(width, height, 0xffffff);
    frame.caller_clip = clip;
    let RoundedDisposition::Shape(shape) =
        RoundedShape::prepare(Profile::Native, frame, geometry, clip, radius)?
    else {
        return Err("rounded fixture did not produce nonempty positive-radius geometry".into());
    };
    let coverage = shape.materialize(|_| Ok(()))?;
    let commands = vec![
        Command::Glyph {
            source: 0,
            rows: 0,
            y: coverage.origin_y(),
            rgba
        };
        repeats
    ];
    let plan = plan_with_masks_for_profile(
        Profile::Native,
        frame,
        &commands,
        &[],
        &[coverage.mask()],
        &[coverage.row_origins()],
    )?;
    Ok(Case {
        name,
        plan,
        expected,
        reference,
    })
}

fn cases() -> Result<Vec<Case>> {
    let full = Rect::new(0.0, 0.0, 2.0, 2.0);
    // These tiny RGB values are written from the coverage/alpha equations,
    // before running either the new preparation code or the GPU shaders.
    // sqrt(0.5) gives coverage202; alpha128 gives effective alpha101.
    let mut cases = vec![
        rounded(
            "round-corner",
            2,
            2,
            full,
            1.0,
            full,
            [0, 0, 0, 255],
            1,
            vec![0x353535; 4],
            "literal-coverage",
        )?,
        rounded(
            "round-alpha-twice",
            2,
            2,
            full,
            1.0,
            full,
            [0, 0, 0, 128],
            2,
            vec![0x5d5d5d; 4],
            "literal-coverage",
        )?,
        rounded(
            "round-clamped",
            2,
            2,
            full,
            100.0,
            full,
            [0, 0, 0, 255],
            1,
            vec![0x353535; 4],
            "literal-coverage",
        )?,
        rounded(
            "round-quarter",
            1,
            1,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            0.25,
            Rect::new(0.0, 0.0, 1.0, 1.0),
            [0, 0, 0, 255],
            1,
            vec![0x404040],
            "literal-coverage",
        )?,
        rounded(
            "round-fractional",
            2,
            1,
            Rect::new(0.25, 0.0, 1.0, 1.0),
            0.25,
            Rect::new(0.0, 0.0, 2.0, 1.0),
            [0, 0, 0, 255],
            1,
            vec![0x404040, 0xc0c0c0],
            "literal-coverage",
        )?,
        rounded(
            "round-clipped",
            2,
            1,
            Rect::new(0.25, 0.0, 1.0, 1.0),
            0.25,
            Rect::new(1.0, 0.0, 1.0, 1.0),
            [0, 0, 0, 255],
            1,
            vec![0xffffff, 0xc0c0c0],
            "literal-coverage",
        )?,
    ];
    let colors = [0x123456, 0xff0000, 0x00ff00, 0x0000ff, 0xabcdef, 0x010203];
    let commands = colors
        .iter()
        .enumerate()
        .map(|(i, &color)| rect((i % 3) as f32, (i / 3) as f32, 1.0, 1.0, color))
        .collect::<Vec<_>>();
    cases.push(simple(
        "asymmetric-colors",
        3,
        2,
        0,
        &commands,
        colors.to_vec(),
    )?);
    let mut desktop = vec![0x123456; 1180 * 880];
    for y in 5..14 {
        desktop[y * 1180 + 7..y * 1180 + 26].fill(0xabcdef);
    }
    cases.push(simple(
        "desktop-padded",
        1180,
        880,
        0x123456,
        &[rect(7.0, 5.0, 19.0, 9.0, 0xabcdef)],
        desktop,
    )?);
    cases.push(simple(
        "native-maximum",
        1280,
        1024,
        0x804020,
        &[],
        vec![0x804020; 1280 * 1024],
    )?);
    let mut odd = vec![0xffffff; 1279 * 3];
    odd[1279 * 3 - 1] = 0x0000ff;
    cases.push(simple(
        "odd-width-last-pixel",
        1279,
        3,
        0xffffff,
        &[rect(1278.0, 2.0, 1.0, 1.0, 0x0000ff)],
        odd,
    )?);
    // Separate differential case: the original painter supplies only the
    // comparison reference. The GPU receives geometry coverage and color.
    let bar = Rect::new(181.0, 31.0, 984.0, 34.0);
    let mut canvas = Canvas::new(1180, 880)?;
    canvas.clear(Color::WHITE);
    canvas.rect(
        CanvasRect {
            x: bar.x,
            y: bar.y,
            width: bar.width,
            height: bar.height,
        },
        Color::rgb(38, 45, 61),
        7.0,
    );
    cases.push(rounded(
        "desktop-address-bar",
        1180,
        880,
        bar,
        7.0,
        Rect::new(0.0, 0.0, 1180.0, 880.0),
        [38, 45, 61, 255],
        1,
        canvas.pixels,
        "Canvas-differential",
    )?);
    Ok(cases)
}

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-surface-check --list | --adapter INDEX".into()),
    };
    let cases = cases()?;
    let borrowed = cases
        .iter()
        .map(|case| (case.name, case.expected.as_slice(), &case.plan))
        .collect::<Vec<_>>();
    let mut compared = 0u64;
    surface_gpu::run(selected, &borrowed, |name, plan, format, bytes| {
        let reference = cases
            .iter()
            .find(|case| case.name == name)
            .ok_or("case identity")?
            .reference;
        println!(
            "PASS {name} format={format:?} width={} height={} draws={} raster_invocations={} conversion_invocations={} planned_bytes={} compared_bytes={bytes} reference={reference} exact=true",
            plan.frame().width,
            plan.frame().height,
            plan.draws().len(),
            plan.raster_invocations(),
            plan.conversion_invocations(),
            plan.gpu_buffer_bytes()
        );
        compared = compared
            .checked_add(bytes)
            .ok_or("comparison total overflow")?;
        Ok(())
    })?;
    if let Some(index) = selected {
        println!(
            "COMPLETE adapter={index} cases={} formats=2 compared_bytes={compared} offscreen=true acquired_surface=false exact=true",
            cases.len()
        );
    }
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
