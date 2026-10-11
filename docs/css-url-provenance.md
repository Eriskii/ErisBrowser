# CSS URL provenance and bounded decoding

Stylesheet loading now retains the URL context needed by future URL-valued CSS
properties. External sheets keep their final response URL through redirects,
imports, source segmentation and rule parsing. Inline sheets keep a marker that
can resolve against the current document base. A separate pure utility decodes
and resolves bounded CSS `url()` values.

This checkpoint does not accept URL background images, fetch resources from CSS
properties, or carry URL context into cascade candidates and property application.
The existing gradient backgrounds, resource policy and rendering paths remain.

## Source context

`css::SourceBase` has two variants:

| Variant | Meaning |
|---|---|
| `Document` | Use the current document base supplied when calling the resolver. |
| `Stylesheet(Arc<url::Url>)` | Use the retained external sheet's final response URL. |

`StyleSource` and parsed `Rule` expose a `base` field. `StyleSource::new` and the
existing public stylesheet parser wrappers default to `Document`.
`StyleSource::with_base` accepts an explicit context. Code constructing either
public struct directly must supply the new field.

The loader shares one final URL allocation per retained external sheet. Cached
sources, emitted segments and rules clone its handle. Each import uses its own
final response URL; containing layer and media metadata do not replace it.
Inline import expansion can therefore contain external and document-context
segments together. Existing Page recollection preserves those contexts, while
changed inline text invalidates its old expanded source without starting a fetch.

For example, a request for `/start.css` redirected to `/assets/v1/main.css`
retains that latter context. The utility resolves `url(Tile.PNG)` to
`/assets/v1/Tile.PNG`. A child redirected to `/cdn/theme/final.css` resolves its
relative values under `/cdn/theme/`. Inline values use the caller's current
document base. This follows the distinction in
[CSS relative URL rules](https://drafts.csswg.org/css-values-4/#relative-urls).

Custom properties retain authored URL text and casing. The future property
consumer must resolve substituted text in the consuming declaration's context;
it must not resolve it when the custom property is defined. A custom property
defined in `/a/style.css` and used by a declaration in `/b/style.css` therefore
uses the latter context. This is the
[CSS Variables rule](https://drafts.csswg.org/css-variables-1/#defining-variables).
No computed URL-valued property or its inheritance is implemented here; the
resolver's already-absolute result can be retained without resolving it again.

## Pure decoder and resolver

`css::url::decode(value, budget)` accepts one complete quoted or unquoted `url()`
value with bounded surrounding whitespace/comments. CSS escapes, escaped
function names, quoted newline continuations and replacement of invalid escaped
code points are handled without lowercasing URL text. Invalid syntax returns
`Ok(None)`; resource refusal returns `UrlError::Limit`. The utility intentionally
rejects unterminated strings, comments or functions instead of applying CSS
Syntax's EOF recovery. Bare strings, `src()`, URL modifiers and multiple values
are outside this utility's accepted syntax.

`resolve(decoded, source_base, current_document_base, budget)` returns:

| Result | Meaning |
|---|---|
| `Absolute(url::Url)` | Successfully parsed absolute URL, retaining any fragment. |
| `LocalFragment(String)` | Decoded text beginning with `#`, kept separate from a document request. |
| `Empty` | An empty decoded value, with no ordinary base parse. |
| `Unresolved(String)` | Decoded text that the URL parser could not resolve. |

Resolution uses the URL library's default UTF-8 processing, without the HTML URL
helper's legacy query-encoding override. This follows the
[CSS URL processing model](https://drafts.csswg.org/css-values-4/#url-processing).
The utility performs no scheme, origin, file or network authorization. Successful
parsing is not permission to fetch; callers must use the existing resource policy
when a future property actually requests a resource.

## Bounds and admission

The loader retains the existing decoded/expanded text, scan, import, recursion,
URL-work and URL-storage limits described in
[stylesheet loading](STYLESHEET_LOADING.md). Variable URL storage is admitted
before creating the shared handle; emitted segments copy no URL string bytes.
Fixed metadata is bounded by the existing 256-source and 10,000-rule limits.
The existing two-chunk 8 MiB text capacity is preserved.

`UrlBudget` carries cumulative `work` and `bytes` allowances across calls. Earlier
debits remain on rejection, and insufficient admission exhausts the corresponding
allowance. Raw CSS input is limited to 4,096 bytes; selected base and absolute
result URLs are limited to 16 MiB. The decoder reserves `3*L + size_of::<String>()`
bytes before its single output allocation, where `L` is raw input byte length.
Its two prepaid scan tariffs are `2*L+1` and `8*L+32` work units.

Resolution pays 8 dispatch units. Local fragments additionally reserve `4*T+8`
work and `T + size_of::<String>()` bytes. For ordinary URL parsing, `T` is decoded
text length and `B` is selected base length: the additional work tariff is
`32*(B+T+1) + 16*(T+1)^2`, with storage reservation
`64*(B+T+1) + 4096 + size_of::<url::Url>()`.
These are conservative admission tariffs derived from the pinned dependency
paths, not measured allocator or instruction bounds. The URL dependency uses
infallible allocation internally. Fixed input/result limits and resource policy
remain distinct from this reservation.

Literal boundary tests include 93 work units for decoding `url(A)` and 456 for
resolving decoded `a` against `https://x/`, with exact and one-short storage
admission. Separate cases exercise shared handles, retained rule limits and
unchanged text capacity.

## Validation record

The final source-2 full suites pass **2,593 native-feature tests on Rust 1.88** and
**2,395 default-feature tests on Rust 1.98**. Each has thirty result groups with
zero failed or ignored tests, using `--include-ignored` and four test threads.
The 25 new groups comprise ten decoder/resolver, seven CSS provenance, six loader
and two Page cases. The Page tests exercise actual local HTTP redirects/imports,
changing document bases, UTF-8 CSS URL resolution, stable rendering and the exact
request list; unsupported background URLs create no image requests.

The first strict Rust 1.98 check rejected a manual comment-prefix slice. The
retained correction replaces it with `strip_prefix`, preserving the same borrowed
suffix and byte offsets. The original failed receipt and both source versions
remain in the evidence. Strict all-target Clippy now passes for native features
on Rust 1.88 and 1.98 and default features on Rust 1.98. The format check passes.

The parent commit `181d1020` completed all nine CI jobs successfully; that record
is separate from this increment's local checks.

All **71 self-authored browser references** pass with zero differing pixels and
zero maximum channel error. Their manifest and test/reference HTML remain exact
to the parent checkpoint. The immutable browser is built from source 2, whose
inventory contains 535 inputs. The probe's browser-bridge suite passes **15 tests
across twelve result groups**, and its strict all-target Clippy check passes.
These are local regression checks; there is no new GPU execution, performance
result or complete CSS URL conformance claim.

The [evidence summary](evidence/css-url-provenance.json) and
[record archive](evidence/css-url-provenance.tar.gz) bind the selected source,
literal tests, immutable browser provenance, failed lint attempt, correction,
reviews and actual final results.
