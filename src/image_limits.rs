//! Allocation preflight for the enabled WebP decoder, not a bitstream decoder.
//!
//! `image-webp` 0.2.4 allocates VP8 planes from the inner frame header before
//! comparing them with the extended canvas/animation frame. The image wrapper's
//! output limits therefore do not protect that allocation. Inspect each frame's
//! uncompressed dimensions before constructing any decoder. Compressed data and
//! metadata remain the codec's responsibility; scratch memory is still bounded
//! by the isolated decoder process, not by the RGBA output budget alone.
//!
//! Header definitions: https://developers.google.com/speed/webp/docs/riff_container
//! https://developers.google.com/speed/webp/docs/webp_lossless_bitstream_specification
//! https://www.rfc-editor.org/rfc/rfc6386.html#section-9.1

const MAX_AXIS: u32 = 4096;
const MAX_RGBA_BYTES: usize = 64 * 1024 * 1024;

type Dimensions = (u32, u32);
type Chunk<'a> = (&'a [u8], &'a [u8]);

/// Other image formats pass through unchanged. A WebP is checked without
/// allocating or recursing over attacker-controlled chunks. Work is linear in
/// its encoded length, which is bounded by the ordinary resource byte limit.
pub(crate) fn validate_webp(bytes: &[u8], pixel_budget: usize) -> Result<(), String> {
    if !bytes.starts_with(b"RIFF") || bytes.get(8..12) != Some(b"WEBP") {
        return Ok(());
    }
    if bytes.len() > crate::net::MAX_RESOURCE_BYTES {
        return Err("WebP encoded byte limit exceeded".into());
    }
    let declared = u64::from(le32(&bytes[4..8])) + 8;
    // The dependency may scan past the declared RIFF endpoint. Require one
    // complete container so neither trailing bytes nor truncated chunks can be
    // interpreted differently by the preflight and decoder.
    if declared != bytes.len() as u64 || !bytes.len().is_multiple_of(2) {
        return Err("invalid WebP RIFF length".into());
    }
    let budget = pixel_budget.min(MAX_RGBA_BYTES);
    let mut chunks = Chunks::new(&bytes[12..]);
    let (first_tag, first_data) = chunks.next()?.ok_or("missing WebP image header")?;
    let extended = first_tag == b"VP8X";
    let canvas = if extended {
        if first_data.len() < 10 {
            return Err("truncated WebP canvas header".into());
        }
        dimensions(
            le24(&first_data[4..7]) + 1,
            le24(&first_data[7..10]) + 1,
            budget,
        )?
    } else {
        bitstream_dimensions(first_tag, first_data, budget)?
    };
    let mut has_image = !extended;
    while let Some((tag, payload)) = chunks.next()? {
        match tag {
            b"VP8 " | b"VP8L" => {
                if bitstream_dimensions(tag, payload, budget)? != canvas {
                    return Err("WebP frame and canvas dimensions differ".into());
                }
                has_image = true;
            }
            b"ANMF" => {
                if !extended {
                    return Err("WebP animation requires a canvas header".into());
                }
                animation_frame(payload, canvas, budget)?;
                has_image = true;
            }
            b"VP8X" => return Err("duplicate WebP canvas header".into()),
            _ => {}
        }
    }
    if !has_image {
        return Err("missing WebP image bitstream".into());
    }
    Ok(())
}

fn dimensions(width: u32, height: u32, budget: usize) -> Result<Dimensions, String> {
    if width == 0
        || height == 0
        || width > MAX_AXIS
        || height > MAX_AXIS
        || u64::from(width) * u64::from(height) * 4 > budget as u64
    {
        return Err("WebP pixel limit exceeded".into());
    }
    Ok((width, height))
}

