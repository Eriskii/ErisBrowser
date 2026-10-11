#![forbid(unsafe_code)]
use eris_vulkan_raster_prototype::{Result, native_box_fixtures, surface_gpu};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-native-box-check --list | --adapter INDEX".into()),
    };
    // Neither GPU encoder receives expected pixels. No CPU fallback is accepted.
    let fixtures = native_box_fixtures::fixtures()?;
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
            "PASS {name} format={format:?} draws={} compared_bytes={bytes} exact=true reference=literal canvas=true packed_native=true color_box_masks=true",
            plan.draws().len()
        );
        Ok(())
    })?;
    if let Some(adapter) = selected {
        use native_box_fixtures::{COMPARED_BYTES, CONVERTED_FRAMES, FIXTURE_NAMES};
        if fixtures.len() != FIXTURE_NAMES.len()
            || frames != CONVERTED_FRAMES
            || compared != COMPARED_BYTES
        {
            return Err("native Color box completion count".into());
        }
        println!(
            "COMPLETE adapter={adapter} fixtures=10 frames={frames} compared_bytes={compared} exact=true color_box_masks=true custom_glyph_shader=true offscreen=true acquired_surface=false"
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
