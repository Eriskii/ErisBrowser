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
    group_info: vec4<u32>, // height, grid numerator, 0=grid/1=raw, raw f32 bits
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

// Reproduce Canvas's separate binary32 rounding stages using integers. Every
// value is nonnegative and normal; pairs are (low, high) on a 2^-24 scale.
// No optional shader integer width, floating-point division, or FMA is used.
fn opacity_round_even(value: u32, shift: u32) -> u32 {
    if (shift == 0u) { return value; }
    // Caller bounds: 1 <= shift <= 24.
    let quotient = value >> shift;
    let remainder = value & ((1u << shift) - 1u);
    let half = 1u << (shift - 1u);
    return quotient + u32(remainder > half || (remainder == half && (quotient & 1u) != 0u));
}

fn opacity_inverse(alpha: u32, k: u32) -> u32 {
    // Called only for 0 < alpha < 65535, 0 < k < 256.
    let h = 31u - countLeadingZeros(alpha);
    let x = alpha << (23u - h);
    let m = x + (x + 32767u) / 65535u;
    let product = m * k; // < 2^32
    let shift = max(32u - countLeadingZeros(product), 24u) - 24u;
    let rounded = opacity_round_even(product, shift);
    // Keep the original scale even if rounding carries into bit 24.
    return 16777216u - opacity_round_even(rounded, 23u - h - shift);
}

fn opacity_add(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let low = a.x + b.x;
    return vec2<u32>(low, a.y + b.y + u32(low < a.x));
}

fn opacity_product(channel: u32, inverse: u32) -> vec2<u32> {
    let low = channel * (inverse & 65535u);
    let high = channel * (inverse >> 16u);
    let sum = low + (high << 16u);
    return vec2<u32>(sum, (high >> 16u) + u32(sum < low));
}

fn opacity_bits(value: vec2<u32>) -> u32 {
    if (value.y != 0u) { return 64u - countLeadingZeros(value.y); }
    return 32u - countLeadingZeros(value.x);
}

fn opacity_round24(value: vec2<u32>) -> vec2<u32> {
    let bits = opacity_bits(value);
    if (bits <= 24u) { return value; }
    let shift = bits - 24u; // at most 17; no shift by 32
    let quotient = (value.x >> shift) | (value.y << (32u - shift));
    let remainder = value.x & ((1u << shift) - 1u);
    let half = 1u << (shift - 1u);
    let rounded = quotient + u32(remainder > half || (remainder == half && (quotient & 1u) != 0u));
    return vec2<u32>(rounded << shift, rounded >> (32u - shift));
}

fn opacity_channel(source: u32, destination: u32, k: u32, inverse: u32) -> vec2<u32> {
    let weighted = source * k;
    let source_pair = vec2<u32>(weighted << 16u, weighted >> 16u);
    let destination_pair = opacity_round24(opacity_product(destination, inverse));
    return opacity_round24(opacity_add(source_pair, destination_pair));
}

fn opacity_parent(value: vec2<u32>) -> u32 {
    return (value.y << 8u) + (value.x >> 24u) + u32((value.x & 16777215u) >= 8388608u);
}

fn opacity_root(value: vec2<u32>) -> u32 {
    let bits = opacity_bits(value);
    if (bits == 0u) { return 0u; }
    var significand: u32;
    if (bits > 24u) {
        let shift = bits - 24u;
        significand = (value.x >> shift) | (value.y << (32u - shift));
    } else {
        significand = value.x << (24u - bits);
    }
    // Exact normalization of division by257. Its odd denominator has no ties.
    var exponent_shift = 8u;
    if (significand < 8421376u) {
        significand *= 2u;
        exponent_shift = 9u;
    }
    let divided = significand - (significand + 128u) / 257u;
    let shift = 48u + exponent_shift - bits;
    if (shift > 24u) { return 0u; }
    // Positive .round(): halfway away from zero, following the f32 division.
    return (divided + (1u << (shift - 1u))) >> shift;
}

// Full finite-f32 opacity uses the same two-word representation on a 2^-48
// scale. Each shift has an explicit word boundary; no shader u64 is required.
fn opacity_full_shl(value: vec2<u32>, shift: u32) -> vec2<u32> {
    if (shift == 0u) { return value; }
    if (shift >= 64u) { return vec2<u32>(0u); }
    if (shift < 32u) {
        return vec2<u32>(value.x << shift, (value.y << shift) | (value.x >> (32u - shift)));
    }
    return vec2<u32>(0u, value.x << (shift - 32u));
}

