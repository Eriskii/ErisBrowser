use super::*;
use crate::{
    Command, Frame, Rect, SourceImage, SourceMask, plan, plan_with_masks_for_profile, rect,
};

fn native(frame: Frame, commands: &[Command]) -> Plan {
    plan_with_masks_for_profile(Profile::Native, frame, commands, &[], &[], &[]).unwrap()
}
fn required(plan: &Plan) -> Preparation {
    Preparation::checked(
        plan,
        wgpu::TextureFormat::Bgra8Unorm,
        &wgpu::Limits::default(),
    )
    .unwrap()
}

#[test]
fn exact_four_buffer_clear_accounting_and_no_dummy_input() {
    let plan = native(Frame::new(4, 2, 0), &[]);
    let prepared = required(&plan);
    assert_eq!(prepared.key.raster.output, 32);
    assert_eq!(prepared.key.raster.parameters, 256);
    assert_eq!(prepared.key.raster.input, 0);
    assert_eq!(prepared.key.converted, 512);
    assert_eq!(prepared.bytes, 32 + 256 + 512 + 16);
    assert_eq!(prepared.bytes, plan.gpu_buffer_bytes());
}

#[test]
fn exact_five_buffer_image_bytes_include_source_and_coordinate_words() {
    let plan = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(4, 2, 0),
        &[Command::Image {
            rect: Rect::new(0.0, 0.0, 4.0, 2.0),
            source: 0,
        }],
        &[SourceImage {
            width: 1,
            height: 1,
            rgba: &[1, 2, 3, 255],
        }],
        &[],
        &[],
    )
    .unwrap();
    let prepared = required(&plan);
    assert_eq!(prepared.key.raster.input, 4 + (4 + 2) * 4);
    assert_eq!(prepared.key.raster.parameters, 512);
    assert_eq!(prepared.bytes, 32 + 512 + 28 + 512 + 16);
}

#[test]
fn glyph_only_input_uses_same_five_buffer_graph() {
    let plan = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(4, 2, 0),
        &[Command::Glyph {
            source: 0,
            rows: 0,
            y: 0,
            rgba: [5, 6, 7, 128],
        }],
        &[],
        &[SourceMask {
            width: 1,
            height: 1,
            coverage: &[127],
        }],
        &[&[0]],
    )
    .unwrap();
    let prepared = required(&plan);
    assert_eq!(prepared.key.raster.input, 8);
    assert_eq!(prepared.bytes, 32 + 512 + 8 + 512 + 16);
}

#[test]
fn native_viewport_targets_are_exact_without_capacity_rounding() {
    let plan = native(Frame::new(1280, 880, 0xabcdef), &[]);
    let prepared = required(&plan);
    assert_eq!(prepared.key.raster.output, 4_505_600);
    assert_eq!(prepared.key.converted, 4_505_600);
    assert_eq!(prepared.key.padded_row, 5120);
    assert_eq!(prepared.bytes, 9_011_200 + 256 + 16);
}

#[test]
fn every_exact_shape_key_component_invalidates_reuse() {
    let key = required(&native(Frame::new(4, 2, 0), &[])).key;
    let owner = Arc::new(());
    let mut changes = Vec::new();
    let mut k = key;
    k.width += 1;
    changes.push(k);
    let mut k = key;
    k.height += 1;
    changes.push(k);
    let mut k = key;
    k.format = wgpu::TextureFormat::Rgba8Unorm;
    changes.push(k);
    let mut k = key;
    k.padded_row += 256;
    changes.push(k);
    let mut k = key;
    k.converted += 256;
    changes.push(k);
    let mut k = key;
    k.raster.output += 4;
    changes.push(k);
    let mut k = key;
    k.raster.parameters += 256;
    changes.push(k);
    let mut k = key;
    k.raster.input = 4;
    changes.push(k);
    let mut k = key;
    k.raster.alignment = 16;
    changes.push(k);
    for changed in changes {
        assert!(!same_requirements(&owner, key, &owner, changed));
    }
}

#[test]
fn owner_identity_requires_same_arc_even_for_equal_values_and_keys() {
    let key = required(&native(Frame::new(4, 2, 0), &[])).key;
    let owner = Arc::new(7_u8);
    let cloned = Arc::clone(&owner);
    let separate = Arc::new(7_u8);
    assert!(same_requirements(&owner, key, &cloned, key));
    assert!(!same_requirements(&owner, key, &separate, key));
}

#[test]
fn changed_clear_and_draw_contents_reuse_shape_but_not_current_work_metadata() {
    let first = native(Frame::new(16, 16, 0), &[rect(0.0, 0.0, 1.0, 1.0, 0xff0000)]);
    let second = native(
        Frame::new(16, 16, 0xffffff),
        &[rect(0.0, 0.0, 16.0, 16.0, 0x00ff00)],
    );
    let a = required(&first);
    let b = required(&second);
    assert_eq!(a.key, b.key);
    assert_ne!(first.parameters(), second.parameters());
    assert_ne!(a.layout.invocations(), b.layout.invocations());
    assert_eq!(a.layout.invocations(), first.invocations());
    assert_eq!(b.layout.invocations(), second.invocations());
}

