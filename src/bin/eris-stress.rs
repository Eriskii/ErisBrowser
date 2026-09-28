//! Deterministic mutation smoke tests, not coverage-guided fuzzing.
//!
//! Run under an operating-system timeout when testing untrusted changes:
//! `timeout 180s cargo run --release --bin eris-stress -- 5000 0xe2152026`.
//! A caught panic or invariant failure writes its input and reproduction metadata
//! to `artifacts/stress-failure/` and exits unsuccessfully. Resource-limit errors
//! from the intentionally bounded script/SVG implementations are expected.

use eris::{
    css::{self, ComputedStyle, GridBreadth, Length},
    dom::{AttributeNamespace, Document, NodeKind},
    graphics::{Canvas, Color, DrawCommand, Fonts, ImageStore, Rect},
    layout,
    script::Runtime,
    svg,
};
use std::{
    any::Any,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    time::Instant,
};

const MAX_SAMPLE: usize = 8192;
const DEFAULT_SEED: u64 = 0xe215_2026;
const SCRIPT_DOCUMENT: &str = "<!doctype html><body><button id='go'>Go</button><div id='out'>Initial</div><input id='field' value='test'></body>";

const HTML_SEEDS: &[&str] = &[
    "<style>main{width:calc(100% - 20px);display:grid;grid-template-columns:calc(50% - 5px) 1fr;gap:calc(10% + 2px)}i{width:calc((25% + 10px) + calc(25% - 20px));padding:calc(5% - 10px);margin-left:calc(10% + 5px)}b{width:40px;width:calc(1px +);width:calc(0 + 1px);height:calc(30px + 0%);background:green}</style><main><i>Mixed lengths</i><b>Token boundaries</b></main>",
    r#"<style>main{--Tone:red;--alias:var(--Tone);--empty:;--loop:var(--missing,var(--loop));--safe:var(--Tone,var(--safe))}i{--Tone:blue;--number:20;color:VAR(--alias);width:var(--number)px;background:var(--loop,rgb(0,128,0));font-family:"var(--Tone)",serif;margin:var(/**/--empty/**/,3px) 2px}b{color:var(--safe);padding:var(--missing,var(--other,1px 2px))}</style><main><i>Computed aliases</i><b>Fallbacks and cycles</b></main>"#,
    "<style>i/**/.tile{background:green}div /**/i{color:blue}div/**/i{color:red}./**/tile{width:30px}[data-x~/**/=a]{height:40px}:/**/is(.tile){display:block}@supports selector(i/**/.tile) and/**/(display:block){i{border:1px solid green}}@supports not selector(ns/**/|i){i{background:red}}</style><div><i class=tile data-x='a b'></i></div>",
    "<style>details{border:1px solid green}summary{display:list-item;list-style-position:inside}@supports (display:grid) and (width:20px){section{display:grid;grid-template-columns:1fr 1fr}}@supports not (padding:auto){i{color:blue}}</style><section><details name=notes open><p>Before summary</p><summary>First</summary><i style='position:fixed;top:0'>Hidden fixed</i><details><summary>Nested</summary>Nested body</details></details><details name=notes open><summary>Second</summary>Content</details><details>No summary</details></section>",
    r#"<style>i{display:block;width:30px;height:20px;background:red}@media unknown(url(foo bar)) or (color){i{background:blue}}@media unknown(url(foo[bar)) or (width >= 100px){i{background:green}}</style><i>URL tokens</i>"#,
    "<style>main{display:flex;flex-flow:column-reverse wrap-reverse;height:90px;gap:4px 6px;align-content:space-around}i{height:35px;width:40px;background:coral}@media ((100px < width <= 400px) or (height < 90px)) and (orientation:landscape){i{background:blue}}</style><main><i>one</i><i>two</i><i>three</i><i>four</i></main>",
    "<style>@layer reset,theme;@layer theme{main{opacity:.5;background:blue}i{position:absolute;left:20px;top:10px;opacity:.7;background:red}}@layer reset{main{position:relative;width:120px;height:80px;background:green}i{width:70px;height:50px}}@layer theme{b{position:fixed;left:0;top:0;background:orange}}</style><main>group<i>nested<b>fixed</b></i></main>",
    "<style>main{position:relative;width:120px;padding:10px;border:2px solid;z-index:0}section{overflow:hidden;width:40px}i{position:absolute;left:10%;right:5px;top:2px;height:20px;z-index:-1;background:red}b{position:fixed;bottom:3px;right:4px;width:20px;height:15px;z-index:2147483647;background:blue}</style><main><section><i><b></b></i></section><p>stacked text</p></main>",
    "<style>main{display:grid;grid-template-columns:40px 40px;position:relative}div{grid-area:1/1;z-index:2;background:coral}span{position:relative;left:-3px;top:2px}i{position:absolute;left:0;right:0;max-width:20px;margin:auto;height:8px}</style><main><div><span>word<i></i></span></div><div style='z-index:1'>lower</div></main>",
    "<style>main{display:grid;grid-template-columns:minmax(20px,1fr) 2fr;grid-template-rows:30px auto;grid-auto-rows:20px 40px;grid-auto-columns:min-content 50px;grid-auto-flow:row dense;gap:3px 7px}i{padding:2px}#wide{grid-column:span 2}#past{grid-column:-5;grid-row:4 / span 2}</style><main><i id=wide>Spanning text</i><i>Auto</i><i id=past>Implicit</i><i style='order:-1;align-self:end'>Ordered</i></main>",
    "<!doctype html><template id=outer><table><tr><td>Hosted cell</td></tr></table><template id=nested><svg><foreignObject><p>Nested content</p></foreignObject></svg></template></template><main><p>Active sibling</p><template><select><option>Inert choice</option></select></template></main><script>const t=document.createElement('template');t.innerHTML='<section><template><b>nested</b></template><i>fragment</i></section>';document.body.appendChild(t.content.cloneNode(true));</script>",
    "<style>body{margin:0}.left{float:left;width:32%;margin:2px;padding:3px;background:coral}.right{float:right;width:40px;height:50px}.clear{clear:both}.isolate{display:flow-root;overflow:hidden}</style><div class=left>Floating <b>text</b></div><div class=right>Right</div><p>Lines around two floats with different heights and margins.</p><section class=isolate><span class=left>Nested float</span>Separate context</section><p class=clear>After floats</p>",
    "<style>.clip{overflow:clip;width:60px;height:30px}a{float:left;min-width:20px;max-width:110px;padding:10%;margin-right:3px;background:#135;color:white}p{clear:left}</style><div class=clip><a href='#done'>Oversize float with wrapped words</a></div><p id=done>Clearance</p>",
    "<svg xmlns='http://www.w3.org/2000/svg' viewbox='0 0 120 80' preserveaspectratio='xMidYMid meet'><g xml:lang='en'><foreignobject x=4 y=4 width=100 height=60><p>HTML <b>integration</b><svg><title>Nested SVG</title><circle r=5 /></svg></p></foreignobject></g></svg><p>After SVG</p>",
    "<math><mi>x<b>HTML text integration</b><mglyph src='none' /></mi><annotation-xml encoding='TEXT/HTML'><p>Annotation <svg><desc><em>HTML inside SVG description</em></desc></svg></p></annotation-xml><mtext><span>text</span></mtext></math>",
    "<svg><g><![CDATA[<& raw text \0 and 🦀]]><title>title &amp; text</title></g><p>HTML breakout</p></svg><math><mrow><![CDATA[<math & data>]]></mrow><font color=red>Breakout</font></math>",
    "<svg xmlns:xlink='http://www.w3.org/1999/xlink'><defs><lineargradient id=paint gradientunits=userSpaceOnUse gradienttransform='rotate(5)' xlink:href='#base'><stop offset='0'/></lineargradient></defs><text textlength=30 lengthadjust=spacingAndGlyphs xml:space=preserve>A B</text></svg><math definitionurl='example' xml:lang=fr><mi>x</mi></math>",
    "<!DOCTYPE html PUBLIC 'example' 'system'><!--before--><?eris check?><table>fostered<tr><td>one<td>two</table><noscript><p>fallback</p></noscript><!--after-->",
    "<style>body{margin:0}.clip{overflow:hidden;width:100px;height:50px;padding:3px;background:#eee}.wide{width:240px;height:90px;background:#d93}.inner{overflow:clip;width:40px;height:20px}</style><div class=clip><div class=wide><div class=inner><a href='/next'>Clipped text content</a></div></div></div>",
    "<!doctype html><style>body{margin:8px;background:#eef}h1{font-size:24px}p{color:#135;line-height:1.4}</style><h1>Render 🦀</h1><p>Hello <strong>bold</strong> café &amp; 日本語</p>",
    "<style>.row{display:flex;gap:4px;flex-wrap:wrap}.row div{padding:6px;border:1px solid #963;width:44px}</style><main class=row><div>first</div><div>second</div><div>third</div></main>",
    "<style>.grid{display:grid;grid-template-columns:1fr 2fr;gap:5px}.grid p{margin:0;padding:4px;background:rgb(30 80 120 / .5)}</style><div class=grid><p>one</p><p>two</p><p>three</p><p>four</p></div>",
    "<table style='border:2px solid blue'><caption>Data</caption><thead><tr><th>Name<th>Value<tbody><tr><td>A<td>10<tr><td>B<td>20</table><ul><li>first<li>second</ul>",
    "<form><label>Name <input id=field name=q placeholder=Search></label><input type=checkbox checked><button>Submit</button><textarea>Multiline\nvalue</textarea><select><option>Alpha<option selected>Beta</select></form>",
    "<style>:root{--ink:#246;--w:80%}#box{color:var(--ink);width:var(--w);padding:calc(2px + 1vw)}@media(max-width:300px){#box{background:lavender}}div:not(.hidden)>span:first-child{font-weight:bold}</style><div id=box><span>Variables</span> and selectors</div>",
    "<!doctype html><style>body{margin:0}div{border-radius:12px;opacity:.6;background:#fc7;padding:5px}pre{white-space:pre-wrap}</style><div><div><div>Nested</div></div></div><pre> x  y\n é Ω 🦀 &lt;&gt;</pre><hr><br>End",
    "<style>.a{position:relative;left:-2px;top:3px;width:90%;max-width:150px;min-height:20px}.b{font-size:125%;vertical-align:middle}a[href^='https']{color:rebeccapurple}</style><p class=a>Text <span class=b>large</span> <a href=https://example.com>link</a></p><img width=16 height=16 alt=missing>",
];
const SCRIPT_SEEDS: &[&str] = &[
    "var input={toString(){return 'a b🦀';}},encoded=encodeURIComponent(input);document.getElementById('out').textContent=decodeURIComponent(encoded);try{decodeURI('%ED%A0%80');}catch(e){if(e instanceof URIError)document.body.className='invalid';}document.getElementById('go').onclick=function(){document.getElementById('out').textContent=decodeURI('%2f%41')+encodeURI('/a b');};",
    "var log='',o={get x(){log+='G';return null;},set x(v){log+='S';}};o.x??=3;var a=0,b=1;a&&=missing;b||=missing;var callback=null;callback??=()=>3;document.getElementById('out').textContent=log+callback.name;document.getElementById('go').onclick=function(){a||=callback();b&&=a;document.getElementById('out').textContent=String(b);};",
    "let trace='';const left={valueOf(){trace+='L';return 'a';}},right={valueOf(){trace+='R';return 1;}};const object={get x(){return left;},set x(value){document.getElementById('field').value=value;}};document.getElementById('out').textContent=(object.x+=right)+':'+trace;document.getElementById('go').onclick=function(){right.valueOf=function(){return new String('b').valueOf();};document.getElementById('out').textContent=left+right;};",
    "let stored=9,trace='';const object={get x(){trace+='G';return stored;},set x(value){trace+='S';stored=value;}};function rhs(){trace+='R';return {valueOf(){trace+='V';return 1;}};}object.x>>>=rhs();stored<<=2;stored>>=1;stored&=15;stored^=3;stored|=1;document.getElementById('out').textContent=stored+':'+trace;document.getElementById('go').onclick=function(){object.x^=rhs();document.getElementById('field').value=String(stored);};",
    "const I=Number.parseInt,F=Number.parseFloat;let trace='';const input={toString(){trace+='s';return '12tail';}},radix={valueOf(){trace+='r';return 10;}};document.getElementById('out').textContent=I(input,radix)+':'+F(input)+':'+trace;document.getElementById('go').onclick=function(){Number.parseInt=function(){throw 'replacement';};document.getElementById('field').value=String(I({toString(){return '0x10';}}));};",
    "const N=Number,F=isFinite,I=isNaN;let trace='';const object={valueOf(){trace+='v';return {};},toString(){trace+='t';return '7';}};const Bound=N.bind(null,object);const box=new Bound();const good=F(box)&&!I(box)&&N(box)===7;document.getElementById('out').textContent=trace+':'+good;document.getElementById('go').onclick=function(){const value={get valueOf(){document.getElementById('field').value='converted';return function(){return '2';};}};document.getElementById('out').textContent=String(N(value));};",
    "const safe=Number.isSafeInteger;const values=[Number.MIN_VALUE,Number.MAX_VALUE,Number.MAX_SAFE_INTEGER,NaN,'2'];const count=values.reduce((total,value)=>total+(safe(value)?1:0),0);document.getElementById('out').textContent=count+':'+Number.isFinite(Number.EPSILON)+':'+Number.isNaN(NaN);document.getElementById('go').onclick=function(){document.getElementById('field').value=String(safe(Number.MIN_SAFE_INTEGER)&&!Number.isInteger(Number.MIN_VALUE));};",
    "const values=[1,,3];const state=values.reduce(function(acc,value,index){if(index===0)values[1]=2;acc.sum+=value;return acc;},{sum:0});const proto={1:4},like=Object.create(proto);like.length=3;like[2]=5;const text=Array.prototype.reduceRight.call(like,(acc,value)=>acc+String(value),'');document.getElementById('out').textContent=state.sum+':'+text;document.getElementById('go').onclick=function(){document.getElementById('field').value=values.reduceRight((acc,value)=>acc+String(value),'');};",
    r#"let π=2;\u03C0++;const ℘=4;const a\u0301=5;let 𐐀=6;const object={\u0069f:7,\u0067et(){return this.if;}};const tail=`${/}/.test('}')} ${\u03C0}`;document.getElementById('out').textContent=tail+':'+\u2118+':'+á+':'+object.get();document.getElementById('go').onclick=function(\u0065vent){\u{10400}++;document.getElementById('field').value=event.type+':'+𐐀;};"#,
    "const sort=Array.prototype.sort;const values=[3,,undefined,1,2];values.sort((a,b)=>a-b);const proto={1:4};const like=Object.create(proto);like.length=3;like[0]=5;like[2]=2;sort.call(like,(a,b)=>a-b);document.getElementById('out').textContent=values.join(':')+':'+like[0];document.getElementById('go').onclick=function(){values.sort((a,b)=>b-a);document.getElementById('field').value=values.join('/');};",
    "const owner=window,saved=Object.getOwnPropertyDescriptor(owner,'globalThis');globalThis={phase:1};delete owner.globalThis;owner.globalThis={phase:2};Object.defineProperty(owner,'globalThis',saved);Object.defineProperty(owner,'NaN',{value:NaN});document.getElementById('out').textContent=(globalThis===owner)+':'+Object.getOwnPropertyDescriptor(owner,'Infinity').enumerable;document.getElementById('go').onclick=function(){owner.globalThis={phase:3};document.getElementById('field').value=String(this===document.getElementById('go'));};",
    "const win=window,saved=Object.getOwnPropertyDescriptor(win,'self');self={phase:1};delete win.self;Object.defineProperty(win,'self',saved);const value={};Object.defineProperty(win,'self',{get(){return value;},set(next){delete win.self;saved.set.call(win,next);},configurable:true});self={phase:2};document.getElementById('out').textContent=win.self.phase;document.getElementById('go').onclick=function(){Object.defineProperty(win,'self',saved);document.getElementById('field').value=String(self===win);};",
    "const base={},middle=Object.create(base),leaf=Object.create(middle);const member=Object.prototype.isPrototypeOf;document.getElementById('out').textContent=base.isPrototypeOf(leaf)+':'+member.call(null,3);document.getElementById('go').onclick=function(){Object.setPrototypeOf(middle,null);document.getElementById('field').value=String(base.isPrototypeOf(leaf));};",
    "function tail(first,...rest){arguments[0]=90;rest.push(first);return rest;}const a=tail(1,undefined,null,4);document.getElementById('out').textContent=a.join('/');document.getElementById('go').onclick=function(...events){document.getElementById('field').value=events[0].type;};",
    "function separated(read=()=>rest,...rest){var rest=[90];return [read(),rest];}const scopes=separated(undefined,7,8);const arrow=(pattern=/[(),]/,...rest)=>pattern.test(rest[0]);document.getElementById('out').textContent=scopes[0].join(',')+':'+arrow(undefined,',');",
    "function early(value=rest,...rest){return value;}let failed=false;try{early();}catch(error){failed=error instanceof ReferenceError;}function outer(n){return (...rest)=>this.flag+arguments[0]+rest[0];}const call=outer.call({flag:5},7);document.getElementById('out').textContent=failed+':'+call(3);",
    "let outer=4;function sample(a=outer,read=()=>a){var a=9;return [read(),a,arguments.length];}const object={method(x=3,y=x+1){return x+y;}};const arrow=(x=/[()]/,text=`matched:${x.test('(')}`)=>text;document.getElementById('out').textContent=sample().join(':')+':'+object.method()+':'+arrow();document.getElementById('out').style.width='calc(50% - 10px)';",
    r#"const s=document.getElementById('out').style;s.cssText='--Tone:green;--tone:red;--note:"a;b:c";background-color:var(--Tone);color:blue!important';s.setProperty('--tone','blue');s.setProperty('color','red','invalid');s.setProperty('background-color',{toString(){s.setProperty('--saved','"x;y:z"');return 'green';}},'IMPORTANT');const prior=s.removeProperty('COLOR');document.getElementById('field').value=s.getPropertyValue('--Tone')+':'+s.getPropertyValue('--note')+':'+s.getPropertyPriority('background-color')+':'+prior+':'+('backgroundColor' in s);s.backgroundColor=null;"#,
    "const key={toString(){return 'CSS';}};const descriptor=Object.getOwnPropertyDescriptor(window,key);const supported=descriptor.value.supports('display','grid');delete window.CSS;const missing=Object.getOwnPropertyDescriptor(window,'CSS');window.CSS=descriptor.value;document.getElementById('out').textContent=descriptor.enumerable+':'+supported+':'+missing+':'+Object.getOwnPropertyDescriptor(window,'CSS').enumerable;",
    "let n=0;const key={toString(){n++;return 'value';}};const object={__proto__:{inherited:4},[key]:3,get ['total'](){return this.value+this.inherited;},['advance'](){this.value++;return this.total;}};document.getElementById('out').textContent=object.advance()+':'+n+':'+object.advance.name;",
    "const a=document.createElement('details');const b=document.createElement('details');a.name='group';b.name='group';document.body.appendChild(a);document.body.appendChild(b);a.ontoggle=function(event){document.getElementById('out').textContent=event.oldState+':'+event.newState;};a.open=true;b.open=true;a.open=true;const event=new ToggleEvent('toggle',{oldState:'closed',newState:'open',source:a});b.dispatchEvent(event);",
    r#"let n=2;const value={toString:function(){n++;return 'v';}};document.getElementById('out').textContent=`head ${value} ${`nested ${n}`} ${/}/.test('}')} \uD800 tail`;"#,
    "const control=new AbortController();const target=new EventTarget();let count=0;target.addEventListener('ping',function(){count++;},{signal:control.signal});target.dispatchEvent(new Event('ping'));control.abort('finished');target.dispatchEvent(new Event('ping'));document.getElementById('out').textContent=count+':'+control.signal.reason;",
    "const target=document.getElementById('go');let trace=[];document.addEventListener('signal',function(e){trace.push(e.eventPhase);},{capture:true,once:true});target.addEventListener('signal',function(e){e.preventDefault();trace.push(e.detail);});const event=new CustomEvent('signal',{bubbles:true,cancelable:true,detail:4});const accepted=target.dispatchEvent(event);document.getElementById('out').textContent=trace.join('/')+accepted;",
    "const target=new EventTarget();let count=0;const callback={handleEvent:function(e){count++;target.removeEventListener('x',callback);}};target.addEventListener('x',callback);target.dispatchEvent(new Event('x'));target.dispatchEvent(new Event('x'));document.getElementById('out').textContent=String(count);",
    r#"const pattern=/(?<name>[A-Z]+)-(\d+)/g; const text='AX-12 BY-34'; const match=pattern.exec(text); document.getElementById('out').textContent=match.groups.name+':'+text.replace(pattern,'$2/$<name>');"#,
    r#"const pattern=/((a|b)+)\1/d; const match=pattern.exec('abbaabba'); document.getElementById('out').textContent=match[0]+':'+match.indices[1].join(',')+':'+('one  two').split(/\s+/).join('/');"#,
    "x=0;function remove(){delete globalThis.x;return 1;}x=remove();document.getElementById('out').textContent=String(x);",
    "'use strict';function inspect(value){arguments[0]=9;return value+':'+arguments[0];}let result='';try{result+=typeof later;let later=2;}catch(error){result+=error.name;}const t=document.createElement('template');t.innerHTML='<section><b>Cloned</b><template><i>Nested</i></template></section>';const clone=t.content.cloneNode(true);document.getElementById('out').appendChild(clone);document.getElementById('field').value=result+':'+inspect(3);",
    "let saved=3;const object={};Object.defineProperty(object,'value',{get:function(){return saved;},set:function(x){saved=x;},enumerable:true,configurable:true});object.value=7;const child=Object.create(object);child.own=2;let result='';for(let key in child){result+=key+':'+child[key]+';';}const descriptor=Object.getOwnPropertyDescriptor(object,'value');document.getElementById('out').textContent=result+descriptor.enumerable;delete object.value;",
    "function Item(x){this.value=x;}const Bound=Item.bind(null,4);const instance=new Bound();const object={};Object.defineProperty(object,'locked',{value:8});try{Object.defineProperty(object,'locked',{value:9});}catch(error){document.getElementById('out').textContent=error.name+':'+instance.value+':'+Object.keys(object).length;}const values=[1,,3];let keys='';for(let key in values){keys+=key;}document.getElementById('field').value=keys;",
    "const table=document.createElement('table');document.getElementById('out').appendChild(table);table.innerHTML='<tr><td>First<td>Second';const row=table.querySelector('tr');row.innerHTML='<td>Changed<td><b>Bold</b>';const select=document.createElement('select');select.innerHTML='<option>Alpha<option>Beta';document.getElementById('out').appendChild(select);",
    "function Palette(n){this.step=n;} Palette.prototype.advance=function(){this.step++;switch(this.step%3){case 0:return 'seafoam';case 1:return 'amber';default:return 'iris';}}; const palette=new Palette(0);document.getElementById('go').onclick=function(){document.getElementById('out').textContent=palette.advance()+' '+(palette instanceof Palette)+' '+(Object.getPrototypeOf(palette)===Palette.prototype);};",
    "let names=[];try{throw new TypeError('explicit');}catch(error){names.push(error.name);names.push(error instanceof Error);}try{null.property;}catch(error){names.push(error.constructor===TypeError);}try{const values=[];values.length=-1;}catch(error){names.push(error.name);}finally{document.getElementById('out').textContent=names.join('/');}",
    "document.getElementById('out').innerHTML='<svg viewBox=\"0 0 20 20\" xmlns:xlink=\"http://www.w3.org/1999/xlink\"><g xml:lang=\"en\"><circle id=\"dot\" r=\"8\" xlink:href=\"#self\" /></g></svg><math><mi>x</mi></math>';const dot=document.getElementById('dot');dot.setAttribute('fill','coral');dot.setAttribute('xlink:href','#changed');document.getElementById('field').value=dot.namespaceURI;",
    r#"const decoded = JSON.parse('{"message":"héllo","values":[1,2,null]}', function(key,value){if(key==='message'){return value.toUpperCase();}return value;}); document.getElementById('out').textContent=JSON.stringify(decoded,null,2);"#,
    "let value = ''; try { throw {name:'test', number:4}; } catch (error) { value = error.name; } finally { value += '-done'; } document.getElementById('out').textContent = value;",
    "let sum = 0; for (let i = 0; i < 6; i++) { sum += i; } document.getElementById('out').textContent = String(sum);",
    "let count = 0; document.getElementById('go').addEventListener('click', () => { count++; document.getElementById('out').textContent = count; });",
    "function twice(x) { return x * 2; } const values = [1,2,3].map(twice); document.querySelector('#out').textContent = values.join('-');",
    "const item = document.createElement('p'); item.textContent = 'café Ω 🦀'; item.classList.add('active'); item.style.backgroundColor = '#aef'; document.getElementById('out').appendChild(item);",
    "function counter() { let n = 0; return () => ++n; } const next = counter(); let result = next() + next(); console.log(result, Math.max(2, 5));",
    "document.addEventListener('DOMContentLoaded', event => { document.querySelector('#out').innerHTML = '<b>ready</b>'; }); document.getElementById('go').onclick = event => { event.preventDefault(); return false; };",
    "const data = { name: 'example', total: 3 }; data.total += 2; if (data.total > 3 && data.name.includes('amp')) { document.getElementById('out').textContent = data.name.toUpperCase(); }",
    "let n = 4; while (n > 0) { n--; } const text = '12.5e2 trailing'; document.querySelector('#out').textContent = parseFloat(text);",
];
const SVG_SEEDS: &[&str] = &[
    "<svg width='128' height='96' viewBox='0 0 20 20' preserveAspectRatio='xMaxYMin slice' xmlns:xlink='http://www.w3.org/1999/xlink'><desc><![CDATA[foreign < & text]]></desc><g xml:lang='en'><rect width='20' height='20' fill='#246'/><circle cx='10' cy='10' r='6' fill='gold' xlink:href='#unused'/></g></svg>",
    "<svg width='128' height='96' viewBox='0 0 40 10' preserveAspectRatio='none'><rect width='40' height='10' fill='#def'/><foreignObject width='20' height='10'><p>HTML integration</p></foreignObject><path d='M0 0 L40 10' stroke='#123' fill='none'/></svg>",
    "<svg width='128' height='96' viewBox='0 0 128 96'><rect x='4' y='4' width='120' height='88' rx='8' fill='#243'/><circle cx='64' cy='48' r='28' fill='orange'/></svg>",
    "<svg width='128' height='96'><g transform='translate(10 8) rotate(5)' fill='blue' opacity='.6'><rect width='50' height='40'/><ellipse cx='70' cy='55' rx='30' ry='20' fill='red'/></g></svg>",
    "<svg width='128' height='96'><path d='M8 48 C8 0 120 0 120 48 Q64 96 8 48Z' fill='coral' stroke='black' stroke-width='2'/></svg>",
    "<svg width='128' height='96'><path d='M20 40 A24 24 0 1 0 68 40 A24 24 0 1 0 20 40Z M32 40A12 12 0 1 1 56 40A12 12 0 1 1 32 40Z' fill-rule='evenodd' fill='#6af'/></svg>",
    "<svg width='128' height='96'><polyline points='5,70 30,10 70,80 120,20' fill='none' stroke='#b13' stroke-width='4'/><polygon points='20,20 70,20 45,60' fill='#7c9'/></svg>",
    "<svg width='128' height='96' viewBox='-10 -10 100 75'><rect x='-10' y='-10' width='100' height='75' fill='#eef'/><text x='40' y='35' text-anchor='middle' font-size='13' fill='#234'>Hello é</text></svg>",
    "<svg width='128' height='96'><path d='M4 40 q24 -40 48 0 t48 0 m-40 30 h40 v15 h-40z' fill='gold' stroke='#345'/><line x1='0' y1='90' x2='128' y2='90' stroke='blue'/></svg>",
];

