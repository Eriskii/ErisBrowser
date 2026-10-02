use crate::{
    Frame, MAX_INVOCATIONS, Profile, plan, plan_with_masks_for_profile,
    surface::{Admission, SurfaceLayout},
};
use wgpu::TextureFormat;

fn native(width: u32, height: u32) -> crate::Plan {
    plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(width, height, 0x123456),
        &[],
        &[],
        &[],
        &[],
    )
    .unwrap()
}

fn admitted(plan: &crate::Plan) -> Admission {
    let frame = plan.frame();
    Admission {
        profile: plan.profile(),
        width: frame.width,
        height: frame.height,
        output_bytes: u64::from(frame.width) * u64::from(frame.height) * 4,
        core_owned_bytes: plan.gpu_buffer_bytes() - plan.conversion_buffer_bytes(),
        conversion_buffer_bytes: plan.conversion_buffer_bytes(),
        planned_buffer_bytes: plan.gpu_buffer_bytes(),
        conversion_invocations: plan.conversion_invocations(),
        invocations: plan.invocations(),
    }
}

#[test]
fn surface_requires_native_reservation_and_only_two_unorm_formats() {
    let probe = plan(Frame::new(3, 2, 0), &[]).unwrap();
    assert!(SurfaceLayout::for_plan(&probe, TextureFormat::Bgra8Unorm).is_err());
    let native = native(3, 2);
    for format in [TextureFormat::Bgra8Unorm, TextureFormat::Rgba8Unorm] {
        let layout = SurfaceLayout::for_plan(&native, format).unwrap();
        assert_eq!(layout.format(), format);
    }
    for format in [
        TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Rgba8Uint,
        TextureFormat::Rgba16Float,
        TextureFormat::R8Unorm,
        TextureFormat::Depth32Float,
    ] {
        assert!(SurfaceLayout::for_plan(&native, format).is_err());
    }
}

#[test]
fn surface_rows_cross_alignment_boundaries_without_charging_padding_as_pixels() {
    // Independently tabulated: width, packed row, padded row, padded 8x8 work.
    for (width, packed, padded, work) in [
        (1, 4u32, 256, 64),
        (3, 12, 256, 64),
        (63, 252, 256, 512),
        (64, 256, 256, 512),
        (65, 260, 512, 576),
        (1180, 4720, 4864, 9472),
        (1279, 5116, 5120, 10240),
        (1280, 5120, 5120, 10240),
    ] {
        let plan = native(width, 2);
        let layout = SurfaceLayout::for_plan(&plan, TextureFormat::Bgra8Unorm).unwrap();
        assert_eq!((layout.width(), layout.height()), (width, 2));
        assert_eq!(layout.padded_bytes_per_row(), padded);
        assert_eq!(layout.storage_bytes(), u64::from(padded) * 2);
        assert_eq!(layout.owned_buffer_bytes(), u64::from(padded) * 2 + 16);
        assert_eq!(layout.conversion_invocations(), work);
        assert_eq!(layout.invocations(), work * 2); // clear plus conversion
        assert_eq!(
            layout.planned_buffer_bytes(),
            u64::from(packed) * 2 + 256 + u64::from(padded) * 2 + 16
        );
    }
}

#[test]
fn surface_native_maximum_and_normal_window_have_exact_combined_storage() {
    for (width, height, stride, output, conversion, total, work) in [
        (
            1280, 1024, 5120, 5_242_880, 5_242_896, 10_486_032, 1_310_720,
        ),
        (1180, 880, 4864, 4_153_600, 4_280_336, 8_434_192, 1_041_920),
    ] {
        let plan = native(width, height);
        let layout = SurfaceLayout::for_plan(&plan, TextureFormat::Rgba8Unorm).unwrap();
        assert_eq!(u64::from(width) * u64::from(height) * 4, output);
        assert_eq!(layout.padded_bytes_per_row(), stride);
        assert_eq!(layout.owned_buffer_bytes(), conversion);
        assert_eq!(layout.planned_buffer_bytes(), total);
        assert_eq!(layout.conversion_invocations(), work);
        assert_eq!(layout.invocations(), work * 2);
        assert_eq!(plan.conversion_buffer_bytes(), conversion);
    }
}

#[test]
fn surface_shape_and_reserved_storage_fail_before_any_device_is_needed() {
    let base = admitted(&native(3, 2));
    for (width, height) in [
        (0, 2),
        (3, 0),
        (1281, 2),
        (3, 1025),
        (u32::MAX, 2),
        (3, u32::MAX),
    ] {
        assert!(
            SurfaceLayout::checked(
                Admission {
                    width,
                    height,
                    ..base
                },
                TextureFormat::Bgra8Unorm
            )
            .is_err()
        );
    }
    let mutations = [
        Admission {
            output_bytes: 23,
            ..base
        },
        Admission {
            output_bytes: 25,
            ..base
        },
        Admission {
            core_owned_bytes: 23,
            ..base
        },
        Admission {
            core_owned_bytes: base.core_owned_bytes + 1,
            ..base
        },
        Admission {
            conversion_buffer_bytes: 512,
            ..base
        }, // missing uniform
        Admission {
            conversion_buffer_bytes: 24 + 16,
            ..base
        }, // no padding
        Admission {
            conversion_buffer_bytes: 527,
            ..base
        },
        Admission {
            conversion_buffer_bytes: 529,
            ..base
        },
        Admission {
            planned_buffer_bytes: base.planned_buffer_bytes - 1,
            ..base
        },
        Admission {
            planned_buffer_bytes: u64::MAX,
            core_owned_bytes: u64::MAX,
            ..base
        },
    ];
    for input in mutations {
        assert!(SurfaceLayout::checked(input, TextureFormat::Bgra8Unorm).is_err());
    }
}

