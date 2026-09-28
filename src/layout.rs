//! Bounded, independent CSS box construction, line layout, and display lists.
//!
//! This implements the useful core of block, inline, flex and grid layout. It is
//! deliberately explicit about its limits: it is not a complete CSS formatter.
use crate::css::{ComputedStyle, Display, GridBreadth, GridLine, GridTrack, Length};
use crate::dom::{Document, Namespace, NodeId, NodeKind};
use crate::graphics::{Color, DrawCommand, Fonts, Rect};
use std::cell::Cell as Counter;

const MAX_DEPTH: usize = 128;
const MAX_VISITS: usize = 100_000;
const MAX_COMMANDS: usize = 200_000;
const MAX_GLYPHS: usize = 500_000;
const MAX_FLEX_WORK: usize = 1_000_000;
const MAX_FLOAT_WORK: usize = 1_000_000;
const MAX_FLOATS: usize = 4096;
const MAX_EXTENT: f32 = 1_000_000.0;

// Grid occupancy is bounded independently from DOM size. Signed line numbers
// are clamped before arithmetic; excessive author grids have a finite UA limit.
const MAX_GRID_TRACKS: usize = 256;
const MAX_GRID_ITEMS: usize = 4096;
const MAX_GRID_WORK: usize = 2_000_000;

#[derive(Clone, Copy, Default)]
struct GridAxis {
    start: Option<i32>,
    span: usize,
}
#[derive(Clone, Copy, Default, Debug)]
struct GridArea {
    column: usize,
    row: usize,
    columns: usize,
    rows: usize,
}
struct GridItem {
    id: NodeId,
    major: GridAxis,
    minor: GridAxis,
    area: Option<GridArea>,
}
struct GridPlan {
    items: Vec<GridItem>,
    columns: usize,
    rows: usize,
    column_origin: usize,
    row_origin: usize,
}

fn grid_axis(start: GridLine, end: GridLine, explicit: usize) -> GridAxis {
    let line = |value: i32| {
        let line = if value > 0 {
            i64::from(value) - 1
        } else {
            explicit as i64 + 1 + i64::from(value)
        };
        line.clamp(-128, 128) as i32
    };
    let span = |value: usize| value.clamp(1, MAX_GRID_TRACKS);
    match (start, end) {
        (GridLine::Line(a), GridLine::Line(b)) => {
            let (a, b) = (line(a), line(b));
            GridAxis {
                start: Some(a.min(b)),
                span: (a - b).unsigned_abs().max(1) as usize,
            }
        }
        (GridLine::Line(a), b) => GridAxis {
            start: Some(line(a)),
            span: if let GridLine::Span(n) = b {
                span(n)
            } else {
                1
            },
        },
        (a, GridLine::Line(b)) => {
            let span = if let GridLine::Span(n) = a {
                span(n)
            } else {
                1
            };
            GridAxis {
                start: Some((line(b) - span as i32).max(-128)),
                span,
            }
        }
        (GridLine::Span(n), _) | (_, GridLine::Span(n)) => GridAxis {
            start: None,
            span: span(n),
        },
        _ => GridAxis {
            start: None,
            span: 1,
        },
    }
}

fn grid_charge(work: &mut usize, cost: usize) -> bool {
    if *work < cost {
        *work = 0;
        false
    } else {
        *work -= cost;
        true
    }
}
fn grid_occupied(
    occupancy: &[bool],
    major: usize,
    minor: usize,
    a: usize,
    b: usize,
    work: &mut usize,
) -> bool {
    if major + a > MAX_GRID_TRACKS || minor + b > MAX_GRID_TRACKS || !grid_charge(work, a * b + 1) {
        return true;
    }
    (major..major + a).any(|row| {
        occupancy[row * MAX_GRID_TRACKS + minor..row * MAX_GRID_TRACKS + minor + b]
            .iter()
            .any(|used| *used)
    })
}
fn grid_place(
    item: &mut GridItem,
    occupancy: &mut [bool],
    major: usize,
    minor: usize,
    transpose: bool,
    work: &mut usize,
) -> bool {
    let (a, b) = (item.major.span, item.minor.span);
    if major + a > MAX_GRID_TRACKS || minor + b > MAX_GRID_TRACKS || !grid_charge(work, a * b + 1) {
        return false;
    }
    for row in major..major + a {
        occupancy[row * MAX_GRID_TRACKS + minor..row * MAX_GRID_TRACKS + minor + b].fill(true);
    }
    item.area = Some(if transpose {
        GridArea {
            column: major,
            row: minor,
            columns: a,
            rows: b,
        }
    } else {
        GridArea {
            column: minor,
            row: major,
            columns: b,
            rows: a,
        }
    });
    true
}
fn grid_plan(
    ids: Vec<NodeId>,
    styles: &[ComputedStyle],
    style: &ComputedStyle,
    work: &mut usize,
) -> GridPlan {
    if !grid_charge(work, ids.len().clamp(1, MAX_GRID_ITEMS)) {
        return GridPlan {
            items: Vec::new(),
            columns: style.grid_template_columns.len().min(MAX_GRID_TRACKS),
            rows: style.grid_template_rows.len().min(MAX_GRID_TRACKS),
            column_origin: 0,
            row_origin: 0,
        };
    }
    let transpose = style
        .grid_auto_flow
        .split_whitespace()
        .any(|v| v == "column");
    let dense = style
        .grid_auto_flow
        .split_whitespace()
        .any(|v| v == "dense");
    let mut items = Vec::new();
    for id in ids.into_iter().take(MAX_GRID_ITEMS) {
        let Some(child) = styles.get(id) else {
            continue;
        };
        if matches!(child.position.as_str(), "absolute" | "fixed") {
            continue;
        }
        let col = grid_axis(
            child.grid_column_start,
            child.grid_column_end,
            style.grid_template_columns.len(),
        );
        let row = grid_axis(
            child.grid_row_start,
            child.grid_row_end,
            style.grid_template_rows.len(),
        );
        items.push(GridItem {
            id,
            major: if transpose { col } else { row },
            minor: if transpose { row } else { col },
            area: None,
        });
    }
    items.sort_by_key(|item| styles[item.id].order);
    let major_origin = items
        .iter()
        .filter_map(|item| item.major.start)
        .min()
        .unwrap_or(0)
        .min(0)
        .unsigned_abs() as usize;
    let minor_origin = items
        .iter()
        .filter_map(|item| item.minor.start)
        .min()
        .unwrap_or(0)
        .min(0)
        .unsigned_abs() as usize;
    let (major_explicit, minor_explicit) = if transpose {
        (
            style.grid_template_columns.len(),
            style.grid_template_rows.len(),
        )
    } else {
        (
            style.grid_template_rows.len(),
            style.grid_template_columns.len(),
        )
    };
    let mut major_count = (major_origin + major_explicit).min(MAX_GRID_TRACKS);
    let mut minor_count = (minor_origin + minor_explicit).clamp(1, MAX_GRID_TRACKS);
    for item in &mut items {
        for (axis, origin, count) in [
            (&mut item.major, major_origin, &mut major_count),
            (&mut item.minor, minor_origin, &mut minor_count),
        ] {
            axis.span = axis.span.clamp(1, MAX_GRID_TRACKS);
            if let Some(start) = axis.start {
                let start = (start + origin as i32).max(0) as usize;
                let start = start.min(MAX_GRID_TRACKS - axis.span);
                axis.start = Some(start as i32);
                *count = (*count).max(start + axis.span);
            }
        }
        minor_count = minor_count.max(item.minor.span);
    }
    if !items.is_empty() && !grid_charge(work, MAX_GRID_TRACKS) {
        return GridPlan {
            items: Vec::new(),
            columns: 0,
            rows: 0,
            column_origin: 0,
            row_origin: 0,
        };
    }
    let mut occupancy = if items.is_empty() {
        Vec::new()
    } else {
        vec![false; MAX_GRID_TRACKS * MAX_GRID_TRACKS]
    };
    // Definite items may overlap. They reserve their cells before any auto item.
    for item in &mut items {
        if let (Some(a), Some(b)) = (item.major.start, item.minor.start) {
            grid_place(
                item,
                &mut occupancy,
                a as usize,
                b as usize,
                transpose,
                work,
            );
        }
    }
    let mut locked_cursor = [0usize; MAX_GRID_TRACKS];
    for item in &mut items {
        if item.area.is_some() || item.major.start.is_none() || item.minor.start.is_some() {
            continue;
        }
        let major = item.major.start.unwrap_or(0) as usize;
        let mut minor = if dense { 0 } else { locked_cursor[major] };
        while *work > 0 && minor + item.minor.span <= MAX_GRID_TRACKS {
            if !grid_occupied(
                &occupancy,
                major,
                minor,
                item.major.span,
                item.minor.span,
                work,
            ) {
                if grid_place(item, &mut occupancy, major, minor, transpose, work) {
                    locked_cursor[major] = minor + item.minor.span;
                    minor_count = minor_count.max(minor + item.minor.span);
                    major_count = major_count.max(major + item.major.span);
                }
                break;
            }
            minor += 1;
        }
    }
    let (mut cursor_major, mut cursor_minor) = (0usize, 0usize);
    for item in &mut items {
        if item.area.is_some() || item.major.start.is_some() {
            continue;
        }
        if dense {
            cursor_major = 0;
            cursor_minor = 0;
        }
        if let Some(minor) = item.minor.start {
            let minor = minor as usize;
            if !dense && minor < cursor_minor {
                cursor_major += 1;
            }
            cursor_minor = minor;
            while *work > 0 && cursor_major + item.major.span <= MAX_GRID_TRACKS {
                if !grid_occupied(
                    &occupancy,
                    cursor_major,
                    cursor_minor,
                    item.major.span,
                    item.minor.span,
                    work,
                ) {
                    break;
                }
                cursor_major += 1;
            }
        } else {
            while *work > 0 && cursor_major + item.major.span <= MAX_GRID_TRACKS {
                if cursor_minor + item.minor.span > minor_count {
                    cursor_minor = 0;
                    cursor_major += 1;
                    continue;
                }
                if !grid_occupied(
                    &occupancy,
                    cursor_major,
                    cursor_minor,
                    item.major.span,
                    item.minor.span,
                    work,
                ) {
                    break;
                }
                cursor_minor += 1;
            }
        }
        if *work > 0
            && grid_place(
                item,
                &mut occupancy,
                cursor_major,
                cursor_minor,
                transpose,
                work,
            )
        {
            major_count = major_count.max(cursor_major + item.major.span);
        }
    }
    GridPlan {
        items,
        columns: if transpose { major_count } else { minor_count },
        rows: if transpose { minor_count } else { major_count },
        column_origin: if transpose {
            major_origin
        } else {
            minor_origin
        },
        row_origin: if transpose {
            minor_origin
        } else {
            major_origin
        },
    }
}

fn grid_track_list(
    explicit: &[GridTrack],
    implicit: &[GridTrack],
    origin: usize,
    count: usize,
) -> Vec<GridTrack> {
    let auto = [GridTrack::default()];
    let implicit = if implicit.is_empty() {
        &auto[..]
    } else {
        implicit
    };
    (0..count.min(MAX_GRID_TRACKS))
        .map(|index| {
            if index >= origin && index - origin < explicit.len() {
                explicit[index - origin]
            } else {
                let index = if index < origin {
                    (implicit.len() - (origin - index) % implicit.len()) % implicit.len()
                } else {
                    (index - origin - explicit.len()) % implicit.len()
                };
                implicit[index]
            }
        })
        .collect()
}
/// Whitespace-only anonymous items do not generate grid/flex boxes. Eligibility
/// scanning is source work too, and cannot be refunded when paint is discarded.
fn has_inline_content(text: &str, source_work: &mut usize) -> bool {
    for character in text.chars().take(*source_work) {
        *source_work -= 1;
        if !character.is_whitespace() {
            return true;
        }
    }
    false
}

fn grid_length(length: Length, reference: Option<f32>) -> Option<f32> {
    match length {
        Length::Percent(_) => reference.and_then(|r| resolve(length, r)),
        _ => resolve(length, reference.unwrap_or(0.0)),
    }
}
fn grid_breadth(breadth: GridBreadth, reference: Option<f32>) -> Option<f32> {
    if let GridBreadth::Length(length) = breadth {
        grid_length(length, reference).map(extent)
    } else {
        None
    }
}
#[derive(Clone, Copy)]
struct GridContribution {
    start: usize,
    span: usize,
    min: f32,
    max: f32,
}
#[derive(Clone, Copy)]
struct GridTrackSize {
    base: f32,
    growth: f32,
    growth_known: bool,
    infinitely_growable: bool,
    min: GridBreadth,
    max: GridBreadth,
    flex: f32,
}

// Distribute a required increase, freezing finite maxima. Planned increases are
// combined by maximum within each equal-span group, so DOM order is immaterial.
fn grid_grow(
    values: &mut [f32],
    limits: &[f32],
    eligible: &[bool],
    mut free: f32,
    work: &mut usize,
) {
    for _ in 0..=values.len() {
        if free <= 0.001 || !grid_charge(work, values.len().max(1)) {
            break;
        }
        let count = values
            .iter()
            .zip(limits)
            .zip(eligible)
            .filter(|((value, limit), eligible)| **eligible && **value + 0.001 < **limit)
            .count();
        if count == 0 {
            break;
        }
        let share = free / count as f32;
        let mut used = 0.0;
        for ((value, limit), eligible) in values.iter_mut().zip(limits).zip(eligible) {
            if *eligible {
                let add = share.min((*limit - *value).max(0.0));
                *value += add;
                used += add;
            }
        }
        if used <= 0.001 {
            break;
        }
        free -= used;
    }
}
fn size_grid_tracks(
    definitions: &[GridTrack],
    reference: Option<f32>,
    gap: f32,
    contributions: &[GridContribution],
    alignment: &str,
    work: &mut usize,
) -> Vec<f32> {
    let mut tracks: Vec<GridTrackSize> = definitions
        .iter()
        .map(|track| {
            let base = grid_breadth(track.min, reference).unwrap_or(0.0);
            let fixed_max = grid_breadth(track.max, reference);
            GridTrackSize {
                base,
                growth: fixed_max.unwrap_or(base).max(base),
                growth_known: fixed_max.is_some(),
                infinitely_growable: false,
                min: track.min,
                max: track.max,
                flex: if let GridBreadth::Length(Length::Fr(n)) = track.max {
                    extent(n)
                } else {
                    0.0
                },
            }
        })
        .collect();
    let mut ordered = contributions.to_vec();
    ordered.sort_by_key(|item| item.span);
    // Resolve each span group completely before longer spans. In particular,
    // a max-content limit established by a short item must constrain the base
    // increase from a longer item; resolving all minima first loses that fact.
    let mut first = 0;
    while first < ordered.len() && *work > 0 {
        let span = ordered[first].span;
        let end = first + ordered[first..].partition_point(|item| item.span == span);
        for phase in 0..4 {
            let growth_phase = phase >= 2;
            let max_content = phase == 1 || phase == 3;
            let before: Vec<f32> = tracks
                .iter()
                .map(|track| {
                    if growth_phase {
                        track.growth
                    } else {
                        track.base
                    }
                })
                .collect();
            let mut planned = before.clone();
            let mut touched = vec![false; tracks.len()];
            for item in &ordered[first..end] {
                let start = item.start;
                let end = (start + item.span).min(tracks.len());
                if start >= end || !grid_charge(work, end - start) {
                    continue;
                }
                let slice = &tracks[start..end];
                let crossing_flex = slice
                    .iter()
                    .any(|track| matches!(track.max, GridBreadth::Length(Length::Fr(_))));
                let intrinsic_max = |track: &GridTrackSize| {
                    grid_breadth(track.max, reference).is_none()
                        && !matches!(track.max, GridBreadth::Length(Length::Fr(_)))
                };
                let content_max = |track: &GridTrackSize| {
                    intrinsic_max(track) && track.max != GridBreadth::MinContent
                };
                let eligible: Vec<bool> = slice
                    .iter()
                    .map(|track| {
                        if span > 1
                            && crossing_flex
                            && (!matches!(track.max, GridBreadth::Length(Length::Fr(_)))
                                || phase >= 2)
                        {
                            false
                        } else {
                            match phase {
                                0 => {
                                    grid_breadth(track.min, reference).is_none()
                                        && !(span > 1
                                            && crossing_flex
                                            && track.min == GridBreadth::Auto)
                                }
                                1 => track.min == GridBreadth::MaxContent,
                                2 => intrinsic_max(track),
                                _ => content_max(track),
                            }
                        }
                    })
                    .collect();
                if !eligible.iter().any(|value| *value) {
                    continue;
                }
                let required = if max_content { item.max } else { item.min };
                let required = (required - gap * (end - start - 1) as f32).max(0.0);
                let mut values = before[start..end].to_vec();
                let limits: Vec<f32> = slice
                    .iter()
                    .map(|track| {
                        if track.growth_known && !(growth_phase && track.infinitely_growable) {
                            track.growth
                        } else {
                            MAX_EXTENT
                        }
                    })
                    .collect();
                let free = (required - values.iter().sum::<f32>()).max(0.0);
                grid_grow(&mut values, &limits, &eligible, free, work);
                // Prefer unused room in non-affected tracks before exceeding a
                // previously established growth limit.
                let others: Vec<bool> = eligible
                    .iter()
                    .map(|value| !(*value || span > 1 && crossing_flex))
                    .collect();
                let free = (required - values.iter().sum::<f32>()).max(0.0);
                grid_grow(&mut values, &limits, &others, free, work);
                let mut beyond: Vec<bool> = slice
                    .iter()
                    .zip(&eligible)
                    .map(|(track, eligible)| {
                        *eligible
                            && if growth_phase || !max_content {
                                intrinsic_max(track)
                            } else {
                                content_max(track)
                            }
                    })
                    .collect();
                if !growth_phase && !beyond.iter().any(|value| *value) {
                    beyond.clone_from(&eligible);
                }
                let free = (required - values.iter().sum::<f32>()).max(0.0);
                grid_grow(
                    &mut values,
                    &vec![MAX_EXTENT; end - start],
                    &beyond,
                    free,
                    work,
                );
                for (offset, value) in values.into_iter().enumerate() {
                    planned[start + offset] = planned[start + offset].max(value);
                    touched[start + offset] |= eligible[offset] || value > before[start + offset];
                }
            }
            for ((track, value), touched) in tracks.iter_mut().zip(planned).zip(touched) {
                if growth_phase {
                    track.growth = extent(value).max(track.base);
                    if touched {
                        track.infinitely_growable |= !track.growth_known;
                        track.growth_known = true;
                    }
                } else {
                    track.base = extent(value);
                    track.growth = track.growth.max(track.base);
                }
            }
        }
        for track in &mut tracks {
            track.infinitely_growable = false;
        }
        first = end;
    }
    let gap_total = gap * tracks.len().saturating_sub(1) as f32;
    let mut values: Vec<f32> = tracks.iter().map(|track| track.base).collect();
    if let Some(reference) = reference {
        let free = (reference - gap_total - values.iter().sum::<f32>()).max(0.0);
        let limits: Vec<f32> = tracks.iter().map(|track| track.growth).collect();
        let eligible: Vec<bool> = tracks
            .iter()
            .map(|track| !matches!(track.max, GridBreadth::Length(Length::Fr(_))))
            .collect();
        grid_grow(&mut values, &limits, &eligible, free, work);
        let mut frozen: Vec<bool> = tracks.iter().map(|track| track.flex == 0.0).collect();
        for _ in 0..=tracks.len() {
            if !grid_charge(work, tracks.len().max(1)) {
                break;
            }
            let occupied: f32 = values
                .iter()
                .zip(&frozen)
                .filter(|(_, frozen)| **frozen)
                .map(|(v, _)| *v)
                .sum();
            let factors: f32 = tracks
                .iter()
                .zip(&frozen)
                .filter(|(_, frozen)| !**frozen)
                .map(|(t, _)| t.flex)
                .sum();
            let fraction = (reference - gap_total - occupied).max(0.0) / factors.max(1.0);
            let mut changed = false;
            for ((track, value), frozen) in tracks.iter().zip(&values).zip(&mut frozen) {
                if !*frozen && *value > fraction * track.flex + 0.001 {
                    *frozen = true;
                    changed = true;
                }
            }
            if !changed {
                for ((track, value), frozen) in tracks.iter().zip(&mut values).zip(&frozen) {
                    if !*frozen {
                        *value = extent(fraction * track.flex).max(*value);
                    }
                }
                break;
            }
        }
        if matches!(alignment, "normal" | "stretch") {
            let free = (reference - gap_total - values.iter().sum::<f32>()).max(0.0);
            let eligible: Vec<bool> = tracks
                .iter()
                .map(|track| track.max == GridBreadth::Auto)
                .collect();
            grid_grow(
                &mut values,
                &vec![MAX_EXTENT; tracks.len()],
                &eligible,
                free,
                work,
            );
        }
    } else {
        for (value, track) in values.iter_mut().zip(&tracks) {
            if track.flex == 0.0 {
                *value = track.growth;
            }
        }
        let mut fraction = tracks
            .iter()
            .filter(|track| track.flex > 0.0)
            .map(|track| track.base / track.flex.max(1.0))
            .fold(0.0, f32::max);
        for item in &ordered {
            let start = item.start;
            let end = (start + item.span).min(tracks.len());
            if start >= end || !grid_charge(work, end - start) {
                continue;
            }
            let factors: f32 = tracks[start..end].iter().map(|track| track.flex).sum();
            if factors > 0.0 {
                let fixed: f32 = tracks[start..end]
                    .iter()
                    .zip(&values[start..end])
                    .filter(|(track, _)| track.flex == 0.0)
                    .map(|(_, v)| *v)
                    .sum();
                fraction = fraction.max(
                    (item.max - fixed - gap * (end - start - 1) as f32).max(0.0) / factors.max(1.0),
                );
            }
        }
        for (value, track) in values.iter_mut().zip(&tracks) {
            if track.flex > 0.0 {
                *value = value.max(extent(fraction * track.flex));
            }
        }
    }
    values
}
fn grid_positions(
    sizes: &[f32],
    gap: f32,
    reference: Option<f32>,
    alignment: &str,
) -> (Vec<f32>, f32) {
    let total = sizes.iter().sum::<f32>() + gap * sizes.len().saturating_sub(1) as f32;
    let (start, between) = distribution(alignment, reference.unwrap_or(total) - total, sizes.len());
    let mut cursor = start;
    let positions = sizes
        .iter()
        .map(|size| {
            let pos = cursor;
            cursor += size + gap + between;
            pos
        })
        .collect();
    (positions, gap + between)
}
fn grid_area_size(sizes: &[f32], start: usize, span: usize, gap: f32) -> f32 {
    extent(
        sizes
            .get(start..start + span)
            .unwrap_or(&[])
            .iter()
            .sum::<f32>()
            + gap * span.saturating_sub(1) as f32,
    )
}
fn grid_alignment<'a>(
    child: &'a ComputedStyle,
    parent: &'a ComputedStyle,
    horizontal: bool,
) -> &'a str {
    let (own, inherited) = if horizontal {
        (&child.justify_self, &parent.justify_items)
    } else {
        (&child.align_self, &parent.align_items)
    };
    let alignment = if own == "auto" {
        inherited.as_str()
    } else {
        own.as_str()
    };
    if alignment == "normal" {
        "stretch"
    } else {
        alignment
    }
}

const MAX_POSITION_WORK: usize = 2_000_000;

#[derive(Clone, Copy)]
struct PaintOwner {
    node: NodeId,
    background: bool,
}
#[derive(Clone, Copy)]
struct BoxGeometry {
    node: NodeId,
    border: Rect,
    padding: Rect,
    content: Rect,
    fixed: bool,
    inline: bool,
}
#[derive(Clone, Copy)]
struct PositionJob {
    node: NodeId,
    x: f32,
    y: f32,
    fixed: bool,
    depth: usize,
}

/// Solve one positioned axis in border-box units. Fixed opposing insets make
/// an auto non-replaced size fill the remaining space; otherwise the caller's
/// shrink-to-fit/natural size supplies the missing dimension. Minimums win.
#[derive(Clone, Copy)]
struct PositionedAxis {
    containing: f32,
    start: Option<f32>,
    end: Option<f32>,
    size: Option<f32>,
    natural: f32,
    min: f32,
    max: f32,
    margin_start: Option<f32>,
    margin_end: Option<f32>,
    static_start: f32,
    horizontal: bool,
}
fn positioned_axis(axis: PositionedAxis) -> (f32, f32) {
    let mut first = axis.margin_start.unwrap_or(0.0);
    let last = axis.margin_end.unwrap_or(0.0);
    let fill = match (axis.start, axis.end) {
        (Some(start), Some(end)) => Some(axis.containing - start - end - first - last),
        _ => None,
    };
    let size = extent(axis.size.or(fill).unwrap_or(axis.natural)).clamp(axis.min, axis.max);
    let offset = match (axis.start, axis.end) {
        (Some(start), Some(end)) => {
            // Re-solving after min/max is the specified-size equation. Auto
            // margins divide the residue; over-constrained LTR keeps start.
            let free = axis.containing - start - end - size - first - last;
            match (axis.margin_start, axis.margin_end) {
                (None, None) if free >= 0.0 || !axis.horizontal => first = free / 2.0,
                (None, Some(_)) => first = free,
                _ => {}
            }
            start + first
        }
        (Some(start), None) => start + first,
        (None, Some(end)) => axis.containing - end - last - size,
        (None, None) => axis.static_start + first,
    };
    (finite(offset, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT), size)
}

#[derive(Debug, Clone)]
pub struct HitRegion {
    pub node: NodeId,
    pub rect: Rect,
    /// Coordinates are viewport-relative when true, document-relative otherwise.
    pub fixed: bool,
}

pub struct LayoutResult {
    pub commands: Vec<DrawCommand>,
    pub hit_regions: Vec<HitRegion>,
    pub content_height: f32,
}

impl LayoutResult {
    /// Paint order is also hit-test order; descendants win over ancestors.
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        self.hit_regions.iter().rev().find_map(|hit| {
            (x >= hit.rect.x
                && y >= hit.rect.y
                && x < hit.rect.x + hit.rect.width
                && y < hit.rect.y + hit.rect.height)
                .then_some(hit.node)
        })
    }
}

#[derive(Clone, Copy, Default)]
struct Sides {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl Sides {
    fn horizontal(self) -> f32 {
        self.left + self.right
    }
    fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Default)]
struct Size {
    width: f32,
    height: f32,
}

struct Fragment {
    size: Size,
    commands: Vec<DrawCommand>,
    owners: Vec<PaintOwner>,
    hits: Vec<HitRegion>,
    boxes: Vec<BoxGeometry>,
    positioned: Vec<PositionJob>,
    rounded_border: bool,
}

enum InlineKind {
    Text(String),
    Space(String),
    Box(Fragment, Sides),
    Float(NodeId),
    Positioned(NodeId),
    Break,
}

#[derive(Clone, Copy)]
struct FloatExclusion {
    outer: Rect,
    right: bool,
}

struct FloatPaint {
    fragment: Fragment,
    x: f32,
    y: f32,
}

#[derive(Default)]
struct FloatContext {
    boxes: Vec<FloatExclusion>,
    paint: Vec<FloatPaint>,
    bottom: f32,
    left_bottom: f32,
    right_bottom: f32,
    source_top: f32,
}

#[derive(Clone, Copy)]
struct FloatBand {
    left: f32,
    right: f32,
    next: Option<f32>,
}
impl FloatBand {
    fn width(self) -> f32 {
        (self.right - self.left).max(0.0)
    }
}

struct InlineItem {
    node: NodeId,
    owner: NodeId,
    kind: InlineKind,
    width: f32,
    height: f32,
    preserve: bool,
}

struct Engine<'a> {
    doc: &'a Document,
    styles: &'a [ComputedStyle],
    fonts: &'a Fonts,
    viewport: Size,
    commands: Vec<DrawCommand>,
    owners: Vec<PaintOwner>,
    hits: Vec<HitRegion>,
    boxes: Vec<BoxGeometry>,
    positioned: Vec<PositionJob>,
    paint_owner: PaintOwner,
    flow_height_reference: Option<f32>,
    position_work_left: usize,
    visits: usize,
    hits_created: usize,
    glyphs_left: usize,
    emitted_glyphs_left: usize,
    intrinsic_work_left: Counter<usize>,
    flex_work_left: usize,
    float_work_left: usize,
    grid_work_left: usize,
    grid_height_reference: Option<(NodeId, Option<f32>)>,
    floats_left: usize,
    floats: FloatContext,
    fragment_root: Option<NodeId>,
    commands_created: usize,
    open_clips: usize,
    canvas_background_node: Option<NodeId>,
    fallback: ComputedStyle,
}

