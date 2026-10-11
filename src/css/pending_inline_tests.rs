//! Independent cascade witnesses; no candidate helper supplies expected values.
use super::*;
use crate::script::{Runtime, Value};

fn execute(doc: &mut Document, source: &str) {
    assert_eq!(
        Runtime::new().execute(source, doc).unwrap(),
        Value::Bool(true)
    );
}

fn margins(style: &ComputedStyle) -> [Length; 4] {
    [
        style.margin.top,
        style.margin.right,
        style.margin.bottom,
        style.margin.left,
    ]
}

fn px(values: [f32; 4]) -> [Length; 4] {
    values.map(Length::Px)
}

#[test]
fn typed_pending_cascade_keeps_priority_and_current_variable_value() {
    let mut doc = Document::parse("<div id=a></div>");
    let id = doc.query_selector("#a").unwrap();
    execute(
        &mut doc,
        r#"
        var s=document.getElementById('a').style;
        s.cssText='--m:1px 2px 3px 4px;margin:var(--m)!important';
        s.marginLeft='9px';s.marginLeft==='9px';
    "#,
    );
    let sheets =
        ["#a{margin-top:99px;margin-right:30px!important;margin-left:12px!important}".to_owned()];
    let styles = compute_styles(&doc, &sheets, 100.0, 100.0);
    assert_eq!(margins(&styles[id]), px([1.0, 2.0, 3.0, 12.0]));
    execute(
        &mut doc,
        "document.getElementById('a').style.setProperty('--m','5px 6px 7px 8px');true;",
    );
    let styles = compute_styles(&doc, &sheets, 100.0, 100.0);
    assert_eq!(margins(&styles[id]), px([5.0, 6.0, 7.0, 12.0]));
    execute(
        &mut doc,
        "document.getElementById('a').style.removeProperty('margin-bottom')==='';",
    );
    let styles = compute_styles(&doc, &sheets, 100.0, 100.0);
    assert_eq!(margins(&styles[id]), px([5.0, 6.0, 0.0, 12.0]));
}

#[test]
fn typed_pending_inherits_live_variables_after_reparenting() {
    let mut doc = Document::parse(
        "<section id=p style='--m:1px 2px 3px 4px'><div id=a></div></section><section id=q style='--m:5px 6px 7px 8px'></section>",
    );
    let id = doc.query_selector("#a").unwrap();
    execute(
        &mut doc,
        "var s=document.getElementById('a').style;s.margin='var(--m)';s.marginLeft='9px';true;",
    );
    assert_eq!(
        margins(&compute_styles(&doc, &[], 100.0, 100.0)[id]),
        px([1.0, 2.0, 3.0, 9.0])
    );
    execute(
        &mut doc,
        "document.getElementById('q').appendChild(document.getElementById('a'));true;",
    );
    assert_eq!(
        margins(&compute_styles(&doc, &[], 100.0, 100.0)[id]),
        px([5.0, 6.0, 7.0, 9.0])
    );
    execute(
        &mut doc,
        "document.getElementById('q').style.setProperty('--m','11px 12px');true;",
    );
    assert_eq!(
        margins(&compute_styles(&doc, &[], 100.0, 100.0)[id]),
        px([11.0, 12.0, 11.0, 9.0])
    );
}

fn absolute(style: &ComputedStyle) -> &str {
    let BackgroundImage::Url(value) = style.backgrounds.image_slots()[0].as_ref().unwrap() else {
        panic!("expected URL image");
    };
    let url::ResolvedUrl::Absolute(value) = value.as_ref() else {
        panic!("expected absolute URL");
    };
    value.as_str()
}

#[test]
fn typed_pending_background_uses_consuming_document_base_after_mutation() {
    let mut doc = Document::parse(
        "<head><base id=b href='https://document.test/first/'></head><body><div id=a></div></body>",
    );
    doc.initialize_url(::url::Url::parse("https://document.test/root/index.html").unwrap());
    let id = doc.query_selector("#a").unwrap();
    let sheets = [StyleSource::with_base(
        "#a{--paint:url(Tile.PNG) no-repeat red}",
        SourceBase::Stylesheet(Arc::new(
            ::url::Url::parse("https://sheet.test/external/main.css").unwrap(),
        )),
    )];
    execute(
        &mut doc,
        "var s=document.getElementById('a').style;s.setProperty('background','var(--paint)','important');s.backgroundClip='content-box';s.background==='';",
    );
    let styles = compute_styles_from_sources(&doc, &sheets, 100.0, 100.0);
    assert_eq!(
        absolute(&styles[id]),
        "https://document.test/first/Tile.PNG"
    );
    assert_eq!(styles[id].backgrounds.clips(), &[BackgroundBox::Content]);
    assert_eq!(styles[id].backgrounds.repeats(), &[[false, false]]);
    assert_eq!(styles[id].background_color, Color::rgb(255, 0, 0));
    execute(
        &mut doc,
        "document.getElementById('b').setAttribute('href','https://document.test/second/');true;",
    );
    assert_eq!(
        absolute(&compute_styles_from_sources(&doc, &sheets, 100.0, 100.0)[id]),
        "https://document.test/second/Tile.PNG"
    );
    execute(
        &mut doc,
        "document.getElementById('a').style.setProperty('--paint','url(Next.PNG) no-repeat blue');true;",
    );
    let styles = compute_styles_from_sources(&doc, &sheets, 100.0, 100.0);
    assert_eq!(
        absolute(&styles[id]),
        "https://document.test/second/Next.PNG"
    );
    assert_eq!(styles[id].background_color, Color::rgb(0, 0, 255));
    assert_eq!(styles[id].backgrounds.clips(), &[BackgroundBox::Content]);
}
