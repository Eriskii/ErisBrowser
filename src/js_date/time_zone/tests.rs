use super::*;
fn parse(text: &str) -> TimeZoneSnapshot {
    TimeZoneSnapshot::parse(
        &ZonePayload::new(ZonePayloadKind::Posix2024, text.as_bytes()).unwrap(),
        &mut ZoneBudget::default(),
    )
    .unwrap()
}
fn ms(y: i32, m: u8, d: u8, h: i64, min: i64) -> i64 {
    days_from_civil(y, m - 1, d) * 86_400_000 + h * 3_600_000 + min * 60_000
}
fn offset(zone: &TimeZoneSnapshot, t: i64) -> i32 {
    zone.offset_at_utc_ms(t, &mut 100_000).unwrap()
}
fn inverse(zone: &TimeZoneSnapshot, t: i64) -> i64 {
    zone.utc_from_local_ms(t, &mut 100_000).unwrap()
}
fn header(version: u8, count: u32, types: u32, chars: u32) -> Vec<u8> {
    let mut out = b"TZif".to_vec();
    out.push(version);
    out.extend_from_slice(&[0; 15]);
    for n in [0, 0, 0, count, types, chars] {
        out.extend_from_slice(&n.to_be_bytes());
    }
    out
}
fn tzif(transitions: &[(i64, u8)], types: &[(i32, bool, &str)], tail: &str) -> Vec<u8> {
    let mut out = header(b'3', 0, 1, 4);
    out.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    out.extend_from_slice(b"UTC\0");
    let mut names = Vec::new();
    let mut rawtypes = Vec::new();
    for &(offset, dst, name) in types {
        rawtypes.extend_from_slice(&offset.to_be_bytes());
        rawtypes.extend_from_slice(&[u8::from(dst), names.len() as u8]);
        names.extend_from_slice(name.as_bytes());
        names.push(0);
    }
    out.extend_from_slice(&header(
        b'3',
        transitions.len() as u32,
        types.len() as u32,
        names.len() as u32,
    ));
    for &(at, _) in transitions {
        out.extend_from_slice(&at.to_be_bytes());
    }
    for &(_, kind) in transitions {
        out.push(kind);
    }
    out.extend_from_slice(&rawtypes);
    out.extend_from_slice(&names);
    out.push(b'\n');
    out.extend_from_slice(tail.as_bytes());
    out.push(b'\n');
    out
}
fn parse_tzif(bytes: &[u8]) -> Result<TimeZoneSnapshot, ZoneError> {
    TimeZoneSnapshot::parse(
        &ZonePayload::new(ZonePayloadKind::Tzif, bytes).unwrap(),
        &mut ZoneBudget::default(),
    )
}
#[test]
fn timezone_northern_fold_gap_and_exact_transition_boundary() {
    let z = parse("EST5EDT,M3.2.0,M11.1.0");
    let spring = ms(2017, 3, 12, 7, 0);
    assert_eq!(offset(&z, spring - 1), -18_000);
    assert_eq!(offset(&z, spring), -14_400);
    assert_eq!(inverse(&z, ms(2017, 3, 12, 2, 30)), ms(2017, 3, 12, 7, 30));
    assert_eq!(inverse(&z, ms(2017, 11, 5, 1, 30)), ms(2017, 11, 5, 5, 30));
    assert_eq!(inverse(&z, ms(2017, 11, 5, 2, 30)), ms(2017, 11, 5, 7, 30));
}
#[test]
fn timezone_southern_half_hour_negative_dst_and_all_year() {
    let z = parse("<+1030>-10:30<+11>-11,M10.1.0,M4.1.0");
    assert_eq!(offset(&z, ms(2020, 1, 1, 0, 0)), 39_600);
    assert_eq!(offset(&z, ms(2020, 7, 1, 0, 0)), 37_800);
    let z = parse("IST-1GMT0,M10.5.0,M3.5.0/1");
    assert_eq!(offset(&z, ms(2020, 1, 1, 0, 0)), 0);
    assert_eq!(offset(&z, ms(2020, 7, 1, 0, 0)), 3600);
    let z = parse("XXX3EDT4,0/0,J365/23");
    for year in [-100, 0, 1, 2000, 2024, 275_000] {
        for month in [1, 7, 12] {
            assert_eq!(offset(&z, ms(year, month, 1, 0, 0)), -14_400);
        }
    }
}
#[test]
fn timezone_julian_ordinal_last_week_and_signed_year_crossings() {
    let julian = parse("AAA0BBB,J60/0,J61/0");
    let ordinal = parse("AAA0BBB,59/0,60/0");
    assert_eq!(offset(&julian, ms(2020, 2, 29, 12, 0)), 0);
    assert_eq!(offset(&ordinal, ms(2020, 2, 29, 12, 0)), 3600);
    let signed = parse("AAA0BBB,M1.1.0/-167,M12.5.0/167");
    for t in [
        ms(2019, 12, 27, 0, 0),
        ms(2020, 1, 1, 0, 0),
        ms(2020, 12, 31, 0, 0),
    ] {
        assert!(matches!(offset(&signed, t), 0 | 3600));
    }
    let last = parse("AAA0BBB,M3.5.0/0,M10.5.0/0");
    assert_eq!(offset(&last, ms(2021, 3, 27, 23, 0)), 0);
    assert_eq!(offset(&last, ms(2021, 3, 28, 0, 0)), 3600);
}
#[test]
fn timezone_tzif_historical_seconds_and_skipped_day() {
    let switch = ms(2011, 12, 30, 10, 0) / 1000;
    let bytes = tzif(
        &[(switch, 1)],
        &[(-36_000, false, "OLD"), (50_400, false, "NEW")],
        "NEW-14",
    );
    let z = parse_tzif(&bytes).unwrap();
    assert_eq!(offset(&z, switch * 1000 - 1), -36_000);
    assert_eq!(offset(&z, switch * 1000), 50_400);
    assert_eq!(
        inverse(&z, ms(2011, 12, 30, 12, 0)),
        ms(2011, 12, 30, 22, 0)
    );
    let z = parse_tzif(&tzif(
        &[(0, 1)],
        &[(20_476, false, "LMT"), (20_700, false, "NPT")],
        "NPT-5:45",
    ))
    .unwrap();
    assert_eq!(offset(&z, -1), 20_476);
    assert_eq!(offset(&z, 0), 20_700);
}
#[test]
fn timezone_inverse_matches_independent_interval_oracle_for_irregular_jumps() {
    // Three overlapping folds and successive gaps, with second-precision offsets.
    let transitions = [(-100, 1), (-80, 2), (-60, 3), (0, 4), (20, 5), (40, 0)];
    let offsets = [0, 50, 20, -30, 60, -10];
    let types = [
        (0, false, "UTC"),
        (50, false, "AAA"),
        (20, false, "BBB"),
        (-30, false, "CCC"),
        (60, false, "DDD"),
        (-10, false, "EEE"),
    ];
    let z = parse_tzif(&tzif(&transitions, &types, "UTC0")).unwrap();
    let actual = |u: i64| -> i64 {
        let mut o = 0;
        for &(at, k) in &transitions {
            if u >= at * 1000 {
                o = offsets[k as usize];
            }
        }
        o
    };
    let candidates = |l: i64| -> Vec<i64> {
        let mut v = Vec::new();
        for o in offsets {
            let u = l - o * 1000;
            if actual(u) == o {
                v.push(u);
            }
        }
        v.sort_unstable();
        v.dedup();
        v
    };
    for local in (-150_000..=150_000).step_by(137) {
        let c = candidates(local);
        let expected = if let Some(&first) = c.first() {
            first
        } else {
            let previous = (-200_000..local)
                .rev()
                .find(|&l| !candidates(l).is_empty())
                .unwrap();
            let last = *candidates(previous).last().unwrap();
            local - actual(last) * 1000
        };
        assert_eq!(inverse(&z, local), expected, "local {local}");
    }
}
#[test]
fn timezone_rejects_undefined_leap_incomplete_and_inconsistent_sources() {
    let valid = tzif(
        &[(0, 1)],
        &[(0, false, "UTC"), (3600, false, "ONE")],
        "ONE-1",
    );
    assert!(parse_tzif(&valid).is_ok());
    let mut leap = valid.clone();
    leap[28..32].copy_from_slice(&1u32.to_be_bytes());
    assert_eq!(parse_tzif(&leap).unwrap_err(), ZoneError::LeapAware);
    assert_eq!(
        parse_tzif(&tzif(
            &[(0, 1)],
            &[(0, false, "-00"), (3600, false, "ONE")],
            "ONE-1"
        ))
        .unwrap_err(),
        ZoneError::IncompleteCoverage
    );
    assert_eq!(
        parse_tzif(&tzif(
            &[(0, 1)],
            &[(0, false, "UTC"), (3600, false, "ONE")],
            ""
        ))
        .unwrap_err(),
        ZoneError::IncompleteCoverage
    );
    assert_eq!(
        parse_tzif(&tzif(
            &[(0, 1)],
            &[(0, false, "UTC"), (3600, false, "ONE")],
            "UTC0"
        ))
        .unwrap_err(),
        ZoneError::Malformed
    );
    assert!(parse_tzif(&tzif(&[(0, 0), (0, 0)], &[(0, false, "UTC")], "UTC0")).is_err());
    assert!(parse_tzif(&tzif(&[(0, 1)], &[(0, false, "UTC")], "UTC0")).is_err());
    for cut in 0..valid.len() {
        assert!(parse_tzif(&valid[..cut]).is_err(), "accepted prefix {cut}");
    }
    let mut trailing = valid;
    trailing.push(0);
    assert!(parse_tzif(&trailing).is_err());
}
#[test]
fn timezone_version_rules_zero_transition_fixed_and_unused_unknown() {
    let mut bytes = header(0, 0, 1, 4);
    bytes.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(b"UTC\0");
    assert_eq!(offset(&parse_tzif(&bytes).unwrap(), 0), 0);
    let mut signed = tzif(&[], &[(0, false, "-00")], "AAA0BBB,M3.5.0/-2,M10.5.0");
    assert!(parse_tzif(&signed).is_ok());
    signed[4] = b'2';
    signed[58] = b'2';
    assert!(parse_tzif(&signed).is_err());
    let z = parse_tzif(&tzif(
        &[(0, 0)],
        &[(0, false, "UTC"), (0, false, "-00")],
        "UTC0",
    ))
    .unwrap();
    assert_eq!(offset(&z, 0), 0);
}
#[test]
fn timezone_limits_are_precharged_and_large_years_take_bounded_work() {
    let payload = ZonePayload::new(ZonePayloadKind::Posix2024, b"EST5EDT,M3.2.0,M11.1.0").unwrap();
    assert_eq!(
        TimeZoneSnapshot::parse(
            &payload,
            &mut ZoneBudget {
                work_left: 0,
                allocation_left: usize::MAX
            }
        )
        .unwrap_err(),
        ZoneError::WorkLimit
    );
    assert_eq!(
        TimeZoneSnapshot::parse(
            &payload,
            &mut ZoneBudget {
                work_left: usize::MAX,
                allocation_left: 0
            }
        )
        .unwrap_err(),
        ZoneError::Allocation
    );
    let z = parse("EST5EDT,M3.2.0,M11.1.0");
    for t in [-8_640_000_000_000_000, 0, 8_640_000_000_000_000] {
        let mut work = 2000;
        z.utc_from_local_ms(t, &mut work).unwrap();
        assert!(work > 1000);
    }
    assert_eq!(
        z.utc_from_local_ms(0, &mut 0).unwrap_err(),
        ZoneError::WorkLimit
    );
    assert_eq!(
        z.offset_at_utc_ms(i64::MAX, &mut 1000).unwrap_err(),
        ZoneError::OutOfRange
    );
    for bad in [
        "EST5EDT",
        "AB0",
        "AAA25",
        "AAA0BBB,M0.1.0,M11.1.0",
        "AAA0BBB,M3.1.0/168,M11.1.0",
        "AAA0BBB,J0,J365",
        "AAA0\0",
    ] {
        assert!(
            TimeZoneSnapshot::parse(
                &ZonePayload::new(ZonePayloadKind::Posix2024, bad.as_bytes()).unwrap(),
                &mut ZoneBudget::default()
            )
            .is_err(),
            "{bad}"
        );
    }
}

