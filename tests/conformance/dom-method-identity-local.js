// eight-function-identity-matrix [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var again=[document.querySelector,document.querySelectorAll,o.querySelector,o.querySelectorAll,g.querySelector,g.querySelectorAll,o.append,g.append];
for(var i=0;i<8;i++){assert.sameValue(methods[i]===again[i],true);assert.sameValue(Object.is(methods[i],again[i]),true);for(var j=0;j<8;j++){assert.sameValue(methods[i]===methods[j],i===j);assert.sameValue(Object.is(methods[i],methods[j]),i===j);}}

// element-identity-across-tags-and-namespace [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var box=document.createElement('div');box.innerHTML='<svg><g></g></svg>';var svg=box.firstChild;
assert.sameValue(svg.namespaceURI,'http://www.w3.org/2000/svg');
assert.sameValue(eq,o.querySelector);assert.sameValue(eq,svg.querySelector);assert.sameValue(ea,svg.querySelectorAll);assert.sameValue(ep,svg.append);
assert.sameValue(svg.querySelector('g'),svg.firstChild);assert.sameValue(svg.querySelectorAll('g')[0],svg.firstChild);

// fragment-and-template-content-identity [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var template=document.createElement('template');template.innerHTML='<i></i>';var content=template.content;
assert.sameValue(content.nodeType,11);assert.sameValue(fq,g.querySelector);assert.sameValue(fq,content.querySelector);assert.sameValue(fa,content.querySelectorAll);assert.sameValue(fp,content.append);
assert.sameValue(fq.call(content,'i'),content.firstChild);assert.sameValue(fa.call(content,'i')[0],content.firstChild);assert.notSameValue(fq,eq);assert.notSameValue(fp,ep);

// saved-aliases-survive-tree-movement [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
document.body.removeChild(e);g.appendChild(e);assert.sameValue(eq,e.querySelector);assert.sameValue(ea,e.querySelectorAll);assert.sameValue(ep,e.append);
assert.sameValue(eq.call(e,'i'),a);assert.sameValue(fq,g.querySelector);assert.sameValue(fq.call(g,'i'),d);
var copy=e.cloneNode(true);assert.sameValue(eq,copy.querySelector);assert.notSameValue(eq.call(copy,'i'),a);

// eight-isolated-function-expando-bags [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var again=[document.querySelector,document.querySelectorAll,o.querySelector,o.querySelectorAll,g.querySelector,g.querySelectorAll,o.append,g.append];
for(var i=0;i<8;i++){var marker={index:i};methods[i].identityToken=marker;assert.sameValue(again[i].identityToken,marker);for(var j=0;j<8;j++)if(j!==i)assert.sameValue(methods[j].identityToken,undefined);assert.sameValue(delete methods[i].identityToken,true);assert.sameValue(Object.getOwnPropertyDescriptor(again[i],'identityToken'),undefined);}

// name-length-descriptors-and-isolation [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var names=['querySelector','querySelectorAll','querySelector','querySelectorAll','querySelector','querySelectorAll','append','append'];
var again=[document.querySelector,document.querySelectorAll,o.querySelector,o.querySelectorAll,g.querySelector,g.querySelectorAll,o.append,g.append];
for(var i=0;i<8;i++){var m=methods[i],name=Object.getOwnPropertyDescriptor(m,'name'),len=Object.getOwnPropertyDescriptor(m,'length');verifyProperty(m,'name',{value:names[i],writable:false,enumerable:false,configurable:true},{restore:true});verifyProperty(m,'length',{value:i<6?1:0,writable:false,enumerable:false,configurable:true},{restore:true});Object.defineProperty(m,'name',{value:'renamed'});Object.defineProperty(m,'length',{value:77});assert.sameValue(again[i].name,'renamed');assert.sameValue(again[i].length,77);for(var j=0;j<8;j++)if(j!==i){assert.sameValue(methods[j].name,names[j]);assert.sameValue(methods[j].length,j<6?1:0);}Object.defineProperty(m,'name',name);Object.defineProperty(m,'length',len);}

