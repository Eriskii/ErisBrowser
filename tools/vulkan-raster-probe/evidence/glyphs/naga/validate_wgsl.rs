#![forbid(unsafe_code)]
fn main() {
    let source = include_str!("glyph.wgsl");
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("glyph.wgsl: {}", error.emit_to_string(source)));
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty(),
    ).validate(&module).unwrap_or_else(|error| panic!("glyph.wgsl: {error:?}"));
    assert_eq!(module.entry_points.len(), 1);
    assert_eq!(module.entry_points[0].name, "glyph");
    assert_eq!(module.entry_points[0].workgroup_size, [8, 8, 1]);
    assert_eq!(module.entry_points[0].stage, naga::ShaderStage::Compute);
    assert!(info.get_entry_point(0).available_stages.contains(naga::valid::ShaderStages::COMPUTE));
    println!("PASS glyph.wgsl Naga=30.0.1 parse=true validation=true flags=all capabilities=empty entry=glyph workgroup=8x8x1");
    println!("COMPLETE shaders=1 offline=true gpu_execution=false");
}
