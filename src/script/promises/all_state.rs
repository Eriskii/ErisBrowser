//! Prepaid private Promise.all ownership and dense result publication.
use super::*;

// Only new All routes use these probes. A warm destination has no fallible
// reserve and therefore produces no probe event. No test state enters Runtime.
macro_rules! reserve_all {
    ($records:expr, $plan:expr, $site:ident) => {
        if let Some(target) = $plan.target {
            #[cfg(test)]
            all_allocation_probe::before(all_allocation_probe::ReserveSite::$site)?;
            $records
                .try_reserve_exact(target - $records.len())
                .map_err(|_| ScriptError::resource("Promise.all backing allocation failed"))?;
            #[cfg(test)]
            all_allocation_probe::completed(all_allocation_probe::ReserveSite::$site);
        }
    };
}

fn row_plan<T>(len: usize, capacity: usize) -> Result<GrowthPlan> {
    let mut plan = growth_plan::<T>(len, capacity, 1)?;
    // Ordinary vectors use physical capacity, plus the conservative cost of
    // every newly retained row even when backing capacity is already present.
    plan.bytes = add(plan.bytes, size_of::<T>())?;
    Ok(plan)
}

fn ready_values(resolve: &mut Value, values: &mut Arena<Value>) -> ReadyAllValues {
    // Called only inside a prepaid, checked, callback-free publication block.
    // Taking the entire Arena resets logical admission as well as its Vec.
    ReadyAllValues {
        resolve: std::mem::replace(resolve, Value::Undefined),
        values: std::mem::take(values).records,
    }
}