#[derive(Clone, Copy)]
enum Kind {
    Html,
    Script,
    Svg,
}
impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Script => "script",
            Self::Svg => "svg",
        }
    }
    fn extension(self) -> &'static str {
        match self {
            Self::Html => "html",
            Self::Script => "js",
            Self::Svg => "svg",
        }
    }
    fn seeds(self) -> &'static [&'static str] {
        match self {
            Self::Html => HTML_SEEDS,
            Self::Script => SCRIPT_SEEDS,
            Self::Svg => SVG_SEEDS,
        }
    }
}

struct Random {
    state: u64,
}
impl Random {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { DEFAULT_SEED } else { seed },
        }
    }
    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }
    fn index(&mut self, len: usize) -> usize {
        if len == 0 {
            0
        } else {
            self.next() as usize % len
        }
    }
    fn boundary(&mut self, text: &str) -> usize {
        let mut index = self.index(text.len() + 1);
        while !text.is_char_boundary(index) {
            index -= 1;
        }
        index
    }
}

fn mutate(kind: Kind, random: &mut Random, iteration: usize) -> String {
    let seeds = kind.seeds();
    let mut text = seeds[random.index(seeds.len())].to_owned();
    let count = if iteration.is_multiple_of(10) {
        0
    } else {
        1 + random.index(8)
    };
    for _ in 0..count {
        let at = random.boundary(&text);
        match random.index(8) {
            0 => {
                let inserts = [
                    "<", ">", "\"", "'", "/>", "</div>", "=", "!", ";", "{", "}", "(", ")", "[",
                    "]", "/*", "*/", "\\", "\n", "&amp;", "🦀", "é", "\0", "<!--", "-->",
                ];
                text.insert_str(at, inserts[random.index(inserts.len())]);
            }
            1 => {
                let mut end = (at + 1 + random.index(40)).min(text.len());
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                text.replace_range(at..end, "");
            }
            2 => text.truncate(at),
            3 => {
                let mut end = (at + 1 + random.index(96)).min(text.len());
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                let piece = text[at..end].repeat(2 + random.index(8));
                text.insert_str(at, &piece);
            }
            4 => {
                let values = [
                    "0",
                    "-1",
                    "1e99",
                    "NaN",
                    "Infinity",
                    "999999",
                    "0.000001",
                    "100%",
                    "calc(1px + 5%)",
                    "var(--cycle)",
                    "1/0",
                ];
                let end = text[at..]
                    .chars()
                    .next()
                    .map_or(at, |ch| at + ch.len_utf8());
                text.replace_range(at..end, values[random.index(values.len())]);
            }
            5 => {
                let (open, close) = match kind {
                    Kind::Html => ("<div>", "</div>"),
                    Kind::Script => ("(", ")"),
                    Kind::Svg => ("<g>", "</g>"),
                };
                let depth = 1 + random.index(24);
                text = format!("{}{}{}", open.repeat(depth), text, close.repeat(depth));
            }
            6 => {
                let punctuation = b"<>!+-=/:;,(){}[]0123456789eE&";
                let end = text[at..]
                    .chars()
                    .next()
                    .map_or(at, |ch| at + ch.len_utf8());
                text.replace_range(
                    at..end,
                    &(punctuation[random.index(punctuation.len())] as char).to_string(),
                );
            }
            _ => {
                let other = seeds[random.index(seeds.len())];
                let start = random.boundary(other);
                let mut end = (start + 1 + random.index(128)).min(other.len());
                while !other.is_char_boundary(end) {
                    end -= 1;
                }
                text.insert_str(at, &other[start..end]);
            }
        }
        let mut end = text.len().min(MAX_SAMPLE);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
    text
}

fn finite(values: &[f32], name: &str) -> Result<(), String> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(format!("non-finite {name}: {values:?}"));
    }
    Ok(())
}
fn rectangle(rect: Rect) -> Result<(), String> {
    finite(&[rect.x, rect.y, rect.width, rect.height], "rectangle")?;
    if rect.width < 0.0 || rect.height < 0.0 {
        return Err(format!("negative rectangle dimensions: {rect:?}"));
    }
    Ok(())
}
fn style_invariants(style: &ComputedStyle) -> Result<(), String> {
    finite(
        &[
            style.font_size,
            style.line_height,
            style.gap,
            style.flex_grow,
            style.flex_shrink,
            style.border_radius,
            style.opacity,
            style.border_width.top,
            style.border_width.right,
            style.border_width.bottom,
            style.border_width.left,
        ],
        "computed style",
    )?;
    let lengths = [
        style.width,
        style.height,
        style.min_width,
        style.min_height,
        style.max_width,
        style.max_height,
        style.flex_basis,
        style.row_gap,
        style.column_gap,
        style.top,
        style.right,
        style.bottom,
        style.left,
        style.margin.top,
        style.margin.right,
        style.margin.bottom,
        style.margin.left,
        style.padding.top,
        style.padding.right,
        style.padding.bottom,
        style.padding.left,
    ];
    let track_lengths = [
        &style.grid_template_columns,
        &style.grid_template_rows,
        &style.grid_auto_columns,
        &style.grid_auto_rows,
    ]
    .into_iter()
    .flat_map(|tracks| tracks.iter())
    .flat_map(|track| [track.min, track.max])
    .filter_map(|breadth| match breadth {
        GridBreadth::Length(length) => Some(length),
        _ => None,
    });
    for length in lengths.into_iter().chain(track_lengths) {
        match length {
            Length::Auto => {}
            Length::Px(value) | Length::Percent(value) | Length::Fr(value) => {
                finite(&[value], "CSS length")?
            }
            Length::Calc { px, percent, .. } => finite(&[px, percent], "CSS calculation")?,
        }
    }
    Ok(())
}
fn dom_invariants(document: &Document) -> Result<(), String> {
    if document.nodes.len() > 100_000 || document.retained_bytes() > 32 * 1024 * 1024 {
        return Err("DOM resource limit exceeded".into());
    }
    if document.root >= document.nodes.len()
        || document.nodes[document.root].parent.is_some()
        || !matches!(document.nodes[document.root].kind, NodeKind::Document)
    {
        return Err("invalid DOM root".into());
    }
    let mut incoming = vec![0usize; document.nodes.len()];
    let mut owners = vec![None; document.nodes.len()];
    for (id, node) in document.nodes.iter().enumerate() {
        match &node.kind {
            NodeKind::Document if id != document.root => {
                return Err("duplicate DOM document".into());
            }
            NodeKind::DocumentFragment { host } => {
                if node.parent.is_some() {
                    return Err("DOM fragment has an ordinary parent".into());
                }
                if let Some(host) = host {
                    let Some(NodeKind::Element(element)) =
                        document.nodes.get(*host).map(|node| &node.kind)
                    else {
                        return Err("invalid DOM fragment host".into());
                    };
                    if element.namespace != eris::dom::Namespace::Html
                        || element.tag != "template"
                        || element.template_contents != Some(id)
                    {
                        return Err("nonreciprocal DOM fragment host".into());
                    }
                }
            }
            NodeKind::Element(element) => {
                let template =
                    element.namespace == eris::dom::Namespace::Html && element.tag == "template";
                if template != element.template_contents.is_some() {
                    return Err("invalid DOM template ownership".into());
                }
                if let Some(contents) = element.template_contents
                    && (!matches!(document.nodes.get(contents).map(|node|&node.kind),Some(NodeKind::DocumentFragment{host:Some(host)}) if *host==id)
                        || owners[contents].replace(id).is_some())
                {
                    return Err("nonreciprocal or duplicate DOM template ownership".into());
                }
            }
            _ => {}
        }
        if !node.children.is_empty()
            && !matches!(
                node.kind,
                NodeKind::Document | NodeKind::DocumentFragment { .. } | NodeKind::Element(_)
            )
        {
            return Err("DOM leaf has children".into());
        }
        if let NodeKind::Element(element) = &node.kind {
            for (name, namespace) in &element.attr_namespaces {
                if !element.attrs.contains_key(name)
                    || AttributeNamespace::from_qualified_name(name) != Some(*namespace)
                {
                    return Err(format!("inconsistent attribute namespace on DOM node {id}"));
                }
            }
        }
        for child in &node.children {
            if *child >= document.nodes.len()
                || *child == id
                || matches!(
                    document.nodes[*child].kind,
                    NodeKind::Document | NodeKind::DocumentFragment { .. }
                )
                || document.nodes[*child].parent != Some(id)
            {
                return Err("inconsistent DOM parent/child relationship".into());
            }
            incoming[*child] += 1;
            if incoming[*child] > 1 {
                return Err("duplicate child in DOM".into());
            }
        }
        if node
            .parent
            .is_some_and(|parent| parent >= document.nodes.len())
        {
            return Err("DOM parent index exceeds arena length".into());
        }
    }
    let mut roots = Vec::new();
    for (id, node) in document.nodes.iter().enumerate() {
        if incoming[id] != usize::from(node.parent.is_some()) {
            return Err("DOM parent does not contain child".into());
        }
        if node.parent.is_none() && owners[id].is_none() {
            roots.push((id, 0usize));
        }
    }
    let mut seen = vec![false; document.nodes.len()];
    while let Some((id, depth)) = roots.pop() {
        if depth > eris::dom::MAX_DEPTH {
            return Err("host-inclusive DOM depth exceeded".into());
        }
        if seen[id] {
            return Err("cycle in DOM".into());
        }
        seen[id] = true;
        roots.extend(
            document.nodes[id]
                .children
                .iter()
                .map(|child| (*child, depth + 1)),
        );
        if let NodeKind::Element(element) = &document.nodes[id].kind
            && let Some(contents) = element.template_contents
        {
            roots.push((contents, depth + 1));
        }
    }
    if seen.iter().any(|visited| !visited) {
        return Err("cycle in detached or hosted DOM".into());
    }
    Ok(())
}

