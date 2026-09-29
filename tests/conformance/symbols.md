# Symbols and property keys

The [constructor-policy follow-up](constructor-policy.md) admits Reflect call/construct
and `new.target` metadata. Current results: **182 passed, 6 failed, 54 unsupported**.
The inventory and source bytes are unchanged. Measurements and policy descriptions
below retain the history of earlier checkpoints.

The later [Window reflection checkpoint](window-reflection.md) adds the two global
Symbol descriptor modes: that checkpoint records **168 passed / six failed /
68 unsupported**, with all 64 controls verified. The original measurements below
remain the evidence for the Symbol implementation itself.

The [String concat follow-up](string-concat.md) preserves those counts and all
controls. The two removed-wrapper conversion failures now reach missing Date;
they remain failures and are not promoted to passing cases.

Eris implements Symbol primitives in its own Rust interpreter. Unique symbols
retain identity independently of descriptions; the registry uses exact UTF-16
keys. Well-known symbols have stable identities. `Symbol` is callable but cannot
be constructed. Description access, primitive/boxed receivers, `Symbol.for`,
`Symbol.keyFor`, `toString`, `valueOf` and intrinsic descriptors are implemented.
The registry currently belongs to one Runtime, the existing page execution agent;
multiple realms and shared cross-realm registries are not implemented.

Object property storage now distinguishes strings from symbol identities.
Assignment, deletion, `in`, descriptors, inherited accessors, object initializers,
method/accessor names and `Object.defineProperties` preserve symbol keys.
`Object.getOwnPropertySymbols` returns symbols in insertion order;
`Reflect.ownKeys` combines indexed strings, other strings and symbols. Existing
string-only enumeration, `for-in` and JSON omit symbol keys. JSON omits primitive
symbol values in objects, uses null in arrays and returns undefined at the root;
replacer and toJSON callbacks still run where required.

`Symbol.toPrimitive` runs before ordinary conversion and receives the appropriate
hint. Symbols remain valid property keys and throw when numeric or implicit
string conversion is required. Explicit `String(symbol)` produces its descriptive
string. Reference conversion remains deferred: plain assignment evaluates its
RHS first, and compound assignment retains the key converted for its read.
`Symbol.toStringTag` is an observable, mutable property, including the initial
Math, JSON, Reflect and Symbol tags. `instanceof` uses `Symbol.hasInstance`, with
ordinary function behavior and bound-target delegation.

The conversion audit also routes Math arguments, apply's array-like length,
array length writes, slice indices and join separators through checked conversion.
Slice reads live properties after index conversion, retaining holes and inherited
values when a callback shrinks the source array. Join snapshots its length before
separator conversion. These changes do not complete all generic Array methods.

## Inventory and evidence

The profile imports **123 unchanged sources** from revision
`7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd` of
[tc39/test262](https://github.com/tc39/test262/tree/7ab7fafa0003f73fc85c1b95d88094d33f7eb8bd/test/built-ins/Symbol):
all 98 `.js` files in the complete Symbol tree, 12 in Object/getOwnPropertySymbols
and 13 in Reflect/ownKeys. The Symbol subtree was separately fetched without
truncation before the importer verified all 25 directory counts and Git blob
hashes. Sources, metadata, harness files and licenses retain their original bytes.
All **242 required modes** remain in every report.

The profile has 64 assertion controls, including paired successful and deliberately
incorrect checks for identity, registry, key storage, reflection, primitive hooks,
tags, instance hooks and JSON. A no-op assertion implementation and the wrong
exception identity fail the runner's independent Python checks. Existing profiles
retain their original execution policies, even where that policy still excludes
Symbol-tagged cases; this profile does not silently broaden older baselines.

The final counts and exact comparisons are recorded in
[symbol-properties.json](symbol-properties.json), with the
[complete Symbol report](test262-symbols-properties.json) and its
[passing-case baseline](test262-symbols-current.json).

```sh
python3 tools/test262_conformance.py --profile symbols \
  --baseline tests/conformance/test262-symbols-current.json
```

## Bounds and missing behavior

Symbol storage, registry scans, property-key lists, descriptive strings, custom
tags and function names use the existing work/allocation ledger. New storage is
charged before publication. Author conversion and instance callbacks share the
existing uncatchable resource limits and unwind the execution state. Formatting
a Symbol for legacy host/console display charges its description scan and UTF-8
allocation before formatting. This does not complete legacy host string coercion.
No script quota was raised, and these estimates are not whole-process accounting.

The subsequent [DOM binding checkpoint](dom-string-conversion.md) replaces display
formatting in the supported DOM operations with checked string conversion and
actual receiver dispatch. Console formatting remains separate; complete DOMString
storage and Web IDL coverage still need work.

Identity constants alone do not implement their protocols. Iteration,
async iteration/disposal, regular-expression Symbol dispatch, species construction,
concat spreadability and unscopables remain incomplete. Proxies, other realms,
Map, Set, Promise, Date, classes and broader ECMAScript behavior remain missing.
Only `ownKeys` is implemented on Reflect. String enumeration on some host objects
remains explicitly unsupported, even where their symbol properties are supported.
This is neither a complete Symbol-protocol result nor full Test262 conformance.

Primary algorithms: [Symbol objects](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-symbol-objects),
[ToPrimitive](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-toprimitive),
[ToPropertyKey](https://tc39.es/ecma262/multipage/abstract-operations.html#sec-topropertykey),
[GetValue](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-getvalue),
[InstanceofOperator](https://tc39.es/ecma262/multipage/ecmascript-language-expressions.html#sec-instanceofoperator),
and [Reflect.ownKeys](https://tc39.es/ecma262/multipage/reflection.html#sec-reflect.ownkeys).
