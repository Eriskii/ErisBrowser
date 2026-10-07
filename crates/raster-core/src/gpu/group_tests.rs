use super::*;
use crate::{Command, Rect, SourceImage, plan_with_masks_for_profile, rect};

fn group(width: f32, height: f32) -> Plan {
    plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(16, 16, 0),
        &[
            Command::PushOpacity(0.5),
            rect(0.0, 0.0, width, height, 0xff0000),
            Command::PopOpacity,
        ],
        &[],
        &[],
        &[],
    )
    .unwrap()
}

#[test]
fn native_group_scratch_is_owned_and_missing_kernels_refuse_before_allocation() {
    let plan = group(4.0, 2.0);
    assert_eq!(plan.group_scratch_bytes(), 64);
    assert_eq!(plan.draws().len(), 4); // root clear, group clear, backing, pop
    assert_eq!(plan.parameters().len(), 4 * PARAM_STRIDE);
    assert_eq!(
        BufferAccounting::for_plan(&plan, 256, false, false).unwrap_err(),
        "missing group pipelines"
    );
    let bytes = BufferAccounting::for_plan(&plan, 256, false, true).unwrap();
    assert_eq!(bytes.output_bytes, 1024);
    assert_eq!(bytes.owned_buffer_bytes, 1024 + 1024 + 64);
    assert_eq!(bytes.planned_buffer_bytes, 1024 + 1024 + 64 + 4096 + 16);
    assert_eq!(bytes.planned_buffer_bytes, plan.gpu_buffer_bytes());
    let requirements =
        RasterRequirements::for_plan(&plan, &wgpu::Limits::default(), false, true).unwrap();
    assert_eq!(requirements.scratch, 64);
    assert_eq!(requirements.input, 0);
}

#[test]
fn explicit_scratch_budget_keeps_exact_cap_one_over_and_overflow_refusals() {
    // Synthetic ledger boundary, not an assertion that a public scene reaches
    // every byte under the independent work/source/command limits.
    let cap = Profile::Native.max_gpu_buffer_bytes();
    let scratch = cap - 4 - 256 - 4 - 272;
    let exact = BufferAccounting::checked(Profile::Native, 4, 256, 4, scratch, 272, cap).unwrap();
    assert_eq!(exact.owned_buffer_bytes, cap - 272);
    assert!(
        BufferAccounting::checked(Profile::Native, 4, 256, 4, scratch + 8, 272, cap + 8).is_err()
    );
    assert!(BufferAccounting::checked(Profile::Native, 4, 256, 4, scratch, 272, cap - 8).is_err());
    assert!(BufferAccounting::checked(Profile::Native, 4, 256, 4, u64::MAX, 272, cap).is_err());
}

#[test]
fn group_scratch_and_layout_device_limits_are_checked_independently() {
    let plan = group(16.0, 16.0);
    let exact = wgpu::Limits {
        max_buffer_size: 2048,
        max_storage_buffer_binding_size: 2048,
        max_uniform_buffer_binding_size: 128,
        max_storage_buffers_per_shader_stage: 2,
        max_bindings_per_bind_group: 3,
        max_bind_groups: 1,
        max_uniform_buffers_per_shader_stage: 1,
        max_dynamic_uniform_buffers_per_pipeline_layout: 1,
        ..Default::default()
    };
    // Raster-only check: conversion has a separately admitted buffer graph.
    RasterRequirements::for_plan(&plan, &exact, false, true).unwrap();
    let mut limits = exact.clone();
    limits.max_buffer_size = 2047; // other raster buffers are1024 each
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact.clone();
    limits.max_storage_buffer_binding_size = 2047;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact.clone();
    limits.max_uniform_buffer_binding_size = 127;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact.clone();
    limits.max_storage_buffers_per_shader_stage = 1;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact.clone();
    limits.max_bindings_per_bind_group = 2;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact.clone();
    limits.max_bind_groups = 0;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact.clone();
    limits.max_uniform_buffers_per_shader_stage = 0;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact.clone();
    limits.max_dynamic_uniform_buffers_per_pipeline_layout = 0;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits = exact;
    limits.max_compute_workgroups_per_dimension = 1;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
}

