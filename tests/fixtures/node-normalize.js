// Independent literal proposals. Invoke one nodeNormalizeCases body per fresh realm/mode.
// No candidate or baseline has executed these sources.
function normalizeCheck(value,message){if(!value)throw new Error(message);}
function normalizeUnits(value,expected){
  normalizeCheck(value.length===expected.length,'unit length');
  for(let i=0;i<expected.length;i++)normalizeCheck(value.charCodeAt(i)===expected[i],'unit '+i);
}
function normalizeChildren(parent,expected){
  const actual=parent.childNodes;normalizeCheck(actual.length===expected.length,'child count');
  for(let i=0;i<expected.length;i++)normalizeCheck(actual[i]===expected[i]&&expected[i].parentNode===parent,'child identity '+i);
}
function normalizeKeys(actual,expected){
  normalizeCheck(actual.length===expected.length,'key count');
  for(let i=0;i<expected.length;i++)normalizeCheck(actual[i]===expected[i],'key order '+i);
}
function normalizePrerequisite(){
  if(typeof Node!=='function'||typeof Text!=='function'||typeof DocumentFragment!=='function')
    throw new TypeError('Node/Text/Fragment prerequisite');
  const d=Object.getOwnPropertyDescriptor(Node.prototype,'normalize');
  if(!d||typeof d.value!=='function')throw new TypeError('Node normalize prerequisite');
  const box=new DocumentFragment(),a=new Text('a'),b=new Text('b');box.append(a,b);
  if(d.value.call(box)!==undefined||box.childNodes.length!==1||box.firstChild!==a||a.data!=='ab'||b.parentNode!==null||b.data!=='b')
    throw new TypeError('working normalize prerequisite');
  return d.value;
}
const normalizeConstantNames=[
  'ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE','ENTITY_REFERENCE_NODE','ENTITY_NODE',
  'PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE','DOCUMENT_FRAGMENT_NODE','NOTATION_NODE',
  'DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS',
  'DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC'
];
const nodeNormalizeCases={
  metadata: function(){
    const method=normalizePrerequisite(),d=Object.getOwnPropertyDescriptor(Node.prototype,'normalize');
    normalizeCheck(d.value===method&&d.writable&&d.enumerable&&d.configurable&&d.get===undefined&&d.set===undefined,'method descriptor');
    const name=Object.getOwnPropertyDescriptor(method,'name'),length=Object.getOwnPropertyDescriptor(method,'length');
    normalizeCheck(name.value==='normalize'&&!name.writable&&!name.enumerable&&name.configurable,'name');
    normalizeCheck(length.value===0&&!length.writable&&!length.enumerable&&length.configurable,'length');
    normalizeCheck(Object.getPrototypeOf(method)===Function.prototype&&Object.getOwnPropertyDescriptor(method,'prototype')===undefined,'ordinary method function');
    for(const node of [document,document.createElement('div'),new DocumentFragment(),new Text('x'),new Comment('x')]){
      normalizeCheck(node.normalize===method&&Object.getOwnPropertyDescriptor(node,'normalize')===undefined,'one inherited defining-interface method');
    }
    let caught=false;try{Reflect.construct(method,[]);}catch(e){caught=e instanceof TypeError;}
    normalizeCheck(caught,'method is not a constructor');return true;
  },
  represented_complete_key_order: function(){
    normalizePrerequisite();
    // Complete represented inventory after this proposed increment, not full DOM Node.
    const strings=['nodeValue','textContent','normalize'].concat(normalizeConstantNames,['constructor']);
    normalizeKeys(Object.getOwnPropertyNames(Node.prototype),strings);
    const all=Reflect.ownKeys(Node.prototype);normalizeCheck(all.length===23,'23 represented own keys');
    for(let i=0;i<22;i++)normalizeCheck(all[i]===strings[i],'prototype string position');
    normalizeCheck(all[22]===Symbol.toStringTag,'tag follows all string keys');
    normalizeKeys(Object.keys(Node.prototype),['nodeValue','textContent','normalize'].concat(normalizeConstantNames));
    const constructor=['length','name','prototype'].concat(normalizeConstantNames);
    normalizeKeys(Object.getOwnPropertyNames(Node),constructor);normalizeKeys(Reflect.ownKeys(Node),constructor);
    normalizeKeys(Object.keys(Node),normalizeConstantNames);return true;
  },
  receiver_is_not_a_descendant: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),empty=new Text(''),a=new Text('a'),b=new Text('b');
    box.append(empty,a,b);
    normalizeCheck(method.call(empty)===undefined&&method.call(a)===undefined,'Text receivers accepted');
    normalizeChildren(box,[empty,a,b]);normalizeCheck(empty.data===''&&a.data==='a'&&b.data==='b','receiver and siblings untouched');
    const detached=new Text('');normalizeCheck(method.call(detached)===undefined&&detached.parentNode===null&&detached.data==='','detached empty receiver remains');
    return true;
  },
  only_empty_descendants_are_removed: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),a=new Text(''),b=new Text(''),c=new Text(''),note=new Comment('');
    box.append(a,b,note,c);normalizeCheck(method.call(box)===undefined,'undefined return');normalizeChildren(box,[note]);
    for(const node of [a,b,c])normalizeCheck(node.parentNode===null&&node.data==='','empty Text detached unchanged');
    normalizeCheck(note.data===''&&note.nodeType===8,'empty Comment retained');
    normalizeCheck(method.call(box)===undefined,'second call');normalizeChildren(box,[note]);return true;
  },
  first_nonempty_survives_with_detached_data: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),e=new Text(''),a=new Text('A'),gap=new Text(''),b=new Text('B'),c=new Text('C'),end=new Text('');
    const survivor={},removed={};a.marker=survivor;b.marker=removed;
    box.append(e,a,gap,b,c,end);method.call(box);normalizeChildren(box,[a]);normalizeCheck(a.data==='ABC'&&a.marker===survivor,'first nonempty survives');
    normalizeCheck(b.parentNode===null&&b.data==='B'&&b.marker===removed&&c.parentNode===null&&c.data==='C','removed payloads and own bags remain');
    for(const node of [e,gap,end])normalizeCheck(node.parentNode===null&&node.data==='','all empties removed');return true;
  },
  exact_pair_repair_keeps_removed_units: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),a=new Text('A\uD83D'),empty=new Text(''),b=new Text('\uDE80B');
    box.append(a,empty,b);method.call(box);normalizeChildren(box,[a]);
    normalizeUnits(a.data,[65,55357,56960,66]);normalizeUnits(b.data,[56960,66]);
    normalizeCheck(empty.parentNode===null&&empty.data===''&&b.parentNode===null,'old nodes retained');
    method.call(box);normalizeChildren(box,[a]);normalizeUnits(a.data,[65,55357,56960,66]);return true;
  },
  exact_units_do_not_unicode_normalize: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),a=new Text('\uD800'),b=new Text('x\uDC00'),c=new Text('e'),d=new Text('\u0301\0');
    box.append(a,b,c,d);method.call(box);normalizeChildren(box,[a]);
    normalizeUnits(a.data,[55296,120,56320,101,769,0]);normalizeUnits(b.data,[120,56320]);normalizeUnits(c.data,[101]);normalizeUnits(d.data,[769,0]);
    for(const node of [b,c,d])normalizeCheck(node.parentNode===null,'removed exact node');return true;
  },
  barriers_and_nested_subtrees: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),a=new Text('a'),note=new Comment(''),b=new Text('b'),c=new Text('c');
    const element=document.createElement('span'),d=new Text('d'),e=new Text('e'),f=new Text('f'),pi=document.createProcessingInstruction('ok',''),g=new Text('g'),h=new Text('h');
    element.append(d,e);box.append(a,note,b,c,element,f,pi,g,h);method.call(box);
    normalizeChildren(box,[a,note,b,element,f,pi,g]);normalizeChildren(element,[d]);
    normalizeCheck(a.data==='a'&&b.data==='bc'&&d.data==='de'&&f.data==='f'&&g.data==='gh','separate runs');
    normalizeCheck(note.data===''&&pi.data===''&&pi.target==='ok','empty non-Text barriers retained');
    for(const row of [[c,'c'],[e,'e'],[h,'h']])normalizeCheck(row[0].parentNode===null&&row[0].data===row[1],'nested removed data');return true;
  },
  detached_fragment_is_idempotent: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),outer=document.createElement('section'),inner=document.createElement('span');
    const a=new Text('a'),b=new Text('b'),empty=new Text(''),x=new Text('x'),y=new Text('y'),c=new Text('c'),d=new Text('d');
    inner.append(empty,x,y);outer.append(inner);box.append(a,b,outer,c,d);method.call(box);
    normalizeChildren(box,[a,outer,c]);normalizeChildren(outer,[inner]);normalizeChildren(inner,[x]);
    normalizeCheck(box.parentNode===null&&a.data==='ab'&&x.data==='xy'&&c.data==='cd','detached nested outputs');
    for(const node of [b,empty,y,d])normalizeCheck(node.parentNode===null,'nested removals');
    method.call(box);normalizeChildren(box,[a,outer,c]);normalizeChildren(inner,[x]);
    normalizeCheck(a.data==='ab'&&x.data==='xy'&&c.data==='cd'&&b.data==='b'&&y.data==='y'&&d.data==='d','second call preserves IDs/data');return true;
  },
  document_descendants_without_detached_arena_scan: function(){
    const method=normalizePrerequisite(),box=document.createElement('div'),a=new Text('a'),b=new Text('b');
    const detached=new DocumentFragment(),x=new Text('x'),y=new Text('y');detached.append(x,y);box.append(a,b);
    const body=document.body;if(!body||typeof body.append!=='function')throw new TypeError('HTML body append prerequisite');
    body.append(box);normalizeCheck(method.call(document)===undefined,'Document brand');normalizeChildren(box,[a]);
    normalizeCheck(a.data==='ab'&&b.parentNode===null&&b.data==='b','Document descendant merge');
    normalizeChildren(detached,[x,y]);normalizeCheck(x.data==='x'&&y.data==='y','unrelated detached tree untouched');return true;
  },
  template_content_is_not_an_ordinary_child: function(){
    const method=normalizePrerequisite(),host=document.createElement('div'),template=document.createElement('template'),content=template.content;
    if(!content||content.nodeType!==11)throw new TypeError('actual template content prerequisite');
    const a=new Text('in'),b=new Text('side'),x=new Text('out'),y=new Text('side');content.append(a,b);template.append(x,y);host.append(template);
    method.call(host);normalizeChildren(template,[x]);normalizeCheck(x.data==='outside'&&y.parentNode===null&&y.data==='side','ordinary template descendants');
    normalizeChildren(content,[a,b]);normalizeCheck(a.data==='in'&&b.data==='side'&&content.parentNode===null,'associated fragment is not traversed');
    method.call(content);normalizeChildren(content,[a]);normalizeCheck(a.data==='inside'&&b.parentNode===null&&b.data==='side','explicit content normalization');
    normalizeChildren(template,[x]);normalizeCheck(template.content===content&&x.data==='outside','host/content independence');return true;
  },
  extra_arguments_are_not_converted: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),a=new Text('a'),b=new Text('b');box.append(a,b);
    let reads=0;const ignored={valueOf(){reads++;throw 'number';},toString(){reads++;throw 'string';}};
    ignored[Symbol.toPrimitive]=function(){reads++;throw 'primitive';};
    normalizeCheck(method.call(box,ignored,Symbol('unused'),null)===undefined,'extra arguments ignored');
    normalizeCheck(reads===0,'no argument conversions');normalizeChildren(box,[a]);normalizeCheck(a.data==='ab','ordinary mutation still occurs');return true;
  },
  authentic_brands_and_saved_leaf_calls: function(){
    const method=normalizePrerequisite();let reads=0;const ignored={toString(){reads++;throw 'coercion';}};
    for(const receiver of [null,undefined,0,'x',{},Object.create(Node.prototype),Node.prototype,EventTarget.prototype,window]){
      let caught=false;try{method.call(receiver,ignored);}catch(e){caught=e instanceof TypeError;}
      normalizeCheck(caught,'forged/non-Node receiver rejected');
    }
    normalizeCheck(reads===0,'forged receiver never converts unused argument');
    const text=new Text('\uD800'),comment=new Comment('kept'),pi=document.createProcessingInstruction('ok','kept');
    for(const node of [text,comment,pi])normalizeCheck(method.call(node,ignored)===undefined&&node.parentNode===null,'authentic leaf accepted');
    normalizeUnits(text.data,[55296]);normalizeCheck(comment.data==='kept'&&pi.data==='kept'&&reads===0,'leaf data retained');return true;
  },
  saved_method_survives_replacement_and_deletion: function(){
    const method=normalizePrerequisite(),descriptor=Object.getOwnPropertyDescriptor(Node.prototype,'normalize'),box=new DocumentFragment(),a=new Text('a'),b=new Text('b');
    box.append(a,b);let calls=0;const replacement=function(){calls++;return 17;};
    try{
      Node.prototype.normalize=replacement;normalizeCheck(box.normalize()===17&&calls===1,'ordinary replacement');normalizeChildren(box,[a,b]);
      normalizeCheck(method.call(box)===undefined&&a.data==='ab'&&b.parentNode===null,'saved original survives');
      normalizeCheck(delete Node.prototype.normalize,'prototype deletion');
      normalizeCheck(Object.getOwnPropertyDescriptor(Node.prototype,'normalize')===undefined&&box.normalize===undefined,'no synthesized resurrection');
      normalizeCheck(method.call(box)===undefined,'saved call after deletion');
      Object.defineProperty(box,'normalize',{value:replacement,writable:true,enumerable:true,configurable:true});
      normalizeCheck(box.normalize()===17&&calls===2,'own override');
      normalizeCheck(delete box.normalize&&box.normalize===undefined,'own deletion stays absent');
    }finally{Object.defineProperty(Node.prototype,'normalize',descriptor);}
    normalizeCheck(box.normalize===method,'prototype descriptor restored');return true;
  },
  internal_slots_ignore_authored_getters: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),a=new Text('a'),b=new Text('b');box.append(a,b);
    const getData=Object.getOwnPropertyDescriptor(CharacterData.prototype,'data').get;
    const getContent=Object.getOwnPropertyDescriptor(Node.prototype,'textContent').get;
    let reads=0;function poison(){reads++;throw 'authored getter';}
    for(const key of ['childNodes','firstChild','lastChild','textContent'])Object.defineProperty(box,key,{get:poison,configurable:true});
    for(const node of [a,b])for(const key of ['data','length','parentNode','nextSibling','previousSibling'])Object.defineProperty(node,key,{get:poison,configurable:true});
    normalizeCheck(method.call(box)===undefined&&reads===0,'no public property reads');
    normalizeCheck(getData.call(a)==='ab'&&getData.call(b)==='b'&&getContent.call(box)==='ab','saved exact getters');
    for(const key of ['childNodes','firstChild','lastChild','textContent'])delete box[key];
    for(const node of [a,b])for(const key of ['data','length','parentNode','nextSibling','previousSibling'])delete node[key];
    normalizeChildren(box,[a]);normalizeCheck(b.parentNode===null&&reads===0,'actual internal links');return true;
  },
  removed_nodes_reuse_their_original_data: function(){
    const method=normalizePrerequisite(),box=new DocumentFragment(),a=new Text('A'),b=new Text('\uD800'),c=new Text('\uDC00');box.append(a,b,c);
    method.call(box);normalizeChildren(box,[a]);normalizeUnits(a.data,[65,55296,56320]);
    normalizeCheck(b.parentNode===null&&c.parentNode===null,'removed nodes detached');normalizeUnits(b.data,[55296]);normalizeUnits(c.data,[56320]);
    const other=new DocumentFragment();other.append(c,b);method.call(other);normalizeChildren(other,[c]);normalizeUnits(c.data,[56320,55296]);
    normalizeCheck(b.parentNode===null,'reused suffix detached again');normalizeUnits(b.data,[55296]);normalizeUnits(a.data,[65,55296,56320]);return true;
  }
};
