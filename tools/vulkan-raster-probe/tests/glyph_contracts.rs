//! Exact semantic recipes from the independent frozen literal-mask inventory.
//! Oracle SHA256: 1a23a33fc1e7e45b3bb825583316cee889eab46a4111af85485b717fb009b5aa.
//! Defaults for omitted fields: 1x1 white frame, normal full clip, zero offsets,
//! no commands/images/masks/rows; a requested Glyph is source0/rows0/y0 with
//! opaque black RGBA. Base-derived recipes retain every original base field.
//! Alpha-zero variants change all Glyph alphas; empty-clip variants change only
//! the caller clip to (0,0,0,0). These are whole-plan admission tests, not pixels.
use eris_vulkan_raster_prototype::{
    Command, Frame, Plan, Rect, SourceImage, SourceMask, plan_with_masks,
};
use std::collections::BTreeSet;

#[path = "../src/glyph_fixtures/literal_inventory.rs"]
mod literal_inventory;
use literal_inventory::{LiteralFixture, OwnedImage, OwnedMask};

fn mask(width: u32, height: u32, fill: u8) -> OwnedMask {
    OwnedMask {
        width,
        height,
        coverage: vec![fill; width as usize * height as usize],
    }
}
fn glyph() -> Command {
    Command::Glyph {
        source: 0,
        rows: 0,
        y: 0,
        rgba: [0, 0, 0, 255],
    }
}
fn defaults(name: &'static str) -> LiteralFixture {
    LiteralFixture {
        name,
        frame: Frame::new(1, 1, 0xffffff),
        commands: vec![],
        images: vec![],
        masks: vec![],
        rows: vec![],
        expected: vec![],
    }
}
fn base() -> LiteralFixture {
    let fixture = literal_inventory::fixtures()
        .into_iter()
        .find(|case| case.name == "coverage-alpha-floor")
        .expect("frozen base exists");
    assert_eq!(fixture.expected.len(), 24);
    fixture
}
fn recipe(name: &'static str) -> LiteralFixture {
    let mut case = defaults(name);
    match name {
        "source-index" | "row-index" => {
            case = base();
            case.name = name;
            for command in &mut case.commands {
                let Command::Glyph { source, rows, .. } = command else {
                    panic!("frozen coverage-alpha-floor is all Glyph");
                };
                if name == "source-index" {
                    *source = 1;
                } else {
                    *rows = 1;
                }
            }
        }
        "row-height" => {
            case.masks.push(mask(1, 2, 255));
            case.rows.push(vec![0]);
            case.commands.push(glyph());
        }
        "noncanonical-empty-0" => case.masks.push(mask(0, 1, 0)),
        "noncanonical-empty-1" => case.masks.push(mask(1, 0, 0)),
        "coverage-length" => case.masks.push(OwnedMask {
            width: 2,
            height: 2,
            coverage: vec![255; 3],
        }),
        "axis-width" => case.masks.push(mask(1025, 1, 0)),
        "axis-height" => case.masks.push(mask(1, 1025, 0)),
        "mask-area" => case.masks.push(mask(513, 512, 0)),
        "aggregate-coverage" => {
            case.masks.push(mask(512, 512, 0));
            case.masks.push(mask(1, 1, 0));
        }
        "combined-sources" => {
            case.images.push(OwnedImage {
                width: 1,
                height: 1,
                rgba: vec![0; 4],
            });
            case.masks = (0..256).map(|_| mask(0, 0, 0)).collect();
        }
        "row-table-count" => case.rows = vec![vec![]; 257],
        "row-table-size" => case.rows.push(vec![0; 1025]),
        "aggregate-rows" => {
            case.rows = vec![vec![0; 1024]; 64];
            case.rows.push(vec![0]);
        }
        "command-count" => {
            case.masks.push(mask(0, 0, 0));
            case.rows.push(vec![]);
            case.commands = vec![
                Command::Glyph {
                    source: 0,
                    rows: 0,
                    y: 0,
                    rgba: [0; 4],
                };
                257
            ];
        }
        "scope-depth" => {
            case.commands = vec![Command::PushFixed; 33];
            case.commands.extend([Command::PopFixed; 33]);
        }
        "gpu-buffer-cap" => {
            case.masks.push(mask(512, 512, 255));
            case.rows.push(vec![0; 512]);
            case.commands.push(glyph());
        }
        "invocation-cap" => {
            case.frame = Frame::new(320, 240, 0xffffff);
            case.masks.push(mask(320, 240, 255));
            case.rows.push(vec![0; 240]);
            case.commands = vec![glyph(); 52];
        }
        "target-high-byte" => case.frame.clear = 0xff000000,
        _ => panic!("unknown frozen refusal recipe {name}"),
    }
    case
}
fn prepare(case: &LiteralFixture) -> Result<Plan, String> {
    let images: Vec<_> = case
        .images
        .iter()
        .map(|image| SourceImage {
            width: image.width,
            height: image.height,
            rgba: &image.rgba,
        })
        .collect();
    let masks: Vec<_> = case
        .masks
        .iter()
        .map(|mask| SourceMask {
            width: mask.width,
            height: mask.height,
            coverage: &mask.coverage,
        })
        .collect();
    let rows: Vec<_> = case.rows.iter().map(Vec::as_slice).collect();
    plan_with_masks(case.frame, &case.commands, &images, &masks, &rows)
}