#[test]
fn buffer_accounting_keeps_exact_cap_and_rejects_one_over_and_overflow() {
    let mut key = required(&native(Frame::new(1, 1, 0), &[])).key;
    let cap = Profile::Native.max_gpu_buffer_bytes();
    key.raster.input = cap - key.raster.output - key.raster.parameters - key.converted - 16;
    assert_eq!(key.bytes().unwrap(), cap);
    key.raster.input += 1;
    assert!(key.bytes().is_err());
    key.raster.input = u64::MAX;
    assert!(key.bytes().is_err());
}

#[test]
fn native_only_format_alignment_and_device_storage_checks_precede_allocation() {
    let probe = plan(Frame::new(4, 2, 0), &[]).unwrap();
    assert!(
        Preparation::checked(
            &probe,
            wgpu::TextureFormat::Bgra8Unorm,
            &wgpu::Limits::default()
        )
        .is_err()
    );
    let plan = native(Frame::new(4, 2, 0), &[]);
    for format in [
        wgpu::TextureFormat::Rgba8UnormSrgb,
        wgpu::TextureFormat::R8Unorm,
    ] {
        assert!(Preparation::checked(&plan, format, &wgpu::Limits::default()).is_err());
    }
    for alignment in [0, 3, 512] {
        let limits = wgpu::Limits {
            min_uniform_buffer_offset_alignment: alignment,
            ..Default::default()
        };
        assert!(Preparation::checked(&plan, wgpu::TextureFormat::Bgra8Unorm, &limits).is_err());
    }
    let limits = wgpu::Limits {
        max_buffer_size: 511,
        ..Default::default()
    };
    assert!(Preparation::checked(&plan, wgpu::TextureFormat::Bgra8Unorm, &limits).is_err());
    let limits = wgpu::Limits {
        max_uniform_buffer_binding_size: 31,
        ..Default::default()
    };
    assert!(Preparation::checked(&plan, wgpu::TextureFormat::Bgra8Unorm, &limits).is_err());
}

#[test]
fn actual_descriptors_require_exact_sizes_and_usages_not_supersets() {
    use crate::gpu::descriptor_matches;
    let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
    assert!(descriptor_matches(512, usage, 512, usage));
    assert!(!descriptor_matches(516, usage, 512, usage));
    assert!(!descriptor_matches(
        512,
        usage | wgpu::BufferUsages::COPY_DST,
        512,
        usage
    ));
    assert!(!descriptor_matches(
        512,
        wgpu::BufferUsages::STORAGE,
        512,
        usage
    ));
}

#[test]
fn lease_cannot_reencode_or_expose_output_before_caller_retirement() {
    let mut lifecycle = Lifecycle::Fresh;
    assert!(lifecycle.compatible());
    assert!(lifecycle.output().is_err());
    assert!(lifecycle.reusable().is_err());
    lifecycle.begin().unwrap();
    assert!(!lifecycle.compatible());
    assert!(lifecycle.output().is_err());
    lifecycle.encoded();
    lifecycle.output().unwrap();
    assert!(!lifecycle.compatible());
    lifecycle.reusable().unwrap();
    assert!(lifecycle.compatible());
    assert!(lifecycle.output().is_err());
    lifecycle.begin().unwrap();
    lifecycle.encoded();
    assert!(lifecycle.begin().is_err());
    assert_eq!(lifecycle, Lifecycle::EncodingFailed);
    assert!(lifecycle.reusable().is_err());
}

#[test]
fn draw_uniform_and_input_arena_device_limits_are_independently_checked() {
    let commands = vec![rect(0.0, 0.0, 1.0, 1.0, 0); 20];
    let many_uniforms = native(Frame::new(1, 1, 0), &commands);
    let limits = wgpu::Limits {
        max_buffer_size: 1024,
        ..Default::default()
    };
    assert!(
        Preparation::checked(&many_uniforms, wgpu::TextureFormat::Bgra8Unorm, &limits).is_err()
    );
    let pixels = vec![255; 4096];
    let image = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(1, 1, 0),
        &[Command::Image {
            rect: Rect::new(0.0, 0.0, 1.0, 1.0),
            source: 0,
        }],
        &[SourceImage {
            width: 32,
            height: 32,
            rgba: &pixels,
        }],
        &[],
        &[],
    )
    .unwrap();
    let limits = wgpu::Limits {
        max_storage_buffer_binding_size: 2048,
        ..Default::default()
    };
    assert!(Preparation::checked(&image, wgpu::TextureFormat::Bgra8Unorm, &limits).is_err());
    let limits = wgpu::Limits {
        max_uniform_buffer_binding_size: 63,
        ..Default::default()
    };
    assert!(Preparation::checked(&image, wgpu::TextureFormat::Bgra8Unorm, &limits).is_err());
}
