//! Literal child geometry and pixels; no new inline background painting oracle.
use eris::{
    dom::{Document, NodeId},
    graphics::{Canvas, Fonts, ImageStore},
    layout::LayoutResult,
};

pub const HEIGHT: u32 = 16;
pub const MUTATION_HTML: &str = r#"<!doctype html><title>Inline edge mutation</title><style>html,body{margin:0;padding:0}body{font-size:1px;line-height:1px}.dot{display:inline-block;width:4px;height:4px;margin:0;padding:0;border:0}#edge{padding-left:2px;padding-right:3px}#red{background:red}#green{background:lime}#blue{background:blue}#following{width:4px;height:2px;background:black}</style><body data-phase=ready><div id=row><span id=edge><i id=red class=dot onclick="document.getElementById('edge').style.paddingLeft='5px';document.getElementById('edge').style.paddingRight='1px';document.body.setAttribute('data-phase','changed')"></i><i id=green class=dot></i></span><i id=blue class=dot></i></div><div id=following></div></body>"#;
pub const BOUNDARIES_HTML: &str = r#"<!doctype html><title>Inline edge boundaries</title><style>html,body{margin:0;padding:0}body{font-size:1px;line-height:1px}.dot{display:inline-block;width:4px;height:4px;margin:0;padding:0;border:0}#outer{padding-left:1px;padding-right:2px}#inner{padding-left:3px;padding-right:4px}#trimrow{width:8px}#trim{padding-left:1px;padding-right:3px}#red{background:red}#green{background:lime}#blue{background:blue}#following{width:4px;height:2px;background:black}</style><body><div id=nested><span>  </span><span id=outer><span id=inner><i id=red class=dot></i></span></span><i id=blue class=dot></i></div><div id=trimrow><span id=trim><i id=green class=dot></i>  </span></div><div id=following></div></body>"#;

#[derive(Debug, PartialEq)]
pub struct Identity {
    pub red: NodeId,
    ids: Vec<NodeId>,
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
}

fn check(
    document: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    width: u32,
    rectangles: &[(&str, u32, u32, u32, u32, u32)],
    previous: Option<&Identity>,
) -> Identity {
    assert!(images.is_empty());
    let mut expected = vec![0xffffff; (width * HEIGHT) as usize];
    let mut ids = Vec::new();
    for &(name, x, y, w, h, color) in rectangles {
        assert!(x + w <= width && y + h <= HEIGHT);
        let node = document.query_selector(&format!("#{name}")).unwrap();
        let hit = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == node)
            .unwrap();
        assert_eq!(
            (
                hit.rect.x,
                hit.rect.y,
                hit.rect.width,
                hit.rect.height,
                hit.fixed
            ),
            (x as f32, y as f32, w as f32, h as f32, false),
            "{name}"
        );
        assert_eq!(layout.hit_test(x as f32 + 0.5, y as f32 + 0.5), Some(node));
        for row in y..y + h {
            expected[(row * width + x) as usize..(row * width + x + w) as usize].fill(color);
        }
        ids.push(node);
    }
    let mut canvas = Canvas::new(width, HEIGHT).unwrap();
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert_eq!(canvas.pixels, expected);
    let identity = Identity {
        red: document.query_selector("#red").unwrap(),
        ids,
        graph: document
            .nodes
            .iter()
            .map(|node| (node.parent, node.children.clone()))
            .collect(),
    };
    if let Some(previous) = previous {
        assert_eq!(
            &identity, previous,
            "style and resize must not replace DOM nodes"
        );
    }
    identity
}

pub fn mutation(
    document: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    width: u32,
    changed: bool,
    previous: Option<&Identity>,
) -> Identity {
    let body = document.query_selector("body").unwrap();
    assert_eq!(
        document.attr(body, "data-phase"),
        Some(if changed { "changed" } else { "ready" })
    );
    assert_eq!(document.title(), "Inline edge mutation");
    // First/last edges belong to the span, not every continuation line.
    // The right edge participates in fitting the green child before blue.
    let positions = match (changed, width) {
        (false, 32) => [(2, 0), (6, 0), (13, 0), (0, 4)],
        (false, 10) => [(2, 0), (0, 4), (0, 8), (0, 12)],
        (true, 32) => [(5, 0), (9, 0), (14, 0), (0, 4)],
        (true, 10) => [(5, 0), (0, 4), (5, 4), (0, 8)],
        _ => panic!("unfrozen mutation dimensions"),
    };
    let [(rx, ry), (gx, gy), (bx, by), (fx, fy)] = positions;
    check(
        document,
        layout,
        images,
        fonts,
        width,
        &[
            ("red", rx, ry, 4, 4, 0xff0000),
            ("green", gx, gy, 4, 4, 0x00ff00),
            ("blue", bx, by, 4, 4, 0x0000ff),
            ("following", fx, fy, 4, 2, 0x000000),
        ],
        previous,
    )
}

pub fn boundaries(
    document: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    previous: Option<&Identity>,
) -> Identity {
    assert_eq!(document.title(), "Inline edge boundaries");
    // Nested advance: 1 + 3 + 4 + 4 + 2 = 14. The leading whitespace-only
    // undecorated span contributes no line. Trimmed trailing spaces must leave
    // the second span's 1 + 4 + 3 = 8 advance, without an extra line.
    check(
        document,
        layout,
        images,
        fonts,
        24,
        &[
            ("red", 4, 0, 4, 4, 0xff0000),
            ("blue", 14, 0, 4, 4, 0x0000ff),
            ("green", 1, 4, 4, 4, 0x00ff00),
            ("following", 0, 8, 4, 2, 0x000000),
        ],
        previous,
    )
}
