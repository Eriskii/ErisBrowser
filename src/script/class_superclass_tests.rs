//! Additive public regression for a bound nonconstructible live superclass.
use super::*;

#[test]
fn class_public_bound_nonconstructor_superclass_validation() {
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let source = r###"function check(v){if(!v)throw new Error('class superclass oracle');}
var result=(function(){
 var trace='',called=0;
 function Base(){trace+='B';called++;}
 var target=()=>{trace+='X';called++;return {};};
 var bad=target.bind(null,7);
 class Explicit extends Base {constructor(){
  trace+='E';super((trace+='A',Object.setPrototypeOf(Explicit,Base),1));trace+='L';
 }}
 class Default extends Base {}
 Object.setPrototypeOf(Explicit,bad);
 try{new Explicit();throw 'missing explicit TypeError';}
 catch(e){check(e instanceof TypeError);trace+='T';}
 try{new Default((trace+='D',Object.setPrototypeOf(Default,bad),2));throw 'missing default TypeError';}
 catch(e){check(e instanceof TypeError);trace+='T';}
 check(called===0&&Object.getPrototypeOf(Explicit)===Base&&Object.getPrototypeOf(Default)===bad);
 return trace;
})();
"###;
        let readback = r###"result
"###;
        let result = if strict {
            runtime.execute_strict(source, &mut document)
        } else {
            runtime.execute(source, &mut document)
        };
        result.unwrap_or_else(|error| panic!("bound superclass strict={strict}: {error:?}"));
        let observed = if strict {
            runtime.execute_strict(readback, &mut document)
        } else {
            runtime.execute(readback, &mut document)
        }
        .unwrap();
        assert_eq!(observed, Value::String("EATDT".into()), "strict={strict}");
    }
}
