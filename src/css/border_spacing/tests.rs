use super::*;
use crate::css::{self, ComputedStyle};
use crate::dom::Document;

const UNITS: [f32; 4] = [20.0, 30.0, 800.0, 600.0];
fn parsed(value: &str) -> Result<Option<[f32; 2]>, Error> {
    parse(value, UNITS, &mut 100_000)
}
fn computed(source: &str, id: &str) -> ComputedStyle {
    let doc = Document::parse(source);
    let styles = css::compute_styles(&doc, &doc.stylesheets(), 800.0, 600.0);
    styles[doc.query_selector(id).unwrap()].clone()
}

#[test]
fn all_existing_length_units_resolve_both_axes_once() {
    for (value, expected) in [
        ("0", [0.0, 0.0]),
        ("2px 3px", [2.0, 3.0]),
        ("1em 1rem", [20.0, 30.0]),
        ("1ex 1ch", [10.0, 10.0]),
        ("1vw 1vh", [8.0, 6.0]),
        ("1vmin 1vmax", [6.0, 8.0]),
        ("1dvw 1dvh", [8.0, 6.0]),
        ("1svw 1svh", [8.0, 6.0]),
        ("1lvw 1lvh", [8.0, 6.0]),
        ("3pt 1pc", [4.0, 16.0]),
        ("1in 2.54cm", [96.0, 96.0]),
        ("25.4mm 101.6q", [96.0, 96.0]),
        ("-0 +0e500", [0.0, 0.0]),
        ("  3PX  ", [3.0, 3.0]),
        ("2px/**/3px", [2.0, 3.0]),
    ] {
        assert_eq!(parsed(value), Ok(Some(expected)), "{value}");
    }
}

#[test]
fn additive_calc_clamps_only_the_complete_nonpercentage_result() {
    for (value, expected) in [
        ("calc(10px + .5em) calc(10px - .5em)", [20.0, 0.0]),
        ("calc(1px - 3px)", [0.0, 0.0]),
        (
            "calc(-10px + 13px) calc(1em + calc(4px - 1px))",
            [3.0, 23.0],
        ),
        ("1000000in", [1_000_000.0, 1_000_000.0]),
        ("calc(1000000in - 1000000in + 2px)", [2.0, 2.0]),
    ] {
        assert_eq!(parsed(value), Ok(Some(expected)), "{value}");
    }
    let zero = parsed("-0px").unwrap().unwrap();
    assert_eq!(zero.map(f32::to_bits), [0, 0]);
}

#[test]
fn invalid_whole_pairs_and_percent_cancellation_do_not_replace_winners() {
    for value in [
        "",
        "-1px",
        "2px -3px",
        "-1e-999px",
        "1",
        "0.00001",
        "1e-999",
        "0%",
        "10%",
        "calc(10% - 10% + 2px)",
        "calc(0%)",
        "auto",
        "normal",
        "2px,3px",
        "1px 2px 3px",
        "inherit 2px",
        "calc(1px",
        "[1px)",
        "calc(1px +)",
        "calc(1px/**/+/**/2px)",
        "calc(2px * 3)",
        "min(1px,2px)",
        "1e999px",
        "1000001px",
        "1fr",
        "2px!",
        "2px\0",
    ] {
        assert_eq!(parsed(value), Ok(None), "{value:?}");
        assert!(
            !css::supports_declaration("border-spacing", value),
            "{value:?}"
        );
        assert!(
            !css::supports_matches(&format!("(border-spacing:{value})")),
            "{value:?}"
        );
        let s = computed(
            &format!("<div id=x style='border-spacing:7px 9px;border-spacing:{value}'></div>"),
            "#x",
        );
        assert_eq!(s.border_spacing, [7.0, 9.0], "{value:?}");
    }
}

#[test]
fn decoded_units_functions_and_wide_keywords_agree_across_supports_and_cascade() {
    for (value, expected) in [
        (r"2p\78  3PX", [2.0, 3.0]),
        (r"c\61 lc(2px + 3px)", [5.0, 5.0]),
        ("calc(1px - 3px) 2px", [0.0, 2.0]),
    ] {
        assert_eq!(parsed(value), Ok(Some(expected)));
        assert!(css::supports_declaration("border-spacing", value));
        assert!(css::supports_matches(&format!("(border-spacing:{value})")));
        let source = format!("<div id=x style='border-spacing:{value}'></div>");
        assert_eq!(computed(&source, "#x").border_spacing, expected);
    }
    for value in [
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        r"in\69 tial",
    ] {
        assert!(
            css::supports_declaration("border-spacing", value),
            "{value}"
        );
        assert!(
            css::supports_matches(&format!("(border-spacing:{value})")),
            "{value}"
        );
    }
}

