use super::*;

const ONE_PIXEL_SVG: &str = "<svg id=s width=1 height=1><rect width=1 height=1 fill=red /></svg>";

fn page(html: &str) -> Page {
    Page::from_html(
        Url::parse("https://example.test/page").unwrap(),
        html,
        false,
    )
}

fn raster(width: u32, height: u32) -> Arc<RasterImage> {
    assert!((1..=4096).contains(&width) && (1..=4096).contains(&height));
    Arc::new(RasterImage {
        width,
        height,
        rgba: vec![0; width as usize * height as usize * 4],
    })
}

fn session() -> PageResources {
    PageResources::new(Fetcher::for_document(
        &Url::parse("https://example.test/page").unwrap(),
    ))
}

#[test]
fn session_moves_the_original_fetcher_and_starts_only_new_image_ledgers() {
    let fetcher = Fetcher::for_document(&Url::parse("https://example.test/page").unwrap());
    let before = fetcher.accounting_snapshot();
    let resources = PageResources::new(fetcher);
    assert_eq!(resources.fetcher.accounting_snapshot(), before);
    assert!(resources.image_cache.is_empty());
    assert!(resources.failed.is_empty());
    assert_eq!(resources.decoded_image_bytes, 0);
    assert_eq!(resources.resource_url_work_left, 33_554_432);
    let mut page = page("");
    assert!(page.resources.is_none());
    page.resources = Some(resources);
    assert_eq!(
        page.resources
            .as_ref()
            .unwrap()
            .fetcher
            .accounting_snapshot(),
        before
    );
}

#[test]
fn live_bytes_count_allocations_across_aliases_and_cache_without_refunding_debits() {
    let mut page = page("");
    let first = raster(2, 1);
    let equal_pixels = raster(2, 1);
    let cache_only = raster(1, 1);
    assert_eq!(first.rgba, equal_pixels.rgba);
    assert!(!Arc::ptr_eq(&first, &equal_pixels));
    page.images.insert("first".into(), first.clone());
    page.images.insert("first-alias".into(), first.clone());
    page.images.insert("equal-distinct".into(), equal_pixels);
    assert_eq!(page.retained_image_bytes(), 16);
    let mut resources = session();
    resources
        .image_cache
        .insert("https://example.test/first".into(), (first, false));
    resources
        .image_cache
        .insert("https://example.test/hidden".into(), (cache_only, true));
    resources
        .failed
        .insert((ResourceKind::Image, "https://example.test/failed".into()));
    resources.decoded_image_bytes = 12;
    resources.resource_url_work_left = 33_554_421;
    page.resources = Some(resources);
    assert_eq!(page.retained_image_bytes(), 20);
    page.images.clear();
    assert_eq!(page.retained_image_bytes(), 12);
    page.refresh_inline_svg();
    assert_eq!(page.retained_image_bytes(), 12);
    let resources = page.resources.as_ref().unwrap();
    assert_eq!(resources.decoded_image_bytes, 12);
    assert_eq!(resources.resource_url_work_left, 33_554_421);
    assert_eq!(resources.failed.len(), 1);
}

#[test]
fn alias_publication_keeps_raw_keys_natural_dimensions_and_original_taint() {
    let mut page = page("<img id=a><img id=b><div id=c></div>");
    let a = page.document.query_selector("#a").unwrap();
    let b = page.document.query_selector("#b").unwrap();
    let c = page.document.query_selector("#c").unwrap();
    let image = raster(2, 1);
    let mut resources = session();
    resources.image_cache.insert(
        "https://example.test/Tile.PNG".into(),
        (image.clone(), false),
    );
    resources.decoded_image_bytes = 8;
    page.resources = Some(resources);
    page.attach_image(a, "./Tile.PNG".into(), image.clone(), false);
    page.attach_image(b, "Tile.PNG#part".into(), image.clone(), false);
    assert_eq!(page.images.len(), 2);
    assert!(Arc::ptr_eq(&page.images["./Tile.PNG"], &image));
    assert!(Arc::ptr_eq(&page.images["Tile.PNG#part"], &image));
    assert!(!page.images.contains_key("https://example.test/Tile.PNG"));
    assert_eq!(page.image_origin_clean("./Tile.PNG"), Some(false));
    assert_eq!(page.image_origin_clean("Tile.PNG#part"), Some(false));
    assert_eq!(page.image_origin_clean("unknown"), None);
    for node in [a, b] {
        assert_eq!(
            page.document.attr(node, "data-eris-natural-width"),
            Some("2")
        );
        assert_eq!(
            page.document.attr(node, "data-eris-natural-height"),
            Some("1")
        );
    }
    assert!(page.document.attr(c, "data-eris-natural-width").is_none());
    assert_eq!(page.retained_image_bytes(), 8);
    assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 8);
}

