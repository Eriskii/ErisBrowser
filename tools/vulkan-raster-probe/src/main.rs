#![forbid(unsafe_code)]

use eris_vulkan_raster_prototype::{Result, alpha_fixtures, fixtures, gpu, image_fixtures};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).take(4).collect();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => {
            Some(index.parse::<usize>().map_err(|_| "adapter index")?)
        }
        _ => return Err("usage: eris-vulkan-raster-prototype --list | --adapter INDEX".into()),
    };
    // Reject the entire CPU-planned input set before any Vulkan operation.
    let fixtures = fixtures::fixtures();
    let mut images = image_fixtures::fixtures();
    images.extend(alpha_fixtures::fixtures());
    let plans = fixtures
        .iter()
        .map(|f| Ok((f.name, f.expected.as_slice(), f.plan()?)))
        .chain(
            images
                .iter()
                .map(|f| Ok((f.name, f.expected.as_slice(), f.plan()?))),
        )
        .collect::<Result<Vec<_>>>()?;
    let borrowed = plans
        .iter()
        .map(|(name, expected, plan)| (*name, *expected, plan))
        .collect::<Vec<_>>();
    gpu::run_plans(selected, &borrowed, |name, plan, expected| {
        let frame = plan.frame();
        println!(
            "PASS {} {}x{} draws={} invocations={} gpu_buffers={} compared_bytes={} exact=true",
            name,
            frame.width,
            frame.height,
            plan.draws().len(),
            plan.invocations(),
            plan.gpu_buffer_bytes(),
            expected.len() * 4
        );
        Ok(())
    })?;
    if let Some(index) = selected {
        println!(
            "COMPLETE adapter={index} fixtures={} exact=true custom_wgsl=true",
            plans.len()
        );
    }
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("FAIL {e}");
            ExitCode::FAILURE
        }
    }
}
