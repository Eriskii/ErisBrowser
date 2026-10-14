//! Promise abstract operations over the realm's private records and shared FIFO.
//! Existing Get/Call/Construct/exception charges remain additional to the named
//! fixed orchestration, handle-copy and private record admission envelopes.
use super::*;

fn semantic_overflow() -> ScriptError {
    ScriptError::resource("Promise semantic accounting overflow")
}

fn species_key_work(count: usize) -> Result<usize> {
    if count == 0 {
        return Ok(2);
    }
    // Pinned B=6 comparison envelope; the well-known table is currently15.
    let mut capacity = count.checked_add(1).ok_or_else(semantic_overflow)? / 2;
    let mut height = 1usize;
    while capacity >= 6 {
        capacity /= 6;
        height = height.checked_add(1).ok_or_else(semantic_overflow)?;
    }
    count
        .min(height.checked_mul(11).ok_or_else(semantic_overflow)?)
        .checked_add(2)
        .ok_or_else(semantic_overflow)
}

impl Runtime {
    fn promise_named_get(
        &mut self,
        receiver: Value,
        name: &str,
        doc: &mut Document,
    ) -> Result<Value> {
        // Every caller uses a fixed ASCII key. This pays both text copies,
        // then reuses ordinary/host Get without bypassing its author getters.
        let key = PropertyKey::String(self.dom_proto_text(name)?);
        self.get_property_key(receiver, &key, doc)
    }

    fn promise_default_constructor(&mut self) -> Result<Value> {
        // Private cache validation8 plus one retained handle copy.
        self.work(9)?;
        self.promises
            .intrinsics
            .as_ref()
            .map(|intrinsics| intrinsics.constructor.clone())
            .ok_or_else(|| ScriptError::resource("Promise intrinsics are not initialized"))
    }

    fn promise_default_prototype(&mut self) -> Result<Value> {
        self.work(8)?;
        self.promises
            .intrinsics
            .as_ref()
            .map(|intrinsics| Value::Object(intrinsics.prototype))
            .ok_or_else(|| ScriptError::resource("Promise intrinsics are not initialized"))
    }

    fn promise_species_constructor(
        &mut self,
        receiver: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let constructor = self.promise_named_get(receiver, "constructor", doc)?;
        if matches!(constructor, Value::Undefined) {
            return self.promise_default_constructor();
        }
        if !js_object(&constructor) {
            return Err(ScriptError::type_error(
                "Promise constructor is not an object",
            ));
        }
        // The symbol-key and receiver handles are paid before either copy.
        self.work(species_key_work(self.well_known_symbol_count())?)?;
        let key = self.well_known_key("species");
        let species = self.get_property_key(constructor.clone(), &key, doc)?;
        if matches!(species, Value::Null | Value::Undefined) {
            return self.promise_default_constructor();
        }
        self.work(1)?;
        if !self.is_constructor(species.clone())? {
            return Err(ScriptError::type_error(
                "Promise species is not a constructor",
            ));
        }
        Ok(species)
    }

    fn new_promise_capability(
        &mut self,
        constructor: Value,
        doc: &mut Document,
    ) -> Result<Capability> {
        self.work(1)?;
        if !self.is_constructor(constructor.clone())? {
            return Err(ScriptError::type_error(
                "Promise capability target is not a constructor",
            ));
        }
        let (capture, executor) = self.new_capability_executor()?;
        let arguments = self.promise_arguments([executor])?;
        let promise = self.construct(constructor, arguments, doc)?;
        // Capture reading pays12 before two handle copies and their cleanup.
        // Values, not an independent flag, determine whether a second capture
        // call is legal; store_capability_capture owns that selected rule.
        let (resolve, reject) = self.read_capability_capture(capture)?;
        if !json_callable(&resolve) || !json_callable(&reject) {
            return Err(ScriptError::type_error(
                "Promise capability callbacks are not callable",
            ));
        }
        Ok(Capability {
            promise,
            resolve,
            reject,
        })
    }

