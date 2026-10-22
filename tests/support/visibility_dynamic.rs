//! Fixed rectangles; the layout algorithm does not derive these expectations.
pub const HTML: &str = include_str!("../fixtures/visibility-dynamic.html");
pub fn expected(width: u32, height: u32, shown: bool) -> Vec<u32> {
    let mut pixels = vec![0xffffff; (width * height) as usize];
    if shown {
        for y in 0..8 {
            pixels[(y * width) as usize..(y * width + 12) as usize].fill(0xff0000);
        }
    }
    for (top, color) in [(0, 0x0000ff), (8, 0x00ff00)] {
        for y in top..top + 4 {
            pixels[(y * width) as usize..(y * width + 4) as usize].fill(color);
        }
    }
    pixels
}

pub fn assert_pixels(actual: &[u32], expected: &[u32], width: u32) {
    assert_eq!(actual.len(), expected.len());
    if let Some(i) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "pixel ({},{}) actual={:06x} expected={:06x}",
            i % width as usize,
            i / width as usize,
            actual[i],
            expected[i]
        );
    }
}
