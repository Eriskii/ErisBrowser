# Flex wrapping and line alignment

This increment implements column line formation and cross-axis line alignment
using [CSS Flexbox main sizing](https://www.w3.org/TR/css-flexbox-1/#algo-main),
[cross sizing](https://www.w3.org/TR/css-flexbox-1/#algo-cross), and
[line alignment](https://www.w3.org/TR/css-flexbox-1/#align-content-property).
The four new reference pairs use independently specified opaque SVG rectangles;
their reference pages contain no flex layout. These are self-authored regression
tests, not imported Web Platform Tests or full Flexbox conformance evidence.

Column and column-reverse containers collect items using their outer hypothetical
main sizes, including margins and the main-axis gap. A definite height or resolved
maximum height can constrain wrapping. An unconstrained auto height does not wrap
at the viewport height. Minimum/maximum height constraints include the container
box model. Auto height constrained by min/max-height does not make percentage
flex bases definite: they still use content sizing. Percentage main-axis gaps
also remain zero when that axis is indefinite.

Each line uses the existing bounded grow/shrink solver, min/max freezing, scaled
shrink factors, main-axis auto margins and justification. Cross sizes come from
the items on that line; auto column item widths initially use bounded fit-content
measurement. Normal/stretch line alignment shares positive free space equally
among lines before stretching eligible items, respecting their box sizing,
min/max constraints and cross-axis auto margins. Descendants are laid out again
when their item's used width or height changes; source scanning work is never
refunded.

For wrapped rows and columns, align-content supports normal/stretch, flex-start,
flex-end, start, end, center, space-between, space-around and space-evenly.
Wrap-reverse changes cross-start independently of main-axis reversal. Logical
start/end remain top/left and bottom/right in the supported horizontal writing
mode. Explicit safe alignment and distributed-alignment fallbacks avoid placing
overflow before the logical start; ordinary center/end can overflow. A nowrap
container always uses its single line's available cross size, so align-content
does not move that line. A wrapping container with just one resulting line still
applies align-content.

The unit regressions cover both reverse axes, margins, independent gaps,
per-line freezing, definite and indefinite percentage bases, min/max-height,
line and item stretch, all listed alignment keywords, overflow fallbacks, and
the nowrap/one-wrapped-line distinction. A 35,000-item column fixture actually
exhausts command construction and verifies retained geometry and balanced clips.
Line construction charges the existing shared one-million-unit flex budget;
the source, intrinsic measurement, visit, scope and 200,000-command limits also
remain in force.

This remains a partial Flexbox implementation. Full automatic minimum-size and
intrinsic sizing rules, baseline line sizing/alignment, orthogonal writing modes,
visibility:collapse struts, fragmentation, and all percentage/aspect-ratio
interactions are not implemented. Existing percentage-definiteness limitations
inside stretched descendants are not a claim of full CSS sizing support.