#[test]
fn timezone_malformed_mutations_remain_bounded_without_panics() {
    let seed = tzif(
        &[(-200, 0), (0, 1), (200, 0)],
        &[(0, false, "UTC"), (3600, true, "DST")],
        "UTC0",
    );
    let mut state = 0x6317_8421u32;
    for trial in 0..2000 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let mut source = seed.clone();
        let index = state as usize % source.len();
        source[index] = (state >> 16) as u8;
        if trial % 7 == 0 {
            source.truncate(index);
        }
        let payload = ZonePayload::new(ZonePayloadKind::Tzif, &source).unwrap();
        let mut budget = ZoneBudget::default();
        let result = TimeZoneSnapshot::parse(&payload, &mut budget);
        assert!(budget.work_left <= 8_388_608 && budget.allocation_left <= 4_194_304);
        if let Ok(zone) = result {
            let mut work = 10_000;
            let _ = zone.utc_from_local_ms(123, &mut work);
            assert!(work <= 10_000);
        }
    }
    for (field, value, error) in [
        (32, u32::MAX, ZoneError::TooLarge),
        (36, 257, ZoneError::TooLarge),
        (40, 65_537, ZoneError::TooLarge),
        (36, 0, ZoneError::Malformed),
    ] {
        let mut source = seed.clone();
        source[field..field + 4].copy_from_slice(&value.to_be_bytes());
        assert_eq!(parse_tzif(&source).unwrap_err(), error);
    }
}

