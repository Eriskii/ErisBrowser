# Pinned Date timezone fixtures

These five fat TZif2 files were compiled from the unchanged official [IANA 2026d release](https://www.iana.org/time-zones/releases/2026d), released September 11, 2026. They are test inputs, not a bundled runtime timezone database. Runtime host timezone discovery is a separate feature.

The original `tzdata2026d.tar.gz` and `tzcode2026d.tar.gz` archives are retained under `upstream/`. [manifest.json](manifest.json) records their exact URLs, byte counts and SHA-256 hashes, selected archive members, unchanged license, source-built zic version and compiler provenance. Downloads used HTTPS; no PGP signature verification is claimed. The release's `zic.c`, `private.h` and `tzfile.h` were compiled with a C17 compiler. No host `/etc/localtime` or installed zone database supplies these fixture bytes.

| Fixture | Independent boundary coverage |
| --- | --- |
| `America/New_York` | Historical second offset; ordinary spring gap and autumn fold |
| `Australia/Lord_Howe` | Thirty-minute gap and fold |
| `Pacific/Apia` | December 2011 skipped calendar day |
| `Europe/Dublin` | Negative daylight saving in the unchanged vanguard rules |
| `Asia/Kathmandu` | Historical second offset and fifteen-minute change |

[expectations.json](expectations.json) retains ten exact transition instants and before/after offsets, derived from the pinned source rules and integer Gregorian calendar arithmetic. The verifier checks them using a separate March-era formula. Eris outcomes, another browser, and host timezone APIs were not used to choose expectations. Historical data remain IANA's reconstruction, not a claim of complete civil-time history.

Verification is offline and does not invoke a compiler:

```sh
python3 tools/generate_date_zones.py --check
python3 -m unittest discover -s tools -p test_generate_date_zones.py
```

An optional bounded rebuild compiles the retained zic sources in a temporary directory, processes the four unchanged regional files, and requires byte-for-byte agreement for all five outputs:

```sh
python3 tools/generate_date_zones.py --rebuild
```

The rebuild needs a POSIX host and a C17 compiler (`--compiler` selects its executable). It does not install anything. Compiler executable hashes may differ across systems; the required reproducible result is the TZif bytes. Source extraction rejects links, duplicate or unsafe member names, oversized archives and excessive members. Compilation uses a finite deadline and process-group cleanup. The files use no leap-second input, range truncation or rearguard conversion; POSIX footer rules are preserved.
