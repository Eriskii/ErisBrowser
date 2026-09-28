# Upstream HTML tree comparisons

The complete set of 62 `.dat` files from WPT's `html/syntax/parsing/resources` directory is vendored under `../upstream/wpt-html`, pinned to commit `f085a1efc1f58fbe263d384b1e335d656fe58e66`. The import manifest records every file's upstream path, size and SHA-256. The upstream license and format documentation are included. No upstream parser implementation is used.

Build the adapter and measure the entire inventory:

```sh
cargo build --locked --release --bin eris-dom
python3 tools/html_conformance.py
```

The adapter reads HTML on stdin and emits the actual DOM in the upstream format. It preserves comments, doctypes, processing instructions, SVG/MathML element and attribute namespaces, attribute values and multiline text. Template `content` lines represent actual separately hosted document fragments in the DOM arena. The adapter does not remove unsupported nodes to obtain a match. Attribute names are sorted by UTF-16 code units as required by the fixture format. The corpus runner validates all imported byte hashes before running anything. It hashes the adapter before and after execution and refuses to publish results or write a baseline if those hashes differ. It never executes downloaded JavaScript or fetches resources from test HTML. The Python runner uses POSIX nonblocking pipes with time and output limits; it is currently validated on Linux.

There are 1,959 source cases. Cases without an explicit scripting flag run in both flag modes, giving 3,876 mode cases. The flag controls parsing, especially `noscript`; it does not enable script execution in this adapter. Of these, 412 mode cases specify a fragment context. The adapter invokes context-sensitive fragment parsing for HTML (including `template`), SVG and MathML contexts; six `scripted_*` cases remain explicitly unsupported. All 418 previously unsupported cases remain in the inventory as matched, mismatched or unsupported outcomes. No entries are silently excluded from the inventory or denominator.

The `--fragment CONTEXT` adapter option accepts an HTML local name, `svg NAME`, or `math NAME`. For example, `printf '<td>x' | target/release/eris-dom --fragment table` produces an implied `tbody` and `tr` around the cell. Following the [WHATWG fragment-parsing rules](https://html.spec.whatwg.org/multipage/parsing.html#parsing-html-fragments), parsing initializes a synthetic HTML stack root, context-dependent tokenizer and insertion mode, the adjusted current node for foreign content, and the form pointer from context ancestry; it does not run the full-document algorithm and discard wrappers. The library method `Document::parse_fragment(context_node_id, source)` returns a separate arena whose root owns the fragment children, leaving the original document unchanged. Its tokenizer preserves the leading newline in a textarea context and does not recognize a context end tag that the tokenizer has never opened. The document retains no-quirks, limited-quirks or quirks mode from its doctype, and library fragment parsing inherits its context owner's mode. Elements inside inert template contents use the contents owner's no-quirks mode. The CLI creates its fragment context in a no-quirks document.

Ordinary HTML templates now use inert fragment trees with host links, a template insertion-mode stack, table/foster-parenting recovery, nested form handling, and foreign-content integration. Document queries do not cross into template contents; scoped fragment queries can. Serialization visits the actual content fragment, and fragment insertion transfers its children. Mutation and IPC validation include host links when checking cycles and depth. Declarative shadow roots, content patching and synchronous parser script execution are not implemented by this parser increment.

Results are **exact DOM tree comparisons**, not a claim to pass the corresponding complete WPT tests. Parse-error counts, direct document compatibility-mode assertions, byte-encoding detection, synchronous script execution, URL/document.write wrappers, and full WPT testharness behavior are not exercised. Compatibility-mode classification and inheritance have separate DOM unit tests. The implemented foreign-content subset includes SVG/MathML namespaces, HTML integration points, foreign tag and attribute case adjustments, namespaced attributes, and HTML breakout handling. Other tokenizer and tree-construction behavior still produces mismatches. These parser results do not establish complete SVG rendering or MathML mathematical layout support. The JSON report lists every case and the full expected/actual trees for mismatches.

The command without a baseline fails if any mode case is unmatched, unsupported, or erroneous. A baseline run checks that previously matching trees remain matched while continuing to report the complete failure inventory:

```sh
python3 tools/html_conformance.py --baseline tests/conformance/html-tree-current.json
```

`--record-baseline PATH` explicitly writes a new status/hash inventory; it must not be used automatically to hide regressions. Recording and checking a baseline are mutually exclusive, and a run with process errors cannot record one. A baseline binds the corpus manifest and each case's source, expected tree, context and flag mode. A baseline run fails on these identities changing, inventory changes, process errors, or previously matched cases that no longer match. Newly matching trees are listed separately. `html-tree-initial.json` preserves the first measurement against the initial parser: 380 matched, 1,359 mismatched, 2,137 unsupported. That parser did not expose a scripting-flag API, so enabled-mode cases were explicitly unsupported. Compare disabled-mode results separately when assessing algorithmic improvement.

After the template and document-mode increment, the current baseline records **3,716 exact matches**, 154 mismatches and six unsupported mode cases. The release-adapter gate preserved all 3,272 previous matches and recovered **444 trees without regressions**. All 412 fragment cases and all 254 `template.dat` mode cases match. Disabled-mode matches are 1,868 and enabled-mode matches are 1,848, an increase of 222 in each unchanged flag-mode inventory. This increment adds ordinary template trees and insertion modes, retained doctype modes, and associated frameset/form recovery; it does not enable another scripting flag mode or remove failures from the inventory.

The fragment-parsing baseline recorded 3,272 exact matches, 590 mismatches and 14 unsupported mode cases. Its release-adapter gate preserved all 2,856 previous matches and recovered 416 trees without regressions: 404 newly supported fragment cases and 12 document cases improved by void-element handling fixes. All 404 then-executed fragment cases matched exactly; eight template fragment cases remained unsupported. Disabled-mode matches were 1,646 and enabled-mode matches were 1,626.

The foreign-content namespace baseline recorded 2,856 exact matches, 602 mismatches and 418 unsupported mode cases, an improvement of 410 trees without regressions. Disabled-mode matches were 1,438 and enabled-mode matches were 1,418. The preceding formatting-recovery baseline recorded 2,446 matches (1,233 disabled and 1,213 enabled), itself an improvement of 194 trees without regressions.

Regenerate the pinned source data only deliberately:

```sh
python3 tools/import_html_tests.py --revision f085a1efc1f58fbe263d384b1e335d656fe58e66
python3 -m unittest discover -s tools -p 'test_*.py'
```

The importer downloads test data from [Web Platform Tests](https://github.com/web-platform-tests/wpt/tree/f085a1efc1f58fbe263d384b1e335d656fe58e66/html/syntax/parsing/resources) over verified HTTPS. A new upstream revision requires a reviewed baseline update.