#[derive(Default)]
struct CaseResult {
    rejected: bool,
    nodes: usize,
    commands: usize,
    paint_limited: bool,
}
fn scope_invariants(commands: &[DrawCommand]) -> Result<(), String> {
    #[derive(PartialEq)]
    enum Scope {
        Clip,
        Fixed,
        Opacity,
    }
    let mut scopes = Vec::new();
    for command in commands {
        match command {
            DrawCommand::PushClip { .. } => scopes.push(Scope::Clip),
            DrawCommand::PushFixed => scopes.push(Scope::Fixed),
            DrawCommand::PushOpacity { .. } => scopes.push(Scope::Opacity),
            DrawCommand::PopClip if scopes.pop() != Some(Scope::Clip) => {
                return Err("clip scope mismatch".into());
            }
            DrawCommand::PopFixed if scopes.pop() != Some(Scope::Fixed) => {
                return Err("fixed scope mismatch".into());
            }
            DrawCommand::PopOpacity if scopes.pop() != Some(Scope::Opacity) => {
                return Err("opacity scope mismatch".into());
            }
            _ => {}
        }
        if scopes.len() > 128 {
            return Err("display list scope depth exceeded".into());
        }
    }
    if !scopes.is_empty() {
        return Err("unclosed display list scope".into());
    }
    Ok(())
}

