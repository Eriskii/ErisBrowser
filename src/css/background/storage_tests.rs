use super::*;

fn initial() -> BackgroundData {
    BackgroundData::default()
}

fn border_builder(pool: &mut BackgroundStorage) -> BackgroundBuilder {
    let mut value = BackgroundBuilder::default();
    let mut work = 4;
    pool.set_origins(&mut value, &[BackgroundBox::Border], &mut work)
        .unwrap();
    assert_eq!(work, 0);
    value
}

fn record_storage() -> usize {
    // Independent standard-layout construction of the chosen header/slack tariff.
    let (layout, _) = std::alloc::Layout::new::<[AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<BackgroundData>())
        .unwrap();
    layout.pad_to_align().size() + 32 + size_of::<Arc<BackgroundData>>() + 64
}

#[test]
fn default_record_has_literal_work_and_no_author_storage() {
    for allowance in [83, 84] {
        let mut pool = BackgroundStorage::default();
        let mut work = allowance;
        let result = pool.finish(BackgroundBuilder::default(), &mut work);
        assert_eq!(work, 0);
        assert_eq!(pool.charged, 0);
        assert!(pool.records.entries.is_empty());
        if allowance == 83 {
            assert!(matches!(result, Err(Error::Limit)));
        } else {
            let value = result.unwrap().unwrap();
            assert!(Arc::ptr_eq(&value, &BackgroundData::initial()));
            assert!(value.is_single_none());
            assert_eq!(value.image_count(), 1);
            assert_eq!(value.bottom_clip(), BackgroundBox::Border);
            assert!(value.layer(1).is_none());
        }
    }
}

#[test]
fn two_item_list_first_and_hit_have_literal_work() {
    let values = [[false, true], [true, false]];
    let mut pool = BackgroundStorage::default();
    let mut first = BackgroundBuilder::default();
    let mut work = 69;
    pool.set_repeats(&mut first, &values, &mut work).unwrap();
    assert_eq!(work, 0);
    let admitted = pool.charged;
    assert!(admitted > 0);
    let mut second = BackgroundBuilder::default();
    let mut work = 54;
    pool.set_repeats(&mut second, &values, &mut work).unwrap();
    assert_eq!(work, 0);
    assert_eq!(pool.charged, admitted);
    match (&first.data.repeats, &second.data.repeats) {
        (BackgroundList::Many(a), BackgroundList::Many(b)) => assert!(Arc::ptr_eq(a, b)),
        _ => panic!("two-item list"),
    }
    for allowance in [68, 0] {
        let mut pool = BackgroundStorage::default();
        let mut builder = BackgroundBuilder::default();
        let mut work = allowance;
        assert_eq!(
            pool.set_repeats(&mut builder, &values, &mut work),
            Err(Error::Limit)
        );
        assert_eq!(work, 0);
        assert!(pool.repeats.entries.is_empty());
        assert_eq!(builder.data.repeats(), [[true, true]]);
    }
}

#[test]
fn record_first_hit_and_one_short_are_independent_of_atom_tariffs() {
    let mut pool = BackgroundStorage::default();
    let first = border_builder(&mut pool);
    let mut work = 244;
    let a = pool.finish(first, &mut work).unwrap().unwrap();
    assert_eq!(work, 0);
    assert_eq!(pool.charged, record_storage());
    let second = border_builder(&mut pool);
    let mut work = 228;
    let b = pool.finish(second, &mut work).unwrap().unwrap();
    assert_eq!(work, 0);
    assert!(Arc::ptr_eq(&a, &b));
    assert_eq!(pool.charged, record_storage());
    let mut pool = BackgroundStorage::default();
    let candidate = border_builder(&mut pool);
    let mut work = 243;
    assert!(matches!(
        pool.finish(candidate, &mut work),
        Err(Error::Limit)
    ));
    assert_eq!(work, 0);
    assert!(pool.records.entries.is_empty());
}

