use eris::{
    graphics::{Canvas, Color, DrawCommand, Fonts},
    page::Page,
};
use url::Url;

fn page(source: &str) -> Page {
    Page::from_html(
        Url::parse("https://example.test/base/page").unwrap(),
        source,
        true,
    )
}

#[test]
fn inline_style_mutations_preserve_values_priorities_and_visible_output() {
    assert_six_scripted_samples(include_str!("fixtures/inline-style.html"));
}

#[test]
fn default_parameters_preserve_scope_and_run_in_event_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/default-parameters.html"));
}

#[test]
fn rest_parameter_arrays_preserve_scope_and_run_in_event_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/rest-parameters.html"));
}

#[test]
fn prototype_membership_observes_identity_and_event_mutations() {
    assert_six_scripted_samples(include_str!("fixtures/prototype-membership.html"));
}

#[test]
fn replaceable_window_self_preserves_lexical_and_event_identity() {
    assert_six_scripted_samples(include_str!("fixtures/window-self.html"));
}

#[test]
fn global_value_properties_preserve_private_realm_and_event_identity() {
    assert_six_scripted_samples(include_str!("fixtures/global-values.html"));
}

#[test]
fn stable_array_sort_preserves_holes_identity_and_callback_effects() {
    assert_six_scripted_samples(include_str!("fixtures/array-sort.html"));
}

#[test]
fn unicode_identifier_bindings_survive_rendering_and_event_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/identifiers.html"));
}

#[test]
fn array_reductions_preserve_direction_live_values_and_accumulator_identity() {
    assert_six_scripted_samples(include_str!("fixtures/array-reduce.html"));
}

#[test]
fn labeled_jumps_preserve_finalizers_and_callback_pixels() {
    assert_six_scripted_samples(include_str!("fixtures/labels.html"));
}

#[test]
fn for_of_protocols_bindings_and_closing_survive_document_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/for-of.html"));
}

#[test]
fn array_from_mapping_construction_and_closing_survive_document_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/array-from.html"));
}

#[test]
fn array_splice_preserves_species_live_properties_and_partial_callback_effects() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/array-splice.html"),
        Some(["ready", "6"]),
    );
}

#[test]
fn array_concat_preserves_spreadability_aliases_and_partial_callback_effects() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/array-concat.html"),
        Some(["ready", "6"]),
    );
}

#[test]
fn array_buffer_resizing_transfer_and_species_survive_document_callbacks() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/array-buffer.html"),
        Some(["ready", "6"]),
    );
}

#[test]
fn data_view_bytes_resize_and_transfer_survive_document_callbacks() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/data-view.html"),
        Some(["ready", "6"]),
    );
}

#[test]
fn object_is_same_value_and_saved_identity_survive_document_callbacks() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/object-is.html"),
        Some(["ready", "6"]),
    );
}

#[test]
fn dom_defining_interface_identity_and_brands_survive_document_callbacks() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/dom-method-identity.html"),
        Some(["ready", "6"]),
    );
}

#[test]
fn own_key_order_and_live_enumeration_survive_document_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/own-keys.html"));
}

#[test]
fn equality_conversions_preserve_nullish_rules_and_callback_pixels() {
    assert_six_scripted_samples(include_str!("fixtures/equality.html"));
}

#[test]
fn relational_conversions_preserve_utf16_order_and_callback_pixels() {
    assert_six_scripted_samples(include_str!("fixtures/relational.html"));
}

#[test]
fn uri_transforms_preserve_unicode_errors_and_callback_pixels() {
    assert_six_scripted_samples(include_str!("fixtures/uri.html"));
}

#[test]
fn logical_assignment_preserves_short_circuit_and_callback_pixels() {
    assert_six_scripted_samples(include_str!("fixtures/logical-assignment.html"));
}

#[test]
fn addition_preserves_conversions_utf16_and_compound_callback_pixels() {
    assert_six_scripted_samples(include_str!("fixtures/addition.html"));
}

#[test]
fn compound_bitwise_assignments_preserve_pixels_and_reference_order() {
    assert_six_scripted_samples(include_str!("fixtures/compound-assignment.html"));
}

#[test]
fn numeric_parsing_preserves_prefixes_conversion_order_and_alias_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/numeric-parsing.html"));
}

#[test]
fn numeric_conversion_preserves_hooks_boxes_and_saved_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/numeric-conversion.html"));
}

#[test]
fn number_statics_preserve_boundaries_and_native_aliases_in_callbacks() {
    assert_six_scripted_samples(include_str!("fixtures/number-statics.html"));
}

#[test]
fn invalid_identifier_scripts_leave_no_effects_and_later_scripts_still_run() {
    for invalid in [
        r"var \u0069f=1;",
        r"var \uD801\uDC00=1;",
        r"var \u0030name=1;",
        r"var x=3in {};",
        "var\u{0085}name=1;",
    ] {
        let source = format!(
            "<!doctype html><body><script>document.body.className='wrong';{invalid}</script>\
             <script>if(document.body.className===''){{let \\u03C0=3;if(π===3)document.body.className='ready';}}</script>"
        );
        let p = page(&source);
        assert_eq!(p.diagnostics.len(), 1, "{invalid}: {:?}", p.diagnostics);
        let body = p.document.query_selector("body").unwrap();
        assert_eq!(p.document.attr(body, "class"), Some("ready"), "{invalid}");
    }
}