#[test]
fn css_initial_ua_table_and_computed_pixel_inheritance_are_distinct() {
    assert_eq!(ComputedStyle::default().border_spacing, [0.0, 0.0]);
    let source = "<section id=p style='font-size:20px;border-spacing:1em 2em'><div id=child style='font-size:10px'></div><table id=ua></table><table id=inherit style='font-size:10px;border-spacing:inherit'></table><table id=unset style='border-spacing:unset'></table><table id=initial style='border-spacing:initial'></table><table id=revert style='border-spacing:9px;border-spacing:revert'></table></section>";
    for (id, pair) in [
        ("#p", [20.0, 40.0]),
        ("#child", [20.0, 40.0]),
        ("#ua", [2.0, 2.0]),
        ("#inherit", [20.0, 40.0]),
        ("#unset", [20.0, 40.0]),
        ("#initial", [0.0, 0.0]),
        ("#revert", [2.0, 2.0]),
    ] {
        assert_eq!(computed(source, id).border_spacing, pair, "{id}");
    }
    assert_eq!(
        computed("<div id=x></div>", "#x").border_spacing,
        [0.0, 0.0]
    );
}

#[test]
fn font_prepass_viewport_units_and_unrelated_border_shorthand_remain_independent() {
    let source = "<html style='font-size:30px'><body><div id=x style='border-spacing:1em 1rem;font-size:20px;border:3px solid red'></div><div id=y style='border-spacing:1vw 1vh'></div></body></html>";
    let x = computed(source, "#x");
    assert_eq!(x.border_spacing, [20.0, 30.0]);
    assert_eq!(x.border_width, css::Edges::all(3.0));
    assert_eq!(computed(source, "#y").border_spacing, [8.0, 6.0]);
}

#[test]
fn all_var_invalid_computed_values_and_layer_rollback_keep_one_pair() {
    let source = "<style>@layer base,later;@layer base{#layer{border-spacing:2px 3px}}@layer later{#layer{border-spacing:9px;border-spacing:revert-layer}}</style><section style='border-spacing:4px 6px'><div id=unset style='border-spacing:8px;all:unset'></div><div id=initial style='all:initial'></div><div id=var style='--s:1em 3px;font-size:20px;border-spacing:var(--s)'></div><div id=bad style='--s:2px 3px 4px;border-spacing:8px;border-spacing:var(--s)'></div><div id=layer></div><div id=important style='border-spacing:2px 3px!important;border-spacing:9px'></div></section>";
    for (id, pair) in [
        ("#unset", [4.0, 6.0]),
        ("#initial", [0.0, 0.0]),
        ("#var", [20.0, 3.0]),
        ("#bad", [4.0, 6.0]),
        ("#layer", [2.0, 3.0]),
        ("#important", [2.0, 3.0]),
    ] {
        assert_eq!(computed(source, id).border_spacing, pair, "{id}");
    }
}

#[test]
fn parser_tariffs_have_literal_exact_and_one_short_boundaries() {
    // Tokenizer2n+1; per component3*first-token-bytes+4; each consumed
    // token raw bytes+1; final pair publication8. Calc consumes one cursor.
    for (text, fee, expected) in [
        ("0", 20, [0.0, 0.0]),
        ("2px", 32, [2.0, 2.0]),
        ("2px 3px", 59, [2.0, 3.0]),
        ("calc(1px - 3px)", 80, [0.0, 0.0]),
        ("calc(1px - 3px) calc(1px - 3px)", 155, [0.0, 0.0]),
    ] {
        let mut exact = fee;
        assert_eq!(parse(text, UNITS, &mut exact), Ok(Some(expected)), "{text}");
        assert_eq!(exact, 0);
        let mut short = fee - 1;
        assert_eq!(parse(text, UNITS, &mut short), Err(Error::Limit), "{text}");
        assert_eq!(short, 0);
    }
    let mut s = ComputedStyle {
        border_spacing: [7.0, 9.0],
        ..ComputedStyle::default()
    };
    css::apply_property(
        &mut s,
        "border-spacing",
        "2px 3px",
        None,
        [30.0, 800.0, 600.0],
        &mut 58,
    );
    assert_eq!(s.border_spacing, [7.0, 9.0]);
    css::apply_property(
        &mut s,
        "border-spacing",
        "2px 3px",
        None,
        [30.0, 800.0, 600.0],
        &mut 59,
    );
    assert_eq!(s.border_spacing, [2.0, 3.0]);
}

#[test]
fn one_shared_depth_term_and_raw_budget_covers_both_components() {
    let many = |n: usize| format!("calc({})", vec!["1px"; n].join(" + "));
    let accepted = format!("{} {}", many(128), many(128));
    assert_eq!(parsed(&accepted), Ok(Some([128.0, 128.0])));
    let over_terms = format!("{} {}", many(129), many(128));
    assert_eq!(parsed(&over_terms), Err(Error::Limit));
    let deep = format!("{}1px{}", "calc(".repeat(18), ")".repeat(18));
    assert_eq!(parsed(&deep), Err(Error::Limit));
    assert_eq!(parsed(&" ".repeat(4097)), Err(Error::Limit));
    assert_eq!(parsed("calc(1px"), Ok(None));
}
