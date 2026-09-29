#![forbid(unsafe_code)]
fn main() {
    for (name, entry, source) in [
        ("rect.wgsl", "rectangle", include_str!("rect.wgsl")),
        ("image.wgsl", "image", include_str!("image.wgsl")),
    ] {
        let module = naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|error| panic!("{name}: {}", error.emit_to_string(source)));
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty(),
        ).validate(&module).unwrap_or_else(|error| panic!("{name}: {error:?}"));
        assert_eq!(module.entry_points.len(), 1, "{name}");
        assert_eq!(module.entry_points[0].name, entry, "{name}");
        assert_eq!(module.entry_points[0].workgroup_size, [8, 8, 1], "{name}");
        assert_eq!(module.entry_points[0].stage, naga::ShaderStage::Compute, "{name}");
        assert!(info.get_entry_point(0).available_stages.contains(naga::valid::ShaderStages::COMPUTE));
        println!("PASS {name} Naga=30.0.1 parse=true validation=true flags=all capabilities=empty entry={entry} workgroup=8x8x1");
    }
    println!("COMPLETE shaders=2 offline=true gpu_execution=false");
}
