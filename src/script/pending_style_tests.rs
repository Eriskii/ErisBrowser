//! Independent Script/DOM wiring witnesses. Frozen before wiring implementation.
use super::*;

fn document() -> Document {
    Document::parse("<body><div id=target></div></body>")
}

fn partial(doc: &mut Document) -> NodeId {
    assert_eq!(
        Runtime::new().execute(
            "var s=document.getElementById('target').style; s.cssText='--m:1px 2px 3px 4px;margin:var(--m)!important'; s.marginLeft='9px'; s.marginLeft==='9px';",
            doc,
        ).unwrap(),
        Value::Bool(true),
    );
    doc.query_selector("#target").unwrap()
}

#[test]
fn pending_setter_conversions_use_latest_typed_block_in_order() {
    let mut doc = document();
    let mut runtime = Runtime::new();
    assert_eq!(
        runtime
            .execute(
                r#"
        var s=document.getElementById('target').style;
        s.cssText='--m:1px 2px 3px 4px;margin:var(--m)!important';
        var order='';
        s.setProperty(
            {toString(){order+='n';s.setProperty('--m','5px 6px 7px 8px');return 'margin-left';}},
            {toString(){order+='v';s.padding='2px';return '9px';}},
            {toString(){order+='p';s.setProperty('margin-right','10px','important');return '';}}
        );
        order==='nvp' && s.length===9 &&
        s.item(0)==='--m' && s.item(1)==='margin-top' &&
        s.item(2)==='margin-right' && s.item(3)==='margin-bottom' &&
        s.item(4)==='margin-left' && s.item(5)==='padding-top' &&
        s.item(8)==='padding-left' && s.margin==='' &&
        s.marginTop==='' && s.marginBottom==='' &&
        s.getPropertyPriority('margin-top')==='important' &&
        s.marginRight==='10px' && s.getPropertyPriority('margin-right')==='important' &&
        s.marginLeft==='9px' && s.getPropertyPriority('margin-left')==='' &&
        s.paddingTop==='2px' && s.getPropertyValue('--m')==='5px 6px 7px 8px';
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
    // A failed outer conversion keeps completed nested mutations, but must not
    // run later conversions or commit an outer declaration.
    assert_eq!(
        runtime
            .execute(
                r#"
        order='';
        try {
            s.setProperty('margin-left',
                {toString(){order+='v';s.setProperty('--m','11px');throw 42;}},
                {toString(){order+='p';return 'important';}});
        } catch(e) {order+=e;}
        order==='v42' && s.marginLeft==='9px' &&
        s.getPropertyPriority('margin-left')==='' &&
        s.marginTop==='' && s.getPropertyPriority('margin-top')==='important' &&
        s.getPropertyValue('--m')==='11px';
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn property_and_css_text_setters_finish_conversions_before_staging() {
    let mut doc = document();
    partial(&mut doc);
    assert_eq!(Runtime::new().execute(r#"
        var e=document.getElementById('target');var s=e.style;var order='';
        s.marginLeft={toString(){order+='v';s.setProperty('margin-right','10px','important');return '11px';}};
        var retained=s.marginLeft==='11px' && s.marginRight==='10px' &&
            s.marginTop==='' && s.getPropertyPriority('margin-top')==='important';
        s.cssText={toString(){order+='c';s.setProperty('--nested','retained-until-replacement');e.setAttribute('data-side-effect','yes');return 'margin:var(--n,2px)!important';}};
        retained && order==='vc' && e.getAttribute('data-side-effect')==='yes' &&
        s.length===4 && s.margin==='var(--n,2px)' &&
        s.getPropertyPriority('margin')==='important' && s.getPropertyValue('--nested')==='';
    "#, &mut doc).unwrap(), Value::Bool(true));
}

#[test]
fn partial_pending_state_survives_subsequent_accesses_and_runtimes() {
    let mut doc = document();
    let id = partial(&mut doc);
    assert_eq!(
        Runtime::new()
            .execute(
                r#"
        var s=document.getElementById('target').style;
        var removed=s.removeProperty('margin-bottom');
        s.setProperty('--m','5px 6px 7px 8px');
        s.color='blue';
        removed==='' && s.length===5 && s.item(0)==='--m' &&
        s.item(1)==='margin-top' && s.item(2)==='margin-right' &&
        s.item(3)==='margin-left' && s.item(4)==='color';
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
    let literal = "--m: 5px 6px 7px 8px; margin-top:  !important; margin-right:  !important; margin-left: 9px; color: blue;";
    assert_eq!(doc.attr(id, "style"), Some(literal));
    assert_eq!(Runtime::new().execute(r#"
        var s=document.getElementById('target').style;
        s.margin==='' && s.marginTop==='' && s.marginRight==='' &&
        s.getPropertyPriority('margin-top')==='important' &&
        s.getPropertyPriority('margin-right')==='important' &&
        s.getPropertyPriority('margin-bottom')==='' && s.marginLeft==='9px' &&
        s.cssText==='--m: 5px 6px 7px 8px; margin-top:  !important; margin-right:  !important; margin-left: 9px; color: blue;';
    "#, &mut doc).unwrap(), Value::Bool(true));
}

#[test]
fn same_text_external_style_write_and_css_text_assignment_reparse() {
    let pending = "--m: 1px 2px 3px 4px; margin-top:  !important; margin-right:  !important; margin-bottom:  !important; margin-left: 9px;";
    let reparsed = "--m: 1px 2px 3px 4px; margin-left: 9px;";
    for (operation, attribute) in [
        ("e.setAttribute('style',e.getAttribute('style'));", pending),
        ("s.cssText=s.cssText;", reparsed),
    ] {
        let mut doc = document();
        let id = partial(&mut doc);
        assert_eq!(doc.attr(id, "style"), Some(pending));
        let source = format!(
            "var e=document.getElementById('target');var s=e.style;{operation} s.length===2 && s.item(0)==='--m' && s.item(1)==='margin-left' && s.marginTop==='' && s.getPropertyPriority('margin-top')==='' && s.marginLeft==='9px' && s.cssText==='--m: 1px 2px 3px 4px; margin-left: 9px;';"
        );
        assert_eq!(
            Runtime::new().execute(&source, &mut doc).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(doc.attr(id, "style"), Some(attribute));
    }
}

#[test]
fn external_style_invalidation_respects_html_case_and_foreign_namespace() {
    let mut doc = Document::parse("<div id=h></div><svg id=s></svg>");
    assert_eq!(
        Runtime::new()
            .execute(
                r#"
        var h=document.getElementById('h');var s=document.getElementById('s');
        h.style.cssText='margin:var(--m,1px)!important';h.style.marginLeft='9px';
        s.style.cssText='margin:var(--m,1px)!important';s.style.marginLeft='9px';
        h.setAttribute('STYLE',h.getAttribute('style'));
        s.setAttribute('STYLE','margin:2px');
        h.style.length===1 && h.style.getPropertyPriority('margin-top')==='' &&
        s.style.length===4 && s.style.getPropertyPriority('margin-top')==='important';
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
    let svg = doc.query_selector("#s").unwrap();
    let before = doc.attr(svg, "style").unwrap().to_owned();
    doc.set_attr_ns(
        svg,
        crate::dom::AttributeNamespace::Xml,
        "xml:space",
        "preserve",
    );
    // Current represented namespace API does not admit an Xml-qualified style.
    doc.set_attr_ns(
        svg,
        crate::dom::AttributeNamespace::Xml,
        "style",
        "margin:3px",
    );
    assert_eq!(doc.attr(svg, "style"), Some(before.as_str()));
    assert_eq!(
        Runtime::new()
            .execute(
                r#"
        var s=document.getElementById('s');
        var retained=s.style.length===4 && s.style.getPropertyPriority('margin-top')==='important';
        s.setAttribute('style',s.getAttribute('style'));
        retained && s.style.length===1 && s.style.marginLeft==='9px' &&
        s.style.getPropertyPriority('margin-top')==='';
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn clone_node_reparses_attributes_but_moving_original_preserves_pending() {
    let mut doc = document();
    partial(&mut doc);
    assert_eq!(
        Runtime::new()
            .execute(
                r#"
        var e=document.getElementById('target');var s=e.style;
        var clone=e.cloneNode(false);
        var textEqual=clone.getAttribute('style')===e.getAttribute('style');
        document.body.removeChild(e);document.body.appendChild(e);
        textEqual && clone.style.length===2 && clone.style.marginLeft==='9px' &&
        clone.style.getPropertyPriority('margin-top')==='' &&
        s.length===5 && s.getPropertyPriority('margin-top')==='important';
    "#,
                &mut doc
            )
            .unwrap(),
        Value::Bool(true)
    );
}
