//! Root-frozen rectangles and phases; no new pointer-events type is referenced.
use eris::{
    dom::{Document, NodeId},
    layout::{HitAction, LayoutResult},
};

pub const HTML: &str = include_str!("../fixtures/pointer-events-dynamic.html");
pub const WIDTHS: [u32; 2] = [16, 20];
pub const HEIGHT: u32 = 12;
const RECTANGLES: [(u32, u32, u32, u32, u32); 3] = [
    (0, 4, 8, 6, 0xff0000),
    (0, 4, 2, 2, 0xffff00),
    (10, 0, 2, 2, 0x00ff00),
];

#[derive(Clone, Copy)]
pub enum Action {
    Initial,
    ClickToggle,
    ClickMain,
    ClickChild,
    DrainTasks,
}

pub const PHASES: [(Action, &str, &str); 6] = [
    (Action::Initial, "over", "auto"),
    (Action::ClickToggle, "under", "none"),
    (Action::ClickMain, "under", "noneU"),
    (Action::ClickChild, "under", "noneUCTB"),
    (Action::ClickToggle, "over", "auto"),
    (Action::DrainTasks, "over", "auto"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Targets {
    pub main: NodeId,
    pub child: NodeId,
    pub toggle: NodeId,
}

type Hit = (NodeId, HitAction, [u32; 4], bool);

#[derive(Debug, PartialEq, Eq)]
pub struct Geometry {
    content_height: u32,
    hits: Vec<Hit>,
}

pub fn assert_geometry(layout: &LayoutResult, baseline: &mut Option<Geometry>) {
    let current = Geometry {
        content_height: layout.content_height.to_bits(),
        hits: layout
            .hit_regions
            .iter()
            .map(|hit| {
                (
                    hit.node,
                    hit.action,
                    [
                        hit.rect.x.to_bits(),
                        hit.rect.y.to_bits(),
                        hit.rect.width.to_bits(),
                        hit.rect.height.to_bits(),
                    ],
                    hit.fixed,
                )
            })
            .collect(),
    };
    if let Some(previous) = baseline.as_ref() {
        assert_eq!(
            &current, previous,
            "hit geometry, order, action or content extent changed"
        );
    } else {
        *baseline = Some(current);
    }
}

pub fn assert_targets(document: &Document, layout: &LayoutResult, main: &str) -> Targets {
    let expected = Targets {
        main: document.query_selector(&format!("#{main}")).unwrap(),
        child: document.query_selector("#child").unwrap(),
        toggle: document.query_selector("#toggle").unwrap(),
    };
    let actual = Targets {
        main: layout
            .hit_test(4.0, 7.0)
            .expect("main point must retain a target"),
        child: layout
            .hit_test(1.0, 5.0)
            .expect("Auto child must retain a target"),
        toggle: layout
            .hit_test(11.0, 1.0)
            .expect("toggle must retain a target"),
    };
    assert_eq!(actual, expected);
    actual
}

pub fn expected(width: u32) -> Vec<u32> {
    assert!(WIDTHS.contains(&width));
    let mut pixels = vec![0xffffff; (width * HEIGHT) as usize];
    for (left, top, w, h, color) in RECTANGLES {
        for y in top..top + h {
            pixels[(y * width + left) as usize..(y * width + left + w) as usize].fill(color);
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
