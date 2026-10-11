#![forbid(unsafe_code)]
use eris_vulkan_raster_prototype::{Result, background_fixtures, surface_gpu};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-background-check --list | --adapter INDEX".into()),
    };
    // Full Canvas/literal/packed-plan agreement precedes any Vulkan call.
    // A native refusal is an error; the checker never substitutes CPU output.
    let fixtures = background_fixtures::fixtures()?;
    let plans = fixtures
        .iter()
        .map(|f| (f.name, f.expected.as_slice(), &f.plan))
        .collect::<Vec<_>>();
    let mut frames = 0;
    let mut compared = 0;
    surface_gpu::run(selected, &plans, |name, plan, format, bytes| {
        frames += 1;
        compared += bytes;
        println!(
            "PASS {name} format={format:?} draws={} compared_bytes={bytes} exact=true reference=literal canvas=true bounded_background_lut=true",
            plan.draws().len()
        );
        Ok(())
    })?;
    if let Some(adapter) = selected {
        use background_fixtures::witness;
        if fixtures.len() != witness::FIXTURE_NAMES.len()
            || frames != witness::CONVERTED_FRAMES
            || compared != witness::COMPARED_BYTES
        {
            return Err("background completion count".into());
        }
        println!(
            "COMPLETE adapter={adapter} fixtures=6 frames={frames} compared_bytes={compared} exact=true bounded_background_lut=true custom_image_shader=true offscreen=true acquired_surface=false"
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