// deleted-own-metadata-stays-deleted [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var keys=['name','length'];
for(var i=0;i<8;i++)for(var j=0;j<2;j++){var key=keys[j],saved=Object.getOwnPropertyDescriptor(methods[i],key);assert.sameValue(delete methods[i][key],true);var again=[document.querySelector,document.querySelectorAll,e.querySelector,e.querySelectorAll,f.querySelector,f.querySelectorAll,e.append,f.append];assert.sameValue(Object.getOwnPropertyDescriptor(again[i],key),undefined);Object.defineProperty(methods[i],key,saved);assert.sameValue(Object.getOwnPropertyDescriptor(again[i],key).value,saved.value);}

// dispatch-ignores-public-name-and-length [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var qn=Object.getOwnPropertyDescriptor(eq,'name'),an=Object.getOwnPropertyDescriptor(ea,'name'),ql=Object.getOwnPropertyDescriptor(eq,'length');
Object.defineProperty(eq,'name',{value:'querySelectorAll'});Object.defineProperty(ea,'name',{value:'querySelector'});Object.defineProperty(eq,'length',{value:99});
assert.sameValue(eq,e.querySelector);assert.sameValue(eq.call(o,'i'),c);var found=ea.call(o,'i');assert.sameValue(found.length,1);assert.sameValue(found[0],c);assert.sameValue(dq.name,'querySelector');assert.sameValue(fq.name,'querySelector');
Object.defineProperty(eq,'name',qn);Object.defineProperty(ea,'name',an);Object.defineProperty(eq,'length',ql);

// selectors-call-distinct-same-interface-receiver [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(dq.call(document,'#domIdentityProbe'),a);assert.sameValue(da.call(document,'#domIdentityProbe')[0],a);
assert.sameValue(eq.call(o,'i'),c);assert.sameValue(ea.call(o,'i')[0],c);assert.sameValue(fq.call(g,'i'),d);assert.sameValue(fa.call(g,'i')[0],d);

// selectors-apply-distinct-same-interface-receiver [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(dq.apply(document,['#domIdentityProbe']),a);assert.sameValue(da.apply(document,['#domIdentityProbe'])[0],a);
assert.sameValue(eq.apply(o,['i']),c);assert.sameValue(ea.apply(o,['i'])[0],c);assert.sameValue(fq.apply(g,['i']),d);assert.sameValue(fa.apply(g,['i'])[0],d);

// selectors-bind-preserves-target-and-fresh-function-identity [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var target=[document,document,o,o,g,g],want=[a,a,c,c,d,d];
for(var i=0;i<6;i++){var bound=methods[i].bind(target[i]),again=methods[i].bind(target[i]);assert.sameValue(bound,bound);assert.notSameValue(bound,again);assert.notSameValue(bound,methods[i]);var r=bound.call({},'i');assert.sameValue(i%2?r[0]:r,want[i]);}

// all-selector-cross-interface-brands-before-coercion [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var kinds=[document,e,f];
for(var i=0;i<6;i++){var touches=0,arg={};Object.defineProperty(arg,'toString',{get:function(){touches++;return function(){touches++;return 'i';};}});for(var j=0;j<3;j++)if(j!==Math.floor(i/2))assert.throws(TypeError,function(){methods[i].call(kinds[j],arg);});assert.sameValue(touches,0);var r=methods[i].call(receivers[i],arg);assert.sameValue(touches,2);assert.sameValue(i%2?r[0]:r,i<4?a:b);}

// selector-noninterface-and-forged-receivers [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var fake={nodeType:1,tagName:'SECTION'},inherited=Object.create(Object.getPrototypeOf(e)),text=document.createTextNode('x');
var bad=[null,undefined,window,{},text,fake,inherited];
for(var i=0;i<6;i++){var touches=0,arg={toString:function(){touches++;return 'i';}};for(var j=0;j<bad.length;j++)assert.throws(TypeError,function(){methods[i].call(bad[j],arg);});assert.sameValue(touches,0);}

// selector-required-argument-versus-explicit-undefined [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
for(var i=0;i<6;i++){assert.throws(TypeError,function(){methods[i].call(receivers[i]);});var r=methods[i].call(receivers[i],undefined);if(i%2)assert.sameValue(r.length,0);else assert.sameValue(r,null);}

