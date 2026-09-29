// Independent expectations frozen in /tmp/eris-date-arithmetic-plan.json
// SHA-256 70f68276f52cba1d4a30e9089a9209cada78516237d5635251b56f8cf33491a4.
// Calendar anchors came from two integer formulas; FP values use a rational
// nearest-even oracle, never this implementation or another JS engine.
use super::*;

fn same(actual: f64, expected: f64) {
    if expected.is_nan() {
        assert!(actual.is_nan(), "{actual:?} should be NaN");
    } else {
        assert_eq!(
            actual.to_bits(),
            expected.to_bits(),
            "{actual:?} != {expected:?}"
        );
    }
}
fn parse(value: &str) -> IsoParse {
    parse_iso(&value.encode_utf16().collect::<Vec<_>>())
}

#[test]
fn independent_calendar_anchors() {
    let cases = [
        (
            -271821,
            3,
            20,
            -100000000_i64,
            2,
            "-271821-04-20T00:00:00.000Z",
        ),
        (-400, 1, 29, -865566_i64, 2, "-000400-02-29T00:00:00.000Z"),
        (-100, 2, 1, -755993_i64, 4, "-000100-03-01T00:00:00.000Z"),
        (-4, 1, 29, -720930_i64, 4, "-000004-02-29T00:00:00.000Z"),
        (-1, 11, 31, -719529_i64, 5, "-000001-12-31T00:00:00.000Z"),
        (0, 0, 1, -719528_i64, 6, "0000-01-01T00:00:00.000Z"),
        (0, 1, 29, -719469_i64, 2, "0000-02-29T00:00:00.000Z"),
        (1, 0, 1, -719162_i64, 1, "0001-01-01T00:00:00.000Z"),
        (99, 11, 31, -683004_i64, 4, "0099-12-31T00:00:00.000Z"),
        (100, 2, 1, -682944_i64, 1, "0100-03-01T00:00:00.000Z"),
        (1582, 9, 4, -141438_i64, 1, "1582-10-04T00:00:00.000Z"),
        (1582, 9, 15, -141427_i64, 5, "1582-10-15T00:00:00.000Z"),
        (1600, 1, 29, -135081_i64, 2, "1600-02-29T00:00:00.000Z"),
        (1900, 2, 1, -25508_i64, 4, "1900-03-01T00:00:00.000Z"),
        (1969, 11, 31, -1_i64, 3, "1969-12-31T00:00:00.000Z"),
        (1970, 0, 1, 0_i64, 4, "1970-01-01T00:00:00.000Z"),
        (2000, 1, 29, 11016_i64, 2, "2000-02-29T00:00:00.000Z"),
        (2038, 0, 19, 24855_i64, 2, "2038-01-19T00:00:00.000Z"),
        (2100, 2, 1, 47541_i64, 1, "2100-03-01T00:00:00.000Z"),
        (2400, 1, 29, 157113_i64, 2, "2400-02-29T00:00:00.000Z"),
        (9999, 11, 31, 2932896_i64, 5, "9999-12-31T00:00:00.000Z"),
        (10000, 0, 1, 2932897_i64, 6, "+010000-01-01T00:00:00.000Z"),
        (
            275760,
            8,
            13,
            100000000_i64,
            6,
            "+275760-09-13T00:00:00.000Z",
        ),
    ];
    for (year, month, day, expected, weekday, iso) in cases {
        assert_eq!(days_from_civil(year, month, day), expected);
        let parts = parts(expected * DAY_MS);
        assert_eq!(
            (parts.year, parts.month, parts.day, parts.weekday),
            (year, month, day, weekday)
        );
        assert_eq!(format_iso(expected * DAY_MS), iso);
    }
}

