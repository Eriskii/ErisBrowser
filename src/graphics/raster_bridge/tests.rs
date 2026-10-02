use super::*;
use crate::graphics::{Color, Rect as BrowserRect};

fn frame() -> Frame {
    Frame::new(4, 3, 0xffffff)
}
fn browser_rect(x: f32, y: f32, width: f32, height: f32) -> BrowserRect {
    BrowserRect {
        x,
        y,
        width,
        height,
    }
}
fn fill(x: f32, y: f32, width: f32, height: f32, color: Color) -> DrawCommand {
    DrawCommand::Rect {
        rect: browser_rect(x, y, width, height),
        color,
        radius: 0.0,
    }
}
fn image(key: &str) -> DrawCommand {
    DrawCommand::Image {
        rect: browser_rect(0.0, 0.0, 1.0, 1.0),
        key: key.into(),
    }
}
fn raster(rgba: [u8; 4]) -> Arc<RasterImage> {
    Arc::new(RasterImage {
        width: 1,
        height: 1,
        rgba: rgba.to_vec(),
    })
}
fn expect_reason(
    commands: &[DrawCommand],
    images: &ImageStore,
    frame: Frame,
    kind: FallbackKind,
    index: Option<usize>,
) {
    let reason = plan_display_list(commands, images, frame).unwrap_err();
    assert_eq!(
        reason,
        FallbackReason {
            kind,
            command_index: index
        }
    );
}
fn parameter(plan: &Plan, draw: usize, field: usize) -> u32 {
    let offset = draw * eris_raster_core::PARAM_STRIDE + field * 4;
    u32::from_le_bytes(plan.parameters()[offset..offset + 4].try_into().unwrap())
}
fn hidden() -> DrawCommand {
    fill(0.0, 0.0, 0.0, 0.0, Color::TRANSPARENT)
}

#[test]
fn browser_bridge_rejects_unsupported_hidden_content_at_original_indices() {
    let mut commands = vec![
        fill(0.0, 0.0, 2.0, 2.0, Color::rgb(255, 0, 0)),
        DrawCommand::PushClip {
            rect: browser_rect(0.0, 0.0, 0.0, 0.0),
        },
        DrawCommand::Text {
            x: 0.0,
            y: 0.0,
            text: "hidden".into(),
            size: 12.0,
            color: Color::TRANSPARENT,
            bold: false,
            italic: false,
            monospace: false,
        },
        DrawCommand::PopClip,
        fill(3.0, 2.0, 1.0, 1.0, Color::rgb(0, 0, 255)),
    ];
    let text = format!("{commands:?}");
    let reason = plan_display_list(&commands, &ImageStore::new(), frame()).unwrap_err();
    assert_eq!(reason.category(), "unsupported-text");
    assert_eq!(reason.command_index, Some(2));
    assert_eq!(format!("{commands:?}"), text);
    commands[2] = DrawCommand::Rect {
        rect: browser_rect(0.0, 0.0, 0.0, 0.0),
        color: Color::TRANSPARENT,
        radius: 3.0,
    };
    let reason = plan_display_list(&commands, &ImageStore::new(), frame()).unwrap_err();
    assert_eq!(reason.category(), "unsupported-rounded-rectangle");
    assert_eq!(reason.command_index, Some(2));
    for command in [
        DrawCommand::PushOpacity { opacity: 1.0 },
        DrawCommand::PushOpacity { opacity: 0.0 },
        DrawCommand::PopOpacity,
    ] {
        commands[2] = command;
        let reason = plan_display_list(&commands, &ImageStore::new(), frame()).unwrap_err();
        assert_eq!(reason.category(), "unsupported-opacity");
        assert_eq!(reason.command_index, Some(2));
    }
    // A preceding omitted image must not renumber a later refusal.
    commands.insert(0, image("absent"));
    expect_reason(
        &commands,
        &ImageStore::new(),
        frame(),
        FallbackKind::UnsupportedOpacity,
        Some(3),
    );
}