fn pipeline(
    document: &Document,
    fonts: &Fonts,
    width: u32,
    height: u32,
) -> Result<CaseResult, String> {
    dom_invariants(document)?;
    let styles = css::compute_styles(
        document,
        &document.stylesheets(),
        width as f32,
        height as f32,
    );
    if styles.len() != document.nodes.len() {
        return Err("computed styles do not match DOM arena length".into());
    }
    for style in &styles {
        style_invariants(style)?;
    }
    let layout = layout::layout(document, &styles, width as f32, height as f32, fonts);
    finite(&[layout.content_height], "content height")?;
    if layout.content_height < 0.0
        || layout.commands.len() > 200_000
        || layout.hit_regions.len() > 100_000
    {
        return Err("layout output exceeded resource or extent limits".into());
    }
    scope_invariants(&layout.commands)?;
    let mut glyphs = 0usize;
    for command in &layout.commands {
        match command {
            DrawCommand::Rect { rect, radius, .. } => {
                rectangle(*rect)?;
                finite(&[*radius], "border radius")?;
            }
            DrawCommand::Text {
                x, y, size, text, ..
            } => {
                finite(&[*x, *y, *size], "text geometry")?;
                glyphs = glyphs.saturating_add(text.chars().count());
            }
            DrawCommand::Image { rect, .. } | DrawCommand::PushClip { rect } => rectangle(*rect)?,
            DrawCommand::PushFixed
            | DrawCommand::PopFixed
            | DrawCommand::PopClip
            | DrawCommand::PopOpacity => {}
            DrawCommand::PushOpacity { opacity } => {
                if !(0.0..=1.0).contains(opacity) {
                    return Err("invalid group opacity".into());
                }
            }
            DrawCommand::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                ..
            } => finite(&[*x1, *y1, *x2, *y2, *width], "line geometry")?,
        }
    }
    if glyphs > 500_000 {
        return Err("layout glyph limit exceeded".into());
    }
    for hit in &layout.hit_regions {
        if hit.node >= document.nodes.len() {
            return Err("hit region refers to missing DOM node".into());
        }
        rectangle(hit.rect)?;
    }
    let mut canvas = Canvas::new(width, height)?;
    canvas.clear(Color::WHITE);
    canvas.paint(&layout.commands, fonts, &ImageStore::new(), 0.0, 0.0);
    if canvas.pixels.len() != width as usize * height as usize {
        return Err("raster buffer length mismatch".into());
    }
    Ok(CaseResult {
        nodes: document.nodes.len(),
        commands: layout.commands.len(),
        paint_limited: canvas.exhausted(),
        ..CaseResult::default()
    })
}