#[test]
fn cached_rasters_outlive_removed_aliases_and_release_with_the_page() {
    let mut page = page("");
    let image = raster(1, 1);
    let weak = Arc::downgrade(&image);
    let mut resources = session();
    resources
        .image_cache
        .insert("https://example.test/a.png".into(), (image.clone(), true));
    resources.decoded_image_bytes = 4;
    page.resources = Some(resources);
    page.images.insert("a.png".into(), image);
    page.images.clear();
    assert!(weak.upgrade().is_some());
    assert_eq!(page.retained_image_bytes(), 4);
    drop(page);
    assert!(weak.upgrade().is_none());
}

#[test]
fn svg_replacement_reuses_live_memory_without_changing_external_decode_lifetime() {
    let mut page = page(ONE_PIXEL_SVG);
    let svg = page.document.query_selector("#s").unwrap();
    let key = format!("eris-inline-svg:{svg}");
    assert_eq!(page.images[&key].rgba, [255, 0, 0, 255]);
    let old = Arc::downgrade(&page.images[&key]);
    let mut resources = session();
    resources
        .image_cache
        .insert("https://example.test/a.png".into(), (raster(1, 1), true));
    resources.decoded_image_bytes = 4;
    page.resources = Some(resources);
    assert_eq!(page.retained_image_bytes(), 8);
    page.refresh_inline_svg();
    assert!(old.upgrade().is_none());
    assert_eq!(page.images[&key].rgba, [255, 0, 0, 255]);
    assert_eq!(page.retained_image_bytes(), 8);
    assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 4);
    let parent = page.document.nodes[svg].parent.unwrap();
    page.document.remove_child(parent, svg);
    page.refresh_inline_svg();
    assert!(!page.images.contains_key(&key));
    assert_eq!(page.image_origin_clean(&key), None);
    assert_eq!(page.retained_image_bytes(), 4);
    assert_eq!(page.resources.as_ref().unwrap().decoded_image_bytes, 4);
}

#[test]
fn valid_rgba_cache_boundaries_leave_four_or_zero_bytes_for_inline_svg() {
    assert_eq!(MAX_DECODED_IMAGE_BYTES, 67_108_864);
    for room in [4, 0] {
        let mut page = page(ONE_PIXEL_SVG);
        let svg = page.document.query_selector("#s").unwrap();
        let key = format!("eris-inline-svg:{svg}");
        // Prepare the same state as loading external images before SVG refresh.
        page.images.clear();
        page.image_origins.clear();
        let mut resources = session();
        if room == 4 {
            let wide = raster(4095, 4096);
            let narrow = raster(1, 4095);
            assert_eq!(wide.rgba.len(), 67_092_480);
            assert_eq!(narrow.rgba.len(), 16_380);
            resources
                .image_cache
                .insert("https://example.test/wide".into(), (wide, true));
            resources.image_cache.insert(
                "https://example.test/narrow".into(),
                (narrow.clone(), false),
            );
            page.images.insert("narrow".into(), narrow.clone());
            page.images.insert("narrow-alias".into(), narrow);
            resources.decoded_image_bytes = 67_108_860;
        } else {
            resources.image_cache.insert(
                "https://example.test/full".into(),
                (raster(4096, 4096), false),
            );
            resources.decoded_image_bytes = 67_108_864;
        }
        page.resources = Some(resources);
        assert_eq!(page.retained_image_bytes(), 67_108_864 - room);
        for _ in 0..2 {
            page.refresh_inline_svg();
            assert_eq!(
                page.resources.as_ref().unwrap().decoded_image_bytes,
                67_108_864 - room
            );
            assert_eq!(page.retained_image_bytes(), 67_108_864);
            if room == 4 {
                assert_eq!(page.images[&key].rgba, [255, 0, 0, 255]);
                assert!(page.diagnostics.is_empty());
            } else {
                assert!(!page.images.contains_key(&key));
                assert_eq!(page.image_origin_clean(&key), None);
                assert!(
                    page.diagnostics
                        .iter()
                        .all(|d| d == "inline SVG: decoded image budget exhausted")
                );
                assert!(!page.diagnostics.is_empty());
            }
        }
    }
}
