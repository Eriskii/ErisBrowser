use super::control::{MAX_PIXELS, PIXEL_BYTES};

pub(super) fn row_layout((width, height): (u32, u32)) -> Result<(u32, u32, usize), String> {
    if width == 0
        || height == 0
        || width > 8192
        || height > 8192
        || u64::from(width) * u64::from(height) > MAX_PIXELS as u64
    {
        return Err("Vulkan transfer dimensions outside budget".into());
    }
    let packed = width.checked_mul(4).ok_or("row overflow")?;
    let padded =
        packed.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bytes = usize::try_from(u64::from(padded) * u64::from(height))
        .map_err(|_| "readback byte overflow")?;
    Ok((packed, padded, bytes))
}

pub(super) fn convert(
    words: &[u32],
    format: wgpu::TextureFormat,
    output: &mut Vec<u8>,
) -> Result<(), String> {
    if !matches!(
        format,
        wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
    ) {
        return Err("unsupported Vulkan upload format".into());
    }
    let length = words
        .len()
        .checked_mul(4)
        .filter(|n| *n <= PIXEL_BYTES)
        .ok_or("Vulkan conversion exceeds budget")?;
    output.clear();
    output
        .try_reserve_exact(length)
        .map_err(|_| "Vulkan conversion allocation failed")?;
    if output.capacity() > PIXEL_BYTES {
        return Err("Vulkan conversion capacity exceeds budget".into());
    }
    for word in words {
        let [b, g, r, _unused] = word.to_le_bytes();
        let pixel = if format == wgpu::TextureFormat::Bgra8Unorm {
            [b, g, r, 255]
        } else {
            [r, g, b, 255]
        };
        output.extend_from_slice(&pixel);
    }
    Ok(())
}

pub(super) fn compare(
    mapped: &[u8],
    expected: &[u8],
    size: (u32, u32),
    padded: u32,
) -> Result<(), String> {
    let (packed, required_padded, bytes) = row_layout(size)?;
    if padded != required_padded
        || mapped.len() != bytes
        || expected.len() != packed as usize * size.1 as usize
    {
        return Err("Vulkan verification buffer length/stride mismatch".into());
    }
    for y in 0..size.1 as usize {
        let actual = &mapped[y * padded as usize..y * padded as usize + packed as usize];
        let reference = &expected[y * packed as usize..(y + 1) * packed as usize];
        if let Some(x) = actual.iter().zip(reference).position(|(a, b)| a != b) {
            return Err(format!(
                "Vulkan acquired-texture mismatch at row {y}, byte {x}: actual={}, expected={}",
                actual[x], reference[x]
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversion_always_sets_opaque_alpha_and_handles_channel_order() {
        let mut bytes = Vec::new();
        for format in [
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureFormat::Bgra8Unorm,
        ] {
            convert(&[0xa5123456, 0x00000000, 0xffffffff], format, &mut bytes).unwrap();
            assert_eq!(&bytes[4..], &[0, 0, 0, 255, 255, 255, 255, 255]);
            assert_eq!(
                &bytes[..4],
                if format == wgpu::TextureFormat::Bgra8Unorm {
                    &[0x56, 0x34, 0x12, 255]
                } else {
                    &[0x12, 0x34, 0x56, 255]
                }
            );
        }
        assert!(convert(&[0], wgpu::TextureFormat::Rgba8UnormSrgb, &mut bytes).is_err());
    }
    #[test]
    fn odd_width_readback_checks_every_pixel_and_ignores_only_padding() {
        let (packed, padded, bytes) = row_layout((3, 2)).unwrap();
        assert_eq!((packed, padded, bytes), (12, 256, 512));
        let expected: Vec<u8> = (0..24).collect();
        let mut mapped = vec![222; bytes];
        mapped[..12].copy_from_slice(&expected[..12]);
        mapped[256..268].copy_from_slice(&expected[12..]);
        compare(&mapped, &expected, (3, 2), padded).unwrap();
        for index in (0..12).chain(256..268) {
            mapped[index] ^= 1;
            assert!(compare(&mapped, &expected, (3, 2), padded).is_err());
            mapped[index] ^= 1;
        }
        assert!(compare(&mapped[..511], &expected, (3, 2), padded).is_err());
        assert!(compare(&mapped, &expected, (3, 2), 12).is_err());
        for size in [
            (0, 1),
            (1, 0),
            (8193, 1),
            (4097, 4096),
            (u32::MAX, u32::MAX),
        ] {
            assert!(row_layout(size).is_err());
        }
    }
}
