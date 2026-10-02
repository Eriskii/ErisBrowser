#![forbid(unsafe_code)]
use eris_vulkan_raster_prototype::{Result, reuse_fixtures, reuse_gpu};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-reuse-check --list | --adapter INDEX".into()),
    };
    let cases = reuse_fixtures::fixtures()?;
    let counts = reuse_gpu::run(selected, &cases)?;
    if let Some(index) = selected {
        println!(
            "COMPLETE adapter={index} frames={} allocations={} reuses={} evictions={} formats=2 compared_bytes={} offscreen=true acquired_surface=false exact=true",
            counts.frames,
            counts.allocations,
            counts.reuses,
            counts.evictions,
            counts.compared_bytes
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