#[test]
fn independent_time_decompositions() {
    let cases = [
        (
            -8640000000000000_i64,
            DateParts {
                year: -271821,
                month: 3,
                day: 20,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 0,
                weekday: 2,
            },
        ),
        (
            -8639999999999999_i64,
            DateParts {
                year: -271821,
                month: 3,
                day: 20,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 1,
                weekday: 2,
            },
        ),
        (
            -86400001_i64,
            DateParts {
                year: 1969,
                month: 11,
                day: 30,
                hour: 23,
                minute: 59,
                second: 59,
                millisecond: 999,
                weekday: 2,
            },
        ),
        (
            -86400000_i64,
            DateParts {
                year: 1969,
                month: 11,
                day: 31,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 0,
                weekday: 3,
            },
        ),
        (
            -1_i64,
            DateParts {
                year: 1969,
                month: 11,
                day: 31,
                hour: 23,
                minute: 59,
                second: 59,
                millisecond: 999,
                weekday: 3,
            },
        ),
        (
            0_i64,
            DateParts {
                year: 1970,
                month: 0,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 0,
                weekday: 4,
            },
        ),
        (
            1_i64,
            DateParts {
                year: 1970,
                month: 0,
                day: 1,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 1,
                weekday: 4,
            },
        ),
        (
            86399999_i64,
            DateParts {
                year: 1970,
                month: 0,
                day: 1,
                hour: 23,
                minute: 59,
                second: 59,
                millisecond: 999,
                weekday: 4,
            },
        ),
        (
            86400000_i64,
            DateParts {
                year: 1970,
                month: 0,
                day: 2,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 0,
                weekday: 5,
            },
        ),
        (
            8639999999999999_i64,
            DateParts {
                year: 275760,
                month: 8,
                day: 12,
                hour: 23,
                minute: 59,
                second: 59,
                millisecond: 999,
                weekday: 5,
            },
        ),
        (
            8640000000000000_i64,
            DateParts {
                year: 275760,
                month: 8,
                day: 13,
                hour: 0,
                minute: 0,
                second: 0,
                millisecond: 0,
                weekday: 6,
            },
        ),
    ];
    for (time, expected) in cases {
        assert_eq!(parts(time), expected);
    }
}

#[test]
fn independent_timeclip_edges() {
    let cases = [
        (-8640000000000001.0, f64::NAN),
        (-8640000000000000.0, -8640000000000000.0),
        (-8639999999999999.0, -8639999999999999.0),
        (8639999999999999.0, 8639999999999999.0),
        (8640000000000000.0, 8640000000000000.0),
        (8640000000000001.0, f64::NAN),
        (f64::NAN, f64::NAN),
        (f64::INFINITY, f64::NAN),
        (f64::NEG_INFINITY, f64::NAN),
        (-0.0, 0.0),
        (0.0, 0.0),
        (-0.9, 0.0),
        (0.9, 0.0),
        (-1.9, -1.0),
        (1.9, 1.0),
        (-f64::from_bits(1), 0.0),
    ];
    for (input, expected) in cases {
        same(time_clip(input), expected);
    }
}

#[test]
fn independent_make_day_vectors() {
    let cases = [
        (1970.0, -1.0, 1.0, -31.0),
        (1970.0, -12.0, 1.0, -365.0),
        (1970.0, -13.0, 1.0, -396.0),
        (1970.0, 0.0, 0.0, -1.0),
        (1970.0, 0.0, -1.0, -2.0),
        (1970.0, -0.9, 1.9, 0.0),
        (2000.0, 1.0, 30.0, 11017.0),
        (1900.0, 1.0, 29.0, -25508.0),
        (1970.0, 0.0, 100000001.0, 100000000.0),
        (-6004799503158692.0, 72057594037927952.0, 1.0, 243.0),
        (6004799503162633.0, -72057594037927952.0, 1.0, 120.0),
        (1971.0, 0.0, 9007199254740994.0, 9007199254741360.0),
        (1970.0, 0.0, 213503982336.0, 213503982335.0),
    ];
    for (year, month, day, expected) in cases {
        same(make_day(year, month, day), expected);
    }
}

#[test]
fn independent_make_time_vectors() {
    let cases = [
        (0.0, 0.0, 0.0, -1.0, -1.0),
        (24.0, 0.0, 0.0, 0.0, 86400000.0),
        (-1.9, -2.9, -3.9, -4.9, -3723004.0),
        (80063993375.0, 29.0, 1.0, -288230376151711740.0, 29312.0),
        (1099511627776.0, 0.0, 1.0, -3958241859993600000.0, 1024.0),
        (17592186044416.0, 0.0, 1.0, -63331869759897600000.0, 0.0),
        (0.0, 0.0, 9007199254740992.0, -9007199254740992000.0, 0.0),
    ];
    for (hour, minute, second, ms, expected) in cases {
        same(make_time(hour, minute, second, ms), expected);
    }
    same(make_time(-0.0, -0.9, -0.0, -0.9), 0.0);
    same(make_time(1e303, -6e304, 0.0, 0.0), f64::NAN);
}

#[test]
fn independent_make_date_vectors() {
    let cases = [
        (213503982335.0, -18446744073709552000.0, 34447360.0),
        (1099511627777.0, -94997804639932792832.0, 0.0),
        (100000001.0, -86400000.0, 8640000000000000.0),
        (-100000001.0, 86400000.0, -8640000000000000.0),
    ];
    for (day, time, expected) in cases {
        same(make_date(day, time), expected);
    }
}