fn opacity_full_shr(value: vec2<u32>, shift: u32) -> vec2<u32> {
    if (shift == 0u) { return value; }
    if (shift >= 64u) { return vec2<u32>(0u); }
    if (shift < 32u) {
        return vec2<u32>((value.x >> shift) | (value.y << (32u - shift)), value.y >> shift);
    }
    return vec2<u32>(value.y >> (shift - 32u), 0u);
}

// The retained quotient has at most24 bits; shift is1..40. Guard, sticky and
// parity implement nearest-even, including exactly32 discarded bits.
fn opacity_full_round_shift(value: vec2<u32>, shift: u32) -> u32 {
    let quotient = opacity_full_shr(value, shift).x;
    var above: bool;
    var tie: bool;
    if (shift < 32u) {
        let remainder = value.x & ((1u << shift) - 1u);
        let half = 1u << (shift - 1u);
        above = remainder > half;
        tie = remainder == half;
    } else if (shift == 32u) {
        above = value.x > 2147483648u;
        tie = value.x == 2147483648u;
    } else {
        let upper_shift = shift - 32u;
        let remainder = value.y & ((1u << upper_shift) - 1u);
        let half = 1u << (upper_shift - 1u);
        above = remainder > half || (remainder == half && value.x != 0u);
        tie = remainder == half && value.x == 0u;
    }
    return quotient + u32(above || (tie && (quotient & 1u) != 0u));
}

fn opacity_full_round24(value: vec2<u32>) -> vec2<u32> {
    let bits = opacity_bits(value);
    if (bits <= 24u) { return value; }
    let shift = bits - 24u;
    let rounded = opacity_full_round_shift(value, shift);
    // Valid channel sums are <(65535+6/512)*2^48, so rounding cannot carry
    // into bit64. The alpha-significand product has only48 input bits.
    return opacity_full_shl(vec2<u32>(rounded, 0u), shift);
}

fn opacity_full_product24(a: u32, b: u32) -> vec2<u32> {
    let low = (a & 65535u) * (b & 65535u);
    let cross_a = (a & 65535u) * (b >> 16u);
    let cross_b = (a >> 16u) * (b & 65535u);
    let first = low + (cross_a << 16u);
    let second = first + (cross_b << 16u);
    let high = (a >> 16u) * (b >> 16u) + (cross_a >> 16u) + (cross_b >> 16u)
        + u32(first < low) + u32(second < first);
    return vec2<u32>(second, high);
}

fn opacity_full_inverse(alpha: u32, raw: u32) -> u32 {
    // 2^-25 < opacity <1; exponent102..126, denominator shift24..48.
    let r = 150u - (raw >> 23u);
    let mantissa = (raw & 8388607u) | 8388608u;
    var rounded = mantissa;
    var denominator = r;
    if (alpha != 65535u) {
        let h = 31u - countLeadingZeros(alpha);
        let x = alpha << (23u - h);
        let alpha_mantissa = x + (x + 32767u) / 65535u;
        let product = opacity_full_product24(alpha_mantissa, mantissa);
        let shift = opacity_bits(product) - 24u; //23 or24
        rounded = opacity_full_round_shift(product, shift);
        denominator = 39u + r - h - shift;
    }
    let shift = denominator - 24u;
    // rounded<=2^24. At shift25 its largest value is a half-way tie to0.
    if (shift > 24u) { return 16777216u; }
    return 16777216u - opacity_round_even(rounded, shift);
}

fn opacity_full_channel(source: u32, destination: u32, raw: u32, inverse: u32) -> vec2<u32> {
    let r = 150u - (raw >> 23u);
    let mantissa = (raw & 8388607u) | 8388608u;
    // Unlike k/256, the source multiplication also requires rounding.
    let weighted_source = opacity_round24(opacity_product(source, mantissa));
    let weighted_destination = opacity_round24(opacity_product(destination, inverse));
    let sum = opacity_add(opacity_full_shl(weighted_source, 48u - r),
                          opacity_full_shl(weighted_destination, 24u));
    return opacity_full_round24(sum);
}

fn opacity_full_parent(value: vec2<u32>) -> u32 {
    return (value.y >> 16u) + u32((value.y & 65535u) >= 32768u);
}

