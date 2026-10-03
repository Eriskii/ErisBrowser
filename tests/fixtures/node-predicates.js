// Independent source-only proposals; invoke one body per fresh realm and mode.
// Expected values below are literals, not observations from any engine.
function predicateCheck(value,message){if(!value)throw new Error(message);}
function predicateTypeError(body,message){
  let caught=false;try{body();}catch(e){caught=e instanceof TypeError;}
  predicateCheck(caught,message);
}
function predicateKeys(actual,expected){
  predicateCheck(actual.length===expected.length,'key count');
  for(let i=0;i<expected.length;i++)predicateCheck(actual[i]===expected[i],'key '+i);
}
function predicatePrerequisite(){
  if(typeof Node!=='function'||typeof Text!=='function'||typeof DocumentFragment!=='function')throw new TypeError('Node constructors prerequisite');
  const names=['hasChildNodes','isSameNode','contains'],methods=[];
  for(const name of names){
    const d=Object.getOwnPropertyDescriptor(Node.prototype,name);
    if(!d||typeof d.value!=='function')throw new TypeError('Node predicate prerequisite '+name);
    methods.push(d.value);
  }
  const box=new DocumentFragment(),a=new Text('x'),b=new Text('x');
  if(methods[0].call(box)!==false||methods[1].call(a,a)!==true||methods[1].call(a,b)!==false||methods[2].call(a,a)!==true||methods[2].call(a,null)!==false)
    throw new TypeError('working leaf predicate prerequisite');
  box.append(a);
  if(methods[0].call(box)!==true||methods[2].call(box,a)!==true||methods[2].call(a,box)!==false)
    throw new TypeError('working child predicate prerequisite');
  return methods;
}
const predicateConstantNames=[
  'ELEMENT_NODE','ATTRIBUTE_NODE','TEXT_NODE','CDATA_SECTION_NODE','ENTITY_REFERENCE_NODE','ENTITY_NODE',
  'PROCESSING_INSTRUCTION_NODE','COMMENT_NODE','DOCUMENT_NODE','DOCUMENT_TYPE_NODE','DOCUMENT_FRAGMENT_NODE','NOTATION_NODE',
  'DOCUMENT_POSITION_DISCONNECTED','DOCUMENT_POSITION_PRECEDING','DOCUMENT_POSITION_FOLLOWING','DOCUMENT_POSITION_CONTAINS',
  'DOCUMENT_POSITION_CONTAINED_BY','DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC'
];
const nodePredicateCases={
  metadata: function(){
    const methods=predicatePrerequisite(),names=['hasChildNodes','isSameNode','contains'],lengths=[0,1,1];
    const nodes=[document,document.createElement('div'),new DocumentFragment(),new Text('x'),new Comment('x')];
    for(let i=0;i<3;i++){
      const name=names[i],method=methods[i],d=Object.getOwnPropertyDescriptor(Node.prototype,name);
      predicateCheck(d.value===method&&d.writable&&d.enumerable&&d.configurable&&d.get===undefined&&d.set===undefined,'owner flags');
      const n=Object.getOwnPropertyDescriptor(method,'name'),l=Object.getOwnPropertyDescriptor(method,'length');
      predicateCheck(n.value===name&&!n.writable&&!n.enumerable&&n.configurable,'name flags');
      predicateCheck(l.value===lengths[i]&&!l.writable&&!l.enumerable&&l.configurable,'length flags');
      predicateKeys(Reflect.ownKeys(method),['length','name']);
      predicateCheck(Object.getPrototypeOf(method)===Function.prototype,'Function prototype');
      for(const node of nodes)predicateCheck(node[name]===method&&Object.getOwnPropertyDescriptor(node,name)===undefined,'one inherited method');
      predicateTypeError(function(){Reflect.construct(method,[]);},'nonconstructable method');
    }
    predicateCheck(methods[0]!==methods[1]&&methods[1]!==methods[2]&&methods[0]!==methods[2],'distinct methods');return true;
  },
  represented_complete_key_order: function(){
    predicatePrerequisite();
    const operations=['hasChildNodes','normalize','isSameNode','contains'];
    const strings=['nodeValue','textContent'].concat(operations,predicateConstantNames,['constructor']);
    predicateKeys(Object.getOwnPropertyNames(Node.prototype),strings);
    const keys=Reflect.ownKeys(Node.prototype);predicateCheck(keys.length===26,'26 represented keys');
    for(let i=0;i<25;i++)predicateCheck(keys[i]===strings[i],'prototype string order');
    predicateCheck(keys[25]===Symbol.toStringTag,'final tag');
    predicateKeys(Object.keys(Node.prototype),['nodeValue','textContent'].concat(operations,predicateConstantNames));
    const constructor=['length','name','prototype'].concat(predicateConstantNames);
    predicateKeys(Reflect.ownKeys(Node),constructor);predicateKeys(Object.getOwnPropertyNames(Node),constructor);
    predicateKeys(Object.keys(Node),predicateConstantNames);return true;
  },
  direct_children_include_empty_text_and_comments: function(){
    const m=predicatePrerequisite(),box=new DocumentFragment(),empty=new Text(''),note=new Comment('');
    predicateCheck(m[0].call(box)===false&&m[0].call(empty)===false&&m[0].call(note)===false,'empty and leaf results');
    box.append(empty);predicateCheck(m[0].call(box)===true,'empty Text is a child');
    box.textContent='';predicateCheck(m[0].call(box)===false&&empty.parentNode===null,'detached empty child');
    box.append(note);predicateCheck(m[0].call(box)===true,'Comment is a child');
    const parked=new DocumentFragment();parked.append(note);
    predicateCheck(m[0].call(box)===false&&m[0].call(parked)===true&&note.parentNode===parked,'fresh child state');return true;
  },
  contains_inclusive_direction_and_disjoint_trees: function(){
    const m=predicatePrerequisite(),outer=new DocumentFragment(),inner=document.createElement('div'),leaf=new Text('x'),sibling=new Text('y'),other=new DocumentFragment();
    inner.append(leaf);outer.append(inner,sibling);
    predicateCheck(m[2].call(outer,outer)===true&&m[2].call(inner,inner)===true&&m[2].call(leaf,leaf)===true,'inclusive self');
    predicateCheck(m[2].call(outer,inner)===true&&m[2].call(outer,leaf)===true&&m[2].call(inner,leaf)===true,'ancestors');
    predicateCheck(m[2].call(leaf,outer)===false&&m[2].call(inner,outer)===false&&m[2].call(sibling,leaf)===false,'direction and siblings');
    predicateCheck(m[2].call(other,leaf)===false&&m[2].call(outer,other)===false,'different roots');
    other.append(inner);predicateCheck(m[2].call(outer,leaf)===false&&m[2].call(other,leaf)===true&&m[2].call(inner,leaf)===true,'moved subtree');return true;
  },
  same_node_uses_identity_not_data_or_position: function(){
    const m=predicatePrerequisite(),a=new Text('\uD800x'),b=new Text('\uD800x'),left=new DocumentFragment(),right=new DocumentFragment();
    predicateCheck(m[1].call(a,a)===true&&m[1].call(a,b)===false&&m[1].call(b,a)===false,'equal exact data has distinct identity');
    left.append(a);const alias=left.firstChild;
    predicateCheck(m[1].call(a,alias)===true&&m[1].call(alias,a)===true,'retrieved alias');
    right.append(a);a.data='changed';
    predicateCheck(m[1].call(alias,a)===true&&m[1].call(a,b)===false&&m[2].call(left,a)===false&&m[2].call(right,a)===true,'identity survives mutation and move');return true;
  },
  required_arguments_differ_from_nullable_values: function(){
    const m=predicatePrerequisite(),a=new Text('x');
    predicateCheck(m[0].call(a)===false,'zero argument operation');
    for(const method of [m[1],m[2]]){
      predicateTypeError(function(){method.call(a);},'missing required Node argument');
      predicateCheck(method.call(a,null)===false&&method.call(a,undefined)===false,'explicit nullable argument');
    }
    return true;
  },
  interface_arguments_do_not_coerce: function(){
    const m=predicatePrerequisite(),a=new Text('x');let seen=0;
    const hostile={};
    for(const key of [Symbol.toPrimitive,'valueOf','toString','nodeType','parentNode','prototype'])Object.defineProperty(hostile,key,{get:function(){seen++;throw new Error('no interface coercion');},configurable:true});
    for(const method of [m[1],m[2]]){
      for(const bad of [hostile,{},Object.create(Node.prototype),0,1,false,'x',Symbol('node'),[]])predicateTypeError(function(){method.call(a,bad);},'authentic interface argument');
    }
    predicateCheck(seen===0,'no conversion hooks read');return true;
  },
  receiver_brand_cannot_be_forged: function(){
    const m=predicatePrerequisite(),a=new Text('x');let seen=0;
    const fake=Object.create(Node.prototype);Object.defineProperty(fake,'nodeType',{get:function(){seen++;return 3;}});
    for(const bad of [null,undefined,globalThis,Node.prototype,fake,{},0,'x',true]){
      for(const method of m)predicateTypeError(function(){method.call(bad,a);},'authentic receiver');
    }
    predicateCheck(seen===0,'brand does not inspect nodeType');return true;
  },
  ignored_extra_arguments_are_evaluated_but_not_converted: function(){
    const m=predicatePrerequisite(),box=new DocumentFragment(),a=new Text('x');let calls=0,coercions=0;
    const poison={toString:function(){coercions++;throw new Error('unused toString');},valueOf:function(){coercions++;throw new Error('unused valueOf');}};
    function insert(){calls++;box.append(a);return poison;}
    predicateCheck(m[0].call(box,insert())===true&&calls===1,'arguments evaluated before zero-arity call');
    predicateCheck(m[1].call(a,a,poison)===true&&m[2].call(box,a,poison)===true&&coercions===0,'extra values not converted');return true;
  },
  argument_expressions_finish_before_tree_observation: function(){
    const m=predicatePrerequisite(),left=new DocumentFragment(),right=new DocumentFragment(),a=new Text('x');left.append(a);let trace='';
    function argument(){trace+='a';return a;}
    function extra(){trace+='b';right.append(a);return {};}
    predicateCheck(m[2].call(left,argument(),extra())===false&&trace==='ab','later ignored expression can move the node');
    predicateCheck(m[2].call(right,a)===true&&m[1].call(a,a)===true,'fresh tree and same identity');
    let reached=false;const marker={};
    function fail(){reached=true;throw marker;}
    let caught=false;try{m[2].call({},fail());}catch(e){caught=e===marker;}
    predicateCheck(caught&&reached,'JS argument evaluation precedes native brand check');return true;
  },
  template_content_is_a_separate_tree: function(){
    const m=predicatePrerequisite(),host=document.createElement('div'),template=document.createElement('template'),content=template.content;
    if(!content||content.nodeType!==11)throw new TypeError('actual template content prerequisite');
    const inside=new Text('inside'),ordinary=new Text('ordinary');content.append(inside);host.append(template);
    predicateCheck(m[0].call(content)===true&&m[0].call(template)===false,'content is not an ordinary child');
    predicateCheck(m[2].call(template,inside)===false&&m[2].call(host,inside)===false&&m[2].call(template,content)===false,'no template-host jump');
    predicateCheck(m[2].call(content,inside)===true&&m[2].call(content,content)===true&&m[1].call(content,template)===false,'separate content identity');
    template.append(ordinary);
    predicateCheck(m[0].call(template)===true&&m[2].call(template,ordinary)===true&&m[2].call(host,ordinary)===true&&m[2].call(content,ordinary)===false,'ordinary template children');return true;
  },
  document_and_public_parent_aliases: function(){
    const m=predicatePrerequisite(),root=document.documentElement,body=document.body;
    if(!root||!body)throw new TypeError('HTML document root prerequisite');
    const alias=root.parentNode,box=document.createElement('div'),a=new Text('x'),detached=new DocumentFragment();box.append(a);body.append(box);
    predicateCheck(alias===document&&m[1].call(document,alias)===true&&m[1].call(alias,document)===true,'public Document identity');
    predicateCheck(m[0].call(document)===true&&m[2].call(document,document)===true&&m[2].call(document,a)===true&&m[2].call(a,document)===false,'Document tree');
    detached.append(box);predicateCheck(m[2].call(document,a)===false&&m[2].call(detached,a)===true&&m[1].call(a,box.firstChild)===true,'detached document descendant');return true;
  },
  saved_calls_survive_shadow_replacement_and_deletion: function(){
    const m=predicatePrerequisite(),a=new Text('x'),names=['hasChildNodes','isSameNode','contains'];
    for(let i=0;i<3;i++){
      const name=names[i],method=m[i],descriptor=Object.getOwnPropertyDescriptor(Node.prototype,name),replacement=function(){return 'authored';};
      a[name]=replacement;predicateCheck(a[name]() ==='authored'&&Node.prototype[name]===method,'own shadow');delete a[name];
      Node.prototype[name]=replacement;predicateCheck(a[name]===replacement,'prototype replacement');
      predicateCheck(method.call(a,a)===(i===0?false:true),'saved native remains authentic');
      delete Node.prototype[name];predicateCheck(a[name]===undefined&&Object.getOwnPropertyDescriptor(Node.prototype,name)===undefined,'no fallback resurrection');
      predicateCheck(method.call(a,a)===(i===0?false:true),'saved native after deletion');
      Object.defineProperty(Node.prototype,name,descriptor);predicateCheck(a[name]===method,'restored native');
    }
    return true;
  },
  native_relations_ignore_authored_getters: function(){
    const m=predicatePrerequisite(),box=new DocumentFragment(),a=new Text('x');box.append(a);let seen=0;
    for(const node of [box,a])for(const key of ['childNodes','firstChild','lastChild','parentNode','nodeType','length'])Object.defineProperty(node,key,{get:function(){seen++;throw new Error('authored getter');},configurable:true});
    predicateCheck(m[0].call(box)===true&&m[0].call(a)===false&&m[2].call(box,a)===true&&m[2].call(a,box)===false&&m[1].call(a,a)===true,'internal graph slots');
    predicateCheck(seen===0,'no authored tree getters');return true;
  },
  authentic_alternate_prototype_still_has_node_brand: function(){
    const m=predicatePrerequisite(),newTarget=(function(){}).bind(null),prototype=Object.create(null);
    Object.defineProperty(newTarget,'prototype',{value:prototype});const a=Reflect.construct(Text,['x'],newTarget),box=new DocumentFragment();
    predicateCheck(Object.getPrototypeOf(a)===prototype&&a.contains===undefined,'actual alternate prototype');
    predicateCheck(m[0].call(a)===false&&m[1].call(a,a)===true&&m[2].call(a,a)===true,'authentic alternate-prototype node');
    box.append(a);predicateCheck(m[2].call(box,a)===true,'alternate-prototype node as argument');
    predicateTypeError(function(){m[2].call(box,prototype);},'prototype object is not a Node');return true;
  },
  represented_leaf_kinds_and_detached_payloads: function(){
    const m=predicatePrerequisite(),box=new DocumentFragment(),a=new Text('\uD800'),b=new Comment('\uDC00'),pi=document.createProcessingInstruction('ok','?');
    box.append(a,b,pi);const nodes=[a,b,pi];
    for(const node of nodes)predicateCheck(m[0].call(node)===false&&m[1].call(node,node)===true&&m[2].call(box,node)===true,'represented leaf brands');
    box.textContent='';
    predicateCheck(m[0].call(box)===false&&a.data.charCodeAt(0)===55296&&b.data.charCodeAt(0)===56320&&pi.data==='?','retained payloads');
    for(const node of nodes)predicateCheck(node.parentNode===null&&m[2].call(box,node)===false&&m[2].call(node,node)===true&&m[1].call(node,node)===true,'detached identity');return true;
  }
};