#[test]
fn independent_lengths_cycle_only_at_used_layer_access() {
    let mut pool = BackgroundStorage::default();
    let mut value = BackgroundBuilder::default();
    let positions = [0.0, 4.0, 6.0].map(|x| BackgroundPosition {
        x: super::super::super::PositionComponent {
            from_end: false,
            offset: Length::Px(x),
        },
        y: super::super::super::PositionComponent {
            from_end: false,
            offset: Length::Px(0.0),
        },
    });
    let mut work = 100_000;
    pool.set_images(&mut value, &[None, None], &mut work)
        .unwrap();
    pool.set_positions(&mut value, &positions, &mut work)
        .unwrap();
    pool.set_clips(
        &mut value,
        &[BackgroundBox::Border, BackgroundBox::Content],
        &mut work,
    )
    .unwrap();
    let parent = pool.finish(value, &mut work).unwrap().unwrap();
    assert!(!parent.is_single_none());
    assert_eq!(parent.image_count(), 2);
    assert_eq!(parent.positions().len(), 3);
    assert_eq!(parent.bottom_clip(), BackgroundBox::Content);
    assert_eq!(parent.layer(1).unwrap().position.x.offset, Length::Px(4.0));
    assert!(parent.layer(2).is_none());
    let mut child = BackgroundBuilder::default();
    let mut inherit_work = 4;
    assert!(
        child
            .copy_property("background-position", &parent, &mut inherit_work)
            .unwrap()
    );
    assert_eq!(inherit_work, 0);
    pool.set_images(&mut child, &[None, None, None], &mut work)
        .unwrap();
    let child = pool.finish(child, &mut work).unwrap().unwrap();
    assert_eq!(child.layer(2).unwrap().position.x.offset, Length::Px(6.0));
    let mut cycle = BackgroundBuilder::default();
    pool.set_images(&mut cycle, &[None, None, None], &mut work)
        .unwrap();
    pool.set_positions(&mut cycle, &positions[..2], &mut work)
        .unwrap();
    let cycle = pool.finish(cycle, &mut work).unwrap().unwrap();
    assert_eq!(cycle.layer(2).unwrap().position.x.offset, Length::Px(0.0));
}

#[test]
fn inherited_handles_and_noncollapsed_equal_lists_share_exact_identity() {
    let mut pool = BackgroundStorage::default();
    let mut work = 100_000;
    let url = pool
        .url(
            "url(a)",
            WinningSource::Document,
            &[],
            &::url::Url::parse("https://x/").unwrap(),
            &mut work,
        )
        .unwrap()
        .unwrap();
    let mut builder = BackgroundBuilder::default();
    pool.set_images(
        &mut builder,
        &[Some(BackgroundImage::Url(url.clone())), None],
        &mut work,
    )
    .unwrap();
    pool.set_origins(
        &mut builder,
        &[BackgroundBox::Padding, BackgroundBox::Padding],
        &mut work,
    )
    .unwrap();
    let parent = pool.finish(builder, &mut work).unwrap().unwrap();
    let mut child = BackgroundBuilder::default();
    for name in ["background-image", "background-origin"] {
        child.copy_property(name, &parent, &mut work).unwrap();
    }
    pool.set_clips(&mut child, &[BackgroundBox::Content], &mut work)
        .unwrap();
    let child = pool.finish(child, &mut work).unwrap().unwrap();
    assert!(!Arc::ptr_eq(&parent, &child));
    assert_eq!(child.origins().len(), 2);
    assert!(std::ptr::eq(parent.image_slots(), child.image_slots()));
    assert!(std::ptr::eq(parent.origins(), child.origins()));
    let Some(BackgroundImage::Url(inherited)) = &child.image_slots()[0] else {
        panic!("url")
    };
    assert!(Arc::ptr_eq(&url, inherited));
}

#[test]
fn exact_record_storage_and_short_refusal_keep_debits() {
    for short in [false, true] {
        let mut pool = BackgroundStorage::default();
        let builder = border_builder(&mut pool);
        pool.charged = MAX_BYTES - record_storage() + usize::from(short);
        let result = pool.finish(builder, &mut 244);
        assert_eq!(pool.charged, MAX_BYTES);
        if short {
            assert!(matches!(result, Err(Error::Limit)));
            assert!(pool.records.entries.is_empty());
        } else {
            assert!(result.unwrap().is_some());
            let builder = border_builder(&mut pool);
            assert!(pool.finish(builder, &mut 228).unwrap().is_some());
            assert_eq!(pool.charged, MAX_BYTES);
        }
    }
}