fn bitstream_dimensions(tag: &[u8], data: &[u8], budget: usize) -> Result<Dimensions, String> {
    match tag {
        b"VP8 " => {
            if data.len() < 10 || data[0] & 1 != 0 || data[3..6] != [0x9d, 0x01, 0x2a] {
                return Err("invalid WebP VP8 keyframe header".into());
            }
            let width = u16::from_le_bytes([data[6], data[7]]) & 0x3fff;
            let height = u16::from_le_bytes([data[8], data[9]]) & 0x3fff;
            dimensions(u32::from(width), u32::from(height), budget)
        }
        b"VP8L" => {
            if data.len() < 5 || data[0] != 0x2f {
                return Err("invalid WebP VP8L header".into());
            }
            let packed = le32(&data[1..5]);
            if packed >> 29 != 0 {
                return Err("unsupported WebP lossless version".into());
            }
            dimensions((packed & 0x3fff) + 1, ((packed >> 14) & 0x3fff) + 1, budget)
        }
        _ => Err("invalid WebP image header".into()),
    }
}

fn animation_frame(data: &[u8], canvas: Dimensions, budget: usize) -> Result<(), String> {
    if data.len() < 16 {
        return Err("truncated WebP animation frame header".into());
    }
    let x = le24(&data[..3]) * 2;
    let y = le24(&data[3..6]) * 2;
    let frame = dimensions(le24(&data[6..9]) + 1, le24(&data[9..12]) + 1, budget)?;
    if u64::from(x) + u64::from(frame.0) > u64::from(canvas.0)
        || u64::from(y) + u64::from(frame.1) > u64::from(canvas.1)
    {
        return Err("WebP animation frame is outside its canvas".into());
    }
    let mut chunks = Chunks::new(&data[16..]);
    let (mut tag, mut payload) = chunks.next()?.ok_or("missing WebP animation bitstream")?;
    if tag == b"ALPH" {
        (tag, payload) = chunks.next()?.ok_or("missing WebP alpha frame bitstream")?;
        // The pinned decoder treats this payload as VP8 even if its FourCC is
        // different. Enforce the format's order before trusting chunk tags.
        if tag != b"VP8 " {
            return Err("WebP alpha frame must contain a VP8 bitstream".into());
        }
    }
    if bitstream_dimensions(tag, payload, budget)? != frame {
        return Err("WebP bitstream and animation dimensions differ".into());
    }
    while let Some((tag, payload)) = chunks.next()? {
        match tag {
            b"VP8 " | b"VP8L" if bitstream_dimensions(tag, payload, budget)? != frame => {
                return Err("WebP bitstream and animation dimensions differ".into());
            }
            b"ANMF" | b"VP8X" => return Err("nested WebP frame header".into()),
            _ => {}
        }
    }
    Ok(())
}

/// Each successful step consumes at least eight bytes; payloads are borrowed
/// and animation only adds one fixed nesting level.
struct Chunks<'a> {
    remaining: &'a [u8],
}

impl<'a> Chunks<'a> {
    fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    fn next(&mut self) -> Result<Option<Chunk<'a>>, String> {
        if self.remaining.is_empty() {
            return Ok(None);
        }
        if self.remaining.len() < 8 {
            return Err("truncated WebP chunk header".into());
        }
        let length = le32(&self.remaining[4..8]) as u64;
        let padded = length + (length & 1);
        if padded > (self.remaining.len() - 8) as u64 {
            return Err("truncated WebP chunk payload".into());
        }
        let end = 8 + length as usize;
        if length & 1 != 0 && self.remaining[end] != 0 {
            return Err("invalid WebP chunk padding".into());
        }
        let result = (&self.remaining[..4], &self.remaining[8..end]);
        self.remaining = &self.remaining[8 + padded as usize..];
        Ok(Some(result))
    }
}

fn le24(bytes: &[u8]) -> u32 {
    u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16)
}