/// Construct a display list in document coordinates. Scrolling is a paint-time
/// transform; callers should not rerun layout for every scroll event.
pub fn layout(
    doc: &Document,
    styles: &[ComputedStyle],
    width: f32,
    height: f32,
    fonts: &Fonts,
) -> LayoutResult {
    let viewport = Size {
        width: finite(width, 800.0).clamp(1.0, 100_000.0),
        height: finite(height, 600.0).clamp(1.0, 100_000.0),
    };
    let html = doc.first_html_element("html");
    let canvas_background_node = html
        .filter(|id| {
            styles
                .get(*id)
                .is_some_and(|style| style.background_color.a > 0)
        })
        .or_else(|| {
            doc.first_html_element("body").filter(|id| {
                styles.get(*id).is_some_and(|style| {
                    style.display != Display::None && style.background_color.a > 0
                })
            })
        });
    let mut engine = Engine {
        doc,
        styles,
        fonts,
        viewport,
        commands: Vec::new(),
        owners: Vec::new(),
        hits: Vec::new(),
        boxes: Vec::new(),
        positioned: Vec::new(),
        paint_owner: PaintOwner {
            node: doc.root,
            background: true,
        },
        flow_height_reference: Some(viewport.height),
        position_work_left: MAX_POSITION_WORK,
        visits: 0,
        hits_created: 0,
        glyphs_left: MAX_GLYPHS,
        emitted_glyphs_left: MAX_GLYPHS,
        intrinsic_work_left: Counter::new(MAX_GLYPHS),
        flex_work_left: MAX_FLEX_WORK,
        float_work_left: MAX_FLOAT_WORK,
        grid_work_left: MAX_GRID_WORK,
        grid_height_reference: None,
        floats_left: MAX_FLOATS,
        floats: FloatContext::default(),
        fragment_root: None,
        commands_created: 0,
        open_clips: 0,
        canvas_background_node,
        fallback: ComputedStyle::default(),
    };
    if let Some(id) = canvas_background_node {
        // A propagated body background belongs to the root element's canvas
        // background group, including the root's opacity, not body's opacity.
        engine.paint_owner.node = html.unwrap_or(id);
        let style = engine.style(id);
        engine.push(DrawCommand::Rect {
            rect: rect(0.0, 0.0, viewport.width, viewport.height),
            color: style.background_color,
            radius: 0.0,
        });
        engine.paint_owner.node = doc.root;
    }
    let size = if doc.nodes.get(doc.root).is_some() {
        engine.layout_box(doc.root, 0.0, 0.0, viewport.width, Some(viewport.width), 0)
    } else {
        Size::default()
    };
    engine.resolve_positioned();
    let painted_bottom = engine
        .hits
        .iter()
        .filter(|hit| !hit.fixed && hit.rect.width > 0.0 && hit.rect.height > 0.0)
        .map(|hit| hit.rect.y + hit.rect.height)
        .fold(size.height, f32::max);
    let content_height = painted_bottom.max(viewport.height).min(MAX_EXTENT);
    if canvas_background_node.is_some()
        && let Some(DrawCommand::Rect { rect, .. }) = engine.commands.first_mut()
    {
        rect.height = content_height;
    }
    engine.restack();
    engine.commands.retain(|command| match command {
        DrawCommand::Rect { rect, color, .. } => {
            color.a > 0 && rect.width > 0.0 && rect.height > 0.0
        }
        DrawCommand::Text { text, color, .. } => !text.is_empty() && color.a > 0,
        _ => true,
    });
    LayoutResult {
        commands: engine.commands,
        hit_regions: engine.hits,
        content_height,
    }
}