#[test]
fn timezone_zero_dst_delta_tracks_designation_and_transition_time_basis() {
    let zone = parse("AAA0BBB0,M3.2.0/2,M11.1.0/2");
    let tail = zone.tail.as_ref().unwrap();
    assert_eq!(
        tail.state(ms(2024, 1, 1, 0, 0), &mut 1000).unwrap(),
        (0, false)
    );
    assert_eq!(
        tail.state(ms(2024, 7, 1, 0, 0), &mut 1000).unwrap(),
        (0, true)
    );
    // A nonempty footer applies globally when there are no transition records.
    let bytes = tzif(&[], &[(0, false, "-00")], "AAA0BBB0,M3.2.0/2,M11.1.0/2");
    assert_eq!(
        offset(&parse_tzif(&bytes).unwrap(), ms(2024, 7, 1, 0, 0)),
        0
    );
}

#[test]
fn timezone_pinned_iana_2026d_non_utc_rules_and_inverse_boundaries() {
    // Expected seconds/offsets come from the independently preserved IANA source
    // records in tests/fixtures/date-zones/expectations.json, not this parser.
    type Fixture<'a> = (&'a [u8], &'a [(i64, i32, i32)]);
    let fixtures: &[Fixture<'_>] = &[
        (
            include_bytes!("../../../tests/fixtures/date-zones/America/New_York"),
            &[
                (-2_717_650_800, -17_762, -18_000),
                (1_710_054_000, -18_000, -14_400),
                (1_730_613_600, -14_400, -18_000),
            ],
        ),
        (
            include_bytes!("../../../tests/fixtures/date-zones/Australia/Lord_Howe"),
            &[
                (1_712_415_600, 39_600, 37_800),
                (1_728_142_200, 37_800, 39_600),
            ],
        ),
        (
            include_bytes!("../../../tests/fixtures/date-zones/Pacific/Apia"),
            &[(1_325_239_200, -36_000, 50_400)],
        ),
        (
            include_bytes!("../../../tests/fixtures/date-zones/Europe/Dublin"),
            &[(1_711_846_800, 0, 3600), (1_729_990_800, 3600, 0)],
        ),
        (
            include_bytes!("../../../tests/fixtures/date-zones/Asia/Kathmandu"),
            &[
                (-1_577_943_676, 20_476, 19_800),
                (504_901_800, 19_800, 20_700),
            ],
        ),
    ];
    for &(bytes, transitions) in fixtures {
        let zone = parse_tzif(bytes).unwrap();
        for &(second, before, after) in transitions {
            let utc = second * 1000;
            assert_eq!(offset(&zone, utc - 1), before);
            assert_eq!(offset(&zone, utc), after);
            assert_eq!(offset(&zone, utc + 1), after);
            let local_mid = utc + i64::from(before + after) * 500;
            // At an isolated transition midpoint, both a gap's pre-transition
            // choice and a fold's earliest UTC candidate use the former offset.
            assert_eq!(
                inverse(&zone, local_mid),
                local_mid - i64::from(before) * 1000
            );
            for instant in [utc - 2_000_000, utc + 2_000_000] {
                let local = instant + i64::from(offset(&zone, instant)) * 1000;
                // Folds intentionally choose the earliest matching instant.
                assert!(inverse(&zone, local) <= instant);
            }
        }
    }
    let ny = parse_tzif(fixtures[0].0).unwrap();
    assert_eq!(offset(&ny, ms(250_000, 1, 1, 0, 0)), -18_000);
    assert_eq!(offset(&ny, ms(250_000, 7, 1, 0, 0)), -14_400);
}

