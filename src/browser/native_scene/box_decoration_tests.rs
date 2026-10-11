use super::*;

#[test]
fn unsupported_box_decoration_refuses_scaled_payload_copies() {
    let input = [super::super::box_decoration_tests::command()];
    for zoom in [1.0, 1.0005] {
        assert!(matches!(
            scaled_page(&input, zoom).unwrap(),
            Cow::Borrowed(_)
        ));
    }
    assert!(
        scaled_page(&input, 2.0)
            .unwrap_err()
            .contains("box decoration")
    );
}