#[test]
fn old_atom_work_is_exact_and_new_records_share_its_limit() {
    let parsed = match gradient::parse_image(
        "linear-gradient(red,blue)",
        [16.0, 16.0, 80.0, 60.0],
        &mut 100_000,
    )
    .unwrap()
    .unwrap()
    {
        gradient::ParsedImage::Gradient(value) => value,
        _ => panic!("gradient"),
    };
    let mut gradient_pool = BackgroundStorage::default();
    let mut gradient_work = 424;
    assert!(
        gradient_pool
            .gradient(&parsed, &mut gradient_work)
            .unwrap()
            .is_some()
    );
    assert_eq!(gradient_work, 0);
    let base = ::url::Url::parse("https://x/").unwrap();
    let mut pool = BackgroundStorage::default();
    let mut work = 579;
    let first = pool
        .url("url(a)", WinningSource::Document, &[], &base, &mut work)
        .unwrap()
        .unwrap();
    assert_eq!(work, 0);
    let mut work = 24;
    let hit = pool
        .url("url(a)", WinningSource::Document, &[], &base, &mut work)
        .unwrap()
        .unwrap();
    assert_eq!(work, 0);
    assert!(Arc::ptr_eq(&first, &hit));
    let used = pool.images.bytes();
    let builder = border_builder(&mut pool);
    pool.charged = MAX_BYTES - used - record_storage() + 1;
    assert!(matches!(pool.finish(builder, &mut 244), Err(Error::Limit)));
    assert_eq!(pool.charged + pool.images.bytes(), MAX_BYTES);
    let hit = pool
        .url("url(a)", WinningSource::Document, &[], &base, &mut 24)
        .unwrap()
        .unwrap();
    assert!(Arc::ptr_eq(&first, &hit));
    assert!(
        pool.url("url(b)", WinningSource::Document, &[], &base, &mut 100_000)
            .is_err()
    );
}

#[test]
fn sorted_list_growth_prepays_full_capacity_and_both_moves() {
    let mut pool = BackgroundStorage::default();
    let mut value = BackgroundBuilder::default();
    pool.set_repeats(&mut value, &[[false, true], [true, false]], &mut 69)
        .unwrap();
    let before = pool.charged;
    let mut work = 90 + 2 * size_of::<Arc<[[bool; 2]]>>();
    pool.set_repeats(&mut value, &[[false, false], [false, false]], &mut work)
        .unwrap();
    assert_eq!(work, 0);
    assert_eq!(pool.repeats.admitted_capacity, 2);
    let (layout, _) = std::alloc::Layout::new::<[AtomicUsize; 2]>()
        .extend(std::alloc::Layout::array::<[bool; 2]>(2).unwrap())
        .unwrap();
    let expected = layout.pad_to_align().size() + 32 + 2 * size_of::<Arc<[[bool; 2]]>>() + 64;
    assert_eq!(pool.charged - before, expected);
    assert_eq!(
        pool.repeats.entries[0].as_ref(),
        [[false, false], [false, false]]
    );
}

#[test]
fn invalid_cardinality_does_not_mutate_builder_or_publish_pool_entry() {
    let mut pool = BackgroundStorage::default();
    let mut value = BackgroundBuilder::default();
    for values in [vec![], vec![BackgroundBox::Content; 9]] {
        assert_eq!(
            pool.set_clips(&mut value, &values, &mut 100_000),
            Err(Error::Invalid)
        );
        assert_eq!(value.data.clips(), initial().clips());
        assert!(pool.clips.entries.is_empty());
        assert_eq!(pool.charged, 0);
    }
    pool.set_clips(&mut value, &[BackgroundBox::Content; 8], &mut 100_000)
        .unwrap();
    assert_eq!(value.data.clips().len(), 8);
}