#[test]
fn window_self_state_survives_fragment_navigation_and_resets_in_a_new_page() {
    let source = include_str!("fixtures/window-self.html");
    let mut p = page(source);
    let mut destination = p.url.clone();
    destination.set_fragment(Some("replacement"));
    assert!(p.navigate_fragment(destination.clone()));
    assert_eq!(p.url, destination);
    let button = p.document.query_selector("#change").unwrap();
    assert!(p.click(button).is_none());
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let body = p.document.query_selector("body").unwrap();
    assert_eq!(p.document.attr(body, "class"), Some("clicked"));
    let fresh = Page::from_html(destination, source, true);
    assert!(fresh.diagnostics.is_empty(), "{:?}", fresh.diagnostics);
    let body = fresh.document.query_selector("body").unwrap();
    assert_eq!(fresh.document.attr(body, "class"), Some("ready"));
}

#[test]
fn rejected_self_declaration_does_not_leave_bindings_for_later_page_scripts() {
    let p = page(
        "<!doctype html><body><script>Object.defineProperty(window,'self',{value:3,writable:false,configurable:false});</script>\
        <script>let rejectedLexical=1;var rejectedVar=2;function rejectedFunction(){}function self(){}document.body.className='wrong';</script>\
        <script>if(typeof rejectedLexical==='undefined'&&typeof rejectedVar==='undefined'&&typeof rejectedFunction==='undefined'&&self===3)document.body.className='ready';</script>",
    );
    assert_eq!(p.diagnostics.len(), 1, "{:?}", p.diagnostics);
    let body = p.document.query_selector("body").unwrap();
    assert_eq!(p.document.attr(body, "class"), Some("ready"));
}

fn assert_six_scripted_samples(source: &str) {
    assert_six_scripted_samples_with_result_text(source, None);
}

fn assert_six_scripted_samples_with_result_text(source: &str, result_text: Option<[&str; 2]>) {
    let mut p = page(source);
    let fonts = Fonts::new();
    for (index, (state, color)) in [("ready", 0x008000), ("clicked", 0x0000ff)]
        .into_iter()
        .enumerate()
    {
        if state == "clicked" {
            let button = p.document.query_selector("#change").unwrap();
            assert!(p.click(button).is_none());
        }
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let body = p.document.query_selector("body").unwrap();
        assert_eq!(p.document.attr(body, "class"), Some(state));
        if let Some(expected) = result_text {
            let result = p.document.query_selector("#result").unwrap();
            assert_eq!(p.document.text_content(result), expected[index], "{state}");
        }
        let layout = p.layout(320.0, 240.0, &fonts);
        let mut canvas = Canvas::new(320, 240).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        for x in [10, 60, 110, 160, 210, 260] {
            assert_eq!(canvas.pixels[10 * 320 + x], color, "{state}, x={x}");
        }
    }
}

#[test]
fn mixed_calculations_recompute_after_resize_and_cssom_mutation() {
    let mut p = page(include_str!("fixtures/calc-resize.html"));
    let fonts = Fonts::new();
    let sample = p.document.query_selector("#sample").unwrap();
    for (width, click, x, box_width, color) in [
        (320, false, 45.0, 140.0, 0x008000),
        (520, false, 65.0, 240.0, 0x008000),
        (520, true, 30.0, 130.0, 0x0000ff),
        (320, false, 20.0, 80.0, 0x0000ff),
    ] {
        if click {
            let button = p.document.query_selector("#change").unwrap();
            assert!(p.click(button).is_none());
        }
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let layout = p.layout(width as f32, 240.0, &fonts);
        let rect = layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == sample)
            .unwrap()
            .rect;
        assert_eq!(
            (rect.x, rect.y, rect.width, rect.height),
            (x, 10.0, box_width, 40.0)
        );
        let mut canvas = Canvas::new(width, 240).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        let row = 20 * width as usize;
        assert_eq!(canvas.pixels[row + x as usize], color);
        assert_eq!(canvas.pixels[row + (x + box_width) as usize - 1], color);
        assert_eq!(canvas.pixels[row + (x + box_width) as usize], 0xeeeeee);
    }
}

#[test]
fn custom_properties_recompute_inherited_aliases_after_parent_changes() {
    let mut p = page(include_str!("fixtures/custom-properties.html"));
    let fonts = Fonts::new();
    for (click, color) in [(false, 0x008000), (true, 0x0000ff)] {
        if click {
            let button = p.document.query_selector("#change").unwrap();
            assert!(p.click(button).is_none());
        }
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let layout = p.layout(320.0, 240.0, &fonts);
        let mut canvas = Canvas::new(320, 240).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert_eq!(canvas.pixels[10 * 320 + 10], color);
    }
}

#[test]
fn css_supports_drives_visible_styles_without_crossing_argument_boundaries() {
    let mut p = page(include_str!("fixtures/css-supports.html"));
    let fonts = Fonts::new();
    for (state, color) in [("ready", 0x008000), ("clicked", 0x0000ff)] {
        if state == "clicked" {
            let button = p.document.query_selector("#check").unwrap();
            assert!(p.click(button).is_none());
        }
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let body = p.document.query_selector("body").unwrap();
        assert_eq!(p.document.attr(body, "class"), Some(state));
        let layout = p.layout(320.0, 240.0, &fonts);
        let mut canvas = Canvas::new(320, 240).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        assert_eq!(canvas.pixels[10 * 320 + 10], color);
    }
}