    pub(in crate::script) fn promise_construct(
        &mut self,
        args: &[Value],
        new_target: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        let result = (|| {
            // Fixed validation and bounded temporary cleanup. Callable checking
            // precedes the observable NewTarget.prototype Get.
            self.work(8)?;
            let executor = args.first().unwrap_or(&Value::Undefined);
            if !json_callable(executor) {
                return Err(ScriptError::type_error("Promise executor is not callable"));
            }
            let prototype = self.promise_named_get(new_target, "prototype", doc)?;
            let prototype = if js_object(&prototype) {
                prototype
            } else {
                self.promise_default_prototype()?
            };
            let (id, promise) = self.new_pending_promise(prototype)?;
            let (resolve, reject) = self.new_resolving_functions(id)?;
            // Keep reject for abrupt executor completion; both new copies are
            // paid before making the owned two-argument vector.
            self.work(2)?;
            let executor = executor.clone();
            let arguments = self.promise_arguments([resolve, reject.clone()])?;
            if let Err(error) = self.call(executor, arguments, Value::Undefined, doc) {
                // Resource/Unsupported remain fatal here; exception_value
                // returns those errors unchanged and preserves thrown Values.
                let reason = self.exception_value(error)?;
                let arguments = self.promise_arguments([reason])?;
                self.call(reject, arguments, Value::Undefined, doc)?;
            }
            Ok(promise)
        })();
        self.observe_microtask_failure(result)
    }

    fn promise_catch(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // Invoke performs an observable GetV before Call. Retain the original
        // receiver and forward the unfiltered argument without Promise branding.
        self.work(1)?;
        let method = self.promise_named_get(receiver.clone(), "then", doc)?;
        self.work(1)?;
        let rejected = args.first().cloned().unwrap_or(Value::Undefined);
        let arguments = self.promise_arguments([Value::Undefined, rejected])?;
        self.call(method, arguments, receiver, doc)
    }

    fn promise_finally(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        if !js_object(&receiver) {
            return Err(ScriptError::type_error(
                "Promise.finally receiver is not an object",
            ));
        }
        self.work(1)?;
        let constructor = self.promise_species_constructor(receiver.clone(), doc)?;
        let on_finally = args.first().unwrap_or(&Value::Undefined);
        // Species is observable even when a noncallable argument passes through.
        let (fulfilled, rejected) = if json_callable(on_finally) {
            self.new_finally_functions(&constructor, on_finally)?
        } else {
            self.work(2)?;
            (on_finally.clone(), on_finally.clone())
        };
        self.work(1)?;
        let method = self.promise_named_get(receiver.clone(), "then", doc)?;
        let arguments = self.promise_arguments([fulfilled, rejected])?;
        self.call(method, arguments, receiver, doc)
    }

    fn promise_then(
        &mut self,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let id = self
            .promise_id(&receiver)?
            .ok_or_else(|| ScriptError::type_error("Promise.then receiver is not a Promise"))?;
        let constructor = self.promise_species_constructor(receiver, doc)?;
        let capability = self.new_promise_capability(constructor, doc)?;
        let fulfilled = args.first().filter(|value| json_callable(value));
        let rejected = args.get(1).filter(|value| json_callable(value));
        self.work(1)?;
        let promise = capability.promise.clone();
        // No source-state snapshot survives generic capability construction.
        // This helper rereads the live source and then admits the pending
        // record or settled job, with no author code across its paid commit.
        self.register_promise_then(id, fulfilled, rejected, capability)?;
        Ok(promise)
    }

    fn promise_resolve(
        &mut self,
        constructor: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        if !js_object(&constructor) {
            return Err(ScriptError::type_error(
                "Promise.resolve receiver is not an object",
            ));
        }
        self.work(1)?;
        let value = args.first().cloned().unwrap_or(Value::Undefined);
        self.promise_resolve_with_constructor(constructor, value, doc)
    }