#[test]
fn timezone_insufficient_multi_unit_work_consumes_remaining_shared_budget() {
    let zone = parse("EST5EDT,M3.2.0,M11.1.0");
    let mut remaining = 7;
    assert_eq!(
        zone.offset_at_utc_ms(0, &mut remaining),
        Err(ZoneError::WorkLimit)
    );
    assert_eq!(remaining, 0);
    let mut remaining = 13;
    assert_eq!(
        zone.utc_from_local_ms(ms(2024, 3, 10, 2, 30), &mut remaining),
        Err(ZoneError::WorkLimit)
    );
    assert_eq!(remaining, 0);
}

#[test]
fn timezone_all_year_and_empty_dst_compare_instants_not_rule_spelling() {
    for rule in [
        "EST5EDT,0/0,J365/25",
        "EST5EDT,J1/0,J365/25",
        "EST5EDT,J2/-24,J365/25",
        "XXX3EDT4,J1/0,J365/23",
    ] {
        let zone = parse(rule);
        for year in [-100, 0, 1, 1900, 2000, 2024, 250_000] {
            for month in [1, 2, 6, 12] {
                assert_eq!(
                    offset(&zone, ms(year, month, 1, 0, 0)),
                    -14_400,
                    "{rule} {year}/{month}"
                );
            }
        }
    }
    let zero = parse("AAA0BBB,0/0,0/1");
    assert_eq!(offset(&zero, ms(2024, 6, 1, 0, 0)), 0);
    assert_eq!(offset(&zero, ms(2024, 1, 1, 0, 0)), 0);
    assert_eq!(offset(&zero, ms(2024, 1, 1, 0, 0) - 1), 0);
}
#[test]
fn timezone_unspecified_footer_never_becomes_utc_but_unused_names_are_allowed() {
    for rule in [
        "<-00>0",
        "<-00>0BBB,M3.2.0,M11.1.0",
        "AAA0<-00>,M3.2.0,M11.1.0",
    ] {
        assert_eq!(
            TimeZoneSnapshot::parse(
                &ZonePayload::new(ZonePayloadKind::Posix2024, rule.as_bytes()).unwrap(),
                &mut ZoneBudget::default()
            )
            .unwrap_err(),
            ZoneError::IncompleteCoverage
        );
        assert_eq!(
            parse_tzif(&tzif(&[], &[(0, false, "UTC")], rule)).unwrap_err(),
            ZoneError::IncompleteCoverage
        );
    }
    let zone = parse("<-00>3EDT4,J1/0,J365/23");
    assert_eq!(offset(&zone, 0), -14_400);
    let zone = parse("UTC0<-00>,0/0,0/1");
    assert_eq!(offset(&zone, 0), 0);
}
#[test]
fn timezone_designation_lookup_is_cached_before_transition_walk() {
    let name = "A".repeat(65_535);
    let transitions: Vec<_> = (0..65_536).map(|second| (second, 0)).collect();
    let source = tzif(&transitions, &[(0, false, &name)], "UTC0");
    assert!(source.len() < MAX_ZONE_SOURCE_BYTES);
    let mut cursor = Cursor {
        bytes: &source,
        at: 0,
    };
    let first = Header::read(&mut cursor).unwrap();
    let mut budget = ZoneBudget::default();
    Block::read(&mut cursor, first, 4, &mut budget).unwrap();
    let second = Header::read(&mut cursor).unwrap();
    let block = Block::read(&mut cursor, second, 8, &mut budget).unwrap();
    assert_eq!(block.types[0].designation_end, 65_535);
    for transition in &block.transitions {
        assert_eq!(block.name(transition.kind as usize).unwrap().len(), 65_535);
    }
    assert_eq!(parse_tzif(&source).unwrap_err(), ZoneError::Malformed); // bounded footer mismatch
}