impl Engine<'_> {
    fn html_tag(&self, id: NodeId) -> Option<&str> {
        (self.doc.namespace(id) == Some(Namespace::Html))
            .then(|| self.doc.tag(id))
            .flatten()
    }
    /// Element-specific layout is available for HTML and SVG viewport roots.
    fn layout_tag(&self, id: NodeId) -> &str {
        match (self.doc.namespace(id), self.doc.tag(id)) {
            (Some(Namespace::Svg), Some("svg")) => "svg",
            (Some(Namespace::Html), Some(tag)) if tag != "svg" => tag,
            _ => "",
        }
    }
    fn style(&self, id: NodeId) -> &ComputedStyle {
        self.styles.get(id).unwrap_or(&self.fallback)
    }

    fn is_hidden(&self, id: NodeId) -> bool {
        self.style(id).display == Display::None
            || self.html_tag(id) == Some("input")
                && self
                    .doc
                    .attr(id, "type")
                    .is_some_and(|kind| kind.eq_ignore_ascii_case("hidden"))
    }

    fn enter(&mut self, id: NodeId, depth: usize) -> bool {
        if depth > MAX_DEPTH
            || self.visits >= MAX_VISITS
            || self.hits_created >= MAX_VISITS
            || self.commands_created + self.open_clips >= MAX_COMMANDS.saturating_sub(8)
            || self.doc.nodes.get(id).is_none()
        {
            return false;
        }
        self.visits += 1;
        true
    }

    fn push_hit(&mut self, hit: HitRegion) {
        if self.hits_created < MAX_VISITS {
            self.hits_created += 1;
            self.hits.push(hit);
        }
    }

    fn push(&mut self, mut command: DrawCommand) -> bool {
        // Closing every emitted clip is part of the allocation, including clips
        // around a fragment whose descendants consume the remaining quota.
        match command {
            DrawCommand::PushClip { .. }
            | DrawCommand::PushFixed
            | DrawCommand::PushOpacity { .. } => {
                if self.commands_created + self.open_clips + 2 > MAX_COMMANDS {
                    return false;
                }
                self.open_clips += 1;
            }
            DrawCommand::PopClip | DrawCommand::PopFixed | DrawCommand::PopOpacity => {
                if self.open_clips == 0 {
                    return false;
                }
                self.open_clips -= 1;
            }
            _ if self.commands_created + self.open_clips >= MAX_COMMANDS => return false,
            _ => {}
        }
        // Tokenization bounds source work; this separate budget also covers
        // generated markers, controls, alt text, and expanded whitespace.
        if let DrawCommand::Text { text, .. } = &mut command {
            let mut end = 0;
            let mut glyphs = 0;
            for (offset, character) in text.char_indices().take(self.emitted_glyphs_left) {
                end = offset + character.len_utf8();
                glyphs += 1;
            }
            text.truncate(end);
            self.emitted_glyphs_left -= glyphs;
            if text.is_empty() {
                return false;
            }
        }
        self.commands_created += 1;
        self.commands.push(command);
        self.owners.push(self.paint_owner);
        true
    }

    fn is_float(&self, id: NodeId) -> bool {
        self.style(id).float != "none"
            && !matches!(self.style(id).position.as_str(), "absolute" | "fixed")
            && matches!(self.doc.nodes[id].kind, NodeKind::Element(_))
    }

    fn establishes_bfc(&self, id: NodeId) -> bool {
        let style = self.style(id);
        id == self.doc.root
            || self.fragment_root == Some(id)
            || self.doc.nodes[id].parent == Some(self.doc.root)
            || self.is_float(id)
            || style.flow_root
            || matches!(
                style.display,
                Display::InlineBlock | Display::Flex | Display::Grid
            )
            || matches!(style.overflow.as_str(), "hidden" | "auto" | "scroll")
            || matches!(style.position.as_str(), "absolute" | "fixed")
            || matches!(self.layout_tag(id), "table" | "td" | "th")
    }

    fn clear_y(&self, id: NodeId, y: f32) -> f32 {
        y.max(match self.style(id).clear.as_str() {
            "left" => self.floats.left_bottom,
            "right" => self.floats.right_bottom,
            "both" => self.floats.bottom,
            _ => y,
        })
    }

    /// Margin boxes exclude complete line bands, not merely a text baseline.
    /// On budget exhaustion conservatively move below all existing floats.
    fn float_band(&mut self, x: f32, y: f32, width: f32, height: f32) -> FloatBand {
        let mut band = FloatBand {
            left: x,
            right: x + width,
            next: None,
        };
        if y >= self.floats.bottom {
            return band;
        }
        if self.float_work_left < self.floats.boxes.len() {
            self.float_work_left = 0;
            band.right = x;
            band.next = Some(self.floats.bottom);
            return band;
        }
        self.float_work_left -= self.floats.boxes.len();
        for float in &self.floats.boxes {
            let r = float.outer;
            if r.height <= 0.0 || r.y >= y + height.max(0.01) || r.y + r.height <= y {
                continue;
            }
            // A nested containing block need not overlap a float's horizontal band.
            if r.x >= x + width || r.x + r.width <= x {
                continue;
            }
            if float.right {
                band.right = band.right.min(r.x);
            } else {
                band.left = band.left.max(r.x + r.width);
            }
            let bottom = r.y + r.height;
            band.next = Some(band.next.map_or(bottom, |old| old.min(bottom)));
        }
        band.left = band.left.clamp(x, x + width);
        band.right = band.right.clamp(x, x + width);
        band
    }

    fn margins(&self, id: NodeId, reference: f32) -> Sides {
        let margin = &self.style(id).margin;
        Sides {
            top: resolve(margin.top, reference).unwrap_or(0.0),
            right: resolve(margin.right, reference).unwrap_or(0.0),
            bottom: resolve(margin.bottom, reference).unwrap_or(0.0),
            left: resolve(margin.left, reference).unwrap_or(0.0),
        }
    }

    fn padding(&self, id: NodeId, reference: f32) -> Sides {
        let padding = &self.style(id).padding;
        Sides {
            top: resolve(padding.top, reference).unwrap_or(0.0).max(0.0),
            right: resolve(padding.right, reference).unwrap_or(0.0).max(0.0),
            bottom: resolve(padding.bottom, reference).unwrap_or(0.0).max(0.0),
            left: resolve(padding.left, reference).unwrap_or(0.0).max(0.0),
        }
    }

    fn borders(&self, id: NodeId) -> Sides {
        let border = &self.style(id).border_width;
        Sides {
            top: extent(border.top),
            right: extent(border.right),
            bottom: extent(border.bottom),
            left: extent(border.left),
        }
    }

    fn width_for(&self, id: NodeId, available: f32, containing_width: f32) -> f32 {
        let style = self.style(id);
        let padding = self.padding(id, containing_width);
        let border = self.borders(id);
        let extra = padding.horizontal() + border.horizontal();
        let css_to_border = if style.box_sizing == "border-box" {
            0.0
        } else {
            extra
        };
        let tag = self.layout_tag(id);
        let mut width = resolve(style.width, containing_width)
            .map(|value| value + css_to_border)
            .unwrap_or_else(|| match tag {
                "img" | "svg" | "canvas" | "video" => {
                    let natural = self.natural_size(id);
                    let value = self.attr_number(id, "width").unwrap_or_else(|| {
                        resolve(style.height, self.viewport.height)
                            .or_else(|| self.attr_number(id, "height"))
                            .map(|height| height * natural.width / natural.height.max(1.0))
                            .unwrap_or(natural.width)
                    });
                    value + extra
                }
                "input"
                    if self.doc.attr(id, "type").is_some_and(|kind| {
                        kind.eq_ignore_ascii_case("checkbox") || kind.eq_ignore_ascii_case("radio")
                    }) =>
                {
                    16.0 + extra
                }
                "input" => {
                    self.attr_number(id, "size").unwrap_or(20.0) * font_size(style) * 0.55
                        + 12.0
                        + extra
                }
                "textarea" => {
                    self.attr_number(id, "cols").unwrap_or(20.0) * font_size(style) * 0.6
                        + 12.0
                        + extra
                }
                "select" => self.intrinsic_width(id, containing_width).max(80.0) + 22.0 + extra,
                _ => available,
            });
        if let Some(max) = resolve(style.max_width, containing_width) {
            width = width.min(max + css_to_border);
        }
        if let Some(min) = resolve(style.min_width, containing_width) {
            width = width.max(min + css_to_border);
        }
        extent(width).max(extra)
    }

    fn layout_box(
        &mut self,
        id: NodeId,
        x: f32,
        y: f32,
        available: f32,
        forced_width: Option<f32>,
        depth: usize,
    ) -> Size {
        self.layout_box_sized(id, x, y, available, (forced_width, None), depth)
    }

    fn layout_box_sized(
        &mut self,
        id: NodeId,
        mut x: f32,
        mut y: f32,
        available: f32,
        forced: (Option<f32>, Option<f32>),
        depth: usize,
    ) -> Size {
        if !self.enter(id, depth) || self.is_hidden(id) {
            return Size::default();
        }
        let style = self.style(id).clone();
        let previous_owner = self.paint_owner;
        let previous_flow_height = self.flow_height_reference;
        self.paint_owner = PaintOwner {
            node: id,
            background: true,
        };
        if style.position == "relative" {
            x += resolve(style.left, available)
                .unwrap_or_else(|| -resolve(style.right, available).unwrap_or(0.0));
            y += grid_length(style.top, previous_flow_height)
                .unwrap_or_else(|| -grid_length(style.bottom, previous_flow_height).unwrap_or(0.0));
        }
        let padding = self.padding(id, available);
        let border = self.borders(id);
        let width = forced
            .0
            .map(extent)
            .unwrap_or_else(|| self.width_for(id, available, available));
        let inner_width = (width - padding.horizontal() - border.horizontal()).max(0.0);
        let inner_x = x + border.left + padding.left;
        let inner_y = y + border.top + padding.top;
        let outer_floats = self.establishes_bfc(id).then(|| {
            std::mem::replace(
                &mut self.floats,
                FloatContext {
                    bottom: inner_y,
                    left_bottom: inner_y,
                    right_bottom: inner_y,
                    source_top: inner_y,
                    ..FloatContext::default()
                },
            )
        });
        let float_paint_start = self.floats.paint.len();
        let extras = padding.vertical() + border.vertical();
        let css_to_border = if style.box_sizing == "border-box" {
            0.0
        } else {
            extras
        };
        let height_reference = self
            .grid_height_reference
            .filter(|(node, _)| *node == id)
            .map(|(_, reference)| reference)
            .unwrap_or_else(|| {
                if self.doc.nodes[id].parent == Some(self.doc.root) {
                    Some(self.viewport.height)
                } else {
                    previous_flow_height
                }
            });
        let definite_height = forced.1.or_else(|| {
            grid_length(style.height, height_reference).map(|height| {
                let (min, max) =
                    self.flex_limits(id, available, height_reference.unwrap_or(0.0), true);
                (height + css_to_border).clamp(min, max)
            })
        });
        self.flow_height_reference = definite_height.map(|height| (height - extras).max(0.0));
        let paint_start = self.commands.len();
        // Reserve paint slots before descendants, then fill in the height.
        for _ in 0..5 {
            self.push(DrawCommand::Rect {
                rect: rect(x, y, 0.0, 0.0),
                color: rgba(0, 0, 0, 0),
                radius: 0.0,
            });
        }
        self.paint_owner.background = false;
        let hit_start = self.hits.len();
        self.push_hit(HitRegion {
            fixed: false,
            node: id,
            rect: rect(x, y, width, 0.0),
        });
        let clip_index = if matches!(style.overflow.as_str(), "hidden" | "clip") {
            let index = self.commands.len();
            let opened = self.push(DrawCommand::PushClip {
                rect: Rect::default(),
            });
            opened.then_some(index)
        } else {
            None
        };
        let tag = self.layout_tag(id).to_owned();
        let children = self.doc.nodes[id].children.clone();
        let mut natural_height = if matches!(tag.as_str(), "img" | "svg" | "canvas" | "video") {
            self.paint_replaced(id, &tag, inner_x, inner_y, inner_width)
        } else if matches!(tag.as_str(), "input" | "textarea" | "select") {
            self.paint_control(id, &tag, inner_x, inner_y, inner_width)
        } else if matches!(self.doc.nodes[id].kind, NodeKind::Text(_)) {
            self.layout_inline(
                &[id],
                inner_x,
                inner_y,
                inner_width,
                &style.text_align,
                depth + 1,
            )
        } else if style.display == Display::Flex {
            self.layout_flex(
                &children,
                inner_x,
                inner_y,
                FlexConstraints {
                    width: inner_width,
                    height: definite_height.map(|h| (h - extras).max(0.0)),
                    min_height: grid_length(style.min_height, height_reference)
                        .map(|h| extent(h + css_to_border - extras))
                        .unwrap_or(0.0),
                    max_height: grid_length(style.max_height, height_reference)
                        .map(|h| extent(h + css_to_border - extras)),
                },
                &style,
                depth + 1,
            )
        } else if style.display == Display::Grid {
            self.layout_grid(
                &children,
                inner_x,
                inner_y,
                Size {
                    width: inner_width,
                    height: definite_height
                        .map(|h| (h - extras).max(0.0))
                        .unwrap_or(-1.0),
                },
                &style,
                depth + 1,
            )
        } else if tag == "table" {
            self.layout_table(&children, inner_x, inner_y, inner_width, depth + 1)
        } else {
            self.layout_flow(
                &children,
                inner_x,
                inner_y,
                inner_width,
                &style.text_align,
                depth + 1,
            )
        };
        if outer_floats.is_some() {
            natural_height = natural_height.max(self.floats.bottom - inner_y);
            // Float layers cover in-flow block backgrounds. Text normally does
            // not intersect them because line boxes use the exclusion bands.
            for paint in std::mem::take(&mut self.floats.paint) {
                self.append_fragment(paint.fragment, paint.x, paint.y);
            }
        }
        let mut height = definite_height.unwrap_or(natural_height + extras);
        if forced.1.is_none()
            && let Some(max) = grid_length(style.max_height, height_reference)
        {
            height = height.min(max + css_to_border);
        }
        if forced.1.is_none()
            && let Some(min) = grid_length(style.min_height, height_reference)
        {
            height = height.max(min + css_to_border);
        }
        height = extent(height).max(extras);
        // Canvas background propagation is emitted once before the root box.
        let paint_height = height;
        let background = if self.canvas_background_node == Some(id) {
            Color::TRANSPARENT
        } else {
            style.background_color
        };
        let border_color = style.border_color;
        let mut replacements = [
            DrawCommand::Rect {
                rect: rect(x, y, width, paint_height),
                color: background,
                radius: style.border_radius,
            },
            DrawCommand::Rect {
                rect: rect(x, y, width, border.top),
                color: border_color,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: rect(x + width - border.right, y, border.right, height),
                color: border_color,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: rect(x, y + height - border.bottom, width, border.bottom),
                color: border_color,
                radius: 0.0,
            },
            DrawCommand::Rect {
                rect: rect(x, y, border.left, height),
                color: border_color,
                radius: 0.0,
            },
        ];
        if self.rounded_border(id) {
            replacements[0] = DrawCommand::Rect {
                rect: rect(x, y, width, paint_height),
                color: border_color,
                radius: style.border_radius,
            };
            replacements[1] = DrawCommand::Rect {
                rect: rect(
                    x + border.left,
                    y + border.top,
                    width - border.horizontal(),
                    paint_height - border.vertical(),
                ),
                color: background,
                radius: (style.border_radius - border.top).max(0.0),
            };
            for command in &mut replacements[2..] {
                *command = DrawCommand::Rect {
                    rect: rect(x, y, 0.0, 0.0),
                    color: Color::TRANSPARENT,
                    radius: 0.0,
                };
            }
        }
        for (offset, command) in replacements.into_iter().enumerate() {
            if let Some(slot) = self.commands.get_mut(paint_start + offset) {
                *slot = command;
            }
        }
        self.hits[hit_start].rect.height = height;
        self.boxes.push(BoxGeometry {
            node: id,
            border: rect(x, y, width, height),
            padding: rect(
                x + border.left,
                y + border.top,
                width - border.horizontal(),
                height - border.vertical(),
            ),
            content: rect(inner_x, inner_y, inner_width, height - extras),
            fixed: false,
            inline: false,
        });
        if tag == "li" && style.list_style_type != "none" {
            self.paint_list_marker(id, inner_x, inner_y, &style);
        }
        if let Some(index) = clip_index {
            let clip = rect(
                x + border.left,
                y + border.top,
                width - border.horizontal(),
                height - border.vertical(),
            );
            if let Some(command) = self.commands.get_mut(index) {
                *command = DrawCommand::PushClip { rect: clip };
            }
            self.push(DrawCommand::PopClip);
            for hit in &mut self.hits[hit_start + 1..] {
                if !hit.fixed {
                    hit.rect = hit.rect.intersect(clip);
                }
            }
            // overflow:clip does not establish a formatting context. Floats
            // escape its flow height, but their deferred paint remains clipped.
            for paint in self.floats.paint.iter_mut().skip(float_paint_start) {
                let local = rect(clip.x - paint.x, clip.y - paint.y, clip.width, clip.height);
                if self.commands_created + self.open_clips + 2 <= MAX_COMMANDS {
                    self.commands_created += 2;
                    paint
                        .fragment
                        .commands
                        .insert(0, DrawCommand::PushClip { rect: local });
                    paint.fragment.commands.push(DrawCommand::PopClip);
                    paint.fragment.owners.insert(0, self.paint_owner);
                    paint.fragment.owners.push(self.paint_owner);
                    for hit in &mut paint.fragment.hits {
                        if !hit.fixed {
                            hit.rect = hit.rect.intersect(local);
                        }
                    }
                } else {
                    paint.fragment.commands.clear();
                    paint.fragment.owners.clear();
                    paint.fragment.hits.clear();
                    paint.fragment.boxes.clear();
                    paint.fragment.positioned.clear();
                }
            }
        }
        if let Some(parent) = outer_floats {
            self.floats = parent;
        }
        self.paint_owner = previous_owner;
        self.flow_height_reference = previous_flow_height;
        Size { width, height }
    }

    fn layout_flow(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        align: &str,
        depth: usize,
    ) -> f32 {
        let mut cursor = y;
        let mut previous_bottom = 0.0;
        let mut inline = Vec::new();
        for &child in children {
            let style = self.style(child);
            if self.is_hidden(child) {
                continue;
            }
            if style.position == "absolute" || style.position == "fixed" {
                self.enqueue_positioned(child, x, cursor + previous_bottom, depth);
                continue;
            }
            if self.is_float(child) {
                inline.push(child);
                continue;
            }
            if matches!(
                style.display,
                Display::Block | Display::Flex | Display::Grid
            ) {
                if !inline.is_empty() {
                    let h = self.layout_inline(
                        &inline,
                        x,
                        cursor + previous_bottom,
                        width,
                        align,
                        depth,
                    );
                    if h > 0.0 {
                        cursor += previous_bottom + h;
                        previous_bottom = 0.0;
                    }
                    inline.clear();
                }
                let margin = self.margins(child, width);
                cursor += collapsed_margin(previous_bottom, margin.top);
                cursor = self.clear_y(child, cursor);
                self.floats.source_top = self.floats.source_top.max(cursor);
                let available = (width - margin.horizontal()).max(0.0);
                let mut child_width = self.width_for(child, available, width);
                let child_size = if !self.floats.boxes.is_empty()
                    && (self.establishes_bfc(child)
                        || matches!(self.layout_tag(child), "img" | "svg" | "canvas" | "video"))
                {
                    let band = self.float_band(x, cursor, width, 0.01);
                    if matches!(self.style(child).width, Length::Auto)
                        && band.width() > margin.horizontal()
                    {
                        child_width =
                            self.width_for(child, band.width() - margin.horizontal(), width);
                    }
                    let fragment = self.fragment(child, width, child_width, depth);
                    let size = fragment.size;
                    let band = loop {
                        let band = self.float_band(x, cursor, width, size.height);
                        if size.width + margin.horizontal() <= band.width() || band.next.is_none() {
                            break band;
                        }
                        cursor = band.next.unwrap().max(cursor);
                    };
                    let child_x =
                        band.left + self.block_left(child, band.width(), child_width, margin);
                    self.append_fragment(fragment, child_x, cursor);
                    size
                } else {
                    let child_x = x + self.block_left(child, width, child_width, margin);
                    self.layout_box(child, child_x, cursor, width, Some(child_width), depth)
                };
                cursor += child_size.height;
                previous_bottom = margin.bottom;
            } else {
                inline.push(child);
            }
            if cursor >= MAX_EXTENT || self.visits >= MAX_VISITS {
                break;
            }
        }
        if !inline.is_empty() {
            let h = self.layout_inline(&inline, x, cursor + previous_bottom, width, align, depth);
            if h > 0.0 {
                cursor += previous_bottom + h;
                previous_bottom = 0.0;
            }
        }
        extent(cursor - y + previous_bottom)
    }

    fn resolve_positioned(&mut self) {
        let mut geometry = vec![None; self.doc.nodes.len()];
        merge_geometry(&mut geometry, &self.boxes);
        let mut processed = vec![false; self.doc.nodes.len()];
        let mut cursor = 0;
        while cursor < self.positioned.len() && grid_charge(&mut self.position_work_left, 1) {
            let job = self.positioned[cursor];
            cursor += 1;
            if processed[job.node] {
                continue;
            }
            processed[job.node] = true;
            let style = self.style(job.node).clone();
            let viewport_fixed = style.position == "fixed";
            let mut containing = None;
            let mut ancestor = self.doc.nodes[job.node].parent;
            let mut depth = 0;
            if !viewport_fixed {
                while let Some(id) = ancestor {
                    if depth >= MAX_DEPTH || !grid_charge(&mut self.position_work_left, 1) {
                        break;
                    }
                    depth += 1;
                    if self.style(id).position != "static" {
                        containing = Some(id);
                        break;
                    }
                    ancestor = self.doc.nodes[id].parent;
                }
            }
            let containing_geometry = containing.and_then(|id| geometry[id]);
            if containing.is_some() && containing_geometry.is_none() {
                continue;
            }
            let cb = containing_geometry
                .map(|g: BoxGeometry| g.padding)
                .unwrap_or(rect(0.0, 0.0, self.viewport.width, self.viewport.height));
            let fixed = viewport_fixed || containing_geometry.is_some_and(|g| g.fixed);
            let padding = self.padding(job.node, cb.width);
            let border = self.borders(job.node);
            let extra_x = padding.horizontal() + border.horizontal();
            let extra_y = padding.vertical() + border.vertical();
            let border_box = style.box_sizing == "border-box";
            let replaced = matches!(
                self.layout_tag(job.node),
                "img" | "svg" | "canvas" | "video" | "input" | "textarea" | "select"
            );
            let (min_width, max_width) = self.flex_limits(job.node, cb.width, cb.width, false);
            let (min_height, max_height) = self.flex_limits(job.node, cb.width, cb.height, true);
            let left = resolve(style.left, cb.width);
            let right = resolve(style.right, cb.width);
            let margin_left = resolve(style.margin.left, cb.width);
            let margin_right = resolve(style.margin.right, cb.width);
            let (preferred_min, preferred_max) = self.preferred_widths(job.node, cb.width, 0);
            let remaining = cb.width
                - left.unwrap_or(0.0)
                - right.unwrap_or(0.0)
                - margin_left.unwrap_or(0.0)
                - margin_right.unwrap_or(0.0);
            let (dx, width) = positioned_axis(PositionedAxis {
                containing: cb.width,
                start: left,
                end: right,
                size: resolve(style.width, cb.width)
                    .map(|v| v + if border_box { 0.0 } else { extra_x })
                    .or_else(|| replaced.then_some(preferred_max)),
                natural: preferred_max.min(remaining.max(preferred_min)),
                min: min_width,
                max: max_width,
                margin_start: margin_left,
                margin_end: margin_right,
                static_start: job.x - cb.x,
                horizontal: true,
            });
            let top = resolve(style.top, cb.height);
            let bottom = resolve(style.bottom, cb.height);
            let specified_height = resolve(style.height, cb.height)
                .map(|v| v + if border_box { 0.0 } else { extra_y });
            let height_axis = PositionedAxis {
                containing: cb.height,
                start: top,
                end: bottom,
                size: specified_height,
                natural: 0.0,
                min: min_height,
                max: max_height,
                margin_start: resolve(style.margin.top, cb.width),
                margin_end: resolve(style.margin.bottom, cb.width),
                static_start: job.y - cb.y,
                horizontal: false,
            };
            let forced_height = (specified_height.is_some()
                || !replaced && top.is_some() && bottom.is_some())
            .then(|| positioned_axis(height_axis).1);
            let saved_height = self
                .grid_height_reference
                .replace((job.node, Some(cb.height)));
            let mut fragment =
                self.fragment_sized(job.node, cb.width, width, forced_height, job.depth);
            self.grid_height_reference = saved_height;
            let (dy, _) = positioned_axis(PositionedAxis {
                natural: fragment.size.height,
                size: specified_height.or_else(|| replaced.then_some(fragment.size.height)),
                ..height_axis
            });
            translate_fragment(&mut fragment, cb.x + dx, cb.y + dy);
            // Absolute descendants are clipped by their containing block and its
            // ancestors, not by intervening static overflow boxes (CSS 2.2 11.1).
            let mut clips = Vec::new();
            if !viewport_fixed {
                let mut ancestor = containing;
                while let Some(id) = ancestor {
                    if clips.len() >= MAX_DEPTH || !grid_charge(&mut self.position_work_left, 1) {
                        break;
                    }
                    if let Some(g) = geometry[id] {
                        if !g.inline
                            && matches!(self.style(id).overflow.as_str(), "hidden" | "clip")
                        {
                            clips.push(g.padding);
                        }
                        if self.style(id).position == "fixed" {
                            break;
                        }
                    }
                    ancestor = self.doc.nodes[id].parent;
                }
            }
            let scope_cost = (clips.len() + usize::from(fixed)) * 2;
            if scope_cost + self.commands_created + self.open_clips > MAX_COMMANDS {
                continue;
            }
            self.commands_created += scope_cost;
            for hit in &mut fragment.hits {
                if !hit.fixed {
                    for clip in &clips {
                        hit.rect = hit.rect.intersect(*clip);
                    }
                }
                hit.fixed |= fixed;
            }
            for g in &mut fragment.boxes {
                g.fixed |= fixed;
            }
            for job in &mut fragment.positioned {
                job.fixed |= fixed;
            }
            let owner = PaintOwner {
                node: job.node,
                background: false,
            };
            let mut commands = Vec::with_capacity(fragment.commands.len() + scope_cost);
            let mut owners = Vec::with_capacity(commands.capacity());
            if fixed {
                commands.push(DrawCommand::PushFixed);
                owners.push(owner);
            }
            for clip in clips.iter().rev() {
                commands.push(DrawCommand::PushClip { rect: *clip });
                owners.push(owner);
            }
            commands.append(&mut fragment.commands);
            owners.append(&mut fragment.owners);
            for _ in &clips {
                commands.push(DrawCommand::PopClip);
                owners.push(owner);
            }
            if fixed {
                commands.push(DrawCommand::PopFixed);
                owners.push(owner);
            }
            fragment.commands = commands;
            fragment.owners = owners;
            merge_geometry(&mut geometry, &fragment.boxes);
            self.append_fragment(fragment, 0.0, 0.0);
        }
    }

    fn restack(&mut self) {
        if self.doc.nodes.get(self.doc.root).is_none() {
            self.commands.clear();
            self.hits.clear();
            return;
        }
        // Build CSS paint groups from DOM ancestry, not the order in which
        // deferred geometry happened to be measured. A positioned auto-z group
        // is atomic for its ordinary descendants; real child stacking contexts
        // still participate in the nearest real ancestor context.
        #[derive(Clone, Copy)]
        enum Entry {
            Owner(NodeId, bool),
            Group(usize),
        }
        struct Group {
            entries: Vec<((u8, i32, usize), Entry)>,
        }
        let mut groups = vec![Group {
            entries: Vec::new(),
        }];
        #[derive(Clone, Copy)]
        struct Opacity {
            parent: usize,
            value: f32,
            depth: usize,
        }
        let mut opacities = vec![Opacity {
            parent: 0,
            value: 1.0,
            depth: 0,
        }];
        let mut owner_opacity = vec![0usize; self.doc.nodes.len()];
        let mut ranks = vec![[usize::MAX; 2]; self.doc.nodes.len()];
        let mut traversal = vec![(self.doc.root, 0usize, 0usize, 0usize)];
        let mut seen = vec![false; self.doc.nodes.len()];
        let mut order = 0usize;
        while let Some((id, parent_group, real_parent, inherited_opacity)) = traversal.pop() {
            if seen[id] || !grid_charge(&mut self.position_work_left, 1) {
                continue;
            }
            seen[id] = true;
            let style = self.style(id);
            if self.is_hidden(id) {
                continue;
            }
            let opacity = finite(style.opacity, 1.0).clamp(0.0, 1.0);
            let opacity_chain = if opacity < 1.0 {
                opacities.push(Opacity {
                    parent: inherited_opacity,
                    value: opacity,
                    depth: opacities[inherited_opacity].depth + 1,
                });
                opacities.len() - 1
            } else {
                inherited_opacity
            };
            owner_opacity[id] = opacity_chain;
            let positioned = style.position != "static";
            let item = self.doc.nodes[id]
                .parent
                .is_some_and(|p| matches!(self.style(p).display, Display::Flex | Display::Grid));
            let real = id == self.doc.root
                || opacity < 1.0
                || style.position == "fixed"
                || style.z_index.is_some() && (positioned || item);
            let floating = self.is_float(id);
            let pseudo = positioned || floating || item || style.display == Display::InlineBlock;
            let mut group = parent_group;
            let mut real_group = real_parent;
            if id != self.doc.root && (real || pseudo) {
                group = groups.len();
                groups.push(Group {
                    entries: Vec::new(),
                });
                let z = if real && (positioned || item) {
                    style.z_index.unwrap_or(0)
                } else {
                    0
                };
                let phase = if real {
                    if z < 0 {
                        1
                    } else if z > 0 {
                        6
                    } else {
                        5
                    }
                } else if positioned {
                    5
                } else if floating {
                    3
                } else {
                    4
                };
                let parent = if real { real_parent } else { parent_group };
                groups[parent]
                    .entries
                    .push(((phase, z, order), Entry::Group(group)));
                if real {
                    real_group = group;
                }
            }
            let owns_group = id == self.doc.root || real || pseudo;
            let background_phase = if owns_group {
                0
            } else if style.display == Display::Inline {
                4
            } else {
                2
            };
            groups[group]
                .entries
                .push(((background_phase, 0, order), Entry::Owner(id, true)));
            groups[group]
                .entries
                .push(((4, 0, order), Entry::Owner(id, false)));
            order += 1;
            let mut children = self.doc.nodes[id].children.clone();
            if matches!(style.display, Display::Flex | Display::Grid) {
                children.sort_by_key(|child| self.style(*child).order);
            }
            traversal.extend(
                children
                    .into_iter()
                    .rev()
                    .map(|child| (child, group, real_group, opacity_chain)),
            );
        }
        for group in &mut groups {
            group.entries.sort_by_key(|entry| entry.0);
        }
        let mut pending = vec![(0usize, 0usize)];
        let mut next_rank = 0usize;
        while let Some((group, index)) = pending.pop() {
            let Some((_, entry)) = groups[group].entries.get(index).copied() else {
                continue;
            };
            pending.push((group, index + 1));
            match entry {
                Entry::Group(child) => pending.push((child, 0)),
                Entry::Owner(node, background) => {
                    ranks[node][usize::from(!background)] = next_rank;
                    next_rank += 1;
                }
            }
        }
        #[derive(Clone, Copy)]
        struct Chain {
            parent: usize,
            rect: Rect,
            depth: usize,
        }
        struct Atom {
            command: DrawCommand,
            rank: usize,
            chain: usize,
            fixed: bool,
            opacity: usize,
            sequence: usize,
        }
        let mut chains = vec![Chain {
            parent: 0,
            rect: Rect::default(),
            depth: 0,
        }];
        let (mut chain, mut fixed) = (0usize, false);
        let mut scopes = Vec::new();
        let mut atoms = Vec::new();
        for (sequence, (command, owner)) in std::mem::take(&mut self.commands)
            .into_iter()
            .zip(std::mem::take(&mut self.owners))
            .enumerate()
        {
            match command {
                DrawCommand::PushClip { rect } => {
                    scopes.push((chain, fixed, false));
                    chains.push(Chain {
                        parent: chain,
                        rect,
                        depth: chains[chain].depth + 1,
                    });
                    chain = chains.len() - 1;
                }
                DrawCommand::PushFixed => {
                    scopes.push((chain, fixed, true));
                    chain = 0;
                    fixed = true;
                }
                DrawCommand::PopClip | DrawCommand::PopFixed => {
                    if let Some((old_chain, old_fixed, _)) = scopes.pop() {
                        chain = old_chain;
                        fixed = old_fixed;
                    }
                }
                _ => {
                    let visible = match &command {
                        DrawCommand::Rect { rect, color, .. } => {
                            color.a > 0 && rect.width > 0.0 && rect.height > 0.0
                        }
                        DrawCommand::Text { text, color, .. } => !text.is_empty() && color.a > 0,
                        _ => true,
                    };
                    if visible {
                        atoms.push(Atom {
                            command,
                            rank: ranks[owner.node][usize::from(!owner.background)],
                            chain,
                            fixed,
                            opacity: owner_opacity[owner.node],
                            sequence,
                        });
                    }
                }
            }
        }
        atoms.sort_by_key(|atom| (atom.rank, atom.sequence));
        let (mut current_chain, mut current_fixed, mut current_opacity) = (0usize, false, 0usize);
        let mut cutoff = usize::MAX;
        for atom in atoms {
            if !grid_charge(&mut self.position_work_left, 1) {
                cutoff = atom.rank;
                break;
            }
            // Opacity scopes enclose complete stacking groups. Coordinate/clip
            // transitions happen inside them, allowing fixed children to escape
            // ancestor overflow without escaping the group's opacity.
            let changing_opacity = current_opacity != atom.opacity;
            let mut common_opacity = current_opacity;
            let mut target_opacity = atom.opacity;
            let mut traversal_cost = 0;
            while opacities[common_opacity].depth > opacities[target_opacity].depth {
                common_opacity = opacities[common_opacity].parent;
                traversal_cost += 1;
            }
            while opacities[target_opacity].depth > opacities[common_opacity].depth {
                target_opacity = opacities[target_opacity].parent;
                traversal_cost += 1;
            }
            while common_opacity != target_opacity {
                common_opacity = opacities[common_opacity].parent;
                target_opacity = opacities[target_opacity].parent;
                traversal_cost += 2;
            }
            let mut opacity_path = Vec::new();
            let mut target_opacity = atom.opacity;
            while target_opacity != common_opacity {
                opacity_path.push(target_opacity);
                target_opacity = opacities[target_opacity].parent;
            }
            let mut common = if !changing_opacity && current_fixed == atom.fixed {
                current_chain
            } else {
                0
            };
            let mut target = atom.chain;
            while chains[common].depth > chains[target].depth {
                common = chains[common].parent;
                traversal_cost += 1;
            }
            while chains[target].depth > chains[common].depth {
                target = chains[target].parent;
                traversal_cost += 1;
            }
            while common != target {
                common = chains[common].parent;
                target = chains[target].parent;
                traversal_cost += 2;
            }
            let mut path = Vec::new();
            let mut target = atom.chain;
            while target != common {
                path.push(target);
                target = chains[target].parent;
            }
            let closes = chains[current_chain].depth - chains[common].depth;
            let fixed_changes = if !changing_opacity && current_fixed == atom.fixed {
                0
            } else {
                usize::from(current_fixed) + usize::from(atom.fixed)
            };
            let opacity_closes = opacities[current_opacity].depth - opacities[common_opacity].depth;
            let final_open =
                chains[atom.chain].depth + usize::from(atom.fixed) + opacities[atom.opacity].depth;
            let cost =
                closes + fixed_changes + path.len() + opacity_closes + opacity_path.len() + 1;
            if final_open > MAX_DEPTH
                || self.commands.len() + cost + final_open > MAX_COMMANDS
                || !grid_charge(&mut self.position_work_left, cost + traversal_cost)
            {
                cutoff = atom.rank;
                break;
            }
            for _ in 0..closes {
                self.commands.push(DrawCommand::PopClip);
            }
            let change_coordinates = changing_opacity || current_fixed != atom.fixed;
            if change_coordinates && current_fixed {
                self.commands.push(DrawCommand::PopFixed);
            }
            for _ in 0..opacity_closes {
                self.commands.push(DrawCommand::PopOpacity);
            }
            for id in opacity_path.into_iter().rev() {
                self.commands.push(DrawCommand::PushOpacity {
                    opacity: opacities[id].value,
                });
            }
            if change_coordinates && atom.fixed {
                self.commands.push(DrawCommand::PushFixed);
            }
            for id in path.into_iter().rev() {
                self.commands.push(DrawCommand::PushClip {
                    rect: chains[id].rect,
                });
            }
            self.commands.push(atom.command);
            current_chain = atom.chain;
            current_fixed = atom.fixed;
            current_opacity = atom.opacity;
        }
        for _ in 0..chains[current_chain].depth {
            self.commands.push(DrawCommand::PopClip);
        }
        if current_fixed {
            self.commands.push(DrawCommand::PopFixed);
        }
        for _ in 0..opacities[current_opacity].depth {
            self.commands.push(DrawCommand::PopOpacity);
        }
        let hit_rank = |hit: &HitRegion| {
            ranks[hit.node][usize::from(matches!(self.doc.nodes[hit.node].kind, NodeKind::Text(_)))]
        };
        self.hits.retain(|hit| hit_rank(hit) < cutoff);
        self.hits.sort_by_key(hit_rank);
    }

    fn block_left(&self, id: NodeId, width: f32, box_width: f32, margin: Sides) -> f32 {
        let style = self.style(id);
        let free = (width - box_width - margin.horizontal()).max(0.0);
        match (
            matches!(style.margin.left, Length::Auto),
            matches!(style.margin.right, Length::Auto),
        ) {
            (true, true) => margin.left + free / 2.0,
            (true, false) => margin.left + free,
            _ => margin.left,
        }
    }

    fn fragment(&mut self, id: NodeId, available: f32, width: f32, depth: usize) -> Fragment {
        self.fragment_sized(id, available, width, None, depth)
    }

    fn fragment_sized(
        &mut self,
        id: NodeId,
        available: f32,
        width: f32,
        height: Option<f32>,
        depth: usize,
    ) -> Fragment {
        let command_start = self.commands.len();
        let hit_start = self.hits.len();
        let box_start = self.boxes.len();
        let positioned_start = self.positioned.len();
        let old_fragment_root = self.fragment_root.replace(id);
        let size = self.layout_box_sized(id, 0.0, 0.0, available, (Some(width), height), depth);
        self.fragment_root = old_fragment_root;
        Fragment {
            size,
            commands: self.commands.split_off(command_start),
            owners: self.owners.split_off(command_start),
            hits: self.hits.split_off(hit_start),
            boxes: self.boxes.split_off(box_start),
            positioned: self.positioned.split_off(positioned_start),
            rounded_border: self.rounded_border(id),
        }
    }

    fn rounded_border(&self, id: NodeId) -> bool {
        let style = self.style(id);
        let border = self.borders(id);
        style.border_radius > 0.0
            && self.canvas_background_node != Some(id)
            && style.background_color.a == 255
            && border.top > 0.0
            && border.top == border.right
            && border.top == border.bottom
            && border.top == border.left
    }

    fn append_fragment(&mut self, mut fragment: Fragment, x: f32, y: f32) {
        translate_fragment(&mut fragment, x, y);
        self.commands.extend(fragment.commands);
        self.owners.extend(fragment.owners);
        self.hits.extend(fragment.hits);
        self.boxes.extend(fragment.boxes);
        self.positioned.extend(fragment.positioned);
    }

    fn enqueue_positioned(&mut self, node: NodeId, x: f32, y: f32, depth: usize) {
        if self.positioned.len() < MAX_VISITS && grid_charge(&mut self.position_work_left, 1) {
            self.positioned.push(PositionJob {
                node,
                x,
                y,
                fixed: false,
                depth,
            });
        }
    }

    fn collect_inline(
        &mut self,
        id: NodeId,
        owner: NodeId,
        width: f32,
        depth: usize,
        output: &mut Vec<InlineItem>,
    ) {
        if !self.enter(id, depth) || self.is_hidden(id) {
            return;
        }
        if matches!(self.style(id).position.as_str(), "absolute" | "fixed") {
            output.push(InlineItem {
                node: id,
                owner: id,
                kind: InlineKind::Positioned(id),
                width: 0.0,
                height: 0.0,
                preserve: false,
            });
            return;
        }
        if self.is_float(id) {
            output.push(InlineItem {
                node: id,
                owner: id,
                kind: InlineKind::Float(id),
                width: 0.0,
                height: 0.0,
                preserve: false,
            });
            return;
        }
        match &self.doc.nodes[id].kind {
            NodeKind::Text(text) => {
                let text = text.chars().take(self.glyphs_left).collect::<String>();
                self.tokenize(id, owner, &text, output);
            }
            NodeKind::Element(_) => {
                let tag = self.layout_tag(id);
                if tag == "br" {
                    output.push(InlineItem {
                        node: id,
                        owner: id,
                        kind: InlineKind::Break,
                        width: 0.0,
                        height: line_height(self.style(id)),
                        preserve: true,
                    });
                    return;
                }
                if matches!(
                    tag,
                    "img" | "svg" | "canvas" | "video" | "input" | "textarea" | "select"
                ) || self.style(id).display == Display::InlineBlock
                {
                    let margin = self.margins(id, width);
                    let inner_width = if matches!(self.style(id).width, Length::Auto)
                        && !matches!(
                            tag,
                            "img" | "svg" | "canvas" | "video" | "input" | "textarea" | "select"
                        ) {
                        self.intrinsic_width(id, width).min(width)
                    } else {
                        self.width_for(id, width, width)
                    };
                    let fragment = self.fragment(id, width, inner_width, depth + 1);
                    output.push(InlineItem {
                        node: id,
                        owner: id,
                        width: fragment.size.width + margin.horizontal(),
                        height: fragment.size.height + margin.vertical(),
                        kind: InlineKind::Box(fragment, margin),
                        preserve: false,
                    });
                } else {
                    let children = self.doc.nodes[id].children.clone();
                    for child in children {
                        self.collect_inline(child, id, width, depth + 1, output);
                    }
                }
            }
            _ => {
                let children = self.doc.nodes[id].children.clone();
                for child in children {
                    self.collect_inline(child, owner, width, depth + 1, output);
                }
            }
        }
    }

    fn tokenize(&mut self, id: NodeId, owner: NodeId, text: &str, output: &mut Vec<InlineItem>) {
        let style = self.style(id).clone();
        let preserve = matches!(
            style.white_space.as_str(),
            "pre" | "pre-wrap" | "break-spaces"
        );
        let newlines = preserve || style.white_space == "pre-line";
        let mut word = String::new();
        let mut whitespace = String::new();
        let mut last_cr = false;
        for character in text.chars().take(self.glyphs_left) {
            self.glyphs_left = self.glyphs_left.saturating_sub(1);
            let is_cr = character == '\r';
            if character == '\n' && last_cr {
                last_cr = false;
                continue;
            }
            last_cr = is_cr;
            if matches!(character, '\n' | '\r') && newlines {
                self.emit_text(id, owner, &mut word, false, preserve, output);
                self.emit_text(id, owner, &mut whitespace, true, preserve, output);
                output.push(InlineItem {
                    node: id,
                    owner,
                    kind: InlineKind::Break,
                    width: 0.0,
                    height: line_height(&style),
                    preserve,
                });
            } else if character.is_ascii_whitespace() {
                self.emit_text(id, owner, &mut word, false, preserve, output);
                if preserve {
                    if character == '\t' {
                        whitespace.push_str("    ");
                    } else {
                        whitespace.push(' ');
                    }
                } else if whitespace.is_empty() {
                    whitespace.push(' ');
                }
            } else {
                self.emit_text(id, owner, &mut whitespace, true, preserve, output);
                if character != '\0' {
                    word.push(character);
                }
            }
        }
        self.emit_text(id, owner, &mut word, false, preserve, output);
        self.emit_text(id, owner, &mut whitespace, true, preserve, output);
    }

    fn emit_text(
        &self,
        id: NodeId,
        owner: NodeId,
        text: &mut String,
        space: bool,
        preserve: bool,
        output: &mut Vec<InlineItem>,
    ) {
        if text.is_empty() {
            return;
        }
        let style = self.style(id);
        let value = std::mem::take(text);
        let width = self.measure(&value, style);
        output.push(InlineItem {
            node: id,
            owner,
            kind: if space {
                InlineKind::Space(value)
            } else {
                InlineKind::Text(value)
            },
            width,
            height: line_height(style),
            preserve,
        });
    }

    /// Preferred minimum/preferred widths for bounded shrink-to-fit sizing.
    /// Block children start new preferred lines; inline children share a line.
    fn preferred_widths(&self, id: NodeId, available: f32, depth: usize) -> (f32, f32) {
        let work = self.intrinsic_work_left.get();
        if depth > MAX_DEPTH || work == 0 || self.is_hidden(id) {
            return (0.0, 0.0);
        }
        self.intrinsic_work_left.set(work - 1);
        let style = self.style(id);
        let tag = self.layout_tag(id);
        if resolve(style.width, available).is_some()
            || matches!(
                tag,
                "img" | "svg" | "canvas" | "video" | "input" | "textarea" | "select"
            )
        {
            let width = self.width_for(id, available, available);
            return (width, width);
        }
        if let NodeKind::Text(text) = &self.doc.nodes[id].kind {
            let value: String = text
                .chars()
                .take(4096.min(self.intrinsic_work_left.get()))
                .collect();
            self.intrinsic_work_left.set(
                self.intrinsic_work_left
                    .get()
                    .saturating_sub(value.chars().count()),
            );
            let preserve = matches!(
                style.white_space.as_str(),
                "pre" | "pre-wrap" | "break-spaces"
            );
            let mut min = 0.0f32;
            let mut max = 0.0f32;
            for line in value.split(['\n', '\r']) {
                let normalized = if preserve {
                    line.replace('\t', "    ")
                } else {
                    line.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
                };
                max = max.max(self.measure(&normalized, style));
                for word in normalized.split_ascii_whitespace() {
                    min = min.max(self.measure(word, style));
                }
            }
            // Normal newlines collapse to spaces rather than forced breaks.
            if !matches!(
                style.white_space.as_str(),
                "pre" | "pre-wrap" | "pre-line" | "break-spaces"
            ) {
                let normalized = value.split_ascii_whitespace().collect::<Vec<_>>().join(" ");
                max = self.measure(&normalized, style);
                // Keep boundary spaces when adjacent inline descendants contribute.
                if value.starts_with(|c: char| c.is_ascii_whitespace()) {
                    max += self.measure(" ", style);
                }
                if value.ends_with(|c: char| c.is_ascii_whitespace()) && !normalized.is_empty() {
                    max += self.measure(" ", style);
                }
            }
            if matches!(style.white_space.as_str(), "pre" | "nowrap") {
                min = max;
            }
            return (extent(min), extent(max));
        }
        let mut min = 0.0f32;
        let mut max = 0.0f32;
        let mut inline = 0.0f32;
        for &child in &self.doc.nodes[id].children {
            if self.intrinsic_work_left.get() == 0 {
                break;
            }
            if self.is_hidden(child)
                || matches!(self.style(child).position.as_str(), "absolute" | "fixed")
            {
                continue;
            }
            let (child_min, child_max) = self.preferred_widths(child, available, depth + 1);
            let margins = self.margins(child, available).horizontal();
            min = min.max(child_min + margins);
            if matches!(
                self.style(child).display,
                Display::Block | Display::Flex | Display::Grid
            ) || self.layout_tag(child) == "br"
            {
                max = max.max(inline).max(child_max + margins);
                inline = 0.0;
            } else {
                inline += child_max + margins;
            }
        }
        let extra = self.padding(id, available).horizontal() + self.borders(id).horizontal();
        (extent(min + extra), extent(max.max(inline) + extra))
    }

    fn place_float(&mut self, id: NodeId, x: f32, y: f32, width: f32, line: Size, depth: usize) {
        if self.floats_left == 0 || self.visits >= MAX_VISITS || y >= MAX_EXTENT {
            return;
        }
        self.floats_left -= 1;
        let margin = self.margins(id, width);
        let style = self.style(id);
        let right = style.float == "right";
        let mut box_width = self.width_for(id, (width - margin.horizontal()).max(0.0), width);
        if matches!(style.width, Length::Auto)
            && !matches!(
                self.layout_tag(id),
                "img" | "svg" | "canvas" | "video" | "input" | "textarea" | "select"
            )
        {
            let (min, preferred) = self.preferred_widths(id, width, depth);
            let (lower, upper) = self.flex_limits(id, width, width, false);
            box_width = min
                .max(width - margin.horizontal())
                .min(preferred)
                .clamp(lower, upper);
        }
        let fragment = self.fragment(id, width, box_width, depth + 1);
        let outer_width = (fragment.size.width + margin.horizontal()).max(0.0);
        let outer_height = (fragment.size.height + margin.vertical()).max(0.0);
        let mut top = self.clear_y(id, y.max(self.floats.source_top));
        let band = loop {
            if top >= MAX_EXTENT {
                return;
            }
            let band = self.float_band(x, top, width, outer_height);
            let same_line = line.width > 0.0 && top < y + line.height;
            let required = outer_width + if same_line { line.width } else { 0.0 };
            if required <= band.width() || (band.next.is_none() && !same_line) {
                break band;
            }
            // A float that does not fit beside earlier inline content starts no
            // higher than that line's bottom; the earlier text remains on its line.
            top = match (band.next, same_line) {
                (Some(next), true) => next.min(y + line.height).max(top + 0.01),
                (Some(next), false) => next,
                (None, true) => y + line.height,
                (None, false) => break band,
            };
        };
        let left = if right {
            band.right - outer_width
        } else {
            band.left
        };
        self.floats.source_top = self.floats.source_top.max(top);
        let bottom = top + outer_height;
        self.floats.bottom = self.floats.bottom.max(bottom);
        if right {
            self.floats.right_bottom = self.floats.right_bottom.max(bottom);
        } else {
            self.floats.left_bottom = self.floats.left_bottom.max(bottom);
        }
        self.floats.boxes.push(FloatExclusion {
            outer: rect(left, top, outer_width, outer_height),
            right,
        });
        self.floats.paint.push(FloatPaint {
            fragment,
            x: left + margin.left,
            y: top + margin.top,
        });
    }

    fn layout_inline(
        &mut self,
        nodes: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        align: &str,
        depth: usize,
    ) -> f32 {
        let mut tokens = Vec::new();
        for &id in nodes {
            self.collect_inline(id, id, width, depth, &mut tokens);
        }
        let mut line = Vec::new();
        let mut line_width = 0.0;
        let mut line_extent = 0.0f32;
        let mut line_baseline = 0.0f32;
        let mut line_descent = 0.0f32;
        let mut line_max_height = 0.0f32;
        let mut cursor = y;
        let mut pending_break = false;
        for token in tokens {
            if cursor >= MAX_EXTENT {
                break;
            }
            if let InlineKind::Float(id) = token.kind {
                self.place_float(
                    id,
                    x,
                    cursor,
                    width,
                    Size {
                        width: line_width,
                        height: line_extent,
                    },
                    depth,
                );
                continue;
            }
            if matches!(token.kind, InlineKind::Break) {
                let band = self.float_band(x, cursor, width, line_extent.max(token.height));
                cursor += self
                    .paint_line(
                        std::mem::take(&mut line),
                        band.left,
                        cursor,
                        band.width(),
                        align,
                        false,
                    )
                    .max(token.height);
                line_width = 0.0;
                line_extent = 0.0;
                line_baseline = 0.0;
                line_descent = 0.0;
                line_max_height = 0.0;
                pending_break = true;
                continue;
            }
            let space = matches!(token.kind, InlineKind::Space(_));
            if space
                && !token.preserve
                && (line.is_empty()
                    || line
                        .last()
                        .is_some_and(|item: &InlineItem| matches!(item.kind, InlineKind::Space(_))))
            {
                continue;
            }
            let can_wrap = !matches!(
                self.style(token.node).white_space.as_str(),
                "pre" | "nowrap"
            );
            let (baseline, descent) = match token.kind {
                InlineKind::Text(_) | InlineKind::Space(_) => {
                    let size = font_size(self.style(token.node));
                    let leading = (token.height - size * 1.3).max(0.0) / 2.0;
                    (size * 0.95 + leading, size * 0.35 + leading)
                }
                _ => (token.height, 0.0),
            };
            let token_height = token.height.max(baseline + descent);
            let prospective = line_max_height
                .max(token.height)
                .max(line_baseline.max(baseline) + line_descent.max(descent));
            let band = self.float_band(x, cursor, width, prospective);
            if can_wrap && !line.is_empty() && line_width + token.width > band.width() && !space {
                let previous = self.float_band(x, cursor, width, line_extent);
                self.floats.source_top = self.floats.source_top.max(cursor);
                cursor += self.paint_line(
                    std::mem::take(&mut line),
                    previous.left,
                    cursor,
                    previous.width(),
                    align,
                    true,
                );
                line_width = 0.0;
                line_baseline = 0.0;
                line_descent = 0.0;
                line_max_height = 0.0;
            }
            if line.is_empty() && !space {
                loop {
                    let band = self.float_band(x, cursor, width, token_height);
                    if token.width <= band.width() || band.next.is_none() {
                        break;
                    }
                    cursor = band.next.unwrap();
                }
            } else if !can_wrap {
                loop {
                    let band = self.float_band(x, cursor, width, prospective);
                    if line_width + token.width <= band.width() || band.next.is_none() {
                        break;
                    }
                    cursor = band.next.unwrap();
                }
            }
            line_width += token.width;
            line_baseline = line_baseline.max(baseline);
            line_descent = line_descent.max(descent);
            line_max_height = line_max_height.max(token.height);
            line_extent = line_max_height.max(line_baseline + line_descent);
            line.push(token);
            pending_break = false;
        }
        if !line.is_empty() {
            let band = self.float_band(x, cursor, width, line_extent);
            self.floats.source_top = self.floats.source_top.max(cursor);
            cursor += self.paint_line(line, band.left, cursor, band.width(), align, false);
        } else if pending_break {
            // A trailing <br> establishes an empty final line.
            cursor += nodes
                .first()
                .map(|id| line_height(self.style(*id)))
                .unwrap_or(0.0);
        }
        extent(cursor - y)
    }

    fn inline_offset(&mut self, node: NodeId, width: f32) -> (f32, f32) {
        let mut ancestor = self.doc.nodes[node].parent;
        let (mut x, mut y) = (0.0, 0.0);
        for _ in 0..MAX_DEPTH {
            let Some(id) = ancestor else { break };
            if !grid_charge(&mut self.position_work_left, 1) {
                break;
            }
            let style = self.style(id);
            if style.display != Display::Inline {
                break;
            }
            if style.position == "relative" {
                x += resolve(style.left, width)
                    .unwrap_or_else(|| -resolve(style.right, width).unwrap_or(0.0));
                y += grid_length(style.top, self.flow_height_reference).unwrap_or_else(|| {
                    -grid_length(style.bottom, self.flow_height_reference).unwrap_or(0.0)
                });
            }
            ancestor = self.doc.nodes[id].parent;
        }
        (
            finite(x, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT),
            finite(y, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT),
        )
    }

    fn record_inline_boxes(&mut self, node: NodeId, area: Rect) {
        let mut ancestor = self.doc.nodes[node].parent;
        for _ in 0..MAX_DEPTH {
            let Some(id) = ancestor else { break };
            if !grid_charge(&mut self.position_work_left, 1) {
                break;
            }
            if self.style(id).display != Display::Inline {
                break;
            }
            self.boxes.push(BoxGeometry {
                node: id,
                border: area,
                padding: area,
                content: area,
                fixed: false,
                inline: true,
            });
            ancestor = self.doc.nodes[id].parent;
        }
    }

    fn paint_line(
        &mut self,
        mut line: Vec<InlineItem>,
        x: f32,
        y: f32,
        available: f32,
        align: &str,
        wrapped: bool,
    ) -> f32 {
        while line
            .last()
            .is_some_and(|item| matches!(item.kind, InlineKind::Space(_)) && !item.preserve)
        {
            line.pop();
        }
        if line.is_empty() {
            return 0.0;
        }
        let width: f32 = line.iter().map(|item| item.width).sum();
        let baseline = line
            .iter()
            .map(|item| match item.kind {
                InlineKind::Text(_) | InlineKind::Space(_) => {
                    let size = font_size(self.style(item.node));
                    size * 0.95 + (item.height - size * 1.3).max(0.0) / 2.0
                }
                _ => item.height,
            })
            .fold(0.0, f32::max);
        let descent = line
            .iter()
            .filter_map(|item| match item.kind {
                InlineKind::Text(_) | InlineKind::Space(_) => {
                    let size = font_size(self.style(item.node));
                    Some(size * 0.35 + (item.height - size * 1.3).max(0.0) / 2.0)
                }
                _ => None,
            })
            .fold(0.0, f32::max);
        let height = line
            .iter()
            .map(|item| item.height)
            .fold(baseline + descent, f32::max);
        let space_count = line
            .iter()
            .filter(|item| matches!(item.kind, InlineKind::Space(_)))
            .count();
        let extra_space = if align == "justify" && wrapped && space_count > 0 {
            (available - width).max(0.0) / space_count as f32
        } else {
            0.0
        };
        let mut cursor = x + match align {
            "center" => (available - width).max(0.0) * 0.5,
            "right" | "end" => (available - width).max(0.0),
            _ => 0.0,
        };
        let saved_owner = self.paint_owner;
        for item in line {
            if matches!(item.kind, InlineKind::Text(_) | InlineKind::Space(_))
                && self.hits_created >= MAX_VISITS
            {
                cursor += item.width;
                continue;
            }
            let (relative_x, relative_y) = self.inline_offset(item.node, available);
            let visual_x = cursor + relative_x;
            let visual_y = y + relative_y;
            self.paint_owner = PaintOwner {
                node: item.node,
                background: false,
            };
            let style = self.style(item.node).clone();
            let is_space = matches!(item.kind, InlineKind::Space(_));
            match item.kind {
                InlineKind::Text(text) | InlineKind::Space(text) => {
                    let size = font_size(&style);
                    let text_y = visual_y + baseline - size * 0.95;
                    let item_rect = rect(
                        visual_x,
                        visual_y,
                        item.width + if is_space { extra_space } else { 0.0 },
                        height,
                    );
                    self.record_inline_boxes(item.node, item_rect);
                    let owner_style = self.style(item.owner);
                    if owner_style.background_color.a > 0 && owner_style.display == Display::Inline
                    {
                        self.push(DrawCommand::Rect {
                            rect: item_rect,
                            color: owner_style.background_color,
                            radius: owner_style.border_radius,
                        });
                    }
                    self.push(DrawCommand::Text {
                        x: visual_x,
                        y: text_y,
                        text,
                        size,
                        color: style.color,
                        bold: style.font_weight >= 600,
                        italic: style.font_style == "italic" || style.font_style == "oblique",
                        monospace: monospace(&style),
                    });
                    if style.text_decoration.contains("underline") {
                        self.push(DrawCommand::Line {
                            x1: visual_x,
                            y1: text_y + size * 1.12,
                            x2: visual_x + item.width,
                            y2: text_y + size * 1.12,
                            color: style.color,
                            width: (size / 16.0).max(1.0),
                        });
                    }
                    if style.text_decoration.contains("line-through") {
                        self.push(DrawCommand::Line {
                            x1: visual_x,
                            y1: text_y + size * 0.65,
                            x2: visual_x + item.width,
                            y2: text_y + size * 0.65,
                            color: style.color,
                            width: (size / 16.0).max(1.0),
                        });
                    }
                    self.push_hit(HitRegion {
                        fixed: false,
                        node: item.node,
                        rect: item_rect,
                    });
                }
                InlineKind::Box(fragment, margin) => {
                    let offset = match style.vertical_align.as_str() {
                        "top" | "text-top" => 0.0,
                        "middle" => (height - item.height) / 2.0,
                        "bottom" | "text-bottom" => height - item.height,
                        _ => baseline - item.height,
                    };
                    self.record_inline_boxes(
                        item.node,
                        rect(visual_x, visual_y + offset, item.width, item.height),
                    );
                    self.append_fragment(
                        fragment,
                        visual_x + margin.left,
                        visual_y + offset + margin.top,
                    );
                }
                InlineKind::Positioned(id) => {
                    self.record_inline_boxes(id, rect(visual_x, visual_y, 0.0, height));
                    self.enqueue_positioned(id, visual_x, visual_y, 0);
                }
                InlineKind::Break | InlineKind::Float(_) => {}
            }
            cursor += item.width + if is_space { extra_space } else { 0.0 };
        }
        self.paint_owner = saved_owner;
        height
    }

    /// Border-box min/max bounds, with minimums winning conflicting limits.
    fn flex_limits(&self, id: NodeId, width: f32, main: f32, column: bool) -> (f32, f32) {
        let style = self.style(id);
        let padding = self.padding(id, width);
        let border = self.borders(id);
        let extra = if column {
            padding.vertical() + border.vertical()
        } else {
            padding.horizontal() + border.horizontal()
        };
        let adjustment = if style.box_sizing == "border-box" {
            0.0
        } else {
            extra
        };
        let (min, max) = if column {
            (style.min_height, style.max_height)
        } else {
            (style.min_width, style.max_width)
        };
        let min = extent(resolve(min, main).map(|v| v + adjustment).unwrap_or(extra)).max(extra);
        let max = extent(
            resolve(max, main)
                .map(|v| v + adjustment)
                .unwrap_or(MAX_EXTENT),
        )
        .max(min);
        (min, max)
    }

    fn flex_metrics(
        &self,
        id: NodeId,
        width: f32,
        main: Option<f32>,
        column: bool,
        natural: f32,
    ) -> FlexSize {
        let style = self.style(id);
        let padding = self.padding(id, width);
        let border = self.borders(id);
        let extra = if column {
            padding.vertical() + border.vertical()
        } else {
            padding.horizontal() + border.horizontal()
        };
        let adjustment = if style.box_sizing == "border-box" {
            0.0
        } else {
            extra
        };
        // A percentage basis in an indefinite main axis behaves as content.
        let resolve_main = |length| match length {
            Length::Percent(_) => main.and_then(|v| resolve(length, v)),
            _ => resolve(length, main.unwrap_or(0.0)),
        };
        let preferred = if column { style.height } else { style.width };
        let base = extent(
            resolve_main(style.flex_basis)
                .or_else(|| resolve_main(preferred))
                .map(|v| v + adjustment)
                .unwrap_or(natural),
        );
        let (min, max) = self.flex_limits(id, width, main.unwrap_or(0.0), column);
        FlexSize {
            base,
            inner: (base - extra).max(0.0),
            min,
            max,
            grow: extent(style.flex_grow),
            shrink: extent(style.flex_shrink),
        }
    }

    fn layout_flex(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        available: FlexConstraints,
        style: &ComputedStyle,
        depth: usize,
    ) -> f32 {
        for &id in children {
            if !self.is_hidden(id)
                && matches!(self.style(id).position.as_str(), "absolute" | "fixed")
            {
                self.enqueue_positioned(id, x, y, depth);
            }
        }
        let mut items = self.flow_items(children);
        // Stable visual order never changes the DOM or sequential navigation order.
        items.sort_by_key(|id| self.style(*id).order);
        let width = available.width;
        let height = available.height;
        let max_height = available.max_height.map(|h| h.max(available.min_height));
        let auto_height =
            |natural: f32| natural.clamp(available.min_height, max_height.unwrap_or(MAX_EXTENT));
        let reverse = style.flex_direction.ends_with("reverse");
        let column = style.flex_direction.starts_with("column");
        let row_gap = extent(grid_length(style.row_gap, height).unwrap_or(0.0));
        let column_gap = extent(grid_length(style.column_gap, Some(width)).unwrap_or(0.0));
        let gap = if column { row_gap } else { column_gap };
        let single_line = style.flex_wrap == "nowrap";
        let wrap_reverse = style.flex_wrap == "wrap-reverse";
        if column {
            let mut columns = Vec::new();
            let mut line = Vec::new();
            let mut used = 0.0;
            let mut natural_main = 0.0;
            let line_limit = height.or(max_height);
            for id in items {
                if !grid_charge(&mut self.flex_work_left, 1) {
                    break;
                }
                let margin = self.margins(id, width);
                let child = self.style(id);
                let align = flex_alignment(child, style);
                let space = (width - margin.horizontal()).max(0.0);
                let auto_margin = matches!(child.margin.left, Length::Auto)
                    || matches!(child.margin.right, Length::Auto);
                let (min, max) = self.flex_limits(id, width, width, false);
                let child_width = if !matches!(child.width, Length::Auto) {
                    self.width_for(id, space, width)
                } else if single_line && align == "stretch" && !auto_margin {
                    // A nowrap column has a known line width before main sizing.
                    space.clamp(min, max)
                } else {
                    // CSS Flexbox §9.2/9.4: an indefinite auto cross size uses
                    // fit-content before line widths and stretch are resolved.
                    let (minimum, preferred) = self.preferred_widths(id, width, depth);
                    minimum.max(space).min(preferred).clamp(min, max)
                };
                let definite_basis = matches!(child.flex_basis, Length::Px(_))
                    || height.is_some() && matches!(child.flex_basis, Length::Percent(_))
                    || matches!(child.height, Length::Px(_))
                    || height.is_some() && matches!(child.height, Length::Percent(_));
                let natural = if definite_basis {
                    None
                } else {
                    Some(self.fragment(id, width, child_width, depth))
                };
                let metrics = self.flex_metrics(
                    id,
                    width,
                    height,
                    true,
                    natural.as_ref().map(|f| f.size.height).unwrap_or(0.0),
                );
                let outer = metrics.hypothetical() + margin.vertical();
                if !single_line
                    && !line.is_empty()
                    && line_limit.is_some_and(|main| used + gap + outer > main)
                {
                    columns.push(std::mem::take(&mut line));
                    used = 0.0;
                }
                if !line.is_empty() {
                    used += gap;
                }
                if !line.is_empty() || !columns.is_empty() {
                    natural_main += gap;
                }
                line.push((id, margin, child_width, metrics, natural));
                used += outer;
                natural_main += outer;
            }
            if !line.is_empty() {
                columns.push(line);
            }
            // Auto main size still obeys min/max-height. A finite maximum can
            // break lines, without making percentage bases definite.
            let main = height.unwrap_or_else(|| auto_height(extent(natural_main)));
            let mut cross_sizes: Vec<_> = columns
                .iter()
                .map(|line| {
                    if single_line {
                        width
                    } else {
                        line.iter()
                            .map(|(_, margin, cross, _, _)| cross + margin.horizontal())
                            .fold(0.0, f32::max)
                    }
                })
                .collect();
            let cross_positions = flex_line_positions(
                &mut cross_sizes,
                width,
                column_gap,
                single_line,
                wrap_reverse,
                &style.align_content,
            );
            for ((line, line_width), line_x) in
                columns.into_iter().zip(cross_sizes).zip(cross_positions)
            {
                let gaps = gap * line.len().saturating_sub(1) as f32;
                let margins: f32 = line.iter().map(|(_, m, _, _, _)| m.vertical()).sum();
                let metrics: Vec<_> = line.iter().map(|(_, _, _, m, _)| *m).collect();
                let targets = resolve_flexible_lengths(
                    &metrics,
                    main - margins - gaps,
                    &mut self.flex_work_left,
                );
                let free = main - margins - gaps - targets.iter().sum::<f32>();
                let auto_count: usize = line
                    .iter()
                    .map(|(id, _, _, _, _)| {
                        let m = self.style(*id).margin;
                        usize::from(matches!(m.top, Length::Auto))
                            + usize::from(matches!(m.bottom, Length::Auto))
                    })
                    .sum();
                let auto_space = if auto_count > 0 {
                    free.max(0.0) / auto_count as f32
                } else {
                    0.0
                };
                let (offset, between) = distribution(
                    &style.justify_content,
                    if auto_count > 0 { free.min(0.0) } else { free },
                    line.len(),
                );
                let mut cursor = offset;
                for ((id, mut margin, mut child_width, _, natural), target) in
                    line.into_iter().zip(targets)
                {
                    let child = self.style(id);
                    if matches!(child.margin.top, Length::Auto) {
                        margin.top = auto_space;
                    }
                    if matches!(child.margin.bottom, Length::Auto) {
                        margin.bottom = auto_space;
                    }
                    let align = flex_alignment(child, style);
                    let auto_left = matches!(child.margin.left, Length::Auto);
                    let auto_right = matches!(child.margin.right, Length::Auto);
                    if align == "stretch"
                        && matches!(child.width, Length::Auto)
                        && !auto_left
                        && !auto_right
                    {
                        let (min, max) = self.flex_limits(id, width, width, false);
                        child_width = (line_width - margin.horizontal()).clamp(min, max);
                    }
                    let dx = cross_offset(
                        flex_cross_alignment(align, wrap_reverse),
                        line_width - child_width - margin.horizontal(),
                        auto_left,
                        auto_right,
                    );
                    let main_pos = if reverse {
                        main - cursor - margin.bottom - target
                    } else {
                        cursor + margin.top
                    };
                    let fragment = match natural {
                        Some(fragment)
                            if (fragment.size.height - target).abs() < 0.001
                                && (fragment.size.width - child_width).abs() < 0.001 =>
                        {
                            fragment
                        }
                        _ => self.fragment_sized(id, width, child_width, Some(target), depth),
                    };
                    self.append_fragment(fragment, x + line_x + margin.left + dx, y + main_pos);
                    cursor += target + margin.vertical() + gap + between;
                }
            }
            return main;
        }
        let mut rows = Vec::new();
        let mut row = Vec::new();
        let mut used = 0.0;
        for id in items {
            if !grid_charge(&mut self.flex_work_left, 1) {
                break;
            }
            let metrics = self.flex_metrics(
                id,
                width,
                Some(width),
                false,
                self.intrinsic_width(id, width),
            );
            let margin = self.margins(id, width);
            let outer = metrics.hypothetical() + margin.horizontal();
            if style.flex_wrap != "nowrap" && !row.is_empty() && used + gap + outer > width {
                rows.push(std::mem::take(&mut row));
                used = 0.0;
            }
            if !row.is_empty() {
                used += gap;
            }
            row.push((id, metrics, margin));
            used += outer;
        }
        if !row.is_empty() {
            rows.push(row);
        }
        let mut laid_out = Vec::new();
        for row in rows {
            let gaps = gap * row.len().saturating_sub(1) as f32;
            let margins: f32 = row.iter().map(|(_, _, m)| m.horizontal()).sum();
            let metrics: Vec<_> = row.iter().map(|(_, m, _)| *m).collect();
            let targets = resolve_flexible_lengths(
                &metrics,
                width - margins - gaps,
                &mut self.flex_work_left,
            );
            let mut fragments = Vec::new();
            let mut row_height = if single_line {
                height.unwrap_or(0.0)
            } else {
                0.0
            };
            for ((id, _, margin), target) in row.into_iter().zip(targets) {
                let child = self.style(id);
                let stretched = (single_line
                    && flex_alignment(child, style) == "stretch"
                    && matches!(child.height, Length::Auto)
                    && !matches!(child.margin.top, Length::Auto)
                    && !matches!(child.margin.bottom, Length::Auto))
                .then_some(height)
                .flatten()
                .map(|h| {
                    let (min, max) = self.flex_limits(id, width, h, true);
                    (h - margin.vertical()).clamp(min, max)
                });
                let fragment = self.fragment_sized(id, width, target, stretched, depth);
                if !single_line || height.is_none() {
                    row_height = row_height.max(fragment.size.height + margin.vertical());
                }
                fragments.push((id, fragment, margin));
            }
            laid_out.push((row_height, fragments));
        }
        let natural_height = extent(
            laid_out.iter().map(|(height, _)| *height).sum::<f32>()
                + row_gap * laid_out.len().saturating_sub(1) as f32,
        );
        let cross_size = height.unwrap_or_else(|| auto_height(natural_height));
        let mut cross_sizes: Vec<_> = laid_out.iter().map(|(height, _)| *height).collect();
        let cross_positions = flex_line_positions(
            &mut cross_sizes,
            cross_size,
            row_gap,
            single_line,
            wrap_reverse,
            &style.align_content,
        );
        for (((_, fragments), row_height), row_y) in
            laid_out.into_iter().zip(cross_sizes).zip(cross_positions)
        {
            let free = width
                - fragments
                    .iter()
                    .map(|(_, f, m)| f.size.width + m.horizontal())
                    .sum::<f32>()
                - gap * fragments.len().saturating_sub(1) as f32;
            let auto_count: usize = fragments
                .iter()
                .map(|(id, _, _)| {
                    let m = self.style(*id).margin;
                    usize::from(matches!(m.left, Length::Auto))
                        + usize::from(matches!(m.right, Length::Auto))
                })
                .sum();
            let auto_space = if auto_count > 0 {
                free.max(0.0) / auto_count as f32
            } else {
                0.0
            };
            let (offset, between) = distribution(
                &style.justify_content,
                if auto_count > 0 { free.min(0.0) } else { free },
                fragments.len(),
            );
            let mut cursor = offset;
            for (id, mut fragment, mut margin) in fragments {
                let child = self.style(id);
                let align = flex_alignment(child, style);
                if matches!(child.margin.left, Length::Auto) {
                    margin.left = auto_space;
                }
                if matches!(child.margin.right, Length::Auto) {
                    margin.right = auto_space;
                }
                let auto_top = matches!(child.margin.top, Length::Auto);
                let auto_bottom = matches!(child.margin.bottom, Length::Auto);
                if align == "stretch"
                    && matches!(child.height, Length::Auto)
                    && !auto_top
                    && !auto_bottom
                {
                    let (min, max) = self.flex_limits(id, width, row_height, true);
                    let target = (row_height - margin.vertical()).clamp(min, max);
                    if (target - fragment.size.height).abs() > 0.001 {
                        // Relayout restores descendant clipping and gives nested flexboxes their used size.
                        fragment = self.fragment_sized(
                            id,
                            width,
                            fragment.size.width,
                            Some(target),
                            depth,
                        );
                    }
                }
                let child = self.style(id);
                let align = flex_cross_alignment(flex_alignment(child, style), wrap_reverse);
                let dy = cross_offset(
                    align,
                    row_height - fragment.size.height - margin.vertical(),
                    auto_top,
                    auto_bottom,
                );
                let main_pos = if reverse {
                    width - cursor - margin.right - fragment.size.width
                } else {
                    cursor + margin.left
                };
                let advance = fragment.size.width + margin.horizontal() + gap + between;
                self.append_fragment(fragment, x + main_pos, y + row_y + margin.top + dy);
                cursor += advance;
            }
        }
        natural_height
    }

    fn layout_grid(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        available: Size,
        style: &ComputedStyle,
        depth: usize,
    ) -> f32 {
        let width = available.width;
        let height = (available.height >= 0.0).then_some(available.height);
        let column_gap = extent(grid_length(style.column_gap, Some(width)).unwrap_or(0.0));
        // Cyclic percentages contribute zero during intrinsic row sizing.
        let row_gap = extent(grid_length(style.row_gap, height).unwrap_or(0.0));
        let plan = grid_plan(
            self.flow_items(children),
            self.styles,
            style,
            &mut self.grid_work_left,
        );
        let columns = grid_track_list(
            &style.grid_template_columns,
            &style.grid_auto_columns,
            plan.column_origin,
            plan.columns,
        );
        let rows = grid_track_list(
            &style.grid_template_rows,
            &style.grid_auto_rows,
            plan.row_origin,
            plan.rows,
        );
        let mut contributions = Vec::new();
        for item in &plan.items {
            let Some(area) = item.area else {
                continue;
            };
            if !grid_charge(&mut self.grid_work_left, 1) {
                break;
            }
            let (min, max) = self.preferred_widths(item.id, width, depth);
            let (lower, upper) = self.flex_limits(item.id, width, width, false);
            let margins = self.margins(item.id, width).horizontal();
            contributions.push(GridContribution {
                start: area.column,
                span: area.columns,
                min: extent(min.clamp(lower, upper) + margins),
                max: extent(max.clamp(lower, upper) + margins),
            });
        }
        let widths = size_grid_tracks(
            &columns,
            Some(width),
            column_gap,
            &contributions,
            &style.justify_content,
            &mut self.grid_work_left,
        );
        let (column_positions, column_gap) =
            grid_positions(&widths, column_gap, Some(width), &style.justify_content);
        let mut prepared = Vec::new();
        // Retain measured fragments when their used height is unchanged. This
        // avoids exponential remeasurement of nested auto-sized grids. A changed
        // height triggers reflow and refunds only that discarded paint output;
        // visits and intrinsic/placement work always remain charged.
        let old_reference = self.grid_height_reference;
        let mut row_contributions = Vec::new();
        for item in &plan.items {
            let Some(area) = item.area else {
                continue;
            };
            if !grid_charge(&mut self.grid_work_left, 1) {
                break;
            }
            let area_width = grid_area_size(&widths, area.column, area.columns, column_gap);
            let child = self.style(item.id);
            let margin = self.margins(item.id, area_width);
            let align = grid_alignment(child, style, true);
            let auto_margin = matches!(child.margin.left, Length::Auto)
                || matches!(child.margin.right, Length::Auto);
            let space = (area_width - margin.horizontal()).max(0.0);
            let child_width =
                if matches!(child.width, Length::Auto) && (align != "stretch" || auto_margin) {
                    let (min, max) = self.preferred_widths(item.id, area_width, depth);
                    self.width_for(item.id, max.min(space).max(min), area_width)
                } else {
                    self.width_for(item.id, space, area_width)
                };
            self.grid_height_reference = Some((item.id, None));
            let checkpoint = (
                self.commands_created,
                self.emitted_glyphs_left,
                self.hits_created,
            );
            let fragment = self.fragment(item.id, area_width, child_width, depth);
            let natural_height = fragment.size.height;
            let paint_cost = (
                self.commands_created.saturating_sub(checkpoint.0),
                checkpoint.1.saturating_sub(self.emitted_glyphs_left),
                self.hits_created.saturating_sub(checkpoint.2),
            );
            row_contributions.push(GridContribution {
                start: area.row,
                span: area.rows,
                min: extent(natural_height + margin.vertical()),
                max: extent(natural_height + margin.vertical()),
            });
            prepared.push((
                item.id,
                area,
                area_width,
                child_width,
                natural_height,
                margin,
                fragment,
                paint_cost,
            ));
        }
        self.grid_height_reference = old_reference;
        let heights = size_grid_tracks(
            &rows,
            height,
            row_gap,
            &row_contributions,
            &style.align_content,
            &mut self.grid_work_left,
        );
        let natural_height =
            extent(heights.iter().sum::<f32>() + row_gap * heights.len().saturating_sub(1) as f32);
        // For an auto-height grid a percentage row-gap is resolved only after
        // the content height is known, and can cause overflow without feeding
        // back into that intrinsic height.
        let final_row_gap =
            extent(grid_length(style.row_gap, height.or(Some(natural_height))).unwrap_or(0.0));
        let (row_positions, final_row_gap) =
            grid_positions(&heights, final_row_gap, height, &style.align_content);
        for (
            id,
            area,
            area_width,
            child_width,
            measured_height,
            margin,
            mut fragment,
            paint_cost,
        ) in prepared
        {
            let area_height = grid_area_size(&heights, area.row, area.rows, final_row_gap);
            let child = self.style(id);
            let align_x = grid_alignment(child, style, true);
            let align_y = grid_alignment(child, style, false);
            let dx = cross_offset(
                align_x,
                area_width - child_width - margin.horizontal(),
                matches!(child.margin.left, Length::Auto),
                matches!(child.margin.right, Length::Auto),
            );
            let auto_y = matches!(child.margin.top, Length::Auto)
                || matches!(child.margin.bottom, Length::Auto);
            let extras = self.padding(id, area_width).vertical() + self.borders(id).vertical();
            let (min, max) = self.flex_limits(id, area_width, area_height, true);
            let child_height = if let Some(value) = grid_length(child.height, Some(area_height)) {
                (value
                    + if child.box_sizing == "border-box" {
                        0.0
                    } else {
                        extras
                    })
                .clamp(min, max)
            } else if align_y == "stretch" && !auto_y {
                (area_height - margin.vertical()).clamp(min, max)
            } else {
                measured_height.clamp(min, max)
            };
            let dy = cross_offset(
                align_y,
                area_height - child_height - margin.vertical(),
                matches!(child.margin.top, Length::Auto),
                matches!(child.margin.bottom, Length::Auto),
            );
            if (child_height - measured_height).abs() > 0.001 {
                drop(fragment);
                self.commands_created = self.commands_created.saturating_sub(paint_cost.0);
                self.hits_created = self.hits_created.saturating_sub(paint_cost.2);
                self.emitted_glyphs_left =
                    (self.emitted_glyphs_left + paint_cost.1).min(MAX_GLYPHS);
                self.grid_height_reference = Some((id, Some(area_height)));
                fragment =
                    self.fragment_sized(id, area_width, child_width, Some(child_height), depth);
                self.grid_height_reference = old_reference;
            }
            self.append_fragment(
                fragment,
                x + column_positions[area.column] + margin.left + dx,
                y + row_positions[area.row] + margin.top + dy,
            );
        }
        for &id in children {
            if !self.is_hidden(id)
                && matches!(self.style(id).position.as_str(), "absolute" | "fixed")
            {
                self.enqueue_positioned(id, x, y, depth);
            }
        }
        natural_height
    }

    fn layout_table(
        &mut self,
        children: &[NodeId],
        x: f32,
        y: f32,
        width: f32,
        depth: usize,
    ) -> f32 {
        struct Cell {
            node: NodeId,
            row: usize,
            column: usize,
            columns: usize,
            rows: usize,
            fragment: Option<Fragment>,
        }
        let mut rows = Vec::new();
        let mut captions = Vec::new();
        let mut pending: Vec<NodeId> = children.iter().rev().copied().collect();
        let mut scanned = 0usize;
        while let Some(id) = pending.pop() {
            scanned += 1;
            if scanned > 4096 {
                break;
            }
            if self.is_hidden(id) {
                continue;
            }
            match self.html_tag(id) {
                Some("tr") => rows.push(id),
                Some("caption") => captions.push(id),
                Some("thead" | "tbody" | "tfoot") => {
                    pending.extend(self.doc.nodes[id].children.iter().rev().copied());
                }
                _ => {}
            }
        }
        let mut caption_height = 0.0;
        for caption in captions {
            let fragment = self.fragment(caption, width, width, depth);
            let height = fragment.size.height;
            self.append_fragment(fragment, x, y + caption_height);
            caption_height += height;
        }
        if rows.is_empty() {
            return caption_height;
        }
        let mut cells = Vec::new();
        let mut occupied = [0usize; 128];
        let mut column_count = 1usize;
        for (row, &id) in rows.iter().enumerate() {
            let mut column = 0usize;
            for &node in &self.doc.nodes[id].children {
                if self.is_hidden(node) || !matches!(self.html_tag(node), Some("td" | "th")) {
                    continue;
                }
                while column < occupied.len() && occupied[column] > 0 {
                    column += 1;
                }
                if column >= occupied.len() {
                    break;
                }
                let columns = self.attr_number(node, "colspan").unwrap_or(1.0) as usize;
                let columns = columns.clamp(1, occupied.len() - column);
                let rowspan = self.attr_number(node, "rowspan").unwrap_or(1.0) as usize;
                let rowspan = if rowspan == 0 {
                    rows.len() - row
                } else {
                    rowspan.min(rows.len() - row)
                };
                for slot in &mut occupied[column..column + columns] {
                    *slot = rowspan;
                }
                cells.push(Cell {
                    node,
                    row,
                    column,
                    columns,
                    rows: rowspan,
                    fragment: None,
                });
                column += columns;
                column_count = column_count.max(column);
            }
            for slot in &mut occupied {
                *slot = slot.saturating_sub(1);
            }
        }
        // Track sizing accepts explicit cell widths, distributing remaining space
        // over auto tracks. Full intrinsic table sizing is a separate algorithm.
        let spacing = 2.0;
        let available = (width - spacing * (column_count + 1) as f32).max(0.0);
        let mut tracks = vec![0.0f32; column_count];
        for cell in &cells {
            if let Some(value) = resolve(self.style(cell.node).width, width)
                .or_else(|| self.attr_number(cell.node, "width"))
            {
                let share = (value - spacing * cell.columns.saturating_sub(1) as f32).max(0.0)
                    / cell.columns as f32;
                for track in &mut tracks[cell.column..cell.column + cell.columns] {
                    *track = track.max(share);
                }
            }
        }
        let fixed = tracks.iter().sum::<f32>();
        let autos = tracks.iter().filter(|track| **track == 0.0).count();
        let remaining = (available - fixed).max(0.0);
        if autos > 0 {
            for track in &mut tracks {
                if *track == 0.0 {
                    *track = remaining / autos as f32;
                }
            }
        } else {
            for track in &mut tracks {
                *track += remaining / column_count as f32;
            }
        }
        let mut heights: Vec<f32> = rows
            .iter()
            .map(|&id| {
                resolve(self.style(id).height, self.viewport.height)
                    .unwrap_or(0.0)
                    .max(0.0)
            })
            .collect();
        for cell in &mut cells {
            let cell_width = tracks[cell.column..cell.column + cell.columns]
                .iter()
                .sum::<f32>()
                + spacing * cell.columns.saturating_sub(1) as f32;
            let fragment = self.fragment(cell.node, cell_width, cell_width, depth + 1);
            if cell.rows == 1 {
                heights[cell.row] = heights[cell.row].max(fragment.size.height);
            }
            cell.fragment = Some(fragment);
        }
        for cell in &cells {
            if cell.rows > 1 {
                let tracks = &mut heights[cell.row..cell.row + cell.rows];
                let allocated =
                    tracks.iter().sum::<f32>() + spacing * cell.rows.saturating_sub(1) as f32;
                let extra = (cell
                    .fragment
                    .as_ref()
                    .map(|fragment| fragment.size.height)
                    .unwrap_or(0.0)
                    - allocated)
                    .max(0.0)
                    / cell.rows as f32;
                for height in tracks {
                    *height += extra;
                }
            }
        }
        let mut row_y = Vec::with_capacity(rows.len());
        let mut cursor = y + caption_height + spacing;
        for (row, &id) in rows.iter().enumerate() {
            row_y.push(cursor);
            let style = self.style(id);
            let row_rect = rect(
                x + spacing,
                cursor,
                available + spacing * column_count.saturating_sub(1) as f32,
                heights[row],
            );
            let background = style.background_color;
            let saved_owner = self.paint_owner;
            self.paint_owner = PaintOwner {
                node: id,
                background: true,
            };
            self.push(DrawCommand::Rect {
                rect: row_rect,
                color: background,
                radius: 0.0,
            });
            self.paint_owner = saved_owner;
            self.push_hit(HitRegion {
                fixed: false,
                node: id,
                rect: row_rect,
            });
            cursor += heights[row] + spacing;
        }
        for cell in cells {
            let Some(mut fragment) = cell.fragment else {
                continue;
            };
            let cell_height = heights[cell.row..cell.row + cell.rows].iter().sum::<f32>()
                + spacing * cell.rows.saturating_sub(1) as f32;
            let extra = (cell_height - fragment.size.height).max(0.0);
            let offset = match self.style(cell.node).vertical_align.as_str() {
                "middle" => extra / 2.0,
                "bottom" | "text-bottom" => extra,
                _ => 0.0,
            };
            let content_start =
                if matches!(fragment.commands.get(5), Some(DrawCommand::PushClip { .. })) {
                    6
                } else {
                    5
                };
            for command in fragment.commands.iter_mut().skip(content_start) {
                translate(command, 0.0, offset);
            }
            for hit in fragment.hits.iter_mut().skip(1) {
                hit.rect.y += offset;
            }
            stretch_fragment(&mut fragment, cell_height);
            let cell_x = x
                + spacing
                + tracks[..cell.column].iter().sum::<f32>()
                + spacing * cell.column as f32;
            self.append_fragment(fragment, cell_x, row_y[cell.row]);
        }
        extent(cursor - y)
    }

    fn flow_items(&mut self, children: &[NodeId]) -> Vec<NodeId> {
        let mut items = Vec::new();
        for &id in children {
            if self.is_hidden(id)
                || matches!(self.style(id).position.as_str(), "absolute" | "fixed")
            {
                continue;
            }
            let visible = match self.doc.nodes.get(id).map(|node| &node.kind) {
                Some(NodeKind::Text(text)) => has_inline_content(text, &mut self.glyphs_left),
                Some(NodeKind::Element(_)) => true,
                _ => false,
            };
            if visible {
                items.push(id);
            }
        }
        items
    }

    fn intrinsic_width(&self, id: NodeId, available: f32) -> f32 {
        let style = self.style(id);
        if let Some(width) = resolve(style.width, available) {
            let extra = if style.box_sizing == "border-box" {
                0.0
            } else {
                self.padding(id, available).horizontal() + self.borders(id).horizontal()
            };
            return extent(width + extra);
        }
        if matches!(
            self.layout_tag(id),
            "img" | "svg" | "canvas" | "video" | "input" | "textarea"
        ) {
            return self.width_for(id, available, available);
        }
        // Do not recurse through an adversarial tree for intrinsic sizing.
        let mut stack = vec![(id, 0usize)];
        let mut measured = 0.0f32;
        let mut count = 0usize;
        while let Some((node, depth)) = stack.pop() {
            let work_left = self.intrinsic_work_left.get();
            if count >= 4096 || depth > MAX_DEPTH || work_left == 0 {
                break;
            }
            self.intrinsic_work_left.set(work_left - 1);
            count += 1;
            let Some(node_value) = self.doc.nodes.get(node) else {
                continue;
            };
            if self.is_hidden(node) {
                continue;
            }
            if let NodeKind::Text(text) = &node_value.kind {
                let bounded: String = text
                    .chars()
                    .take(4096.min(self.intrinsic_work_left.get()))
                    .collect();
                self.intrinsic_work_left.set(
                    self.intrinsic_work_left
                        .get()
                        .saturating_sub(bounded.chars().count()),
                );
                measured += self.measure(&bounded, self.style(node));
            } else {
                stack.extend(
                    node_value
                        .children
                        .iter()
                        .rev()
                        .map(|&child| (child, depth + 1)),
                );
            }
            if measured >= available {
                break;
            }
        }
        (measured + self.padding(id, available).horizontal() + self.borders(id).horizontal())
            .clamp(0.0, MAX_EXTENT)
    }

    fn measure(&self, text: &str, style: &ComputedStyle) -> f32 {
        finite(
            self.fonts.measure(
                text,
                font_size(style),
                style.font_weight >= 600,
                style.font_style == "italic" || style.font_style == "oblique",
                monospace(style),
            ),
            0.0,
        )
        .max(0.0)
    }

    fn attr_number(&self, id: NodeId, attribute: &str) -> Option<f32> {
        self.doc
            .attr(id, attribute)?
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|value| value.is_finite() && *value >= 0.0)
            .map(extent)
    }

    fn natural_size(&self, id: NodeId) -> Size {
        Size {
            width: self
                .attr_number(id, "data-eris-natural-width")
                .or_else(|| self.attr_number(id, "width"))
                .unwrap_or(300.0),
            height: self
                .attr_number(id, "data-eris-natural-height")
                .or_else(|| self.attr_number(id, "height"))
                .unwrap_or(150.0),
        }
    }

    fn paint_replaced(&mut self, id: NodeId, tag: &str, x: f32, y: f32, width: f32) -> f32 {
        let style = self.style(id).clone();
        let specified = resolve(style.height, self.viewport.height)
            .map(|height| {
                if style.box_sizing == "border-box" {
                    (height - self.padding(id, width).vertical() - self.borders(id).vertical())
                        .max(0.0)
                } else {
                    height
                }
            })
            .or_else(|| self.attr_number(id, "height"));
        let natural = self.natural_size(id);
        let height = specified
            .unwrap_or(width * natural.height / natural.width.max(1.0))
            .max(0.0);
        if tag == "svg" {
            self.push(DrawCommand::Image {
                rect: rect(x, y, width, height),
                key: format!("eris-inline-svg:{id}"),
            });
        } else if tag == "img" {
            // Successful decoding installs natural-size metadata. A missing
            // image fallback must not become an opaque backdrop (or alt-text
            // watermark) beneath a loaded image's transparent pixels.
            let loaded = self
                .attr_number(id, "data-eris-natural-width")
                .is_some_and(|v| v > 0.0)
                && self
                    .attr_number(id, "data-eris-natural-height")
                    .is_some_and(|v| v > 0.0);
            if !loaded {
                self.push(DrawCommand::Rect {
                    rect: rect(x, y, width, height),
                    color: rgba(236, 238, 242, 255),
                    radius: 0.0,
                });
                if let Some(alt) = self.doc.attr(id, "alt").filter(|alt| !alt.is_empty()) {
                    self.push(DrawCommand::Text {
                        x: x + 4.0,
                        y: y + 4.0,
                        text: alt.chars().take(256).collect(),
                        size: font_size(&style).min(16.0),
                        color: style.color,
                        bold: false,
                        italic: false,
                        monospace: false,
                    });
                }
            }
            if let Some(src) = self.doc.attr(id, "src").filter(|src| !src.is_empty()) {
                self.push(DrawCommand::Image {
                    rect: rect(x, y, width, height),
                    key: src.to_owned(),
                });
            }
        } else if tag == "video" {
            self.push(DrawCommand::Rect {
                rect: rect(x, y, width, height),
                color: rgba(20, 23, 30, 255),
                radius: 0.0,
            });
            if let Some(poster) = self.doc.attr(id, "poster") {
                self.push(DrawCommand::Image {
                    rect: rect(x, y, width, height),
                    key: poster.to_owned(),
                });
            }
        }
        extent(height)
    }

    fn paint_control(&mut self, id: NodeId, tag: &str, x: f32, y: f32, width: f32) -> f32 {
        let style = self.style(id).clone();
        let kind = self
            .doc
            .attr(id, "type")
            .unwrap_or("text")
            .to_ascii_lowercase();
        if tag == "input" && kind == "hidden" {
            return 0.0;
        }
        let check = tag == "input" && matches!(kind.as_str(), "checkbox" | "radio");
        let intrinsic_height = if check {
            16.0
        } else if tag == "textarea" {
            self.attr_number(id, "rows").unwrap_or(2.0) * line_height(&style) + 10.0
        } else {
            line_height(&style) + 10.0
        };
        let height = resolve(style.height, self.viewport.height)
            .map(|height| {
                if style.box_sizing == "border-box" {
                    (height - self.padding(id, width).vertical() - self.borders(id).vertical())
                        .max(0.0)
                } else {
                    height.max(0.0)
                }
            })
            .unwrap_or(intrinsic_height);
        self.push(DrawCommand::Rect {
            rect: rect(x, y, width, height),
            color: rgba(160, 167, 180, 255),
            radius: if kind == "radio" { 8.0 } else { 3.0 },
        });
        self.push(DrawCommand::Rect {
            rect: rect(x + 1.0, y + 1.0, (width - 2.0).max(0.0), height - 2.0),
            color: rgba(255, 255, 255, 255),
            radius: if kind == "radio" { 7.0 } else { 2.0 },
        });
        if check {
            if self.doc.attr(id, "checked").is_some() {
                self.push(DrawCommand::Rect {
                    rect: rect(x + 4.0, y + 4.0, (width - 8.0).max(0.0), height - 8.0),
                    color: rgba(50, 105, 225, 255),
                    radius: if kind == "radio" { 4.0 } else { 1.0 },
                });
            }
            return height;
        }
        let text = if tag == "textarea" {
            self.doc.text_content(id)
        } else if tag == "select" {
            let mut selected = None;
            let mut first = None;
            let mut stack = self.doc.nodes[id].children.clone();
            stack.reverse();
            let mut count = 0usize;
            while let Some(node) = stack.pop() {
                count += 1;
                if count > 4096 {
                    break;
                }
                if self.html_tag(node) == Some("option") {
                    if first.is_none() {
                        first = Some(node);
                    }
                    if self.doc.attr(node, "selected").is_some() {
                        selected = Some(node);
                        break;
                    }
                }
                if let Some(node) = self.doc.nodes.get(node) {
                    stack.extend(node.children.iter().rev().copied());
                }
            }
            selected
                .or(first)
                .map(|node| self.doc.text_content(node))
                .unwrap_or_default()
        } else {
            self.doc
                .attr(id, "value")
                .or_else(|| self.doc.attr(id, "placeholder"))
                .unwrap_or("")
                .to_owned()
        };
        let text = if kind == "password" {
            "•".repeat(text.chars().count().min(256))
        } else {
            text.chars().take(4096).collect()
        };
        let text = fit_text(
            &text,
            (width - if tag == "select" { 25.0 } else { 12.0 }).max(0.0),
            |value| self.measure(value, &style),
        );
        self.push(DrawCommand::Text {
            x: x + 6.0,
            y: y + 5.0,
            text,
            size: font_size(&style),
            color: style.color,
            bold: style.font_weight >= 600,
            italic: false,
            monospace: monospace(&style),
        });
        if tag == "select" {
            self.push(DrawCommand::Text {
                x: x + (width - 18.0).max(0.0),
                y: y + 5.0,
                text: "▾".into(),
                size: font_size(&style),
                color: style.color,
                bold: false,
                italic: false,
                monospace: false,
            });
        }
        height
    }

    fn paint_list_marker(&mut self, id: NodeId, x: f32, y: f32, style: &ComputedStyle) {
        let marker = match style.list_style_type.as_str() {
            "decimal" | "decimal-leading-zero" => {
                let parent = self.doc.nodes[id].parent;
                let mut number = parent
                    .and_then(|parent| self.attr_number(parent, "start"))
                    .unwrap_or(1.0) as usize;
                if let Some(parent) = parent {
                    for &child in &self.doc.nodes[parent].children {
                        if child == id {
                            break;
                        }
                        if self.html_tag(child) == Some("li") {
                            number += 1;
                        }
                    }
                }
                if let Some(value) = self.attr_number(id, "value") {
                    number = value as usize;
                }
                format!("{number}.")
            }
            "circle" => "◦".into(),
            "square" => "▪".into(),
            _ => "•".into(),
        };
        let marker_width = self.measure(&marker, style);
        self.push(DrawCommand::Text {
            x: x - marker_width - font_size(style) * 0.5,
            y,
            text: marker,
            size: font_size(style),
            color: style.color,
            bold: false,
            italic: false,
            monospace: monospace(style),
        });
    }
}