fn exercise(
    kind: Kind,
    source: &str,
    fonts: &Fonts,
    iteration: usize,
) -> Result<CaseResult, String> {
    if source.len() > MAX_SAMPLE {
        return Err("harness sample exceeds 8 KiB".into());
    }
    match kind {
        Kind::Html => pipeline(
            &Document::parse(source),
            fonts,
            160 + (iteration % 4) as u32 * 48,
            120,
        ),
        Kind::Script => {
            let mut document = Document::parse(SCRIPT_DOCUMENT);
            let mut runtime = Runtime::new();
            let mut rejected = runtime.execute(source, &mut document).is_err();
            if !rejected {
                rejected |= runtime.dispatch_dom_content_loaded(&mut document).is_err();
                if let Some(button) = document.query_selector("#go") {
                    rejected |= runtime.dispatch_click(button, &mut document).is_err();
                }
            }
            let mut result = pipeline(&document, fonts, 192, 120)?;
            result.rejected = rejected;
            Ok(result)
        }
        Kind::Svg => match svg::render(source, Some(128), Some(96)) {
            Ok(image) => {
                if image.width > 4096
                    || image.height > 4096
                    || image.rgba.len() != image.width as usize * image.height as usize * 4
                    || image.rgba.len() > 4_194_304 * 4
                {
                    return Err("SVG image exceeded resource bounds".into());
                }
                Ok(CaseResult::default())
            }
            Err(_) => Ok(CaseResult {
                rejected: true,
                ..CaseResult::default()
            }),
        },
    }
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else {
        "non-string panic payload".into()
    }
}
fn preserve_failure(
    kind: Kind,
    iteration: usize,
    seed: u64,
    state: u64,
    source: &str,
    error: &str,
) -> Result<PathBuf, String> {
    let directory = PathBuf::from("artifacts/stress-failure");
    fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let basename = format!("{}-{iteration}-{seed:016x}", kind.name());
    let path = directory.join(format!("{basename}.{}", kind.extension()));
    fs::write(&path, source).map_err(|e| e.to_string())?;
    let metadata = format!(
        "Deterministic mutation smoke test failure\nkind: {}\niteration (zero-based): {iteration}\ninitial seed: 0x{seed:016x}\ngenerator state before sample: 0x{state:016x}\nsource bytes: {}\nfailure: {error}\nreproduce all preceding cases: cargo run --release --bin eris-stress -- {} 0x{seed:016x}\nThis is smoke fuzzing, not coverage-guided fuzzing or a security certification.\n",
        kind.name(),
        source.len(),
        iteration + 1
    );
    fs::write(directory.join(format!("{basename}.txt")), metadata).map_err(|e| e.to_string())?;
    Ok(path)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("eris-stress: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let first = args.next();
    if first
        .as_deref()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!(
            "Usage: eris-stress [ITERATIONS [SEED]]\n\nDefaults: 2000 iterations, seed 0x{DEFAULT_SEED:x}. Each iteration exercises one mutated HTML/CSS, JavaScript, and SVG input. Seeds accept decimal or 0x hexadecimal. Run under an OS timeout. Panics and invariant failures save reproducers in artifacts/stress-failure/.\n\nThis is deterministic mutation smoke fuzzing, not coverage-guided fuzzing or proof of security."
        );
        return Ok(());
    }
    let iterations: usize = first
        .map(|arg| arg.parse())
        .transpose()
        .map_err(|_| "invalid iteration count")?
        .unwrap_or(2000);
    if !(1..=1_000_000).contains(&iterations) {
        return Err("iterations must be between 1 and 1000000".into());
    }
    let seed = match args.next() {
        Some(value) => if let Some(hex) = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
        {
            u64::from_str_radix(hex, 16)
        } else {
            value.parse()
        }
        .map_err(|_| "invalid seed")?,
        None => DEFAULT_SEED,
    };
    if args.next().is_some() {
        return Err("too many arguments; use --help".into());
    }
    let start = Instant::now();
    let mut random = Random::new(seed);
    let fonts = Fonts::new();
    let mut accepted = [0usize; 3];
    let mut rejected = [0usize; 3];
    let mut paint_limited = 0;
    let mut max_nodes = 0;
    let mut max_commands = 0;
    println!(
        "Deterministic mutation smoke fuzzing: {iterations} iterations, {} cases, seed 0x{seed:016x}; inputs <=8 KiB; no network",
        iterations * 3
    );
    for iteration in 0..iterations {
        for (index, kind) in [Kind::Html, Kind::Script, Kind::Svg]
            .into_iter()
            .enumerate()
        {
            let state = random.state;
            let source = mutate(kind, &mut random, iteration);
            let result = catch_unwind(AssertUnwindSafe(|| {
                exercise(kind, &source, &fonts, iteration)
            }));
            let result = match result {
                Ok(result) => result,
                Err(payload) => Err(format!("panic: {}", panic_message(payload))),
            };
            match result {
                Ok(result) => {
                    if result.rejected {
                        rejected[index] += 1;
                    } else {
                        accepted[index] += 1;
                    }
                    paint_limited += usize::from(result.paint_limited);
                    max_nodes = max_nodes.max(result.nodes);
                    max_commands = max_commands.max(result.commands);
                }
                Err(error) => {
                    let path = preserve_failure(kind, iteration, seed, state, &source, &error)?;
                    return Err(format!(
                        "{} case {iteration} failed: {error}; reproducer {}",
                        kind.name(),
                        path.display()
                    ));
                }
            }
        }
        if (iteration + 1).is_multiple_of(500) {
            eprintln!(
                "checked {} cases ({:.2}s)",
                (iteration + 1) * 3,
                start.elapsed().as_secs_f64()
            );
        }
    }
    println!(
        "PASS: {} generated cases, zero caught panics or invariant failures in {:.2}s\nHTML pipeline: {} accepted; scripts: {} accepted / {} expected rejections; SVG: {} accepted / {} expected rejections\nMaximum DOM nodes: {max_nodes}; display commands: {max_commands}; bounded paint stops: {paint_limited}\nThis smoke run is not coverage-guided fuzzing, full conformance testing, or proof of security.",
        iterations * 3,
        start.elapsed().as_secs_f64(),
        accepted[0],
        accepted[1],
        rejected[1],
        accepted[2],
        rejected[2]
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_list_scope_invariant_distinguishes_fixed_from_clip_and_bounds_depth() {
        let clip = DrawCommand::PushClip {
            rect: Rect::default(),
        };
        assert!(
            scope_invariants(&[
                DrawCommand::PushFixed,
                clip.clone(),
                DrawCommand::PopClip,
                DrawCommand::PopFixed
            ])
            .is_ok()
        );
        assert!(
            scope_invariants(&[
                DrawCommand::PushFixed,
                clip.clone(),
                DrawCommand::PopFixed,
                DrawCommand::PopClip
            ])
            .is_err()
        );
        assert!(scope_invariants(&[DrawCommand::PopFixed]).is_err());
        assert!(scope_invariants(std::slice::from_ref(&clip)).is_err());
        let mut deep = vec![clip; 129];
        deep.extend(vec![DrawCommand::PopClip; 129]);
        assert!(scope_invariants(&deep).is_err());
        let group = DrawCommand::PushOpacity { opacity: 0.5 };
        assert!(
            scope_invariants(&[
                group.clone(),
                DrawCommand::PushFixed,
                DrawCommand::PopFixed,
                DrawCommand::PopOpacity,
            ])
            .is_ok()
        );
        assert!(
            scope_invariants(&[
                group.clone(),
                DrawCommand::PushFixed,
                DrawCommand::PopOpacity,
                DrawCommand::PopFixed,
            ])
            .is_err()
        );
        assert!(scope_invariants(&[group, DrawCommand::PopClip,]).is_err());
        assert!(scope_invariants(&[DrawCommand::PopOpacity]).is_err());
    }

    #[test]
    fn template_invariants_check_ownership_fragment_parents_and_host_cycles() {
        let mut document = Document::parse(
            "<template id=outer><b>Inert</b><template><i>Nested</i></template></template><template id=other></template>",
        );
        document.create_document_fragment();
        assert!(dom_invariants(&document).is_ok());
        let outer = document.query_selector("#outer").unwrap();
        let contents = document.template_contents(outer).unwrap();
        let other = document.query_selector("#other").unwrap();
        let mut invalid = document.clone();
        invalid.nodes[contents].parent = Some(outer);
        assert!(
            dom_invariants(&invalid)
                .unwrap_err()
                .contains("fragment has an ordinary parent")
        );
        let mut invalid = document.clone();
        invalid.nodes[contents].kind = NodeKind::DocumentFragment { host: Some(other) };
        assert!(dom_invariants(&invalid).is_err());
        let mut invalid = document.clone();
        let NodeKind::Element(element) = &mut invalid.nodes[other].kind else {
            unreachable!()
        };
        element.template_contents = Some(contents);
        assert!(dom_invariants(&invalid).is_err());
        let mut invalid = document.clone();
        let parent = invalid.nodes[outer].parent.take().unwrap();
        invalid.nodes[parent]
            .children
            .retain(|child| *child != outer);
        invalid.nodes[outer].parent = Some(contents);
        invalid.nodes[contents].children.push(outer);
        assert!(dom_invariants(&invalid).unwrap_err().contains("cycle"));
    }

    #[test]
    fn template_invariants_limit_host_inclusive_depth_with_one_shared_walk() {
        let mut document = Document::parse("<p>active</p>");
        let mut previous = None;
        for _ in 0..(eris::dom::MAX_DEPTH / 2 + 3) {
            let template = document.create_element("template");
            let contents = document.template_contents(template).unwrap();
            if let Some(parent) = previous {
                document.nodes[template].parent = Some(parent);
                document.nodes[parent].children.push(template);
            }
            previous = Some(contents);
        }
        assert!(
            dom_invariants(&document)
                .unwrap_err()
                .contains("host-inclusive DOM depth")
        );
    }

    #[test]
    fn grid_track_invariants_cover_implicit_rows_minmax_and_both_gaps() {
        let document = Document::parse(
            "<div style='display:grid;grid-template-rows:minmax(10px,2fr);grid-auto-columns:min-content 20%;grid-auto-rows:3em;gap:10% 20px'></div>",
        );
        let styles = css::compute_styles(&document, &[], 800.0, 600.0);
        let style = &styles[document.query_selector("div").unwrap()];
        assert!(style_invariants(style).is_ok());
        let mut bad = style.clone();
        std::sync::Arc::make_mut(&mut bad.grid_auto_rows)[0].min =
            GridBreadth::Length(Length::Px(f32::INFINITY));
        assert!(style_invariants(&bad).is_err());
        let mut bad = style.clone();
        bad.column_gap = Length::Percent(f32::NAN);
        assert!(style_invariants(&bad).is_err());
        for (px, percent) in [(f32::INFINITY, 0.0), (0.0, f32::NAN)] {
            let mut bad = style.clone();
            bad.width = Length::Calc {
                px,
                percent,
                has_percent: true,
            };
            assert!(style_invariants(&bad).is_err());
        }
    }

    #[test]
    fn namespace_invariants_reject_orphaned_and_mismatched_metadata() {
        let document = Document::parse(
            "<svg xmlns:xlink='http://www.w3.org/1999/xlink'><g xml:lang=en xlink:href='#x'/></svg>",
        );
        assert!(dom_invariants(&document).is_ok());
        let group = document.query_selector("g").unwrap();
        for orphaned in [false, true] {
            let mut invalid = document.clone();
            let NodeKind::Element(element) = &mut invalid.nodes[group].kind else {
                unreachable!();
            };
            if orphaned {
                element.attrs.remove("xlink:href");
            } else {
                element
                    .attr_namespaces
                    .insert("xlink:href".into(), AttributeNamespace::Xml);
            }
            assert!(
                dom_invariants(&invalid)
                    .unwrap_err()
                    .contains("attribute namespace")
            );
        }
    }
}
