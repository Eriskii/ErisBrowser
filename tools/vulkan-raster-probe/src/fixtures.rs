use crate::{Command, Frame, Plan, Rect, Result, plan, rect};

pub struct Fixture {
    pub name: &'static str,
    pub frame: Frame,
    pub commands: Vec<Command>,
    pub expected: Vec<u32>,
}
impl Fixture {
    pub fn plan(&self) -> Result<Plan> {
        plan(self.frame, &self.commands)
    }
}
const R: u32 = 0xff0000;
const B: u32 = 0x0000ff;
const G: u32 = 0x00ff00;
const Y: u32 = 0xffff00;
const M: u32 = 0xff00ff;
fn literal(rows: &[&str]) -> Vec<u32> {
    rows.iter()
        .flat_map(|row| row.bytes())
        .map(|c| match c {
            b'.' => 0xffffff,
            b'R' => R,
            b'B' => B,
            b'G' => G,
            b'Y' => Y,
            b'M' => M,
            _ => panic!("unknown fixture color"),
        })
        .collect()
}
// Expectations are authored independently of planner/shader outputs. Tiny cases
// use literal final pixels; large cases use a direct band predicate or constant.
pub fn fixtures() -> Vec<Fixture> {
    let overlap = Fixture {
        name: "overlapping-order",
        frame: Frame::new(8, 6, 0xffffff),
        commands: vec![
            rect(1., 1., 5., 4., R),
            rect(3., 0., 4., 4., B),
            rect(2., 3., 2., 2., G),
            rect(0., 5., 8., 1., Y),
        ],
        expected: literal(&[
            "...BBBB.", ".RRBBBB.", ".RRBBBB.", ".RGGBBB.", ".RGGRR..", "YYYYYYYY",
        ]),
    };
    let fraction = Fixture {
        name: "fractional-clips",
        frame: Frame::new(8, 6, 0xffffff),
        commands: vec![
            Command::PushClip(Rect::new(0.2, 0.8, 5.2, 3.4)),
            rect(-0.4, -0.2, 6., 5., R),
            Command::PushClip(Rect::new(2.2, 1.2, 2., 2.)),
            rect(2., 1., 4., 3., B),
            Command::PopClip,
            rect(0.3, 3.7, 1.2, 0.3, G),
            Command::PopClip,
        ],
        expected: literal(&[
            "........", ".RRRRR..", ".RRBBR..", ".GRBBR..", ".RRRRR..", "........",
        ]),
    };
    let mut fixed_frame = Frame::new(10, 7, 0xffffff);
    fixed_frame.caller_clip = Rect::new(1., 1., 8., 5.);
    fixed_frame.document_offset = (-2., -2.);
    fixed_frame.viewport_offset = (1., 1.);
    let fixed = Fixture {
        name: "fixed-resets-and-restores",
        frame: fixed_frame,
        commands: vec![
            Command::PushClip(Rect::new(4., 4., 3., 3.)),
            rect(2., 2., 8., 8., R),
            Command::PushFixed,
            rect(0., 0., 3., 2., B),
            Command::PushClip(Rect::new(4., 2., 2., 2.)),
            rect(-10., -10., 30., 30., G),
            Command::PushFixed,
            rect(7., 4., 2., 2., Y),
            Command::PopFixed,
            Command::PopClip,
            Command::PopFixed,
            rect(5., 6., 3., 2., M),
            Command::PopClip,
            rect(2., 2., 1., 1., 0x00ffff),
        ],
        expected: literal(&[
            "..........",
            ".BBB......",
            ".BBBR.....",
            "..RRRGG...",
            "..RMMGG...",
            "........Y.",
            "..........",
        ]),
    };
    let outside = Fixture {
        name: "out-of-bounds-and-empty",
        frame: Frame::new(7, 5, 0xffffff),
        commands: vec![
            rect(-2., -1., 4., 3., R),
            rect(5.8, 3.4, 10., 10., B),
            rect(20., 20., 10., 10., G),
            rect(0., 0., 0., 5., G),
            rect(0., 0., -2., 5., G),
            Command::PushClip(Rect::new(20., 20., 10., 10.)),
            rect(0., 0., 7., 5., G),
            Command::PopClip,
        ],
        expected: literal(&["RR.....", "RR.....", ".......", ".....BB", ".....BB"]),
    };
    let bands = Fixture {
        name: "319x239-dispatch-edges",
        frame: Frame::new(319, 239, 0x102030),
        commands: vec![
            rect(0., 0., 159., 239., R),
            rect(159., 0., 160., 239., B),
            rect(0., 120., 319., 1., G),
        ],
        expected: (0..239)
            .flat_map(|y| {
                (0..319).map(move |x| {
                    if y == 120 {
                        G
                    } else if x < 159 {
                        R
                    } else {
                        B
                    }
                })
            })
            .collect(),
    };
    let repeated = Fixture {
        name: "forty-ordered-full-writes",
        frame: Frame::new(320, 240, 0),
        commands: (1..=40)
            .map(|i| rect(0., 0., 320., 240., 0x550000 + i))
            .collect(),
        expected: vec![0x550028; 320 * 240],
    };
    let clear = Fixture {
        name: "gpu-clear-only",
        frame: Frame::new(7, 3, 0x123456),
        commands: vec![],
        expected: vec![0x123456; 21],
    };
    vec![overlap, fraction, fixed, outside, bands, repeated, clear]
}
