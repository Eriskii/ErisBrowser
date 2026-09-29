// CASE: two-hundred-nested-captures
var p='hello';for(var i=0;i<200;i++)p='('+p+')';var m=new RegExp(p).exec('hello');assert.sameValue(m.length,201);assert.sameValue(m.index,0);assert.sameValue(m.input,'hello');for(i=0;i<201;i++)assert.sameValue(m[i],'hello');
// CASE: two-hundred-noncapturing-groups
var p='hello';for(var i=0;i<200;i++)p='(?:'+p+')';var m=new RegExp(p).exec('hello');assert.sameValue(m.length,1);assert.sameValue(m[0],'hello');
// CASE: two-hundred-empty-captures
var p='';for(var i=0;i<200;i++)p+='()';var m=new RegExp(p).exec('');assert.sameValue(m.length,201);for(i=0;i<201;i++)assert.sameValue(m[i],'');
// CASE: deep-named-capture
var p='(?<deep>x)';for(var i=0;i<199;i++)p='('+p+')';var m=new RegExp(p).exec('x');assert.sameValue(m.length,201);assert.sameValue(m[200],'x');assert.sameValue(m.groups.deep,'x');
// CASE: two-hundredth-backreference
var p='(a)';for(var i=0;i<199;i++)p='('+p+')';var m=new RegExp(p+'\\200').exec('aa');assert.sameValue(m.length,201);assert.sameValue(m[0],'aa');assert.sameValue(m[200],'a');
// CASE: deep-repeat-clears-inner-capture
var p='(a(b)?)*';for(var i=0;i<197;i++)p='('+p+')';var m=new RegExp('^'+p+'$').exec('aba');assert.sameValue(m.length,200);for(i=0;i<198;i++)assert.sameValue(m[i],'aba');assert.sameValue(m[198],'a');assert.sameValue(m[199],undefined);
// CASE: deep-lazy-alternation
var p='(a|ab)+?b';for(var i=0;i<200;i++)p='(?:'+p+')';var m=new RegExp(p).exec('aab');assert.sameValue(m[0],'aab');assert.sameValue(m[1],'a');
// CASE: deep-group-single-lookahead
var p='(?=(a))a';for(var i=0;i<200;i++)p='(?:'+p+')';var m=new RegExp(p).exec('a');assert.sameValue(m.length,2);assert.sameValue(m[1],'a');
// CASE: deep-unclosed-group-is-syntax-error
var p='a';for(var i=0;i<200;i++)p='('+p+')';p='('+p;assert.throws(SyntaxError,function(){new RegExp(p);});
// CASE: deep-empty-alternative
var p='(a|)b';for(var i=0;i<200;i++)p='(?:'+p+')';var r=new RegExp(p),m=r.exec('b');assert.sameValue(m[0],'b');assert.sameValue(m[1],'');m=r.exec('ab');assert.sameValue(m[0],'ab');assert.sameValue(m[1],'a');
