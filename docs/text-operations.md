# Exact Text splitting and adjacent reads

Represented Text nodes now expose ordinary `Text.prototype.splitText` and
`wholeText` properties. Splitting counts UTF-16 units and creates a distinct
Text, including at zero, at the end, and for empty input. An attached result is
inserted immediately after the original. A detached original produces a detached
result. The original retains its identity; the returned node uses the default
Text prototype.

`wholeText` returns the exact concatenation of adjacent ordinary Text siblings,
including empty Texts. Comments, processing instructions and elements stop the
run. Descendant text is not included. Ordinary template children and the separate
`template.content` fragment have independent sibling lists. A surrogate pair can
span two nodes in the returned string without rewriting either node's storage.

## Conversion and ordinary properties

The getter and method have distinct cached function objects. The getter is named
`get wholeText`, with length zero and no setter. The method is named `splitText`,
with length one. Both prototype properties are enumerable and configurable;
`splitText` is writable. Their ordinary property bags, saved calls, replacements,
deletion and strict readonly behavior are tested. Changing a prototype does not
confer the authentic Text receiver brand.

`splitText` checks its receiver before the required offset argument and before
Number-hint author conversion. Missing offset is a TypeError; explicit undefined
converts to zero. Web IDL unsigned-long conversion wraps modulo 2^32. Data, length
and parent are read after conversion callbacks. An offset beyond the current
length throws an ordinary IndexSizeError with code one and creates no suffix.

## Resource admission and retained effects

The operation performs separate admitted stages:

1. Count the current units, plan the exact suffix, admit its storage and a new
   node, then create it detached.
2. Validate the local parent list and bounded ancestor path, admit any child
   vector growth and shifts, then insert the suffix.
3. Plan and admit the original's exact prefix, then replace its data.

Quota refusal can retain a newly created detached suffix, or an inserted suffix
while the original still has its full data. The operation does not promise
transactional rollback. Conversion callback effects also remain. Private tests
exercise every insufficient work prefix and exact/one-short work and heap limits.

New suffix admission accounts for the full original plus the new payload before
later truncation. Canonical storage can change from UTF-8 to UTF-16 units or back.
For example, splitting `A🚀B` inside its pair temporarily retains ten payload
bytes, then finishes with eight across two nonscalar nodes. No future truncation
credit is borrowed. Node and child vector growth pay their actual requested
storage; no DOM or runtime cap is raised.

`wholeText` discovers one local sibling run, counts its exact units and pays for
a second read/copy pass. Its temporary Vec and final JavaScript Rc string have
separate cumulative charges. It uses the existing 262,144-unit JavaScript string
limit. Both success and failure copy the DOM helper's absolute budget counters
back to the runtime. Reads leave nodes, links and payloads unchanged.

Internal insertion accepts coherent existing Document-parent Text admitted by
the Rust snapshot/host interfaces, and preserves the accepted 256-edge depth
limit. It is a local fresh-Text helper, not a global validator for arbitrary
public Rust graph mutations. Text insertion leaves frozen base metadata, summary
NodeIds and named-disclosure groups unchanged.

## Initialization

The two cached function bags add 386 charged work units and 6,335 charged bytes.
Measured raw bootstrap leaves 10,636 work units and charges 1,741,271 bytes, with
681 objects/capacity, 321 native entries and 25 legacy prototype entries. Only
three descriptive literals change after measurement. The 100,000-unit bootstrap
and author work limits, cumulative 8 MiB heap and reset sites remain unchanged.

## Browser and compatibility evidence

Rust 1.88 passes **1,733 default tests** and Rust 1.98 passes **1,856 native
Vulkan tests**, including ignored confinement checks. Formatting, strict native
and presenter Clippy, and the release build pass. There are 45 new test groups:
15 DOM, 28 runtime, and the two browser/worker integrations. Passing focused
commands contain 149 observations covering 143 distinct groups.

The initial all-target compile and all 43 new DOM/runtime groups passed. Before
updating measured counts, the bootstrap filter recorded 24 passes/two stale
assertion failures and the identity filter 20 passes/one repeated stale failure.
The original sources, failures, correction and successful reruns are retained.
Production and the independent fixture stayed byte-identical after source review.

The release comparison preserves all **4,176 established case records and 432
controls**, including their existing failures and unsupported outcomes. All 28
new independent strict/sloppy Text modes pass, up from zero on the prior release;
all 12 controls are healthy, up from four. Positive/wrong controls require both
the expected exception and a healthy same-mode positive partner. No input drift,
removed cases or other record changes occurred. These selected records are not a
platform-wide conformance rate.

The [summary](evidence/text-operations.json) and
[archive](evidence/text-operations.tar.gz) bind held source, fee ledgers, all
commands/logs, before/after reports, independent controls and review receipts.
The preceding [Node constant checkpoint CI receipt](evidence/node-constants-ci.json)
records all nine GitHub jobs passing for `774c275`.

The Page and confined-worker witness loads once, then dispatches one real click.
Seven retained Text pairs cover scalar and nonscalar visible content, title,
stylesheet, clean textarea, template content and ordinary template children.
A second split before a summary changes its child index while preserving its
NodeId. Page explicitly primes that cache before the click; the worker witness
checks the decoded snapshot's cache and makes no original-worker cache claim.
Title metadata, exact node identities/units, links, fixed base URL and literal
style pixels are checked in both phases.

The pixel reference uses two separate scalar Text nodes containing replacement
glyphs for the split surrogate halves. This matches current per-node projection;
it does not establish cross-node shaping equivalence. Nonempty glyph bands and
literal green/blue pixels keep the comparison nonvacuous. Existing exact ERWA
transport needs no protocol change.

This increment does not add CDATASection, XMLDocument, live Range adjustments,
MutationObserver records, slotting or custom-element reactions. A later [Node normalization increment](node-normalize.md) handles represented
ordinary Text descendant runs. Legacy exact writers, dirty textarea semantics
and full DOM coverage remain unfinished. No page, GPU, resident-memory or Chromium performance result is
claimed. Full compatibility and production security remain unmet goals.

Normative references: [Text](https://dom.spec.whatwg.org/#interface-text),
[split a Text node](https://dom.spec.whatwg.org/#concept-text-split),
[contiguous Text nodes](https://dom.spec.whatwg.org/#contiguous-text-nodes),
[Web IDL unsigned long](https://webidl.spec.whatwg.org/#es-unsigned-long), and
[interface prototype installation](https://webidl.spec.whatwg.org/#es-interface-prototype-object).