fn finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn extent(value: f32) -> f32 {
    finite(value, 0.0).clamp(0.0, MAX_EXTENT)
}

fn resolve(length: Length, reference: f32) -> Option<f32> {
    match length {
        Length::Auto | Length::Fr(_) => None,
        Length::Px(value) => Some(finite(value, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT)),
        Length::Percent(value) => {
            Some(finite(reference * value / 100.0, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT))
        }
    }
}

fn font_size(style: &ComputedStyle) -> f32 {
    finite(style.font_size, 16.0).clamp(1.0, 512.0)
}

fn line_height(style: &ComputedStyle) -> f32 {
    finite(style.line_height, font_size(style) * 1.3).clamp(1.0, 4096.0)
}

fn monospace(style: &ComputedStyle) -> bool {
    let family = style.font_family.to_ascii_lowercase();
    family.contains("monospace") || family.contains("courier") || family.contains("consolas")
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x: finite(x, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT),
        y: finite(y, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT),
        width: extent(width),
        height: extent(height),
    }
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color { r, g, b, a }
}

fn collapsed_margin(previous: f32, next: f32) -> f32 {
    previous.max(next).max(0.0) + previous.min(next).min(0.0)
}

#[derive(Clone, Copy)]
struct FlexConstraints {
    width: f32,
    height: Option<f32>,
    min_height: f32,
    max_height: Option<f32>,
}

