#![forbid(unsafe_code)]
use eris_vulkan_raster_prototype::{Result, opacity_fixtures, reuse_gpu, surface_gpu};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-opacity-check --list | --adapter INDEX".into()),
    };
    let cases = opacity_fixtures::fixtures()?;
    if cases.len() != 9 {
        return Err("opacity literal inventory".into());
    }
    let plans = cases
        .iter()
        .map(|case| (case.name, case.expected.as_slice(), &case.plan))
        .collect::<Vec<_>>();
    let mut frames = 0;
    let mut compared = 0;
    surface_gpu::run(selected, &plans, |name, plan, format, bytes| {
        frames += 1;
        compared += bytes;
        println!(
            "PASS opacity-{name} format={format:?} scratch_bytes={} compared_bytes={bytes} exact=true reference=literal",
            plan.group_scratch_bytes()
        );
        Ok(())
    })?;
    if let Some(adapter) = selected {
        if frames != 18 || compared != 376 {
            return Err("opacity literal completion count".into());
        }
        let reuse = opacity_fixtures::reuse_fixtures()?;
        let counts = reuse_gpu::run_opacity(Some(adapter), &reuse)?;
        println!(
            "COMPLETE adapter={adapter} literal_frames={frames} literal_bytes={compared} refusals=3 reuse_frames={} reuse_bytes={} allocations={} reuses={} evictions={} offscreen=true acquired_surface=false exact=true",
            counts.frames,
            counts.compared_bytes,
            counts.allocations,
            counts.reuses,
            counts.evictions
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
