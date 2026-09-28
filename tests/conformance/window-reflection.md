# Window binding reflection

The supported Window object-environment properties now participate in
`hasOwnProperty`, `propertyIsEnumerable`, `Object.keys`, `Object.values`,
`Object.getOwnPropertyNames`, `Reflect.ownKeys` and `for-in`. Reflection uses the
same binding records as descriptor queries. Own membership and enumerability do
not inspect prototypes or invoke getters. The separate global lexical environment
and private `this` binding are excluded from enumeration.

Each object-environment binding has a creation serial. Overwriting its value or
changing a supported descriptor keeps that serial; deletion and recreation assign
a new one. Own string keys list array indices numerically, then other strings by
creation order. Symbol keys retain their separate insertion order and follow the
strings in `Reflect.ownKeys`. String-only enumeration omits symbols. The existing
`for-in` driver checks live deletion and respects nonenumerable own properties
that shadow inherited enumerable names.

Supported global ECMAScript constructors/functions are initialized as writable,
nonenumerable and configurable. Math and JSON now also have those flags. The
existing immutable value properties and browser-specific bindings retain their
separate rules. These flags describe the bindings, not completeness of the objects
or functions to which they initially refer.

## Evidence

The [comparison](window-reflection.json) retains all 32 frozen self-authored probe
modes and their source/case fingerprints. The exact 16 sources are also in
[the regression fixture](../fixtures/window-reflection.tsv). Twenty-six modes
improve, two previous passes remain and four general Window definition modes
remain unsupported. The complete existing upstream inventories are compared
without changing their feature policies or source files.

Six Rust groups exercise the original probes, intrinsic flags, numeric/string/
symbol ordering across deletion and redefinition, conversion order and getter
suppression, live `for-in` behavior, and storage/work/counter refusal. Older tests
that explicitly expected Window enumeration to be unsupported now assert success.
The statement-unwind test retains its original Window source as a successful
`finally` check and uses unsupported Document enumeration for the host-stop case.
Its original looping resource cases and allocation-boundary checks are unchanged.

## Bounds and remaining behavior

Key snapshots precharge name scans, UTF-16 copies, both vectors and sorting work.
Buffers reserve fallibly and sorting uses no extra allocation. A checked monotonic
counter rejects exhaustion before publishing a new property. Binding storage
accounting includes the additional serial; the resource quotas are unchanged.
The ledger remains an allocation estimate, not exact process memory accounting.

This implements reflection over the currently supported binding inventory. It
does not complete the Window interface, frame indices, named child properties,
WindowProxy, cross-origin/realm behavior or prototype hierarchy. Event-handler
descriptor/membership reflection remains explicitly unsupported. General string
descriptor definitions, general global accessors, arbitrary UTF-16 Window property
storage and extensibility remain incomplete. Other DOM host reflection is separate
work. The four unsupported definition probes are retained rather than counted as
passing or dropped.

Primary algorithms: [property-key order](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinaryownpropertykeys),
[own membership](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.prototype.hasownproperty),
[enumerability](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-object.prototype.propertyisenumerable),
[built-in property conventions](https://tc39.es/ecma262/multipage/ecmascript-standard-built-in-objects.html),
and [Window](https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-window-object).
