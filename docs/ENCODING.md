# Document and resource encoding

Fetched HTML selects its encoding in this order: byte order mark, valid transport
charset, then a 1,024-byte prescan for HTML meta declarations or an XML declaration
fallback. Without a recognized declaration this implementation uses windows-1252.
The prescan follows byte-level attribute/comment handling; it is intentionally
different from tokenizing an HTML document. Transport parameters honor quoted
semicolons, escapes, invalid parameter syntax and duplicate parameter order.

An accepted meta declaration during actual tree construction can replace a
tentative encoding. The loader reparses the original response bytes at most once,
before fetching subresources or executing author scripts. BOM/transport selections
and UTF-16 selections are retained. The network request is never repeated, including
for POST navigation. HTML meta tokens processed inside template contents still
participate in encoding selection; string-fragment parsing and foreign meta
elements do not. HTML meta UTF-16 labels become
UTF-8; x-user-defined becomes windows-1252. Character references are interpreted
during actual meta processing, while the byte prescan sees their literal bytes.

External CSS uses BOM, transport charset, an exact initial ASCII
`@charset "label";` sequence within 1,024 bytes, then the document encoding. Classic
scripts use BOM, transport charset, their `charset` attribute, then the document
encoding. Decoded script cache entries include the fallback encoding, so two script
elements with different charset attributes do not incorrectly share decoded text.
Modules are not implemented and do not use this classic-script path.

Documents retain the canonical encoding name through native IPC. JavaScript
`document.characterSet`, `document.charset` and `document.inputEncoding` expose it.
`Document::parse`, `Page::from_html` and other string-based DOM APIs already receive
Unicode and default to UTF-8; a meta element does not reinterpret their strings.
The `encoding_rs` dependency provides the Encoding Standard codec tables and
replacement behavior. The encoding-selection algorithms are implemented here.

Tests exercise precedence, quoted transport syntax, comments/attributes, prescan
bounds, late and character-reference declarations, inert templates, CSS/script
fallbacks, readonly document metadata and IPC. A loopback regression loads a
Shift_JIS page with external CSS and JavaScript, checks resulting pixels/text,
and verifies that its original POST is sent exactly once.

This is not full encoding or loading conformance. There is no locale/user encoding
override, navigation-history encoding preference, streaming parser restart,
incremental parser-script execution, full XML parser, or exhaustive upstream
encoding test runner. The parser currently consumes complete fetched bodies;
that limits behavior around scripts which mutate encoding declarations while a
document is still being parsed. Generic non-HTML text resource decoding uses BOM,
transport charset and UTF-8 fallback.

Normative references: [HTML encoding selection and changes](https://html.spec.whatwg.org/multipage/parsing.html#determining-the-character-encoding),
[MIME parameter parsing](https://mimesniff.spec.whatwg.org/#parse-a-mime-type),
[CSS input byte decoding](https://drafts.csswg.org/css-syntax/#input-byte-stream),
and the [Encoding Standard](https://encoding.spec.whatwg.org/).
