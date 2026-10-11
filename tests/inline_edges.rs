//! Ordinary inline horizontal edges through direct Page layout and CSSOM click.
#[path = "support/inline_edges.rs"]
mod witness;

use eris::{graphics::Fonts, page::Page};
use url::Url;

fn healthy(page: &Page) {
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    assert!(
        page.runtime.console.is_empty(),
        "{:?}",
        page.runtime.console
    );
}

#[test]
fn inline_padding_mutation_and_resize_keep_literal_child_positions_and_identity() {
    let mut page = Page::from_html(
        Url::parse("about:blank").unwrap(),
        witness::MUTATION_HTML,
        true,
    );
    let fonts = Fonts::new();
    let mut previous: Option<witness::Identity> = None;
    for changed in [false, true] {
        if changed {
            assert!(page.click(previous.as_ref().unwrap().red).is_none());
        }
        for width in [32, 10, 32] {
            healthy(&page);
            let layout = page.layout(width as f32, witness::HEIGHT as f32, &fonts);
            previous = Some(witness::mutation(
                &page.document,
                &layout,
                &page.images,
                &fonts,
                width,
                changed,
                previous.as_ref(),
            ));
        }
    }
}

#[test]
fn inline_nested_and_trimmed_edges_preserve_literal_following_block_flow() {
    let page = Page::from_html(
        Url::parse("about:blank").unwrap(),
        witness::BOUNDARIES_HTML,
        false,
    );
    let fonts = Fonts::new();
    let mut previous = None;
    for _ in 0..2 {
        healthy(&page);
        let layout = page.layout(24.0, witness::HEIGHT as f32, &fonts);
        previous = Some(witness::boundaries(
            &page.document,
            &layout,
            &page.images,
            &fonts,
            previous.as_ref(),
        ));
    }
}
