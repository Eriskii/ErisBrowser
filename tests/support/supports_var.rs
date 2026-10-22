//! Frozen whole-frame oracle; expectations do not inspect the display list.
pub const HTML: &str = include_str!("../fixtures/supports-var.html");
pub const WIDTH: u32 = 8;
pub const HEIGHT: u32 = 6;
pub const EXPECTED: [u32; 48] = [
    0x00ff00, 0x00ff00, 0xff0000, 0xff0000, 0xffff00, 0xffff00, 0x00ffff, 0x00ffff, 0x00ff00,
    0x00ff00, 0xff0000, 0xff0000, 0xffff00, 0xffff00, 0x00ffff, 0x00ffff, 0xff00ff, 0xff00ff,
    0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff,
    0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff,
    0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff, 0xff00ff,
    0xff00ff, 0xff00ff, 0xff00ff,
];

pub fn assert_pixels(actual: &[u32]) {
    assert_eq!(actual.len(), EXPECTED.len());
    if let Some(i) = actual.iter().zip(EXPECTED).position(|(a, b)| *a != b) {
        panic!(
            "pixel ({},{}) actual={:06x} expected={:06x}",
            i % WIDTH as usize,
            i / WIDTH as usize,
            actual[i],
            EXPECTED[i]
        );
    }
}
