#![forbid(unsafe_code)]
use eris_vulkan_raster_prototype::{Result, gradient_fixtures, surface_gpu};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-gradient-check --list | --adapter INDEX".into()),
    };
    // Canvas, independently literal pixels and all actual Native plans must
    // agree before any Vulkan call. A refusal is an error, never a fallback.
    let fixtures = gradient_fixtures::fixtures()?;
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
            "PASS {name} format={format:?} draws={} compared_bytes={bytes} exact=true reference=literal canvas=true cpu_prepared_gradient=true",
            plan.draws().len()
        );
        Ok(())
    })?;
    if let Some(adapter) = selected {
        if fixtures.len() != 3 || frames != 6 || compared != 608 {
            return Err("gradient completion count".into());
        }
        println!(
            "COMPLETE adapter={adapter} fixtures=3 frames={frames} compared_bytes={compared} exact=true cpu_prepared_gradient=true custom_image_shader=true offscreen=true acquired_surface=false"
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