#[test]
fn browser_bridge_validates_original_scalars_before_alpha_or_missing_elision() {
    let empty = ImageStore::new();
    for bad in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        MAX_COORDINATE + 1.0,
    ] {
        for rect in [
            browser_rect(bad, 0.0, 0.0, 0.0),
            browser_rect(0.0, bad, 0.0, 0.0),
            browser_rect(0.0, 0.0, bad, 0.0),
            browser_rect(0.0, 0.0, 0.0, bad),
        ] {
            let commands = [DrawCommand::Rect {
                rect,
                color: Color::TRANSPARENT,
                radius: 0.0,
            }];
            expect_reason(
                &commands,
                &empty,
                frame(),
                FallbackKind::InvalidGeometry,
                Some(0),
            );
            expect_reason(
                &[DrawCommand::Image {
                    rect,
                    key: "absent".into(),
                }],
                &empty,
                frame(),
                FallbackKind::InvalidGeometry,
                Some(0),
            );
            expect_reason(
                &[DrawCommand::PushClip { rect }],
                &empty,
                frame(),
                FallbackKind::InvalidGeometry,
                Some(0),
            );
        }
        expect_reason(
            &[DrawCommand::Rect {
                rect: browser_rect(0.0, 0.0, 0.0, 0.0),
                color: Color::TRANSPARENT,
                radius: bad,
            }],
            &empty,
            frame(),
            FallbackKind::InvalidGeometry,
            Some(0),
        );
    }
    expect_reason(
        &[DrawCommand::Rect {
            rect: browser_rect(0.0, 0.0, 0.0, 0.0),
            color: Color::TRANSPARENT,
            radius: -1.0,
        }],
        &empty,
        frame(),
        FallbackKind::UnsupportedRoundedRect,
        Some(0),
    );
    // Preserve allowed negative extents and signed-zero radius as inert shapes.
    let commands = [DrawCommand::Rect {
        rect: browser_rect(0.0, 0.0, -1.0, -1.0),
        color: Color::WHITE,
        radius: -0.0,
    }];
    assert_eq!(
        plan_display_list(&commands, &empty, frame())
            .unwrap()
            .plan()
            .draws()
            .len(),
        1
    );
}

#[test]
fn browser_bridge_frame_caps_and_unconditional_clear_remain_exact() {
    let empty = ImageStore::new();
    for (width, height) in [(0, 1), (1, 0), (MAX_WIDTH + 1, 1), (1, MAX_HEIGHT + 1)] {
        expect_reason(
            &[],
            &empty,
            Frame::new(width, height, 0),
            FallbackKind::ViewportLimit,
            None,
        );
    }
    let mut f = Frame::new(MAX_WIDTH, MAX_HEIGHT, 0x204060);
    f.caller_clip = Rect::new(1.0, 1.0, 0.0, 0.0);
    let accepted = plan_display_list(&[], &empty, f).unwrap();
    assert_eq!(accepted.plan().draws().len(), 1);
    assert_eq!(
        accepted.plan().draws()[0].bounds(),
        (0, 0, MAX_WIDTH, MAX_HEIGHT)
    );
    assert_eq!(parameter(accepted.plan(), 0, 6), 0xff204060);
    f.clear = 0x01000000;
    expect_reason(&[], &empty, f, FallbackKind::InvalidGeometry, None);
    f.clear = 0;
    f.document_offset.0 = f32::NAN;
    expect_reason(&[], &empty, f, FallbackKind::InvalidGeometry, None);
    f.document_offset.0 = 0.0;
    f.viewport_offset.1 = MAX_COORDINATE + 1.0;
    expect_reason(&[], &empty, f, FallbackKind::InvalidGeometry, None);
    f.viewport_offset.1 = 0.0;
    f.caller_clip.width = f32::INFINITY;
    expect_reason(&[], &empty, f, FallbackKind::InvalidGeometry, None);
}