impl Runtime {
    pub(super) fn new_all_group(&mut self, resolve: Value) -> Result<AllCaptureId> {
        let result = (|| {
            let arena = &self.promises.all_captures;
            let plan = growth_plan::<AllCapture>(arena.records.len(), arena.admitted_capacity, 1)?;
            // Four fields, record publication and fixed owner/release credits.
            self.work(add(18, plan.work)?)?;
            self.charge(plan.bytes)?;
            reserve_all!(self.promises.all_captures.records, plan, AllCapture);
            let arena = &mut self.promises.all_captures;
            let id = AllCaptureId(arena.records.len());
            arena.records.push(AllCapture::Group {
                resolve,
                values: Arena::default(),
                remaining: 1,
                input_done: false,
            });
            if let Some(target) = plan.target {
                arena.admitted_capacity = target;
            }
            Ok(id)
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn append_all_placeholder(&mut self, id: AllCaptureId) -> Result<usize> {
        let result = (|| {
            // Direct lookup, checked index, initialization and eventual release.
            self.work(12)?;
            let Some(AllCapture::Group {
                values,
                input_done: false,
                ..
            }) = self.promises.all_captures.records.get(id.0)
            else {
                return Err(invalid());
            };
            let index = values.records.len();
            let plan = growth_plan::<Value>(index, values.admitted_capacity, 1)?;
            self.work(plan.work)?;
            self.charge(plan.bytes)?;
            // No author callback or other arena mutation has intervened.
            let AllCapture::Group { values, .. } = &mut self.promises.all_captures.records[id.0]
            else {
                unreachable!("validated Promise.all group");
            };
            if let Some(target) = plan.target {
                values
                    .records
                    .try_reserve_exact(target - values.records.len())
                    .map_err(|_| ScriptError::resource("Promise.all values allocation failed"))?;
            }
            values.records.push(Value::Undefined);
            if let Some(target) = plan.target {
                values.admitted_capacity = target;
            }
            Ok(index)
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn complete_all_element(
        &mut self,
        id: AllCaptureId,
        value: &Value,
    ) -> Result<Option<ReadyAllValues>> {
        let result = (|| {
            self.work(12)?;
            let Some(AllCapture::Element {
                group,
                index,
                called,
            }) = self.promises.all_captures.records.get(id.0)
            else {
                return Err(invalid());
            };
            // Duplicates do not inspect a completed group's moved-out values.
            if *called {
                return Ok(None);
            }
            let (group, index) = (*group, *index);
            self.work(16)?;
            let Some(AllCapture::Group {
                values,
                remaining,
                input_done,
                ..
            }) = self.promises.all_captures.records.get(group.0)
            else {
                return Err(invalid());
            };
            if index >= values.records.len() || values.records.len() > values.admitted_capacity {
                return Err(invalid());
            }
            let next = remaining.checked_sub(1).ok_or_else(invalid)?;
            if next == 0 && !*input_done {
                return Err(invalid());
            }
            // The first mutation and its clone are funded. No author call,
            // allocation, fallible lookup or arithmetic remains in this block.
            let ready = {
                let AllCapture::Group {
                    resolve,
                    values,
                    remaining,
                    ..
                } = &mut self.promises.all_captures.records[group.0]
                else {
                    unreachable!("validated Promise.all group");
                };
                values.records[index] = value.clone();
                *remaining = next;
                if next == 0 {
                    Some(ready_values(resolve, values))
                } else {
                    None
                }
            };
            let AllCapture::Element { called, .. } = &mut self.promises.all_captures.records[id.0]
            else {
                unreachable!("validated Promise.all element");
            };
            *called = true;
            Ok(ready)
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_all_settled_snapshot(
        &mut self,
        kind: ReactionKind,
        value: &Value,
    ) -> Result<Value> {
        let result = (|| {
            // Fixed checks/planning, one payload clone and six owner releases.
            // Lookup, object backing and staged bag/text/property fees follow.
            self.work(17)?;
            let (status, payload) = match kind {
                ReactionKind::Fulfill => ("fulfilled", "value"),
                ReactionKind::Reject => ("rejected", "reason"),
            };
            self.work(install::tree(self.prototypes.len()).0)?;
            let prototype = self
                .prototypes
                .get("Object")
                .copied()
                .filter(|id| *id < self.objects.len())
                .ok_or_else(invalid)?;
            let objects =
                factories::object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
            self.work(objects.work)?;
            self.charge(objects.bytes)?;
            reserve_all!(self.objects, objects, Objects);
            let bag = self.dom_proto_object(Some(Value::Object(prototype)), 2)?;
            let status_key = self.dom_proto_text("status")?;
            let status_text = self.dom_proto_text(status)?;
            let payload_key = self.dom_proto_text(payload)?;
            self.dom_proto_property(
                bag,
                status_key.into(),
                Property::data(Value::String(status_text), true, true, true),
            )?;
            self.dom_proto_property(
                bag,
                payload_key.into(),
                Property::data(value.clone(), true, true, true),
            )?;
            Ok(Value::Object(bag))
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn complete_all_settled_element(
        &mut self,
        id: AllCaptureId,
        kind: ReactionKind,
        value: &Value,
    ) -> Result<Option<ReadyAllValues>> {
        let result = (|| {
            self.work(12)?;
            let Some(AllCapture::Element {
                group,
                index,
                called,
            }) = self.promises.all_captures.records.get(id.0)
            else {
                return Err(invalid());
            };
            // Either callback consumes the same gate. Duplicates do not inspect
            // moved-out Group storage or retain the caller's payload.
            if *called {
                return Ok(None);
            }
            let (group, index) = (*group, *index);
            self.work(14)?;
            let Some(AllCapture::Group {
                values,
                remaining,
                input_done,
                ..
            }) = self.promises.all_captures.records.get(group.0)
            else {
                return Err(invalid());
            };
            if index >= values.records.len() || values.records.len() > values.admitted_capacity {
                return Err(invalid());
            }
            let next = remaining.checked_sub(1).ok_or_else(invalid)?;
            if next == 0 && !*input_done {
                return Err(invalid());
            }
            let AllCapture::Element { called, .. } = &mut self.promises.all_captures.records[id.0]
            else {
                unreachable!("validated Promise.allSettled element");
            };
            *called = true;
            // Snapshot creation has no author call or AllCapture mutation.
            // A terminal refusal leaves this gate consumed and Group untouched.
            let snapshot = self.new_all_settled_snapshot(kind, value)?;
            let AllCapture::Group {
                resolve,
                values,
                remaining,
                ..
            } = &mut self.promises.all_captures.records[group.0]
            else {
                unreachable!("validated Promise.allSettled group");
            };
            // All remaining publication is prepaid and cannot fail.
            values.records[index] = snapshot;
            *remaining = next;
            Ok(if next == 0 {
                Some(ready_values(resolve, values))
            } else {
                None
            })
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn finish_all_input(&mut self, id: AllCaptureId) -> Result<Option<ReadyAllValues>> {
        let result = (|| {
            self.work(16)?;
            let Some(AllCapture::Group {
                resolve,
                values,
                remaining,
                input_done,
            }) = self.promises.all_captures.records.get_mut(id.0)
            else {
                return Err(invalid());
            };
            if *input_done || values.records.len() > values.admitted_capacity {
                return Err(invalid());
            }
            let next = remaining.checked_sub(1).ok_or_else(invalid)?;
            // Only normal exhaustion calls this method. An ordinary abort
            // keeps this sentinel so saved callbacks cannot finish the group.
            *input_done = true;
            *remaining = next;
            Ok(if next == 0 {
                Some(ready_values(resolve, values))
            } else {
                None
            })
        })();
        self.observe_microtask_failure(result)
    }

    pub(super) fn new_all_values_array(&mut self, values: Vec<Value>) -> Result<Value> {
        let result = (|| {
            self.work(23)?;
            let id = self.arrays.len();
            if self.array_properties.len() != id
                || self.array_holes.len() != id
                || self.array_lengths.len() != id
            {
                return Err(invalid());
            }
            let prototype = self
                .array_prototype
                .filter(|prototype| *prototype < id)
                .ok_or_else(invalid)?;
            let length = u32::try_from(values.len()).map_err(|_| invalid())?;
            let logical = add(32, mul(values.len(), size_of::<Value>())?)?;
            let objects =
                factories::object_slot_plan(self.objects.len(), self.objects.capacity(), 1)?;
            let arrays = row_plan::<Vec<Value>>(id, self.arrays.capacity())?;
            let properties = row_plan::<usize>(id, self.array_properties.capacity())?;
            let holes = row_plan::<BTreeSet<usize>>(id, self.array_holes.capacity())?;
            let lengths =
                row_plan::<array_properties::ArrayLength>(id, self.array_lengths.capacity())?;
            let mut work = 0;
            let mut bytes = logical;
            for plan in [objects, arrays, properties, holes, lengths] {
                work = add(work, plan.work)?;
                bytes = add(bytes, plan.bytes)?;
            }
            self.work(work)?;
            self.charge(bytes)?;
            // Reserve every destination before the first object/Array publish.
            reserve_all!(self.objects, objects, Objects);
            reserve_all!(self.arrays, arrays, ArrayValues);
            reserve_all!(self.array_properties, properties, ArrayProperties);
            reserve_all!(self.array_holes, holes, ArrayHoles);
            reserve_all!(self.array_lengths, lengths, ArrayLengths);
            let properties = self.dom_proto_object(Some(Value::Array(prototype)), 0)?;
            // Existing bag9 is the final fallible stage. These pushes are all
            // reserved, own-data publication; no mutable Array or setters run.
            self.arrays.push(values);
            self.array_properties.push(properties);
            self.array_holes.push(BTreeSet::new());
            self.array_lengths.push(array_properties::ArrayLength {
                value: length,
                writable: true,
            });
            Ok(Value::Array(id))
        })();
        self.observe_microtask_failure(result)
    }
}

#[cfg(test)]
mod all_settled_quota_tests;
