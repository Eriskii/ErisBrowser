//! Inert layout mirrors only; root may register this test before Promise exists.
#[allow(dead_code)]
mod shapes {
    use crate::script::Value;
    use std::collections::VecDeque;
    pub struct PromiseId(pub usize);
    pub struct GateId(pub usize);
    pub struct CaptureId(pub usize);
    pub struct JobId(pub usize);
    pub struct Arena<T> {
        pub records: Vec<T>,
        pub admitted_capacity: usize,
    }
    pub struct State {
        pub promises: Arena<PromiseRecord>,
        pub gates: Arena<ResolveGate>,
        pub callbacks: Arena<CallbackRecord>,
        pub captures: Arena<CapabilityCapture>,
        pub jobs: Arena<Option<PromiseJob>>,
        pub intrinsics: Option<Intrinsics>,
    }
    pub struct Intrinsics {
        pub constructor: Value,
        pub prototype: usize,
    }
    pub struct PromiseRecord {
        pub object_id: usize,
        pub settlement: Settlement,
        pub reactions: Arena<ReactionPair>,
        pub handled: bool,
    }
    pub enum Settlement {
        Pending,
        Fulfilled(Value),
        Rejected(Value),
    }
    pub struct Capability {
        pub promise: Value,
        pub resolve: Value,
        pub reject: Value,
    }
    pub struct ReactionPair {
        pub fulfilled: Option<Value>,
        pub rejected: Option<Value>,
        pub capability: Capability,
    }
    pub struct ResolveGate {
        pub target: Option<PromiseId>,
    }
    pub struct CallbackRecord {
        pub object_id: usize,
        pub kind: CallbackKind,
    }
    pub enum CallbackKind {
        Resolve(GateId),
        Reject(GateId),
        CapabilityExecutor(CaptureId),
    }
    pub struct CapabilityCapture {
        pub resolve: Value,
        pub reject: Value,
    }
    pub enum PromiseJob {
        Reaction {
            handler: Option<Value>,
            capability: Capability,
            argument: Value,
            kind: ReactionKind,
        },
        ResolveThenable {
            target: PromiseId,
            thenable: Value,
            then: Value,
        },
    }
    pub enum ReactionKind {
        Fulfill,
        Reject,
    }
    pub enum QueueJob {
        Callback(Value),
        Promise(JobId),
    }
    pub struct MicrotaskState {
        pub queue: VecDeque<QueueJob>,
        pub admitted_capacity: usize,
        pub author_depth: usize,
        pub performing_checkpoint: bool,
        pub suspended: bool,
    }
}
#[test]
fn proposed_promise_private_type_layouts() {
    use shapes::*;
    use std::mem::{align_of, size_of};
    macro_rules! row {
        ($name:literal, $ty:ty) => {
            println!(
                "PROMISE_LAYOUT {} bytes={} align={}",
                $name,
                size_of::<$ty>(),
                align_of::<$ty>()
            );
        };
    }
    row!("Value", crate::script::Value);
    row!("ScriptObject", crate::script::ScriptObject);
    row!("CurrentMicrotaskState", super::microtasks::State);
    row!("ProposedMicrotaskState", MicrotaskState);
    row!("State", State);
    row!("Intrinsics", Intrinsics);
    row!("PromiseRecord", PromiseRecord);
    row!("ReactionPair", ReactionPair);
    row!("Capability", Capability);
    row!("ResolveGate", ResolveGate);
    row!("CallbackRecord", CallbackRecord);
    row!("CapabilityCapture", CapabilityCapture);
    row!("PromiseJob", PromiseJob);
    row!("JobSlot", Option<PromiseJob>);
    row!("QueueJob", QueueJob);
    row!("PromiseArena", Arena<PromiseRecord>);
    row!("PromiseId", PromiseId);
    row!("OptionPromiseId", Option<PromiseId>);
}