#[test]
fn browser_bridge_original_command_and_cpu_work_caps_precede_lowering() {
    let missing = vec![image("missing"); MAX_COMMANDS];
    let accepted = plan_display_list(&missing, &ImageStore::new(), Frame::new(1, 1, 0)).unwrap();
    assert_eq!(accepted.stats().original_commands, 256);
    assert_eq!(accepted.stats().lowered_commands, 0);
    assert_eq!(accepted.stats().missing_images, 256);
    let mut too_many = missing;
    too_many.push(hidden());
    let reason = plan_display_list(&too_many, &ImageStore::new(), Frame::new(1, 1, 0)).unwrap_err();
    assert_eq!(reason.category(), "command-budget");
    let f = Frame::new(250, 200, 0);
    // 20 * 50,000 is exactly the unchanged minimum 1,000,000 pixel budget.
    assert!(plan_display_list(&vec![hidden(); 20], &ImageStore::new(), f).is_ok());
    expect_reason(
        &vec![hidden(); 21],
        &ImageStore::new(),
        f,
        FallbackKind::CpuPaintBudget,
        None,
    );
    assert!(cpu_work(Frame::new(320, 240, 0), 16).is_ok());
    assert_eq!(
        cpu_work(Frame::new(320, 240, 0), 17).unwrap_err().kind,
        FallbackKind::CpuPaintBudget
    );
    assert_eq!(
        cpu_work(Frame::new(1, 1, 0), usize::MAX).unwrap_err().kind,
        FallbackKind::CpuPaintBudget
    );
}

#[test]
fn browser_bridge_combined_typed_scopes_check_boundaries_and_mismatches() {
    let mut commands = Vec::new();
    for n in 0..MAX_SCOPES {
        commands.push(if n % 2 == 0 {
            DrawCommand::PushFixed
        } else {
            DrawCommand::PushClip {
                rect: browser_rect(0.0, 0.0, 1.0, 1.0),
            }
        });
    }
    for n in (0..MAX_SCOPES).rev() {
        commands.push(if n % 2 == 0 {
            DrawCommand::PopFixed
        } else {
            DrawCommand::PopClip
        });
    }
    assert!(plan_display_list(&commands, &ImageStore::new(), Frame::new(1, 1, 0)).is_ok());
    commands.insert(MAX_SCOPES, DrawCommand::PushFixed);
    commands.insert(MAX_SCOPES + 1, DrawCommand::PopFixed);
    let reason = plan_display_list(&commands, &ImageStore::new(), Frame::new(1, 1, 0)).unwrap_err();
    assert_eq!(reason.category(), "scope-budget");
    assert_eq!(reason.command_index, Some(32));
    for commands in [vec![DrawCommand::PopClip], vec![DrawCommand::PopFixed]] {
        expect_reason(
            &commands,
            &ImageStore::new(),
            frame(),
            FallbackKind::InvalidScope,
            Some(0),
        );
    }
    expect_reason(
        &[DrawCommand::PushFixed, DrawCommand::PopClip],
        &ImageStore::new(),
        frame(),
        FallbackKind::InvalidScope,
        Some(1),
    );
    expect_reason(
        &[
            DrawCommand::PushClip {
                rect: browser_rect(0.0, 0.0, 1.0, 1.0),
            },
            DrawCommand::PopFixed,
        ],
        &ImageStore::new(),
        frame(),
        FallbackKind::InvalidScope,
        Some(1),
    );
    expect_reason(
        &[DrawCommand::PushFixed],
        &ImageStore::new(),
        frame(),
        FallbackKind::InvalidScope,
        None,
    );
}

#[test]
fn browser_bridge_map_capacity_is_bounded_before_a_sparse_iteration() {
    let mut overreserved = ImageStore::with_capacity(MAX_STORE_CAPACITY + 1);
    assert!(overreserved.capacity() > MAX_STORE_CAPACITY);
    expect_reason(
        &[],
        &overreserved,
        frame(),
        FallbackKind::ImageStoreLimit,
        None,
    );
    overreserved.insert("single".into(), raster([1, 2, 3, 255]));
    expect_reason(
        &[],
        &overreserved,
        frame(),
        FallbackKind::ImageStoreLimit,
        None,
    );
    let image = raster([4, 5, 6, 255]);
    let mut images = ImageStore::with_capacity(MAX_SOURCE_ENTRIES);
    for n in 0..MAX_SOURCE_ENTRIES {
        images.insert(format!("{n:03}"), image.clone());
    }
    assert!(images.capacity() <= MAX_STORE_CAPACITY);
    let accepted = plan_display_list(&[], &images, frame()).unwrap();
    assert_eq!(accepted.stats().image_store_entries, 256);
    assert_eq!(accepted.stats().unique_sources, 1);
    assert_eq!(accepted.stats().total_rgba_bytes, 4);
    images.insert("extra".into(), image);
    expect_reason(&[], &images, frame(), FallbackKind::ImageStoreLimit, None);
}

