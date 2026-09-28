# Window properties and global bindings

Window now supports ordinary string-keyed data and accessor definitions beyond
the previously special-cased global names. Bare global reads and writes consult
the same current property records as member access. Getters and setters receive
Window; calling a function through an identifier still uses the identifier call
receiver. Lexical bindings remain separate and take precedence.

Resolution includes inherited properties without invoking getters during the
lookup itself. Writes observe deletion, replacement with an accessor, readonly
transitions and inherited setters that occur during right-hand evaluation.
Strict assignment to a captured global that was deleted throws ReferenceError;
a sloppy assignment can recreate it. Descriptor flags survive value writes.

The execution receiver is a private environment slot. Ordinary functions set
that slot, arrows inherit it, and the root retains the realm's Window. Author
properties named `this`, including accessors and their deletion, cannot replace
that receiver. Replacing `globalThis` likewise preserves Window/event identity.

UTF-8 global names remain in their existing binding records. Lone-surrogate
UTF-16 keys have separate property records because no identifier can name them.
They retain exact code-unit identity, descriptors, deletion behavior and creation
serials. Enumeration merges both string stores in creation order after numeric
indices; symbols follow the string keys. No replacement-character alias is used.

Global function declarations validate property compatibility for every name
before publishing declarations. The last duplicate declaration supplies the
function; those selected functions are created in their source order before new
`var` properties. Existing configurable properties become standard function
bindings; nonconfigurable writable/enumerable data properties retain their flags.
Cross-script lexical conflicts precede function-property compatibility checks.
Global `var` declarations reuse existing own records and create own properties
when a name previously existed only on a prototype.

## Evidence

The [comparison](window-global-bindings.json) preserves **64 frozen self-authored
modes**, including the preceding reflection inventory: 32 previously passed,
24 were unsupported and eight failed. All 64 now pass with identical sources,
language modes and case fingerprints. The original fixtures remain in
[window-reflection.tsv](../fixtures/window-reflection.tsv) and
[window-global-bindings.tsv](../fixtures/window-global-bindings.tsv).

Seven new Rust groups cover these probes, private receivers/defaults/arrows/bind/
constructors/events, accessor reentrancy and thrown identity, UTF-16 order and
flags, cross-script declaration validation, var/prototype handling, and shared
resource limits. Three older groups initially expected general Window definitions
to be unsupported; their original operations now assert the resulting properties.
Event-handler descriptor/deletion gaps retain explicit unsupported checks.

The full run passes **988 Rust tests** (806 library, none ignored), formatting,
strict all-target Clippy, release compilation, **155 Python checks**, **57 exact
pixel references**, and **15,000 deterministic mutation cases** with no caught
panic or invariant failure.

All **21 pinned Test262 profiles / 7,121 modes / 1,744 controls** preserve sources,
fingerprints and runner policies. **13 modes improve**: 11 compound-assignment
cases exercise getters that delete a captured global before its strict write;
two RegExp cases define throwing global properties before checking a detached
method call. Every other observation and every control is unchanged. No pass is
lost. The two healthy improved baselines retain their full inventories; all 17
healthy gates pass. Four resource-stopped inventories remain observations.
HTML remains 3,868 matched, two mismatched and six unsupported.

## Bounds and remaining behavior

Private receiver slots, declaration selection maps, new property records,
UTF-8/UTF-16 conversions and snapshots use the shared work/storage ledger.
Creation serials are checked before new properties are published. Callbacks share
existing invocation and execution limits; resource errors remain uncatchable and
unwind interpreter frames. No quota increased. Allocation accounting is still an
estimate, and no independent-agent review or security audit is claimed.

Window event-handler descriptor reflection, definition and deletion are explicitly
unsupported. Full Window interfaces, WindowProxy, cross-origin/realm handling,
frames/named properties, extensibility and complete Web IDL behavior remain
unfinished. Other DOM hosts still need exact DOMString storage. This change does
not complete ECMAScript, the web platform, production security, Vulkan browser
integration, or the requested Chromium performance comparison.

Primary algorithms: [Object Environment Records](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-object-environment-records),
[Global Environment Records](https://tc39.es/ecma262/multipage/executable-code-and-execution-contexts.html#sec-global-environment-records),
[GlobalDeclarationInstantiation](https://tc39.es/ecma262/multipage/ecmascript-language-scripts-and-modules.html#sec-globaldeclarationinstantiation),
and [own property key order](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-ordinaryownpropertykeys).
