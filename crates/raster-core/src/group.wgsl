// Native-only cropped RGBA16 groups. Existing RGB shaders remain unchanged.
// One invocation owns a destination pixel; every operation is a separate pass.
struct Params {
    origin: vec2<u32>,
    extent: vec2<u32>,
    canvas: vec2<u32>,
    color: u32,
    reserved: u32,
    source: vec4<u32>,
    tables: vec4<u32>,
    destination: vec4<u32>, // even word base, absolute x, absolute y, width
    destination_info: vec4<u32>, // height, 0=root or 1=group, zero, zero
    group_source: vec4<u32>, // even word base, absolute x, absolute y, width
    group_info: vec4<u32>, // height, opacity numerator k/256, zero, zero
}
@group(0) @binding(0) var<storage, read_write> pixels: array<u32>;
@group(0) @binding(1) var<uniform> p: Params;
@group(0) @binding(2) var<storage, read_write> layers: array<u32>;
@group(0) @binding(3) var<storage, read> inputs: array<u32>;

// Return the arena length as an invalid sentinel. Check all arithmetic before
// multiplication/addition, including the final pair of words.
fn group_index(region: vec4<u32>, height: u32, x: u32, y: u32) -> u32 {
    let words = arrayLength(&layers);
    if ((region.x & 1u) != 0u || region.x > words || region.w == 0u || height == 0u) { return words; }
    if (x < region.y || y < region.z) { return words; }
    let dx = x - region.y;
    let dy = y - region.z;
    if (dx >= region.w || dy >= height) { return words; }
    let available = (words - region.x) / 2u;
    if (region.w > available || height > available / region.w) { return words; }
    return region.x + 2u * (dy * region.w + dx);
}

fn dispatch_visible(id: vec3<u32>) -> bool {
    if (id.x >= p.extent.x || id.y >= p.extent.y) { return false; }
    if (p.origin.x >= p.canvas.x || p.origin.y >= p.canvas.y) { return false; }
    return id.x < p.canvas.x - p.origin.x && id.y < p.canvas.y - p.origin.y;
}

fn load_group(index: u32) -> vec4<u32> {
    let low = layers[index];
    let high = layers[index + 1u];
    return vec4<u32>(high & 65535u, low >> 16u, low & 65535u, high >> 16u);
}
fn store_group(index: u32, value: vec4<u32>) {
    layers[index] = value.z | (value.y << 16u);
    layers[index + 1u] = value.x | (value.w << 16u);
}
fn blend_group(index: u32, color: u32) {
    let alpha = (color >> 24u) * 257u;
    if (alpha == 0u) { return; }
    let rgb = vec3<u32>((color >> 16u) & 255u, (color >> 8u) & 255u, color & 255u);
    let source = vec4<u32>((rgb * alpha + vec3<u32>(127u)) / 255u, alpha);
    // 65535^2+32767=4294868992 fits u32. Premultiplied channels stay <=65535.
    let destination = load_group(index);
    let result = source + (destination * (65535u - alpha) + vec4<u32>(32767u)) / 65535u;
    store_group(index, result);
}

@compute @workgroup_size(8, 8, 1)
fn group_clear(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!dispatch_visible(id) || p.destination_info.y != 1u) { return; }
    let index = group_index(p.destination, p.destination_info.x, p.origin.x + id.x, p.origin.y + id.y);
    if (index >= arrayLength(&layers)) { return; }
    layers[index] = 0u;
    layers[index + 1u] = 0u;
}

@compute @workgroup_size(8, 8, 1)
fn group_rectangle(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!dispatch_visible(id) || p.destination_info.y != 1u) { return; }
    let index = group_index(p.destination, p.destination_info.x, p.origin.x + id.x, p.origin.y + id.y);
    if (index >= arrayLength(&layers)) { return; }
    blend_group(index, p.color);
}

@compute @workgroup_size(8, 8, 1)
fn group_image(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!dispatch_visible(id) || p.destination_info.y != 1u) { return; }
    let index = group_index(p.destination, p.destination_info.x, p.origin.x + id.x, p.origin.y + id.y);
    if (index >= arrayLength(&layers)) { return; }
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
    let source_index = sy * p.source.y + sx;
    if (source_index >= p.source.w) { return; }
    blend_group(index, inputs[p.source.x + source_index]);
}

// Same signed-origin arithmetic as the existing glyph shader.
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
fn group_glyph(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!dispatch_visible(id) || p.destination_info.y != 1u) { return; }
    let x = p.origin.x + id.x;
    let y = p.origin.y + id.y;
    let index = group_index(p.destination, p.destination_info.x, x, y);
    if (index >= arrayLength(&layers)) { return; }
    let words = arrayLength(&inputs);
    if (p.source.x > words || p.tables.x > words) { return; }
    if (p.source.w == 0u || p.source.w > words - p.source.x) { return; }
    if (p.source.y == 0u || p.source.z == 0u || p.source.y > 1024u || p.source.z > 1024u) { return; }
    if (p.source.w > 262144u || p.source.y > p.source.w) { return; }
    if (p.source.z != p.source.w / p.source.y || p.source.w % p.source.y != 0u) { return; }
    if (p.tables.z != p.source.z || p.tables.z > words - p.tables.x) { return; }
    let sy = source_coordinate(y, p.tables.y, p.source.z);
    if (sy >= p.source.z) { return; }
    let sx = source_coordinate(x, inputs[p.tables.x + sy], p.source.y);
    if (sx >= p.source.y) { return; }
    let cell = sy * p.source.y + sx;
    if (cell >= p.source.w) { return; }
    let coverage = inputs[p.source.x + cell];
    if (coverage > 255u) { return; }
    let alpha = (p.color >> 24u) * coverage / 255u;
    blend_group(index, (p.color & 0x00ffffffu) | (alpha << 24u));
}

@compute @workgroup_size(8, 8, 1)
fn group_composite(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!dispatch_visible(id) || p.group_info.y == 0u || p.group_info.y >= 256u) { return; }
    let x = p.origin.x + id.x;
    let y = p.origin.y + id.y;
    let source_index = group_index(p.group_source, p.group_info.x, x, y);
    if (source_index >= arrayLength(&layers)) { return; }
    let source = load_group(source_index);
    // The complete planner proves an opaque backing for this whole source.
    if (source.w != 65535u) { return; }
    let k = p.group_info.y;
    if (p.destination_info.y == 1u) {
        let destination_index = group_index(p.destination, p.destination_info.x, x, y);
        if (destination_index >= arrayLength(&layers)) { return; }
        let destination = load_group(destination_index);
        let numerator = source * k + destination * (256u - k);
        store_group(destination_index, (numerator + vec4<u32>(128u)) / 256u);
    } else if (p.destination_info.y == 0u) {
        let words = arrayLength(&pixels);
        if (p.canvas.x == 0u || p.canvas.x > words || y >= words / p.canvas.x) { return; }
        let index = y * p.canvas.x + x;
        let packed = pixels[index];
        let destination = vec3<u32>((packed >> 16u) & 255u, (packed >> 8u) & 255u, packed & 255u) * 257u;
        let numerator = source.xyz * k + destination * (256u - k);
        let rgb = (numerator + vec3<u32>(32896u)) / 65792u;
        pixels[index] = (rgb.x << 16u) | (rgb.y << 8u) | rgb.z;
    }
}
