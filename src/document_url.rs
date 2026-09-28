//! Document-relative URL parsing. A base element changes resolution, never origin.
use crate::dom::Document;
use url::Url;

pub(crate) fn parse(document: &Document, base: &Url, input: &str) -> Result<Url, url::ParseError> {
    let encoding = encoding_rs::Encoding::for_label(document.character_set().as_bytes())
        .unwrap_or(encoding_rs::UTF_8)
        .output_encoding();
    let encode: &dyn Fn(&str) -> std::borrow::Cow<'_, [u8]> = &|value| encoding.encode(value).0;
    Url::options()
        .base_url(Some(base))
        .encoding_override(Some(encode))
        .parse(input)
}

pub(crate) fn base_url(document: &Document) -> Url {
    document.base_url().clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_connected_html_base_controls_resolution_without_changing_document_url() {
        let mut document = Document::parse(
            "<template><base href='/inert/'></template><base href='../assets/'><base href='/ignored/'>",
        );
        document.initialize_url(Url::parse("https://example.test/docs/page.html").unwrap());
        assert_eq!(base_url(&document).as_str(), "https://example.test/assets/");
        assert_eq!(
            document.url().as_str(),
            "https://example.test/docs/page.html"
        );
        let first = document.query_selector("base").unwrap();
        document.set_attr(first, "href", "javascript:alert(1)");
        assert_eq!(base_url(&document), *document.url());
        document.remove_attr(first, "href");
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/ignored/"
        );
    }
    #[test]
    fn legacy_document_encoding_affects_query_but_not_path() {
        let mut document = Document::parse("");
        document.set_encoding(encoding_rs::WINDOWS_1252);
        let base = Url::parse("https://example.test/").unwrap();
        assert_eq!(
            parse(&document, &base, "é?q=é").unwrap().as_str(),
            "https://example.test/%C3%A9?q=%E9"
        );
    }
    #[test]
    fn frozen_base_survives_document_url_and_later_base_changes() {
        let mut document =
            Document::parse("<base id=first href='javascript:x'><base id=later href='later'>");
        document.initialize_url(Url::parse("https://example.test/first/page#old").unwrap());
        let first = document.query_selector("#first").unwrap();
        let later = document.query_selector("#later").unwrap();
        document.set_url(Url::parse("https://example.test/first/page#new").unwrap());
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/first/page#old"
        );
        document.set_attr(later, "href", "changed");
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/first/page#old"
        );
        document.set_attr(first, "href", "javascript:x");
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/first/page#new"
        );
        document.set_attr(first, "href", "");
        document.set_url(Url::parse("https://example.test/second/page#other").unwrap());
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/first/page"
        );
        document.remove_attr(first, "href");
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/second/changed"
        );
    }
    #[test]
    fn structural_mutations_freeze_only_the_affected_first_base() {
        let mut document = Document::parse(
            "<head><base id=first href='javascript:x'></head><body><section><base id=later href='/second/'></section><div><span></span></div>",
        );
        document.initialize_url(Url::parse("https://example.test/page#old").unwrap());
        let first = document.query_selector("#first").unwrap();
        let later = document.query_selector("#later").unwrap();
        let head = document.query_selector("head").unwrap();
        let body = document.query_selector("body").unwrap();
        let section = document.query_selector("section").unwrap();
        let div = document.query_selector("div").unwrap();
        document.set_url(Url::parse("https://example.test/page#new").unwrap());
        document.clear_children(div);
        document.append_child(body, div);
        document.set_attr(first, "id", "renamed");
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/page#old"
        );
        document.append_child(head, first);
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/page#new"
        );
        document.append_child(body, first);
        assert_eq!(document.frozen_base().unwrap().0, later);
        assert_eq!(base_url(&document).as_str(), "https://example.test/second/");
        document.remove_child(body, section);
        assert_eq!(document.frozen_base().unwrap().0, first);
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/page#new"
        );
        document.set_text_content(body, "removed");
        assert!(document.frozen_base().is_none());
        assert_eq!(base_url(&document), *document.url());
    }
    #[test]
    fn fragments_and_template_transfers_update_connected_base_identity() {
        let mut document =
            Document::parse("<template><base href='/inert/'></template><body><div></div>");
        document.initialize_url(Url::parse("https://example.test/page").unwrap());
        let body = document.query_selector("body").unwrap();
        let template = document.query_selector("template").unwrap();
        let content = document.template_contents(template).unwrap();
        assert!(document.frozen_base().is_none());
        document.append_child(body, content);
        assert_eq!(base_url(&document).as_str(), "https://example.test/inert/");
        let fragment = document.create_document_fragment();
        let base = document.create_element("base");
        document.set_attr(base, "href", "/fragment/");
        document.append_child(fragment, base);
        document.clear_children(body);
        document.append_child(body, fragment);
        assert_eq!(document.frozen_base().unwrap().0, base);
        assert_eq!(
            base_url(&document).as_str(),
            "https://example.test/fragment/"
        );
    }
    #[test]
    fn frozen_snapshot_restore_checks_identity_and_disallowed_schemes() {
        let mut document = Document::parse("<base href='javascript:x'><base href='/other/'>");
        document.initialize_url(Url::parse("data:text/html,example#new").unwrap());
        let first = document.frozen_base().unwrap().0;
        assert!(
            document
                .restore_frozen_base(first, Url::parse("data:text/html,example#old").unwrap())
                .is_ok()
        );
        assert!(
            document
                .restore_frozen_base(first, Url::parse("data:text/html,different").unwrap())
                .is_err()
        );
        assert!(
            document
                .restore_frozen_base(first, Url::parse("javascript:alert(1)").unwrap())
                .is_err()
        );
        assert!(
            document
                .restore_frozen_base(document.root, Url::parse("https://example.test/").unwrap())
                .is_err()
        );
        let second = document.query_selector_all("base")[1];
        assert!(
            document
                .restore_frozen_base(second, Url::parse("https://example.test/").unwrap())
                .is_err()
        );
    }
    #[test]
    fn base_work_estimates_skip_unrelated_mutations_and_cover_retained_href_bytes() {
        let mut document = Document::parse("<base href='/first/'><body><div><span></span></div>");
        document.initialize_url(Url::parse("https://example.test/").unwrap());
        let base = document.query_selector("base").unwrap();
        let body = document.query_selector("body").unwrap();
        let div = document.query_selector("div").unwrap();
        assert_eq!(document.base_tree_change_work(body, div, false), 0);
        assert_eq!(document.base_clear_work(div), 0);
        assert_eq!(document.base_remove_work(div), 0);
        assert_eq!(document.base_attribute_work(base, "id"), 0);
        assert!(
            document.base_attribute_work(base, "href") >= document.nodes.len() + "/first/".len()
        );
        let detached = document.create_element("base");
        document.set_attr(detached, "href", &"a".repeat(10_000));
        assert_eq!(document.base_attribute_work(detached, "href"), 0);
        assert!(
            document.base_tree_change_work(body, detached, true) >= document.nodes.len() + 10_000
        );
        document.remove_attr(detached, "href");
        assert!(document.base_attribute_work(base, "href") < 10_000);
    }
}
