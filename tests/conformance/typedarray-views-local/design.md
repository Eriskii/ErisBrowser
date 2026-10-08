# Independent oracle design

These bodies were written against the language algorithms and the selected
scope before its implementation. Literal outcomes do not come from a candidate
renderer, parser or JavaScript engine. All cases return true only after their
assertions; each runs in its own realm in both modes.

| Case | Distinguishing expected observation |
| --- | --- |
| metadata_alias_and_saved_identity | Subarray/join/toString lengths 2/1/0; ordinary writable, nonenumerable, configurable properties; exact Array alias; saved methods survive deletion. |
| all_ten_kinds_share_bytes_and_preserve_kind | Ten literal widths 1/1/1/2/2/4/4/2/4/8; subview offset one element, length two; writes visible both ways, no expando copy. |
| literal_clamped_indices_and_empty_offsets | Eight fixed offset/length pairs distinguish NaN, minus zero, infinities, relative indices, truncation and empty end-before-start slices. |
| invalid_receiver_precedes_argument_hooks | Primitive, ordinary, Array, DataView and forged-prototype receivers throw TypeError without invoking either index conversion. |
| callback_order_and_species_argument_values | Exact BECSN trace; original buffer, byte offset 1, length 3; Construct receives the selected species as new.target. |
| abrupt_conversion_and_species_identity | Begin/end/species/constructor abrupt identity propagates; trace proves later stages did not run. |
| intrinsic_default_survives_global_replacement | Undefined constructor and null species choose the saved original Uint8Array; null constructor and nonconstructor species throw. |
| species_accepts_short_different_kind_and_same_object | Float64 length one, Uint16 length zero, and the source itself are accepted despite a longer requested subarray; ordinary result is rejected. |
| species_result_bounds_are_refreshed_after_constructor | Returned fixed view invalidated during species call throws; restored bounds permit it; a newly detached result throws. |
| tracking_and_fixed_species_arity | Literal schedule `2:3:undefined;2:3:undefined;3:3:3;E;3:3:0;`, including explicit undefined versus an object converting to undefined. |
| tracking_subviews_follow_growth_and_fixed_subviews_do_not | Tracking lengths 4→8→0→4, fixed 2→2→0→2; restored bytes are zero after shrink/grow. |
| initially_out_of_bounds_sources_keep_stored_offset | Both sources initially have public offset zero; callback growth yields stored offset 2, tracking length 6 and fixed length zero. |
| captured_source_length_and_fresh_default_bounds | Growth during start still clamps end to old length 4; default construction after shrink throws RangeError; an unrelated valid result survives source detachment. |
| detached_source_can_reach_custom_species | Start conversion and species still run (BN); species sees detached original buffer, stored offset 2 and length zero. |
| bound_species_is_constructed_with_original_target | Bound custom constructor sees its original unbound target in new.target and receives the shared buffer arguments once. |
| join_all_kinds_and_literal_number_strings | All ten kinds produce `1,2,3`, `1/2/3`, `123`; Float64 text includes NaN, signed-zero normalization, infinities and the held shortest-decimal tie. |
| join_validates_receiver_before_separator | Initially detached/OOB and non-TypedArray receivers throw before separator coercion. |
| join_separator_coercion_once_even_when_empty | String-hint callback once per call, including empty view; explicit null differs from undefined; Symbol and authored errors propagate. |
| join_captures_length_but_refreshes_each_element | Tracking shrink `7\|8\|\|`; fixed shrink `\|\|\|`; growth `21:12`; detachment `--`. |
| join_ignores_shadow_length_and_invalid_index_prototypes | Own length getter and inherited numeric poison getter remain uncalled; output `4,,`. |
| join_retains_exact_utf16_separator_units | Nine literal UTF-16 units include isolated high/low surrogates separated by X; no replacement characters. |
| to_string_alias_is_generic_and_returns_join_result | Exact saved alias, getter then call, original receiver, zero arguments, and an object token returned without coercion. |
| to_string_fallback_and_detached_custom_join | Noncallable join uses intrinsic `[object Card]` fallback with JT trace; detached custom join works although the default typed join throws. |

All ten Number kinds use ordinary authored constructors, without upstream
factory multiplication or class syntax. The fixed/tracking witnesses use tiny
buffers and fixed literal bounds. No full-buffer copy, allocation count or
implementation-specific budget is assumed in the public oracle.

The tracking-arity species returns a separate empty view deliberately: subarray
does not impose the one-argument minimum-length check on a two/three-argument
species result. Conversely fresh result validation must still reject detached
and OOB results. The initially invalid source and invalid result rules are
different and are tested independently.

The control pairs isolate UTF-16 preservation, shared subarray mutation,
tracking argument arity, fresh species-result rejection, captured join length
and generic alias receiver/result identity. Their wrong literals describe
plausible semantic mistakes rather than a constant unconditional throw.
Positive prerequisites and matched same-mode partners prevent missing-method
exceptions from establishing control health.
