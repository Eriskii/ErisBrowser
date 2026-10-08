# Adversarial case design

| Case | Independent observable oracle |
| --- | --- |
| Metadata | Nine exact names/lengths; ordinary descriptors; genuine construct guard before rejecting method construction |
| Target validation | Six primitive targets across nine methods; poisoned key hook count remains zero |
| Get Receiver | Exact object/undefined/null/number identity returned by strict inherited getter; thrown object identity preserved |
| Key conversion | Literal K then KS trace; prototype mutation precedes fresh walk; Symbol key remains distinct |
| Presence/descriptor | Getter count zero; inherited presence but absent own descriptor; fresh descriptor and lone-surrogate/Symbol keys |
| Delete | True for absent/inherited/configurable; false for fixed; array length stays two with a hole |
| Set Receiver | Distinct Receiver own accessor is false/zero setter calls; target setter observes undefined and number |
| Set order | Literal TKVREPS: argument expressions and ignored extra argument precede string-hint key conversion and setter |
| Typed target/Receiver | Target invalid distinct Receiver true; invalid typed Receiver false; exact conversion counts zero/one/two |
| Fresh bounds | RHS grows/shrinks before validity; four key hooks shrink before get/has/descriptor/delete; transfer removes descriptor |
| Typed descriptors | Literal flags and value17; delete false; invalid -0 blocks poisoned prototype while ordinary01 traverses |
| Extensibility | Ordinary/fixed-buffer true; tracking and fixed-length resizable views false without losing extensibility |
| Prototype | Exact identities, cycle false, same nonextensible prototype true, different false, null accepted |
| Saved methods | Own replacement/deletion does not replace saved native identity; all saved property routes preserve Symbol identity |

The source plan deliberately avoids runtime quota predictions. Later private
tests should cut actual key conversion, descriptor materialization, prototype
walk and final write admissions while retaining already completed authored
callbacks. These private accounting tests belong with the implementation, not
the portable fixture population.

The five control pairs use fixed, distinct wrong observations: malformed UTF-16
replacement, Receiver defaulting, incorrectly calling the Receiver setter,
confusing TypedArray target with Receiver semantics, and incorrectly preventing
extensions on a resizable view. They cannot be satisfied by an absent method
because each pair begins with a successful positive operation prerequisite.
