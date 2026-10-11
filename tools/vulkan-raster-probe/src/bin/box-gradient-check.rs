#![forbid(unsafe_code)]
use eris_vulkan_raster_prototype::{Result, box_gradient_fixtures, surface_gpu};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-box-gradient-check --list | --adapter INDEX".into()),
    };
    // Literal pixels only reach the host comparator; GPU encoding uses the plan.
    // Native refusal and missing/out-of-range Vulkan adapters are fatal.
    let fixtures = box_gradient_fixtures::fixtures()?;
    let plans = fixtures
        .iter()
        .map(|fixture| (fixture.name, fixture.expected.as_slice(), &fixture.plan))
        .collect::<Vec<_>>();
    let mut frames = 0;
    let mut compared = 0;
    surface_gpu::run(selected, &plans, |name, plan, format, bytes| {
        frames += 1;
        compared += bytes;
        println!(
            "PASS {name} format={format:?} draws={} compared_bytes={bytes} exact=true reference=literal canvas=true packed_native=true box_gradients=true zero_masks=true",
            plan.draws().len()
        );
        Ok(())
    })?;
    if let Some(adapter) = selected {
        use box_gradient_fixtures::{COMPARED_BYTES, CONVERTED_FRAMES, FIXTURE_NAMES};
        if fixtures.len() != FIXTURE_NAMES.len()
            || frames != CONVERTED_FRAMES
            || compared != COMPARED_BYTES
        {
            return Err("Box gradient completion count".into());
        }
        println!(
            "COMPLETE adapter={adapter} fixtures=6 frames={frames} compared_bytes={compared} exact=true box_gradients=true custom_image_shader=true offscreen=true acquired_surface=false"
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
