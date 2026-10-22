//! Independent literal CSS visibility grammar, cascade and feature-query controls.
use super::*;

fn styles(html: &str, sheet: &str) -> (Document, Vec<ComputedStyle>) {
    let doc = Document::parse(html);
    let computed = compute_styles(&doc, &[sheet.to_owned()], 40.0, 32.0);
    (doc, computed)
}

fn assert_nodes(html: &str, sheet: &str, expected: &[(&str, Visibility)]) {
    let (doc, computed) = styles(html, sheet);
    assert_eq!(computed.len(), doc.nodes.len());
    for &(id, visibility) in expected {
        let node = doc.query_selector(&format!("#{id}")).unwrap();
        assert_eq!(computed[node].visibility, visibility, "{id}");
    }
}

#[test]
fn visibility_single_identifier_values() {
    for (value, expected) in [
        (r#"visible"#, Visibility::Visible),
        (r#"hidden"#, Visibility::Hidden),
        (r#"HIDDEN"#, Visibility::Hidden),
        (r#"ViSiBlE"#, Visibility::Visible),
        (r#"  hidden  "#, Visibility::Hidden),
        (r#"/*a*/hidden/*b*/"#, Visibility::Hidden),
        (r#"\68 idden"#, Visibility::Hidden),
        (r#"h\69 dden"#, Visibility::Hidden),
        (r#"h\000069 dden"#, Visibility::Hidden),
        (r#"\76 isible"#, Visibility::Visible),
        (r#"\69 nitial"#, Visibility::Visible),
        (r#"\69 nherit"#, Visibility::Hidden),
        (r#"u\6e set"#, Visibility::Hidden),
        (r#"revert"#, Visibility::Hidden),
        (r#"revert-layer"#, Visibility::Hidden),
    ] {
        let sheet = format!(
            "main {{{}}} #target {{{}visibility:{value}}}",
            r#"visibility:hidden"#, r#"visibility:visible;"#
        );
        assert_nodes(
            "<main><i id=target></i></main>",
            &sheet,
            &[("target", expected)],
        );
    }
}

#[test]
fn visibility_invalid_values_do_not_replace_valid_declarations() {
    for (value, expected) in [
        (r#"collapse"#, Visibility::Visible),
        (r#"auto"#, Visibility::Visible),
        (r#"none"#, Visibility::Visible),
        (r#"visible hidden"#, Visibility::Visible),
        (r#"hidden,visible"#, Visibility::Visible),
        (r#""hidden""#, Visibility::Visible),
        (r#"'hidden'"#, Visibility::Visible),
        (r#"hidden()"#, Visibility::Visible),
        (r#"calc(1)"#, Visibility::Visible),
        (r#"0"#, Visibility::Visible),
        (r#"hi/**/dden"#, Visibility::Visible),
        (r#"hidden/**/visible"#, Visibility::Visible),
        (r#"hidden !bogus"#, Visibility::Visible),
    ] {
        let sheet = format!(
            "main {{{}}} #target {{{}visibility:{value}}}",
            r#"visibility:hidden"#, r#"visibility:visible;"#
        );
        assert_nodes(
            "<main><i id=target></i></main>",
            &sheet,
            &[("target", expected)],
        );
    }
}

#[test]
fn visibility_inherited_initial_unset_and_all() {
    assert_nodes(
        r#"<main id=p><i id=default><b id=grand></b></i><i id=inherit></i><i id=unset></i><i id=initial></i><i id=shown><b id=shownchild></b></i><i id=allinitial></i><i id=allunset></i></main>"#,
        r#"#p{visibility:hidden}#inherit{visibility:inherit}#unset{visibility:unset}#initial{visibility:initial}#shown{visibility:visible}#allinitial{all:initial}#allunset{all:unset}"#,
        &[
            ("p", Visibility::Hidden),
            ("default", Visibility::Hidden),
            ("grand", Visibility::Hidden),
            ("inherit", Visibility::Hidden),
            ("unset", Visibility::Hidden),
            ("initial", Visibility::Visible),
            ("shown", Visibility::Visible),
            ("shownchild", Visibility::Visible),
            ("allinitial", Visibility::Visible),
            ("allunset", Visibility::Hidden),
        ],
    );
}

#[test]
fn visibility_variables_use_inherited_invalid_computed_value() {
    assert_nodes(
        r#"<main id=p><i id=good></i><i id=fallback></i><i id=invalid></i><i id=missing></i><i id=nested></i><i id=empty></i><i id=cycle></i><i id=wide></i></main>"#,
        r#"#p{visibility:hidden;--good:visible;--bad:collapse;--empty:;--a:var(--b);--b:var(--a)}#good{visibility:var(--good)}#fallback{visibility:var(--absent,visible)}#invalid{visibility:visible;visibility:var(--bad)}#missing{visibility:visible;visibility:var(--absent)}#nested{visibility:var(--absent,var(--good))}#empty{visibility:visible;visibility:var(--empty,visible)}#cycle{visibility:var(--a,visible)}#wide{visibility:var(--absent,initial)}"#,
        &[
            ("good", Visibility::Visible),
            ("fallback", Visibility::Visible),
            ("invalid", Visibility::Hidden),
            ("missing", Visibility::Hidden),
            ("nested", Visibility::Visible),
            ("empty", Visibility::Hidden),
            ("cycle", Visibility::Visible),
            ("wide", Visibility::Visible),
        ],
    );
}

#[test]
fn visibility_cascade_priority_layers_and_revert() {
    assert_nodes(
        r#"<main style="visibility:hidden"><i id=author style="visibility:hidden"></i><i id=inline style="visibility:hidden!important"></i><i id=normal></i><i id=important></i><i id=rollback></i><i id=origin></i></main>"#,
        r#"@layer first,second;@layer first{#normal,#important,#rollback{visibility:hidden}#important{visibility:hidden!important}}@layer second{#normal,#rollback{visibility:visible}#important{visibility:visible!important}#rollback{visibility:revert-layer}}#author,#inline{visibility:visible!important}#origin{visibility:visible;visibility:revert}"#,
        &[
            ("author", Visibility::Visible),
            ("inline", Visibility::Hidden),
            ("normal", Visibility::Visible),
            ("important", Visibility::Hidden),
            ("rollback", Visibility::Hidden),
            ("origin", Visibility::Hidden),
        ],
    );
}

#[test]
fn visibility_supports_token_boundaries_and_property_overloads() {
    for (property, value, expected) in [
        (r#"visibility"#, r#"visible"#, true),
        (r#"visibility"#, r#"hidden"#, true),
        (r#"visibility"#, r#"HIDDEN"#, true),
        (r#"visibility"#, r#"\68 idden"#, true),
        (r#"visibility"#, r#"hidden/**/"#, true),
        (r#"visibility"#, r#"initial"#, true),
        (r#"visibility"#, r#"inherit"#, true),
        (r#"visibility"#, r#"unset"#, true),
        (r#"visibility"#, r#"revert"#, true),
        (r#"visibility"#, r#"revert-layer"#, true),
        (r#"visibility"#, r#"collapse"#, false),
        (r#"visibility"#, r#"hi/**/dden"#, false),
        (r#"visibility"#, r#"visible hidden"#, false),
        (r#"visibility"#, r#""hidden""#, false),
        (r#"visibility"#, r#"hidden()"#, false),
        (r#"visibility"#, r#"var(--x)"#, true),
        (r#"visibility"#, r#"hidden!important"#, false),
        (r#"visibility"#, r#"hidden;color:red"#, false),
        (r#"vi\73 ibility"#, r#"hidden"#, false),
        (r#" visibility"#, r#"hidden"#, false),
    ] {
        assert_eq!(
            supports_declaration(property, value),
            expected,
            "{property}: {value}"
        );
    }
    for (condition, expected) in [
        (r#"(visibility:hidden)"#, true),
        (r#"(visibility:HIDDEN)"#, true),
        (r#"(visibility:\68 idden)"#, true),
        (r#"(vi\73 ibility:hidden)"#, true),
        (r#"(visibility:hi/**/dden)"#, false),
        (r#"(visibility:collapse)"#, false),
        (r#"(visibility:var(--x))"#, true),
        (r#"(visibility:hidden !important)"#, true),
        (r#"not (visibility:collapse)"#, true),
        (r#"(visibility:hidden) and (visibility:visible)"#, true),
    ] {
        assert_eq!(supports_matches(condition), expected, "{condition}");
    }
    assert_nodes(
        r#"<i id=yes></i><i id=no></i><i id=escaped></i>"#,
        r#"@supports (visibility:hidden){#yes{visibility:hidden}}@supports (visibility:collapse){#no{visibility:hidden}}@supports (vi\73 ibility:\68 idden){#escaped{visibility:hidden}}"#,
        &[
            ("yes", Visibility::Hidden),
            ("no", Visibility::Visible),
            ("escaped", Visibility::Hidden),
        ],
    );
}