// selector-primitive-getter-hint-and-ignored-extra [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
for(var i=0;i<6;i++){var count=0,arg={},extra={toString:function(){throw new Test262Error('ignored conversion');}};Object.defineProperty(arg,Symbol.toPrimitive,{get:function(){count++;return function(hint){assert.sameValue(this,arg);assert.sameValue(hint,'string');count++;return 'i';};}});var r=methods[i].call(receivers[i],arg,extra);assert.sameValue(count,2);assert.sameValue(i%2?r[0]:r,i<4?a:b);}

// selector-conversion-throws-exact-sentinel [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
for(var i=0;i<6;i++){var marker={},count=0,arg={};Object.defineProperty(arg,'toString',{get:function(){count++;throw marker;}});try{methods[i].call(receivers[i],arg);throw new Test262Error('no conversion throw');}catch(error){assert.sameValue(error,marker);}assert.sameValue(count,1);}

// selectors-observe-live-tree-after-conversion [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var targets=[e,f],qs=[eq,fq],qas=[ea,fa];
for(var i=0;i<2;i++){var count=0,added=document.createElement('strong'),arg={toString:function(){count++;targets[i].appendChild(added);return 'strong';}};assert.sameValue(qs[i].call(targets[i],arg),added);assert.sameValue(count,1);var next=document.createElement('em'),allArg={toString:function(){count++;targets[i].appendChild(next);return 'em';}};var r=qas[i].call(targets[i],allArg);assert.sameValue(r.length,1);assert.sameValue(r[0],next);assert.sameValue(count,2);}

// append-call-apply-bind-distinct-receivers [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(ep,o.append);assert.sameValue(fp,g.append);assert.notSameValue(ep,fp);
assert.sameValue(ep.call(o,'A'),undefined);assert.sameValue(fp.apply(g,['B']),undefined);assert.sameValue(ep.bind(o)('C'),undefined);assert.sameValue(fp.bind(g)('D'),undefined);
assert.sameValue(o.textContent,'AC');assert.sameValue(g.textContent,'BD');assert.sameValue(e.textContent,'');assert.sameValue(f.textContent,'');assert.sameValue(o.firstChild,c);assert.sameValue(g.firstChild,d);

// append-brand-before-any-argument-conversion [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var ms=[ep,fp],rs=[e,f],invalid=[[document,f,{},document.createTextNode('x'),null,undefined],[document,e,{},document.createTextNode('x'),null,undefined]];
for(var i=0;i<2;i++){var hits=0,arg={toString:function(){hits++;return 'X';}};for(var j=0;j<invalid[i].length;j++)assert.throws(TypeError,function(){ms[i].call(invalid[i][j],arg);});assert.sameValue(hits,0);assert.sameValue(ms[i].call(rs[i],arg),undefined);assert.sameValue(hits,1);assert.sameValue(rs[i].textContent,'X');}

// append-all-conversions-before-operation-mutation [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var ms=[ep,fp];
for(var i=0;i<2;i++){var destination=i?document.createDocumentFragment():document.createElement('div'),holder=document.createElement('aside'),child=document.createElement('b'),marker={},trace='';holder.appendChild(child);var first={toString:function(){trace+='a';destination.textContent='author';return 'prepared';}},second={toString:function(){trace+='b';throw marker;}};try{ms[i].call(destination,child,first,second);throw new Test262Error('no throw');}catch(error){assert.sameValue(error,marker);}assert.sameValue(trace,'ab');assert.sameValue(destination.textContent,'author');assert.sameValue(child.parentNode,holder);}

// append-reentry-and-fragment-move [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var ms=[ep,fp];
for(var i=0;i<2;i++){var destination=i?document.createDocumentFragment():document.createElement('div'),part=document.createDocumentFragment(),child=document.createElement('b');part.appendChild(child);var hits=0,arg={toString:function(){hits++;destination.textContent='during';return 'after';}};assert.sameValue(ms[i].call(destination,part,arg),undefined);assert.sameValue(hits,1);assert.sameValue(destination.textContent,'duringafter');assert.sameValue(child.parentNode,destination);assert.sameValue(part.firstChild,null);}

