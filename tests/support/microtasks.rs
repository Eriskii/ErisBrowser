//! Literal host ordering and text-free rectangles, authored without execution.
pub const NORMAL: &str = include_str!("../fixtures/microtask-host.html");
pub const FAILURE: &str = include_str!("../fixtures/microtask-failure.html");
pub const INITIAL: &str = "ABmCDdLl";
pub const CLICKED: &str = "ABmCDdLlHNInhGOgoPpT2Z";
pub const LINK: &str = "ABmCDdLlHNInhGOgoPpT2ZQq";
pub const WHITE: u32 = 0xffffff;
pub fn expected(width: u32, height: u32, red_x: u32) -> Vec<u32> {
    let mut pixels = vec![WHITE; (width * height) as usize];
    for (x, color) in [(red_x, 0xff0000), (20, 0x0000ff)] {
        assert!(x + 4 <= width && height >= 6);
        for y in 2..6 {
            for column in x..x + 4 {
                pixels[(y * width + column) as usize] = color;
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
