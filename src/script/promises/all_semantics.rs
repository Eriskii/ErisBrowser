//! Observable Promise combinator iteration over the realm's job machinery.
use super::*;

pub(super) struct AllIterator {
    iterator: Value,
    next: Value,
    done: bool,
}

fn terminal(error: &ScriptError) -> bool {
    error.is_resource_limit() || error.is_unsupported()
}

impl Runtime {
    pub(super) fn promise_all(
        &mut self,
        constructor: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // Fixed orchestration and temporary cleanup; Get/Call/Construct,
        // retained handle copies and private admission remain additional.
        self.work(16)?;
        self.work(1)?;
        let Capability {
            promise,
            resolve,
            reject,
        } = self.new_promise_capability(constructor.clone(), doc)?;
        let mut iterator = None;
        let result = (|| {
            // GetPromiseResolve precedes any access to the iterable.
            self.work(1)?;
            let promise_resolve = self.promise_named_get(constructor.clone(), "resolve", doc)?;
            if !json_callable(&promise_resolve) {
                return Err(ScriptError::type_error("Promise resolve is not callable"));
            }
            self.work(1)?;
            let items = args.first().cloned().unwrap_or(Value::Undefined);
            let method = self
                .array_from_iterator_method(&items, doc)?
                .ok_or_else(|| ScriptError::type_error("Promise.all input is not iterable"))?;
            let object = self.call(method, Vec::new(), items, doc)?;
            if !js_object(&object) {
                return Err(ScriptError::type_error("iterator is not an object"));
            }
            self.work(1)?;
            let next = self.promise_named_get(object.clone(), "next", doc)?;
            // A failed next getter never completes iterator acquisition.
            // Call, rather than this acquisition step, checks callability.
            iterator = Some(AllIterator {
                iterator: object,
                next,
                done: false,
            });
            let group = self.new_all_group(resolve)?;
            let record = iterator.as_mut().expect("acquired iterator record");
            loop {
                let Some(value) = self.all_iterator_step(record, doc)? else {
                    if let Some(ready) = self.finish_all_input(group)? {
                        self.resolve_all_ready(ready, doc)?;
                    }
                    return Ok(());
                };
                // Reserve the indexed slot before invoking cached C.resolve.
                let index = self.append_all_placeholder(group)?;
                self.work(2)?;
                let arguments = self.promise_arguments([value])?;
                let next_promise =
                    self.call(promise_resolve.clone(), arguments, constructor.clone(), doc)?;
                // This factory revalidates current private state after the
                // author call and commits the count before any then getter.
                let fulfilled = self.new_all_resolve_element(group, index)?;
                self.work(2)?;
                let rejected = reject.clone();
                let then = self.promise_named_get(next_promise.clone(), "then", doc)?;
                let arguments = self.promise_arguments([fulfilled, rejected])?;
                self.call(then, arguments, next_promise, doc)?;
            }
        })();
        if let Err(mut error) = result {
            if terminal(&error) {
                return Err(error);
            }
            if let Some(record) = iterator.as_ref().filter(|record| !record.done) {
                error = self.array_from_close(&record.iterator, error, doc);
            }
            // Fatal close failures stay terminal. An ordinary pending error
            // retains its original value even if return itself throws.
            let reason = self.exception_value(error)?;
            let arguments = self.promise_arguments([reason])?;
            // An abrupt custom reject escapes once; it is never rejected again.
            self.call(reject, arguments, Value::Undefined, doc)?;
        }
        Ok(promise)
    }

    pub(super) fn promise_all_settled(
        &mut self,
        constructor: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // Fixed orchestration and temporary cleanup; Get/Call/Construct,
        // retained handle copies and private admission remain additional.
        self.work(16)?;
        self.work(1)?;
        let Capability {
            promise,
            resolve,
            reject,
        } = self.new_promise_capability(constructor.clone(), doc)?;
        let mut iterator = None;
        let result = (|| {
            // GetPromiseResolve precedes any access to the iterable.
            self.work(1)?;
            let promise_resolve = self.promise_named_get(constructor.clone(), "resolve", doc)?;
            if !json_callable(&promise_resolve) {
                return Err(ScriptError::type_error("Promise resolve is not callable"));
            }
            self.work(1)?;
            let items = args.first().cloned().unwrap_or(Value::Undefined);
            let method = self
                .array_from_iterator_method(&items, doc)?
                .ok_or_else(|| {
                    ScriptError::type_error("Promise.allSettled input is not iterable")
                })?;
            let object = self.call(method, Vec::new(), items, doc)?;
            if !js_object(&object) {
                return Err(ScriptError::type_error("iterator is not an object"));
            }
            self.work(1)?;
            let next = self.promise_named_get(object.clone(), "next", doc)?;
            // A failed next getter never completes iterator acquisition.
            // Call, rather than this acquisition step, checks callability.
            iterator = Some(AllIterator {
                iterator: object,
                next,
                done: false,
            });
            let group = self.new_all_group(resolve)?;
            let record = iterator.as_mut().expect("acquired iterator record");
            loop {
                let Some(value) = self.all_iterator_step(record, doc)? else {
                    if let Some(ready) = self.finish_all_input(group)? {
                        self.resolve_all_ready(ready, doc)?;
                    }
                    return Ok(());
                };
                // Reserve the indexed slot before invoking cached C.resolve.
                let index = self.append_all_placeholder(group)?;
                self.work(2)?;
                let arguments = self.promise_arguments([value])?;
                let next_promise =
                    self.call(promise_resolve.clone(), arguments, constructor.clone(), doc)?;
                // This factory revalidates current private state after the
                // author call and commits the count before any then getter.
                let (fulfilled, rejected) = self.new_all_settled_elements(group, index)?;
                self.work(1)?;
                let then = self.promise_named_get(next_promise.clone(), "then", doc)?;
                let arguments = self.promise_arguments([fulfilled, rejected])?;
                self.call(then, arguments, next_promise, doc)?;
            }
        })();
        if let Err(mut error) = result {
            if terminal(&error) {
                return Err(error);
            }
            if let Some(record) = iterator.as_ref().filter(|record| !record.done) {
                error = self.array_from_close(&record.iterator, error, doc);
            }
            // Fatal close failures stay terminal. An ordinary pending error
            // retains its original value even if return itself throws.
            let reason = self.exception_value(error)?;
            let arguments = self.promise_arguments([reason])?;
            // An abrupt custom reject escapes once; it is never rejected again.
            self.call(reject, arguments, Value::Undefined, doc)?;
        }
        Ok(promise)
    }

