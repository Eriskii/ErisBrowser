//! Per-layer preparation shares the existing page session and cache authority.
use super::*;

#[test]
fn aliases_retain_none_slots_share_raster_and_keep_readonly_fetch_free() {
    with_page(
        "<style>html,body{margin:0}#a{width:6px;height:1px;background-image:url(tile.png#one),none,url(./tile.png#two);background-size:2px 1px;background-position:0 0,2px 0,4px 0;background-repeat:no-repeat}</style><div id=a></div>",
        |page| {
            let fonts = Fonts::new();
            assert!(keys(&page.layout(80.0, 60.0, &fonts)).is_empty());
            assert_eq!(requests(), ["https://page.test/start"]);
            let result = page.prepare_layout(80.0, 60.0, &fonts);
            let selected = keys(&result);
            assert_eq!(selected.len(), 2);
            assert_eq!(selected[0], selected[1]);
            assert_eq!(page.image_origin_clean(selected[0]), Some(false));
            assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 8);
            let mut canvas = crate::graphics::Canvas::new(80, 60).unwrap();
            canvas.clear(crate::graphics::Color::WHITE);
            canvas.paint(&result.commands, &fonts, &page.images, 0.0, 0.0);
            assert_eq!(
                &canvas.pixels[..6],
                &[0xff0000, 0x00ff00, 0xffffff, 0xffffff, 0xff0000, 0x00ff00]
            );
            let cached = page
                .resources
                .as_ref()
                .unwrap()
                .fetcher
                .accounting_snapshot();
            assert_eq!(keys(&page.layout(100.0, 60.0, &fonts)).len(), 2);
            assert_eq!(keys(&page.prepare_layout(100.0, 60.0, &fonts)).len(), 2);
            assert_eq!(
                cached,
                page.resources
                    .as_ref()
                    .unwrap()
                    .fetcher
                    .accounting_snapshot()
            );
            assert_eq!(
                requests(),
                [
                    "https://page.test/start",
                    "https://page.test/final/tile.png"
                ]
            );
        },
    );
}

#[test]
fn failed_layer_occurrences_share_negative_cache_but_keep_session_debits() {
    with_page(
        "<div style='width:6px;height:1px;background-image:url(missing.png),url(tile.png),url(missing.png)'></div>",
        |page| {
            let fonts = Fonts::new();
            let result = page.prepare_layout(80.0, 60.0, &fonts);
            assert_eq!(keys(&result).len(), 1);
            assert_eq!(
                requests(),
                [
                    "https://page.test/start",
                    "https://page.test/final/missing.png",
                    "https://page.test/final/tile.png"
                ]
            );
            let resources = page.resources.as_ref().unwrap();
            let before = (
                resources.resource_url_work_left,
                resources.background_metadata_left,
            );
            let diagnostic_count = page.diagnostics.len();
            assert_eq!(keys(&page.prepare_layout(80.0, 60.0, &fonts)).len(), 1);
            let resources = page.resources.as_ref().unwrap();
            // The 35-byte missing URL is charged once for each of its two slots.
            assert_eq!(before.0 - resources.resource_url_work_left, 816);
            assert_eq!(before.1 - resources.background_metadata_left, 1234);
            assert_eq!(resources.failed.len(), 1);
            assert_eq!(page.diagnostics.len(), diagnostic_count);
            assert_eq!(requests().len(), 3);
        },
    );
}

#[test]
fn corrupt_published_alias_invalidates_every_borrowed_layer_then_repairs_once() {
    with_page(
        "<div style='width:6px;height:1px;background-image:url(tile.png),none,url(tile.png#other)'></div>",
        |page| {
            let fonts = Fonts::new();
            let first = page.prepare_layout(80.0, 60.0, &fonts);
            let key = keys(&first)[0].to_owned();
            let stranger = Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![0, 0, 255, 255],
            });
            page.images.insert(key.clone(), stranger.clone());
            assert!(keys(&page.layout(80.0, 60.0, &fonts)).is_empty());
            let repaired = page.prepare_layout(80.0, 60.0, &fonts);
            assert_eq!(
                keys(&repaired),
                ["eris-css-background:1", "eris-css-background:1"]
            );
            assert!(Arc::ptr_eq(&page.images[&key], &stranger));
            assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 8);
            assert_eq!(
                page.image_origin_clean("eris-css-background:1"),
                Some(false)
            );
            assert_eq!(requests().len(), 2);
        },
    );
}

#[test]
fn every_layer_uses_original_shared_request_limit() {
    let mut html = String::new();
    for i in 0..47 {
        html.push_str(&format!("<img src='/{i}.png'>"));
    }
    html.push_str(
        "<div style='height:2px;background-image:url(a.png),none,url(b.png),url(a.png)'></div>",
    );
    with_page(&html, |page| {
        let before = page
            .resources
            .as_ref()
            .unwrap()
            .fetcher
            .accounting_snapshot();
        assert_eq!(requests().len(), 48);
        assert!(keys(&page.prepare_layout(80.0, 60.0, &Fonts::new())).is_empty());
        assert_eq!(requests().len(), 48);
        assert_eq!(
            before,
            page.resources
                .as_ref()
                .unwrap()
                .fetcher
                .accounting_snapshot()
        );
        let failures = &page.resources.as_ref().unwrap().failed;
        assert!(failures.contains(&(ResourceKind::Image, "https://page.test/final/a.png".into())));
        assert!(failures.contains(&(ResourceKind::Image, "https://page.test/final/b.png".into())));
    });
}