#[test]
fn browser_bridge_individual_key_limit_counts_utf8_bytes() {
    let exact = "é".repeat(MAX_KEY_BYTES / 2);
    assert_eq!(exact.len(), MAX_KEY_BYTES);
    let one_over = format!("{exact}x");
    let commands = [image(&exact)];
    assert_eq!(
        plan_display_list(&commands, &ImageStore::new(), frame())
            .unwrap()
            .stats()
            .missing_images,
        1
    );
    expect_reason(
        &[image(&one_over)],
        &ImageStore::new(),
        frame(),
        FallbackKind::KeyBudget,
        Some(0),
    );
    let mut images = ImageStore::new();
    images.insert(exact, raster([1, 2, 3, 255]));
    assert!(plan_display_list(&[], &images, frame()).is_ok());
    assert_eq!(
        plan_display_list(&commands, &images, frame())
            .unwrap()
            .stats()
            .referenced_sources,
        1
    );
    let absent = format!("{}ê", "é".repeat(MAX_KEY_BYTES / 2 - 1));
    assert_eq!(absent.len(), MAX_KEY_BYTES);
    assert_eq!(
        plan_display_list(&[image(&absent)], &images, frame())
            .unwrap()
            .stats()
            .missing_images,
        1
    );
    images.clear();
    images.insert(one_over, raster([1, 2, 3, 255]));
    expect_reason(&[], &images, frame(), FallbackKind::KeyBudget, None);
}

#[test]
fn browser_bridge_aggregate_key_limits_precede_sorting_and_lookup() {
    let long = "x".repeat(MAX_KEY_BYTES);
    let mut commands = vec![image(&long); 16];
    assert!(plan_display_list(&commands, &ImageStore::new(), Frame::new(1, 1, 0)).is_ok());
    commands.push(image("a"));
    expect_reason(
        &commands,
        &ImageStore::new(),
        Frame::new(1, 1, 0),
        FallbackKind::KeyBudget,
        Some(16),
    );
    let mut images = ImageStore::new();
    let same = raster([1, 2, 3, 255]);
    for n in 0..16 {
        images.insert(format!("{}{n:04}", "x".repeat(4092)), same.clone());
    }
    assert_eq!(
        images.keys().map(String::len).sum::<usize>(),
        MAX_STORE_KEY_BYTES
    );
    assert!(plan_display_list(&[], &images, frame()).is_ok());
    images.insert("a".into(), same);
    expect_reason(&[], &images, frame(), FallbackKind::KeyBudget, None);
}

#[test]
fn browser_bridge_private_comparison_prechecks_cover_maximum_and_overflow() {
    let work = key_work(256, 65_536, 65_536, 256).unwrap();
    assert_eq!(work.sort_bytes, 16_711_680);
    assert_eq!(work.lookup_units, 657_920);
    assert_eq!(
        key_work(0, 0, 0, 0).unwrap(),
        KeyWork {
            sort_bytes: 0,
            lookup_units: 0
        }
    );
    assert_eq!(key_work(1, 4096, 0, 0).unwrap().sort_bytes, 0);
    assert!(key_work(256, MAX_SORT_COMPARISON_BYTES / 255 + 1, 0, 0).is_err());
    assert!(key_work(256, 0, MAX_LOOKUP_WORK / 10 + 1, 0).is_err());
    assert!(key_work(usize::MAX, usize::MAX, 0, 0).is_err());
    assert!(key_work(256, 0, usize::MAX, 1).is_err());
}

