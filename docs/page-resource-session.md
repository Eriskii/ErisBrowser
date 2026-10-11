# Retained Page resource sessions

Pages loaded through navigation now retain the original resource session after
initial loading. This preserves the Fetcher that loaded the document, successful
image cache, failed-resource keys, decoded-image debit and remaining URL work.
The initial resource loop still runs in DOM order before author scripts and
inline SVG refresh. No extra fetch is scheduled by retaining this state.

This is preparation for later image consumers. URL background images remain
unsupported. Layout stays read-only; changing `img.src`, a base URL, classes or
styles does not acquire another image. Click and task processing keep their
existing behavior, including previously written natural-size attributes.

## Ownership and authority

`PageResources` is private. Initial loading moves the original Fetcher and cache
allocations into it, then moves the session into Page before author callbacks.
It does not create a replacement Fetcher or copy decoded pixels. Dropping Page
releases its retained owners; any independent trusted host owner remains separate.
`from_html`, built-in pages and error pages retain no resource session and do not
gain network access.

In direct loading, the same Fetcher retains the document's request count, body
bytes and original start time. Native confined loading still uses its captured
bridge and the existing parent/broker accounting. The bridge bypasses the direct
Fetcher's local counters, so zero local counts do not mean zero broker requests.
The committed navigation URL continues to govern subresource authority; a changed
document base or stylesheet URL only affects reference resolution.

The existing opaque-image path supplies already decoded pixels and an origin
flag. Retaining an image does not decode its empty opaque response body again,
expose hidden response metadata, or clear cross-origin taint. This checkpoint
adds no bridge, worker, snapshot or public network capability.

## Cache identity and image memory

Successful image cache keys remain the requested absolute URL with its fragment
removed. Public image aliases retain the original `src` spelling. Different raw
aliases of one request share its raster and origin flag. Two distinct requested
URLs that redirect to the same final image remain separate entries and separate
decodes. Image-MIME navigation also preserves its existing second fetch for the
synthesized image element.

External decoded bytes remain a cumulative debit. Removing an alias neither
refunds that debit nor releases a raster still owned by the session cache. The
same applies when a trusted host clears the public ImageStore. Retaining the
Fetcher also keeps its existing direct HTTP agent alive until the page is dropped.

Inline SVG refresh first removes previous inline rasters. Its live-memory check
then counts the union of public aliases and retained cache values, once per Arc
allocation. Equal pixels in separate allocations count separately; cache-only
rasters still count. Replacing an inline SVG can reuse released live memory and
does not add to the external-image lifetime debit.

## Existing limits

No limit is increased by this change:

| Resource | Existing limit |
|---|---:|
| Fetches, including the document | 48 |
| Individual fetched/decompressed body | 8 MiB |
| Aggregate fetched page bodies | 32 MiB |
| Navigation fetch sequence | 30 seconds |
| Subresource URL-resolution work | 32 MiB |
| Image dimensions | 4,096 per axis |
| External decoded images / live page raster allowance | 64 MiB each |

Failed image and script keys retain their existing kind/request-URL distinction.
No new 48-entry negative-cache limit is imposed: the old DOM traversal and URL
work bounds still govern keys created after fetch admission is exhausted.
Codec scratch and other allocations are not fully described by retained pixel
counts; see the existing [security boundaries](SECURITY.md).

## Validation

The independent HTTP fixtures froze these expectations before execution; all
five groups passed:

- Document, CSS, script and two image aliases: four Fetcher calls, 620 response
  bytes and one eight-byte decoded raster.
- Duplicate 404 and malformed-image aliases: three calls and no decoded raster;
  later layout, click, tasks, base/style changes and `src` changes fetch nothing.
- Document plus 47 one-pixel images: 48 calls and 188 decoded bytes. Cached reuse
  adds no debit; the next uncached attempt is refused before reaching the server.
- Image-MIME navigation: two calls and one eight-byte decoded raster.
- Two requested URLs redirected to one foreign image: three Fetcher calls but
  five HTTP requests, two distinct cache entries and preserved false origin flags.

Separate private tests cover Fetcher move identity, cache/alias/SVG memory unions,
Page-drop release and the exact remaining space for a one-pixel SVG. A disposable
thread with a fake bridge checks captured transport identity and supplied opaque
pixels; that unit fixture is not an isolated-process security test.

All 13 new groups passed: six private ownership/accounting groups, five HTTP
groups and two bridge/no-I/O groups. The complete Rust 1.88 native suite passed
2,606 tests and the Rust 1.98 default suite passed 2,408, with no failed or ignored
tests and 30 result rows in each run. Strict Clippy passed for native Rust 1.88
and 1.98 and default Rust 1.98; formatting also passed.

The final browser, bound to the same 546-input source snapshot, passed all 71
unchanged self-authored render comparisons with zero differing pixels and zero
channel error. The published parent `63eb8ed1` also completed all nine CI jobs.
Every recorded command and test in this increment succeeded; no corrective
source revision was needed after execution.

The [evidence summary](evidence/page-resource-session.json) and
[retained source and observations](evidence/page-resource-session.tar.gz) bind
the fixtures, source, browser and raw receipts. No new probe, GPU, performance,
complete compatibility or production-security claim is made by this foundation.
