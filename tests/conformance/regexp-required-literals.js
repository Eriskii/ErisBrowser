// CASE: missing-second-hyphen
var s='a';for(var i=0;i<9;i++)s+=s;assert.sameValue(/[^-]*-([^-][^-]*-)*-/.test(s+'-'+s),false);
// CASE: missing-closing-brackets
var s='a';for(var i=0;i<9;i++)s+=s;assert.sameValue(/[^\]]*\]([^\]]+\])*\]+/.test(s),false);
// CASE: missing-question-mark
var s='a';for(var i=0;i<9;i++)s+=s;assert.sameValue(/[^?]*\?+/.test(s),false);
// CASE: common-alternative-literal-missing
var s='x';for(var i=0;i<9;i++)s+=s;assert.sameValue(/.*(?:abz|cdz)/.test(s),false);
// CASE: required-delimiters-present
var m=/[^-]*-([^-][^-]*-)*-/.exec('aa-bb--');assert.sameValue(m[0],'aa-bb--');assert.sameValue(m[1],'bb-');assert.sameValue(m.index,0);
// CASE: lookahead-counts-do-not-overlap
assert.sameValue(/(?=a)a+/.exec('a')[0],'a');assert.sameValue(/(?!a)b+/.exec('bb')[0],'bb');assert.sameValue(/(?=(a))a+\1/.exec('aa')[1],'a');
// CASE: alternative-intersection-and-empty-branch
assert.sameValue(/(?:ab|a)+/.exec('a')[0],'a');assert.sameValue(/(?:abc|)+/.exec('')[0],'');assert.sameValue(/(?:a+b|c+)/.exec('ccc')[0],'ccc');
// CASE: optional-groups-and-zero-repeats
assert.sameValue(/(?:ab)?c*/.exec('')[0],'');assert.sameValue(/(?:abc){0}d*/.exec('ddd')[0],'ddd');assert.sameValue(/(?:a{3})*b+/.exec('b')[0],'b');
// CASE: backreferences-do-not-add-requirements
var m=/(?:a|(b))+\1/.exec('bb');assert.sameValue(m[0],'bb');assert.sameValue(m[1],'b');assert.sameValue(/(a)?b+\1/.exec('b')[0],'b');
// CASE: ignore-case-canonicalization
assert.sameValue(/a+A/i.exec('Aa')[0],'Aa');assert.sameValue(/\u03c2+\u03c3/i.exec('\u03a3\u03c2')[0],'\u03a3\u03c2');assert.sameValue(/s+/i.test('\u017f'),false);assert.sameValue(/k+/i.test('\u212a'),false);
// CASE: surrogate-code-unit-counts
assert.sameValue(/\ud800+\ud800/.exec('\ud800\ud800')[0],'\ud800\ud800');assert.sameValue(/\ud83d+\ude00/.exec('\ud83d\ude00')[0],'\ud83d\ude00');assert.sameValue(/\u0000+\u0000/.test('\u0000\u0000'),true);
// CASE: global-start-and-reset
var r=/a+b/g;r.lastIndex=2;assert.sameValue(r.exec('abaaab').index,2);assert.sameValue(r.lastIndex,6);assert.sameValue(r.exec('abaaab'),null);assert.sameValue(r.lastIndex,0);
// CASE: sticky-start-and-reset
var r=/a+b/y;r.lastIndex=1;assert.sameValue(r.exec('xaaab')[0],'aaab');assert.sameValue(r.lastIndex,5);r.lastIndex=2;assert.sameValue(r.exec('abxx'),null);assert.sameValue(r.lastIndex,0);
// CASE: more-than-four-required-units
assert.sameValue(/a+bcdef/.exec('abcdef')[0],'abcdef');assert.sameValue(/(?:abcdef|ab)+g/.exec('abg')[0],'abg');assert.sameValue(/(?:abcdef)?g+/.exec('g')[0],'g');
// CASE: saturating-positive-repeat-counts
assert.sameValue(/(?:(?:a{1000000}){1000000}){1000000}b*/.test('ab'),false);assert.sameValue(/(?:(?:a{1000000}){1000000}){0}b*/.exec('b')[0],'b');
