use super::*;

fn native(frame: Frame, commands: &[Command]) -> Result<Plan> {
    plan_with_masks_for_profile(Profile::Native, frame, commands, &[], &[], &[])
}

#[test]
fn native_normal_window_includes_conversion_while_old_api_refuses_it() {
    let frame = Frame::new(1180, 880, 0x123456);
    let commands = [rect(4., 5., 100., 80., 0xff0000)];
    assert_eq!(plan(frame, &commands).unwrap_err(), "viewport budget");
    assert!(scope::CoordinateState::new(frame).is_err());
    let p = native(frame, &commands).unwrap();
    assert_eq!(p.profile(), Profile::Native);
    assert_eq!(p.draws().len(), 2);
    assert_eq!(p.conversion_invocations(), 1_041_920); // 148*110*64
    assert_eq!(p.raster_invocations(), 1_050_240); // plus 13*10*64
    assert_eq!(p.invocations(), 2_092_160);
    assert_eq!(p.conversion_buffer_bytes(), 4_280_336); // 4864*880+16
    assert_eq!(p.gpu_buffer_bytes(), 8_434_448);
}

#[test]
fn selected_viewport_axes_are_closed_and_zero_or_one_over_refuse() {
    assert_eq!(
        (Profile::Probe.max_width(), Profile::Probe.max_height()),
        (320, 240)
    );
    assert_eq!(
        (Profile::Native.max_width(), Profile::Native.max_height()),
        (1280, 1024)
    );
    let max = native(Frame::new(1280, 1024, 0), &[]).unwrap();
    assert_eq!(max.invocations(), 2_621_440);
    for (width, height) in [(0, 1), (1, 0), (1281, 1), (1, 1025), (u32::MAX, u32::MAX)] {
        let frame = Frame::new(width, height, 0);
        assert_eq!(native(frame, &[]).unwrap_err(), "viewport budget");
        assert!(scope::CoordinateState::new_for_profile(Profile::Native, frame).is_err());
    }
    assert!(plan(Frame::new(320, 240, 0), &[]).is_ok());
    assert!(plan(Frame::new(321, 240, 0), &[]).is_err());
    assert!(plan(Frame::new(320, 241, 0), &[]).is_err());
}

#[test]
fn conversion_is_padded_and_included_at_the_existing_work_boundary() {
    assert_eq!(Profile::Native.conversion_invocations(9, 9).unwrap(), 256);
    assert_eq!(Profile::Probe.conversion_invocations(9, 9).unwrap(), 0);
    let frame = Frame::new(1000, 1000, 0);
    let full = rect(0., 0., 1000., 1000., 1);
    let exact = native(frame, &[full, full]).unwrap();
    assert_eq!(exact.raster_invocations(), 3_000_000);
    assert_eq!(exact.conversion_invocations(), 1_000_000);
    assert_eq!(exact.invocations(), MAX_INVOCATIONS);
    assert_eq!(
        native(frame, &[full, full, rect(0., 0., 1., 1., 1)]).unwrap_err(),
        "GPU invocation budget"
    );
    // Seed arithmetic boundaries independently of ordinary scene reachability.
    assert_eq!(
        profile::add_work(MAX_INVOCATIONS, 0).unwrap(),
        MAX_INVOCATIONS
    );
    assert_eq!(
        profile::add_work(MAX_INVOCATIONS, 1).unwrap_err(),
        "GPU invocation budget"
    );
    assert_eq!(
        profile::add_work(u64::MAX, 1).unwrap_err(),
        "invocation overflow"
    );
    assert_eq!(
        profile::padded_invocations(u32::MAX, u32::MAX).unwrap_err(),
        "invocation overflow"
    );
}

#[test]
fn conversion_storage_alignment_and_absolute_byte_caps_are_explicit() {
    assert_eq!(Profile::Native.conversion_buffer_bytes(64, 2).unwrap(), 528);
    assert_eq!(
        Profile::Native.conversion_buffer_bytes(65, 2).unwrap(),
        1040
    );
    assert_eq!(Profile::Probe.conversion_buffer_bytes(64, 2).unwrap(), 0);
    for (profile, cap) in [(Profile::Probe, 1_048_576), (Profile::Native, 16_777_216)] {
        assert_eq!(profile.max_gpu_buffer_bytes(), cap);
        // Absolute policy helper boundaries; structural/work caps may make
        // these totals unreachable through a public scene.
        assert!(profile.validate_buffer_bytes(cap).is_ok());
        assert_eq!(
            profile.validate_buffer_bytes(cap + 1).unwrap_err(),
            "GPU buffer budget"
        );
    }
    assert_eq!(
        profile::padded_conversion_bytes(u32::MAX, u32::MAX).unwrap_err(),
        "GPU buffer overflow"
    );
}