#[test]
fn browser_bridge_borrowed_sort_and_binary_lookup_match_spelled_keys() {
    let a = raster([1, 0, 0, 255]);
    let b = raster([2, 0, 0, 255]);
    let c = raster([3, 0, 0, 255]);
    let mut entries = vec![("é", &c), ("ab", &b), ("", &a), ("a", &a)];
    sort_entries(&mut entries);
    assert_eq!(
        entries.iter().map(|e| e.0).collect::<Vec<_>>(),
        ["", "a", "ab", "é"]
    );
    assert!(Arc::ptr_eq(find_image(&entries, "ab").unwrap(), &b));
    assert!(Arc::ptr_eq(find_image(&entries, "").unwrap(), &a));
    for absent in ["aa", "abc", "e", "ê", "zz"] {
        assert!(find_image(&entries, absent).is_none());
    }
    assert!(find_image(&[], "a").is_none());
    assert_eq!(Arc::strong_count(&a), 1);
    assert_eq!(Arc::strong_count(&b), 1);
    assert_eq!(Arc::strong_count(&c), 1);
}

#[test]
fn browser_bridge_unused_and_hidden_bad_sources_are_refused() {
    for image in [
        RasterImage {
            width: 0,
            height: 1,
            rgba: vec![],
        },
        RasterImage {
            width: 1,
            height: 0,
            rgba: vec![],
        },
        RasterImage {
            width: 1,
            height: 1,
            rgba: vec![1, 2, 3],
        },
        RasterImage {
            width: 1,
            height: 1,
            rgba: vec![1, 2, 3, 255, 0],
        },
        RasterImage {
            width: u32::MAX,
            height: u32::MAX,
            rgba: vec![],
        },
    ] {
        let mut images = ImageStore::new();
        images.insert("bad".into(), Arc::new(image));
        expect_reason(&[], &images, frame(), FallbackKind::InvalidImage, None);
        let commands = [DrawCommand::Image {
            rect: browser_rect(0.0, 0.0, 0.0, 0.0),
            key: "bad".into(),
        }];
        expect_reason(
            &commands,
            &images,
            frame(),
            FallbackKind::InvalidImage,
            None,
        );
    }
}

#[test]
fn browser_bridge_unique_rgba_cap_counts_aliases_once_and_distinct_sources_twice() {
    let maximum = Arc::new(RasterImage {
        width: (eris_raster_core::MAX_SOURCE_RGBA_BYTES / 4) as u32,
        height: 1,
        rgba: vec![0; eris_raster_core::MAX_SOURCE_RGBA_BYTES],
    });
    let mut images = ImageStore::new();
    images.insert("a".into(), maximum.clone());
    images.insert("b".into(), maximum);
    let accepted = plan_display_list(&[], &images, frame()).unwrap();
    assert_eq!(
        accepted.stats().total_rgba_bytes,
        eris_raster_core::MAX_SOURCE_RGBA_BYTES
    );
    assert_eq!(accepted.stats().unique_sources, 1);
    assert!(accepted.plan().input_bytes().is_empty());
    images.clear();
    let half = eris_raster_core::MAX_SOURCE_RGBA_BYTES / 2;
    for key in ["a", "b"] {
        images.insert(
            key.into(),
            Arc::new(RasterImage {
                width: (half / 4) as u32,
                height: 1,
                rgba: vec![0; half],
            }),
        );
    }
    let accepted = plan_display_list(&[], &images, frame()).unwrap();
    assert_eq!(accepted.stats().unique_sources, 2);
    assert_eq!(
        accepted.stats().total_rgba_bytes,
        eris_raster_core::MAX_SOURCE_RGBA_BYTES
    );
    images.insert("c".into(), raster([0, 0, 0, 0]));
    expect_reason(&[], &images, frame(), FallbackKind::InvalidImage, None);
    images.clear();
    images.insert(
        "over".into(),
        Arc::new(RasterImage {
            width: (eris_raster_core::MAX_SOURCE_RGBA_BYTES / 4 + 1) as u32,
            height: 1,
            rgba: vec![0; eris_raster_core::MAX_SOURCE_RGBA_BYTES + 4],
        }),
    );
    expect_reason(&[], &images, frame(), FallbackKind::InvalidImage, None);
}