// operations-nonconstructable-and-no-own-prototype [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(typeof Reflect.construct,'function');function C(){}var ordinary=Reflect.construct(C,[]);assert.sameValue(Object.getPrototypeOf(ordinary),C.prototype);
for(var i=0;i<8;i++){var m=methods[i];assert.sameValue(Object.getOwnPropertyDescriptor(m,'prototype'),undefined);assert.throws(TypeError,function(){new m();});assert.throws(TypeError,function(){Reflect.construct(m,[]);});assert.throws(TypeError,function(){Reflect.construct(C,[],m);});var bound=m.bind(receivers[i]);assert.throws(TypeError,function(){new bound();});}

// text-and-comment-have-no-parentnode-methods [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var text=document.createTextNode('x'),box=document.createElement('div');box.innerHTML='<!--x-->';var comment=box.firstChild;assert.sameValue(text.nodeType,3);assert.sameValue(comment.nodeType,8);
var wrong=[text,comment];for(var i=0;i<2;i++){assert.sameValue(wrong[i].querySelector,undefined);assert.sameValue(wrong[i].querySelectorAll,undefined);assert.sameValue(wrong[i].append,undefined);}

// node-inherited-methods-are-not-split [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var text=document.createTextNode('x');var names=['appendChild','removeChild','cloneNode'];for(var i=0;i<3;i++){assert.sameValue(typeof e[names[i]],'function');assert.sameValue(e[names[i]],f[names[i]]);assert.sameValue(e[names[i]],text[names[i]]);}
var node=document.createElement('b');assert.sameValue(e.appendChild.call(g,node),node);assert.sameValue(node.parentNode,g);assert.sameValue(f.removeChild.call(g,node),node);assert.sameValue(node.parentNode,null);assert.notSameValue(text.cloneNode.call(a,true),a);

// document-append-standard-prerequisite [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(typeof document.append,'function');assert.notSameValue(document.append,ep);assert.notSameValue(document.append,fp);assert.sameValue(document.append(),undefined);

// real-interface-prototypes-standard-prerequisite [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(typeof Document,'function');assert.sameValue(typeof Element,'function');assert.sameValue(typeof DocumentFragment,'function');
var protos=[Document.prototype,Element.prototype,DocumentFragment.prototype],objs=[document,e,f],qs=[dq,eq,fq],qas=[da,ea,fa];
for(var i=0;i<3;i++){assert.sameValue(protos[i].isPrototypeOf(objs[i]),true);verifyProperty(protos[i],'querySelector',{value:qs[i],writable:true,enumerable:true,configurable:true},{restore:true});verifyProperty(protos[i],'querySelectorAll',{value:qas[i],writable:true,enumerable:true,configurable:true},{restore:true});assert.sameValue(Object.getOwnPropertyDescriptor(objs[i],'querySelector'),undefined);}

// host-method-replacement-standard-prerequisite [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
var targets=[document,e,f],saved=[dq,eq,fq];
for(var i=0;i<3;i++){var replacement=function(){return 73;};targets[i].querySelector=replacement;assert.sameValue(targets[i].querySelector,replacement);assert.sameValue(targets[i].querySelector(),73);verifyProperty(targets[i],'querySelector',{value:replacement,writable:true,enumerable:true,configurable:true},{restore:true});assert.sameValue(delete targets[i].querySelector,true);assert.sameValue(targets[i].querySelector,saved[i]);Object.defineProperty(targets[i],'querySelector',{value:replacement,writable:true,enumerable:true,configurable:true});assert.sameValue(targets[i].querySelector,replacement);assert.sameValue(delete targets[i].querySelector,true);assert.sameValue(targets[i].querySelector,saved[i]);}

// repeated-method-acquisition-terminal-policy [resource]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(e.querySelector.call(e,'i'),a);for(;;){var next=e.querySelector;}

// cumulative-function-property-growth-terminal-policy [resource]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
eq.policyProbe=1;assert.sameValue(eq.policyProbe,1);for(var n=0;;n++){eq['owned'+n]=n;}