#[derive(Clone, Copy)]
struct FlexSize {
    base: f32,
    inner: f32,
    min: f32,
    max: f32,
    grow: f32,
    shrink: f32,
}

impl FlexSize {
    fn hypothetical(self) -> f32 {
        self.base.clamp(self.min, self.max)
    }
}

/// CSS Flexbox §9.7. The shared work budget also bounds adversarial sequences
/// in which only one item freezes per iteration. All arithmetic sums use f64.
fn resolve_flexible_lengths(items: &[FlexSize], available: f32, work: &mut usize) -> Vec<f32> {
    let available = f64::from(available);
    let grow = items
        .iter()
        .map(|item| f64::from(item.hypothetical()))
        .sum::<f64>()
        < available;
    let factor = |item: &FlexSize| f64::from(if grow { item.grow } else { item.shrink });
    let mut targets: Vec<f64> = items.iter().map(|item| f64::from(item.base)).collect();
    let mut frozen: Vec<bool> = items
        .iter()
        .map(|item| {
            factor(item) == 0.0
                || if grow {
                    item.base > item.hypothetical()
                } else {
                    item.base < item.hypothetical()
                }
        })
        .collect();
    for ((target, frozen), item) in targets.iter_mut().zip(&frozen).zip(items) {
        if *frozen {
            *target = f64::from(item.hypothetical());
        }
    }
    let initial = available - targets.iter().sum::<f64>();
    loop {
        if frozen.iter().all(|v| *v) {
            break;
        }
        if *work < items.len() {
            break;
        }
        *work -= items.len();
        let mut remaining = available;
        let mut factors = 0.0;
        let mut scaled = 0.0;
        for (index, item) in items.iter().enumerate() {
            remaining -= if frozen[index] {
                targets[index]
            } else {
                f64::from(item.base)
            };
            if !frozen[index] {
                factors += factor(item);
                scaled += factor(item) * f64::from(item.inner);
            }
        }
        if factors < 1.0 && (initial * factors).abs() < remaining.abs() {
            remaining = initial * factors;
        }
        let mut violations = vec![0.0; items.len()];
        let mut total = 0.0;
        for (index, item) in items.iter().enumerate() {
            if frozen[index] {
                continue;
            }
            let adjustment = if grow && factors > 0.0 {
                remaining * factor(item) / factors
            } else if !grow && scaled > 0.0 {
                -remaining.abs() * factor(item) * f64::from(item.inner) / scaled
            } else {
                0.0
            };
            let raw = f64::from(item.base) + adjustment;
            targets[index] = raw.clamp(f64::from(item.min), f64::from(item.max));
            violations[index] = targets[index] - raw;
            total += violations[index];
        }
        let mut progress = false;
        for index in 0..items.len() {
            if !frozen[index]
                && (total.abs() < 1e-7
                    || total > 0.0 && violations[index] > 0.0
                    || total < 0.0 && violations[index] < 0.0)
            {
                frozen[index] = true;
                progress = true;
            }
        }
        if !progress {
            break;
        }
    }
    targets
        .into_iter()
        .zip(items)
        .map(|(target, item)| (target as f32).clamp(item.min, item.max))
        .collect()
}

fn flex_alignment<'a>(child: &'a ComputedStyle, parent: &'a ComputedStyle) -> &'a str {
    let align = if child.align_self == "auto" {
        parent.align_items.as_str()
    } else {
        child.align_self.as_str()
    };
    if align == "normal" { "stretch" } else { align }
}

fn flex_cross_alignment(align: &str, reverse: bool) -> &str {
    if reverse {
        match align {
            "flex-start" | "stretch" | "normal" => "flex-end",
            "flex-end" => "flex-start",
            _ => align,
        }
    } else {
        align
    }
}

/// CSS Flexbox §9.4/9.6: size and align complete lines before aligning items.
/// Positions are measured from the physical top/left; wrap-reverse reverses
/// cross-start without reversing source order or changing physical start/end.
fn flex_line_positions(
    sizes: &mut [f32],
    available: f32,
    gap: f32,
    single_line: bool,
    reverse: bool,
    align: &str,
) -> Vec<f32> {
    if sizes.is_empty() {
        return Vec::new();
    }
    if single_line {
        sizes[0] = available;
        return vec![0.0];
    }
    let free = available - sizes.iter().sum::<f32>() - gap * sizes.len().saturating_sub(1) as f32;
    let (safe, align) = if let Some(value) = align.strip_prefix("safe ") {
        (true, value)
    } else {
        (false, align.strip_prefix("unsafe ").unwrap_or(align))
    };
    let (offset, between) = if safe && free < 0.0 {
        (if reverse { free } else { 0.0 }, 0.0)
    } else {
        match align {
            "normal" | "stretch" if free > 0.0 => {
                let extra = free / sizes.len() as f32;
                for size in sizes.iter_mut() {
                    *size += extra;
                }
                (0.0, 0.0)
            }
            "start" if reverse => (free, 0.0),
            "end" if reverse => (0.0, 0.0),
            "space-between" | "space-around" | "space-evenly" if free < 0.0 => {
                // Distributed alignment falls back to a safe positional value:
                // overflow stays at the logical (top/left) start edge.
                (if reverse { free } else { 0.0 }, 0.0)
            }
            _ => distribution(align, free, sizes.len()),
        }
    };
    let mut cursor = offset;
    sizes
        .iter()
        .map(|size| {
            let position = if reverse {
                available - cursor - size
            } else {
                cursor
            };
            cursor += size + gap + between;
            position
        })
        .collect()
}

fn cross_offset(align: &str, free: f32, start_auto: bool, end_auto: bool) -> f32 {
    if start_auto || end_auto {
        return if start_auto {
            free.max(0.0) / if end_auto { 2.0 } else { 1.0 }
        } else {
            0.0
        };
    }
    match align {
        "center" => free * 0.5,
        "end" | "self-end" | "flex-end" => free,
        _ => 0.0,
    }
}

fn distribution(justify: &str, free: f32, count: usize) -> (f32, f32) {
    match justify {
        "center" => (free * 0.5, 0.0),
        "end" | "flex-end" => (free, 0.0),
        "space-between" if count > 1 => (0.0, free.max(0.0) / (count - 1) as f32),
        "space-around" if count > 0 && free >= 0.0 => {
            (free / count as f32 / 2.0, free / count as f32)
        }
        "space-evenly" if count > 0 && free >= 0.0 => {
            (free / (count + 1) as f32, free / (count + 1) as f32)
        }
        "space-around" | "space-evenly" => (free * 0.5, 0.0),
        _ => (0.0, 0.0),
    }
}

fn merge_geometry(target: &mut [Option<BoxGeometry>], boxes: &[BoxGeometry]) {
    for &g in boxes {
        if let Some(old) = target[g.node].as_mut().filter(|old| old.inline && g.inline) {
            let right = (old.padding.x + old.padding.width).max(g.padding.x + g.padding.width);
            let bottom = (old.padding.y + old.padding.height).max(g.padding.y + g.padding.height);
            old.padding.x = old.padding.x.min(g.padding.x);
            old.padding.y = old.padding.y.min(g.padding.y);
            old.padding.width = extent(right - old.padding.x);
            old.padding.height = extent(bottom - old.padding.y);
            old.border = old.padding;
            old.content = old.padding;
        } else {
            target[g.node] = Some(g);
        }
    }
}

