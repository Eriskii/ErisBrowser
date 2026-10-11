//! Independently frozen title/rectangle expectations shared by two integrations.
pub const HTML: &str = include_str!("../fixtures/cssom-var-getter.html");
pub const WHITE: u32 = 0xffffff;
type Rectangle = (u32, u32, u32, u32, u32);
pub struct State {
    pub title: &'static str,
    pub rectangles: [Rectangle; 2],
}
pub const STATES: [State; 3] = [
    State {
        title: "initial|||important|var(--m),var(--m),var(--m),var(--m)|important,important,important,important|var(--m)|important||important|2px|2px",
        rectangles: [(4, 4, 6, 4, 0xff0000), (22, 4, 6, 4, 0x0000ff)],
    },
    State {
        title: "explicit||||,,,|,,,|var(--m)|important||important|2px|2px",
        rectangles: [(2, 2, 6, 4, 0xff0000), (22, 4, 6, 4, 0x0000ff)],
    },
    State {
        title: "pending|var(--m)|||,,,|,,,|||||2px|2px",
        rectangles: [(2, 2, 6, 4, 0xff0000), (20, 2, 6, 4, 0x0000ff)],
    },
];
pub fn expected(width: u32, height: u32, state: &State) -> Vec<u32> {
    let mut pixels = vec![WHITE; (width * height) as usize];
    for (x, y, w, h, color) in state.rectangles {
        assert!(x + w <= width && y + h <= height);
        for row in y..y + h {
            for column in x..x + w {
                pixels[(row * width + column) as usize] = color;
            }
        }
    }
    pixels
}
pub fn assert_pixels(actual: &[u32], expected: &[u32], width: u32, stage: usize) {
    assert_eq!(actual.len(), expected.len());
    if let Some(i) = actual.iter().zip(expected).position(|(a, b)| a != b) {
        panic!(
            "stage={stage} pixel ({},{}) actual={:06x} expected={:06x}",
            i % width as usize,
            i / width as usize,
            actual[i],
            expected[i]
        );
    }
}