#[test]
fn browser_bridge_source_ids_use_first_reference_then_lexical_unused_order() {
    let red = raster([255, 0, 0, 255]);
    let green = raster([0, 255, 0, 128]);
    let blue = raster([0, 0, 255, 0]);
    let mut images = ImageStore::new();
    images.insert("z".into(), red.clone());
    images.insert("y".into(), red);
    images.insert("b".into(), green);
    images.insert("a".into(), blue);
    let bridge = plan_display_list(
        &[image("b"), image("z"), image("y"), image("absent")],
        &images,
        frame(),
    )
    .unwrap();
    assert_eq!(
        *bridge.stats(),
        BridgeStats {
            original_commands: 4,
            lowered_commands: 3,
            image_store_entries: 4,
            unique_sources: 3,
            referenced_sources: 2,
            total_rgba_bytes: 12,
            referenced_rgba_bytes: 8,
            missing_images: 1
        }
    );
    // Numeric bases express first-reference ordering; unused blue still packs
    // once and keeps its hidden RGB despite alpha zero.
    assert_eq!(parameter(bridge.plan(), 1, 8), 0);
    assert_eq!(parameter(bridge.plan(), 2, 8), 1);
    assert_eq!(parameter(bridge.plan(), 3, 8), 1);
    assert_eq!(
        &bridge.plan().input_bytes()[..12],
        &[0, 255, 0, 128, 0, 0, 255, 255, 255, 0, 0, 0]
    );
}

#[test]
fn browser_bridge_randomized_map_insertion_does_not_change_plan_or_counts() {
    let sources = [
        raster([1, 2, 3, 255]),
        raster([4, 5, 6, 0]),
        raster([7, 8, 9, 128]),
    ];
    let commands = [
        image("key-19"),
        image("missing"),
        image("key-00"),
        image("key-07"),
    ];
    let mut expected: Option<BridgePlan> = None;
    for seed in 1..=32u32 {
        let mut order: Vec<usize> = (0..24).collect();
        let mut random = seed;
        for i in (1..order.len()).rev() {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            order.swap(i, random as usize % (i + 1));
        }
        let mut images = ImageStore::new();
        for n in order {
            images.insert(format!("key-{n:02}"), sources[n % 3].clone());
        }
        let actual = plan_display_list(&commands, &images, frame()).unwrap();
        if let Some(old) = &expected {
            assert_eq!(actual.stats(), old.stats());
            assert_eq!(actual.plan().parameters(), old.plan().parameters());
            assert_eq!(actual.plan().input_bytes(), old.plan().input_bytes());
            assert_eq!(
                actual.plan().gpu_buffer_bytes(),
                old.plan().gpu_buffer_bytes()
            );
            assert_eq!(actual.plan().invocations(), old.plan().invocations());
        } else {
            expected = Some(actual);
        }
    }
}

#[test]
fn browser_bridge_borrows_sources_during_planning_and_owns_only_finished_payload() {
    let source = raster([19, 23, 29, 0]);
    let mut images = ImageStore::new();
    images.insert("source".into(), source.clone());
    let before = Arc::strong_count(&source);
    let pointer = source.rgba.as_ptr();
    let bridge = plan_display_list(&[image("source")], &images, frame()).unwrap();
    assert_eq!(Arc::strong_count(&source), before);
    assert_eq!(source.rgba.as_ptr(), pointer);
    assert_eq!(source.rgba, [19, 23, 29, 0]);
    assert_eq!(bridge.plan().draws().len(), 2);
    assert!(bridge.plan().has_images());
    drop(images);
    drop(source);
    assert_eq!(&bridge.plan().input_bytes()[..4], &[29, 23, 19, 0]);
}

#[test]
fn browser_bridge_hidden_images_keep_reference_stats_without_arena_allocation() {
    let mut images = ImageStore::new();
    images.insert("stored".into(), raster([99, 77, 55, 0]));
    let commands = [
        DrawCommand::PushClip {
            rect: browser_rect(0.0, 0.0, 0.0, 0.0),
        },
        image("stored"),
        DrawCommand::PopClip,
        image("missing"),
    ];
    let bridge = plan_display_list(&commands, &images, frame()).unwrap();
    assert_eq!(bridge.stats().referenced_sources, 1);
    assert_eq!(bridge.stats().referenced_rgba_bytes, 4);
    assert_eq!(bridge.stats().missing_images, 1);
    assert_eq!(bridge.stats().lowered_commands, 3);
    assert!(bridge.plan().input_bytes().is_empty());
    assert_eq!(bridge.plan().draws().len(), 1);
}