const REFUSALS: [&str; 19] = [
    "source-index",
    "row-index",
    "row-height",
    "noncanonical-empty-0",
    "noncanonical-empty-1",
    "coverage-length",
    "axis-width",
    "axis-height",
    "mask-area",
    "aggregate-coverage",
    "combined-sources",
    "row-table-count",
    "row-table-size",
    "aggregate-rows",
    "command-count",
    "scope-depth",
    "gpu-buffer-cap",
    "invocation-cap",
    "target-high-byte",
];

#[test]
fn frozen_refusal_inventory_covers_nineteen_recipes_and_six_culling_variants() {
    let mut names = BTreeSet::new();
    for name in REFUSALS {
        assert!(names.insert(name.to_owned()));
        assert!(prepare(&recipe(name)).is_err(), "frozen refusal {name}");
    }
    for name in ["source-index", "row-index", "row-height"] {
        for variant in ["alpha0", "empty_clip"] {
            let label = format!("{name}/{variant}");
            assert!(names.insert(label.clone()));
            let mut case = recipe(name);
            if variant == "alpha0" {
                for command in &mut case.commands {
                    if let Command::Glyph { rgba, .. } = command {
                        rgba[3] = 0;
                    }
                }
            } else {
                case.frame.caller_clip = Rect::new(0.0, 0.0, 0.0, 0.0);
            }
            assert!(prepare(&case).is_err(), "frozen refusal {label}");
        }
    }
    assert_eq!(names.len(), 25);
}

#[test]
fn frozen_positive_inventory_covers_all_three_exact_cap_plans() {
    let mut names = BTreeSet::new();
    let mut invocation = recipe("invocation-cap");
    invocation.name = "invocation-near-cap";
    invocation.commands.pop();
    assert_eq!(invocation.commands.len(), 51);
    assert!(names.insert(invocation.name));
    let plan = prepare(&invocation).expect("invocation-near-cap accepts");
    assert_eq!(plan.invocations(), 3_993_600);
    assert_eq!(plan.gpu_buffer_bytes(), 935_872);

    let mut rows = defaults("exact-row-total");
    rows.rows = vec![vec![0; 1024]; 64];
    assert_eq!(rows.rows.iter().map(Vec::len).sum::<usize>(), 65_536);
    assert!(names.insert(rows.name));
    prepare(&rows).expect("exact-row-total accepts");

    let mut sources = defaults("exact-combined-source-count");
    sources.masks = (0..256).map(|_| mask(0, 0, 0)).collect();
    assert!(sources.rows.is_empty());
    assert!(names.insert(sources.name));
    prepare(&sources).expect("exact-combined-source-count accepts");
    assert_eq!(names.len(), 3);
}
