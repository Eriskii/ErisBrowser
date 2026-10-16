//! Literal class, Promise and rendering expectations, fixed before execution.
pub const HTML: &str = include_str!("../fixtures/classes.html");
pub const INITIAL: &str = "SBDTQR";
pub const CLICKED: &str = "SBDTQRCTRL";
pub const WHITE: u32 = 0xffffff;
pub fn expected(width: u32, height: u32, red_x: u32) -> Vec<u32> {
    let mut pixels = vec![WHITE; (width * height) as usize];
    for (left, color) in [(red_x, 0xff0000), (20, 0x0000ff)] {
        for y in 2..6 {
            for x in left..left + 4 {
                pixels[(y * width + x) as usize] = color;
            }
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
