//! Natural final Any rejection consumes its gate before an intrinsic reserve failure.
use super::*;

#[test]
fn any_final_reject_reserve_failure_consumes_ready_and_suspends_queued_author_work() {
    use all_allocation_probe::{ReserveSite, fail_once};
    const SOURCE: &str = r###"
var saved,trace='',caught=0,finalized=0,after=false;
function C(exec){exec(v=>{trace+='F';},e=>{trace+='R';});}
var thenable={then:(yes,no)=>{saved=no;trace+='T';}};
C.resolve=v=>thenable;
Promise.any.call(C,[1]);trace+='S';
var reason={};
var queued=()=>{trace+='Q';};
var trigger=()=>{try{trace+='J';saved(reason);after=true;}catch(e){caught++;trace+='K';}finally{finalized++;trace+='Z';}after=true;};
trigger
"###;
    for strict in [false, true] {
        let mut runtime = Runtime::try_new().unwrap();
        let mut document = Document::parse("");
        let trigger = if strict {
            runtime.execute_strict(SOURCE, &mut document)
        } else {
            runtime.execute(SOURCE, &mut document)
        }
        .unwrap();
        assert!(matches!(trigger, Value::Function(_)));
        assert_eq!(
            runtime.environments[0].bindings["trace"].value,
            Value::String("TS".into())
        );
        let element = runtime
            .promises
            .callbacks
            .records
            .iter()
            .find_map(|record| {
                if let CallbackKind::AnyReject(element) = record.kind {
                    Some(element)
                } else {
                    None
                }
            })
            .unwrap();
        let AllCapture::Element {
            group,
            index,
            called,
        } = &runtime.promises.all_captures.records[element.0]
        else {
            panic!("Element")
        };
        assert_eq!((*index, *called), (0, false));
        let group = *group;
        let AllCapture::Group {
            values,
            remaining,
            input_done,
            ..
        } = &runtime.promises.all_captures.records[group.0]
        else {
            panic!("Group")
        };
        assert_eq!((*remaining, *input_done), (1, true));
        assert_eq!(values.records, [Value::Undefined]);
        let queued = runtime.environments[0].bindings["queued"].value.clone();
        runtime
            .queue_microtask(&Value::Undefined, std::slice::from_ref(&queued))
            .unwrap();
        assert_eq!(runtime.microtasks.queue.len(), 1);
        // Arrow trigger/callbacks avoid unrelated arguments objects or Arrays.
        // Only ArrayValues is forced cold; the object path has two spare slots.
        runtime.objects.try_reserve_exact(2).unwrap();
        assert!(runtime.objects.capacity() >= runtime.objects.len() + 2);
        runtime.arrays = std::mem::take(&mut runtime.arrays)
            .into_boxed_slice()
            .into_vec();
        assert_eq!(runtime.arrays.len(), runtime.arrays.capacity());
        let objects = runtime.objects.len();
        let rows = [
            runtime.arrays.len(),
            runtime.array_properties.len(),
            runtime.array_holes.len(),
            runtime.array_lengths.len(),
        ];
        assert!(rows.iter().all(|length| *length == rows[0]));
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
            rows
        );
        assert_eq!(runtime.arrays.capacity(), rows[0]);
        assert_eq!(runtime.objects.len(), objects + 1);
        assert_eq!(
            runtime.objects[objects].intrinsic_error,
            Some("AggregateError")
        );
        assert!(runtime.objects[objects].values.is_empty());
        assert!(runtime.objects[objects].order.is_empty());
        let AllCapture::Element { called, .. } = &runtime.promises.all_captures.records[element.0]
        else {
            panic!("Element")
        };
        assert!(*called);
        let AllCapture::Group {
            resolve,
            values,
            remaining,
            input_done,
        } = &runtime.promises.all_captures.records[group.0]
        else {
            panic!("Group")
        };
        assert_eq!(resolve, &Value::Undefined);
        assert_eq!(
            (
                *remaining,
                *input_done,
                values.records.len(),
                values.admitted_capacity
            ),
            (0, true, 0, 0)
        );
        assert_eq!(values.records.capacity(), 0);
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
                Value::String("TSJ".into())
            );
            for name in ["caught", "finalized"] {
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
                Value::String("TSJ".into())
            );
        }
        assert_eq!(guard.observation().attempts, observed.attempts);
        assert_eq!(guard.observation().completed, observed.completed);
    }
}
