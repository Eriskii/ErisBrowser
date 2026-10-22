use eris::{page::Page, script::Value};
use url::Url;

fn check(source: &str) {
    for strict in [false, true] {
        let mut page = Page::from_html(
            Url::parse("https://pointer-cssom.invalid/").unwrap(),
            "<div id=target></div>",
            false,
        );
        let result = if strict {
            page.runtime.execute_strict(source, &mut page.document)
        } else {
            page.runtime.execute(source, &mut page.document)
        };
        assert_eq!(result.unwrap(), Value::Bool(true), "strict={strict}");
    }
}

#[test]
fn pointer_events_cssom_alias_set_remove_and_priority() {
    check(
        r#"
      var s = document.getElementById('target').style;
      s.pointerEvents = 'none';
      var ok = s.pointerEvents === 'none' && s.getPropertyValue('pointer-events') === 'none';
      s.setProperty('pointer-events', 'AUTO', 'important');
      ok = ok && s.pointerEvents === 'auto' && s.getPropertyPriority('pointer-events') === 'important';
      s.setProperty('pointer-events', 'visiblePainted');
      ok = ok && s.pointerEvents === 'auto' && s.getPropertyPriority('pointer-events') === 'important';
      var old = s.removeProperty('pointer-events');
      ok && old === 'auto' && s.pointerEvents === '' && s.getPropertyPriority('pointer-events') === '';
    "#,
    );
}

#[test]
fn pointer_events_cssom_source_escapes_preserve_literal_api_names() {
    check(
        r#"
      var s = document.getElementById('target').style;
      s.cssText = 'pointer\\2d events: \\6e one; color:red';
      var ok = s.pointerEvents === 'none' && s.length === 2;
      s.setProperty('pointer\\2d events', 'auto');
      ok = ok && s.pointerEvents === 'none' && s.length === 2;
      s.pointerEvents = '';
      ok && s.pointerEvents === '' && s.length === 1 && s.color === 'red';
    "#,
    );
}

#[test]
fn pointer_events_cssom_pending_variable_and_all_reset() {
    check(
        r#"
      var s = document.getElementById('target').style;
      s.setProperty('pointer-events', 'var(--late)');
      var ok = s.getPropertyValue('pointer-events') === 'var(--late)';
      s.cssText = 'all:initial';
      ok && s.pointerEvents === 'initial' && s.length === 88;
    "#,
    );
}

#[test]
fn pointer_events_cssom_supports_both_forms() {
    check(
        r#"
      CSS.supports('pointer-events','auto') && CSS.supports('pointer-events','none') &&
      CSS.supports('pointer-events','\\6e one') && CSS.supports('(pointer\\2d events:none)') &&
      !CSS.supports('pointer\\2d events','none') && !CSS.supports('pointer-events','visiblePainted') &&
      !CSS.supports('pointer-events','no/**/ne') && !CSS.supports('pointer-events','none!important');
    "#,
    );
}
