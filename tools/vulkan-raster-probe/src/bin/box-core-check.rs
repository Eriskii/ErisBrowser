#![forbid(unsafe_code)]
use eris_vulkan_raster_prototype::{Result, box_core_fixtures, surface_gpu};
use std::process::ExitCode;

fn run() -> Result<()> {
    let args = std::env::args().skip(1).take(4).collect::<Vec<_>>();
    let selected = match args.as_slice() {
        [mode] if mode == "--list" => None,
        [mode, index] if mode == "--adapter" => Some(index.parse().map_err(|_| "adapter index")?),
        _ => return Err("usage: eris-vulkan-box-core-check --list | --adapter INDEX".into()),
    };
    // Expected pixels only reach comparators; GPU encoding receives the Native plan.
    let fixtures = box_core_fixtures::fixtures()?;
    let plans = fixtures
        .iter()
        .map(|fixture| (fixture.name, fixture.expected.as_slice(), &fixture.plan))
        .collect::<Vec<_>>();
    let mut frames = 0;
    let mut compared = 0;
    surface_gpu::run(selected, &plans, |name, plan, format, bytes| {
        let fixture = fixtures
            .iter()
            .find(|fixture| fixture.name == name)
            .ok_or("unknown fixture")?;
        let m = fixture.metrics;
        let frame = plan.frame();
        frames += 1;
        compared += bytes;
        println!(
            "PASS {name} format={format:?} width={} height={} draws={} total_invocations={} conversion_invocations={} planned_bytes={} compared_bytes={bytes} cores=1 masks={} coverage={} rows={} box_work={} cpu_work={} lowered={} exact=true reference=literal canvas=true packed_native=true rounded_color_core=true",
            frame.width,
            frame.height,
            plan.draws().len(),
            plan.invocations(),
            plan.conversion_invocations(),
            plan.gpu_buffer_bytes(),
            m.masks,
            m.coverage,
            m.rows,
            m.box_work,
            m.cpu_work,
            m.lowered
        );
        Ok(())
    })?;
    if let Some(adapter) = selected {
        use box_core_fixtures::{COMPARED_BYTES, CONVERTED_FRAMES, FIXTURE_NAMES};
        if fixtures.len() != FIXTURE_NAMES.len()
            || frames != CONVERTED_FRAMES
            || compared != COMPARED_BYTES
        {
            return Err("rounded-core completion count".into());
        }
        println!(
            "COMPLETE adapter={adapter} fixtures=10 frames={frames} compared_bytes={compared} exact=true rounded_color_core=true custom_rectangle_glyph_shaders=true offscreen=true acquired_surface=false"
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
