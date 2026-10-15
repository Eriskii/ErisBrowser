//! Existing-API identity and compatibility controls; parent-baseline safe.
use super::*;

fn empty(runtime: &Runtime) {
    macro_rules! arena {
        ($field:ident) => {
            assert!(runtime.promises.$field.records.is_empty());
            assert_eq!(runtime.promises.$field.records.capacity(), 0);
            assert_eq!(runtime.promises.$field.admitted_capacity, 0);
        };
    }
    arena!(promises);
    arena!(gates);
    arena!(callbacks);
    arena!(captures);
    arena!(jobs);
    arena!(finally_captures);
    arena!(all_captures);
    assert!(runtime.microtasks.queue.is_empty());
    assert_eq!(runtime.microtasks.queue.capacity(), 0);
    assert_eq!(runtime.microtasks.admitted_capacity, 0);
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
}
fn native_bag(runtime: &Runtime, name: &str) -> usize {
    let Value::Native(native) = &runtime.environments[0].bindings[name].value else {
        panic!("global is not native");
    };
    native.properties.unwrap().get()
}
fn named_key<'a>(runtime: &'a Runtime, owner: usize, name: &str) -> &'a JsString {
    runtime.objects[owner]
        .values
        .keys()
        .filter_map(PropertyKey::as_string)
        .find(|key| key.units().iter().copied().eq(name.bytes().map(u16::from)))
        .unwrap()
}
fn promise_coherent(runtime: &Runtime) {
    let intrinsics = runtime.promises.intrinsics.as_ref().unwrap();
    let Value::Native(private) = &intrinsics.constructor else {
        panic!("private Promise constructor");
    };
    let Value::Native(global) = &runtime.environments[0].bindings["Promise"].value else {
        panic!("global Promise constructor");
    };
    assert!(Rc::ptr_eq(private, global));
    assert!(!runtime.environments[0].bindings["Promise"].enumerable);
    assert!(runtime.environments[0].bindings["queueMicrotask"].enumerable);
    assert_eq!(
        runtime.environments[0].bindings["queueMicrotask"].global_order,
        210
    );
    assert_eq!(
        runtime.environments[0].bindings["Promise"].global_order,
        211
    );
    assert_eq!(runtime.native_properties.len(), 299);
    assert_eq!(runtime.prototypes.len(), 25);
    assert_eq!(runtime.environments[0].bindings.len(), 212);
}

#[test]
fn shared_job_key_payloads_preserve_all_ten_function_bags_and_existing_ids() {
    let runtime = Runtime::try_new().unwrap();
    assert_eq!(runtime.objects.len(), 751);
    assert_eq!(runtime.objects.capacity(), 751);
    promise_coherent(&runtime);
    empty(&runtime);
    let producer = native_bag(&runtime, "queueMicrotask");
    let constructor = native_bag(&runtime, "Promise");
    assert_eq!((producer, constructor), (740, 741));
    assert_eq!(runtime.promises.intrinsics.as_ref().unwrap().prototype, 742);
    for owner in [740, 741, 743, 744, 745, 746, 747, 748, 749, 750] {
        for (index, name) in ["length", "name"].into_iter().enumerate() {
            let expected = named_key(&runtime, producer, name);
            let actual = named_key(&runtime, owner, name);
            assert!(
                std::ptr::eq(actual.units().as_ptr(), expected.units().as_ptr()),
                "owner={owner} key={name}"
            );
            assert!(std::ptr::eq(
                runtime.objects[owner].order[index]
                    .as_string()
                    .unwrap()
                    .units()
                    .as_ptr(),
                expected.units().as_ptr()
            ));
        }
    }
    // Sharing text never merges independently configurable property records.
    assert_eq!(runtime.objects[constructor].values.len(), 8);
    for owner in [740, 743, 744, 745, 746, 747, 748, 749, 750] {
        assert_eq!(runtime.objects[owner].values.len(), 2);
    }
}

#[test]
fn shared_job_keys_keep_public_descriptors_mutation_and_callable_identity_independent() {
    const SOURCE: &str = r#"
var log='',ok=true,q=queueMicrotask,P=Promise;
var rows=[[q,'queueMicrotask',1],[P,'Promise',1],[P.prototype.then,'then',2],
[P.resolve,'resolve',1],[P.reject,'reject',1],
[Object.getOwnPropertyDescriptor(P,Symbol.species).get,'get [Symbol.species]',0],
[P.prototype.catch,'catch',1],[P.prototype.finally,'finally',1],[P.all,'all',1],[P.race,'race',1]];
for(var i=0;i<rows.length;i++){
 var f=rows[i][0],n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length');
 ok=ok&&n.value===rows[i][1]&&!n.writable&&!n.enumerable&&n.configurable;
 ok=ok&&l.value===rows[i][2]&&!l.writable&&!l.enumerable&&l.configurable;
 ok=ok&&Object.getPrototypeOf(f)===Function.prototype;
 for(var j=0;j<i;j++)ok=ok&&f!==rows[j][0];
}
ok=ok&&q===window.queueMicrotask&&q===globalThis.queueMicrotask;
ok=ok&&Object.getOwnPropertyDescriptor(globalThis,'queueMicrotask').enumerable;
ok=ok&&!Object.getOwnPropertyDescriptor(globalThis,'Promise').enumerable;
ok=ok&&Object.getOwnPropertyNames(P).join(',')==='length,name,prototype,resolve,reject,all,race';
ok=ok&&Object.getOwnPropertyNames(P.prototype).join(',')==='constructor,then,catch,finally';
Object.defineProperty(q,'name',{value:'changed'});Object.defineProperty(q,'length',{value:99});
Object.defineProperty(P.prototype.then,'name',{value:'changed then'});
ok=ok&&q.name==='changed'&&q.length===99&&P.name==='Promise'&&P.length===1;
ok=ok&&P.prototype.then.name==='changed then'&&P.resolve.name==='resolve'&&P.reject.name==='reject'&&P.prototype.catch.name==='catch'&&P.prototype.finally.name==='finally';
q(function(){log+='Q';});P.resolve(7).catch(function(){log+='!';}).then(function(v){log+='V'+v;});log+='S';ok;
"#;
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let result = if strict {
            runtime.execute_strict(SOURCE, &mut document)
        } else {
            runtime.execute(SOURCE, &mut document)
        };
        assert_eq!(result.unwrap(), Value::Bool(true));
        assert_eq!(
            runtime.execute("log", &mut document).unwrap(),
            Value::String("SQV7".into())
        );
        assert!(runtime.console.is_empty());
        assert!(runtime.frames.is_empty());
        assert_eq!(
            (runtime.calls, runtime.eval_depth, runtime.stack_units),
            (0, 0, 0)
        );
    }
}
