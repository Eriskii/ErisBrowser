use super::*;

#[test]
fn mixed_css_wide_background_components_have_no_shorthand_serialization() {
    for keyword in ["inherit", "initial", "unset", "revert", "revert-layer"] {
        for property in ["background-color", "background-image"] {
            let mut style =
                InlineStyle::parse("background:linear-gradient(red) lime", &mut 1_000_000).unwrap();
            assert!(
                style
                    .set_property(property, keyword, "", &mut 1_000_000)
                    .unwrap()
            );
            assert_eq!(
                style.get_property_value(property, &mut 1_000_000).unwrap(),
                keyword
            );
            assert_eq!(
                style
                    .get_property_value("background", &mut 1_000_000)
                    .unwrap(),
                ""
            );
            assert!(
                style
                    .set_property("background", keyword, "", &mut 1_000_000)
                    .unwrap()
            );
            assert_eq!(
                style
                    .get_property_value("background", &mut 1_000_000)
                    .unwrap(),
                keyword
            );
        }
    }
}

#[test]
fn background_cssom_keeps_both_components_and_rejects_invalid_later_values() {
    let mut style =
        InlineStyle::parse("background:linear-gradient(red,blue) lime", &mut 1_000_000).unwrap();
    let get = |style: &InlineStyle, name| style.get_property_value(name, &mut 1_000_000).unwrap();
    assert_eq!(get(&style, "background-image"), "linear-gradient(red,blue)");
    assert_eq!(get(&style, "background-color"), "lime");
    let combined = get(&style, "background");
    assert!(combined.contains("linear-gradient(red,blue)") && combined.contains("lime"));
    for value in [
        "linear-gradient(red,) orange",
        "linear-gradient(red) repeat",
        "url(x) lime",
        "linear-gradient(red),blue",
    ] {
        assert!(
            !style
                .set_property("background", value, "", &mut 1_000_000)
                .unwrap()
        );
        assert_eq!(get(&style, "background"), combined);
    }
    assert!(
        style
            .set_property("background", "blue", "", &mut 1_000_000)
            .unwrap()
    );
    assert_eq!(get(&style, "background-image"), "none");
    assert!(
        style
            .set_property(
                "background",
                "linear-gradient(currentColor)",
                "",
                &mut 1_000_000
            )
            .unwrap()
    );
    assert_eq!(get(&style, "background-color"), "transparent");
}

#[test]
fn gradient_cssom_quota_failure_is_atomic_and_exact_budget_succeeds() {
    let initial = InlineStyle::parse("background:lime", &mut 1_000_000).unwrap();
    let value = "linear-gradient(in srgb to right,red calc(2em + 10%),blue 150%) orange";
    let mut probe = initial.clone();
    let mut work = 1_000_000;
    assert!(
        probe
            .set_property("background", value, "important", &mut work)
            .unwrap()
    );
    let cost = 1_000_000 - work;
    let mut exact = initial.clone();
    let mut budget = cost;
    assert!(
        exact
            .set_property("background", value, "important", &mut budget)
            .unwrap()
    );
    assert_eq!(budget, 0);
    let mut short = initial.clone();
    assert_eq!(
        short.set_property("background", value, "important", &mut (cost - 1)),
        Err(Error::Limit)
    );
    assert_eq!(
        short
            .get_property_value("background", &mut 1_000_000)
            .unwrap(),
        initial
            .get_property_value("background", &mut 1_000_000)
            .unwrap()
    );
}

#[test]
fn malformed_gradient_setters_reject_syntax_without_terminal_quota() {
    let original = InlineStyle::parse("background:lime", &mut 1_000_000).unwrap();
    for property in ["background", "background-image"] {
        for value in [
            "linear-gradient(",
            "linear-gradient(red]",
            "linear-gradient([red))",
            "linear-gradient(red,)",
            "linear-gradient(red))",
        ] {
            let mut style = original.clone();
            let mut work = 1_000_000;
            assert_eq!(
                style.set_property(property, value, "", &mut work),
                Ok(false),
                "{property}: {value}"
            );
            assert!(work > 0);
            assert_eq!(style, original);
            assert_eq!(
                css::valid_background_property(property, value, &mut 1_000_000),
                Ok(false)
            );
        }
    }
}
