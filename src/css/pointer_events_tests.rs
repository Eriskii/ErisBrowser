//! Literal CSS rules, independent of point-selection implementation.
use super::*;

fn check(sheet: &str, expected: &[(&str, PointerEvents)]) {
    let doc = Document::parse(
        "<main id=p><i id=a>text</i><i id=b></i><i id=c></i><i id=d></i><i id=e></i><i id=f></i><i id=g></i><i id=h></i></main>",
    );
    let styles = compute_styles(&doc, &[sheet.to_owned()], 80.0, 60.0);
    assert_eq!(styles.len(), doc.nodes.len());
    for (name, value) in expected {
        let id = doc.query_selector(&format!("#{name}")).unwrap();
        assert_eq!(styles[id].pointer_events, *value, "{name}: {sheet}");
    }
}

#[test]
fn pointer_events_css_single_ident_and_css_wide_values() {
    use PointerEvents::{Auto, None};
    for (value, expected) in [
        ("auto", Auto),
        ("none", None),
        ("AUTO", Auto),
        ("NoNe", None),
        (r"\6e one", None),
        (r"n\6f ne", None),
        ("/*x*/none/**/", None),
        ("inherit", None),
        ("initial", Auto),
        ("unset", None),
        ("revert", None),
        ("revert-layer", None),
        (r"\69 nitial", Auto),
    ] {
        check(
            &format!("#p{{pointer-events:none}}#a{{pointer-events:auto;pointer-events:{value}}}"),
            &[("a", expected)],
        );
    }
    check(r"#a{pointer\2d events:none}", &[("a", None)]);
}

#[test]
fn pointer_events_css_invalid_values_preserve_authored_winner() {
    for value in [
        "visible",
        "hidden",
        "collapse",
        "visiblePainted",
        "bounding-box",
        "all",
        "auto none",
        "none,auto",
        "\"none\"",
        "none()",
        "no/**/ne",
        "none/**/auto",
        "0",
    ] {
        check(
            &format!("#p{{pointer-events:none}}#a{{pointer-events:auto;pointer-events:{value}}}"),
            &[("a", PointerEvents::Auto)],
        );
    }
}

#[test]
fn pointer_events_css_inherits_and_resets_through_all() {
    use PointerEvents::{Auto, None};
    check(
        "#p{pointer-events:none}#b{pointer-events:inherit}#c{pointer-events:unset}#d{pointer-events:initial}#e{pointer-events:auto}#f{all:initial}#g{all:unset}",
        &[
            ("p", None),
            ("a", None),
            ("b", None),
            ("c", None),
            ("d", Auto),
            ("e", Auto),
            ("f", Auto),
            ("g", None),
        ],
    );
    let doc = Document::parse(
        "<div id=p style='pointer-events:none'>text<span id=c style='pointer-events:auto'>child</span></div>",
    );
    let styles = compute_styles(&doc, &[], 80.0, 60.0);
    for (selector, expected) in [("#p", None), ("#c", Auto)] {
        let node = doc.query_selector(selector).unwrap();
        let text = doc.nodes[node].children[0];
        assert!(matches!(
            doc.nodes[text].kind,
            crate::dom::NodeKind::Text(_)
        ));
        assert_eq!(styles[text].pointer_events, expected);
    }
}

#[test]
fn pointer_events_css_variables_use_inherited_invalid_computed_value() {
    use PointerEvents::{Auto, None};
    check(
        "#p{pointer-events:none;--good:auto;--bad:visiblePainted;--empty:;--x:var(--y);--y:var(--x)}#a{pointer-events:var(--good)}#b{pointer-events:var(--absent,auto)}#c{pointer-events:auto;pointer-events:var(--bad)}#d{pointer-events:auto;pointer-events:var(--absent)}#e{pointer-events:auto;pointer-events:var(--empty,auto)}#f{pointer-events:var(--x,auto)}#g{pointer-events:var(--absent,initial)}#h{pointer-events:var(--absent,var(--good))}",
        &[
            ("a", Auto),
            ("b", Auto),
            ("c", None),
            ("d", None),
            ("e", None),
            ("f", Auto),
            ("g", Auto),
            ("h", Auto),
        ],
    );
}

#[test]
fn pointer_events_css_layers_importance_and_revert() {
    use PointerEvents::{Auto, None};
    check(
        "#p{pointer-events:none}@layer first,second;@layer first{#a,#b,#c{pointer-events:none}#b{pointer-events:none!important}}@layer second{#a,#c{pointer-events:auto}#b{pointer-events:auto!important}#c{pointer-events:revert-layer}}#d{pointer-events:auto;pointer-events:revert}#e{pointer-events:none!important;pointer-events:auto}",
        &[
            ("a", Auto),
            ("b", None),
            ("c", None),
            ("d", None),
            ("e", None),
        ],
    );
}

#[test]
fn pointer_events_css_supports_respects_property_and_value_tokens() {
    for value in [
        "auto",
        "none",
        "AUTO",
        r"\6e one",
        "none/**/",
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
    ] {
        assert!(supports_declaration("pointer-events", value), "{value}");
    }
    for (property, value) in [
        ("pointer-events", "visiblePainted"),
        ("pointer-events", "no/**/ne"),
        ("pointer-events", "auto none"),
        ("pointer-events", "none()"),
        ("pointer-events", "var(--x)"),
        ("pointer-events", "none!important"),
        ("pointer-events", "none;color:red"),
        (r"pointer\2d events", "none"),
        (" pointer-events", "none"),
    ] {
        assert!(
            !supports_declaration(property, value),
            "{property}: {value}"
        );
    }
    for (query, expected) in [
        ("(pointer-events:none)", true),
        (r"(pointer\2d events:\6e one)", true),
        ("(pointer-events:no/**/ne)", false),
        ("(pointer-events:visiblePainted)", false),
        ("(pointer-events:none!important)", true),
        ("not (pointer-events:visiblePainted)", true),
        ("(pointer-events:auto) and (pointer-events:none)", true),
    ] {
        assert_eq!(supports_matches(query), expected, "{query}");
    }
    check(
        r"@supports (pointer\2d events:\6e one){#a{pointer-events:none}}@supports (pointer-events:visiblePainted){#b{pointer-events:none}}",
        &[("a", PointerEvents::None), ("b", PointerEvents::Auto)],
    );
}
