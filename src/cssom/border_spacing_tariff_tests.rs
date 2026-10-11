use super::*;

#[test]
fn spacing_longhand_parser_uses_the_cssom_callers_shared_remainder() {
    // Existing longhand validation12*n+1=85, shared parser59; no local budget.
    let mut exact = 144;
    assert_eq!(
        valid_longhand("border-spacing", "2px 3px", &mut exact),
        Ok(true)
    );
    assert_eq!(exact, 0);
    let mut short = 143;
    assert_eq!(
        valid_longhand("border-spacing", "2px 3px", &mut short),
        Err(Error::Limit)
    );
    assert_eq!(short, 0);
    for value in ["calc(1px", "[1px)", "calc(1px +)", "calc(0% + 2px)"] {
        let mut work = 100_000;
        assert_eq!(
            valid_longhand("border-spacing", value, &mut work),
            Ok(false),
            "{value}"
        );
        assert!(work > 0);
    }
}