// dom-method-identity-document-query-selector-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(dq.call(document,'#domIdentityProbe'),a);assert.sameValue(dq.name,'querySelector');

// dom-method-identity-document-query-selector-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(dq.call(document,'#domIdentityProbe'),a);assert.sameValue(dq.name,'deliberately-wrong-name');

// dom-method-identity-document-query-selector-all-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(da.call(document,'#domIdentityProbe')[0],a);assert.sameValue(da.name,'querySelectorAll');

// dom-method-identity-document-query-selector-all-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(da.call(document,'#domIdentityProbe')[0],a);assert.sameValue(da.name,'deliberately-wrong-name');

// dom-method-identity-element-query-selector-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(eq.call(o,'i'),c);assert.sameValue(eq.name,'querySelector');

// dom-method-identity-element-query-selector-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(eq.call(o,'i'),c);assert.sameValue(eq.name,'deliberately-wrong-name');

// dom-method-identity-element-query-selector-all-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(ea.call(o,'i')[0],c);assert.sameValue(ea.name,'querySelectorAll');

// dom-method-identity-element-query-selector-all-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(ea.call(o,'i')[0],c);assert.sameValue(ea.name,'deliberately-wrong-name');

// dom-method-identity-fragment-query-selector-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(fq.call(g,'i'),d);assert.sameValue(fq.name,'querySelector');

// dom-method-identity-fragment-query-selector-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(fq.call(g,'i'),d);assert.sameValue(fq.name,'deliberately-wrong-name');

// dom-method-identity-fragment-query-selector-all-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(fa.call(g,'i')[0],d);assert.sameValue(fa.name,'querySelectorAll');

// dom-method-identity-fragment-query-selector-all-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
assert.sameValue(fa.call(g,'i')[0],d);assert.sameValue(fa.name,'deliberately-wrong-name');

// dom-method-identity-element-append-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
ep.call(o,'control');assert.sameValue(o.textContent,'control');assert.sameValue(ep.name,'append');

// dom-method-identity-element-append-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
ep.call(o,'control');assert.sameValue(o.textContent,'control');assert.sameValue(ep.name,'deliberately-wrong-name');

// dom-method-identity-fragment-append-positive [pass]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
fp.call(g,'control');assert.sameValue(g.textContent,'control');assert.sameValue(fp.name,'append');

// dom-method-identity-fragment-append-wrong [Test262Error]
assert.sameValue(typeof Object.is,'function');assert.sameValue(Object.is(NaN,NaN),true);assert.sameValue(Object.is(0,-0),false);
var e=document.createElement('section'),o=document.createElement('article');
var f=document.createDocumentFragment(),g=document.createDocumentFragment();
var a=document.createElement('i'),b=document.createElement('i'),c=document.createElement('i'),d=document.createElement('i');
a.setAttribute('id','domIdentityProbe');e.appendChild(a);f.appendChild(b);o.appendChild(c);g.appendChild(d);document.body.appendChild(e);
var dq=document.querySelector,da=document.querySelectorAll,eq=e.querySelector,ea=e.querySelectorAll,fq=f.querySelector,fa=f.querySelectorAll,ep=e.append,fp=f.append;
var methods=[dq,da,eq,ea,fq,fa,ep,fp],receivers=[document,document,e,e,f,f,e,f];
for(var i=0;i<8;i++)assert.sameValue(typeof methods[i],'function');
for(var i=0;i<6;i++){var r=methods[i].call(receivers[i],'i'),want=i<4?a:b;if(i%2){assert.sameValue(r.length,1);assert.sameValue(r[0],want);}else assert.sameValue(r,want);}
var epGuard=document.createElement('p'),fpGuard=document.createDocumentFragment();
assert.sameValue(ep.call(epGuard,'E'),undefined);assert.sameValue(epGuard.textContent,'E');
assert.sameValue(fp.call(fpGuard,'F'),undefined);assert.sameValue(fpGuard.textContent,'F');
fp.call(g,'control');assert.sameValue(g.textContent,'control');assert.sameValue(fp.name,'deliberately-wrong-name');