#[test]
fn browser_bridge_full_source_packing_can_refuse_unchanged_gpu_byte_cap() {
    let mut images = ImageStore::new();
    images.insert("tiny".into(), raster([1, 2, 3, 255]));
    images.insert(
        "unused".into(),
        Arc::new(RasterImage {
            width: (eris_raster_core::MAX_SOURCE_RGBA_BYTES / 4 - 1) as u32,
            height: 1,
            rgba: vec![0; eris_raster_core::MAX_SOURCE_RGBA_BYTES - 4],
        }),
    );
    // Metadata alone fits the source cap. A visible draw requires all source
    // words plus output/readback/parameters/LUT, exceeding the same GPU cap.
    assert!(plan_display_list(&[], &images, frame()).is_ok());
    expect_reason(
        &[image("tiny")],
        &images,
        frame(),
        FallbackKind::PlannerLimit,
        None,
    );
}

#[test]
fn browser_bridge_lines_keep_canvas_f32_order_and_original_endpoint_checks() {
    let commands = [
        DrawCommand::Line {
            x1: 4.0,
            y1: 2.0,
            x2: 1.0,
            y2: 0.0,
            width: 1.0,
            color: Color::rgb(0, 0, 255),
        },
        DrawCommand::Line {
            x1: 0.0,
            y1: 3.0,
            x2: 4.0,
            y2: 3.0,
            width: 1.0,
            color: Color::rgb(255, 0, 0),
        },
        DrawCommand::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            width: 2.0,
            color: Color::rgb(0, 255, 0),
        },
    ];
    let bridge =
        plan_display_list(&commands, &ImageStore::new(), Frame::new(5, 4, 0xffffff)).unwrap();
    assert_eq!(
        bridge
            .plan()
            .draws()
            .iter()
            .map(|d| d.bounds())
            .collect::<Vec<_>>(),
        [(0, 0, 5, 4), (1, 0, 3, 2), (0, 3, 4, 1), (0, 0, 2, 2)]
    );
    let rect = line_rect(0.01, 1.0, 0.02, 2.0, 0.0, 0).unwrap();
    assert_eq!(rect.x.to_bits(), 0.01f32.to_bits());
    assert_eq!(rect.width.to_bits(), (0.02f32 - 0.01f32).to_bits());
    assert_ne!(
        (0.02f32 + 1_000_000.0) - (0.01f32 + 1_000_000.0),
        rect.width
    );
    for (x1, y1, x2, y2, width) in [
        (f32::NAN, 0.0, 0.0, 0.0, 0.0),
        (0.0, f32::NAN, 0.0, 0.0, 0.0),
        (0.0, 0.0, f32::NAN, 0.0, 0.0),
        (0.0, 0.0, 0.0, f32::NAN, 0.0),
        (0.0, 0.0, 0.0, 0.0, f32::NAN),
        (-MAX_COORDINATE, 0.0, MAX_COORDINATE, 0.0, 0.0),
    ] {
        assert_eq!(
            line_rect(x1, y1, x2, y2, width, 7)
                .unwrap_err()
                .command_index,
            Some(7)
        );
    }
    let mut f = Frame::new(1, 1, 0);
    f.document_offset = (MAX_COORDINATE, 0.0);
    // Both original scalars fit. Do not invent a cap on the translated sum.
    assert!(
        plan_display_list(
            &[fill(MAX_COORDINATE, 0.0, 1.0, 1.0, Color::WHITE)],
            &ImageStore::new(),
            f
        )
        .is_ok()
    );
}

