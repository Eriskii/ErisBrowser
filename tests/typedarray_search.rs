//! Exercise all four TypedArray search methods through a visible page event.
use eris::{
    graphics::{Canvas, Color, DrawCommand, Fonts},
    page::Page,
};
use url::Url;

const HTML: &str = include_str!("../examples/typedarray-search.html");
const EXPECTED: [&str; 2] = [
    "Bytes: 16 126 32 48 126 64 90\nLast byte: 90\nContains marker 126: true\nFirst marker: 1\nLast marker: 4",
    "Bytes: 16 8 126 48 126 64 99\nLast byte: 99\nContains marker 126: true\nFirst marker: 2\nLast marker: 4",
];

#[test]
fn typedarray_search_page_changes_visible_results_on_click() {
    let address = Url::parse("https://example.test/packet-search").unwrap();
    let mut page = Page::from_html(address, HTML, true);
    let fonts = Fonts::new();
    let button = page.document.query_selector("#change").unwrap();
    let result = page.document.query_selector("#result").unwrap();
    let mut previous_pixels = None;
    let mut click = button;
    for (phase, expected) in EXPECTED.iter().enumerate() {
        if phase != 0 {
            assert!(page.click(click).is_none());
        }
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        assert!(
            page.runtime.console.is_empty(),
            "{:?}",
            page.runtime.console
        );
        assert_eq!(page.document.text_content(result), *expected);
        assert_eq!(page.title(), ["Packet search", "Packet changed"][phase]);
        let layout = page.layout(640.0, 480.0, &fonts);
        let region = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == button)
            .unwrap();
        click = layout
            .hit_test(region.rect.x + 1.0, region.rect.y + 1.0)
            .unwrap();
        assert_eq!(click, button);
        let output = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == result)
            .unwrap()
            .rect;
        assert!(output.width > 2.0 && output.height > 2.0);
        assert!(output.x >= 0.0 && output.y >= 0.0);
        assert!(output.x + output.width <= 640.0 && output.y + output.height <= 480.0);
        let mut lines: Vec<(f32, String)> = Vec::new();
        for command in &layout.commands {
            if let DrawCommand::Text { x, y, text, .. } = command
                && *x >= output.x
                && *x < output.x + output.width
                && *y >= output.y
                && *y < output.y + output.height
            {
                if let Some((top, line)) = lines.last_mut()
                    && *top == *y
                {
                    line.push_str(text);
                } else {
                    lines.push((*y, text.clone()));
                }
            }
        }
        assert_eq!(
            lines
                .iter()
                .map(|(_, line)| line.as_str())
                .collect::<Vec<_>>(),
            expected.split('\n').collect::<Vec<_>>()
        );
        // Restrict ink/change checks to the result. Heading ink and button focus
        // cannot satisfy these assertions if script output is never painted.
        let mut canvas = Canvas::new(640, 480).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert!(canvas.pixels.contains(&0x245bcc));
        let mut output_pixels = Vec::new();
        for y in output.y.ceil() as usize + 1..(output.y + output.height).floor() as usize - 1 {
            let left = output.x.ceil() as usize + 1;
            let right = (output.x + output.width).floor() as usize - 1;
            output_pixels.extend_from_slice(&canvas.pixels[y * 640 + left..y * 640 + right]);
        }
        assert!(output_pixels.contains(&0x172033));
        if let Some(before) = previous_pixels {
            assert_ne!(output_pixels, before);
        }
        previous_pixels = Some(output_pixels);
    }
}