fn le32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn chunk(tag: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(payload);
        if !payload.len().is_multiple_of(2) {
            out.push(0);
        }
        out
    }

    fn container(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = b"RIFF".to_vec();
        let length = 4 + chunks.iter().map(Vec::len).sum::<usize>();
        out.extend_from_slice(&(length as u32).to_le_bytes());
        out.extend_from_slice(b"WEBP");
        for chunk in chunks {
            out.extend_from_slice(chunk);
        }
        out
    }

    fn vp8(width: u16, height: u16) -> Vec<u8> {
        let mut payload = vec![0x10, 0, 0, 0x9d, 0x01, 0x2a];
        payload.extend_from_slice(&width.to_le_bytes());
        payload.extend_from_slice(&height.to_le_bytes());
        chunk(b"VP8 ", &payload)
    }

    fn vp8l(width: u32, height: u32) -> Vec<u8> {
        let mut payload = vec![0x2f];
        payload.extend_from_slice(&((width - 1) | ((height - 1) << 14)).to_le_bytes());
        chunk(b"VP8L", &payload)
    }

    fn canvas(width: u32, height: u32, animation: bool) -> Vec<u8> {
        let mut payload = vec![if animation { 2 } else { 0 }, 0, 0, 0];
        payload.extend_from_slice(&(width - 1).to_le_bytes()[..3]);
        payload.extend_from_slice(&(height - 1).to_le_bytes()[..3]);
        chunk(b"VP8X", &payload)
    }

    fn frame(x: u32, y: u32, width: u32, height: u32, chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut payload = Vec::new();
        for n in [x / 2, y / 2, width - 1, height - 1, 1] {
            payload.extend_from_slice(&n.to_le_bytes()[..3]);
        }
        payload.push(0);
        for chunk in chunks {
            payload.extend_from_slice(chunk);
        }
        chunk(b"ANMF", &payload)
    }

    #[test]
    fn oversized_inner_vp8_is_rejected_before_production_decoder_construction() {
        let source = include_bytes!("../tests/fixtures/webp-oversized-inner-vp8.webp");
        assert_eq!(source.len(), 48);
        assert!(
            validate_webp(source, MAX_RGBA_BYTES)
                .unwrap_err()
                .contains("pixel limit")
        );
        assert!(
            crate::page::decode_image_with_budget(source, MAX_RGBA_BYTES)
                .unwrap_err()
                .contains("WebP pixel limit")
        );
    }

    #[test]
    fn frame_headers_enforce_axes_budget_and_canvas_consistency() {
        for image in [vp8(4097, 1), vp8(0, 1), vp8l(1, 4097), vp8l(16384, 16384)] {
            assert!(validate_webp(&container(&[image]), MAX_RGBA_BYTES).is_err());
        }
        let bounded = container(&[vp8l(8, 4)]);
        assert!(validate_webp(&bounded, 128).is_ok());
        assert!(validate_webp(&bounded, 127).is_err());
        assert!(validate_webp(&container(&[canvas(1, 1, false), vp8(2, 1)]), 16).is_err());
        assert!(validate_webp(&container(&[canvas(1, 1, false), vp8l(1, 2)]), 16).is_err());
        assert!(
            validate_webp(
                &container(&[canvas(4097, 1, false), vp8(1, 1)]),
                MAX_RGBA_BYTES
            )
            .is_err()
        );
        // VP8 scaling bits are not part of its encoded pixel dimensions.
        assert!(validate_webp(&container(&[vp8(0xc002, 0x8003)]), 24).is_ok());
    }

    #[test]
    fn animation_checks_all_frames_offsets_and_alpha_bitstream_order() {
        let valid = frame(2, 2, 2, 2, &[vp8(2, 2)]);
        let base = canvas(4, 4, true);
        assert!(validate_webp(&container(&[base.clone(), valid.clone()]), 64).is_ok());
        for invalid in [
            frame(4, 0, 1, 1, &[vp8(1, 1)]),
            frame(0, 0, 1, 1, &[vp8(16383, 16383)]),
            frame(0, 0, 1, 1, &[vp8l(2, 1)]),
            frame(0, 0, 4097, 1, &[vp8(1, 1)]),
            frame(0, 0, 1, 1, &[chunk(b"ALPH", &[0, 255]), vp8l(1, 1)]),
        ] {
            // Even a later frame must be checked before the first is decoded.
            assert!(
                validate_webp(&container(&[base.clone(), valid.clone(), invalid]), 64).is_err()
            );
        }
        let mut disguised = vp8(16383, 16383);
        disguised[..4].copy_from_slice(b"JUNK");
        let alpha = frame(0, 0, 1, 1, &[chunk(b"ALPH", &[0, 255]), disguised]);
        assert!(
            validate_webp(&container(&[base, alpha]), 64)
                .unwrap_err()
                .contains("alpha frame must contain a VP8")
        );
    }

    #[test]
    fn riff_and_nested_chunk_lengths_are_checked_without_allocation() {
        let valid = container(&[canvas(1, 1, true), frame(0, 0, 1, 1, &[vp8l(1, 1)])]);
        for n in 12..valid.len() {
            let mut truncated = valid[..n].to_vec();
            truncated[4..8].copy_from_slice(&((n - 8) as u32).to_le_bytes());
            assert!(validate_webp(&truncated, 4).is_err(), "accepted prefix {n}");
        }
        let mut overlong = valid.clone();
        overlong.extend_from_slice(&[0, 0]);
        assert!(validate_webp(&overlong, 4).is_err());
        let mut nested_length = valid;
        // RIFF(12), VP8X(18), ANMF header(8), frame header(16), FourCC(4).
        nested_length[58..62].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(validate_webp(&nested_length, 4).is_err());
        let mut padding = container(&[vp8l(1, 1)]);
        *padding.last_mut().unwrap() = 1;
        assert!(validate_webp(&padding, 4).is_err());
        let mut metadata = vec![vp8(1, 1)];
        metadata.extend((0..10000).map(|_| chunk(b"JUNK", &[])));
        assert!(validate_webp(&container(&metadata), 4).is_ok());
    }

    #[test]
    fn ordinary_lossy_lossless_extended_and_animated_webp_still_decode() {
        let lossy = include_bytes!("../tests/fixtures/webp-lossy-2x2.webp");
        let mut lossless = Vec::new();
        let rgba = [0x24, 0x68, 0xac, 0xff].repeat(4);
        image::codecs::webp::WebPEncoder::new_lossless(&mut lossless)
            .encode(&rgba, 2, 2, image::ExtendedColorType::Rgba8)
            .unwrap();
        let lossless_chunk = lossless[12..].to_vec();
        let extended = container(&[
            canvas(2, 2, false),
            lossless_chunk.clone(),
            chunk(b"JUNK", b"metadata"),
        ]);
        let animated = container(&[
            canvas(2, 2, true),
            chunk(b"ANIM", &[0; 6]),
            frame(0, 0, 2, 2, &[lossless_chunk]),
        ]);
        let mut alpha_canvas = canvas(2, 2, false);
        alpha_canvas[8] |= 0x10;
        let alpha = chunk(b"ALPH", &[0, 128, 128, 128, 128]);
        let alpha_still = container(&[alpha_canvas.clone(), alpha.clone(), lossy[12..].to_vec()]);
        alpha_canvas[8] |= 2;
        let alpha_animated = container(&[
            alpha_canvas,
            chunk(b"ANIM", &[0; 6]),
            frame(0, 0, 2, 2, &[alpha, lossy[12..].to_vec()]),
        ]);
        for source in [
            lossy.as_slice(),
            &lossless,
            &extended,
            &animated,
            &alpha_still,
            &alpha_animated,
        ] {
            validate_webp(source, 16).unwrap();
            let decoded = crate::page::decode_image_with_budget(source, 16).unwrap();
            assert_eq!(
                (decoded.width, decoded.height, decoded.rgba.len()),
                (2, 2, 16)
            );
        }
    }

    #[test]
    fn non_webp_png_and_jpeg_still_use_the_existing_decoder() {
        let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            2,
            2,
            image::Rgb([12, 34, 56]),
        ));
        for format in [image::ImageFormat::Png, image::ImageFormat::Jpeg] {
            let mut encoded = Cursor::new(Vec::new());
            image.write_to(&mut encoded, format).unwrap();
            let source = encoded.into_inner();
            // Even a zero WebP allowance has no effect on other formats.
            validate_webp(&source, 0).unwrap();
            let decoded = crate::page::decode_image_with_budget(&source, 16).unwrap();
            assert_eq!((decoded.width, decoded.height), (2, 2));
        }
    }
}
