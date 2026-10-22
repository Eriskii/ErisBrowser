//! Selected legacy var() feature-query semantics, frozen before producer inspection.
//! Custom-property queries and newer arbitrary first-argument substitution remain deferred.
use eris::{
    css,
    graphics::{Canvas, Color, Fonts},
    page::Page,
    script::Value,
};
use url::Url;

fn pairs(rows: &[(&str, &str, bool)]) {
    for &(property, value, expected) in rows {
        assert_eq!(
            css::supports_declaration(property, value),
            expected,
            "property={property:?}, value={value:?}"
        );
    }
}

fn conditions(rows: &[(&str, bool)]) {
    for &(query, expected) in rows {
        assert_eq!(css::supports_matches(query), expected, "query={query:?}");
    }
}

#[test]
fn supports_var_complete_implemented_name_roster() {
    // Explicit source-derived 88 longhands, 29 shorthands and 13 existing aliases.
    // This table is independent of the producer's future recognition helper.
    for (property, query) in [
        ("display", "(display:var(--missing))"),
        ("visibility", "(visibility:var(--missing))"),
        ("pointer-events", "(pointer-events:var(--missing))"),
        ("object-fit", "(object-fit:var(--missing))"),
        ("object-position", "(object-position:var(--missing))"),
        ("float", "(float:var(--missing))"),
        ("clear", "(clear:var(--missing))"),
        ("width", "(width:var(--missing))"),
        ("height", "(height:var(--missing))"),
        ("min-width", "(min-width:var(--missing))"),
        ("min-height", "(min-height:var(--missing))"),
        ("max-width", "(max-width:var(--missing))"),
        ("max-height", "(max-height:var(--missing))"),
        ("margin-top", "(margin-top:var(--missing))"),
        ("margin-right", "(margin-right:var(--missing))"),
        ("margin-bottom", "(margin-bottom:var(--missing))"),
        ("margin-left", "(margin-left:var(--missing))"),
        ("padding-top", "(padding-top:var(--missing))"),
        ("padding-right", "(padding-right:var(--missing))"),
        ("padding-bottom", "(padding-bottom:var(--missing))"),
        ("padding-left", "(padding-left:var(--missing))"),
        ("border-spacing", "(border-spacing:var(--missing))"),
        ("border-top-width", "(border-top-width:var(--missing))"),
        ("border-right-width", "(border-right-width:var(--missing))"),
        (
            "border-bottom-width",
            "(border-bottom-width:var(--missing))",
        ),
        ("border-left-width", "(border-left-width:var(--missing))"),
        ("border-top-style", "(border-top-style:var(--missing))"),
        ("border-right-style", "(border-right-style:var(--missing))"),
        (
            "border-bottom-style",
            "(border-bottom-style:var(--missing))",
        ),
        ("border-left-style", "(border-left-style:var(--missing))"),
        ("border-top-color", "(border-top-color:var(--missing))"),
        ("border-right-color", "(border-right-color:var(--missing))"),
        (
            "border-bottom-color",
            "(border-bottom-color:var(--missing))",
        ),
        ("border-left-color", "(border-left-color:var(--missing))"),
        ("color", "(color:var(--missing))"),
        ("background-color", "(background-color:var(--missing))"),
        ("background-image", "(background-image:var(--missing))"),
        ("background-size", "(background-size:var(--missing))"),
        (
            "background-position",
            "(background-position:var(--missing))",
        ),
        ("background-repeat", "(background-repeat:var(--missing))"),
        ("background-origin", "(background-origin:var(--missing))"),
        ("background-clip", "(background-clip:var(--missing))"),
        ("font-size", "(font-size:var(--missing))"),
        ("font-family", "(font-family:var(--missing))"),
        ("font-style", "(font-style:var(--missing))"),
        ("font-weight", "(font-weight:var(--missing))"),
        ("line-height", "(line-height:var(--missing))"),
        ("text-align", "(text-align:var(--missing))"),
        ("white-space", "(white-space:var(--missing))"),
        ("text-decoration", "(text-decoration:var(--missing))"),
        ("flex-direction", "(flex-direction:var(--missing))"),
        ("flex-wrap", "(flex-wrap:var(--missing))"),
        ("justify-content", "(justify-content:var(--missing))"),
        ("align-items", "(align-items:var(--missing))"),
        ("align-self", "(align-self:var(--missing))"),
        ("order", "(order:var(--missing))"),
        ("row-gap", "(row-gap:var(--missing))"),
        ("column-gap", "(column-gap:var(--missing))"),
        ("flex-grow", "(flex-grow:var(--missing))"),
        ("flex-shrink", "(flex-shrink:var(--missing))"),
        ("flex-basis", "(flex-basis:var(--missing))"),
        (
            "grid-template-columns",
            "(grid-template-columns:var(--missing))",
        ),
        ("grid-template-rows", "(grid-template-rows:var(--missing))"),
        ("grid-auto-columns", "(grid-auto-columns:var(--missing))"),
        ("grid-auto-rows", "(grid-auto-rows:var(--missing))"),
        ("grid-auto-flow", "(grid-auto-flow:var(--missing))"),
        ("grid-column-start", "(grid-column-start:var(--missing))"),
        ("grid-column-end", "(grid-column-end:var(--missing))"),
        ("grid-row-start", "(grid-row-start:var(--missing))"),
        ("grid-row-end", "(grid-row-end:var(--missing))"),
        ("justify-items", "(justify-items:var(--missing))"),
        ("justify-self", "(justify-self:var(--missing))"),
        ("align-content", "(align-content:var(--missing))"),
        ("position", "(position:var(--missing))"),
        ("z-index", "(z-index:var(--missing))"),
        ("top", "(top:var(--missing))"),
        ("right", "(right:var(--missing))"),
        ("bottom", "(bottom:var(--missing))"),
        ("left", "(left:var(--missing))"),
        ("overflow", "(overflow:var(--missing))"),
        ("overflow-x", "(overflow-x:var(--missing))"),
        ("overflow-y", "(overflow-y:var(--missing))"),
        ("border-radius", "(border-radius:var(--missing))"),
        ("opacity", "(opacity:var(--missing))"),
        ("box-sizing", "(box-sizing:var(--missing))"),
        ("list-style-type", "(list-style-type:var(--missing))"),
        (
            "list-style-position",
            "(list-style-position:var(--missing))",
        ),
        ("vertical-align", "(vertical-align:var(--missing))"),
        ("all", "(all:var(--missing))"),
        ("gap", "(gap:var(--missing))"),
        ("grid-gap", "(grid-gap:var(--missing))"),
        ("grid-row", "(grid-row:var(--missing))"),
        ("grid-column", "(grid-column:var(--missing))"),
        ("place-items", "(place-items:var(--missing))"),
        ("place-self", "(place-self:var(--missing))"),
        ("place-content", "(place-content:var(--missing))"),
        ("list-style", "(list-style:var(--missing))"),
        ("flex-flow", "(flex-flow:var(--missing))"),
        ("margin-inline", "(margin-inline:var(--missing))"),
        ("padding-inline", "(padding-inline:var(--missing))"),
        ("margin-block", "(margin-block:var(--missing))"),
        ("padding-block", "(padding-block:var(--missing))"),
        ("grid-area", "(grid-area:var(--missing))"),
        ("inset", "(inset:var(--missing))"),
        ("margin", "(margin:var(--missing))"),
        ("padding", "(padding:var(--missing))"),
        ("border-width", "(border-width:var(--missing))"),
        ("border-color", "(border-color:var(--missing))"),
        ("border-style", "(border-style:var(--missing))"),
        ("background", "(background:var(--missing))"),
        ("font", "(font:var(--missing))"),
        ("border", "(border:var(--missing))"),
        ("flex", "(flex:var(--missing))"),
        ("border-top", "(border-top:var(--missing))"),
        ("border-right", "(border-right:var(--missing))"),
        ("border-bottom", "(border-bottom:var(--missing))"),
        ("border-left", "(border-left:var(--missing))"),
        ("inline-size", "(inline-size:var(--missing))"),
        ("block-size", "(block-size:var(--missing))"),
        ("grid-row-gap", "(grid-row-gap:var(--missing))"),
        ("grid-column-gap", "(grid-column-gap:var(--missing))"),
        (
            "margin-inline-start",
            "(margin-inline-start:var(--missing))",
        ),
        ("margin-inline-end", "(margin-inline-end:var(--missing))"),
        ("margin-block-start", "(margin-block-start:var(--missing))"),
        ("margin-block-end", "(margin-block-end:var(--missing))"),
        (
            "padding-inline-start",
            "(padding-inline-start:var(--missing))",
        ),
        ("padding-inline-end", "(padding-inline-end:var(--missing))"),
        (
            "padding-block-start",
            "(padding-block-start:var(--missing))",
        ),
        ("padding-block-end", "(padding-block-end:var(--missing))"),
        (
            "text-decoration-line",
            "(text-decoration-line:var(--missing))",
        ),
    ] {
        assert!(
            css::supports_declaration(property, "var(--missing)"),
            "{property}"
        );
        assert!(css::supports_matches(query), "{query}");
    }
}

