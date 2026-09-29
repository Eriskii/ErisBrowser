// Source colors and exact scalar sampling indices are immutable CPU inputs.
// Each invocation gathers one original source pixel and writes one output pixel.
// Display-list ordering is supplied by separate compute passes, not a barrier here.
struct Params {
    origin: vec2<u32>,
    extent: vec2<u32>,
    canvas: vec2<u32>,
    color: u32,
    reserved: u32,
    source: vec4<u32>, // base, width, height, pixel count
    tables: vec4<u32>, // x base, y base, x count, y count
}
@group(0) @binding(0) var<storage, read_write> pixels: array<u32>;
@group(0) @binding(1) var<uniform> p: Params;
@group(0) @binding(2) var<storage, read> inputs: array<u32>;

@compute @workgroup_size(8, 8, 1)
fn image(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= p.extent.x || id.y >= p.extent.y) { return; }
    if (p.origin.x >= p.canvas.x || p.origin.y >= p.canvas.y) { return; }
    if (id.x >= p.canvas.x - p.origin.x || id.y >= p.canvas.y - p.origin.y) { return; }
    let x = p.origin.x + id.x;
    let y = p.origin.y + id.y;
    let output_words = arrayLength(&pixels);
    if (p.canvas.x == 0u || p.canvas.x > output_words) { return; }
    if (y >= output_words / p.canvas.x) { return; }
    let output_index = y * p.canvas.x + x;
    if (output_index >= output_words) { return; }

    let words = arrayLength(&inputs);
    if (p.tables.z != p.extent.x || p.tables.w != p.extent.y) { return; }
    if (p.tables.x > words || p.tables.y > words || p.source.x > words) { return; }
    if (p.tables.z > words - p.tables.x || p.tables.w > words - p.tables.y) { return; }
    if (p.source.w == 0u || p.source.w > words - p.source.x) { return; }
    if (p.source.y == 0u || p.source.z == 0u || p.source.y > p.source.w) { return; }
    if (p.source.z != p.source.w / p.source.y || p.source.w % p.source.y != 0u) { return; }
    let sx = inputs[p.tables.x + id.x];
    let sy = inputs[p.tables.y + id.y];
    if (sx >= p.source.y || sy >= p.source.z) { return; }
    // The exact nonzero source dimensions above prove this product cannot wrap.
    let source_index = sy * p.source.y + sx;
    if (source_index >= p.source.w) { return; }
    pixels[output_index] = inputs[p.source.x + source_index];
}