    pub(super) fn promise_race(
        &mut self,
        constructor: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // Eight units for orchestration, nine for outer handle cleanup.
        // Unlike all, race retains resolve without transferring it to a Group.
        self.work(17)?;
        self.work(1)?;
        let Capability {
            promise,
            resolve,
            reject,
        } = self.new_promise_capability(constructor.clone(), doc)?;
        let mut iterator = None;
        let result = (|| {
            self.work(1)?;
            let promise_resolve = self.promise_named_get(constructor.clone(), "resolve", doc)?;
            if !json_callable(&promise_resolve) {
                return Err(ScriptError::type_error("Promise resolve is not callable"));
            }
            self.work(1)?;
            let items = args.first().cloned().unwrap_or(Value::Undefined);
            let method = self
                .array_from_iterator_method(&items, doc)?
                .ok_or_else(|| ScriptError::type_error("Promise.race input is not iterable"))?;
            let object = self.call(method, Vec::new(), items, doc)?;
            if !js_object(&object) {
                return Err(ScriptError::type_error("iterator is not an object"));
            }
            self.work(1)?;
            let next = self.promise_named_get(object.clone(), "next", doc)?;
            iterator = Some(AllIterator {
                iterator: object,
                next,
                done: false,
            });
            let record = iterator.as_mut().expect("acquired iterator record");
            loop {
                let Some(value) = self.all_iterator_step(record, doc)? else {
                    // Empty/exhausted input does not itself settle the capability.
                    return Ok(());
                };
                self.work(2)?;
                let arguments = self.promise_arguments([value])?;
                let next_promise =
                    self.call(promise_resolve.clone(), arguments, constructor.clone(), doc)?;
                // Retain the Get receiver and both shared capability callbacks.
                // No per-element wrapper or early-settlement check is introduced.
                self.work(3)?;
                let then = self.promise_named_get(next_promise.clone(), "then", doc)?;
                let arguments = self.promise_arguments([resolve.clone(), reject.clone()])?;
                self.call(then, arguments, next_promise, doc)?;
            }
        })();
        if let Err(mut error) = result {
            if terminal(&error) {
                return Err(error);
            }
            if let Some(record) = iterator.as_ref().filter(|record| !record.done) {
                error = self.array_from_close(&record.iterator, error, doc);
            }
            let reason = self.exception_value(error)?;
            let arguments = self.promise_arguments([reason])?;
            self.call(reject, arguments, Value::Undefined, doc)?;
        }
        Ok(promise)
    }

    fn all_iterator_step(
        &mut self,
        record: &mut AllIterator,
        doc: &mut Document,
    ) -> Result<Option<Value>> {
        let result = (|| {
            self.work(8)?;
            self.work(2)?;
            let step = self.call(
                record.next.clone(),
                Vec::new(),
                record.iterator.clone(),
                doc,
            )?;
            if !js_object(&step) {
                return Err(ScriptError::type_error("iterator result is not an object"));
            }
            // Get consumes its receiver; keep the original for the value Get.
            self.work(1)?;
            let done = self.promise_named_get(step.clone(), "done", doc)?;
            if done.truthy() {
                record.done = true;
                return Ok(None);
            }
            self.promise_named_get(step, "value", doc).map(Some)
        })();
        if matches!(&result, Err(error) if !terminal(error)) {
            // IteratorStepValue marks an abrupt next/done/value step done.
            record.done = true;
        }
        result
    }

    fn resolve_all_ready(&mut self, ready: ReadyAllValues, doc: &mut Document) -> Result<Value> {
        let ReadyAllValues { resolve, values } = ready;
        let array = self.new_all_values_array(values)?;
        let arguments = self.promise_arguments([array])?;
        self.call(resolve, arguments, Value::Undefined, doc)
    }

    pub(super) fn run_all_callback(
        &mut self,
        element: AllCaptureId,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // The private helper pays before retaining the argument, checks the
        // once bit before moved-out storage, and commits before author calls.
        let value = args.first().unwrap_or(&Value::Undefined);
        match self.complete_all_element(element, value)? {
            Some(ready) => self.resolve_all_ready(ready, doc),
            None => Ok(Value::Undefined),
        }
    }

    pub(super) fn run_all_settled_callback(
        &mut self,
        element: AllCaptureId,
        kind: ReactionKind,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let value = args.first().unwrap_or(&Value::Undefined);
        match self.complete_all_settled_element(element, kind, value)? {
            Some(ready) => self.resolve_all_ready(ready, doc),
            None => Ok(Value::Undefined),
        }
    }
}

#[cfg(test)]
mod race_quota_tests;
