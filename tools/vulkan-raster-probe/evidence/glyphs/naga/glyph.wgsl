// Coverage bytes are independent font/mask inputs, not CPU-composited pixels.
// Each invocation owns one destination pixel; glyphs use ordered separate passes.
struct Params {
    origin: vec2<u32>,
    extent: vec2<u32>,
    canvas: vec2<u32>,
    color: u32,
    reserved: u32,
    source: vec4<u32>, // coverage base, width, height, cell count
    rows: vec4<u32>, // row base, bitcast absolute signed y, row count, reserved
}
@group(0) @binding(0) var<storage, read_write> pixels: array<u32>;
@group(0) @binding(1) var<uniform> p: Params;
@group(0) @binding(2) var<storage, read> inputs: array<u32>;

// Return extent as an invalid sentinel. Neither signed subtraction nor signed
// negation is used; even an i32::MIN origin is handled without overflow.
fn source_coordinate(destination: u32, origin_bits: u32, extent: u32) -> u32 {
    let origin = bitcast<i32>(origin_bits);
    if (origin >= 0) {
        let base = u32(origin);
        if (destination < base) { return extent; }
        let delta = destination - base;
        if (delta >= extent) { return extent; }
        return delta;
    }
    let magnitude = (~origin_bits) + 1u;
    if (magnitude >= extent) { return extent; }
    if (destination >= extent - magnitude) { return extent; }
    return destination + magnitude;
}

@compute @workgroup_size(8, 8, 1)
fn glyph(@builtin(global_invocation_id) id: vec3<u32>) {
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
    if (p.source.x > words || p.rows.x > words) { return; }
    if (p.source.w == 0u || p.source.w > words - p.source.x) { return; }
    if (p.source.y == 0u || p.source.z == 0u || p.source.y > 1024u || p.source.z > 1024u) { return; }
    if (p.source.w > 262144u || p.source.y > p.source.w) { return; }
    if (p.source.z != p.source.w / p.source.y || p.source.w % p.source.y != 0u) { return; }
    if (p.rows.z != p.source.z || p.rows.z > words - p.rows.x) { return; }
    let sy = source_coordinate(y, p.rows.y, p.source.z);
    if (sy >= p.source.z) { return; }
    let sx = source_coordinate(x, inputs[p.rows.x + sy], p.source.y);
    if (sx >= p.source.y) { return; }
    // Exact validated dimensions and arena ranges prove both additions/products.
    let cell = sy * p.source.y + sx;
    if (cell >= p.source.w) { return; }
    let coverage = inputs[p.source.x + cell];
    if (coverage > 255u) { return; }
    // Canvas first truncates color-alpha times coverage, then rounds RGB over.
    let alpha = (p.color >> 24u) * coverage / 255u;
    if (alpha == 0u) { return; }
    if (alpha == 255u) {
        pixels[output_index] = p.color & 0x00ffffffu;
        return;
    }
    let destination = pixels[output_index];
    let inverse = 255u - alpha;
    let r = (((p.color >> 16u) & 255u) * alpha + ((destination >> 16u) & 255u) * inverse + 127u) / 255u;
    let g = (((p.color >> 8u) & 255u) * alpha + ((destination >> 8u) & 255u) * inverse + 127u) / 255u;
    let b = ((p.color & 255u) * alpha + (destination & 255u) * inverse + 127u) / 255u;
    pixels[output_index] = (r << 16u) | (g << 8u) | b;
}
