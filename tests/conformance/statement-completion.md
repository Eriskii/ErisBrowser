# Statement completion values

The interpreter distinguishes an empty completion from JavaScript `undefined`.
Empty statements, declarations and empty blocks preserve the preceding result:
`7; var a; {}` completes with 7. An expression that produces undefined replaces
that result: `7; undefined; {}` completes with undefined. The public Runtime API
maps a wholly empty script result to undefined.

Statement lists fill empty normal, break and continue completions with their
preceding value. Labels preserve the value of the matching break. If statements
fill empty branch results with undefined. While, do, for and for-in loops retain
their last body value, including matching continues; departing jumps preserve
their own value or take the loop's previous value. Conditions, initializers and
updates do not supply the loop result. Switch preserves values across fallthrough
and consumes only untargeted breaks. Try/catch retains the selected result;
normal finally results are ignored, while abrupt finally results replace it.

Return and throw values retain exact identity. A function's normal body completion
still produces an implicit undefined return. Host resource/unsupported termination
remains uncatchable and skips author cleanup. Value propagation uses shallow
copies and adds no content scans or allocation. Existing execution, allocation,
source, parser and runtime nesting ceilings are unchanged.

Nine [Rust regression groups](../script_completion.rs) use the public interpreter
API. The [120-source table](../fixtures/script-completion.tsv) runs in both strict
and sloppy mode, covering empty/undefined boundaries, all four supported loops,
switches, labels, try/catch/finally and returns, including negative zero and lone
UTF-16 surrogates. Additional checks preserve object/array/function/boxed-number
identity and NaN, reject attempts to override host termination, and execute nine
recursive statement forms until the existing resource guard stops them.

The [recorded comparison](statement-completion.json) moves from **118/240** to
**240/240** expected scalar results, with 122 gains and no losses. The before
probe links to the actual release library from `f4c287c`; the after probe uses
the final frozen source. Cargo JSON identifies each library artifact. The report
includes the probe source, protocol and source/library/executable hashes. Both
probes use Eris directly, without eval or another JavaScript engine. These are
root-authored, specification-guided expectations, not upstream conformance tests.

All **20 existing Test262 profiles / 6,879 modes / 1,680 controls** retain every
case, fingerprint, policy and observation. All controls verify. Fifteen healthy
baseline gates pass; resource-stopped profiles remain observations without new
healthy baselines. No profile or supported-feature filter changes. In particular,
unchanged upstream completion cases that invoke eval remain unsupported.

An initial implementation failed the existing native-stack regression. Extracting
declaration setup so its frame ends before nested execution, plus four runtime
branch helpers, corrected it. Local debug dispatcher size fell from 15,400 to
7,000 bytes, and statement-list size from 7,800 to 1,352 bytes. These measurements
compare the initial and final candidates in this increment, not the prior
checkpoint. No stack size or quota was increased. The original regression and
the nine new recursive forms pass. Frame sizes are compiler-specific evidence,
not a portable stack proof; native recursion remains architectural work.

Full validation records **907 Rust tests**, **151 Python checks**, **57 exact
pixel references** and **15,000 deterministic mutation cases** passing. Formatting,
strict all-target Clippy and release compilation pass. No native-window, Vulkan
or Chromium performance result is claimed for this change. Full compatibility
and production security remain unverified.

Run the focused checks with:

```sh
cargo test --locked --test script_completion
```

Primary algorithms:
[UpdateEmpty](https://tc39.es/ecma262/multipage/ecmascript-data-types-and-values.html#sec-updateempty),
[statement lists](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-block-runtime-semantics-evaluation),
[if](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-if-statement-runtime-semantics-evaluation),
[iteration](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-iteration-statements),
[switch](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-switch-statement),
[try](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-try-statement-runtime-semantics-evaluation).
