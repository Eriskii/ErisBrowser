//! Private ordinary-inline edges and line geometry. Large token/fragment enums
//! retain their representation; bounded side vectors own the extra metadata.
use super::*;

pub(super) fn charge(work: &mut usize, amount: usize) -> bool {
    if amount > *work {
        return false;
    }
    *work -= amount;
    true
}

pub(super) struct InlineSpan {
    pub(super) node: NodeId,
    pub(super) parent: Option<usize>,
    pub(super) border: Sides,
    pub(super) padding: Sides,
    pub(super) margin: (f32, f32),
    pub(super) first: Option<usize>,
    pub(super) last: Option<usize>,
    pub(super) font_size: f32,
}
impl InlineSpan {
    fn before(&self) -> f32 {
        self.margin.0 + self.border.left + self.padding.left
    }
    fn after(&self) -> f32 {
        self.padding.right + self.border.right + self.margin.1
    }
    pub(super) fn decorated(&self) -> bool {
        self.border.horizontal() != 0.0
            || self.border.vertical() != 0.0
            || self.padding.horizontal() != 0.0
            || self.padding.vertical() != 0.0
            || self.margin.0 != 0.0
            || self.margin.1 != 0.0
    }
    fn geometry(
        &self,
        left: f32,
        right: f32,
        line_top: f32,
        baseline: f32,
        first: bool,
        last: bool,
    ) -> BoxGeometry {
        // Existing nominal text ascent/descent, without line-height leading.
        // This is an explicit engine metric, not measured font-outline bounds.
        let content = rect(
            left,
            line_top + (baseline - self.font_size * 0.95),
            right - left,
            self.font_size * 0.95 + self.font_size * 0.35,
        );
        let pl = if first { self.padding.left } else { 0.0 };
        let pr = if last { self.padding.right } else { 0.0 };
        let bl = if first { self.border.left } else { 0.0 };
        let br = if last { self.border.right } else { 0.0 };
        let padding = rect(
            content.x - pl,
            content.y - self.padding.top,
            content.width + pl + pr,
            content.height + self.padding.vertical(),
        );
        let border = rect(
            padding.x - bl,
            padding.y - self.border.top,
            padding.width + bl + br,
            padding.height + self.border.vertical(),
        );
        BoxGeometry {
            node: self.node,
            border,
            padding,
            content,
            fixed: false,
            inline: true,
        }
    }
}
pub(super) fn push_span(
    spans: &mut Vec<InlineSpan>,
    span: InlineSpan,
    work: &mut usize,
) -> Option<usize> {
    if spans.len() >= MAX_VISITS || !charge(work, 1) || spans.try_reserve(1).is_err() {
        return None;
    }
    let id = spans.len();
    spans.push(span);
    Some(id)
}
fn content(item: &InlineItem) -> bool {
    matches!(
        item.kind,
        InlineKind::Text(_) | InlineKind::Box(..) | InlineKind::Empty
    ) || matches!(item.kind, InlineKind::Space(_)) && item.preserve
}
fn member(item: &InlineItem) -> bool {
    matches!(
        item.kind,
        InlineKind::Text(_) | InlineKind::Space(_) | InlineKind::Box(..) | InlineKind::Empty
    )
}
#[derive(Default)]
pub(super) struct InlineCollection {
    pub(super) items: Vec<InlineItem>,
    pub(super) spans: Vec<InlineSpan>,
    owners: Vec<Option<usize>>,
    pub(super) offsets: Vec<(f32, f32)>,
    pub(super) failed: bool,
}
impl InlineCollection {
    pub(super) fn mark(&mut self, owner: Option<usize>, work: &mut usize) {
        let count = self.items.len().saturating_sub(self.owners.len());
        if !charge(work, count) || self.owners.try_reserve(count).is_err() {
            self.failed = true;
            return;
        }
        self.owners.resize(self.items.len(), owner);
    }
    pub(super) fn has_content(&mut self, start: usize, work: &mut usize) -> bool {
        let count = self.items.len() - start;
        if !charge(work, count) {
            self.failed = true;
            return true;
        }
        self.items[start..].iter().any(content)
    }
    pub(super) fn finish(mut self, work: &mut usize) -> Option<(Vec<InlineItem>, InlineEdges)> {
        if self.failed
            || self.owners.len() != self.items.len()
            || self.offsets.len() != self.spans.len()
        {
            return None;
        }
        // The first token classification scan is separate from vector initialization.
        if !charge(work, self.items.len()) {
            return None;
        }
        for (index, _) in self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| content(item))
        {
            let mut owner = self.owners[index];
            while let Some(id) = owner {
                if !charge(work, 1) {
                    return None;
                }
                let span = &mut self.spans[id];
                span.first.get_or_insert(index);
                span.last = Some(index);
                owner = span.parent;
            }
        }
        let mut extra = Vec::new();
        let mut scratch = Vec::new();
        if !charge(work, self.items.len().saturating_add(self.spans.len()))
            || extra.try_reserve_exact(self.items.len()).is_err()
            || scratch.try_reserve_exact(self.spans.len()).is_err()
        {
            return None;
        }
        extra.resize(self.items.len(), 0.0);
        scratch.resize(self.spans.len(), Occurrence::default());
        for span in &self.spans {
            if let Some(first) = span.first {
                extra[first] += span.before();
            }
            if let Some(last) = span.last {
                extra[last] += span.after();
            }
        }
        Some((
            self.items,
            InlineEdges {
                spans: self.spans,
                owners: self.owners,
                offsets: self.offsets,
                extra,
                scratch,
                generation: 0,
            },
        ))
    }
}
#[derive(Clone, Copy, Default)]
struct Occurrence {
    generation: usize,
    first: usize,
    last: usize,
    opening: bool,
    closing: bool,
    anchor: bool,
    start: f32,
}
pub(super) struct InlineEdges {
    pub(super) spans: Vec<InlineSpan>,
    owners: Vec<Option<usize>>,
    offsets: Vec<(f32, f32)>,
    extra: Vec<f32>,
    scratch: Vec<Occurrence>,
    generation: usize,
}
pub(super) struct LinePlan {
    pub(super) positions: Vec<f32>,
    pub(super) boxes: Vec<(BoxGeometry, (f32, f32))>,
}
impl InlineEdges {
    pub(super) fn extra(&self, index: usize) -> f32 {
        self.extra[index]
    }
    #[allow(clippy::too_many_arguments)]
    pub(super) fn line_plan(
        &mut self,
        line: &[InlineItem],
        indices: &[usize],
        x: f32,
        baseline: f32,
        line_box: (f32, f32),
        extra_space: f32,
        work: &mut usize,
    ) -> Option<LinePlan> {
        // Charge even nonmembers before walking the complete line.
        if !charge(work, line.len()) {
            return None;
        }
        self.generation = self.generation.checked_add(1)?;
        let mut touched = Vec::new();
        // Each membership charges its bounded parent lookup and record update.
        // Stale scratch records are ignored by generation; refusal is transactional.
        for (at, (&index, item)) in indices.iter().zip(line).enumerate() {
            let anchor = matches!(item.kind, InlineKind::Positioned(_));
            if !member(item) && !anchor {
                continue;
            }
            let mut owner = self.owners[index];
            while let Some(id) = owner {
                if !charge(work, 2) {
                    return None;
                }
                let span = &self.spans[id];
                if anchor && span.first.is_some() {
                    owner = span.parent;
                    continue;
                }
                let slot = &mut self.scratch[id];
                if slot.generation != self.generation {
                    touched.try_reserve(1).ok()?;
                    touched.push(id);
                    *slot = Occurrence {
                        generation: self.generation,
                        first: at,
                        last: at,
                        anchor,
                        ..Occurrence::default()
                    };
                }
                slot.last = at;
                slot.anchor &= anchor;
                slot.opening |= !anchor && span.first.is_none_or(|first| first == index);
                slot.closing |= !anchor && span.last.is_none_or(|last| last == index);
                owner = span.parent;
            }
        }
        let count = touched.len();
        let sort_depth = (usize::BITS - count.leading_zeros()) as usize;
        let cost = count
            .checked_mul(2 * (sort_depth + 2))?
            .checked_add(line.len())?;
        if !charge(work, cost) {
            return None;
        }
        let mut opens = Vec::new();
        let mut closes = Vec::new();
        opens.try_reserve_exact(count).ok()?;
        closes.try_reserve_exact(count).ok()?;
        for id in touched {
            opens.push((self.scratch[id].first, id));
            closes.push((self.scratch[id].last, std::cmp::Reverse(id)));
        }
        // Span IDs are preorder: outer opening first and inner closing first.
        opens.sort_unstable();
        closes.sort_unstable();
        let mut positions = Vec::new();
        let mut boxes = Vec::new();
        positions.try_reserve_exact(line.len()).ok()?;
        boxes.try_reserve_exact(count).ok()?;
        let (mut opening, mut closing, mut cursor) = (0, 0, x);
        for (at, item) in line.iter().enumerate() {
            while opening < opens.len() && opens[opening].0 == at {
                let id = opens[opening].1;
                let span = &self.spans[id];
                let slot = &mut self.scratch[id];
                if slot.opening {
                    cursor += span.before();
                }
                slot.start = cursor;
                opening += 1;
            }
            positions.push(cursor);
            cursor += item.width
                + if matches!(item.kind, InlineKind::Space(_)) {
                    extra_space
                } else {
                    0.0
                };
            while closing < closes.len() && closes[closing].0 == at {
                let id = closes[closing].1.0;
                let span = &self.spans[id];
                let slot = self.scratch[id];
                // Out-of-flow-only owners retain the old line-top anchor. They
                // neither consume physical edges nor acquire a font-height box.
                let geometry = if slot.anchor {
                    let area = rect(slot.start, line_box.0, 0.0, line_box.1);
                    BoxGeometry {
                        node: span.node,
                        border: area,
                        padding: area,
                        content: area,
                        fixed: false,
                        inline: true,
                    }
                } else {
                    span.geometry(
                        slot.start,
                        cursor,
                        line_box.0,
                        baseline,
                        slot.opening,
                        slot.closing,
                    )
                };
                boxes.push((geometry, self.offsets[id]));
                if slot.closing {
                    cursor += span.after();
                }
                closing += 1;
            }
            if !cursor.is_finite() || cursor.abs() > MAX_EXTENT {
                return None;
            }
        }
        Some(LinePlan { positions, boxes })
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod independent_tests;