#[test]
fn supports_var_names() {
    pairs(&[
        ("WIDTH", "var(--x)", true),
        ("font-size", "var(--x)", true),
        ("inline-size", "var(--x)", true),
        (" width", "var(--x)", false),
        ("width ", "var(--x)", false),
        ("w\\69 dth", "var(--x)", false),
        ("width/**/", "var(--x)", false),
        ("wi/**/dth", "var(--x)", false),
        ("unknown-property", "var(--x)", false),
        ("transform", "var(--x)", false),
        ("--theme", "var(--x)", false),
        ("--theme", "blue", false),
        ("--theme", "", false),
        ("--", "var(--x)", false),
    ]);
    conditions(&[
        ("(w\\69 dth:var(--x))", true),
        ("(pointer\\2d events:var(--x))", true),
        ("( WIDTH : var(--x) )", true),
        ("(inline-size:var(--x))", true),
        ("(wi/**/dth:var(--x))", false),
        ("(unknown-property:var(--x))", false),
        ("(transform:var(--x))", false),
        ("(--theme:var(--x))", false),
        ("(--theme:blue)", false),
        ("(--theme:)", false),
    ]);
}

#[test]
fn supports_var_real_tokens() {
    pairs(&[
        ("width", "VAR(--x)", true),
        ("width", "v\\61 r(--x)", true),
        ("width", "var(--\\78)", true),
        ("width", "fn(var(--x))", true),
        ("width", "/**/var(--x)/**/", true),
        ("width", "\"var(--x)\"", false),
        ("width", "/*var(--x)*/", false),
        ("width", "/*var(--x)*/1px", true),
        ("width", "v/**/ar(--x)", false),
        ("width", "var /*gap*/(--x)", false),
        ("width", "myvar(--x)", false),
        ("width", "var(--x,\"var(foo)\")", true),
    ]);
    conditions(&[
        ("(width:v\\61 r(--x))", true),
        ("(width:\"var(--x)\")", false),
        ("(width:v/**/ar(--x))", false),
    ]);
}

