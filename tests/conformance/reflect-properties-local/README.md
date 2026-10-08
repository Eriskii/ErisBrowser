# Independent Reflect property fixtures

This source proposal covers the nine missing Reflect property methods. It has
14 named fresh-realm bodies / 28 strict and sloppy modes, plus five positive/wrong
pairs / 20 control modes. No fixture, parser, compiler or engine has run. These
files live outside the TypedArray worktree and do not change its frozen evidence.

Load `cases.js` and invoke one member of `reflectPropertyCases` per fresh realm.
For strict mode, prepend the directive to the complete file and invocation.
Successful bodies return true. Preserve source bytes when a runner is prepared;
do not extract or rewrite the negative branches to make a baseline healthy.

Every case begins with successful calls to all nine methods before any negative
catch. TypedArray-specific cases also verify a genuine initialized indexed view.
The controls have smaller same-method prerequisites. Wrong partners differ only
in their final literal comparison and require a successful positive in the same
mode plus intrinsic runtime Error identity. Prerequisite TypeErrors are unhealthy.

The implementation review should keep these distinctions explicit:

- Invalid primitive targets fail before property-key hooks; successful keys use
  one string-hint conversion, followed by a fresh property or buffer witness.
- An omitted Receiver defaults to the target. Explicit undefined, null and
  primitive Receivers are preserved for strict accessors.
- Ordinary writable target data plus a Receiver own accessor returns false
  without calling its setter. A setter on the target receives the actual Receiver.
- An invalid canonical index on a TypedArray target with a distinct Receiver
  returns true without value conversion. An ordinary target writing through an
  invalid TypedArray Receiver instead reaches DefineOwnProperty and returns false.
- Same-Receiver TypedArray assignment converts its value before fresh bounds,
  so a callback can make an index valid or invalid. Callback-thrown TypeErrors
  propagate with identity; they are not Boolean refusal shortcuts.
- Presence and own-descriptor queries avoid getters. Descriptors are fresh copies;
  integer-indexed deletion can refuse despite configurable:true in its descriptor.
- PreventExtensions and SetPrototypeOf return Boolean refusal. Resizable-backed
  views stay extensible; ordinary cycle and nonextensible changes return false.

Ordinary objects, arrays, functions, boxed strings and authentic Number
TypedArrays provide the target domain. The fixture plan adds no Proxy, class,
cross-realm, BigInt, shared/immutable buffer or host reflection support. The
existing four Reflect methods remain prerequisites, not newly claimed work.

The parent reported complete local TypedArray validation while this proposal was
authored. Its actual public predecessor must be bound additively before running
the next baseline. No current result, predicted pass count or complete ECMAScript
support is claimed here.

Primary algorithm references read for this plan are recorded in `spec-notes.json`.
