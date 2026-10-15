//! One natural AggregateError entry reaches the unchanged dense-Array probe.
use super::*;

#[test]
fn aggregate_error_dense_array_reserve_failure_suspends_author_and_queue() {
    use all_allocation_probe::{ReserveSite, fail_once};
    let mut runtime = Runtime::try_new().unwrap();
    let mut document = Document::parse("");
    let trigger = runtime.execute(
        r###"var trace='',returnGets=0,returnCalls=0,caught=0,finalized=0,after=false;
var doneResult={done:true};Object.defineProperty(doneResult,'value',{get:()=>{throw new Error('exhausted value read');}});
var it={next:()=>{trace+='N';return doneResult;}};
Object.defineProperty(it,'return',{get:()=>{returnGets++;trace+='R';return ()=>{returnCalls++;trace+='r';return {};};}});
var iteratorMethod=()=>{trace+='i';return it;};
var items={};Object.defineProperty(items,Symbol.iterator,{get:()=>{trace+='I';return iteratorMethod;}});
var queued=()=>{trace+='Q';};
var trigger=()=>{try{AggregateError(items);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='F';}after=true;};
trigger"###,
        &mut document,
    ).unwrap();
    assert!(matches!(trigger, Value::Function(_)));
    let queued = runtime.environments[0].bindings["queued"].value.clone();
    runtime
        .queue_microtask(&Value::Undefined, std::slice::from_ref(&queued))
        .unwrap();
    assert_eq!(runtime.microtasks.queue.len(), 1);
    // Arrow callbacks above avoid creating unrelated arguments objects/Arrays.
    // Physical preparation before the operation: one fresh instance plus the
    // possible Array bag are warm; ArrayValues alone is forced to need reserve.
    runtime.objects.try_reserve_exact(2).unwrap();
    assert!(runtime.objects.capacity() - runtime.objects.len() >= 2);
    runtime.arrays = std::mem::take(&mut runtime.arrays)
        .into_boxed_slice()
        .into_vec();
    assert_eq!(runtime.arrays.len(), runtime.arrays.capacity());
    let old_objects = runtime.objects.len();
    let old_rows = [
        runtime.arrays.len(),
        runtime.array_properties.len(),
        runtime.array_holes.len(),
        runtime.array_lengths.len(),
    ];
    assert!(old_rows.iter().all(|length| *length == old_rows[0]));
    let guard = fail_once(ReserveSite::ArrayValues);
    let result = runtime.with_author_entry(&mut document, |runtime, document| {
        runtime.call(trigger, Vec::new(), Value::Undefined, document)
    });
    let error = result.unwrap_err();
    assert!(error.is_resource_limit());
    assert_eq!(error.message, "injected Promise.all reserve refusal");
    let observed = guard.observation();
    assert_eq!(observed.attempts, [0, 0, 0, 1, 0, 0, 0]);
    assert_eq!(observed.completed, [0; 7]);
    assert_eq!(
        [
            runtime.arrays.len(),
            runtime.array_properties.len(),
            runtime.array_holes.len(),
            runtime.array_lengths.len()
        ],
        old_rows
    );
    assert_eq!(runtime.arrays.capacity(), old_rows[0]);
    // Earlier private instance admission persists; errors/Array publication did not happen.
    assert_eq!(runtime.objects.len(), old_objects + 1);
    assert_eq!(
        runtime.objects[old_objects].intrinsic_error,
        Some("AggregateError")
    );
    assert!(runtime.objects[old_objects].values.is_empty());
    assert!(runtime.objects[old_objects].order.is_empty());
    assert!(runtime.microtasks_suspended());
    assert!(runtime.frames.is_empty());
    assert!(runtime.console.is_empty());
    assert_eq!(
        (runtime.calls, runtime.eval_depth, runtime.stack_units),
        (0, 0, 0)
    );
    let counters = (
        runtime.steps,
        runtime.allocated,
        runtime.microtasks.admitted_capacity,
    );
    for _ in 0..2 {
        assert_eq!(
            runtime.environments[0].bindings["trace"].value,
            Value::String("IiN".into())
        );
        for name in ["returnGets", "returnCalls", "caught", "finalized"] {
            assert_eq!(
                runtime.environments[0].bindings[name].value,
                Value::Number(0.0)
            );
        }
        assert_eq!(
            runtime.environments[0].bindings["after"].value,
            Value::Bool(false)
        );
        assert_eq!(runtime.microtasks.queue.len(), 1);
        assert_eq!(
            runtime.microtasks.queue.front(),
            Some(&QueueJob::Callback(queued.clone()))
        );
        assert_eq!(runtime.microtasks.author_depth, 0);
        assert!(!runtime.microtasks.performing_checkpoint);
        runtime.microtask_checkpoint(&mut document).unwrap();
        assert!(runtime.microtasks_suspended());
        assert_eq!(
            (
                runtime.steps,
                runtime.allocated,
                runtime.microtasks.admitted_capacity
            ),
            counters
        );
        assert_eq!(runtime.microtasks.queue.len(), 1);
        assert_eq!(
            runtime.microtasks.queue.front(),
            Some(&QueueJob::Callback(queued.clone()))
        );
        assert_eq!(
            runtime.environments[0].bindings["trace"].value,
            Value::String("IiN".into())
        );
    }
    assert_eq!(guard.observation().attempts, observed.attempts);
    assert_eq!(guard.observation().completed, observed.completed);
}