#[test]
fn disclosure_notes_preserve_dom_state_and_reflow_only_the_open_group_member() {
    let mut p = page(include_str!("../examples/disclosures.html"));
    let fonts = Fonts::new();
    for (action, open, title) in [
        (None, Some(1), "The shape of a fern"),
        (Some("#next-note"), Some(2), "A sky in a puddle"),
        (Some("#summary-3"), Some(3), "The colors of lichen"),
        (Some("#summary-3"), None, "The colors of lichen"),
        (Some("#next-note"), Some(1), "The shape of a fern"),
    ] {
        if let Some(selector) = action {
            assert!(
                p.click(p.document.query_selector(selector).unwrap())
                    .is_none()
            );
        }
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let status = p.document.query_selector("#status").unwrap();
        assert_eq!(
            p.document.text_content(status),
            format!("Now reading · {title}")
        );
        for width in [960, 390, 960] {
            let layout = p.layout(width as f32, 1300.0, &fonts);
            for (index, content) in [
                (1, "#fern-content"),
                (2, "#rain-content"),
                (3, "#lichen-content"),
            ] {
                let note = p
                    .document
                    .query_selector(&format!("#note-{index}"))
                    .unwrap();
                assert_eq!(p.document.attr(note, "open").is_some(), open == Some(index));
                let content = p.document.query_selector(content).unwrap();
                assert_eq!(
                    layout.hit_regions.iter().any(|hit| hit.node == content),
                    open == Some(index)
                );
                let summary = p
                    .document
                    .query_selector(&format!("#summary-{index}"))
                    .unwrap();
                assert!(layout.hit_regions.iter().any(|hit| hit.node == summary));
            }
            let mut canvas = Canvas::new(width, 1300).unwrap();
            canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
            assert!(!canvas.exhausted());
            assert_eq!(canvas.pixels[0], 0xf3f1e8);
        }
    }
}

#[test]
fn responsive_notes_reflow_on_resize_and_interpolate_clicked_readings() {
    let mut p = page(include_str!("../examples/responsive.html"));
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let button = p.document.query_selector("#advance").unwrap();
    for (status, next) in [
        ("Reading 2 · midday", "Return at evening"),
        ("Reading 3 · evening", "Return tomorrow morning"),
        ("Reading 4 · morning", "Return at midday"),
    ] {
        assert!(p.click(button).is_none());
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        for (selector, expected) in [("#status", status), ("#next", next)] {
            assert_eq!(
                p.document
                    .text_content(p.document.query_selector(selector).unwrap()),
                expected
            );
        }
    }
    let cards = p.document.query_selector_all(".sample");
    assert_eq!(cards.len(), 6);
    let fonts = Fonts::new();
    for (width, visible) in [
        (960, "#wide"),
        (800, "#medium"),
        (390, "#compact"),
        (960, "#wide"),
    ] {
        let layout = p.layout(width as f32, 900.0, &fonts);
        let boxes = cards
            .iter()
            .map(|node| {
                layout
                    .hit_regions
                    .iter()
                    .find(|hit| hit.node == *node)
                    .unwrap()
                    .rect
            })
            .collect::<Vec<_>>();
        if width >= 640 {
            for pair in boxes.as_chunks::<2>().0 {
                assert_eq!(pair[0].x, pair[1].x);
                assert!((pair[1].y - pair[0].y - 90.0).abs() < 0.01);
            }
            assert!(boxes[0].x < boxes[2].x && boxes[2].x < boxes[4].x);
            assert_eq!(boxes[0].y, boxes[2].y);
            assert_eq!(boxes[0].y, boxes[4].y);
        } else {
            for pair in boxes.windows(2) {
                assert_eq!(pair[0].x, pair[1].x);
                assert!((pair[1].y - pair[0].y - 90.0).abs() < 0.01);
            }
        }
        for selector in ["#wide", "#medium", "#compact"] {
            let node = p.document.query_selector(selector).unwrap();
            assert_eq!(
                layout.hit_regions.iter().any(|hit| hit.node == node),
                selector == visible
            );
        }
        let mut canvas = Canvas::new(width, 900).unwrap();
        canvas.clear(Color::WHITE);
        canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
        assert!(!canvas.exhausted());
        for rect in boxes {
            let x = (rect.x + 2.0) as usize;
            let y = (rect.y + 40.0) as usize;
            if y < 900 {
                assert_eq!(canvas.pixels[y * width as usize + x], 0x203441);
            }
        }
    }
}

