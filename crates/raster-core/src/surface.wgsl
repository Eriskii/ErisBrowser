// Exact integer packing only. No color-space transform or second alpha blend.
// One invocation writes one active pixel; row padding is never copied as pixels.
struct Params {
    width: u32,
    height: u32,
    stride_words: u32,
    red_shift: u32, // 16 for BGRA bytes, 0 for RGBA bytes
}
@group(0) @binding(0) var<storage, read> packed: array<u32>;
@group(0) @binding(1) var<storage, read_write> padded: array<u32>;
@group(0) @binding(2) var<uniform> p: Params;

@compute @workgroup_size(8, 8, 1)
fn surface_convert(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= p.width || id.y >= p.height) { return; }
    let source_index = id.y * p.width + id.x;
    let destination_index = id.y * p.stride_words + id.x;
    if (source_index >= arrayLength(&packed) || destination_index >= arrayLength(&padded)) { return; }
    let rgb = packed[source_index];
    let red = (rgb >> 16u) & 255u;
    let green = rgb & 0x0000ff00u;
    let blue = rgb & 255u;
    padded[destination_index] = 0xff000000u | (red << p.red_shift) | green | (blue << (16u - p.red_shift));
}
