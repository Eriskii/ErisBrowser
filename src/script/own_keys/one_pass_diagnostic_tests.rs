use super::*;

#[test]
fn own_key_pass_diagnostic_boundaries() {
    for (name, source) in [
        ("eight", "({z:1,a:2,m:3,b:4,y:5,c:6,x:7,d:8})"),
        ("shared", "Object.getPrototypeOf(Uint8Array.prototype)"),
        (
            "mixed",
            "var q={};for(var i=0;i<18;i++)q['k'+i]=i;q[Symbol('one')]=1;q[Symbol('two')]=2;q;",
        ),
        (
            "symbols",
            "var q={};for(var i=0;i<20;i++)q[Symbol('s')]=i;q;",
        ),
        (
            "leading",
            "var q={};q[Symbol('one')]=1;q[Symbol('two')]=2;for(var i=0;i<6;i++)q['k'+i]=i;q;",
        ),
        (
            "digit-first",
            "var q={'2':99};for(var i=0;i<19;i++)q['k'+i]=i;q;",
        ),
        (
            "digit-last",
            "var q={};for(var i=0;i<19;i++)q['k'+i]=i;q['2']=99;q;",
        ),
    ] {
        for boundary in ["helper", "own_keys", "for_in_snapshot"] {
            let mut runtime = Runtime::try_new().unwrap();
            let mut document = Document::parse("<p>unchanged</p>");
            let value = runtime.execute(source, &mut document).unwrap();
            let id = runtime.property_object(&value).unwrap();
            let order = runtime.objects[id].order.clone();
            let fields = format!("{:?}", runtime.objects[id].values);
            let object_count = runtime.objects.len();
            let moved = value.clone();
            let before = (runtime.steps, runtime.allocated);
            let keys = match boundary {
                "helper" => runtime.own_key_stored_strings(id).unwrap(),
                "own_keys" => Some(runtime.own_keys(&value).unwrap()),
                _ => Some(runtime.for_in_snapshot(moved).unwrap().1),
            };
            let work = before.0 - runtime.steps;
            let heap = runtime.allocated - before.1;
            let count = keys.as_ref().map(Vec::len);
            eprintln!(
                "OWN_KEY_BOUNDARY case={name} boundary={boundary} work={work} heap={heap} count={count:?}"
            );
            assert_eq!(runtime.objects[id].order, order);
            assert_eq!(format!("{:?}", runtime.objects[id].values), fields);
            assert_eq!(runtime.objects.len(), object_count);
            assert!(runtime.frames.is_empty());
            assert!(runtime.active_array_joins.is_empty());
            assert_eq!(
                (
                    runtime.calls,
                    runtime.eval_depth,
                    runtime.stack_units,
                    runtime.json_depth
                ),
                (0, 0, 0, 0)
            );
        }
    }
}