    // Internal PromiseResolve uses its captured constructor, never C.resolve
    // or the mutable global binding. The old public resolve body/fees stay here.
    fn promise_resolve_with_constructor(
        &mut self,
        constructor: Value,
        value: Value,
        doc: &mut Document,
    ) -> Result<Value> {
        if self.promise_id(&value)?.is_some() {
            self.work(1)?;
            let observed = self.promise_named_get(value.clone(), "constructor", doc)?;
            // Genuine Promise identity precedes IsConstructor(C), including
            // an object-valued C that is not a constructor.
            if self.object_is_values(&observed, &constructor)? {
                return Ok(value);
            }
        }
        let Capability {
            promise,
            resolve,
            reject: _,
        } = self.new_promise_capability(constructor, doc)?;
        let arguments = self.promise_arguments([value])?;
        self.call(resolve, arguments, Value::Undefined, doc)?;
        Ok(promise)
    }

    fn promise_reject(
        &mut self,
        constructor: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let Capability {
            promise,
            resolve: _,
            reject,
        } = self.new_promise_capability(constructor, doc)?;
        self.work(1)?;
        let reason = args.first().cloned().unwrap_or(Value::Undefined);
        let arguments = self.promise_arguments([reason])?;
        self.call(reject, arguments, Value::Undefined, doc)?;
        Ok(promise)
    }

    pub(in crate::script) fn promise_native_operation(
        &mut self,
        name: &str,
        receiver: Value,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let result = (|| {
            // One fixed public-operation envelope, including the bounded
            // argument/capability/return temporaries on all abrupt paths.
            self.work(8)?;
            match name {
                "Promise" => Err(ScriptError::type_error("Promise constructor requires new")),
                "Promise.then" => self.promise_then(receiver, args, doc),
                "Promise.catch" => self.promise_catch(receiver, args, doc),
                "Promise.finally" => self.promise_finally(receiver, args, doc),
                "Promise.resolve" => self.promise_resolve(receiver, args, doc),
                "Promise.reject" => self.promise_reject(receiver, args, doc),
                "Promise.getSpecies" => Ok(receiver),
                _ => Err(ScriptError::resource("invalid private Promise operation")),
            }
        })();
        self.observe_microtask_failure(result)
    }

    fn resolve_promise_value(
        &mut self,
        id: PromiseId,
        value: Value,
        doc: &mut Document,
    ) -> Result<()> {
        let object_id = self.promise_object_id(id)?;
        if matches!(&value, Value::Object(found) if *found == object_id) {
            let reason =
                self.exception_value(ScriptError::type_error("Promise cannot resolve to itself"))?;
            return self.settle_promise(id, ReactionKind::Reject, reason);
        }
        if !js_object(&value) {
            return self.settle_promise(id, ReactionKind::Fulfill, value);
        }
        self.work(1)?;
        let then = match self.promise_named_get(value.clone(), "then", doc) {
            Ok(then) => then,
            Err(error) => {
                let reason = self.exception_value(error)?;
                return self.settle_promise(id, ReactionKind::Reject, reason);
            }
        };
        if json_callable(&then) {
            self.enqueue_resolve_thenable(id, &value, &then)
        } else {
            self.settle_promise(id, ReactionKind::Fulfill, value)
        }
    }

    fn run_finally_callback(
        &mut self,
        capture: FinallyCaptureId,
        kind: ReactionKind,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        // Fixed orchestration and bounded temporary cleanup precede any copy.
        self.work(8)?;
        let (constructor, on_finally) = self.read_finally_finalizer(capture)?;
        self.work(1)?;
        let outcome = args.first().cloned().unwrap_or(Value::Undefined);
        let result = self.call(on_finally, Vec::new(), Value::Undefined, doc)?;
        let promise = self.promise_resolve_with_constructor(constructor, result, doc)?;
        let thunk = self.new_finally_thunk(&outcome, kind)?;
        self.work(1)?;
        let method = self.promise_named_get(promise.clone(), "then", doc)?;
        let arguments = self.promise_arguments([thunk])?;
        self.call(method, arguments, promise, doc)
    }