#[test]
fn positioning_demo_loads_imports_executes_regexp_and_keeps_fixed_pixels_during_scroll() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/positioning.html");
    let mut p = Page::load(Url::from_file_path(path).unwrap().as_str(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(p.document.url(), &p.url);
    p.click(p.document.query_selector("#extract").unwrap());
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#result").unwrap()),
        "128 · 64 · 3"
    );
    let fonts = Fonts::new();
    let layout = p.layout(960.0, 800.0, &fonts);
    assert!(layout.content_height > 800.0);
    let header = p.document.query_selector(".masthead").unwrap();
    assert!(
        layout
            .hit_regions
            .iter()
            .any(|h| h.node == header && h.fixed)
    );
    assert!(
        layout
            .commands
            .iter()
            .any(|c| matches!(c, DrawCommand::PushFixed))
    );
    let mut before = Canvas::new(960, 800).unwrap();
    let mut after = Canvas::new(960, 800).unwrap();
    before.clear(Color::WHITE);
    after.clear(Color::WHITE);
    before.paint_with_viewport(&layout.commands, &fonts, &p.images, (0.0, 0.0), (0.0, 0.0));
    after.paint_with_viewport(
        &layout.commands,
        &fonts,
        &p.images,
        (0.0, -500.0),
        (0.0, 0.0),
    );
    assert_eq!(before.pixels[10 * 960 + 10], 0x142c35);
    assert_eq!(before.pixels[..72 * 960], after.pixels[..72 * 960]);
    assert_ne!(before.pixels[100 * 960..], after.pixels[100 * 960..]);
    assert!(!before.exhausted() && !after.exhausted());
}
#[test]
fn script_click_flows_through_dom_style_layout_and_pixels() {
    let mut p = page(
        "<style>body{margin:0}#box{width:100px;height:50px;background:red}</style><div id=box></div><button id=go>Change</button><script>document.getElementById('go').addEventListener('click', function(){document.getElementById('box').style.backgroundColor='blue';});</script>",
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let fonts = Fonts::new();
    let mut c = Canvas::new(200, 100).unwrap();
    c.paint(
        &p.layout(200.0, 100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(c.pixels[20 * 200 + 20], 0xff0000);
    p.click(p.document.query_selector("#go").unwrap());
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    c.clear(Color::WHITE);
    c.paint(
        &p.layout(200.0, 100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(c.pixels[20 * 200 + 20], 0x0000ff);
}

#[test]
fn namespace_demo_rebuilds_svg_with_a_prototype_switch_and_paints_each_palette() {
    use eris::dom::Namespace;

    let mut p = Page::from_html(
        Url::parse("https://example.test/namespaces.html").unwrap(),
        include_str!("../examples/namespaces.html"),
        true,
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let initial_svg = p.document.query_selector("#scene").unwrap();
    let initial_key = format!("eris-inline-svg:{initial_svg}");
    let initial_image = &p.images[&initial_key];
    assert_eq!((initial_image.width, initial_image.height), (760, 240));
    // The 380:160 viewBox meets a 760:240 viewport with transparent side bands.
    // This checks that the adjusted preserveAspectRatio attribute reaches SVG paint.
    let side_pixel = (120 * 760 + 40) * 4;
    assert_eq!(&initial_image.rgba[side_pixel..side_pixel + 4], &[0; 4]);
    let center_pixel = (120 * 760 + 215) * 4;
    assert_eq!(
        &initial_image.rgba[center_pixel..center_pixel + 4],
        &[0x5b, 0xd4, 0xba, 255]
    );

    let button = p.document.query_selector("#advance").unwrap();
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(960, 900).unwrap();
    for (step, name, fill, rgb) in [
        (1, "Amber", "#ffc46b", 0xffc46b),
        (2, "Iris", "#b7a5f5", 0xb7a5f5),
        (3, "Seafoam", "#5bd4ba", 0x5bd4ba),
    ] {
        assert!(p.click(button).is_none());
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        let svg = p.document.query_selector("#scene").unwrap();
        let accent = p.document.query_selector("#accent").unwrap();
        assert_ne!(svg, initial_svg);
        assert_eq!(p.document.namespace(svg), Some(Namespace::Svg));
        assert_eq!(p.document.namespace(accent), Some(Namespace::Svg));
        assert_eq!(p.document.attr(svg, "viewBox"), Some("0 0 380 160"));
        assert_eq!(p.document.attr(svg, "viewbox"), None);
        assert_eq!(
            p.document.attr(svg, "preserveAspectRatio"),
            Some("xMidYMid meet")
        );
        assert_eq!(p.document.attr(accent, "fill"), Some(fill));
        for (selector, expected) in [
            ("#state", format!("{name} / update {step}")),
            ("#namespace", Namespace::Svg.uri().to_owned()),
            ("#viewport", "0 0 380 160 / xMidYMid meet".to_owned()),
            (
                "#runtime",
                format!(
                    "SceneCycle instance: true / prototype method / switch case {}",
                    step % 3
                ),
            ),
        ] {
            assert_eq!(
                p.document
                    .text_content(p.document.query_selector(selector).unwrap()),
                expected
            );
        }
        assert!(!p.images.contains_key(&initial_key));
        assert_eq!(p.images.len(), 1, "removed SVG rasters must be released");
        canvas.clear(Color::WHITE);
        canvas.paint(
            &p.layout(960.0, 900.0, &fonts).commands,
            &fonts,
            &p.images,
            0.0,
            0.0,
        );
        assert!(
            canvas.pixels.iter().filter(|pixel| **pixel == rgb).count() > 8_000,
            "the {name} SVG palette must reach the page canvas"
        );
    }
}

#[test]
fn form_get_serializes_successful_controls_and_clicks_do_not_submit_text_inputs() {
    let mut p = page(
        "<form action=/search><input name=q value='hello world'><input type=checkbox name=on checked><input type=checkbox name=off><input name=no disabled value=bad><textarea name=body>α&amp;β</textarea><button name=go value=yes>Go</button></form>",
    );
    let input = p.document.query_selector("input").unwrap();
    assert!(p.click(input).is_none());
    let button = p.document.query_selector("button").unwrap();
    let target = p.click(button).unwrap();
    let pairs = Url::parse(&target.address)
        .unwrap()
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect::<Vec<_>>();
    assert_eq!(
        pairs,
        vec![
            ("q".into(), "hello world".into()),
            ("on".into(), "on".into()),
            ("body".into(), "α&β".into()),
            ("go".into(), "yes".into())
        ]
    );
    let unchecked = p.document.query_selector("input[name=off]").unwrap();
    p.click(unchecked);
    assert!(p.document.attr(unchecked, "checked").is_some());
}
#[test]
fn cancelled_links_and_forms_stay_in_document() {
    let mut p = page(
        "<a href='/next' onclick='event.preventDefault()'>Link</a><form action='/send' onsubmit='event.preventDefault()'><button>Send</button></form>",
    );
    assert!(p.click(p.document.query_selector("a").unwrap()).is_none());
    assert!(
        p.click(p.document.query_selector("button").unwrap())
            .is_none()
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
}
#[test]
fn scripts_disabled_and_template_content_inert() {
    let p = Page::from_html(
        Url::parse("https://example.test").unwrap(),
        "<p id=x>original</p><script>document.getElementById('x').textContent='changed';</script><template><h1>hidden template</h1></template>",
        false,
    );
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "original"
    );
    let result = p.layout(400.0, 200.0, &Fonts::new());
    assert!(
        !result
            .commands
            .iter()
            .any(|c| matches!(c,DrawCommand::Text{text,..} if text.contains("template")))
    );
}

#[test]
fn noscript_parsing_and_rendering_follow_the_page_scripting_flag() {
    let source = "<!doctype html><body><p id=main>Page</p><noscript><p id=fallback>Fallback content</p><script>document.getElementById('main').textContent='wrong';</script></noscript>";
    let enabled = Page::from_html(Url::parse("https://example.test/").unwrap(), source, true);
    let disabled = Page::from_html(Url::parse("https://example.test/").unwrap(), source, false);
    assert!(enabled.document.query_selector("#fallback").is_none());
    assert!(disabled.document.query_selector("#fallback").is_some());
    assert_eq!(
        enabled
            .document
            .text_content(enabled.document.query_selector("#main").unwrap()),
        "Page"
    );
    let fonts = Fonts::new();
    let has_fallback = |p: &Page| {
        p.layout(400.0, 200.0, &fonts).commands.iter().any(
            |command| matches!(command,DrawCommand::Text{text,..} if text.contains("Fallback")),
        )
    };
    assert!(!has_fallback(&enabled));
    assert!(has_fallback(&disabled));
    assert!(enabled.diagnostics.is_empty(), "{:?}", enabled.diagnostics);
}

#[test]
fn json_demo_reads_a_textarea_runs_callbacks_and_updates_the_document() {
    let mut page = Page::from_html(
        Url::parse("https://example.test/standards.html").unwrap(),
        include_str!("../examples/standards.html"),
        true,
    );
    let button = page.document.query_selector("#format").unwrap();
    assert!(page.click(button).is_none());
    let output = page.document.query_selector("#result").unwrap();
    assert_eq!(
        page.document.text_content(output),
        "{\n  \"title\": \"Custom engine\",\n  \"count\": 14,\n  \"items\": [\n    2,\n    4,\n    6\n  ]\n}"
    );
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
}
#[test]
fn file_resource_loading_preserves_stylesheet_order_and_script_events() {
    let path = std::env::temp_dir().join(format!("eris-pipeline-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("site.css"), "#box{background:red}").unwrap();
    std::fs::write(
        path.join("main.js"),
        "document.getElementById('box').textContent='loaded';",
    )
    .unwrap();
    std::fs::write(path.join("index.html"),"<link rel=stylesheet href=site.css><style>#box{background:blue;width:100px;height:30px}body{margin:0}</style><div id=box>start</div><script src=main.js></script>").unwrap();
    let p = Page::load(path.join("index.html").to_str().unwrap(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#box").unwrap()),
        "loaded"
    );
    let fonts = Fonts::new();
    let mut c = Canvas::new(200, 100).unwrap();
    c.paint(
        &p.layout(200.0, 100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(c.pixels[25 * 200 + 80], 0x0000ff);
    std::fs::remove_dir_all(path).unwrap();
}
#[test]
fn local_file_scope_follows_canonical_paths() {
    use eris::net::{Fetcher, ResourceKind};
    let path = std::env::temp_dir().join(format!("eris-scope-{}", std::process::id()));
    std::fs::create_dir_all(path.join("allowed")).unwrap();
    std::fs::write(path.join("allowed/index.html"), "ok").unwrap();
    std::fs::write(path.join("outside.txt"), "outside").unwrap();
    let url = Url::from_file_path(path.join("allowed/index.html")).unwrap();
    let mut fetcher = Fetcher::for_document(&url);
    assert!(fetcher.fetch(&url, None, ResourceKind::Document).is_ok());
    assert!(
        fetcher
            .fetch(
                &url.join("../outside.txt").unwrap(),
                Some(&url),
                ResourceKind::Image
            )
            .is_err()
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(path.join("outside.txt"), path.join("allowed/link.txt"))
            .unwrap();
        assert!(
            fetcher
                .fetch(
                    &url.join("link.txt").unwrap(),
                    Some(&url),
                    ResourceKind::Image
                )
                .is_err()
        );
    }
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn template_scripts_styles_metadata_and_resources_are_inert() {
    let path = std::env::temp_dir().join(format!("eris-template-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(path.join("index.html"), "<body><p id=x>original</p><template><meta http-equiv='Content-Security-Policy' content=\"default-src 'none'\"><link rel=stylesheet href=missing.css><style>p{color:red}</style><script>document.getElementById('x').textContent='unsafe';</script><script src=missing.js></script><img src=missing.png><svg width=10 height=10><rect width=10 height=10 fill=red /></svg></template><script>document.getElementById('x').textContent='active';</script>").unwrap();
    let p = Page::load(path.join("index.html").to_str().unwrap(), true).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert!(p.scripts_enabled);
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "active"
    );
    assert!(p.stylesheets().is_empty());
    assert!(p.images.is_empty());
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn from_html_enforces_meta_csp_before_scripts_and_styles() {
    let p = page(
        "<meta http-equiv='content-security-policy' content=\"script-src 'none'\"><style>p{color:red}</style><p id=x style='color:blue'>original</p><script>document.getElementById('x').textContent='unsafe';</script>",
    );
    let paragraph = p.document.query_selector("#x").unwrap();
    assert!(!p.scripts_enabled);
    assert_eq!(p.document.text_content(paragraph), "original");
    assert!(p.document.attr(paragraph, "style").is_none());
    assert!(p.stylesheets().is_empty());
    assert_eq!(p.diagnostics.len(), 1);
    assert!(p.load_ms > 0.0);
}

#[test]
fn disabled_control_ancestors_block_events_and_submission_except_first_legend() {
    let mut p = page(
        "<p id=x>original</p><form action=/search><fieldset disabled><legend><input id=legend type=checkbox name=legend value=yes></legend><input id=locked type=checkbox name=locked checked><button id=disabled><span id=child onclick=\"document.getElementById('x').textContent='unsafe'\">Blocked</span></button><legend><input name=later value=bad></legend></fieldset><div id=plain>Not a submitter</div><button id=submit>Go</button></form><div inert><a id=inert href=/unsafe>Inert</a></div>",
    );
    assert!(
        p.click(p.document.query_selector("#child").unwrap())
            .is_none()
    );
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "original"
    );
    let locked = p.document.query_selector("#locked").unwrap();
    assert!(p.click(locked).is_none());
    assert!(p.document.attr(locked, "checked").is_some());
    let legend = p.document.query_selector("#legend").unwrap();
    p.click(legend);
    assert!(p.document.attr(legend, "checked").is_some());
    assert!(
        p.click(p.document.query_selector("#plain").unwrap())
            .is_none()
    );
    assert!(
        p.click(p.document.query_selector("#inert").unwrap())
            .is_none()
    );
    let target = p
        .click(p.document.query_selector("#submit").unwrap())
        .unwrap();
    assert_eq!(
        Url::parse(&target.address).unwrap().query(),
        Some("legend=yes")
    );
}

#[test]
fn form_select_serializes_enabled_selected_options_and_limits_output() {
    let mut p = page(
        "<form><select name=choice multiple><optgroup disabled><option selected value=bad>Bad</option></optgroup><optgroup><option selected value=one>One</option><option selected value=two>Two</option><option disabled selected value=bad2>Bad</option></optgroup></select><button>Go</button></form>",
    );
    let button = p.document.query_selector("button").unwrap();
    assert_eq!(
        Url::parse(&p.click(button).unwrap().address)
            .unwrap()
            .query(),
        Some("choice=one&choice=two")
    );
    let input = p.document.create_element("input");
    p.document.set_attr(input, "name", "large");
    p.document
        .set_attr(input, "value", &"a".repeat(1024 * 1024 + 1));
    p.document
        .append_child(p.document.query_selector("form").unwrap(), input);
    assert!(p.click(button).is_none());
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.contains("submission byte budget"))
    );
}

#[test]
fn inline_svg_reaches_pixels_and_removed_nodes_release_rasters() {
    let mut p = page(
        "<style>body{margin:0}</style><div id=holder><svg width=20 height=20 viewBox='0 0 20 20'><rect width=20 height=20 fill='#12843c'/></svg></div>",
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let svg = p.document.query_selector("svg").unwrap();
    let key = format!("eris-inline-svg:{svg}");
    assert!(p.images.contains_key(&key));
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(80, 60).unwrap();
    canvas.paint(
        &p.layout(80.0, 60.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(canvas.pixels[10 * 80 + 10], 0x12843c);
    let holder = p.document.query_selector("#holder").unwrap();
    p.document.set_text_content(holder, "removed");
    p.refresh_inline_svg();
    assert!(!p.images.contains_key(&key));
}

#[test]
fn repeated_external_images_share_rasters_and_resource_budget() {
    let path = std::env::temp_dir().join(format!("eris-svg-resources-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    std::fs::write(
        path.join("image.svg"),
        "<svg width='12' height='8'><rect width='12' height='8' fill='#285fac'/></svg>",
    )
    .unwrap();
    let mut html = "<style>body{margin:0}img{display:block}</style>".to_owned();
    for n in 0..80 {
        html.push_str(if n % 2 == 0 {
            "<img src='image.svg'>"
        } else {
            "<img src='./image.svg'>"
        });
    }
    std::fs::write(path.join("index.html"), html).unwrap();
    let p = Page::load(path.join("index.html").to_str().unwrap(), false).unwrap();
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(p.images.len(), 2);
    assert!(std::sync::Arc::ptr_eq(
        p.images.get("image.svg").unwrap(),
        p.images.get("./image.svg").unwrap()
    ));
    let fonts = Fonts::new();
    let mut canvas = Canvas::new(80, 60).unwrap();
    canvas.paint(
        &p.layout(80.0, 60.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert_eq!(canvas.pixels[4 * 80 + 6], 0x285fac);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn post_navigation_uses_action_query_and_submitter_overrides() {
    let mut p = page(
        "<form action='/default?old=1' method=get enctype='multipart/form-data'><input name=q value='one two'><button id=post formaction='/posted?keep=1' formmethod=PoSt formenctype='application/x-www-form-urlencoded' name=go value=yes>Post</button><button id=get>Get</button></form>",
    );
    let post = p
        .click(p.document.query_selector("#post").unwrap())
        .unwrap();
    assert_eq!(post.address, "https://example.test/posted?keep=1");
    assert_eq!(post.form_body.as_deref(), Some("q=one+two&go=yes"));
    let get = p.click(p.document.query_selector("#get").unwrap()).unwrap();
    assert_eq!(get.address, "https://example.test/default?q=one+two");
    assert!(get.form_body.is_none());
}

#[test]
fn post_rejects_unsupported_encoding_downgrade_and_encoded_payload_growth() {
    for attributes in [
        "method=post enctype='multipart/form-data'",
        "method=post enctype='text/plain'",
        "method=post action='http://example.test/unsafe'",
    ] {
        let mut p = page(&format!(
            "<form {attributes}><input name=q value=test><button>Send</button></form>"
        ));
        assert!(
            p.click(p.document.query_selector("button").unwrap())
                .is_none()
        );
        assert!(!p.diagnostics.is_empty());
    }
    let mut p = page("<form method=post><input name=q><button>Send</button></form>");
    let input = p.document.query_selector("input").unwrap();
    p.document.set_attr(input, "value", &"&".repeat(400_000));
    assert!(
        p.click(p.document.query_selector("button").unwrap())
            .is_none()
    );
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.contains("submission byte budget"))
    );
}

#[test]
fn navigation_post_rejects_local_and_builtin_schemes_before_loading() {
    use eris::page::Navigation;
    for address in [
        "file:///etc/passwd",
        "data:text/html,hello",
        "about:blank",
        "eris:home",
    ] {
        let navigation = Navigation {
            address: address.into(),
            form_body: Some("q=test".into()),
        };
        assert!(Page::load_navigation(&navigation, false).is_err());
    }
    assert_eq!(Navigation::get("https://example.test").form_body, None);
}

#[test]
fn urlencoded_forms_normalize_line_breaks_for_get_and_post() {
    for method in ["get", "post"] {
        let mut p = page(&format!(
            "<form method={method}><textarea name=body>one\ntwo\rthree\r\nfour</textarea><button>Send</button></form>"
        ));
        let navigation = p
            .click(p.document.query_selector("button").unwrap())
            .unwrap();
        let encoded = navigation.form_body.unwrap_or_else(|| {
            Url::parse(&navigation.address)
                .unwrap()
                .query()
                .unwrap()
                .to_owned()
        });
        assert_eq!(encoded, "body=one%0D%0Atwo%0D%0Athree%0D%0Afour");
    }
}

#[test]
fn fragment_identifiers_decode_utf8_without_form_plus_rules() {
    let p = page("<p id='café'>one</p><p id='a+b'>two</p>");
    let id = eris::page::find_fragment(&p.document, "caf%C3%A9").unwrap();
    assert_eq!(p.document.attr(id, "id"), Some("café"));
    let id = eris::page::find_fragment(&p.document, "a+b").unwrap();
    assert_eq!(p.document.attr(id, "id"), Some("a+b"));
}

#[test]
fn html_bytes_select_encoding_and_reparse_late_declarations_before_scripts() {
    use base64::Engine;
    fn load(bytes: &[u8], mime: &str) -> Page {
        let address = format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        );
        Page::load(&address, true).unwrap()
    }
    let p = load(b"<p id=x>caf\xe9", "text/html");
    assert_eq!(p.document.character_set(), "windows-1252");
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "café"
    );
    let p = load(b"<meta charset=utf-8><p id=x>caf\xc3\xa9", "text/html");
    assert_eq!(p.document.character_set(), "UTF-8");
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "café"
    );
    let p = load(
        b"<meta charset=utf-8><p id=x>caf\xe9",
        "text/html;charset=windows-1252",
    );
    assert_eq!(p.document.character_set(), "windows-1252");
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "café"
    );
    for declaration in [
        "<meta charset=utf-8>",
        "<meta charset=utf&#45;8>",
        "<template><meta charset=utf-8></template><meta charset=windows-1252>",
        "<script>'<meta charset=shift_jis>';</script><meta charset=utf-8>",
        "<meta charset=unknown><meta charset=utf-8><meta charset=shift_jis>",
    ] {
        let source = format!(
            "<!doctype html><head><!--{}-->{declaration}</head><body><p id=x>café</p><p id=encoding></p><script>document.getElementById('x').textContent += '!'; document.getElementById('encoding').textContent = document.characterSet + '/' + document.charset + '/' + document.inputEncoding;</script>",
            " ".repeat(1100)
        );
        let p = load(source.as_bytes(), "text/html");
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        assert_eq!(p.document.character_set(), "UTF-8", "{declaration}");
        assert_eq!(
            p.document
                .text_content(p.document.query_selector("#x").unwrap()),
            "café!",
            "{declaration}"
        );
        assert_eq!(
            p.document
                .text_content(p.document.query_selector("#encoding").unwrap()),
            "UTF-8/UTF-8/UTF-8"
        );
    }
    let p = page("<meta charset=windows-1252><p id=x>café");
    assert_eq!(
        p.document.character_set(),
        "UTF-8",
        "string DOM APIs do not reinterpret bytes"
    );
    assert_eq!(
        p.document
            .text_content(p.document.query_selector("#x").unwrap()),
        "café"
    );
}

#[test]
fn authoritative_control_edit_policy_blocks_inert_disabled_and_readonly_nodes() {
    let p = page(
        "<input id=a><input id=b readonly><fieldset disabled><input id=c></fieldset><template><input id=d></template><input id=e type=HIDDEN><input id=f type=PASSWORD><input id=g type=CHECKBOX><textarea id=h></textarea>",
    );
    for name in ["a", "f", "h"] {
        assert!(
            p.can_edit_control(p.document.query_selector(&format!("#{name}")).unwrap()),
            "{name}"
        );
    }
    for name in ["b", "c", "e", "g"] {
        assert!(
            !p.can_edit_control(p.document.query_selector(&format!("#{name}")).unwrap()),
            "{name}"
        );
    }
    let template = p.document.query_selector("template").unwrap();
    let contents = p.document.template_contents(template).unwrap();
    let inert = p.document.query_selector_from(contents, "#d").unwrap();
    assert!(!p.can_edit_control(inert));
    assert!(!p.can_edit_control(usize::MAX));
}

#[test]
fn float_demo_descriptor_setter_updates_contextual_table_fragments_and_paint() {
    let mut p = page(include_str!("../examples/flow.html"));
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let fonts = Fonts::new();
    let button = p.document.query_selector("#advance").unwrap();
    let status = p.document.query_selector("#status").unwrap();
    for (step, title) in [(2, "Summer"), (3, "Autumn"), (1, "Spring")] {
        assert!(p.click(button).is_none());
        assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
        assert_eq!(
            p.document.text_content(status),
            format!("Chapter {step} / {title}")
        );
        let table = p.document.query_selector("#chapters").unwrap();
        let tbody = p.document.nodes[table].children[0];
        assert_eq!(p.document.tag(tbody), Some("tbody"));
        let row = p.document.nodes[tbody].children[0];
        assert_eq!(p.document.tag(row), Some("tr"));
        assert_eq!(p.document.nodes[row].children.len(), 2);
        let cell = p.document.nodes[row].children[1];
        assert_eq!(p.document.text_content(cell), title);
        let layout = p.layout(1100.0, 1000.0, &fonts);
        assert!(
            layout
                .commands
                .iter()
                .any(|c| matches!(c, DrawCommand::Text{text,..} if text==title))
        );
        let note = p.document.query_selector(".note").unwrap();
        let rect = |node| {
            layout
                .hit_regions
                .iter()
                .find(|h| h.node == node)
                .unwrap()
                .rect
        };
        assert!(rect(button).y >= rect(note).y + rect(note).height);
        let mut canvas = Canvas::new(1100, 1000).unwrap();
        canvas.paint(&layout.commands, &fonts, &p.images, 0.0, 0.0);
        assert!(canvas.pixels.contains(&0x173f35));
    }
}

#[test]
fn template_demo_strict_callbacks_clone_cards_and_reflow_grid() {
    let mut p = page(include_str!("../examples/templates.html"));
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let collection = p.document.query_selector("#collection").unwrap();
    let template = p.document.query_selector("#note").unwrap();
    let original = p.document.template_contents(template).unwrap();
    assert_eq!(p.document.nodes[collection].children.len(), 5);
    assert_eq!(p.document.nodes[original].children.len(), 1);
    assert!(p.document.nodes[template].children.is_empty());
    let fonts = Fonts::new();
    let second = p.document.query_selector("#note-2").unwrap();
    let first = p.document.query_selector("#note-1").unwrap();
    let initial = p.layout(1100.0, 1100.0, &fonts);
    let rect = |layout: &eris::layout::LayoutResult, node| {
        layout
            .hit_regions
            .iter()
            .find(|hit| hit.node == node)
            .unwrap()
            .rect
    };
    assert!((rect(&initial, first).y - rect(&initial, second).y).abs() < 0.1);
    p.click(p.document.query_selector("#arrange").unwrap());
    let compact = p.layout(1100.0, 1100.0, &fonts);
    assert!(
        rect(&compact, second).y >= rect(&compact, first).y + rect(&compact, first).height + 17.9
    );
    for count in 6..=9 {
        p.click(p.document.query_selector("#add").unwrap());
        assert_eq!(p.document.nodes[collection].children.len(), count);
        assert_eq!(
            p.document
                .text_content(p.document.query_selector("#status").unwrap()),
            format!("{count} notes in the collection")
        );
    }
    p.click(p.document.query_selector("#add").unwrap());
    assert_eq!(p.document.nodes[collection].children.len(), 9);
    assert_eq!(p.document.nodes[original].children.len(), 1);
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let mut canvas = Canvas::new(1100, 1100).unwrap();
    canvas.paint(
        &p.layout(1100.0, 1100.0, &fonts).commands,
        &fonts,
        &p.images,
        0.0,
        0.0,
    );
    assert!(canvas.pixels.contains(&0x263c38));
    assert!(canvas.pixels.contains(&0xe0e7db));
}

#[test]
fn event_demo_capture_custom_dispatch_and_imported_layers_reach_pixels() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/events.html");
    let mut page = Page::load(path.to_str().unwrap(), true).unwrap();
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    let send = page.document.query_selector("#send").unwrap();
    let status = page.document.query_selector("#status").unwrap();
    let trace = page.document.query_selector("#trace").unwrap();
    for count in 1..=3 {
        assert!(page.click(send).is_none());
        assert_eq!(
            page.document.text_content(status),
            format!("Signal {count} received · CustomEvent")
        );
        assert_eq!(
            page.document.text_content(trace),
            "document capture → panel capture → target → panel bubble → document bubble"
        );
    }
    let fonts = Fonts::new();
    let layout = page.layout(1100.0, 1600.0, &fonts);
    assert!(layout.commands.iter().any(
        |c| matches!(c, eris::graphics::DrawCommand::PushOpacity { opacity } if *opacity == 0.55)
    ));
    let mut canvas = Canvas::new(1100, 1600).unwrap();
    canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    assert!(
        canvas.pixels.contains(&0x315f64),
        "palette layer button color reaches pixels"
    );
    assert!(canvas.pixels.contains(&0xf4f0e7));
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
}

#[test]
fn document_append_failure_prefixes_and_root_restoration_survive_callbacks() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/document-append.html"),
        Some(["ready", "6"]),
    );
}

#[test]
fn dom_own_properties_and_native_fallback_survive_page_callbacks() {
    assert_six_scripted_samples_with_result_text(
        include_str!("fixtures/dom-own-properties.html"),
        Some(["ready", "6"]),
    );
}