#[test]
fn group_images_admit_three_storage_bindings_and_preserve_input_arena_bytes() {
    let plan = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(4, 2, 0),
        &[
            Command::PushOpacity(0.5),
            rect(0.0, 0.0, 4.0, 2.0, 0xff0000),
            Command::Image {
                rect: Rect::new(1.0, 0.0, 2.0, 2.0),
                source: 0,
            },
            Command::PopOpacity,
        ],
        &[SourceImage {
            width: 1,
            height: 1,
            rgba: &[2, 3, 4, 127],
        }],
        &[],
        &[],
    )
    .unwrap();
    let mut limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 3,
        max_bindings_per_bind_group: 4,
        max_uniform_buffer_binding_size: 128,
        ..Default::default()
    };
    let requirements = RasterRequirements::for_plan(&plan, &limits, false, true).unwrap();
    assert_eq!(requirements.input, 4 + (2 + 2) * 4);
    assert_eq!(requirements.scratch, 4 * 2 * 8);
    limits.max_storage_buffers_per_shader_stage = 2;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
    limits.max_storage_buffers_per_shader_stage = 3;
    limits.max_bindings_per_bind_group = 3;
    assert!(RasterRequirements::for_plan(&plan, &limits, false, true).is_err());
}

#[test]
fn unit_scopes_keep_old_uniform_bytes_and_no_optional_gpu_storage() {
    let frame = Frame::new(4, 2, 0x123456);
    let draw = rect(0.0, 0.0, 2.0, 1.0, 0x654321);
    let plain =
        plan_with_masks_for_profile(Profile::Native, frame, &[draw], &[], &[], &[]).unwrap();
    let unit = plan_with_masks_for_profile(
        Profile::Native,
        frame,
        &[Command::PushOpacity(1.0), draw, Command::PopOpacity],
        &[],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(unit.group_scratch_bytes(), 0);
    assert_eq!(unit.parameters(), plain.parameters());
    assert_eq!(unit.draws(), plain.draws());
    assert_eq!(unit.input_bytes(), plain.input_bytes());
    assert_eq!(unit.gpu_buffer_bytes(), plain.gpu_buffer_bytes());
    assert_eq!(
        BufferAccounting::for_plan(&plain, 256, false, false).unwrap(),
        BufferAccounting::for_plan(&unit, 256, false, true).unwrap()
    );
    for record in unit.parameters().chunks_exact(PARAM_STRIDE) {
        assert!(record[64..].iter().all(|byte| *byte == 0));
    }
}

#[test]
fn group_uniform_window_matches_clear_primitive_and_composite_abi() {
    let plan = group(4.0, 2.0);
    let words = |record: usize| -> Vec<u32> {
        plan.parameters()[record * PARAM_STRIDE..record * PARAM_STRIDE + 128]
            .chunks_exact(4)
            .map(|bytes| u32::from_le_bytes(bytes.try_into().unwrap()))
            .collect()
    };
    assert_eq!(plan.draws()[1].kind(), DrawKind::GroupClear);
    assert!(plan.draws()[1].target_is_group());
    assert_eq!(plan.draws()[2].kind(), DrawKind::Rectangle);
    assert!(plan.draws()[2].target_is_group());
    assert_eq!(plan.draws()[3].kind(), DrawKind::GroupComposite);
    assert!(!plan.draws()[3].target_is_group());
    let clear = words(1);
    assert_eq!(&clear[..8], &[0, 0, 4, 2, 16, 16, 0, 0]);
    assert_eq!(&clear[16..24], &[0, 0, 0, 4, 2, 1, 0, 0]);
    assert_eq!(&clear[24..], &[0; 8]);
    let primitive = words(2);
    assert_eq!(&primitive[16..24], &[0, 0, 0, 4, 2, 1, 0, 0]);
    assert_eq!(&primitive[24..], &[0; 8]);
    let composite = words(3);
    assert_eq!(&composite[16..24], &[0, 0, 0, 16, 16, 0, 0, 0]);
    assert_eq!(&composite[24..], &[0, 0, 0, 4, 2, 128, 0, 0]);
    for record in plan.parameters().chunks_exact(PARAM_STRIDE) {
        assert!(record[128..].iter().all(|byte| *byte == 0));
    }
}
