//! Independent principal-box pixels; no layout or mask helper constructs the oracle.
use eris::{
    dom::{Document, NodeId},
    graphics::{Canvas, DrawCommand, Fonts, ImageStore, Rect, box_decoration::BoxPaint},
    layout::LayoutResult,
};

pub const HEIGHT: u32 = 20;
pub const PNG: &[u8] = include_bytes!("../reftests/url-background-assets/pattern.png");
const IMAGE_RGBA: [u8; 32] = [
    255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 0, 255, 255, 255, 255, 0, 255, 255, 255, 255,
    0, 255, 0, 0, 0, 255, 255, 128, 0, 255,
];
const HTML: &str = r#"<!doctype html><title>Background box selection</title><style>
html,body{margin:0;padding:0;background:white}
#box{width:50%;height:4px;margin:2px;padding:2px;border:2px solid black;background-image:@IMAGE@;background-size:100% 100%;background-repeat:no-repeat;background-origin:content-box;background-clip:content-box}
#following{width:4px;height:2px;background:black}
</style><body data-phase=ready data-mutation=ready><div id=box onclick="var s=document.getElementById('box').style;s.backgroundOrigin='border-box';s.backgroundClip='padding-box';var before=s.cssText;s.setProperty('background-clip','text');if(s.cssText===before&amp;&amp;s.backgroundOrigin==='border-box'&amp;&amp;s.backgroundClip==='padding-box'){document.body.setAttribute('data-mutation','yes')}document.body.setAttribute('data-phase','changed')"></div><div id=following></div><script>if(CSS.supports('background-origin','content-box')&&CSS.supports('(background-clip: padding-box)')){document.body.setAttribute('data-supports','yes')}</script></body>"#;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paint {
    Gradient,
    Image,
}
impl Paint {
    pub fn html(self) -> String {
        HTML.replace(
            "@IMAGE@",
            match self {
                Self::Gradient => "linear-gradient(to right, red 4px, blue 4px)",
                Self::Image => "url(pattern.png)",
            },
        )
    }
}

#[derive(Clone, Copy)]
pub struct Case {
    pub paint: Paint,
    pub width: u32,
    pub changed: bool,
}

#[derive(Debug, PartialEq)]
pub struct Identity {
    pub click: NodeId,
    following: NodeId,
    graph: Vec<(Option<NodeId>, Vec<NodeId>)>,
    image_key: Option<String>,
}

fn color(letter: u8) -> u32 {
    match letter {
        b'R' => 0xff0000,
        b'G' => 0x00ff00,
        b'B' => 0x0000ff,
        b'C' => 0x00ffff,
        b'M' => 0xff00ff,
        b'Y' => 0xffff00,
        b'K' => 0,
        b'O' => 0xff8000,
        _ => panic!("unfrozen palette entry"),
    }
}

fn expected(case: Case) -> Vec<u8> {
    let mut rgb = vec![0xffffff; (case.width * HEIGHT) as usize];
    let (outer, inner) = match case.width {
        32 => (24, 20),
        20 => (18, 14),
        _ => panic!("unfrozen viewport"),
    };
    for y in 2..14 {
        rgb[(y * case.width + 2) as usize..(y * case.width + 2 + outer) as usize].fill(0);
    }
    for y in 4..12 {
        rgb[(y * case.width + 4) as usize..(y * case.width + 4 + inner) as usize].fill(0xffffff);
    }
    let (x, y, top, bottom, rows) = match (case.paint, case.changed, case.width) {
        (Paint::Gradient, false, 32) => (6, 6, "RRRRBBBBBBBBBBBB", "RRRRBBBBBBBBBBBB", 2),
        (Paint::Gradient, false, 20) => (6, 6, "RRRRBBBBBB", "RRRRBBBBBB", 2),
        (Paint::Gradient, true, 32) => (4, 4, "RRBBBBBBBBBBBBBBBBBB", "RRBBBBBBBBBBBBBBBBBB", 4),
        (Paint::Gradient, true, 20) => (4, 4, "RRBBBBBBBBBBBB", "RRBBBBBBBBBBBB", 4),
        (Paint::Image, false, 32) => (6, 6, "RRRRGGGGBBBBCCCC", "MMMMYYYYKKKKOOOO", 2),
        (Paint::Image, false, 20) => (6, 6, "RRGGGBBCCC", "MMYYYKKOOO", 2),
        (Paint::Image, true, 32) => (4, 4, "RRRRGGGGGGBBBBBBCCCC", "MMMMYYYYYYKKKKKKOOOO", 4),
        (Paint::Image, true, 20) => (4, 4, "RRGGGGGBBBBCCC", "MMYYYYYKKKKOOO", 4),
        _ => panic!("unfrozen frame"),
    };
    for (half, row) in [top, bottom].into_iter().enumerate() {
        for dy in 0..rows {
            for (dx, byte) in row.bytes().enumerate() {
                rgb[((y + half * rows + dy) * case.width as usize) + x + dx] = color(byte);
            }
        }
    }
    for y in 16..18 {
        rgb[(y * case.width) as usize..(y * case.width + 4) as usize].fill(0);
    }
    // Canvas exports opaque RGB; compare every channel of its opaque RGBA frame.
    rgb.into_iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, p as u8, 255])
        .collect()
}