#[test]
fn independent_iso_values() {
    let cases = [
        ("1970", 0, 0_i64),
        ("1970-01", 0, 0_i64),
        ("1970-01-01", 19800, 0_i64),
        ("1970-01-01T00:00", 19800, -19800000_i64),
        ("1970-01-01T00:00:00Z", 19800, 0_i64),
        ("1970-01-01T00:00:00.001+05:30", 0, -19799999_i64),
        ("1970-01-01T00:00:00-03:30", 0, 12600000_i64),
        ("1995-02-04T24:00Z", 0, 791942400000_i64),
        ("2000-02-29T24:00:00.000Z", 0, 951868800000_i64),
        ("0000-01-01T00:00:00.000Z", 0, -62167219200000_i64),
        ("+000000-01-01T00:00:00.000Z", 0, -62167219200000_i64),
        ("0099-01-01T00:00:00.000Z", 0, -59042995200000_i64),
        ("-000001-01-01T00:00:00.000Z", 0, -62198755200000_i64),
        ("+010000-01-01T00:00:00.000Z", 0, 253402300800000_i64),
        ("-271821-04-20T00:00:00.000Z", 0, -8640000000000000_i64),
        ("+275760-09-13T00:00:00.000Z", 0, 8640000000000000_i64),
        ("+275760-09-13T01:00:00.000+01:00", 0, 8640000000000000_i64),
        ("-271821-04-19T23:00:00.000-01:00", 0, -8640000000000000_i64),
    ];
    for (source, offset, expected) in cases {
        let IsoParse::Value { local_ms, is_local } = parse(source) else {
            panic!("{source}");
        };
        same(
            local_ms
                - if is_local {
                    f64::from(offset) * 1000.0
                } else {
                    0.0
                },
            expected as f64,
        );
    }
}

#[test]
fn independent_iso_rejections() {
    for source in [
        "-000000-01-01T00:00:00Z",
        "-271821-04-19T23:59:59.999Z",
        "+275760-09-13T00:00:00.001Z",
        "2000-00-01T00:00Z",
        "2000-13-01T00:00Z",
        "2000-01-00T00:00Z",
        "2000-01-32T00:00Z",
        "2000-01-01T25:00Z",
        "2000-01-01T24:01Z",
        "2000-01-01T24:00:00.001Z",
        "2000-01-01T00:60Z",
        "2000-01-01T00:00:60Z",
        "2000-01-01T00:00+24:00",
    ] {
        assert_eq!(parse(source), IsoParse::Invalid, "{source}");
    }
}

#[test]
fn nonfinite_components_and_full_year() {
    for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for index in 0..3 {
            let mut args = [1970.0, 0.0, 1.0];
            args[index] = invalid;
            assert!(make_day(args[0], args[1], args[2]).is_nan());
        }
        for index in 0..4 {
            let mut args = [0.0; 4];
            args[index] = invalid;
            assert!(make_time(args[0], args[1], args[2], args[3]).is_nan());
        }
        assert!(make_date(invalid, 0.0).is_nan());
        assert!(make_date(0.0, invalid).is_nan());
        assert!(full_year(invalid).is_nan());
    }
    for (input, expected) in [
        (-0.9, 1900.0),
        (0.0, 1900.0),
        (99.9, 1999.0),
        (100.0, 100.0),
        (-1.0, -1.0),
    ] {
        same(full_year(input), expected);
    }
}

// This January-based reference deliberately differs from parts' March-era inverse.
fn reference_day(year: i64, month: u8, day: u8) -> i64 {
    let before = year - 1;
    let jan = 365 * (year - 1970) + before.div_euclid(4)
        - 1969_i64.div_euclid(4)
        - before.div_euclid(100)
        + 1969_i64.div_euclid(100)
        + before.div_euclid(400)
        - 1969_i64.div_euclid(400);
    let mut date = jan + i64::from(day) - 1;
    for m in 0..month {
        date += match m {
            1 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
            1 => 28,
            3 | 5 | 8 | 10 => 30,
            _ => 31,
        };
    }
    date
}

