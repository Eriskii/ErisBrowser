//! Bounded object adapters and immutable native callback captures.
use super::*;

pub(in crate::script) fn object_slot_plan(
    len: usize,
    physical_capacity: usize,
    count: usize,
) -> Result<GrowthPlan> {
    let mut plan = growth_plan::<ScriptObject>(len, physical_capacity, count)?;
    plan.bytes = add(plan.bytes, mul(count, size_of::<ScriptObject>())?)?;
    Ok(plan)
}

fn reserve_arena<T>(arena: &mut Arena<T>, plan: GrowthPlan) -> Result<()> {
    if let Some(target) = plan.target {
        arena
            .records
            .try_reserve_exact(target - arena.records.len())
            .map_err(|_| ScriptError::resource("Promise private arena allocation failed"))?;
    }
    Ok(())
}
fn publish_arena<T>(arena: &mut Arena<T>, plan: GrowthPlan) {
    if let Some(target) = plan.target {
        arena.admitted_capacity = target;
    }
}

#[derive(Clone, Copy)]
enum FactoryExtra<'a> {
    #[cfg(test)]
    None,
    Gate(PromiseId),
    Capture,
    Finally(&'a Value, &'a Value),
    Outcome(&'a Value),
    AllElement(AllCaptureId, usize),
}

impl Runtime {
    fn reserve_promise_objects(&mut self, plan: GrowthPlan) -> Result<()> {
        if let Some(target) = plan.target {
            self.objects
                .try_reserve_exact(target - self.objects.len())
                .map_err(|_| ScriptError::resource("Promise object arena allocation failed"))?;
        }
        Ok(())
    }

