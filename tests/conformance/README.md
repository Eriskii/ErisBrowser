# Upstream HTML tree comparisons

The complete set of 62 `.dat` files from WPT's `html/syntax/parsing/resources` directory is vendored under `../upstream/wpt-html`, pinned to commit `f085a1efc1f58fbe263d384b1e335d656fe58e66`. The import manifest records every file's upstream path, size and SHA-256. The upstream license and format documentation are included. No upstream parser implementation is used.

Build the adapter and measure the entire inventory:

```sh
cargo build --locked --release --bin eris-dom
python3 tools/html_conformance.py
```

The adapter reads HTML on stdin and emits the actual DOM in the upstream format. It preserves comments, doctypes, processing instructions, attribute values and multiline text; it does not remove unsupported nodes to obtain a match. Attribute names are sorted by UTF-16 code units as required by the fixture format. The corpus runner validates all imported byte hashes before running anything. It never executes downloaded JavaScript or fetches resources from test HTML. The Python runner uses POSIX nonblocking pipes with time and output limits; it is currently validated on Linux.

There are 1,959 source cases. Cases without an explicit scripting flag run in both flag modes, giving 3,876 mode cases. The flag controls parsing, especially `noscript`; it does not enable script execution in this adapter. Fragment contexts and the `scripted_*` cases are recorded as unsupported. No entries are silently excluded from the inventory or denominator.

Results are **exact DOM tree comparisons**, not a claim to pass the corresponding complete WPT tests. Parse-error counts, document compatibility/quirks mode, encoding detection, fragment parsing, synchronous script execution, URL/document.write wrappers, and full WPT testharness behavior are not exercised. Missing namespace/template DOM representation produces mismatches. The JSON report lists every case and the full expected/actual trees for mismatches.

The command without a baseline fails if any mode case is unmatched, unsupported, or erroneous. A baseline run checks that previously matching trees remain matched while continuing to report the complete failure inventory:

```sh
python3 tools/html_conformance.py --baseline tests/conformance/html-tree-current.json
```

`--record-baseline PATH` explicitly writes a new status/hash inventory; it must not be used automatically to hide regressions. Recording and checking a baseline are mutually exclusive, and a run with process errors cannot record one. A baseline binds the corpus manifest and each case's source, expected tree, context and flag mode. A baseline run fails on these identities changing, inventory changes, process errors, or previously matched cases that no longer match. Newly matching trees are listed separately. `html-tree-initial.json` preserves the first measurement against the initial parser: 380 matched, 1,359 mismatched, 2,137 unsupported. That parser did not expose a scripting-flag API, so enabled-mode cases were explicitly unsupported. Compare disabled-mode results separately when assessing algorithmic improvement.

After the formatting-recovery increment, the current baseline records 2,446 exact matches, 1,012 mismatches and 418 unsupported mode cases. The prior baseline gate passed with 194 newly matching trees and no regressions before this update was recorded. Disabled-mode matches are 1,233 and enabled-mode matches are 1,213.

Regenerate the pinned source data only deliberately:

```sh
python3 tools/import_html_tests.py --revision f085a1efc1f58fbe263d384b1e335d656fe58e66
python3 -m unittest discover -s tools -p 'test_*.py'
```

The importer downloads test data from [Web Platform Tests](https://github.com/web-platform-tests/wpt/tree/f085a1efc1f58fbe263d384b1e335d656fe58e66/html/syntax/parsing/resources) over verified HTTPS. A new upstream revision requires a reviewed baseline update.