#[test]
fn full_era_month_boundaries_and_translated_negative_eras() {
    for year in -400..0 {
        for month in 0..12 {
            let day = reference_day(year, month, 1);
            for edge in [-1, 0, i64::from(days_in_month(year as i32, month)) - 1] {
                let timestamp = (day + edge) * DAY_MS;
                let value = parts(timestamp);
                assert_eq!(
                    reference_day(i64::from(value.year), value.month, value.day),
                    day + edge
                );
                assert_eq!(
                    parse(&format_iso(timestamp)),
                    IsoParse::Value {
                        local_ms: timestamp as f64,
                        is_local: false
                    }
                );
            }
        }
    }
    assert_eq!(reference_day(0, 0, 1) - reference_day(-400, 0, 1), 146097);
}

#[test]
fn deterministic_full_range_inverse_and_iso_sweep() {
    let mut seed = 0xdec0_2620_2026_0929_u64;
    for _ in 0..4096 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let timestamp = (seed % 17_280_000_000_000_001) as i64 - 8_640_000_000_000_000;
        let p = parts(timestamp);
        let day = reference_day(i64::from(p.year), p.month, p.day);
        let reconstructed = day * DAY_MS
            + i64::from(p.hour) * 3_600_000
            + i64::from(p.minute) * 60_000
            + i64::from(p.second) * 1000
            + i64::from(p.millisecond);
        assert_eq!(timestamp, reconstructed);
        assert_eq!(
            parse(&format_iso(timestamp)),
            IsoParse::Value {
                local_ms: timestamp as f64,
                is_local: false
            }
        );
    }
    for timestamp in [
        i64::MIN,
        i64::MAX,
        -8_640_000_691_200_000,
        8_640_000_691_200_000,
    ] {
        let p = parts(timestamp);
        let reconstructed = i128::from(reference_day(i64::from(p.year), p.month, p.day))
            * i128::from(DAY_MS)
            + i128::from(p.hour) * 3_600_000
            + i128::from(p.minute) * 60_000
            + i128::from(p.second) * 1000
            + i128::from(p.millisecond);
        assert_eq!(i128::from(timestamp), reconstructed);
    }
}

#[test]
fn iso_required_forms_and_utf16_rejection() {
    for input in ["1970T00:00Z", "1970-01T00:00Z", "1970-01-01T00:00Z"] {
        assert_eq!(
            parse(input),
            IsoParse::Value {
                local_ms: 0.0,
                is_local: false
            }
        );
    }
    assert_eq!(
        parse("2000-02-30T00:00Z"),
        IsoParse::Value {
            local_ms: 951868800000.0,
            is_local: false
        }
    );
    assert_eq!(
        parse("+275760-09-13T01:00"),
        IsoParse::Value {
            local_ms: MAX_TIME + 3600000.0,
            is_local: true
        }
    );
    for input in [
        "1970-01-01T",
        "1970-01-01T00",
        "1970-01-01T00:00:00.",
        "1970-01-01T00:00.000Z",
        "1970-01-01T00:00:00.0000Z",
        "1970-01-01T00:00Z\0",
        "1970-01-01T00:00:00+00",
        "1970-01-01T00:00ZZ",
        "+0000000-01-01",
        "-000000T00:00Z",
    ] {
        assert_eq!(parse(input), IsoParse::Invalid, "{input}");
    }
    for input in ["", "Thu, 01 Jan 1970 00:00:00 GMT", "01/01/1970", "hello"] {
        assert_eq!(parse(input), IsoParse::Unrecognized, "{input}");
    }
    let mut source: Vec<u16> = "1970-01-01T00:00Z".encode_utf16().collect();
    for value in [0, 0xd800, 0xdc00, 0xffff] {
        source[15] = value;
        assert_eq!(parse_iso(&source), IsoParse::Invalid);
    }
    assert_eq!(parse_iso(&[48; 65536]), IsoParse::Unrecognized);
}

#[test]
fn bounded_wide_integer_conversion_division_and_rounding() {
    for n in [i64::MIN, -100, -13, -12, -1, 0, 1, 12, 13, 100, i64::MAX] {
        let wide = Wide::small(n);
        for d in [4, 12, 100, 400] {
            let (q, r) = wide.div_floor(d);
            same(q.to_float(), n.div_euclid(i64::from(d)) as f64);
            assert_eq!(r, n.rem_euclid(i64::from(d)) as u32);
        }
    }
    for exponent in 0..=1023 {
        for mantissa in [0, 1, 0x5_5555_5555_5555, 0xf_ffff_ffff_ffff] {
            let n = f64::from_bits(((exponent as u64 + 1023) << 52) | mantissa).trunc();
            for value in [n, -n] {
                same(Wide::from_float(value).to_float(), value);
            }
        }
    }
    same(
        Wide::from_float(9007199254740992.0)
            .add(Wide::small(1))
            .to_float(),
        9007199254740992.0,
    );
    same(
        Wide::from_float(9007199254740992.0)
            .add(Wide::small(3))
            .to_float(),
        9007199254740996.0,
    );
}