#[test]
fn browser_bridge_fixed_escape_restores_offsets_and_scopes_without_flattening() {
    let mut f = Frame::new(6, 4, 0xffffff);
    f.caller_clip = Rect::new(1.0, 0.0, 4.0, 4.0);
    f.document_offset = (0.0, -1.0);
    f.viewport_offset = (1.0, 1.0);
    let commands = [
        fill(1.0, 1.0, 4.0, 3.0, Color::rgb(255, 0, 0)),
        DrawCommand::PushClip {
            rect: browser_rect(2.0, 1.0, 1.0, 2.0),
        },
        fill(0.0, 0.0, 6.0, 6.0, Color::rgb(0, 255, 0)),
        DrawCommand::PushFixed,
        fill(0.0, 0.0, 2.0, 1.0, Color::rgb(0, 0, 255)),
        DrawCommand::PushClip {
            rect: browser_rect(1.0, 0.0, 1.0, 1.0),
        },
        DrawCommand::PushFixed,
        fill(2.0, 1.0, 2.0, 1.0, Color::rgb(255, 255, 0)),
        DrawCommand::PopFixed,
        fill(0.0, 0.0, 4.0, 4.0, Color::rgb(255, 0, 255)),
        DrawCommand::PopClip,
        DrawCommand::PopFixed,
        fill(2.0, 1.0, 1.0, 1.0, Color::rgb(0, 255, 255)),
        DrawCommand::PopClip,
        fill(4.0, 3.0, 1.0, 1.0, Color::BLACK),
    ];
    let bridge = plan_display_list(&commands, &ImageStore::new(), f).unwrap();
    assert_eq!(bridge.stats().lowered_commands, 15);
    assert_eq!(
        bridge
            .plan()
            .draws()
            .iter()
            .map(|d| d.bounds())
            .collect::<Vec<_>>(),
        [
            (0, 0, 6, 4),
            (1, 0, 4, 3),
            (2, 0, 1, 2),
            (1, 1, 2, 1),
            (3, 2, 2, 1),
            (2, 1, 1, 1),
            (2, 0, 1, 1),
            (4, 2, 1, 1)
        ]
    );
}

#[test]
fn browser_bridge_translucent_clip_and_source_alpha_reach_existing_planner() {
    let mut f = Frame::new(5, 3, 0xffffff);
    f.caller_clip = Rect::new(0.5, 0.5, 3.0, 1.5);
    let bridge = plan_display_list(
        &[fill(-0.25, -0.25, 5.0, 3.0, Color::rgba(255, 0, 0, 128))],
        &ImageStore::new(),
        f,
    )
    .unwrap();
    assert_eq!(bridge.plan().draws()[1].bounds(), (1, 1, 3, 1));
    assert_eq!(parameter(bridge.plan(), 1, 6), 0x80ff0000);
    let bridge = plan_display_list(
        &[fill(-0.25, -0.25, 5.0, 3.0, Color::rgba(255, 0, 0, 0))],
        &ImageStore::new(),
        f,
    )
    .unwrap();
    assert_eq!(bridge.stats().lowered_commands, 1);
    assert_eq!(bridge.plan().draws().len(), 1);
}

#[test]
fn browser_bridge_snapshot_delegation_borrows_public_fields_without_mutation() {
    let mut images = ImageStore::new();
    let source = raster([1, 2, 3, 255]);
    images.insert("one".into(), source.clone());
    let snapshot = crate::worker::Snapshot {
        generation: 81,
        processed_edit_sequence: 0,
        task_state: crate::page::TaskState::Idle,
        layout: crate::layout::LayoutResult {
            commands: vec![image("one")],
            hit_regions: Vec::new(),
            content_height: 3.0,
        },
        images,
        document: crate::dom::Document::default(),
        title: "held".into(),
        url: "about:blank".into(),
        diagnostics: vec!["retained".into()],
        load_ms: 0.0,
    };
    let count = Arc::strong_count(&source);
    let bridge = plan_snapshot(&snapshot, frame()).unwrap();
    assert_eq!(snapshot.generation, 81);
    assert_eq!(snapshot.diagnostics, ["retained"]);
    assert_eq!(snapshot.title, "held");
    assert_eq!(Arc::strong_count(&source), count);
    drop(snapshot);
    assert_eq!(&bridge.plan().input_bytes()[..4], &[3, 2, 1, 255]);
}