pub fn check(
    document: &Document,
    layout: &LayoutResult,
    images: &ImageStore,
    fonts: &Fonts,
    case: Case,
    previous: Option<&Identity>,
) -> Identity {
    let body = document.query_selector("body").unwrap();
    assert_eq!(document.title(), "Background box selection");
    assert_eq!(document.attr(body, "data-supports"), Some("yes"));
    assert_eq!(
        document.attr(body, "data-phase"),
        Some(if case.changed { "changed" } else { "ready" })
    );
    assert_eq!(
        document.attr(body, "data-mutation"),
        Some(if case.changed { "yes" } else { "ready" })
    );
    let click = document.query_selector("#box").unwrap();
    let following = document.query_selector("#following").unwrap();
    let (outer, padding, content) = match case.width {
        32 => (24.0, 20.0, 16.0),
        20 => (18.0, 14.0, 10.0),
        _ => panic!("unfrozen viewport"),
    };
    for (id, wanted) in [
        (click, (2.0, 2.0, outer, 12.0)),
        (following, (0.0, 16.0, 4.0, 2.0)),
    ] {
        let hit = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == id)
            .unwrap();
        assert_eq!(
            (hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height),
            wanted
        );
        assert!(!hit.fixed);
        assert_eq!(layout.hit_test(wanted.0 + 0.5, wanted.1 + 0.5), Some(id));
    }
    let content_rect = Rect {
        x: 6.0,
        y: 6.0,
        width: content,
        height: 4.0,
    };
    let (shape, tile) = if case.changed {
        (
            Rect {
                x: 4.0,
                y: 4.0,
                width: padding,
                height: 8.0,
            },
            Rect {
                x: 2.0,
                y: 2.0,
                width: outer,
                height: 12.0,
            },
        )
    } else {
        (content_rect, content_rect)
    };
    let decorations: Vec<_> = layout
        .commands
        .iter()
        .filter_map(|command| match command {
            DrawCommand::BoxDecoration(value)
                if matches!(
                    &value.paint,
                    BoxPaint::Image { .. } | BoxPaint::LinearGradient { .. }
                ) =>
            {
                Some(value)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        decorations.len(),
        1,
        "nondefault clip crosses the real BoxDecoration wire path"
    );
    let decoration = decorations[0];
    assert_eq!(decoration.shape.rect, shape);
    assert_eq!(decoration.shape.radii, [[0.0; 2]; 4]);
    assert!(decoration.cutout.is_none());
    let image_key = match (&decoration.paint, case.paint) {
        (
            BoxPaint::LinearGradient {
                tile: actual,
                repeat,
                stop_scale,
                ..
            },
            Paint::Gradient,
        ) => {
            assert_eq!(*actual, tile);
            assert_eq!(*repeat, [false, false]);
            assert_eq!(*stop_scale, 1.0);
            assert!(images.is_empty());
            None
        }
        (
            BoxPaint::Image {
                tile: actual,
                repeat,
                key,
            },
            Paint::Image,
        ) => {
            assert_eq!(*actual, tile);
            assert_eq!(*repeat, [false, false]);
            assert_eq!(images.len(), 1);
            let source = images.get(key).unwrap();
            assert_eq!((source.width, source.height), (4, 2));
            assert_eq!(source.rgba.as_slice(), IMAGE_RGBA);
            Some(key.clone())
        }
        _ => panic!("wrong prepared background paint"),
    };
    let mut canvas = Canvas::new(case.width, HEIGHT).unwrap();
    canvas.paint(&layout.commands, fonts, images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    let actual: Vec<u8> = canvas
        .pixels
        .iter()
        .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8, 255])
        .collect();
    assert_eq!(
        actual,
        expected(case),
        "{case_width}px changed={changed} paint={paint:?}",
        case_width = case.width,
        changed = case.changed,
        paint = case.paint
    );
    let identity = Identity {
        click,
        following,
        graph: document
            .nodes
            .iter()
            .map(|node| (node.parent, node.children.clone()))
            .collect(),
        image_key,
    };
    if let Some(previous) = previous {
        assert_eq!(
            &identity, previous,
            "resize and style edits preserve DOM identity and prepared image key"
        );
    }
    identity
}