    pub(in crate::script) fn promise_callback(
        &mut self,
        native: &Native,
        args: &[Value],
        doc: &mut Document,
    ) -> Result<Value> {
        let result = (|| {
            // Only immutable metadata identity chooses a record. The dynamic
            // invocation receiver never supplies a gate or capture ID.
            match self.callback_target(native)? {
                CallbackKind::Resolve(gate) => {
                    let Some(id) = self.take_resolve_gate(gate)? else {
                        return Ok(Value::Undefined);
                    };
                    // Consume the original gate before argument processing or
                    // Get(then); reentrant use cannot reject this resolution.
                    self.work(1)?;
                    let value = args.first().cloned().unwrap_or(Value::Undefined);
                    self.resolve_promise_value(id, value, doc)?;
                }
                CallbackKind::Reject(gate) => {
                    let Some(id) = self.take_resolve_gate(gate)? else {
                        return Ok(Value::Undefined);
                    };
                    self.work(1)?;
                    let reason = args.first().cloned().unwrap_or(Value::Undefined);
                    self.settle_promise(id, ReactionKind::Reject, reason)?;
                }
                CallbackKind::CapabilityExecutor(capture) => {
                    self.store_capability_capture(capture, args)?;
                }
                CallbackKind::ThenFinally(capture) => {
                    return self.run_finally_callback(capture, ReactionKind::Fulfill, args, doc);
                }
                CallbackKind::CatchFinally(capture) => {
                    return self.run_finally_callback(capture, ReactionKind::Reject, args, doc);
                }
                CallbackKind::ReturnValue(capture) => return self.read_finally_outcome(capture),
                CallbackKind::ThrowReason(capture) => {
                    let reason = self.read_finally_outcome(capture)?;
                    // The exact original payload is retained. Formatting uses
                    // the existing non-observing error path, not author getters.
                    return Err(self.thrown_error(reason)?);
                }
            }
            Ok(Value::Undefined)
        })();
        self.observe_microtask_failure(result)
    }

    pub(in crate::script) fn run_promise_job(
        &mut self,
        job: PromiseJob,
        doc: &mut Document,
    ) -> Result<Value> {
        let result = (|| match job {
            PromiseJob::Reaction {
                handler,
                capability,
                argument,
                kind,
            } => {
                // This is deliberately the first body debit, including an
                // absent handler. Shared dispatch already paid32 before take.
                let arguments = self.promise_arguments([argument])?;
                let Capability {
                    promise: _,
                    resolve,
                    reject,
                } = capability;
                let Some(handler) = handler else {
                    // Forward exact values using the same paid argument list;
                    // absence is not a manufactured call or thrown diagnostic.
                    let callback = match kind {
                        ReactionKind::Fulfill => resolve,
                        ReactionKind::Reject => reject,
                    };
                    return self.call(callback, arguments, Value::Undefined, doc);
                };
                let (callback, value) = match self.call(handler, arguments, Value::Undefined, doc) {
                    Ok(value) => (resolve, value),
                    Err(error) => (reject, self.exception_value(error)?),
                };
                let arguments = self.promise_arguments([value])?;
                // An abrupt custom capability callback remains an abrupt job,
                // for the shared dispatcher's dedicated Promise report path.
                self.call(callback, arguments, Value::Undefined, doc)
            }
            PromiseJob::ResolveThenable {
                target,
                thenable,
                then,
            } => {
                let (resolve, reject) = self.new_resolving_functions(target)?;
                self.work(1)?;
                let arguments = self.promise_arguments([resolve, reject.clone()])?;
                if let Err(error) = self.call(then, arguments, thenable, doc) {
                    let reason = self.exception_value(error)?;
                    let arguments = self.promise_arguments([reason])?;
                    self.call(reject, arguments, Value::Undefined, doc)?;
                }
                Ok(Value::Undefined)
            }
        })();
        self.observe_microtask_failure(result)
    }
}