    pub(super) fn new_promise_capability_result(
        &mut self,
        capability: Capability,
    ) -> Result<Value> {
        let result = (|| {
            // Four bounded control roles and five local owners: prototype,
            // result and three keys. The producer prepaid capability cleanup.
            self.work(9)?;
            self.work(intrinsic_tree(self.prototypes.len()).0 * (1 + "Object".len() / 8))?;
            let prototype = self
                .prototypes
                .get("Object")
                .copied()
                .filter(|id| *id < self.objects.len())
                .ok_or_else(|| ScriptError::resource("Promise result Object prototype missing"))?;
            // NewPromiseCapability may have run arbitrary author construction.
            // Recompute the physical arena plan after all of those effects.
            let plan = object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
            self.work(plan.work)?;
            self.charge(plan.bytes)?;
            self.reserve_promise_objects(plan)?;
            let bag = self.dom_proto_object(Some(Value::Object(prototype)), 3)?;
            let promise = self.dom_proto_text("promise")?;
            let resolve = self.dom_proto_text("resolve")?;
            let reject = self.dom_proto_text("reject")?;
            self.dom_proto_property(
                bag,
                promise.into(),
                Property::data(capability.promise, true, true, true),
            )?;
            self.dom_proto_property(
                bag,
                resolve.into(),
                Property::data(capability.resolve, true, true, true),
            )?;
            self.dom_proto_property(
                bag,
                reject.into(),
                Property::data(capability.reject, true, true, true),
            )?;
            Ok(Value::Object(bag))
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_pending_promise(&mut self, prototype: Value) -> Result<(PromiseId, Value)> {
        let result = (|| {
            let record = growth_plan::<PromiseRecord>(
                self.promises.promises.records.len(),
                self.promises.promises.admitted_capacity,
                1,
            )?;
            let objects = object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
            // The old bag helper supplies the remaining9 and its own bytes.
            self.work(add(add(11, record.work)?, objects.work)?)?;
            self.charge(add(record.bytes, objects.bytes)?)?;
            reserve_arena(&mut self.promises.promises, record)?;
            self.reserve_promise_objects(objects)?;
            let object_id = self.dom_proto_object(Some(prototype), 0)?;
            let id = PromiseId(self.promises.promises.records.len());
            self.promises.promises.records.push(PromiseRecord {
                object_id,
                settlement: Settlement::Pending,
                reactions: Arena::default(),
                handled: false,
            });
            publish_arena(&mut self.promises.promises, record);
            Ok((id, Value::Object(object_id)))
        })();
        self.observe_microtask_failure(result)
    }

    fn promise_callback_metadata(&mut self, arity: usize) -> Result<Value> {
        let properties =
            self.dom_proto_object(Some(Value::Function(self.function_prototype)), 2)?;
        let display = self.dom_proto_text("")?;
        let length_key = self.dom_proto_text("length")?;
        self.dom_proto_property(
            properties,
            length_key.into(),
            Property::data(Value::Number(arity as f64), false, false, true),
        )?;
        let name_key = self.dom_proto_text("name")?;
        self.dom_proto_property(
            properties,
            name_key.into(),
            Property::data(Value::String(display), false, false, true),
        )?;
        self.promise_native(properties, "Promise.callback", Value::Object(properties))
    }

    fn build_promise_callbacks<const N: usize>(
        &mut self,
        kinds: [CallbackKind; N],
        extra: FactoryExtra<'_>,
    ) -> Result<[Value; N]> {
        if N == 0 || N > 2 {
            return Err(invalid());
        }
        let callbacks = growth_plan::<CallbackRecord>(
            self.promises.callbacks.records.len(),
            self.promises.callbacks.admitted_capacity,
            N,
        )?;
        let objects = object_slot_plan(self.objects.len(), self.objects.capacity(), N)?;
        let (extra_fixed, extra_plan) = match extra {
            #[cfg(test)]
            FactoryExtra::None => (
                0,
                GrowthPlan {
                    target: None,
                    work: 0,
                    bytes: 0,
                },
            ),
            FactoryExtra::Gate(_) => (
                9,
                growth_plan::<ResolveGate>(
                    self.promises.gates.records.len(),
                    self.promises.gates.admitted_capacity,
                    1,
                )?,
            ),
            FactoryExtra::Capture => (
                10,
                growth_plan::<CapabilityCapture>(
                    self.promises.captures.records.len(),
                    self.promises.captures.admitted_capacity,
                    1,
                )?,
            ),
            FactoryExtra::Finally(_, _) | FactoryExtra::Outcome(_) => (
                if matches!(extra, FactoryExtra::Finally(_, _)) {
                    14
                } else {
                    11
                },
                growth_plan::<FinallyCapture>(
                    self.promises.finally_captures.records.len(),
                    self.promises.finally_captures.admitted_capacity,
                    1,
                )?,
            ),
            FactoryExtra::AllElement(_, _) => (
                20,
                growth_plan::<AllCapture>(
                    self.promises.all_captures.records.len(),
                    self.promises.all_captures.admitted_capacity,
                    1,
                )?,
            ),
        };
        // Nine record operations plus six temporary-release credits per
        // callable precede every reserve/metadata allocation. Each metadata
        // helper then retains its original staged101work/byte schedule.
        let work = add(
            add(
                add(add(mul(15, N)?, extra_fixed)?, callbacks.work)?,
                objects.work,
            )?,
            extra_plan.work,
        )?;
        self.work(work)?;
        match extra {
            FactoryExtra::Gate(id) if id.0 >= self.promises.promises.records.len() => {
                return Err(invalid());
            }
            #[cfg(test)]
            FactoryExtra::None => {
                for kind in kinds {
                    let valid = match kind {
                        CallbackKind::Resolve(id) | CallbackKind::Reject(id) => {
                            id.0 < self.promises.gates.records.len()
                        }
                        CallbackKind::CapabilityExecutor(id) => {
                            id.0 < self.promises.captures.records.len()
                        }
                        CallbackKind::ThenFinally(id) | CallbackKind::CatchFinally(id) => {
                            matches!(
                                self.promises.finally_captures.records.get(id.0),
                                Some(FinallyCapture::Finalizer { .. })
                            )
                        }
                        CallbackKind::ReturnValue(id) | CallbackKind::ThrowReason(id) => {
                            matches!(
                                self.promises.finally_captures.records.get(id.0),
                                Some(FinallyCapture::Outcome(_))
                            )
                        }
                        CallbackKind::AllResolve(id)
                        | CallbackKind::AnyReject(id)
                        | CallbackKind::AllSettledFulfill(id)
                        | CallbackKind::AllSettledReject(id) => matches!(
                            self.promises.all_captures.records.get(id.0),
                            Some(AllCapture::Element { .. })
                        ),
                    };
                    if !valid {
                        return Err(invalid());
                    }
                }
            }
            _ => {}
        }
        // The preceding C.resolve call may have run arbitrary author code.
        // Validate the current group and prepare its count before metadata;
        // only scalar IDs/counts survive the reserves below.
        let all_remaining = if let FactoryExtra::AllElement(group, index) = extra {
            let Some(AllCapture::Group {
                values,
                remaining,
                input_done: false,
                ..
            }) = self.promises.all_captures.records.get(group.0)
            else {
                return Err(invalid());
            };
            if *remaining == 0
                || index >= values.records.len()
                || values.records.len() > values.admitted_capacity
            {
                return Err(invalid());
            }
            add(*remaining, 1)?
        } else {
            0
        };
        self.charge(add(add(callbacks.bytes, objects.bytes)?, extra_plan.bytes)?)?;
        match extra {
            #[cfg(test)]
            FactoryExtra::None => {}
            FactoryExtra::Gate(_) => reserve_arena(&mut self.promises.gates, extra_plan)?,
            FactoryExtra::Capture => reserve_arena(&mut self.promises.captures, extra_plan)?,
            FactoryExtra::Finally(_, _) | FactoryExtra::Outcome(_) => {
                reserve_arena(&mut self.promises.finally_captures, extra_plan)?;
            }
            FactoryExtra::AllElement(_, _) => {
                #[cfg(test)]
                if extra_plan.target.is_some() {
                    all_allocation_probe::before(all_allocation_probe::ReserveSite::AllCapture)?;
                }
                reserve_arena(&mut self.promises.all_captures, extra_plan)?;
                #[cfg(test)]
                if extra_plan.target.is_some() {
                    all_allocation_probe::completed(all_allocation_probe::ReserveSite::AllCapture);
                }
            }
        }
        #[cfg(test)]
        if matches!(extra, FactoryExtra::AllElement(_, _)) && callbacks.target.is_some() {
            all_allocation_probe::before(all_allocation_probe::ReserveSite::Callbacks)?;
        }
        reserve_arena(&mut self.promises.callbacks, callbacks)?;
        #[cfg(test)]
        if matches!(extra, FactoryExtra::AllElement(_, _)) && callbacks.target.is_some() {
            all_allocation_probe::completed(all_allocation_probe::ReserveSite::Callbacks);
        }
        #[cfg(test)]
        if matches!(extra, FactoryExtra::AllElement(_, _)) && objects.target.is_some() {
            all_allocation_probe::before(all_allocation_probe::ReserveSite::Objects)?;
        }
        self.reserve_promise_objects(objects)?;
        #[cfg(test)]
        if matches!(extra, FactoryExtra::AllElement(_, _)) && objects.target.is_some() {
            all_allocation_probe::completed(all_allocation_probe::ReserveSite::Objects);
        }
        let mut values = std::array::from_fn(|_| Value::Undefined);
        for (slot, kind) in values.iter_mut().zip(kinds) {
            let arity = match kind {
                CallbackKind::CapabilityExecutor(_) => 2,
                CallbackKind::ReturnValue(_) | CallbackKind::ThrowReason(_) => 0,
                _ => 1,
            };
            *slot = self.promise_callback_metadata(arity)?;
        }
        // Every fallible operation is complete. Private lengths/capacities
        // publish together; ordinary bags left by an earlier failure remain
        // unexposed and do not create any callback/gate/capture record.
        match extra {
            #[cfg(test)]
            FactoryExtra::None => {}
            FactoryExtra::Gate(id) => {
                self.promises
                    .gates
                    .records
                    .push(ResolveGate { target: Some(id) });
                publish_arena(&mut self.promises.gates, extra_plan);
            }
            FactoryExtra::Capture => {
                self.promises.captures.records.push(CapabilityCapture {
                    resolve: Value::Undefined,
                    reject: Value::Undefined,
                });
                publish_arena(&mut self.promises.captures, extra_plan);
            }
            FactoryExtra::Finally(constructor, on_finally) => {
                // All metadata exists before either retained payload is cloned.
                // The extra14 paid both clones, field moves and final releases.
                self.promises
                    .finally_captures
                    .records
                    .push(FinallyCapture::Finalizer {
                        constructor: constructor.clone(),
                        on_finally: on_finally.clone(),
                    });
                publish_arena(&mut self.promises.finally_captures, extra_plan);
            }
            FactoryExtra::Outcome(value) => {
                // Extra11 likewise covers the single immutable payload.
                self.promises
                    .finally_captures
                    .records
                    .push(FinallyCapture::Outcome(value.clone()));
                publish_arena(&mut self.promises.finally_captures, extra_plan);
            }
            FactoryExtra::AllElement(group, index) => {
                self.promises
                    .all_captures
                    .records
                    .push(AllCapture::Element {
                        group,
                        index,
                        called: false,
                    });
                publish_arena(&mut self.promises.all_captures, extra_plan);
                let AllCapture::Group { remaining, .. } =
                    &mut self.promises.all_captures.records[group.0]
                else {
                    unreachable!("validated Promise.all group");
                };
                *remaining = all_remaining;
            }
        }
        for (value, kind) in values.iter().zip(kinds) {
            let Value::Native(native) = value else {
                unreachable!("fixed native factory");
            };
            let Value::Object(object_id) = native.receiver else {
                unreachable!("fixed private receiver");
            };
            self.promises
                .callbacks
                .records
                .push(CallbackRecord { object_id, kind });
        }
        publish_arena(&mut self.promises.callbacks, callbacks);
        Ok(values)
    }

    // Isolated one-callback quota entry. The production pair/executor routes
    // use the same batch implementation, without an unused production API.
    #[cfg(test)]
    pub(super) fn new_promise_callback(&mut self, kind: CallbackKind) -> Result<Value> {
        let result = self
            .build_promise_callbacks([kind], FactoryExtra::None)
            .map(|[value]| value);
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_resolving_functions(&mut self, id: PromiseId) -> Result<(Value, Value)> {
        let gate = GateId(self.promises.gates.records.len());
        let result = self
            .build_promise_callbacks(
                [CallbackKind::Resolve(gate), CallbackKind::Reject(gate)],
                FactoryExtra::Gate(id),
            )
            .map(|[resolve, reject]| (resolve, reject));
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_capability_executor(&mut self) -> Result<(CaptureId, Value)> {
        let capture = CaptureId(self.promises.captures.records.len());
        let result = self
            .build_promise_callbacks(
                [CallbackKind::CapabilityExecutor(capture)],
                FactoryExtra::Capture,
            )
            .map(|[value]| (capture, value));
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_finally_functions(
        &mut self,
        constructor: &Value,
        on_finally: &Value,
    ) -> Result<(Value, Value)> {
        let capture = FinallyCaptureId(self.promises.finally_captures.records.len());
        let result = self
            .build_promise_callbacks(
                [
                    CallbackKind::ThenFinally(capture),
                    CallbackKind::CatchFinally(capture),
                ],
                FactoryExtra::Finally(constructor, on_finally),
            )
            .map(|[fulfilled, rejected]| (fulfilled, rejected));
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_finally_thunk(&mut self, value: &Value, kind: ReactionKind) -> Result<Value> {
        let capture = FinallyCaptureId(self.promises.finally_captures.records.len());
        let kind = match kind {
            ReactionKind::Fulfill => CallbackKind::ReturnValue(capture),
            ReactionKind::Reject => CallbackKind::ThrowReason(capture),
        };
        let result = self
            .build_promise_callbacks([kind], FactoryExtra::Outcome(value))
            .map(|[value]| value);
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_all_resolve_element(
        &mut self,
        group: AllCaptureId,
        index: usize,
    ) -> Result<Value> {
        let element = AllCaptureId(self.promises.all_captures.records.len());
        let result = self
            .build_promise_callbacks(
                [CallbackKind::AllResolve(element)],
                FactoryExtra::AllElement(group, index),
            )
            .map(|[value]| value);
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_any_reject_element(
        &mut self,
        group: AllCaptureId,
        index: usize,
    ) -> Result<Value> {
        let element = AllCaptureId(self.promises.all_captures.records.len());
        let result = self
            .build_promise_callbacks(
                [CallbackKind::AnyReject(element)],
                FactoryExtra::AllElement(group, index),
            )
            .map(|[value]| value);
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_all_settled_elements(
        &mut self,
        group: AllCaptureId,
        index: usize,
    ) -> Result<(Value, Value)> {
        // Both immutable callback identities share one append-only Element.
        // The existing batch factory publishes its count once, after both bags.
        let element = AllCaptureId(self.promises.all_captures.records.len());
        let result = self
            .build_promise_callbacks(
                [
                    CallbackKind::AllSettledFulfill(element),
                    CallbackKind::AllSettledReject(element),
                ],
                FactoryExtra::AllElement(group, index),
            )
            .map(|[fulfilled, rejected]| (fulfilled, rejected));
        self.observe_microtask_failure(result)
    }

    pub(super) fn read_finally_finalizer(
        &mut self,
        id: FinallyCaptureId,
    ) -> Result<(Value, Value)> {
        let result = (|| {
            // Check8 plus two copies and their eventual temporary releases.
            self.work(12)?;
            let Some(FinallyCapture::Finalizer {
                constructor,
                on_finally,
            }) = self.promises.finally_captures.records.get(id.0)
            else {
                return Err(invalid());
            };
            Ok((constructor.clone(), on_finally.clone()))
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn read_finally_outcome(&mut self, id: FinallyCaptureId) -> Result<Value> {
        let result = (|| {
            // Check8 plus one copy and its eventual temporary release.
            self.work(10)?;
            let Some(FinallyCapture::Outcome(value)) =
                self.promises.finally_captures.records.get(id.0)
            else {
                return Err(invalid());
            };
            Ok(value.clone())
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn callback_target(&mut self, native: &Native) -> Result<CallbackKind> {
        let result = (|| {
            self.work(search_work(self.promises.callbacks.records.len()))?;
            let Value::Object(object_id) = native.receiver else {
                return Err(invalid());
            };
            if native.name != "Promise.callback"
                || native.properties.map(|id| id.get()) != Some(object_id)
            {
                return Err(invalid());
            }
            let index = self
                .promises
                .callbacks
                .records
                .binary_search_by_key(&object_id, |record| record.object_id)
                .map_err(|_| invalid())?;
            Ok(self.promises.callbacks.records[index].kind)
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn take_resolve_gate(&mut self, id: GateId) -> Result<Option<PromiseId>> {
        let result = (|| {
            self.work(8)?;
            Ok(self
                .promises
                .gates
                .records
                .get_mut(id.0)
                .ok_or_else(invalid)?
                .target
                .take())
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn store_capability_capture(
        &mut self,
        id: CaptureId,
        arguments: &[Value],
    ) -> Result<()> {
        let result = (|| {
            self.work(14)?;
            let capture = self
                .promises
                .captures
                .records
                .get_mut(id.0)
                .ok_or_else(invalid)?;
            if !matches!(capture.resolve, Value::Undefined)
                || !matches!(capture.reject, Value::Undefined)
            {
                return Err(ScriptError::type_error(
                    "Promise capability executor already called",
                ));
            }
            capture.resolve = arguments.first().cloned().unwrap_or(Value::Undefined);
            capture.reject = arguments.get(1).cloned().unwrap_or(Value::Undefined);
            Ok(())
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn read_capability_capture(&mut self, id: CaptureId) -> Result<(Value, Value)> {
        let result = (|| {
            self.work(12)?;
            let capture = self
                .promises
                .captures
                .records
                .get(id.0)
                .ok_or_else(invalid)?;
            Ok((capture.resolve.clone(), capture.reject.clone()))
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn promise_arguments<const N: usize>(
        &mut self,
        arguments: [Value; N],
    ) -> Result<Vec<Value>> {
        let result = (|| {
            if N != 1 && N != 2 {
                return Err(invalid());
            }
            self.work(8 + 2 * N)?;
            self.charge(32 + N * size_of::<Value>())?;
            let mut values = Vec::new();
            values
                .try_reserve_exact(N)
                .map_err(|_| ScriptError::resource("Promise argument allocation failed"))?;
            values.extend(arguments);
            Ok(values)
        })();
        self.observe_microtask_failure(result)
    }
}