#[test]
fn small_probe_payload_is_unchanged_when_only_profile_reservations_differ() {
    let frame = Frame::new(3, 1, 0);
    let image = SourceImage {
        width: 2,
        height: 1,
        rgba: &[1, 2, 3, 4, 5, 6, 7, 8],
    };
    let commands = [Command::Image {
        rect: Rect::new(0., 0., 3., 1.),
        source: 0,
    }];
    let probe = plan_with_images(frame, &commands, &[image]).unwrap();
    let native =
        plan_with_masks_for_profile(Profile::Native, frame, &commands, &[image], &[], &[]).unwrap();
    assert_eq!(probe.profile(), Profile::Probe);
    assert_eq!(probe.draws(), native.draws());
    assert_eq!(probe.parameters(), native.parameters());
    assert_eq!(probe.input_bytes(), native.input_bytes());
    assert_eq!(probe.gpu_buffer_bytes(), 560);
    assert_eq!(native.gpu_buffer_bytes(), 820);
    assert_eq!(probe.invocations(), 128);
    assert_eq!(native.raster_invocations(), probe.invocations());
    assert_eq!(native.invocations(), 192);
    assert_eq!(probe.conversion_invocations(), 0);
    assert_eq!(probe.conversion_buffer_bytes(), 0);
}

#[test]
fn native_lut_bound_expands_only_with_axes_and_stays_byte_charged() {
    assert_eq!(Profile::Probe.max_lut_entries(), 143_360);
    assert_eq!(Profile::Native.max_lut_entries(), 589_824);
    let source = SourceImage {
        width: 1,
        height: 1,
        rgba: &[10, 20, 30, 255],
    };
    let commands = vec![
        Command::Image {
            rect: Rect::new(0., 0., 1280., 1.),
            source: 0
        };
        120
    ];
    let p = plan_with_masks_for_profile(
        Profile::Native,
        Frame::new(1280, 8, 0),
        &commands,
        &[source],
        &[],
        &[],
    )
    .unwrap();
    assert!(p.input_bytes().len() / 4 - 1 > MAX_LUT_ENTRIES);
    assert_eq!(p.input_bytes().len(), 614_884); // one pixel +120*(1280+1) words
    assert_eq!(p.gpu_buffer_bytes(), 727_796);
    assert_eq!(p.invocations(), 1_249_280);
}

#[test]
fn native_keeps_command_source_mask_row_and_scope_limits() {
    let frame = Frame::new(1180, 880, 0);
    assert_eq!(
        native(frame, &[rect(0., 0., 0., 0., 0); 257]).unwrap_err(),
        "command budget"
    );
    assert_eq!(
        native(frame, &[Command::PushFixed; 33]).unwrap_err(),
        "scope budget"
    );
    let sources = [SourceImage {
        width: 1,
        height: 1,
        rgba: &[0, 0, 0, 0],
    }; 257];
    assert_eq!(
        plan_with_masks_for_profile(Profile::Native, frame, &[], &sources, &[], &[]).unwrap_err(),
        "source entry budget"
    );
    let rgba = vec![0; MAX_SOURCE_RGBA_BYTES + 4];
    let oversized = [SourceImage {
        width: (MAX_SOURCE_RGBA_BYTES / 4 + 1) as u32,
        height: 1,
        rgba: &rgba,
    }];
    assert_eq!(
        plan_with_masks_for_profile(Profile::Native, frame, &[], &oversized, &[], &[]).unwrap_err(),
        "source byte budget"
    );
    let coverage = [0; 1025];
    let masks = [SourceMask {
        width: 1025,
        height: 1,
        coverage: &coverage,
    }];
    assert_eq!(
        plan_with_masks_for_profile(Profile::Native, frame, &[], &[], &masks, &[]).unwrap_err(),
        "mask dimensions"
    );
    let row = [0; 1024];
    let rows: [&[i32]; 65] = [&row; 65];
    assert_eq!(
        plan_with_masks_for_profile(Profile::Native, frame, &[], &[], &[], &rows).unwrap_err(),
        "row entry budget"
    );
}

#[test]
fn native_coordinates_keep_fixed_escape_and_restore_the_document_clip() {
    let mut frame = Frame::new(1180, 880, 0);
    frame.caller_clip = Rect::new(10., 20., 1000., 800.);
    frame.document_offset = (3., 4.);
    frame.viewport_offset = (-2., -3.);
    let mut state = scope::CoordinateState::new_for_profile(Profile::Native, frame).unwrap();
    state
        .apply(&Command::PushClip(Rect::new(0., 0., 0., 0.)))
        .unwrap();
    let empty = state.clip();
    state.apply(&Command::PushFixed).unwrap();
    assert_eq!(state.clip(), frame.caller_clip);
    assert_eq!(state.offset(), frame.viewport_offset);
    state.apply(&Command::PopFixed).unwrap();
    assert_eq!(state.clip(), empty);
    assert_eq!(state.offset(), frame.document_offset);
    state.apply(&Command::PopClip).unwrap();
    assert_eq!(state.clip(), frame.caller_clip);
    state.finish().unwrap();
}