#[test]
fn wide_cancellation_and_precharge_classification() {
    assert_eq!(make_day_work(1970.0, 0.0, 1.0), FAST_WORK);
    assert_eq!(make_day_work(f64::MAX, -f64::MAX, 1.0), WIDE_WORK);
    assert_eq!(make_day_work(1e6, 12e6, 1e300), FAST_WORK);
    // A huge quotient can exactly cancel its year even beyond i128 inputs.
    for exponent in [60, 100, 200, 500, 1000] {
        let month = f64::from_bits(((exponent + 1023) as u64) << 52);
        let (quotient, remainder) = Wide::from_float(month).div_floor(12);
        same(
            make_day(-quotient.to_float(), month, 1.0),
            reference_day(0, remainder as u8, 1) as f64,
        );
    }
    assert!(make_day(f64::MAX, 0.0, 1.0).is_nan());
    assert!(make_day(-f64::MAX, 0.0, 1.0).is_nan());
    // Large but representable calendar days must not be rejected at TimeClip.
    for year in [i32::MIN, i32::MAX] {
        same(
            make_day(f64::from(year), 0.0, 1.0),
            reference_day(i64::from(year), 0, 1) as f64,
        );
    }
}

// 104 independent wide-path expectations: fixed Fraction derivation, no engine.
#[test]
fn independent_extreme_year_and_month_vectors() {
    let cases: &[(u64, u64, u64, Option<u64>)] = &[
        (
            0x41f0000000000000,
            0x4020000000000000,
            0x3ff0000000000000,
            Some(0x4276d3e09812e000),
        ),
        (
            0x41f0000000000001,
            0x4022000000000000,
            0x3ff0000000000000,
            Some(0x4276d3e09814c000),
        ),
        (
            0xc1f0000000000000,
            0x4020000000000000,
            0x3ff0000000000000,
            Some(0xc276d3e1f7496000),
        ),
        (
            0xc1f0000000000001,
            0x4022000000000000,
            0x3ff0000000000000,
            Some(0xc276d3e1f7478000),
        ),
        (
            0x4270000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            Some(0x42f6d3e146fe7190),
        ),
        (
            0x4270000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            Some(0x42f6d3e146fe7380),
        ),
        (
            0xc270000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            Some(0xc2f6d3e1485db770),
        ),
        (
            0xc270000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            Some(0xc2f6d3e1485db580),
        ),
        (
            0x42a0000000000000,
            0x401c000000000000,
            0x3ff0000000000000,
            Some(0x4326d3e1479820d4),
        ),
        (
            0x42a0000000000001,
            0x4020000000000000,
            0x3ff0000000000000,
            Some(0x4326d3e147982112),
        ),
        (
            0xc2a0000000000000,
            0x401c000000000000,
            0x3ff0000000000000,
            Some(0xc326d3e147c40820),
        ),
        (
            0xc2a0000000000001,
            0x4020000000000000,
            0x3ff0000000000000,
            Some(0xc326d3e147c407e2),
        ),
        (
            0x42b0000000000000,
            0x4020000000000000,
            0x3ff0000000000000,
            Some(0x4336d3e147a31ac7),
        ),
        (
            0x42b0000000000001,
            0x4022000000000000,
            0x3ff0000000000000,
            Some(0x4336d3e147a31ae5),
        ),
        (
            0xc2b0000000000000,
            0x4020000000000000,
            0x3ff0000000000000,
            Some(0xc336d3e147b90e2f),
        ),
        (
            0xc2b0000000000001,
            0x4022000000000000,
            0x3ff0000000000000,
            Some(0xc336d3e147b90e11),
        ),
        (
            0x42c0000000000000,
            0x4022000000000000,
            0x3ff0000000000000,
            Some(0x4346d3e147a897b0),
        ),
        (
            0x42c0000000000001,
            0x4024000000000000,
            0x3ff0000000000000,
            Some(0x4346d3e147a897c0),
        ),
        (
            0xc2c0000000000000,
            0x4022000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc2c0000000000001,
            0x4024000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x42e0000000000000,
            0x4026000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x42e0000000000001,
            0x0000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc2e0000000000000,
            0x4026000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc2e0000000000001,
            0x0000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x42f0000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x42f0000000000001,
            0x3ff0000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc2f0000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc2f0000000000001,
            0x3ff0000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4310000000000000,
            0x4000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4310000000000001,
            0x4008000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc310000000000000,
            0x4000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc310000000000001,
            0x4008000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4330000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4330000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            Some(0x43b6d3e147ae0982),
        ),
        (
            0xc330000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            Some(0xc3b6d3e147ae1f75),
        ),
        (
            0xc330000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4350000000000000,
            0x4018000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4350000000000001,
            0x401c000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc350000000000000,
            0x4018000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc350000000000001,
            0x401c000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x43b0000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x43b0000000000001,
            0x3ff0000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc3b0000000000000,
            0x0000000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc3b0000000000001,
            0x3ff0000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4630000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x4630000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc630000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc630000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x5f30000000000000,
            0x4020000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x5f30000000000001,
            0x4022000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xdf30000000000000,
            0x4020000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xdf30000000000001,
            0x4022000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x7e70000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0x7e70000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xfe70000000000000,
            0x4010000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xfe70000000000001,
            0x4014000000000000,
            0x3ff0000000000000,
            None,
        ),
        (
            0xc305555555555550,
            0x4340000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xc30c71c71c71c718,
            0x4345555555555555,
            0x3ff0000000000000,
            Some(0xc125f3e400000000),
        ),
        (
            0xc315555555555554,
            0x434fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f4d800000000),
        ),
        (
            0x4305555555555558,
            0xc340000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x430c71c71c71c720,
            0xc345555555555555,
            0x3ff0000000000000,
            Some(0xc125f3e400000000),
        ),
        (
            0x4315555555555558,
            0xc34fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f2ee00000000),
        ),
        (
            0xc335555555555555,
            0x4370000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0xc33c71c71c71c71c,
            0x4375555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0xc345555555555554,
            0x437fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x4335555555555556,
            0xc370000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0x433c71c71c71c71c,
            0xc375555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0x4345555555555555,
            0xc37fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xc3b5555555555555,
            0x43f0000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0xc3bc71c71c71c71c,
            0x43f5555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0xc3c5555555555555,
            0x43ffffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x43b5555555555555,
            0xc3f0000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0x43bc71c71c71c71c,
            0xc3f5555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0x43c5555555555555,
            0xc3ffffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xc5f5555555555555,
            0x4630000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0xc5fc71c71c71c71c,
            0x4635555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0xc605555555555555,
            0x463fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x45f5555555555555,
            0xc630000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0x45fc71c71c71c71c,
            0xc635555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0x4605555555555555,
            0xc63fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xcc35555555555555,
            0x4c70000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0xcc3c71c71c71c71c,
            0x4c75555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0xcc45555555555555,
            0x4c7fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x4c35555555555555,
            0xcc70000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0x4c3c71c71c71c71c,
            0xcc75555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0x4c45555555555555,
            0xcc7fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xdef5555555555555,
            0x5f30000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0xdefc71c71c71c71c,
            0x5f35555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0xdf05555555555555,
            0x5f3fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x5ef5555555555555,
            0xdf30000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0x5efc71c71c71c71c,
            0xdf35555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0x5f05555555555555,
            0xdf3fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xfe35555555555555,
            0x7e70000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0xfe3c71c71c71c71c,
            0x7e75555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0xfe45555555555555,
            0x7e7fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x7e35555555555555,
            0xfe70000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0x7e3c71c71c71c71c,
            0xfe75555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0x7e45555555555555,
            0xfe7fffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xffa5555555555555,
            0x7fe0000000000000,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0xffac71c71c71c71c,
            0x7fe5555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0xffb5555555555555,
            0x7fefffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f36800000000),
        ),
        (
            0x7fa5555555555555,
            0xffe0000000000000,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
        (
            0x7fac71c71c71c71c,
            0xffe5555555555555,
            0x3ff0000000000000,
            Some(0xc125f55000000000),
        ),
        (
            0x7fb5555555555555,
            0xffefffffffffffff,
            0x3ff0000000000000,
            Some(0xc125f45e00000000),
        ),
    ];
    for &(year, month, day, expected) in cases {
        let actual = make_day(
            f64::from_bits(year),
            f64::from_bits(month),
            f64::from_bits(day),
        );
        match expected {
            Some(bits) => assert_eq!(actual.to_bits(), bits, "year={year:x}, month={month:x}"),
            None => assert!(actual.is_nan(), "year={year:x}, month={month:x}: {actual}"),
        }
    }
}