fn translate_fragment(fragment: &mut Fragment, dx: f32, dy: f32) {
    let mut fixed = 0usize;
    for command in &mut fragment.commands {
        match command {
            DrawCommand::PushFixed => fixed += 1,
            DrawCommand::PopFixed => fixed = fixed.saturating_sub(1),
            _ if fixed == 0 => translate(command, dx, dy),
            _ => {}
        }
    }
    let move_rect = |r: &mut Rect| {
        r.x = finite(r.x + dx, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
        r.y = finite(r.y + dy, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
    };
    for hit in &mut fragment.hits {
        if !hit.fixed {
            move_rect(&mut hit.rect);
        }
    }
    for geometry in &mut fragment.boxes {
        if !geometry.fixed {
            move_rect(&mut geometry.border);
            move_rect(&mut geometry.padding);
            move_rect(&mut geometry.content);
        }
    }
    for job in &mut fragment.positioned {
        if !job.fixed {
            job.x = finite(job.x + dx, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
            job.y = finite(job.y + dy, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
        }
    }
}

fn translate(command: &mut DrawCommand, dx: f32, dy: f32) {
    match command {
        DrawCommand::Rect { rect, .. }
        | DrawCommand::Image { rect, .. }
        | DrawCommand::PushClip { rect } => {
            rect.x = finite(rect.x + dx, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
            rect.y = finite(rect.y + dy, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
        }
        DrawCommand::Text { x, y, .. } => {
            *x = finite(*x + dx, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
            *y = finite(*y + dy, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
        }
        DrawCommand::Line { x1, y1, x2, y2, .. } => {
            *x1 = finite(*x1 + dx, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
            *x2 = finite(*x2 + dx, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
            *y1 = finite(*y1 + dy, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
            *y2 = finite(*y2 + dy, 0.0).clamp(-MAX_EXTENT, MAX_EXTENT);
        }
        DrawCommand::PopClip
        | DrawCommand::PushFixed
        | DrawCommand::PopFixed
        | DrawCommand::PushOpacity { .. }
        | DrawCommand::PopOpacity => {}
    }
}

/// Stretch the principal box paint, keeping child content at its start edge.
fn stretch_fragment(fragment: &mut Fragment, height: f32) {
    let height = extent(height).max(fragment.size.height);
    if let Some(hit) = fragment.hits.first_mut() {
        hit.rect.height = height;
    }
    let old_height = fragment.size.height;
    if let Some(node) = fragment.hits.first().map(|hit| hit.node)
        && let Some(g) = fragment.boxes.iter_mut().find(|g| g.node == node)
    {
        let extra = height - old_height;
        g.border.height += extra;
        g.padding.height += extra;
        g.content.height += extra;
    }
    for (index, command) in fragment.commands.iter_mut().take(5).enumerate() {
        if let DrawCommand::Rect { rect, .. } = command {
            match index {
                1 if fragment.rounded_border => rect.height += height - old_height,
                0 | 2 | 4 => rect.height = height,
                3 => rect.y += height - old_height,
                _ => {}
            }
        }
    }
    if let Some(DrawCommand::PushClip { rect }) = fragment.commands.get_mut(5) {
        rect.height += height - old_height;
    }
    fragment.size.height = height;
}

/// Use binary search at character boundaries, avoiding quadratic measurement.
fn fit_text(text: &str, width: f32, measure: impl Fn(&str) -> f32) -> String {
    let single_line = text.lines().next().unwrap_or("");
    if measure(single_line) <= width {
        return single_line.to_owned();
    }
    let boundaries: Vec<usize> = single_line
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(single_line.len()))
        .collect();
    let mut low = 0;
    let mut high = boundaries.len().saturating_sub(1);
    while low < high {
        let middle = (low + high).div_ceil(2);
        if measure(&single_line[..boundaries[middle]]) <= width {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    single_line[..boundaries[low]].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn foreign_lookalikes_use_text_flow_and_only_svg_roots_are_replaced() {
        let mut document = Document::parse(
            "<style>#foreign{display:block}</style><math><input id=foreign type=hidden value=secret>foreign text</input><svg id=mathsvg width=20 height=20>math svg text</svg></math><input type=hidden value=hidden><svg id=actual width=20 height=20></svg>",
        );
        let body = document.query_selector("body").unwrap();
        for (namespace, tag, text) in [
            (Namespace::Html, "svg", "html svg text"),
            (Namespace::Svg, "img", "svg img text"),
            (Namespace::MathMl, "table", "math table text"),
        ] {
            let node = document.create_element_ns(namespace, tag);
            document.set_attr(node, "style", "display:block");
            document.set_attr(node, "src", "must-not-load.png");
            document.set_text_content(node, text);
            document.append_child(body, node);
        }
        let styles = crate::css::compute_styles(&document, &document.stylesheets(), 400.0, 300.0);
        let result = layout(&document, &styles, 400.0, 300.0, &Fonts::new());
        let text = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<String>();
        for expected in [
            "foreign text",
            "math svg text",
            "html svg text",
            "svg img text",
            "math table text",
        ] {
            assert!(text.contains(expected), "{expected}: {text}");
        }
        assert!(!text.contains("secret"));
        let images = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Image { key, .. } => Some(key.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            images,
            [format!(
                "eris-inline-svg:{}",
                document.query_selector("#actual").unwrap()
            )]
        );
    }

    fn render(source: &str, width: f32) -> (Document, LayoutResult) {
        let document = Document::parse(source);
        let styles = crate::css::compute_styles(&document, &document.stylesheets(), width, 400.0);
        let result = layout(&document, &styles, width, 400.0, &Fonts::new());
        (document, result)
    }

    fn bounds(document: &Document, result: &LayoutResult, selector: &str) -> Rect {
        let node = document.query_selector(selector).expect("element exists");
        result
            .hit_regions
            .iter()
            .find(|hit| hit.node == node)
            .expect("element is laid out")
            .rect
    }

    #[test]
    fn token_and_detached_fragment_hits_share_the_native_snapshot_budget() {
        let source = format!(
            "<style>body{{margin:0}}span{{display:inline-block;width:180px}}</style><span>{}</span><span>{}</span>",
            "x ".repeat(30_000),
            "y ".repeat(30_000)
        );
        let (_, result) = render(&source, 400.0);
        assert!(result.hit_regions.len() <= MAX_VISITS);
        assert!(
            result.hit_regions.len() > 90_000,
            "fixture must exercise the shared hit limit"
        );
        assert!(result.commands.len() <= MAX_COMMANDS);
    }

    #[test]
    fn invalid_document_root_does_not_enter_stacking_traversal() {
        let mut document = Document::parse("");
        document.root = usize::MAX;
        let result = layout(&document, &[], 200.0, 100.0, &Fonts::new());
        assert!(result.commands.is_empty());
        assert!(result.hit_regions.is_empty());
    }

    #[test]
    fn overlapping_auto_z_grid_items_paint_their_text_as_an_atomic_group() {
        use crate::graphics::{Canvas, ImageStore};
        let (_, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:40px;grid-template-rows:40px}div{grid-area:1/1;width:40px;height:40px;font-size:30px}#b{background:blue}</style><main><div>MMM</div><div id=b></div></main>",
            200.0,
        );
        let mut canvas = Canvas::new(200, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        for y in 0..40 {
            for x in 0..40 {
                assert_eq!(canvas.pixels[y * 200 + x], 0x0000ff, "{x},{y}");
            }
        }
    }

    #[test]
    fn absolute_replaced_auto_sizes_use_intrinsic_ratio_with_opposing_insets() {
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;width:200px;height:200px}img{position:absolute;left:0;right:0;top:0;bottom:0;margin:auto;width:100px}</style><main><img id=a data-eris-natural-width=200 data-eris-natural-height=100></main>",
            300.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(50.0, 75.0, 100.0, 50.0));
    }

    #[test]
    fn percentage_height_uses_definite_parent_and_relative_auto_parent_ignores_percentage_top() {
        let (doc, result) = render(
            "<style>html,body{height:100%;margin:0}main{height:50%;position:relative}#child{height:50%}section{position:relative}#relative{position:relative;top:50%;height:20px}</style><main><div id=child></div></main><section><div id=relative></div></section>",
            300.0,
        );
        assert_eq!(bounds(&doc, &result, "main").height, 200.0);
        assert_eq!(bounds(&doc, &result, "#child").height, 100.0);
        assert_eq!(bounds(&doc, &result, "#relative").y, 200.0);
    }

    #[test]
    fn stacking_ties_follow_mutated_dom_tree_order_instead_of_arena_ids() {
        let mut document = Document::parse(
            "<style>body{margin:0}div{position:absolute;left:0;top:0;width:50px;height:50px}#a{background:red}#b{background:blue}</style><main><div id=a></div><div id=b></div></main>",
        );
        let parent = document.query_selector("main").unwrap();
        let a = document.query_selector("#a").unwrap();
        document.append_child(parent, a);
        let styles = crate::css::compute_styles(&document, &document.stylesheets(), 200.0, 100.0);
        let result = layout(&document, &styles, 200.0, 100.0, &Fonts::new());
        assert_eq!(pixel(&result, 10, 10), 0xff0000);
        assert_eq!(result.hit_test(10.0, 10.0), Some(a));
    }

    #[test]
    fn deep_fixed_clip_opacity_scopes_and_exhausted_command_quota_remain_typed_and_balanced() {
        let mut source = String::from(
            "<style>body{margin:0}.outer{opacity:.9;overflow:hidden;position:relative;width:40px;height:40px}.fixed{position:fixed;overflow:hidden;left:0;top:0;width:40px;height:40px}.leaf{position:relative;overflow:hidden;height:1px;background:red}.leaf:nth-child(2n){z-index:1}</style>",
        );
        for _ in 0..12 {
            source.push_str("<div class=outer><div class=fixed>");
        }
        source.push_str(&"<div class=leaf>x</div>".repeat(40_000));
        for _ in 0..12 {
            source.push_str("</div></div>");
        }
        let (doc, result) = render(&source, 200.0);
        let leaves = result
            .hit_regions
            .iter()
            .filter(|hit| doc.attr(hit.node, "class") == Some("leaf"))
            .count();
        assert!(
            (1..40_000).contains(&leaves),
            "expected actual quota truncation: {leaves}"
        );
        let mut scopes = Vec::new();
        let mut saw_fixed = false;
        let mut saw_opacity = false;
        for command in &result.commands {
            match command {
                DrawCommand::PushFixed => {
                    saw_fixed = true;
                    scopes.push(1);
                }
                DrawCommand::PushClip { .. } => scopes.push(0),
                DrawCommand::PushOpacity { .. } => {
                    saw_opacity = true;
                    scopes.push(2);
                }
                DrawCommand::PopFixed => assert_eq!(scopes.pop(), Some(1)),
                DrawCommand::PopClip => assert_eq!(scopes.pop(), Some(0)),
                DrawCommand::PopOpacity => assert_eq!(scopes.pop(), Some(2)),
                _ => {}
            }
            assert!(scopes.len() <= 128);
        }
        assert!(saw_fixed && saw_opacity);
        assert!(scopes.is_empty());
        assert!(result.commands.len() <= MAX_COMMANDS);
        assert!(result.hit_regions.len() <= MAX_VISITS);
    }

    #[test]
    fn positioned_axis_clamps_before_solving_and_handles_negative_auto_margin_residue() {
        let axis = PositionedAxis {
            containing: 100.0,
            start: Some(10.0),
            end: Some(10.0),
            size: Some(120.0),
            natural: 0.0,
            min: 0.0,
            max: MAX_EXTENT,
            margin_start: None,
            margin_end: None,
            static_start: 0.0,
            horizontal: true,
        };
        assert_eq!(positioned_axis(axis), (10.0, 120.0));
        assert_eq!(
            positioned_axis(PositionedAxis {
                horizontal: false,
                ..axis
            }),
            (-10.0, 120.0)
        );
        assert_eq!(
            positioned_axis(PositionedAxis { max: 40.0, ..axis }),
            (30.0, 40.0)
        );
        assert_eq!(
            positioned_axis(PositionedAxis {
                size: None,
                min: 120.0,
                ..axis
            }),
            (10.0, 120.0)
        );
    }

    #[test]
    fn absolute_containing_block_skips_static_intermediates_and_uses_padding_box() {
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;width:200px;height:120px;padding:20px;border:5px solid}section{margin:30px;padding:10px}#a{position:absolute;left:10%;right:20px;top:10px;bottom:30px;padding:3px;border:2px solid}</style><main><section><i id=a></i></section></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(29.0, 15.0, 196.0, 120.0));
        assert_eq!(bounds(&doc, &result, "main"), rect(0.0, 0.0, 250.0, 170.0));
    }

    #[test]
    fn absolute_auto_height_ancestor_and_min_max_rebalance_auto_margins() {
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;width:200px;padding:10px}section{height:80px}#a{position:absolute;left:10px;right:10px;top:10px;bottom:10px;max-width:100px;max-height:40px;margin:auto}#b{position:absolute;right:0;bottom:0;width:20px;height:10px}</style><main><section></section><i id=a></i><i id=b></i></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "main").height, 100.0);
        assert_eq!(bounds(&doc, &result, "#a"), rect(60.0, 30.0, 100.0, 40.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(200.0, 90.0, 20.0, 10.0));
    }

    #[test]
    fn absolute_shrink_to_fit_keeps_out_of_flow() {
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;width:200px}#a{position:absolute;padding:5px;right:10px}#a b{display:block;width:60px;height:20px}#after{height:10px}</style><main><aside id=a><b></b></aside><div id=after></div></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(120.0, 0.0, 70.0, 30.0));
        assert_eq!(bounds(&doc, &result, "#after"), rect(0.0, 0.0, 200.0, 10.0));
    }

    #[test]
    fn relative_offsets_preserve_normal_flow_and_inline_containing_geometry() {
        let (doc, result) = render(
            "<style>body{margin:0}#a{position:relative;left:20px;top:30px;height:40px}#b{height:10px}span{position:relative;left:10px;top:5px}i{position:absolute;left:0;top:0;width:5px;height:5px}</style><div id=a></div><div id=b></div><p style='margin:0'><span>word<i id=inside></i></span></p>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(20.0, 30.0, 400.0, 40.0));
        assert_eq!(bounds(&doc, &result, "#b").y, 40.0);
        let text = result
            .hit_regions
            .iter()
            .find(|h| matches!(&doc.nodes[h.node].kind,NodeKind::Text(t) if t=="word"))
            .unwrap();
        assert_eq!(bounds(&doc, &result, "#inside").x, text.rect.x);
        assert_eq!(bounds(&doc, &result, "#inside").y, text.rect.y);
    }

    #[test]
    fn absolute_descendants_escape_static_overflow_but_honor_containing_clip() {
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;width:100px;height:100px;overflow:hidden}section{width:20px;height:20px;overflow:hidden}#a{position:absolute;left:50px;top:50px;width:100px;height:100px;background:red}</style><main><section><a id=a></a></section></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(50.0, 50.0, 50.0, 50.0));
        assert_eq!(result.hit_test(60.0, 60.0), doc.query_selector("#a"));
        assert_ne!(result.hit_test(110.0, 60.0), doc.query_selector("#a"));
    }

    #[test]
    fn absolute_items_leave_flex_and_grid_flow_and_use_positioned_ancestor() {
        for display in ["flex", "grid"] {
            let (doc, result) = render(
                &format!(
                    "<style>body{{margin:0}}main{{position:relative;width:200px;height:100px;padding:10px}}section{{display:{display};grid-template-columns:20px 20px}}b{{width:20px;height:10px}}i{{position:absolute;right:0;bottom:0;width:30px;height:20px}}</style><main><section><b id=a></b><i id=abs></i><b id=b></b></section></main>"
                ),
                400.0,
            );
            assert_eq!(
                bounds(&doc, &result, "#abs"),
                rect(190.0, 100.0, 30.0, 20.0),
                "{display}"
            );
            assert_eq!(bounds(&doc, &result, "#b").x, 30.0, "{display}");
        }
    }

    #[test]
    fn fixed_descendants_escape_overflow_and_do_not_extend_scroll_height() {
        use crate::graphics::{Canvas, ImageStore};
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;overflow:hidden;width:20px;height:20px;margin:100px}#a{position:fixed;left:30px;top:40px;width:40px;height:30px;background:red}#b{position:absolute;left:10px;top:10px;width:10px;height:10px;background:blue}</style><main><div id=a><i id=b></i></div></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(30.0, 40.0, 40.0, 30.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(40.0, 50.0, 10.0, 10.0));
        assert!(
            result
                .hit_regions
                .iter()
                .filter(|h| [
                    doc.query_selector("#a").unwrap(),
                    doc.query_selector("#b").unwrap()
                ]
                .contains(&h.node))
                .all(|h| h.fixed)
        );
        let mut canvas = Canvas::new(200, 100).unwrap();
        canvas.paint_with_viewport(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            (0.0, -100.0),
            (0.0, 0.0),
        );
        assert_eq!(canvas.pixels[45 * 200 + 35], 0xff0000);
        assert_eq!(canvas.pixels[55 * 200 + 45], 0x0000ff);
        assert_eq!(result.content_height, 400.0);
    }

    fn pixel(result: &LayoutResult, x: usize, y: usize) -> u32 {
        use crate::graphics::{Canvas, ImageStore};
        let mut canvas = Canvas::new(200, 150).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        canvas.pixels[y * 200 + x]
    }

    #[test]
    fn opacity_wraps_background_border_descendants_and_preserves_zero_hits() {
        let (doc, result) = render(
            "<style>body{margin:0}main{opacity:.5;background:red;border:10px solid lime;width:80px;height:60px}div{background:blue;width:40px;height:40px}#zero{opacity:0;position:absolute;left:0;top:100px;width:30px;height:20px}</style><main><div></div></main><button id=zero>invisible</button>",
            200.0,
        );
        assert_eq!(pixel(&result, 4, 4), 0x80ff80);
        assert_eq!(pixel(&result, 20, 20), 0x8080ff);
        assert_eq!(pixel(&result, 70, 20), 0xff8080);
        assert_eq!(pixel(&result, 10, 110), 0xffffff);
        let zero = doc.query_selector("#zero").unwrap();
        let hit = result.hit_test(10.0, 110.0).unwrap();
        assert!(hit == zero || doc.nodes[hit].parent == Some(zero));
        assert_eq!(
            result
                .commands
                .iter()
                .filter(|c| matches!(c, DrawCommand::PushOpacity { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn nested_opacity_groups_isolate_high_z_descendants_and_follow_source_order() {
        let (doc, result) = render(
            "<style>body{margin:0}main{opacity:.5;background:red;position:relative;width:100px;height:80px}section{opacity:.5;position:absolute;left:0;top:0;width:40px;height:40px}section i{position:absolute;z-index:999;left:0;top:0;width:40px;height:40px;background:blue}aside{position:absolute;left:20px;top:0;width:20px;height:40px;background:lime}</style><main><section><i id=nested></i></section><aside id=top></aside></main>",
            200.0,
        );
        assert_eq!(pixel(&result, 10, 10), 0xbf80bf);
        assert_eq!(pixel(&result, 30, 10), 0x80ff80);
        assert_eq!(pixel(&result, 60, 10), 0xff8080);
        assert_eq!(result.hit_test(30.0, 10.0), doc.query_selector("#top"));
    }

    #[test]
    fn inline_opacity_encloses_wrapped_runs_once_and_keeps_primitive_alpha() {
        let (_, result) = render(
            "<style>body{margin:0;width:75px;font:20px monospace}span{opacity:.5;background:red;color:blue}</style><span>alpha beta gamma delta</span>",
            200.0,
        );
        assert_eq!(
            result
                .commands
                .iter()
                .filter(|c| matches!(c, DrawCommand::PushOpacity { .. }))
                .count(),
            1
        );
        let mut text_lines = std::collections::HashSet::new();
        for command in &result.commands {
            if let DrawCommand::Text { y, color, .. } = command {
                text_lines.insert(y.to_bits());
                assert_eq!(*color, Color::rgb(0, 0, 255));
            }
        }
        assert!(text_lines.len() >= 3);
        assert!(result.commands.iter().any(
            |c| matches!(c, DrawCommand::Rect { color, .. } if *color == Color::rgb(255, 0, 0))
        ));
    }

    #[test]
    fn fixed_opacity_descendant_stays_faded_under_scroll_and_outside_ancestor_clip() {
        use crate::graphics::{Canvas, ImageStore};
        let (doc, result) = render(
            "<style>body{margin:0}main{opacity:.5;overflow:hidden;width:10px;height:10px;margin-top:100px}b{position:fixed;left:20px;top:20px;width:40px;height:40px;background:blue}</style><main><b id=fixed></b></main>",
            200.0,
        );
        let mut canvas = Canvas::new(200, 150).unwrap();
        canvas.paint_with_viewport(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            (0.0, -100.0),
            (0.0, 0.0),
        );
        assert_eq!(canvas.pixels[30 * 200 + 30], 0x8080ff);
        assert!(!canvas.exhausted());
        assert!(
            result
                .hit_regions
                .iter()
                .any(|hit| hit.node == doc.query_selector("#fixed").unwrap() && hit.fixed)
        );
    }

    #[test]
    fn decoded_transparent_images_do_not_paint_missing_image_fallbacks() {
        use crate::graphics::{Canvas, ImageStore, RasterImage};
        let (_, result) = render(
            "<style>body{margin:0}main{opacity:.5;background:blue;width:80px;height:40px}img{display:block;width:40px;height:40px}</style><main><img src=alpha alt=missing data-eris-natural-width=1 data-eris-natural-height=1></main>",
            200.0,
        );
        assert!(
            !result.commands.iter().any(
                |command| matches!(command, DrawCommand::Text { text, .. } if text == "missing")
            )
        );
        let mut images = ImageStore::new();
        images.insert(
            "alpha".into(),
            std::sync::Arc::new(RasterImage {
                width: 1,
                height: 1,
                rgba: vec![255, 0, 0, 128],
            }),
        );
        let mut canvas = Canvas::new(200, 150).unwrap();
        canvas.paint(&result.commands, &Fonts::new(), &images, 0.0, 0.0);
        assert_eq!(canvas.pixels[0], 0xc080bf);
        assert_eq!(canvas.pixels[60], 0x8080ff);
        assert!(!canvas.exhausted());
    }

    #[test]
    fn root_opacity_contains_propagated_background_and_row_background_has_its_owner() {
        let (_, root) = render(
            "<style>html{opacity:.5}body{margin:0;background:red}div{background:blue;width:40px;height:40px}</style><div></div>",
            200.0,
        );
        assert_eq!(pixel(&root, 10, 10), 0x8080ff);
        assert_eq!(pixel(&root, 100, 100), 0xff8080);
        let (_, table) = render(
            "<style>body{margin:0}table{width:80px;border-spacing:0}tr{opacity:.5;background:red}td{width:40px;height:40px;padding:0;background:blue}</style><table><tr><td></td><td></td></tr></table>",
            200.0,
        );
        assert_eq!(pixel(&table, 10, 10), 0x8080ff);
    }

    #[test]
    fn stacking_levels_isolate_real_contexts_and_hit_the_top_painted_box() {
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;z-index:0;width:100px;height:100px}div{position:absolute;width:80px;height:80px;left:0;top:0}#low{z-index:1;background:red}#high{z-index:2;background:blue}#nested{z-index:999;background:lime}</style><main><div id=high></div><div id=low><div id=nested></div></div></main>",
            200.0,
        );
        assert_eq!(pixel(&result, 10, 10), 0x0000ff);
        assert_eq!(result.hit_test(10.0, 10.0), doc.query_selector("#high"));
    }

    #[test]
    fn positioned_auto_context_allows_real_descendant_to_escape_and_keeps_clip() {
        let (doc, result) = render(
            "<style>body{margin:0}main{position:relative;width:100px;height:100px}div{position:absolute;left:0;top:0;width:80px;height:80px}#auto{overflow:hidden;width:40px}#high{z-index:2;background:blue}#nested{z-index:9;background:lime}</style><main><div id=auto><div id=nested></div></div><div id=high></div></main>",
            200.0,
        );
        assert_eq!(pixel(&result, 10, 10), 0x00ff00);
        assert_eq!(pixel(&result, 60, 10), 0x0000ff);
        assert_eq!(result.hit_test(10.0, 10.0), doc.query_selector("#nested"));
        assert_eq!(result.hit_test(60.0, 10.0), doc.query_selector("#high"));
    }

    #[test]
    fn negative_context_paints_above_own_context_background_below_normal_children() {
        let (_, result) = render(
            "<style>body{margin:0}main{position:relative;z-index:0;width:100px;height:100px;background:red}#neg{position:absolute;z-index:-1;left:0;top:0;width:80px;height:80px;background:lime}#flow{width:40px;height:40px;background:blue}</style><main><i id=neg></i><div id=flow></div></main>",
            200.0,
        );
        assert_eq!(pixel(&result, 10, 10), 0x0000ff);
        assert_eq!(pixel(&result, 60, 10), 0x00ff00);
        assert_eq!(pixel(&result, 90, 10), 0xff0000);
    }

    #[test]
    fn grid_item_z_index_creates_context_without_position_and_respects_dom_order() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:80px}div{grid-area:1/1;width:80px;height:80px}#a{z-index:2;background:red}#b{z-index:1;background:blue}</style><main><div id=a></div><div id=b></div></main>",
            200.0,
        );
        assert_eq!(pixel(&result, 10, 10), 0xff0000);
        assert_eq!(result.hit_test(10.0, 10.0), doc.query_selector("#a"));
    }

    #[test]
    fn floats_exclude_line_bands_and_restore_full_width_below_margin_boxes() {
        let mut source = String::from(
            "<style>body{margin:0}main{width:240px}aside{float:left;width:60px;height:30px;margin-right:20px;margin-bottom:10px}i{display:inline-block;width:40px;height:20px}</style><main><aside id=f></aside>",
        );
        for id in 0..14 {
            source.push_str(&format!("<i id=t{id}></i>"));
        }
        source.push_str("</main>");
        let (doc, result) = render(&source, 320.0);
        assert_eq!(bounds(&doc, &result, "#t0").x, 80.0);
        assert_eq!(bounds(&doc, &result, "#t3").x, 200.0);
        assert_eq!(bounds(&doc, &result, "#t4").x, 80.0);
        assert_eq!(bounds(&doc, &result, "#t4").y, 20.0);
        assert_eq!(bounds(&doc, &result, "#t8").x, 0.0);
        assert_eq!(bounds(&doc, &result, "#t8").y, 40.0);
        assert_eq!(bounds(&doc, &result, "main").height, 60.0);
    }

    #[test]
    fn same_and_opposite_floats_pack_then_drop_to_the_first_available_bottom() {
        let (doc, result) = render(
            "<style>body{margin:0}main{width:240px}aside{float:left;margin:5px}#a{width:70px;height:30px}#b{width:90px;height:20px}#c{float:right;width:40px;height:60px}#d{width:100px;height:10px}section{height:10px}#left{clear:left}#both{clear:both}</style><main><aside id=a></aside><aside id=b></aside><aside id=c></aside><aside id=d></aside><section id=left></section><section id=both></section></main>",
            320.0,
        );
        for (id, x, y) in [
            ("#a", 5.0, 5.0),
            ("#b", 85.0, 5.0),
            ("#c", 195.0, 5.0),
            ("#d", 85.0, 35.0),
        ] {
            let r = bounds(&doc, &result, id);
            assert_eq!((r.x, r.y), (x, y), "{id}");
        }
        assert_eq!(bounds(&doc, &result, "#left").y, 50.0);
        assert_eq!(bounds(&doc, &result, "#both").y, 70.0);
    }

    #[test]
    fn clear_right_and_floating_clear_respect_only_the_selected_side() {
        let (doc, result) = render(
            "<style>body{margin:0}#l{float:left;width:40px;height:80px}#r{float:right;width:40px;height:20px}#s{clear:right;height:10px}#f{float:left;clear:left;width:30px;height:15px}</style><div id=l></div><div id=r></div><section id=s></section><div id=f></div>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#s").y, 20.0);
        assert_eq!(bounds(&doc, &result, "#f").y, 80.0);
    }

    #[test]
    fn floats_escape_ordinary_block_height_and_wrap_later_nested_text() {
        let (doc, result) = render(
            "<style>body,p{margin:0}#f{float:left;width:80px;height:70px}p{font-size:10px;line-height:20px}</style><div id=outer><div id=f></div></div><section id=next><p>one two three four five six seven eight nine ten eleven twelve thirteen fourteen</p></section>",
            160.0,
        );
        assert_eq!(bounds(&doc, &result, "#outer").height, 0.0);
        assert_eq!(bounds(&doc, &result, "#next").y, 0.0);
        let lines: Vec<_> = result
            .commands
            .iter()
            .filter_map(|c| {
                if let DrawCommand::Text { x, y, text, .. } = c {
                    Some((*x, *y, text))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(lines[0].0, 80.0);
        assert!(lines.iter().any(|(x, y, _)| *x == 0.0 && *y >= 70.0));
    }

    #[test]
    fn fitting_float_after_inline_text_repositions_that_same_line() {
        let (doc, result) = render(
            "<style>body{margin:0}main{width:200px;font-size:10px;line-height:20px}aside{float:left;width:60px;height:40px}</style><main>before<aside id=f></aside> after</main>",
            240.0,
        );
        let before = result
            .commands
            .iter()
            .find_map(|c| {
                if let DrawCommand::Text { x, y, text, .. } = c {
                    (text == "before").then_some((*x, *y))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(before.0, 60.0);
        assert!(before.1 < 20.0);
        assert_eq!(bounds(&doc, &result, "#f").y, 0.0);
    }

    #[test]
    fn unbreakable_text_moves_below_floats_instead_of_overlapping_them() {
        let (_, result) = render(
            "<style>body{margin:0}aside{float:left;width:190px;height:40px}p{margin:0;font-size:10px;line-height:20px}</style><aside></aside><p>longword</p>",
            200.0,
        );
        let (x, y) = result
            .commands
            .iter()
            .find_map(|c| {
                if let DrawCommand::Text { x, y, text, .. } = c {
                    (text == "longword").then_some((*x, *y))
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(x, 0.0);
        assert!(y >= 40.0);
    }

    #[test]
    fn percentage_inline_margins_keep_the_containing_width_beside_floats() {
        let (doc, result) = render(
            "<style>body{margin:0}aside{float:left;width:80px;height:50px}i{display:inline-block;width:20px;height:20px;margin-left:10%}</style><aside></aside><i id=a></i>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").x, 100.0);
    }

    #[test]
    fn float_placement_uses_the_full_mixed_inline_baseline_and_descent() {
        let (doc, result) = render(
            "<style>body{margin:0;font:10px monospace;line-height:20px}i{display:inline-block;width:40px;height:40px}aside{float:left;width:160px;height:20px}</style><i></i>M<aside id=f></aside>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#f").y, 47.0);
    }

    #[test]
    fn nowrap_runs_move_below_floats_without_splitting_words_into_lines() {
        let (_, result) = render(
            "<style>body{margin:0;font:10px monospace;line-height:20px}aside{float:right;width:160px;height:40px}span{white-space:nowrap}</style><aside></aside><span>AA AA AA</span>",
            200.0,
        );
        let ys: Vec<_> = result
            .commands
            .iter()
            .filter_map(|c| {
                if let DrawCommand::Text { y, text, .. } = c {
                    (!text.trim().is_empty()).then_some(*y)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(ys.len(), 3);
        assert!(ys.iter().all(|y| *y >= 40.0 && *y == ys[0]));
    }

    #[test]
    fn shrink_to_fit_uses_preferred_minimum_available_width_and_box_model() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flow-root;width:200px}#small{width:70px}aside{float:left;padding:10px}i{display:inline-block;width:40px;height:20px}</style><main><aside id=a><i></i><i></i></aside></main><main id=small><aside id=b><i></i><i></i></aside></main>",
            240.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 100.0);
        assert_eq!(bounds(&doc, &result, "#a").height, 40.0);
        assert_eq!(bounds(&doc, &result, "#b").width, 70.0);
        assert_eq!(bounds(&doc, &result, "#b").height, 60.0);
        assert_eq!(bounds(&doc, &result, "#small").y, 40.0);
        assert_eq!(bounds(&doc, &result, "#small").height, 60.0);
    }

    #[test]
    fn independent_contexts_contain_internal_floats_without_leaking_exclusions() {
        for rule in [
            "display:flow-root",
            "overflow:hidden",
            "display:inline-block",
        ] {
            let source = format!(
                "<style>body{{margin:0}}#box{{{rule};width:120px}}aside{{float:left;width:80px;height:40px}}#next{{height:10px}}</style><div id=box><aside></aside></div><div id=next></div>"
            );
            let (doc, result) = render(&source, 200.0);
            assert_eq!(bounds(&doc, &result, "#box").height, 40.0, "{rule}");
            assert_eq!(bounds(&doc, &result, "#next").y, 40.0, "{rule}");
        }
        let (doc, result) = render(
            "<style>body{margin:0}aside{float:left;width:80px;height:50px}section{overflow:hidden;height:30px}i{float:left;width:20px;height:20px}</style><aside></aside><section id=b><i id=inner></i></section><div id=clear style='clear:both;height:1px'></div>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#b").x, 80.0);
        assert_eq!(bounds(&doc, &result, "#b").width, 120.0);
        assert_eq!(bounds(&doc, &result, "#inner").x, 80.0);
        assert_eq!(bounds(&doc, &result, "#clear").y, 50.0);
    }

    #[test]
    fn overflow_clip_does_not_contain_floats_but_clips_deferred_hits() {
        let (doc, result) = render(
            "<style>body{margin:0}#clip{overflow:clip;width:40px;height:20px}#f{float:left;width:80px;height:60px;background:red}#after{clear:both;height:10px}</style><div id=clip><a id=f href='#x'></a></div><div id=after></div>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#after").y, 60.0);
        let f = bounds(&doc, &result, "#f");
        assert_eq!((f.width, f.height), (40.0, 20.0));
        assert_ne!(result.hit_test(50.0, 10.0), doc.query_selector("#f"));
    }

    #[test]
    fn float_inventory_and_pairwise_exclusion_work_are_bounded() {
        let source = format!(
            "<style>body{{margin:0}}i{{float:left;width:1px;height:1px}}</style>{}",
            "<i></i>".repeat(6000)
        );
        let (_, result) = render(&source, 100.0);
        assert!(result.commands.len() <= MAX_COMMANDS);
        assert!(
            result
                .hit_regions
                .iter()
                .all(|h| h.rect.x.is_finite() && h.rect.y.is_finite())
        );
        assert!(result.hit_regions.len() < 5000);
    }

    #[test]
    fn command_quota_preserves_all_closures_in_nested_clipped_float_fragments() {
        let mut source = String::from(
            "<style>body{margin:0}.clip{overflow:hidden;width:40px}.float{float:left;width:40px}.leaf{height:1px;overflow:hidden}</style>",
        );
        for _ in 0..16 {
            source.push_str("<div class=clip><div class=float>");
        }
        source.push_str(&"<div class=leaf>x</div>".repeat(30_000));
        for _ in 0..16 {
            source.push_str("</div></div>");
        }
        let (doc, result) = render(&source, 200.0);
        let leaves = result
            .hit_regions
            .iter()
            .filter(|h| doc.attr(h.node, "class") == Some("leaf"))
            .count();
        assert!(
            (20_000..30_000).contains(&leaves),
            "quota must actually truncate leaf layout: {leaves}"
        );
        let mut open = 0usize;
        for command in &result.commands {
            match command {
                DrawCommand::PushClip { .. } => open += 1,
                DrawCommand::PopClip => {
                    assert!(open > 0, "clip stack underflow");
                    open -= 1;
                }
                _ => {}
            }
        }
        assert_eq!(open, 0, "quota must reserve every closing clip");
        assert!(result.commands.len() <= MAX_COMMANDS);
    }

    #[test]
    fn flex_grow_redistributes_after_maximum_freezes() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;width:300px}i{flex:1 1 50px;height:20px}#a{max-width:60px}</style><main><i id=a></i><i id=b></i><i id=c></i></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 60.0);
        assert_eq!(bounds(&doc, &result, "#b").width, 120.0);
        assert_eq!(bounds(&doc, &result, "#c").x, 180.0);
    }

    #[test]
    fn flex_shrink_redistributes_after_minimum_freezes() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;width:180px}i{width:100px;height:20px;min-width:0}#a{min-width:90px}</style><main><i id=a></i><i id=b></i><i id=c></i></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 90.0);
        assert_eq!(bounds(&doc, &result, "#b").width, 45.0);
        assert_eq!(bounds(&doc, &result, "#c").x, 135.0);
    }

    #[test]
    fn flex_shrink_weights_use_inner_basis_without_padding() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;width:240px}i{height:20px;min-width:0}#a{width:100px;padding:0 20px}#b{width:200px}</style><main><i id=a></i><i id=b></i></main>",
            400.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        assert!((a.width - 106.66667).abs() < 0.001, "{a:?}");
        assert!((b.width - 133.33333).abs() < 0.001, "{b:?}");
        assert!((b.x + b.width - 240.0).abs() < 0.001);
    }

    #[test]
    fn fractional_flex_factors_can_leave_free_space_or_overflow() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;width:200px}i{flex:.25 1 0px;height:20px}section{display:flex;width:100px}b{width:100px;height:20px;flex-shrink:.25;min-width:0}</style><main><i id=a></i><i id=b></i></main><section><b id=c></b><b id=d></b></section>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 50.0);
        assert_eq!(bounds(&doc, &result, "#b").x, 50.0);
        assert_eq!(bounds(&doc, &result, "#c").width, 75.0);
        assert_eq!(bounds(&doc, &result, "#d").x, 75.0);
    }

    #[test]
    fn flex_reverse_starts_at_main_end_and_preserves_paint_order() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-direction:row-reverse;width:300px;gap:10px}i{width:40px;height:20px}#b{width:60px}section{display:flex;flex-direction:row-reverse;width:200px}b{width:80px;height:20px}#d{margin-right:-40px}</style><main><i id=a></i><i id=b></i></main><section><b id=c></b><b id=d></b></section>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").x, 260.0);
        assert_eq!(bounds(&doc, &result, "#b").x, 190.0);
        assert_eq!(result.hit_test(130.0, 25.0), doc.query_selector("#d"));
    }

    #[test]
    fn flex_order_is_stable_and_does_not_mutate_dom_order() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;gap:10px}i{width:40px;height:20px}#b{order:-1}#c{order:1}</style><main><i id=a></i><!-- no flex item --><i id=b></i><i id=c></i><i id=d></i></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#b").x, 0.0);
        assert_eq!(bounds(&doc, &result, "#a").x, 50.0);
        assert_eq!(bounds(&doc, &result, "#d").x, 100.0);
        assert_eq!(bounds(&doc, &result, "#c").x, 150.0);
        assert!(doc.query_selector("#a").unwrap() < doc.query_selector("#b").unwrap());
        assert!(result.hit_regions.iter().all(|h| !matches!(
            doc.nodes[h.node].kind,
            NodeKind::Comment(_) | NodeKind::Doctype(_)
        )));
    }

    #[test]
    fn flex_column_reverse_resolves_growth_and_nested_definite_height() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-direction:column-reverse;width:100px;height:300px}section{display:flex;flex-direction:column;flex:1 1 50px;min-height:0}#a{max-height:80px}i{flex:1 1 0px;min-height:0}</style><main><section id=a><i id=inner></i></section><section id=b></section></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").height, 80.0);
        assert_eq!(bounds(&doc, &result, "#a").y, 220.0);
        assert_eq!(bounds(&doc, &result, "#b").height, 220.0);
        assert_eq!(bounds(&doc, &result, "#b").y, 0.0);
        assert_eq!(bounds(&doc, &result, "#inner").height, 80.0);
    }

    #[test]
    fn flex_align_self_and_cross_axis_limits_use_container_height() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;height:100px}i{width:50px}#a{height:20px;align-self:center}#b{max-height:40px}#c{height:10px;align-self:flex-end}#d{height:20px;margin-top:auto;margin-bottom:auto}</style><main><i id=a></i><i id=b></i><i id=c></i><i id=d></i></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").y, 40.0);
        assert_eq!(bounds(&doc, &result, "#b").height, 40.0);
        assert_eq!(bounds(&doc, &result, "#c").y, 90.0);
        assert_eq!(bounds(&doc, &result, "#d").y, 40.0);
    }

    #[test]
    fn flex_wrap_uses_hypothetical_size_and_auto_margins_use_remainder() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-wrap:wrap;width:100px;gap:10px}i{flex-basis:0;min-width:80px;height:20px}section{display:flex;width:200px}b{width:40px;height:20px}#d{margin-left:auto}</style><main><i id=a></i><i id=b></i></main><section><b id=c></b><b id=d></b></section>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 80.0);
        assert_eq!(bounds(&doc, &result, "#b").y, 30.0);
        assert_eq!(bounds(&doc, &result, "#d").x, 160.0);
    }

    #[test]
    fn flex_column_wrap_and_both_reverse_axes_keep_independent_gaps() {
        for (direction, wrap, expected) in [
            ("column", "wrap", [(0.0, 0.0), (0.0, 50.0), (70.0, 0.0)]),
            (
                "column-reverse",
                "wrap",
                [(0.0, 60.0), (0.0, 10.0), (70.0, 60.0)],
            ),
            (
                "column",
                "wrap-reverse",
                [(170.0, 0.0), (150.0, 50.0), (100.0, 0.0)],
            ),
            (
                "column-reverse",
                "wrap-reverse",
                [(170.0, 60.0), (150.0, 10.0), (100.0, 60.0)],
            ),
        ] {
            let (doc, result) = render(
                &format!(
                    "<style>body{{margin:0}}main{{display:flex;flex-direction:{direction};flex-wrap:{wrap};width:200px;height:100px;gap:10px 20px;align-content:flex-start;align-items:flex-start}}i{{width:30px;height:40px}}#b{{width:50px}}</style><main><i id=a></i><i id=b></i><i id=c></i></main>"
                ),
                400.0,
            );
            for (id, position) in ["#a", "#b", "#c"].into_iter().zip(expected) {
                let item = bounds(&doc, &result, id);
                assert_eq!((item.x, item.y), position, "{direction} {wrap} {id}");
            }
        }
    }

    #[test]
    fn flex_column_lines_resolve_freezing_and_auto_margins_independently() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-direction:column;flex-wrap:wrap;width:160px;height:100px;gap:10px 20px;align-content:flex-start}i{width:40px;flex:1 1 40px;min-height:0}#a{max-height:42px}section{display:flex;flex-direction:column;flex-wrap:wrap;width:120px;height:100px;gap:10px 20px;align-content:flex-start}b{width:40px;height:30px;margin-top:auto}</style><main><i id=a></i><i id=b></i><i id=c></i><i id=d></i></main><section><b id=e></b><b id=f></b><b id=g></b></section>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(0.0, 0.0, 40.0, 42.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(0.0, 52.0, 40.0, 48.0));
        assert_eq!(bounds(&doc, &result, "#c"), rect(60.0, 0.0, 40.0, 45.0));
        assert_eq!(bounds(&doc, &result, "#d"), rect(60.0, 55.0, 40.0, 45.0));
        assert_eq!(bounds(&doc, &result, "#e"), rect(0.0, 115.0, 40.0, 30.0));
        assert_eq!(bounds(&doc, &result, "#f"), rect(0.0, 170.0, 40.0, 30.0));
        assert_eq!(bounds(&doc, &result, "#g"), rect(60.0, 170.0, 40.0, 30.0));
    }

    #[test]
    fn flex_column_indefinite_height_uses_content_and_does_not_wrap_at_viewport() {
        for direction in ["column", "column-reverse"] {
            let (doc, result) = render(
                &format!(
                    "<style>body{{margin:0}}main{{display:flex;flex-direction:{direction};flex-wrap:wrap;width:100px;row-gap:10%;align-content:flex-start}}i{{flex-basis:50%;width:20px}}b{{display:block;height:250px}}</style><main id=main><i id=a><b></b></i><i id=b><b></b></i></main>"
                ),
                200.0,
            );
            let a = bounds(&doc, &result, "#a");
            let b = bounds(&doc, &result, "#b");
            assert_eq!(bounds(&doc, &result, "#main").height, 500.0);
            assert_eq!((a.x, b.x, a.height, b.height), (0.0, 0.0, 250.0, 250.0));
            assert_eq!(
                (a.y, b.y),
                if direction == "column" {
                    (0.0, 250.0)
                } else {
                    (250.0, 0.0)
                }
            );
        }
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-direction:column;flex-wrap:wrap;width:100px;height:100px;row-gap:10%;column-gap:20%;align-content:flex-start}i{flex-basis:50%;width:20px}</style><main><i id=a></i><i id=b></i></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(0.0, 0.0, 20.0, 50.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(40.0, 0.0, 20.0, 50.0));
    }

    #[test]
    fn flex_column_max_height_limits_lines_without_resolving_percentage_bases() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-direction:column;flex-wrap:wrap;width:120px;max-height:124px;padding:10px;border:2px solid red;box-sizing:border-box;gap:10px;align-content:flex-start}i{flex:0 1 50%;width:20px}b{display:block;height:40px}</style><main id=main><i id=a><b></b></i><i id=b><b></b></i><i id=c><b></b></i></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#main"), rect(0.0, 0.0, 120.0, 124.0));
        assert_eq!(bounds(&doc, &result, "#a"), rect(12.0, 12.0, 20.0, 40.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(12.0, 62.0, 20.0, 40.0));
        assert_eq!(bounds(&doc, &result, "#c"), rect(42.0, 12.0, 20.0, 40.0));
    }

    #[test]
    fn flex_min_height_distributes_main_space_and_stretches_wrapped_cross_lines() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-direction:column;flex-wrap:wrap;width:100px;min-height:100px;max-height:50px;justify-content:space-between;align-content:flex-start}i{width:20px;flex-basis:50%}b{display:block;height:20px}section{display:flex;flex-wrap:wrap;width:40px;min-height:100px;row-gap:10px;align-items:flex-start}section i{width:30px;flex-basis:auto;height:10px}#d{height:20px}</style><main id=main><i id=a><b></b></i><i id=b><b></b></i></main><section><i id=c></i><i id=d></i></section>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#main").height, 100.0);
        assert_eq!(bounds(&doc, &result, "#a"), rect(0.0, 0.0, 20.0, 20.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(0.0, 80.0, 20.0, 20.0));
        assert_eq!(bounds(&doc, &result, "#c"), rect(0.0, 100.0, 30.0, 10.0));
        assert_eq!(bounds(&doc, &result, "#d"), rect(0.0, 150.0, 30.0, 20.0));
    }

    #[test]
    fn flex_align_content_distributes_and_stretches_lines_on_both_axes() {
        for (align, forward, reversed) in [
            ("flex-start", [0.0, 20.0], [90.0, 60.0]),
            ("flex-end", [60.0, 80.0], [30.0, 0.0]),
            ("start", [0.0, 20.0], [30.0, 0.0]),
            ("end", [60.0, 80.0], [90.0, 60.0]),
            ("center", [30.0, 50.0], [60.0, 30.0]),
            ("space-between", [0.0, 80.0], [90.0, 0.0]),
            ("space-around", [15.0, 65.0], [75.0, 15.0]),
            ("space-evenly", [20.0, 60.0], [70.0, 20.0]),
            ("normal", [0.0, 50.0], [90.0, 30.0]),
            ("stretch", [0.0, 50.0], [90.0, 30.0]),
        ] {
            for column in [false, true] {
                let (direction, container, item, second) = if column {
                    (
                        "column",
                        "width:100px;height:45px;column-gap:10px",
                        "height:30px;width:10px",
                        "width:20px",
                    )
                } else {
                    (
                        "row",
                        "width:45px;height:100px;row-gap:10px",
                        "width:30px;height:10px",
                        "height:20px",
                    )
                };
                for (wrap, expected) in [("wrap", forward), ("wrap-reverse", reversed)] {
                    let (doc, result) = render(
                        &format!(
                            "<style>body{{margin:0}}main{{display:flex;flex-direction:{direction};flex-wrap:{wrap};{container};align-items:flex-start;align-content:{align}}}i{{{item}}}#b{{{second}}}</style><main><i id=a></i><i id=b></i></main>"
                        ),
                        200.0,
                    );
                    for (id, position) in ["#a", "#b"].into_iter().zip(expected) {
                        let item = bounds(&doc, &result, id);
                        assert_eq!(
                            if column { item.x } else { item.y },
                            position,
                            "{direction} {wrap} {align} {id}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn flex_align_content_overflow_uses_positional_or_safe_distribution_fallbacks() {
        for (align, forward, reversed) in [
            ("flex-start", [0.0, 50.0], [20.0, -40.0]),
            ("flex-end", [-40.0, 10.0], [60.0, 0.0]),
            ("center", [-20.0, 30.0], [40.0, -20.0]),
            ("unsafe center", [-20.0, 30.0], [40.0, -20.0]),
            ("safe center", [0.0, 50.0], [60.0, 0.0]),
            ("safe flex-end", [0.0, 50.0], [60.0, 0.0]),
            ("space-between", [0.0, 50.0], [60.0, 0.0]),
            ("space-around", [0.0, 50.0], [60.0, 0.0]),
            ("space-evenly", [0.0, 50.0], [60.0, 0.0]),
            ("stretch", [0.0, 50.0], [20.0, -40.0]),
        ] {
            for (wrap, expected) in [("wrap", forward), ("wrap-reverse", reversed)] {
                let (doc, result) = render(
                    &format!(
                        "<style>body{{margin:0}}main{{display:flex;flex-wrap:{wrap};width:40px;height:60px;margin-top:100px;row-gap:10px;align-content:{align}}}i{{width:30px;height:40px}}#b{{height:50px}}</style><main><i id=a></i><i id=b></i></main>"
                    ),
                    200.0,
                );
                assert_eq!(
                    bounds(&doc, &result, "#a").y,
                    100.0 + expected[0],
                    "{wrap} {align} a"
                );
                assert_eq!(
                    bounds(&doc, &result, "#b").y,
                    100.0 + expected[1],
                    "{wrap} {align} b"
                );
            }
        }
    }

    #[test]
    fn flex_column_line_stretch_honors_box_constraints_and_cross_auto_margins() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-direction:column;flex-wrap:wrap;width:200px;height:100px;gap:10px 20px}i{height:40px;padding:5px;border:2px solid red;box-sizing:border-box}b{display:block;width:26px;height:10px}#a{max-width:64px}#b{width:40px;margin-left:auto}#c b{width:46px}</style><main><i id=a><b></b></i><i id=b><b></b></i><i id=c><b></b></i></main>",
            400.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(0.0, 0.0, 64.0, 40.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(40.0, 50.0, 40.0, 40.0));
        assert_eq!(bounds(&doc, &result, "#c"), rect(100.0, 0.0, 100.0, 40.0));
    }

    #[test]
    fn flex_align_content_applies_to_one_wrapped_line_but_not_nowrap() {
        for (wrap, expected) in [("wrap", 40.0), ("nowrap", 0.0)] {
            let (doc, result) = render(
                &format!(
                    "<style>body{{margin:0}}main{{display:flex;flex-wrap:{wrap};width:100px;height:100px;align-content:center;align-items:flex-start}}i{{width:30px;height:20px}}</style><main><i id=a></i></main>"
                ),
                200.0,
            );
            assert_eq!(bounds(&doc, &result, "#a").y, expected);
        }
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-wrap:wrap;width:40px;height:100px;row-gap:10px}i{width:30px}#a{min-height:10px}#b{min-height:20px;max-height:40px}</style><main><i id=a></i><i id=b></i></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(0.0, 0.0, 30.0, 40.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(0.0, 50.0, 30.0, 40.0));
    }

    #[test]
    fn flex_column_wrapping_command_exhaustion_keeps_clips_balanced_and_geometry_finite() {
        let source = format!(
            "<style>body{{margin:0}}main{{display:flex;flex-direction:column;flex-wrap:wrap-reverse;width:100px;height:2px}}i{{display:block;width:2px;height:1px;overflow:hidden;background:red}}</style><main>{}</main>",
            "<i></i>".repeat(35_000)
        );
        let (doc, result) = render(&source, 200.0);
        let painted = result
            .hit_regions
            .iter()
            .filter(|hit| doc.tag(hit.node) == Some("i"))
            .count();
        assert!(
            (1..35_000).contains(&painted),
            "expected actual quota truncation: {painted}"
        );
        assert!(result.commands.len() <= MAX_COMMANDS);
        let mut open = 0;
        for command in &result.commands {
            match command {
                DrawCommand::PushClip { .. } => open += 1,
                DrawCommand::PopClip => {
                    assert!(open > 0);
                    open -= 1;
                }
                _ => {}
            }
        }
        assert_eq!(open, 0);
        assert!(result.hit_regions.iter().all(|hit| {
            [hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height]
                .iter()
                .all(|value| value.is_finite())
        }));
    }

    #[test]
    fn flexible_length_solver_is_bounded_even_with_no_work_left() {
        let items = vec![
            FlexSize {
                base: 100.0,
                inner: 100.0,
                min: 20.0,
                max: 80.0,
                grow: 1.0,
                shrink: 1.0
            };
            1000
        ];
        let mut work = 0;
        let sizes = resolve_flexible_lengths(&items, 20_000.0, &mut work);
        assert!(
            sizes
                .iter()
                .all(|size| size.is_finite() && *size >= 20.0 && *size <= 80.0)
        );
        assert_eq!(work, 0);
    }

    #[test]
    fn block_box_model_and_auto_margins() {
        let (doc, result) = render(
            "<style>body{margin:0}#box{width:100px;height:30px;padding:10px;border:2px solid red;margin-left:auto;margin-right:auto}</style><div id=box></div>",
            300.0,
        );
        let bounds = bounds(&doc, &result, "#box");
        assert_eq!(bounds.width, 124.0);
        assert_eq!(bounds.height, 54.0);
        assert_eq!(bounds.x, 88.0);
    }

    #[test]
    fn text_wraps_without_losing_words() {
        let (_, result) = render(
            "<style>body{margin:0}p{margin:0;font-size:16px}</style><p>alpha beta gamma delta epsilon</p>",
            100.0,
        );
        let words: Vec<(&str, f32)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, y, .. } if !text.trim().is_empty() => {
                    Some((text.as_str(), *y))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            words.iter().map(|(text, _)| *text).collect::<Vec<_>>(),
            ["alpha", "beta", "gamma", "delta", "epsilon"]
        );
        assert!(words.last().unwrap().1 > words.first().unwrap().1);
        assert!(words.windows(2).all(|pair| pair[1].1 >= pair[0].1));
    }

    #[test]
    fn flex_grow_distributes_remaining_space() {
        let (doc, result) = render(
            "<style>body{margin:0}#row{display:flex;width:300px;gap:10px}#row div{width:50px;height:20px;flex-grow:1}</style><div id=row><div id=a></div><div id=b></div></div>",
            400.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        assert_eq!(a.width, 145.0);
        assert_eq!(b.width, 145.0);
        assert_eq!(b.x - a.x, 155.0);
        assert_eq!(a.y, b.y);
    }

    #[test]
    fn grid_numeric_lines_spans_and_negative_lines_use_explicit_grid() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;width:250px;grid-template-columns:50px 70px 90px;grid-template-rows:20px 30px;gap:5px 10px}#a{grid-column:2 / span 2;grid-row:2}#b{grid-column:-4 / -3;grid-row:1 / 3}#c{grid-column:3 / 2;grid-row:1}</style><main><i id=a></i><i id=b></i><i id=c></i></main>",
            300.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(60.0, 25.0, 170.0, 30.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(0.0, 0.0, 50.0, 55.0));
        assert_eq!(bounds(&doc, &result, "#c"), rect(60.0, 0.0, 70.0, 20.0));
    }

    #[test]
    fn grid_implicit_tracks_repeat_on_both_sides_of_explicit_grid() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:40px;grid-auto-columns:10px 20px;grid-auto-rows:15px 25px;justify-content:start;gap:3px 2px}i{grid-row:1}#a{grid-column:-4}#b{grid-column:-3}#c{grid-column:1}#d{grid-column:2}#e{grid-column:3;grid-row:3}</style><main><i id=a></i><i id=b></i><i id=c></i><i id=d></i><i id=e></i></main>",
            300.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(0.0, 0.0, 10.0, 15.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(12.0, 0.0, 20.0, 15.0));
        assert_eq!(bounds(&doc, &result, "#c"), rect(34.0, 0.0, 40.0, 15.0));
        assert_eq!(bounds(&doc, &result, "#d"), rect(76.0, 0.0, 10.0, 15.0));
        assert_eq!(bounds(&doc, &result, "#e"), rect(88.0, 46.0, 20.0, 15.0));
    }

    #[test]
    fn grid_dense_fills_holes_and_sparse_cursor_keeps_order_in_each_axis() {
        for (flow, expected) in [
            ("row", (80.0, 20.0)),
            ("row dense", (80.0, 0.0)),
            ("column", (40.0, 40.0)),
            ("column dense", (0.0, 40.0)),
        ] {
            let span = if flow.starts_with("column") {
                "grid-row:span 2"
            } else {
                "grid-column:span 2"
            };
            let source = format!(
                "<style>body{{margin:0}}main{{display:grid;grid-template-columns:repeat(3,40px);grid-template-rows:repeat(3,20px);grid-auto-columns:40px;grid-auto-rows:20px;grid-auto-flow:{flow}}}#a,#b{{{span}}}</style><main><i id=a></i><i id=b></i><i id=c></i></main>"
            );
            let (doc, result) = render(&source, 300.0);
            let c = bounds(&doc, &result, "#c");
            assert_eq!((c.x, c.y), expected, "{flow}");
        }
    }

    #[test]
    fn grid_auto_items_respect_definite_reservations_row_locks_and_order() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:repeat(3,40px);grid-auto-rows:20px}#locked{grid-row:1;grid-column:2}#row{grid-row:1}#first{order:-1}#col{grid-column:2}</style><main><i id=auto></i><i id=locked></i><i id=row></i><i id=first></i><i id=col></i></main>",
            300.0,
        );
        assert_eq!(bounds(&doc, &result, "#row"), rect(0.0, 0.0, 40.0, 20.0));
        assert_eq!(bounds(&doc, &result, "#first"), rect(80.0, 0.0, 40.0, 20.0));
        assert_eq!(bounds(&doc, &result, "#auto"), rect(0.0, 20.0, 40.0, 20.0));
        assert_eq!(bounds(&doc, &result, "#col"), rect(40.0, 20.0, 40.0, 20.0));
        let parent = doc.query_selector("main").unwrap();
        assert_eq!(doc.attr(doc.nodes[parent].children[0], "id"), Some("auto"));
    }

    #[test]
    fn grid_explicit_overlaps_paint_in_order_modified_document_order() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:50px;grid-template-rows:30px}i{grid-area:1 / 1 / 2 / 2}#a{order:2}</style><main><i id=a></i><i id=b></i></main>",
            200.0,
        );
        assert_eq!(result.hit_test(20.0, 10.0), doc.query_selector("#a"));
        assert_eq!(bounds(&doc, &result, "#a"), bounds(&doc, &result, "#b"));
    }

    #[test]
    fn grid_minmax_fraction_tracks_freeze_at_intrinsic_and_explicit_minima() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;width:300px;grid-template-columns:minmax(100px,1fr) minmax(0,2fr);column-gap:20px}i{height:20px}#a{min-width:140px}</style><main><i id=a></i><i id=b></i></main>",
            350.0,
        );
        // A fixed 100px minimum does not enlarge from a larger item minimum:
        // item a overflows its track. Its neighbor starts after that track+gap.
        assert_eq!(bounds(&doc, &result, "#a").width, 140.0);
        assert_eq!(bounds(&doc, &result, "#b"), rect(120.0, 0.0, 180.0, 20.0));
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;width:300px;grid-template-columns:1fr 2fr;column-gap:20px}i{height:20px}#a{min-width:140px}</style><main><i id=a></i><i id=b></i></main>",
            350.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 140.0);
        assert_eq!(bounds(&doc, &result, "#b"), rect(160.0, 0.0, 140.0, 20.0));
    }

    #[test]
    fn grid_spanning_contributions_size_intrinsic_tracks_and_do_not_depend_on_order() {
        for reverse in [false, true] {
            let items = if reverse {
                "<i id=b></i><i id=a></i>"
            } else {
                "<i id=a></i><i id=b></i>"
            };
            let source = format!(
                "<style>body{{margin:0}}main{{display:grid;grid-template-columns:auto auto auto;justify-content:start;column-gap:10px}}i{{height:20px;grid-row:1}}#a{{grid-column:1 / 3;width:110px}}#b{{grid-column:2 / 4;width:150px}}</style><main>{items}<b style='grid-column:1;grid-row:2;height:10px'></b><b style='grid-column:2;grid-row:2;height:10px'></b><b id=probe style='grid-column:3;grid-row:2;height:10px'></b></main>"
            );
            let (doc, result) = render(&source, 400.0);
            assert_eq!(
                bounds(&doc, &result, "#probe"),
                rect(140.0, 20.0, 70.0, 10.0)
            );
        }
    }

    #[test]
    fn grid_row_spans_size_auto_rows_and_explicit_rows_keep_their_size() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:40px 40px;grid-template-rows:20px auto;row-gap:10px}#a{grid-row:1 / 3;height:80px}#b{grid-column:2;grid-row:2}</style><main><i id=a></i><i id=b></i></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#b"), rect(40.0, 30.0, 40.0, 50.0));
        assert_eq!(bounds(&doc, &result, "main").height, 80.0);
    }

    #[test]
    fn grid_indefinite_fractional_rows_use_content_not_viewport_or_fixed_tracks() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:40px;grid-template-rows:100px 1fr 2fr;row-gap:5px}#b{height:20px}#c{height:30px}</style><main><i id=a></i><i id=b></i><i id=c></i></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").height, 100.0);
        assert_eq!(bounds(&doc, &result, "#b").y, 105.0);
        assert_eq!(bounds(&doc, &result, "#c").y, 130.0);
        assert_eq!(bounds(&doc, &result, "main").height, 170.0);
    }

    #[test]
    fn grid_definite_rows_percentages_and_item_percent_height_use_grid_area() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;width:200px;height:200px;grid-template-columns:1fr;grid-template-rows:25% minmax(20px,1fr);row-gap:10px}#a{height:50%;align-self:end}#b{padding:10px;box-sizing:border-box;min-height:80px}</style><main><i id=a></i><i id=b></i></main>",
            250.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(0.0, 25.0, 200.0, 25.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(0.0, 60.0, 200.0, 140.0));
    }

    #[test]
    fn grid_alignment_and_auto_margins_use_remaining_area_after_box_sizing() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;width:300px;height:200px;grid-template-columns:80px 80px;grid-template-rows:60px;gap:10px;justify-content:space-between;align-content:center;justify-items:end;align-items:center}i{width:30px;height:20px;padding:5px;border:2px solid black;box-sizing:border-box}#b{margin-left:auto;margin-right:auto;align-self:end}</style><main><i id=a></i><i id=b></i></main>",
            350.0,
        );
        assert_eq!(bounds(&doc, &result, "#a"), rect(50.0, 90.0, 30.0, 20.0));
        assert_eq!(bounds(&doc, &result, "#b"), rect(245.0, 110.0, 30.0, 20.0));
    }

    #[test]
    fn grid_out_of_flow_and_non_rendering_nodes_do_not_reserve_cells() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:40px 40px;grid-auto-rows:20px}#overlay{position:absolute;width:5px;height:5px;left:150px}#hidden{display:none}</style><main><!--comment--><i id=a></i><i id=overlay></i><i id=hidden></i><template><i></i></template><i id=b></i></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#b"), rect(40.0, 0.0, 40.0, 20.0));
        assert_eq!(bounds(&doc, &result, "main").height, 20.0);
        assert_eq!(bounds(&doc, &result, "#overlay").x, 150.0);
    }

    #[test]
    fn grid_spanning_flexible_tracks_does_not_enlarge_adjacent_auto_tracks() {
        let definitions = [
            GridTrack::single(GridBreadth::Length(Length::Fr(1.0))),
            GridTrack::default(),
        ];
        let mut work = MAX_GRID_WORK;
        assert_eq!(
            size_grid_tracks(
                &definitions,
                Some(300.0),
                0.0,
                &[GridContribution {
                    start: 0,
                    span: 2,
                    min: 200.0,
                    max: 200.0
                }],
                "stretch",
                &mut work
            ),
            vec![300.0, 0.0]
        );
    }

    #[test]
    fn separate_row_and_column_gaps_preserve_flex_axis_semantics() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:flex;flex-wrap:wrap;width:100px;gap:7px 20px}i{width:40px;height:10px}</style><main><i id=a></i><i id=b></i><i id=c></i></main>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#b"), rect(60.0, 0.0, 40.0, 10.0));
        assert_eq!(bounds(&doc, &result, "#c"), rect(0.0, 17.0, 40.0, 10.0));
    }

    #[test]
    fn grid_short_items_limit_spanning_growth_without_equalizing_tracks() {
        let mut work = MAX_GRID_WORK;
        let definitions = [GridTrack::default(); 2];
        let contributions = [
            GridContribution {
                start: 0,
                span: 1,
                min: 10.0,
                max: 10.0,
            },
            GridContribution {
                start: 0,
                span: 2,
                min: 30.0,
                max: 100.0,
            },
        ];
        assert_eq!(
            size_grid_tracks(
                &definitions,
                Some(200.0),
                0.0,
                &contributions,
                "start",
                &mut work
            ),
            vec![10.0, 90.0]
        );
        let definitions = [
            GridTrack {
                min: GridBreadth::Auto,
                max: GridBreadth::Length(Length::Px(50.0)),
            },
            GridTrack::default(),
        ];
        assert_eq!(
            size_grid_tracks(
                &definitions,
                Some(200.0),
                0.0,
                &[GridContribution {
                    start: 0,
                    span: 2,
                    min: 200.0,
                    max: 200.0
                }],
                "start",
                &mut work
            ),
            vec![50.0, 150.0]
        );
    }

    #[test]
    fn anonymous_item_whitespace_scan_consumes_shared_nonrefundable_source_work() {
        let text = format!("{}x", " ".repeat(20_000));
        let mut source_work = 30_000;
        assert!(has_inline_content(&text, &mut source_work));
        assert_eq!(source_work, 9_999);
        assert!(!has_inline_content(&text, &mut source_work));
        assert_eq!(source_work, 0);
        assert!(!has_inline_content("x", &mut source_work));
    }

    #[test]
    fn nested_grid_reflow_cannot_replay_an_entire_whitespace_source_budget() {
        let source = format!(
            "<style>section{{display:grid;grid-template-rows:100px;padding:1px}}</style>{}{}x{}",
            "<section>".repeat(14),
            " ".repeat(500_000),
            "</section>".repeat(14)
        );
        let (_, result) = render(&source, 320.0);
        assert!(result.commands.len() < 1000);
        assert!(
            result
                .commands
                .iter()
                .all(|command| !matches!(command,DrawCommand::Text{text,..} if text.contains('x')))
        );
    }

    #[test]
    fn grid_nested_measurement_reuses_fragments_and_preserves_clip_balance() {
        let mut source = String::from(
            "<style>body{margin:0}section{display:grid;overflow:hidden;grid-template-columns:minmax(0,1fr)}i{height:20px}</style>",
        );
        source.push_str(&"<section>".repeat(32));
        source.push_str("<i id=leaf>Still rendered</i>");
        source.push_str(&"</section>".repeat(32));
        let (doc, result) = render(&source, 200.0);
        assert_eq!(bounds(&doc, &result, "#leaf").height, 20.0);
        assert!(
            result
                .commands
                .iter()
                .any(|command| matches!(command,DrawCommand::Text{text,..} if text=="Still"))
        );
        let mut balance = 0;
        for command in &result.commands {
            match command {
                DrawCommand::PushClip { .. } => balance += 1,
                DrawCommand::PopClip => {
                    assert!(balance > 0);
                    balance -= 1;
                }
                _ => {}
            }
        }
        assert_eq!(balance, 0);
        assert!(result.commands.len() < 500);
    }

    #[test]
    fn grid_far_implicit_tracks_keep_translated_geometry_inside_engine_limits() {
        let (doc, result) = render(
            "<style>body{margin:0}main{display:grid;grid-template-columns:repeat(64,1000000px);grid-auto-rows:1000000px}i{grid-column:64;grid-row:64}</style><main><i id=far>Far</i></main>",
            200.0,
        );
        let far = bounds(&doc, &result, "#far");
        assert!(far.x.abs() <= MAX_EXTENT && far.y.abs() <= MAX_EXTENT);
        assert!(result.commands.iter().all(|command| match command {
            DrawCommand::Text { x, y, .. } => x.abs() <= MAX_EXTENT && y.abs() <= MAX_EXTENT,
            _ => true,
        }));
    }

    #[test]
    fn grid_placement_extreme_indices_spans_and_scanning_have_shared_limits() {
        let mut source = String::from(
            "<style>body{margin:0}main{display:grid;grid-template-columns:repeat(64,1px);grid-auto-rows:1px;grid-auto-flow:dense}i{grid-column:span 999999}#negative{grid-area:-2147483648 / -2147483648 / 2147483647 / 2147483647}</style><main><i id=negative></i>",
        );
        source.push_str(&"<i></i>".repeat(5000));
        source.push_str("</main>");
        let doc = Document::parse(&source);
        let styles = crate::css::compute_styles(&doc, &doc.stylesheets(), 300.0, 400.0);
        let main = doc.query_selector("main").unwrap();
        let mut work = MAX_GRID_WORK;
        let plan = grid_plan(
            doc.nodes[main].children.clone(),
            &styles,
            &styles[main],
            &mut work,
        );
        assert!(plan.items.len() <= MAX_GRID_ITEMS);
        assert!(plan.columns <= MAX_GRID_TRACKS && plan.rows <= MAX_GRID_TRACKS);
        assert!(work < MAX_GRID_WORK);
        for area in plan.items.iter().filter_map(|item| item.area) {
            assert!(
                area.column + area.columns <= MAX_GRID_TRACKS
                    && area.row + area.rows <= MAX_GRID_TRACKS
            );
        }
        let mut exhausted = 0;
        let sizes = size_grid_tracks(
            &[GridTrack::default(); 64],
            Some(300.0),
            10.0,
            &[GridContribution {
                start: 0,
                span: 64,
                min: 100.0,
                max: 500.0,
            }],
            "stretch",
            &mut exhausted,
        );
        assert_eq!(exhausted, 0);
        assert!(sizes.iter().all(|v| v.is_finite()));
    }

    #[test]
    fn grid_places_rows_and_fixed_tracks() {
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;grid-template-columns:80px 120px;gap:10px}#grid div{height:25px}</style><div id=grid><div id=a></div><div id=b></div><div id=c></div></div>",
            300.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        let c = bounds(&doc, &result, "#c");
        assert_eq!(a.width, 80.0);
        assert_eq!(b.width, 120.0);
        assert_eq!(b.x - a.x, 90.0);
        assert_eq!(c.y - a.y, 35.0);
    }

    #[test]
    fn padded_auto_and_fractional_grids_keep_tiles_inside_content_box() {
        for tracks in ["auto auto auto", "1fr 1fr 1fr"] {
            let source = format!(
                "<style>*{{box-sizing:border-box}}body{{margin:0}}section{{width:360px;padding:20px;border:2px solid black}}#grid{{display:grid;padding:10px;border:1px solid black;gap:12px;grid-template-columns:{tracks}}}#grid div{{padding:20px;border:1px solid blue}}</style><section><div id=grid><div id=a>A</div><div id=b>B</div><div id=c>C</div></div></section>"
            );
            let (doc, result) = render(&source, 400.0);
            let grid = bounds(&doc, &result, "#grid");
            let a = bounds(&doc, &result, "#a");
            let b = bounds(&doc, &result, "#b");
            let c = bounds(&doc, &result, "#c");
            assert_eq!(grid.width, 316.0, "{tracks}");
            if tracks.starts_with("1fr") {
                assert_eq!(a.width, 90.0, "{tracks}");
            } else {
                assert!((a.width + b.width + c.width - 270.0).abs() < 0.001);
            }
            assert_eq!(b.x - (a.x + a.width), 12.0, "{tracks}");
            assert_eq!(a.x, grid.x + 11.0, "{tracks}");
            assert_eq!(c.x + c.width, grid.x + grid.width - 11.0, "{tracks}");
        }
    }

    #[test]
    fn fractional_tracks_share_space_after_fixed_tracks_and_gaps() {
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;width:400px;gap:20px;grid-template-columns:100px 1fr 2fr}#grid div{height:10px}</style><div id=grid><div id=a></div><div id=b></div><div id=c></div></div>",
            500.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        let c = bounds(&doc, &result, "#c");
        assert_eq!(a.width, 100.0);
        assert!((b.width - 260.0 / 3.0).abs() < 0.001);
        assert!((c.width - 520.0 / 3.0).abs() < 0.001);
        assert!((c.x + c.width - 400.0).abs() < 0.001);
    }

    #[test]
    fn percentage_tracks_keep_container_percentage_and_fr_uses_remainder() {
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;width:400px;gap:20px;grid-template-columns:100px 25% 1fr}#grid div{height:10px}</style><div id=grid><div id=a></div><div id=b></div><div id=c></div></div>",
            500.0,
        );
        assert_eq!(bounds(&doc, &result, "#a").width, 100.0);
        assert_eq!(bounds(&doc, &result, "#b").width, 100.0);
        assert_eq!(bounds(&doc, &result, "#c").width, 160.0);
        let (_, result) = render(
            "<style>body{margin:0}#grid{display:grid;width:400px;gap:20px;grid-template-columns:50% 50%}#grid div{height:10px}</style><div id=grid><div id=a></div><div id=b></div></div>",
            500.0,
        );
        let widths: Vec<f32> = result
            .hit_regions
            .iter()
            .filter(|hit| hit.rect.height == 10.0 && hit.rect.width == 200.0)
            .map(|hit| hit.rect.width)
            .collect();
        assert_eq!(widths, [200.0, 200.0]);
    }

    #[test]
    fn table_cells_follow_columns_and_spanning_rows() {
        let (doc, result) = render(
            "<style>body{margin:0}table{width:300px}td{padding:0;height:20px}</style><table><tr><td id=a rowspan=2>A</td><td id=b>B</td></tr><tr><td id=c>C</td></tr><tr><td id=d colspan=2>D</td></tr></table>",
            400.0,
        );
        let a = bounds(&doc, &result, "#a");
        let b = bounds(&doc, &result, "#b");
        let c = bounds(&doc, &result, "#c");
        let d = bounds(&doc, &result, "#d");
        assert_eq!(a.y, b.y);
        assert_eq!(b.x, c.x);
        assert!(c.y > b.y);
        assert!(d.y >= a.y + a.height);
        assert_eq!(a.height, b.height + c.height + 2.0);
        assert_eq!(d.width, a.width + b.width + 2.0);
    }

    #[test]
    fn rounded_opaque_borders_leave_corner_pixels_clear() {
        use crate::graphics::{Canvas, ImageStore};
        let (_, result) = render(
            "<style>body{margin:0}div{width:50px;height:50px;border:3px solid red;border-radius:12px;background:blue}</style><div></div>",
            100.0,
        );
        let mut canvas = Canvas::new(100, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[0], 0xffffff);
        assert_eq!(canvas.pixels[28], 0xff0000);
        assert_eq!(canvas.pixels[28 * 100 + 28], 0x0000ff);
    }

    #[test]
    fn body_background_propagates_across_canvas_without_double_blending() {
        use crate::graphics::{Canvas, ImageStore};
        let (_, result) = render(
            "<style>body{width:60px;margin:0 auto;background:rgba(255,0,0,0.5)}</style><p>page</p>",
            100.0,
        );
        let mut canvas = Canvas::new(100, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[99 * 100], canvas.pixels[99 * 100 + 50]);
        assert_eq!(canvas.pixels[99 * 100], 0xff7f7f);
    }

    #[test]
    fn html_background_takes_precedence_over_body_propagation() {
        use crate::graphics::{Canvas, ImageStore};
        let (_, result) = render(
            "<style>html{background:blue}body{width:50px;height:20px;margin:0;background:red}</style>",
            100.0,
        );
        let mut canvas = Canvas::new(100, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[0], 0xff0000);
        assert_eq!(canvas.pixels[99], 0x0000ff);
        assert_eq!(canvas.pixels[99 * 100], 0x0000ff);
    }

    #[test]
    fn image_css_width_preserves_natural_aspect_ratio() {
        let (doc, result) = render(
            "<style>body{margin:0}img{width:120px}</style><img id=photo src=photo.png data-eris-natural-width=400 data-eris-natural-height=200>",
            300.0,
        );
        let photo = bounds(&doc, &result, "#photo");
        assert_eq!(photo.width, 120.0);
        assert_eq!(photo.height, 60.0);
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Image { rect, key } if key == "photo.png" && rect.width == 120.0 && rect.height == 60.0)));
    }

    #[test]
    fn inline_svg_is_one_replaced_image() {
        let (doc, result) = render(
            "<style>body{margin:0}svg{width:200px}</style><svg id=vector data-eris-natural-width=40 data-eris-natural-height=20><text>vector label</text></svg>",
            300.0,
        );
        let node = doc.query_selector("#vector").unwrap();
        let geometry = bounds(&doc, &result, "#vector");
        assert_eq!(geometry.width, 200.0);
        assert_eq!(geometry.height, 100.0);
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Image { key, .. } if key == &format!("eris-inline-svg:{node}"))));
        assert!(
            !result
                .commands
                .iter()
                .any(|command| matches!(command, DrawCommand::Text { .. }))
        );
    }

    #[test]
    fn extreme_css_geometry_never_emits_nonfinite_coordinates() {
        let (_, result) = render(
            "<style>body{margin:0}div{display:flex;width:1e38px;padding:1e38%;gap:1e38px}span{width:1e38%;height:1e38px;position:relative;left:1e38%}</style><div><span>huge</span><span>page</span></div>",
            800.0,
        );
        assert!(result.content_height.is_finite());
        for command in &result.commands {
            let coordinates = match command {
                DrawCommand::Rect { rect, .. }
                | DrawCommand::Image { rect, .. }
                | DrawCommand::PushClip { rect } => {
                    vec![rect.x, rect.y, rect.width, rect.height]
                }
                DrawCommand::Text { x, y, size, .. } => vec![*x, *y, *size],
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    width,
                    ..
                } => vec![*x1, *y1, *x2, *y2, *width],
                DrawCommand::PopClip
                | DrawCommand::PushFixed
                | DrawCommand::PopFixed
                | DrawCommand::PushOpacity { .. }
                | DrawCommand::PopOpacity => vec![],
            };
            assert!(coordinates.iter().all(|value| value.is_finite()));
        }
    }

    #[test]
    fn hidden_subtrees_never_paint_or_hit() {
        let (doc, result) = render(
            "<style>#hidden{display:none}</style><div id=hidden><a href='/secret'>secret</a></div><p>visible</p>",
            300.0,
        );
        let hidden = doc.query_selector("#hidden").unwrap();
        assert!(result.hit_regions.iter().all(|hit| hit.node != hidden));
        assert!(!result.commands.iter().any(
            |command| matches!(command, DrawCommand::Text { text, .. } if text.contains("secret"))
        ));
    }

    #[test]
    fn mixed_case_password_type_never_emits_plaintext() {
        let (_, result) = render("<input type=PaSsWoRd value='secret123'>", 300.0);
        let text: Vec<&str> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(text, ["•••••••••"]);
        assert!(!result.commands.iter().any(
            |command| matches!(command, DrawCommand::Text { text, .. } if text.contains("secret"))
        ));
    }

    #[test]
    fn mixed_case_checkbox_and_radio_use_intrinsic_control_geometry() {
        use crate::graphics::{Canvas, ImageStore};
        for kind in ["CHECKBOX", "ChEcKbOx", "RADIO", "RaDiO"] {
            let source = format!(
                "<style>body{{margin:0}}input{{width:auto;padding:0;border:0}}</style><input id=control type={kind} checked value='must not paint'>"
            );
            let (doc, result) = render(&source, 100.0);
            assert_eq!(
                bounds(&doc, &result, "#control"),
                rect(0.0, 0.0, 16.0, 16.0),
                "{kind}"
            );
            assert!(
                !result
                    .commands
                    .iter()
                    .any(|command| matches!(command, DrawCommand::Text { .. })),
                "{kind}"
            );
            let mut canvas = Canvas::new(100, 50).unwrap();
            canvas.paint(
                &result.commands,
                &Fonts::new(),
                &ImageStore::new(),
                0.0,
                0.0,
            );
            assert_eq!(canvas.pixels[8 * 100 + 8], 0x3269e1, "{kind}");
        }
    }

    #[test]
    fn checkbox_and_radio_defaults_do_not_inherit_text_input_width() {
        for kind in ["checkbox", "CHECKBOX", "radio", "RADIO"] {
            let source = format!("<style>body{{margin:0}}</style><input id=control type={kind}>");
            let (doc, result) = render(&source, 300.0);
            assert_eq!(
                bounds(&doc, &result, "#control"),
                rect(0.0, 0.0, 16.0, 16.0),
                "{kind}"
            );
        }
    }

    #[test]
    fn authored_checkbox_sizing_and_box_model_override_native_defaults() {
        let source = "<style>body{margin:0}input{width:40px;height:30px;padding:3px;border:2px solid red}</style><input id=control type=checkbox>";
        let (doc, result) = render(source, 300.0);
        assert_eq!(
            bounds(&doc, &result, "#control"),
            rect(0.0, 0.0, 50.0, 40.0)
        );
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Rect { rect: bounds, color, .. } if *bounds == rect(5.0, 5.0, 40.0, 30.0) && *color == rgba(160, 167, 180, 255))));
        let (doc, result) = render(
            &source.replace("width:40px", "box-sizing:border-box;width:40px"),
            300.0,
        );
        assert_eq!(
            bounds(&doc, &result, "#control"),
            rect(0.0, 0.0, 40.0, 30.0)
        );
        assert!(result.commands.iter().any(|command| matches!(command, DrawCommand::Rect { rect: bounds, color, .. } if *bounds == rect(5.0, 5.0, 30.0, 20.0) && *color == rgba(160, 167, 180, 255))));
    }

    #[test]
    fn mixed_case_hidden_input_neither_paints_nor_reserves_layout_space() {
        let (doc, result) = render(
            "<style>body{margin:0}input{display:block;width:200px;margin:30px}</style><input id=hidden type=HiDdEn value='secret'><span>visible</span>",
            300.0,
        );
        let hidden = doc.query_selector("#hidden").unwrap();
        assert!(result.hit_regions.iter().all(|hit| hit.node != hidden));
        let painted: Vec<_> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { x, y, text, .. } => Some((*x, *y, text.as_str())),
                _ => None,
            })
            .collect();
        assert_eq!(painted, [(0.0, 0.0, "visible")]);
    }

    #[test]
    fn nested_overflow_clips_pixels_and_link_hit_regions_but_keeps_borders() {
        use crate::graphics::{Canvas, ImageStore};
        for overflow in ["hidden", "clip"] {
            let source = format!(
                "<style>body{{margin:0}}#outer{{width:60px;height:60px;border:2px solid black;background:blue;overflow:{overflow}}}#inner{{width:50px;height:50px;margin:30px 0 0 30px;overflow:{overflow};background:red}}#link{{display:block;width:100px;height:100px;background:lime}}</style><div id=outer><div id=inner><a id=link href='/hidden'>Link</a></div></div>"
            );
            let (doc, result) = render(&source, 100.0);
            let link = bounds(&doc, &result, "#link");
            assert_eq!(link, rect(32.0, 32.0, 30.0, 30.0));
            assert_eq!(result.hit_test(70.0, 50.0), doc.query_selector("body"));
            let mut canvas = Canvas::new(100, 100).unwrap();
            canvas.paint(
                &result.commands,
                &Fonts::new(),
                &ImageStore::new(),
                0.0,
                0.0,
            );
            assert_eq!(canvas.pixels[55 * 100 + 55], 0x00ff00, "{overflow}");
            assert_eq!(canvas.pixels[55 * 100 + 25], 0x0000ff, "{overflow}");
            assert_eq!(canvas.pixels[55 * 100 + 70], 0xffffff, "{overflow}");
            assert_eq!(canvas.pixels[70 * 100 + 55], 0xffffff, "{overflow}");
            assert_eq!(canvas.pixels[63 * 100 + 40], 0x000000, "{overflow}");
        }
    }

    #[test]
    fn clipped_absolute_content_does_not_extend_document_scroll_height() {
        let (_, result) = render(
            "<style>body{margin:0}div{position:relative;width:100px;height:50px;overflow:hidden}a{position:absolute;top:1000px;display:block;width:50px;height:50px}</style><div><a href='/hidden'>Hidden</a></div>",
            200.0,
        );
        assert_eq!(result.content_height, 400.0);
        assert_eq!(result.hit_test(10.0, 1010.0), None);
    }

    #[test]
    fn clipping_in_grid_fragments_uses_positioned_coordinates() {
        use crate::graphics::{Canvas, ImageStore};
        let (doc, result) = render(
            "<style>body{margin:0}#grid{display:grid;grid-template-columns:50px 50px;gap:20px}#grid div{height:30px;overflow:hidden}a{display:block;width:100px;height:100px;background:red}</style><div id=grid><div></div><div><a id=link href='/x'></a></div></div>",
            200.0,
        );
        assert_eq!(bounds(&doc, &result, "#link"), rect(70.0, 0.0, 50.0, 30.0));
        let mut canvas = Canvas::new(200, 100).unwrap();
        canvas.paint(
            &result.commands,
            &Fonts::new(),
            &ImageStore::new(),
            0.0,
            0.0,
        );
        assert_eq!(canvas.pixels[10 * 200 + 80], 0xff0000);
        assert_eq!(canvas.pixels[10 * 200 + 130], 0xffffff);
        assert_eq!(canvas.pixels[40 * 200 + 80], 0xffffff);
    }

    #[test]
    fn preformatted_newlines_and_spaces_survive() {
        let (_, result) = render(
            "<style>body{margin:0}pre{margin:0}</style><pre>a  b\nc</pre>",
            300.0,
        );
        let text: Vec<(&str, f32)> = result
            .commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, y, .. } => Some((text.as_str(), *y)),
                _ => None,
            })
            .collect();
        assert!(text.iter().any(|(text, _)| *text == "  "));
        let a_y = text.iter().find(|(text, _)| *text == "a").unwrap().1;
        let c_y = text.iter().find(|(text, _)| *text == "c").unwrap().1;
        assert!(c_y > a_y);
    }

    #[test]
    fn deep_input_and_nonfinite_viewport_stay_bounded() {
        let source = format!("{}deep{}", "<div>".repeat(300), "</div>".repeat(300));
        let document = Document::parse(&source);
        let styles = crate::css::compute_styles(&document, &[], 800.0, 600.0);
        let result = layout(&document, &styles, f32::NAN, f32::INFINITY, &Fonts::new());
        assert!(result.content_height.is_finite());
        assert!(result.commands.len() <= MAX_COMMANDS);
        assert!(result.hit_regions.iter().all(|hit| {
            [hit.rect.x, hit.rect.y, hit.rect.width, hit.rect.height]
                .iter()
                .all(|v| v.is_finite())
        }));
    }

    #[test]
    fn margins_collapse_positive_and_negative_values() {
        assert_eq!(collapsed_margin(12.0, 20.0), 20.0);
        assert_eq!(collapsed_margin(-12.0, -20.0), -20.0);
        assert_eq!(collapsed_margin(-12.0, 20.0), 8.0);
    }

    #[test]
    fn untrusted_dimensions_are_finite_and_bounded() {
        assert_eq!(extent(f32::NAN), 0.0);
        assert_eq!(extent(f32::INFINITY), 0.0);
        assert_eq!(resolve(Length::Percent(50.0), 200.0), Some(100.0));
        let bounds = rect(f32::NAN, f32::INFINITY, f32::MAX, -1.0);
        assert!(bounds.x.is_finite() && bounds.y.is_finite());
        assert_eq!(bounds.width, MAX_EXTENT);
        assert_eq!(bounds.height, 0.0);
    }

    #[test]
    fn text_clipping_respects_unicode_boundaries() {
        let text = "a日本語🙂";
        assert_eq!(
            fit_text(text, 3.0, |value| value.chars().count() as f32),
            "a日本"
        );
        assert_eq!(
            fit_text(text, 0.0, |value| value.chars().count() as f32),
            ""
        );
    }

    #[test]
    fn hit_testing_prefers_later_painted_descendants() {
        let result = LayoutResult {
            commands: Vec::new(),
            hit_regions: vec![
                HitRegion {
                    fixed: false,
                    node: 1,
                    rect: rect(0.0, 0.0, 100.0, 100.0),
                },
                HitRegion {
                    fixed: false,
                    node: 2,
                    rect: rect(20.0, 20.0, 20.0, 20.0),
                },
            ],
            content_height: 100.0,
        };
        assert_eq!(result.hit_test(21.0, 21.0), Some(2));
        assert_eq!(result.hit_test(0.0, 0.0), Some(1));
        assert_eq!(result.hit_test(100.0, 100.0), None);
    }
}
