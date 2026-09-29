// One invocation owns one pixel. Rectangles are separate ordered compute passes;
// no workgroup barrier can order different dispatches, and none is used here.
struct Params {
    origin: vec2<u32>,
    extent: vec2<u32>,
    canvas: vec2<u32>,
    color: u32,
    reserved: u32,
}
@group(0) @binding(0) var<storage, read_write> pixels: array<u32>;
@group(0) @binding(1) var<uniform> p: Params;

@compute @workgroup_size(8, 8, 1)
fn rectangle(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= p.extent.x || id.y >= p.extent.y) { return; }
    let x = p.origin.x + id.x;
    let y = p.origin.y + id.y;
    if (x >= p.canvas.x || y >= p.canvas.y) { return; }
    let index = y * p.canvas.x + x;
    if (index >= arrayLength(&pixels)) { return; }
    pixels[index] = p.color;
}