#[test]
fn supports_var_legacy_fallbacks() {
    pairs(&[
        ("color", "var(--x,)", true),
        ("color", "var(--x, var(--y, red))", true),
        ("color", "var(--x, var(--y,))", true),
        ("color", "var(--x,,)", true),
        ("color", "var(--x,\";\")", true),
        ("color", "var(--x,\"!important\")", true),
        ("color", "var(--x, fn(;))", true),
        ("color", "var(--x, {a;b})", true),
        ("color", "var(--x, [a;b])", true),
        ("width", "var()", false),
        ("width", "var(foo)", false),
        ("width", "var(--)", false),
        ("width", "var(--x extra)", false),
        ("width", "var(--x, var(foo))", false),
        ("width", "var(var(--name))", false),
        ("width", "var(--x, !important)", false),
        ("width", "var(--x, a;b)", false),
        ("width", "var(--x])", false),
        ("width", "var(--x", false),
        ("width", "var(--x)\0", false),
    ]);
    conditions(&[
        ("(color:var(--x, fn(;)))", true),
        ("(color:var(--x, {a;b}))", true),
        ("(color:var(--x,\";\"))", true),
        ("(width:var(--x, var(foo)))", false),
    ]);
}

#[test]
fn supports_var_deferred_grammar() {
    pairs(&[
        ("width", "nonsense var(--x)", true),
        ("width", "calc(var(--x) + )", true),
        ("color", "future(var(--x))", true),
        ("visibility", "var(--x,collapse)", true),
        ("pointer-events", "var(--x,visiblePainted)", true),
        (
            "background-image",
            "var(--x,url(\"https://no-query-fetch.invalid/a.png\"))",
            true,
        ),
        ("border", "var(--x,unrecognized)", true),
        ("all", "var(--reset)", true),
    ]);
    conditions(&[
        ("(width:calc(var(--x) + ))", true),
        ("(pointer-events:var(--x,visiblePainted))", true),
        ("(all:var(--reset))", true),
    ]);
}

#[test]
fn supports_var_priority_boolean() {
    pairs(&[
        ("width", "var(--x)!important", false),
        ("width", "var(--x) !/**/IMPORTANT", false),
        ("width", "var(--x);color:red", false),
    ]);
    conditions(&[
        ("(width:var(--x) !important)", true),
        ("(width:var(--x) !/**/IMPORTANT)", true),
        ("(width:var(--x);color:red)", false),
        ("not (width:var(--x))", false),
        ("not (width:var(foo))", true),
        ("(width:var(--x)) and (display:grid)", true),
        ("(width:var(foo)) or (display:grid)", true),
        ("(unknown-property:var(--x)) or (width:var(--x))", true),
        ("not (--theme:var(--x))", true),
        ("not (width:var(--x])", false),
        ("(display:grid) or (width:var(--x])", false),
        ("width:var(--x)", false),
    ]);
}

