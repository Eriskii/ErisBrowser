use super::*;
const WORK: usize = 4_000_000;
fn get(style: &InlineStyle, name: &str) -> String {
    style.get_property_value(name, &mut WORK.clone()).unwrap()
}
#[test]
fn urls_preserve_authored_payload_case_and_atomic_five_longhand_roundtrip() {
    let mut s = InlineStyle::parse(
        "background:url(A/**/Tile.PNG) center / 8px 4px no-repeat red",
        &mut WORK.clone(),
    )
    .unwrap();
    assert_eq!(get(&s, "background-image"), "url(A/**/Tile.PNG)");
    assert_eq!(get(&s, "background-size"), "8px 4px");
    assert_eq!(get(&s, "background-repeat"), "no-repeat");
    let text = s.serialize(&mut WORK.clone()).unwrap();
    assert_eq!(InlineStyle::parse(&text, &mut WORK.clone()).unwrap(), s);
    let before = s.clone();
    for value in [
        "url(A),,url(B)",
        "url(A) url(B)",
        "url(A B)",
        "url(A);color:blue",
    ] {
        assert_eq!(
            s.set_property("background", value, "", &mut WORK.clone()),
            Ok(false)
        );
        assert_eq!(s, before);
    }
    assert_eq!(
        s.set_property("background", "url(New.PNG)", "", &mut WORK.clone()),
        Ok(true)
    );
    assert_eq!(get(&s, "background-size"), "auto auto");
    assert_eq!(get(&s, "background-position"), "0% 0%");
    assert_eq!(get(&s, "background-repeat"), "repeat");
}
#[test]
fn url_longhand_keeps_missing_and_mixed_wide_shorthand_empty() {
    let mut s = InlineStyle::default();
    assert_eq!(
        s.set_property("background-image", "url(A.PNG)", "", &mut WORK.clone()),
        Ok(true)
    );
    assert_eq!(get(&s, "background"), "");
    assert_eq!(
        s.set_property("background", "none", "", &mut WORK.clone()),
        Ok(true)
    );
    assert_eq!(
        s.set_property(
            "background-image",
            r"u\72l(A\)B.PNG)",
            "",
            &mut WORK.clone()
        ),
        Ok(true)
    );
    assert!(!get(&s, "background").is_empty());
    assert_eq!(
        s.set_property("background-color", "inherit", "", &mut WORK.clone()),
        Ok(true)
    );
    assert_eq!(get(&s, "background"), "");
}
#[test]
fn escaped_and_quoted_url_payloads_do_not_create_style_records() {
    let s = InlineStyle::parse(
        "background-image:url('A;!{B}.PNG');color:red",
        &mut WORK.clone(),
    )
    .unwrap();
    assert_eq!(get(&s, "background-image"), "url('A;!{B}.PNG')");
    assert_eq!(get(&s, "color"), "red");
    assert_eq!(get(&s, "width"), "");
}
