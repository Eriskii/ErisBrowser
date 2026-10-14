# Shared metadata keys during DOM startup

The ten ProcessingInstruction and CharacterData function metadata bags reuse
`length` and `name` keys from the existing DOM bootstrap pool. Each bag and
callable keeps its own identity, descriptors and property order. The shared
UTF-16 key payloads are immutable; changing one function's metadata does not
change another function.

The change removes twenty repeated key allocations. It preserves the existing
property insertion, bag, display-name and native-callable charges, adds paid
handle clones, and keeps the runtime's work and heap limits unchanged. The
verified reduction is 332 charged work units and 1,680 cumulative charged bytes.
These accounting units do not measure processor instructions or total allocator
usage, and this is not a browser performance comparison.

The raw bootstrap boundary falls from 99,207 to 98,875 work units,
leaving 1,125 of the existing 100,000-unit allowance. Author execution still
receives its normal allowance after successful initialization. This makes room
for subsequent Promise installation; this checkpoint does not expose Promise.

## Validation

All four new groups pass. On the unchanged parent, the public descriptor control
passed while the backing-sharing and reduced-bootstrap-budget groups failed as
predicted; the private factory group required the new API and was not run there.
The complete Rust 1.88 suites pass 3,107 native and 2,803 default tests with none
ignored. Strict Clippy passes on Rust 1.88 and 1.98, formatting is clean, and all
102 rendering references pass with 204 PNGs unchanged from the throw-diagnostics
checkpoint. The [evidence summary](evidence/pi-key-cache/summary.json) binds the
769 source inputs and retained checks.
The frozen controls cover all ten public metadata descriptors and function
identities, actual sharing of the UTF-16 backing storage, raw bootstrap limits,
and exact/one-short factory work and storage admission. Two historical tests
have explicit accounting migrations; their other assertions remain unchanged.

During source review, two newly authored control literals used incorrect
CharacterData method lengths. They were corrected to the existing production
values (`deleteData.length === 2`, `replaceData.length === 3`) before baseline
execution. The original hold and additive correction remain in the evidence.