#[test]
fn surface_total_cap_and_conversion_work_are_not_independent_allowances() {
    let mut exact = admitted(&native(3, 2));
    exact.planned_buffer_bytes = Profile::Native.max_gpu_buffer_bytes();
    exact.core_owned_bytes = exact.planned_buffer_bytes - exact.conversion_buffer_bytes;
    exact.invocations = MAX_INVOCATIONS;
    let layout = SurfaceLayout::checked(exact, TextureFormat::Rgba8Unorm).unwrap();
    assert_eq!(
        layout.planned_buffer_bytes(),
        Profile::Native.max_gpu_buffer_bytes()
    );
    assert_eq!(layout.invocations(), MAX_INVOCATIONS);
    // Internally supplied metadata exercises the cap, without constructing a
    // forged public Plan or allocating a synthetic full-cap buffer.
    for input in [
        Admission {
            planned_buffer_bytes: exact.planned_buffer_bytes + 1,
            core_owned_bytes: exact.core_owned_bytes + 1,
            ..exact
        },
        Admission {
            invocations: MAX_INVOCATIONS + 1,
            ..exact
        },
        Admission {
            invocations: 63,
            ..exact
        },
        Admission {
            conversion_invocations: 0,
            ..exact
        },
        Admission {
            conversion_invocations: 63,
            ..exact
        },
        Admission {
            conversion_invocations: 65,
            ..exact
        },
        Admission {
            profile: Profile::Probe,
            ..exact
        },
    ] {
        assert!(SurfaceLayout::checked(input, TextureFormat::Rgba8Unorm).is_err());
    }
}

#[test]
fn surface_uniform_literals_define_channel_order_stride_and_opaque_targets() {
    let plan = native(3, 2);
    let bgra = SurfaceLayout::for_plan(&plan, TextureFormat::Bgra8Unorm).unwrap();
    let rgba = SurfaceLayout::for_plan(&plan, TextureFormat::Rgba8Unorm).unwrap();
    assert_eq!(
        bgra.uniform(),
        [3, 0, 0, 0, 2, 0, 0, 0, 64, 0, 0, 0, 16, 0, 0, 0]
    );
    assert_eq!(
        rgba.uniform(),
        [3, 0, 0, 0, 2, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0]
    );
    // Test only the host-selected channel metadata against independent literal
    // byte targets. This model does NOT execute/validate WGSL; the supervised
    // end-to-end conversion test must separately verify the actual shader.
    let inputs = [
        0x00123456u32,
        0xab123456,
        0x00ff0000,
        0x0000ff00,
        0x000000ff,
        0,
        u32::MAX,
    ];
    let bgra_bytes = [
        [0x56, 0x34, 0x12, 255],
        [0x56, 0x34, 0x12, 255],
        [0, 0, 255, 255],
        [0, 255, 0, 255],
        [255, 0, 0, 255],
        [0, 0, 0, 255],
        [255, 255, 255, 255],
    ];
    let rgba_bytes = [
        [0x12, 0x34, 0x56, 255],
        [0x12, 0x34, 0x56, 255],
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
        [0, 0, 0, 255],
        [255, 255, 255, 255],
    ];
    for (layout, expected) in [(bgra, bgra_bytes), (rgba, rgba_bytes)] {
        let metadata = layout.uniform();
        let red_shift = u32::from_le_bytes(metadata[12..16].try_into().unwrap());
        for (input, literal) in inputs.into_iter().zip(expected) {
            let encoded = 0xff00_0000
                | (((input >> 16) & 255) << red_shift)
                | (input & 0x0000_ff00)
                | ((input & 255) << (16 - red_shift));
            assert_eq!(encoded.to_le_bytes(), literal);
        }
    }
}

#[test]
fn surface_device_preflight_checks_each_required_limit_before_allocation() {
    let layout = SurfaceLayout::for_plan(&native(65, 9), TextureFormat::Bgra8Unorm).unwrap();
    let mut exact = wgpu::Limits::downlevel_defaults();
    exact.max_buffer_size = layout.storage_bytes();
    exact.max_storage_buffer_binding_size = layout.storage_bytes();
    exact.max_uniform_buffer_binding_size = 16;
    exact.max_compute_workgroups_per_dimension = 9;
    exact.max_compute_workgroup_size_x = 8;
    exact.max_compute_workgroup_size_y = 8;
    exact.max_compute_invocations_per_workgroup = 64;
    layout.check_device(&exact).unwrap();
    let mut mutations = Vec::new();
    let mut changed = exact.clone();
    changed.max_buffer_size -= 1;
    mutations.push(changed);
    let mut changed = exact.clone();
    changed.max_storage_buffer_binding_size -= 1;
    mutations.push(changed);
    let mut changed = exact.clone();
    changed.max_uniform_buffer_binding_size = 15;
    mutations.push(changed);
    let mut changed = exact.clone();
    changed.max_compute_workgroups_per_dimension = 8;
    mutations.push(changed);
    let mut changed = exact.clone();
    changed.max_compute_workgroup_size_x = 7;
    mutations.push(changed);
    let mut changed = exact.clone();
    changed.max_compute_workgroup_size_y = 7;
    mutations.push(changed);
    let mut changed = exact;
    changed.max_compute_invocations_per_workgroup = 63;
    mutations.push(changed);
    for changed in mutations {
        assert!(layout.check_device(&changed).is_err());
    }
}