fn opacity_full_root(value: vec2<u32>) -> u32 {
    let bits = opacity_bits(value);
    if (bits == 0u) { return 0u; }
    var significand: u32;
    if (bits > 24u) {
        significand = opacity_full_shr(value, bits - 24u).x;
    } else {
        significand = value.x << (24u - bits);
    }
    var exponent_shift = 8u;
    if (significand < 8421376u) {
        significand *= 2u;
        exponent_shift = 9u;
    }
    let divided = significand - (significand + 128u) / 257u;
    let shift = 72u + exponent_shift - bits;
    if (shift > 24u) { return 0u; }
    return (divided + (1u << (shift - 1u))) >> shift;
}

@compute @workgroup_size(8, 8, 1)
fn group_composite(@builtin(global_invocation_id) id: vec3<u32>) {
    if (!dispatch_visible(id)) { return; }
    let full = p.group_info.z == 1u;
    let raw = p.group_info.w;
    if (full) {
        if (p.group_info.y != 0u || raw == 0u || raw >= 1065353216u) { return; }
        // Exact identity of the final stored parent/root pixel, including all
        // subnormal opacities. Planning, painting and dispatch charges remain.
        if (raw <= 855638016u) { return; } //2^-25
    } else if (p.group_info.z != 0u || raw != 0u || p.group_info.y == 0u || p.group_info.y >= 256u) {
        return;
    }
    let x = p.origin.x + id.x;
    let y = p.origin.y + id.y;
    let source_index = group_index(p.group_source, p.group_info.x, x, y);
    if (source_index >= arrayLength(&layers)) { return; }
    let source = load_group(source_index);
    if (source.w == 0u) { return; }
    let k = p.group_info.y;
    var inverse = 0u;
    if (full) {
        inverse = opacity_full_inverse(source.w, raw);
    } else if (source.w != 65535u) {
        inverse = opacity_inverse(source.w, k);
    }
    if (p.destination_info.y == 1u) {
        let destination_index = group_index(p.destination, p.destination_info.x, x, y);
        if (destination_index >= arrayLength(&layers)) { return; }
        let destination = load_group(destination_index);
        if (full) {
            store_group(destination_index, vec4<u32>(
                opacity_full_parent(opacity_full_channel(source.x, destination.x, raw, inverse)),
                opacity_full_parent(opacity_full_channel(source.y, destination.y, raw, inverse)),
                opacity_full_parent(opacity_full_channel(source.z, destination.z, raw, inverse)),
                opacity_full_parent(opacity_full_channel(source.w, destination.w, raw, inverse))
            ));
        } else if (source.w == 65535u) {
            let numerator = source * k + destination * (256u - k);
            store_group(destination_index, (numerator + vec4<u32>(128u)) / 256u);
        } else {
            store_group(destination_index, vec4<u32>(
                opacity_parent(opacity_channel(source.x, destination.x, k, inverse)),
                opacity_parent(opacity_channel(source.y, destination.y, k, inverse)),
                opacity_parent(opacity_channel(source.z, destination.z, k, inverse)),
                opacity_parent(opacity_channel(source.w, destination.w, k, inverse))
            ));
        }
    } else if (p.destination_info.y == 0u) {
        let words = arrayLength(&pixels);
        if (p.canvas.x == 0u || p.canvas.x > words || y >= words / p.canvas.x) { return; }
        let index = y * p.canvas.x + x;
        let packed = pixels[index];
        let destination = vec3<u32>((packed >> 16u) & 255u, (packed >> 8u) & 255u, packed & 255u) * 257u;
        var rgb: vec3<u32>;
        if (full) {
            rgb = vec3<u32>(
                opacity_full_root(opacity_full_channel(source.x, destination.x, raw, inverse)),
                opacity_full_root(opacity_full_channel(source.y, destination.y, raw, inverse)),
                opacity_full_root(opacity_full_channel(source.z, destination.z, raw, inverse))
            );
        } else if (source.w == 65535u) {
            let numerator = source.xyz * k + destination * (256u - k);
            rgb = (numerator + vec3<u32>(32896u)) / 65792u;
        } else {
            rgb = vec3<u32>(
                opacity_root(opacity_channel(source.x, destination.x, k, inverse)),
                opacity_root(opacity_channel(source.y, destination.y, k, inverse)),
                opacity_root(opacity_channel(source.z, destination.z, k, inverse))
            );
        }
        pixels[index] = (rgb.x << 16u) | (rgb.y << 8u) | rgb.z;
    }
}