#[test]
fn supports_var_no_var() {
    pairs(&[
        ("width", "1px", true),
        ("width", "1 px", false),
        ("display", "grid", true),
        ("visibility", "hidden", true),
        ("pointer-events", "none", true),
        ("pointer-events", "visiblePainted", false),
        ("font-size", "12px", false),
        ("unknown-property", "inherit", false),
    ]);
    conditions(&[
        ("(display:grid)", true),
        ("not (position:sticky)", true),
        ("(w\\69 dth:1px)", false),
        ("(visibility:\\68 idden)", true),
    ]);
}

#[test]
fn supports_var_top_level_curly_structure() {
    pairs(&[
        ("width", "{var(--x)}", true),
        ("width", "{} var(--x)", false),
        ("width", "var(--x) {}", false),
        ("width", "var(--x,{foo:bar;})", true),
    ]);
    conditions(&[
        ("(width:{var(--x)})", true),
        ("(width:{} var(--x))", false),
        ("(width:var(--x) {})", false),
        ("(width:var(--x,{foo:bar;}))", true),
    ]);
}

fn runtime(source: &str) {
    for strict in [false, true] {
        let mut page = Page::from_html(
            Url::parse("https://supports-var.invalid/").unwrap(),
            "<div id=target></div>",
            false,
        );
        let result = if strict {
            page.runtime.execute_strict(source, &mut page.document)
        } else {
            page.runtime.execute(source, &mut page.document)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
    }
}

#[test]
fn supports_var_runtime_forms() {
    runtime(
        r###"CSS.supports('width','var(--missing)') &&
CSS.supports('width:var(--missing)') &&
CSS.supports('(width:var(--missing))') &&
CSS.supports('(w\\69 dth:var(--missing))') &&
!CSS.supports('w\\69 dth','var(--missing)') &&
CSS.supports('width:var(--missing) !important') &&
!CSS.supports('width','var(--missing)!important') &&
!CSS.supports('unknown-property','var(--x)') &&
!CSS.supports('--theme','var(--x)');
"###,
    );
}

#[test]
fn supports_var_runtime_coercion() {
    runtime(
        r###"var trace = '';
function argument(label, value) {
  return {toString:function(){trace += label + ';';return value;}};
}
var extra = {toString:function(){trace += 'extra;';throw 1;}};
var accepted = CSS.supports(argument('property','width'), argument('value','var(--x)'), extra);
var first = accepted && trace === 'property;value;';
trace = '';
var rejected = CSS.supports(argument('property','unknown-property'), argument('value','var(--x)'));
var second = !rejected && trace === 'property;value;';
trace = '';
var sentinel = {};
var caughtFirst = false;
try { CSS.supports({toString:function(){trace += 'first;';throw sentinel;}}, argument('second','var(--x)')); }
catch (error) { caughtFirst = error === sentinel; }
var third = caughtFirst && trace === 'first;';
trace = '';
var caughtValue = false;
try { CSS.supports('unknown-property', {toString:function(){trace += 'value;';throw sentinel;}}); }
catch (error) { caughtValue = error === sentinel; }
first && second && third && caughtValue && trace === 'value;';
"###,
    );
}

#[test]
fn supports_var_runtime_computed_independence() {
    runtime(
        r###"var s = document.getElementById('target').style;
var a = CSS.supports('width','var(--x)');
s.setProperty('--x','17px');
var b = CSS.supports('width','var(--x)');
s.setProperty('--x','not-a-width');
var c = CSS.supports('width','var(--x)');
s.setProperty('--x','var(--y)');
s.setProperty('--y','var(--x)');
var d = CSS.supports('width','var(--x)');
s.removeProperty('--x');
var e = CSS.supports('width','var(--x)');
a && b && c && d && e && s.getPropertyValue('--x') === '' && s.getPropertyValue('--y') === 'var(--x)';
"###,
    );
}

#[test]
fn supports_var_stylesheet_condition_selects_literal_pixels() {
    let page = Page::from_html(
        Url::parse("https://supports-var.invalid/").unwrap(),
        r###"<!doctype html><style>
html,body{margin:0;padding:0}
#target{width:4px;height:3px;background:#ff0000;--loop:var(--loop)}
@supports (visibility:var(--missing)){#target{background:#00ff00}}
@supports not (visibility:var(--missing)){#target{background:#0000ff}}
</style><div id=target></div>
"###,
        false,
    );
    let fonts = Fonts::new();
    let layout = page.layout(8.0, 6.0, &fonts);
    let mut canvas = Canvas::new(8, 6).unwrap();
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, &fonts, &page.images, 0.0, 0.0);
    assert!(!canvas.exhausted());
    let mut expected = vec![0xffffff; 8 * 6];
    for row in 0..3 {
        expected[row * 8..row * 8 + 4].fill(0x00ff00);
    }
    assert_eq!(canvas.pixels, expected);
    assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
}
