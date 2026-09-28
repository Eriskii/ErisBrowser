# DOM string conversion and operation receivers

The supported DOM methods now forward their actual JavaScript receiver and
perform checked string conversion before their DOM operation. Methods saved from
one element can be applied to another valid receiver. Invalid receivers and
missing required arguments produce TypeError before invoking argument-conversion
hooks. Native method metadata has stable identity, standard short names and
lengths; these methods cannot be constructed.

The operation set covers document element/text/fragment creation, supported
queries, attribute get/has/set/remove, append/appendChild/removeChild/remove,
cloneNode, and the existing class-list add/remove/contains/toggle methods.
Canonical native entries use the existing guarded call/apply/bind bridge.
`createEvent` and other interface APIs retain their separate implementations.
This is not a complete generated Web IDL binding layer.

String conversion uses the runtime's string hint: live Symbol.toPrimitive,
toString/valueOf fallback, exact thrown values and ECMAScript numeric formatting.
Primitive and boxed Symbols reject when their conversion produces a Symbol.
Title and text/markup/value setters now use that conversion as well. Boolean
properties continue to use truthiness without running string hooks. Nullable
textContent maps null/undefined to empty; the supported legacy-null string setters
map null to empty while keeping explicit undefined distinct.

All variadic append arguments convert before any of those arguments is attached
or any corresponding text node is created. Conversion callbacks can themselves
mutate the live DOM; those author effects survive an eventual exception.
setAttribute converts both arguments in order before its write. Title lookup and
class-list reads happen after conversion, so callbacks cannot leave these methods
writing a stale snapshot. Full DOM fragment/hierarchy semantics remain separate
work; this does not promise atomic mutation for every DOM or resource failure.

Class-list argument conversion precedes token validation. add/remove/toggle use
ASCII whitespace and throw DOMException SyntaxError/InvalidCharacterError for
invalid tokens. Unicode spaces outside that ASCII set remain part of a token.
Stored tokens are deduplicated in insertion order; length and operations read the
current attribute. Forced toggle outcomes that make no change preserve the raw
attribute, including whitespace. Explicit undefined for the optional force
argument behaves as missing. An empty token set does not create an absent class
attribute.

## Evidence

The [comparison](dom-string-conversion.json) retains the frozen Symbol-checkpoint
probes and their final outcomes, plus every existing upstream profile comparison.
The nine original self-authored DOM probe sources are retained in
[the fixture](../fixtures/dom-string-conversion.tsv) and execute in both modes.
Four separately recorded global-reflection probes remain unsupported; they are
not removed from the comparison.

Six Rust groups cover the original probes, receiver/argument checks and method
identity, reentrant/abrupt effects, token conversion/validation, nullable/Boolean
behavior, and allocation/work/callback limits. Two older internal native-call
fixtures now use the canonical method identifiers; their budgets, input trees,
operations and no-mutation assertions remain unchanged.

These are focused implementation checks, not an upstream DOM or Web IDL pass rate.
The broad HTML/Test262 inventories provide regression evidence but do not certify
these DOM operation semantics.

## Resource handling and remaining gaps

String scans and worst-case UTF-8 storage are charged before decoding. Staged
variadic argument lists, live class-token parsing/deduplication, vector growth,
serialized attributes, query scratch and native method handles use the existing
shared work/storage ledger. Recursive or looping conversion hooks terminate under
uncatchable host limits and release the guarded execution state. Quotas remain
unchanged; the ledger is still estimated allocation accounting.

DOM storage still uses UTF-8 and replaces lone UTF-16 surrogates at that boundary.
Complete DOMString storage, interface prototype trees and descriptors, all receiver
exposure rules, XML element/attribute name validation, customized element options,
Trusted Types, live collections, full query behavior, fragment/hierarchy exceptions
and the rest of DOMTokenList/Web IDL remain incomplete. Diagnostic console output
still uses display formatting independently of DOM string conversion.

Primary algorithms: [Web IDL DOMString](https://webidl.spec.whatwg.org/#es-DOMString),
[nullable types](https://webidl.spec.whatwg.org/#es-nullable-type),
[argument conversion](https://webidl.spec.whatwg.org/#dfn-overload-resolution-algorithm),
[DOM textContent](https://dom.spec.whatwg.org/#dom-node-textcontent),
[DOMTokenList](https://dom.spec.whatwg.org/#interface-domtokenlist), and
[ParentNode.append](https://dom.spec.whatwg.org/#dom-parentnode-append).
