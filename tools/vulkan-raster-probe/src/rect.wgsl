// One invocation owns one pixel. Rectangles are separate ordered compute passes;
// no workgroup barrier can order different dispatches, and none is used here.
struct Params {
    origin: vec2<u32>,
    extent: vec2<u32>,
    canvas: vec2<u32>,
    color: u32, // source 0xAARRGGBB; clear always has alpha 255
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
    let alpha = p.color >> 24u;
    if (alpha == 0u) { return; }
    if (alpha == 255u) {
        // Clear reaches this branch without reading the previous target.
        pixels[index] = p.color & 0x00ffffffu;
        return;
    }
    let destination = pixels[index];
    let inverse = 255u - alpha;
    // Match Canvas::blend, rounding each ordered pass separately. Each
    // channel numerator is at most 255*255+127, well within u32.
    let r = (((p.color >> 16u) & 255u) * alpha + ((destination >> 16u) & 255u) * inverse + 127u) / 255u;
    let g = (((p.color >> 8u) & 255u) * alpha + ((destination >> 8u) & 255u) * inverse + 127u) / 255u;
    let b = ((p.color & 255u) * alpha + (destination & 255u) * inverse + 127u) / 255u;
    pixels[index] = (r << 16u) | (g << 8u) | b;
}
