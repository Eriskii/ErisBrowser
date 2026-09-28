use eris::script::Runtime;

#[test]
fn declaration_name_validation_preserves_syntax_and_diagnostic_order() {
    let mut count = 0;
    for line in include_str!("fixtures/scope-names.tsv").lines() {
        if line.starts_with('#') {
            continue;
        }
        let columns: Vec<_> = line.splitn(4, '\t').collect();
        assert_eq!(columns.len(), 4);
        let [id, expected, message, source] = columns.as_slice() else {
            unreachable!()
        };
        let (_, mode) = id.rsplit_once('/').unwrap();
        let result = match mode {
            "strict" => Runtime::parse_only_strict(source),
            "sloppy" => Runtime::parse_only(source),
            _ => panic!("unknown mode in {id}"),
        };
        match *expected {
            "complete" => result.unwrap_or_else(|error| panic!("{id}: {error}")),
            "exception" => {
                let error = result.unwrap_err();
                assert!(error.is_parse_error(), "{id}: {error}");
                assert_eq!(error.name(), "SyntaxError", "{id}");
                assert_eq!(error.message, *message, "{id}");
            }
            _ => panic!("unknown expectation in {id}"),
        }
        count += 1;
    }
    assert_eq!(count, 144);
}
