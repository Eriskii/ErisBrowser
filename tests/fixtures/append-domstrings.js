// Unexecuted independent sketches. Invoke one named body per fresh realm/mode.
function appendExactCheck(value, message) { if (!value) throw new Error(message); }
function appendExactUnits(value, expected) {
  appendExactCheck(value.length === expected.length, 'unit length');
  for (let i=0; i<expected.length; i++)
    appendExactCheck(value.charCodeAt(i) === expected[i], 'unit '+i);
}
var appendExactCases = {
  separate_text_arguments_keep_units_and_identity: function () {
    const box=document.createElement('div'), marker=document.createElement('span');
    box.append('\uD800', marker, '\uDC00');
    appendExactCheck(box.childNodes.length===3 && box.childNodes[1]===marker, 'three children');
    const high=box.childNodes[0], low=box.childNodes[2];
    appendExactCheck(high!==low && high.nodeType===3 && low.nodeType===3, 'separate Texts');
    appendExactUnits(high.data,[55296]); appendExactUnits(low.data,[56320]);
    appendExactUnits(box.textContent,[55296,56320]);
    return true;
  },
  empty_and_nullish_are_actual_string_arguments: function () {
    const box=document.createElement('div'); box.append();
    appendExactCheck(box.childNodes.length===0, 'zero arguments');
    box.append(undefined,null,'');
    appendExactCheck(box.childNodes.length===3, 'three materialized Texts');
    appendExactCheck(box.childNodes[0].data==='undefined' && box.childNodes[1].data==='null', 'nonnull DOMString conversion');
    appendExactUnits(box.childNodes[2].data,[]);
    box.append('A\uD800B\uDC00');
    appendExactUnits(box.childNodes[3].data,[65,55296,66,56320]);
    return true;
  },
  conversions_complete_before_moves_and_nodes_are_not_coerced: function () {
    const old=document.createElement('aside'), box=document.createElement('div');
    const child=document.createElement('i'); old.append(child); let trace='';
    child.toString=function(){throw new Error('Node coerced');};
    const a={toString(){trace+='A';appendExactCheck(child.parentNode===old,'first before move');return '\uD800';}};
    const b={toString(){trace+='B';appendExactCheck(child.parentNode===old,'second before move');box.append(new Text('prefix'));return '\uDC00';}};
    box.append(child,a,b);
    appendExactCheck(trace==='AB' && box.childNodes.length===4 && box.childNodes[1]===child, 'conversion sequence and prefix');
    appendExactCheck(box.childNodes[0].data==='prefix' && old.firstChild===null, 'callback prefix retained');
    appendExactUnits(box.childNodes[2].data,[55296]); appendExactUnits(box.childNodes[3].data,[56320]);
    return true;
  },
  abrupt_later_conversion_keeps_author_prefix_without_moves: function () {
    const old=document.createElement('aside'), box=document.createElement('div'), child=document.createElement('i');
    old.append(child); let trace='',observed;
    const a={toString(){trace+='A';box.append(new Text('\uD800'));return '\uDC00';}};
    const later={toString(){trace+='L';return 'later';}};
    try{box.append(child,a,Symbol('stop'),later);}catch(error){observed=error;}
    appendExactCheck(observed instanceof TypeError && trace==='A', 'abrupt conversion');
    appendExactCheck(child.parentNode===old && box.childNodes.length===1, 'no outer moves or strings');
    appendExactUnits(box.firstChild.data,[55296]);
    return true;
  },
  final_document_failure_retains_exact_assembled_fragment: function () {
    const old=document.createElement('aside'), child=document.createElement('i');old.append(child);
    const root=document.documentElement;let observed;
    try{document.append(child,'\uD800');}catch(error){observed=error;}
    appendExactCheck(observed && observed.name==='HierarchyRequestError' && observed.code===3, 'Document hierarchy');
    const temporary=child.parentNode;
    appendExactCheck(temporary && temporary.nodeType===11 && temporary.parentNode===null, 'temporary owner');
    appendExactCheck(old.firstChild===null && temporary.childNodes.length===2 && temporary.firstChild===child, 'assembly prefix retained');
    appendExactUnits(temporary.childNodes[1].data,[55296]);
    appendExactCheck(document.documentElement===root && root.parentNode===document, 'unrelated root retained');
    return true;
  },
  cycle_failure_keeps_full_exact_argument_order: function () {
    const holder=document.createElement('aside'), target=document.createElement('section'), child=document.createElement('i');
    holder.append(target);target.append(child);let observed;
    try{target.append('\uD800',child,target,'\uDC00');}catch(error){observed=error;}
    appendExactCheck(observed && observed.name==='HierarchyRequestError' && observed.code===3, 'cycle error');
    const temporary=target.parentNode;
    appendExactCheck(temporary && temporary.nodeType===11 && temporary.parentNode===null && temporary.childNodes.length===4, 'temporary fragment');
    appendExactCheck(holder.firstChild===null && target.firstChild===null && temporary.childNodes[1]===child && temporary.childNodes[2]===target, 'completed moves');
    appendExactUnits(temporary.childNodes[0].data,[55296]);appendExactUnits(temporary.childNodes[3].data,[56320]);
    return true;
  },
  defining_interface_brand_precedes_string_conversion: function () {
    const box=document.createElement('div');box.append('guard');
    appendExactCheck(box.firstChild.data==='guard', 'working append prerequisite');
    const documentAppend=document.append;
    appendExactCheck(typeof documentAppend==='function', 'Document append callable');
    appendExactCheck(documentAppend.call(document)===undefined, 'working Document append prerequisite');
    let conversions=0,observed;
    const value={toString(){conversions++;return '\uD800';}};
    try{documentAppend.call(box,value);}catch(error){observed=error;}
    appendExactCheck(observed instanceof TypeError && conversions===0 && box.childNodes.length===1, 'brand first');
    appendExactCheck(box.firstChild.data==='guard', 'unchanged receiver');
    return true;
  }
};
