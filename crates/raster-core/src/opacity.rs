//! Native-only cropped intermediates. Admission proves an opaque first backing
//! rectangle for every nonempty partial-opacity group. This permits exact
//! integer k/256 compositing without changing Canvas's rounding semantics.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Region {
    pub word_base: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Operation {
    Paint,
    Clear,
    Composite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Parameters {
    pub operation: Operation,
    pub destination: Option<Region>,
    pub source: Option<Region>,
    pub opacity: u32,
}
impl Parameters {
    pub fn write(self, frame: Frame, record: &mut [u8]) {
        let destination = self.destination.unwrap_or(Region {
            word_base: 0,
            x: 0,
            y: 0,
            width: frame.width,
            height: frame.height,
        });
        let mut fields = [0u32; 16];
        fields[..6].copy_from_slice(&[
            destination.word_base,
            destination.x,
            destination.y,
            destination.width,
            destination.height,
            u32::from(self.destination.is_some()),
        ]);
        if let Some(source) = self.source {
            fields[8..14].copy_from_slice(&[
                source.word_base,
                source.x,
                source.y,
                source.width,
                source.height,
                self.opacity,
            ]);
        }
        for (field, slot) in fields
            .into_iter()
            .zip(record[64..128].as_chunks_mut::<4>().0)
        {
            slot.copy_from_slice(&field.to_le_bytes());
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(super) enum Pending {
    #[default]
    Root,
    Paint(usize),
    Clear(usize),
    Composite(usize),
}

#[derive(Clone, Copy)]
struct Bounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
impl Bounds {
    fn of(draw: Draw) -> Self {
        Self {
            x: draw.x,
            y: draw.y,
            width: draw.width,
            height: draw.height,
        }
    }
    fn union(self, other: Self) -> Self {
        // Every input is an admitted framebuffer rectangle, bounded to 1280x1024.
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            width: (self.x + self.width).max(other.x + other.width) - x,
            height: (self.y + self.height).max(other.y + other.height) - y,
        }
    }
    fn contains(self, other: Self) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.width >= other.x + other.width
            && self.y + self.height >= other.y + other.height
    }
}

struct Group {
    parent: Option<usize>,
    opacity: u32,
    bounds: Option<Bounds>,
    // Some(None) means the first effective paint was not an opaque rectangle.
    first: Option<Option<Bounds>>,
    region: Option<Region>,
}
impl Group {
    fn include(&mut self, bounds: Bounds, backing: bool) {
        if self.first.is_none() {
            self.first = Some(backing.then_some(bounds));
        }
        self.bounds = Some(self.bounds.map_or(bounds, |old| old.union(bounds)));
    }
}

#[derive(Clone, Copy)]
struct Saved {
    destination: Option<usize>,
    suppressed: bool,
    created: Option<usize>,
}

pub(super) struct State {
    groups: Vec<Group>,
    stack: Vec<Saved>,
    destination: Option<usize>,
    suppressed: bool,
    group_limit: usize,
}
impl State {
    pub fn new(command_count: usize) -> Result<Self> {
        Ok(Self {
            groups: Vec::new(),
            stack: Vec::new(),
            destination: None,
            suppressed: false,
            group_limit: command_count / 2,
        })
    }
    pub fn suppressed(&self) -> bool {
        self.suppressed
    }
    pub fn in_group(&self) -> bool {
        self.destination.is_some()
    }

    pub fn push(&mut self, opacity: f32) -> Result<Option<PendingDraw>> {
        // CoordinateState separately checks profile, range and the mixed stack.
        let scaled = opacity * 256.0;
        if scaled != scaled.trunc() {
            return Err("unsupported opacity value".into());
        }
        if self.stack.len() == MAX_SCOPES {
            return Err("scope budget".into());
        }
        if self.stack.capacity() == 0 {
            self.stack = reserved(MAX_SCOPES)?;
        }
        let created = if !self.suppressed && opacity > 0.0 && opacity < 1.0 {
            if self.groups.len() == self.group_limit {
                // Two balanced original commands are required per materialized
                // group; never let malformed unclosed input grow this vector.
                return Err("opacity group budget".into());
            }
            if self.groups.capacity() == 0 {
                self.groups = reserved(self.group_limit)?;
            }
            let index = self.groups.len();
            self.groups.push(Group {
                parent: self.destination,
                opacity: scaled as u32,
                bounds: None,
                first: None,
                region: None,
            });
            Some(index)
        } else {
            None
        };
        self.stack.push(Saved {
            destination: self.destination,
            suppressed: self.suppressed,
            created,
        });
        if opacity == 0.0 {
            self.suppressed = true;
        }
        if let Some(index) = created {
            self.destination = Some(index);
        }
        Ok(created.map(|index| placeholder(Pending::Clear(index))))
    }

    pub fn pop(&mut self) -> Result<Option<PendingDraw>> {
        let saved = self.stack.pop().ok_or("mismatched opacity scope")?;
        self.destination = saved.destination;
        self.suppressed = saved.suppressed;
        if let Some(index) = saved.created {
            if let (Some(parent), Some(bounds)) = (saved.destination, self.groups[index].bounds) {
                self.groups[parent].include(bounds, false);
            }
            Ok(Some(placeholder(Pending::Composite(index))))
        } else {
            Ok(None)
        }
    }

    pub fn paint(&mut self, draw: Draw, opaque_rectangle: bool) -> Pending {
        match self.destination {
            Some(index) => {
                self.groups[index].include(Bounds::of(draw), opaque_rectangle);
                Pending::Paint(index)
            }
            None => Pending::Root,
        }
    }

    pub fn finish_phase(&self) -> Result<()> {
        if self.stack.is_empty() && self.destination.is_none() && !self.suppressed {
            Ok(())
        } else {
            Err("unclosed opacity scope".into())
        }
    }

    pub fn materialize(
        mut self,
        pending: &mut Vec<PendingDraw>,
        invocations: &mut u64,
    ) -> Result<u64> {
        self.finish_phase()?;
        let mut bytes = 0u64;
        for group in &mut self.groups {
            let Some(bounds) = group.bounds else { continue };
            if !group
                .first
                .flatten()
                .is_some_and(|first| first.contains(bounds))
            {
                return Err("opacity group requires opaque rectangular backing".into());
            }
            let area_bytes = u64::from(bounds.width)
                .checked_mul(u64::from(bounds.height))
                .and_then(|area| area.checked_mul(8))
                .ok_or("opacity scratch overflow")?;
            let base = u32::try_from(bytes / 4).map_err(|_| "opacity scratch overflow")?;
            bytes = bytes
                .checked_add(area_bytes)
                .ok_or("opacity scratch overflow")?;
            Profile::Native.validate_buffer_bytes(bytes)?;
            group.region = Some(Region {
                word_base: base,
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
            });
            let work = profile::padded_invocations(bounds.width, bounds.height)?;
            *invocations = profile::add_work(profile::add_work(*invocations, work)?, work)?;
        }
        pending.retain_mut(|item| {
            let (index, operation) = match item.group {
                Pending::Root => return true,
                Pending::Paint(index) => (index, Operation::Paint),
                Pending::Clear(index) => (index, Operation::Clear),
                Pending::Composite(index) => (index, Operation::Composite),
            };
            let group = &self.groups[index];
            let Some(region) = group.region else {
                return false;
            };
            let destination = if operation == Operation::Composite {
                group.parent.map(|parent| {
                    self.groups[parent]
                        .region
                        .expect("nonempty child bound propagates")
                })
            } else {
                Some(region)
            };
            if operation != Operation::Paint {
                item.draw.x = region.x;
                item.draw.y = region.y;
                item.draw.width = region.width;
                item.draw.height = region.height;
            }
            item.draw.group = Some(Parameters {
                operation,
                destination,
                source: (operation == Operation::Composite).then_some(region),
                opacity: group.opacity,
            });
            true
        });
        Ok(bytes)
    }
}

fn placeholder(group: Pending) -> PendingDraw {
    PendingDraw {
        draw: Draw {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            color: 0,
            image: None,
            group: None,
        },
        image: None,
        glyph: None,
        group,
    }
}

pub(super) fn metadata_peak_bytes() -> Result<usize> {
    (MAX_COMMANDS / 2)
        .checked_mul(std::mem::size_of::<Group>())
        .and_then(|bytes| bytes.checked_add(MAX_SCOPES * std::mem::size_of::<Saved>()))
        .ok_or_else(|| "opacity metadata overflow".into())
}
