//! Immutable independent background lists under the shared image admission.
use super::{BackgroundBox, BackgroundPosition, BackgroundSize, MAX_BACKGROUND_LAYERS};
use crate::css::{BackgroundImage, Length, Rule, WinningSource, gradient, image, url};
use crate::graphics::gradient::LinearGradient;
use crate::selectors::{self, Error};
use std::cmp::Ordering;
use std::mem::{align_of, size_of};
use std::sync::{Arc, LazyLock, atomic::AtomicUsize};

const MAX_BYTES: usize = gradient::MAX_RETAINED_GRADIENT_BYTES;
const ITEM_WORDS: usize = 10;
const RECORD_WORDS: usize = 72;

#[derive(Debug, Clone)]
enum BackgroundList<T> {
    One(T),
    Many(Arc<[T]>),
}
impl<T> BackgroundList<T> {
    fn values(&self) -> &[T] {
        match self {
            Self::One(value) => std::slice::from_ref(value),
            Self::Many(values) => values,
        }
    }
    fn used(&self, index: usize) -> &T {
        let values = self.values();
        &values[index % values.len()]
    }
}

/// The six computed lists retain independent cardinalities. Color is separate.
#[derive(Debug, Clone)]
pub struct BackgroundData {
    images: BackgroundList<Option<BackgroundImage>>,
    sizes: BackgroundList<BackgroundSize>,
    positions: BackgroundList<BackgroundPosition>,
    repeats: BackgroundList<[bool; 2]>,
    origins: BackgroundList<BackgroundBox>,
    clips: BackgroundList<BackgroundBox>,
}

#[derive(Clone, Copy, Debug)]
pub struct UsedBackgroundLayer<'a> {
    pub index: usize,
    pub image: Option<&'a BackgroundImage>,
    pub size: BackgroundSize,
    pub position: BackgroundPosition,
    pub repeat: [bool; 2],
    pub origin: BackgroundBox,
    pub clip: BackgroundBox,
}

static INITIAL: LazyLock<Arc<BackgroundData>> =
    LazyLock::new(|| Arc::new(BackgroundData::default()));

impl Default for BackgroundData {
    fn default() -> Self {
        Self::single(
            None,
            BackgroundSize::default(),
            BackgroundPosition::default(),
            [true, true],
            BackgroundBox::Padding,
            BackgroundBox::Border,
        )
    }
}

impl BackgroundData {
    /// Pure singleton constructor for trusted programmatic styles. Author data
    /// uses BackgroundStorage; constructing this value grants no resource I/O.
    pub fn single(
        image: Option<BackgroundImage>,
        size: BackgroundSize,
        position: BackgroundPosition,
        repeat: [bool; 2],
        origin: BackgroundBox,
        clip: BackgroundBox,
    ) -> Self {
        Self {
            images: BackgroundList::One(image),
            sizes: BackgroundList::One(size),
            positions: BackgroundList::One(position),
            repeats: BackgroundList::One(repeat),
            origins: BackgroundList::One(origin),
            clips: BackgroundList::One(clip),
        }
    }
    pub fn initial() -> Arc<Self> {
        Arc::clone(&INITIAL)
    }
    pub fn image_slots(&self) -> &[Option<BackgroundImage>] {
        self.images.values()
    }
    pub fn sizes(&self) -> &[BackgroundSize] {
        self.sizes.values()
    }
    pub fn positions(&self) -> &[BackgroundPosition] {
        self.positions.values()
    }
    pub fn repeats(&self) -> &[[bool; 2]] {
        self.repeats.values()
    }
    pub fn origins(&self) -> &[BackgroundBox] {
        self.origins.values()
    }
    pub fn clips(&self) -> &[BackgroundBox] {
        self.clips.values()
    }
    pub fn image_count(&self) -> usize {
        self.image_slots().len()
    }
    pub fn is_single_none(&self) -> bool {
        matches!(&self.images, BackgroundList::One(None))
    }
    pub fn layer(&self, index: usize) -> Option<UsedBackgroundLayer<'_>> {
        let image = self.image_slots().get(index)?.as_ref();
        Some(UsedBackgroundLayer {
            index,
            image,
            size: *self.sizes.used(index),
            position: *self.positions.used(index),
            repeat: *self.repeats.used(index),
            origin: *self.origins.used(index),
            clip: *self.clips.used(index),
        })
    }
    pub fn bottom_clip(&self) -> BackgroundBox {
        *self.clips.used(self.image_count() - 1)
    }
}

#[cfg(test)]
macro_rules! test_setter {
    ($name:ident, $field:ident, $ty:ty) => {
        pub(crate) fn $name(value: &mut Arc<Self>, item: $ty) {
            Arc::make_mut(value).$field = BackgroundList::One(item);
        }
    };
}
#[cfg(test)]
impl BackgroundData {
    test_setter!(test_replace_image, images, Option<BackgroundImage>);
    test_setter!(test_replace_size, sizes, BackgroundSize);
    test_setter!(test_replace_position, positions, BackgroundPosition);
    test_setter!(test_replace_repeat, repeats, [bool; 2]);
    test_setter!(test_replace_clip, clips, BackgroundBox);
}

#[derive(Default)]
pub(in crate::css) struct BackgroundBuilder {
    data: BackgroundData,
}

impl BackgroundBuilder {
    pub(in crate::css) fn copy_property(
        &mut self,
        name: &str,
        from: &BackgroundData,
        work: &mut usize,
    ) -> Result<bool, Error> {
        macro_rules! copy {
            ($field:ident) => {{
                selectors::spend(work, 4)?;
                self.data.$field = from.$field.clone();
            }};
        }
        match name {
            "background-image" => copy!(images),
            "background-size" => copy!(sizes),
            "background-position" => copy!(positions),
            "background-repeat" => copy!(repeats),
            "background-origin" => copy!(origins),
            "background-clip" => copy!(clips),
            _ => return Ok(false),
        }
        Ok(true)
    }
}

trait ItemKey: Clone {
    fn key(&self) -> [u64; ITEM_WORDS];
}

fn float(value: f32) -> u64 {
    u64::from(if value == 0.0 { 0 } else { value.to_bits() })
}
fn length(value: Length) -> [u64; 4] {
    match value {
        Length::Auto => [0, 0, 0, 0],
        Length::Px(v) => [1, float(v), 0, 0],
        Length::Percent(v) => [2, float(v), 0, 0],
        Length::Calc {
            px,
            percent,
            has_percent,
        } => [3, float(px), float(percent), u64::from(has_percent)],
        Length::Fr(v) => [4, float(v), 0, 0],
    }
}
impl ItemKey for Option<BackgroundImage> {
    fn key(&self) -> [u64; ITEM_WORDS] {
        let mut key = [0; ITEM_WORDS];
        match self {
            None => {}
            Some(BackgroundImage::Gradient(value)) => {
                key[0] = 1;
                key[1] = Arc::as_ptr(value) as usize as u64;
            }
            Some(BackgroundImage::Url(value)) => {
                key[0] = 2;
                key[1] = Arc::as_ptr(value) as usize as u64;
            }
        }
        key
    }
}
impl ItemKey for BackgroundSize {
    fn key(&self) -> [u64; ITEM_WORDS] {
        let mut key = [0; ITEM_WORDS];
        match *self {
            Self::Explicit { width, height } => {
                key[1..5].copy_from_slice(&length(width));
                key[5..9].copy_from_slice(&length(height));
            }
            Self::Cover => key[0] = 1,
            Self::Contain => key[0] = 2,
        }
        key
    }
}
impl ItemKey for BackgroundPosition {
    fn key(&self) -> [u64; ITEM_WORDS] {
        let mut key = [0; ITEM_WORDS];
        key[0] = u64::from(self.x.from_end);
        key[1..5].copy_from_slice(&length(self.x.offset));
        key[5] = u64::from(self.y.from_end);
        key[6..10].copy_from_slice(&length(self.y.offset));
        key
    }
}
impl ItemKey for [bool; 2] {
    fn key(&self) -> [u64; ITEM_WORDS] {
        let mut key = [0; ITEM_WORDS];
        key[0] = u64::from(self[0]) | (u64::from(self[1]) << 1);
        key
    }
}
impl ItemKey for BackgroundBox {
    fn key(&self) -> [u64; ITEM_WORDS] {
        let mut key = [0; ITEM_WORDS];
        key[0] = match self {
            Self::Border => 0,
            Self::Padding => 1,
            Self::Content => 2,
        };
        key
    }
}

fn compare_values<T: ItemKey>(a: &[T], b: &[T]) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| {
        a.iter()
            .zip(b)
            .map(|(a, b)| a.key().cmp(&b.key()))
            .find(|order| *order != Ordering::Equal)
            .unwrap_or(Ordering::Equal)
    })
}
fn compare_list<T: ItemKey>(a: &BackgroundList<T>, b: &BackgroundList<T>) -> Ordering {
    match (a, b) {
        (BackgroundList::One(a), BackgroundList::One(b)) => a.key().cmp(&b.key()),
        (BackgroundList::One(_), BackgroundList::Many(_)) => Ordering::Less,
        (BackgroundList::Many(_), BackgroundList::One(_)) => Ordering::Greater,
        (BackgroundList::Many(a), BackgroundList::Many(b)) => a
            .len()
            .cmp(&b.len())
            .then_with(|| (a.as_ptr() as usize).cmp(&(b.as_ptr() as usize))),
    }
}
fn compare_data(a: &BackgroundData, b: &BackgroundData) -> Ordering {
    compare_list(&a.images, &b.images)
        .then_with(|| compare_list(&a.sizes, &b.sizes))
        .then_with(|| compare_list(&a.positions, &b.positions))
        .then_with(|| compare_list(&a.repeats, &b.repeats))
        .then_with(|| compare_list(&a.origins, &b.origins))
        .then_with(|| compare_list(&a.clips, &b.clips))
}
fn levels(count: usize) -> usize {
    (usize::BITS - count.leading_zeros()) as usize
}
fn aligned(value: usize, align: usize) -> usize {
    value.saturating_add(align - 1) & !(align - 1)
}
fn arc_bytes<T>(count: usize) -> usize {
    let header = aligned(2 * size_of::<AtomicUsize>(), align_of::<T>());
    aligned(
        header.saturating_add(count.saturating_mul(size_of::<T>())),
        align_of::<T>().max(align_of::<AtomicUsize>()),
    )
    .saturating_add(32)
}
fn reserve_bytes(left: &mut usize, cost: usize) -> Result<(), Error> {
    if cost > *left {
        *left = 0;
        return Err(Error::Limit);
    }
    *left -= cost;
    Ok(())
}

struct ListPool<T> {
    entries: Vec<Arc<[T]>>,
    admitted_capacity: usize,
}
impl<T> Default for ListPool<T> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            admitted_capacity: 0,
        }
    }
}
impl<T: ItemKey> ListPool<T> {
    fn intern(
        &mut self,
        values: &[T],
        work: &mut usize,
        bytes: &mut usize,
    ) -> Result<BackgroundList<T>, Error> {
        if values.is_empty() || values.len() > MAX_BACKGROUND_LAYERS {
            return Err(Error::Invalid);
        }
        if values.len() == 1 {
            selectors::spend(work, 4)?;
            return Ok(BackgroundList::One(values[0].clone()));
        }
        let count = self.entries.len();
        let words = 1 + ITEM_WORDS * values.len();
        selectors::spend(
            work,
            8usize.saturating_add(words.saturating_mul(levels(count) + 1)),
        )?;
        let (mut lo, mut hi) = (0, count);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match compare_values(&self.entries[mid], values) {
                Ordering::Less => lo = mid + 1,
                Ordering::Greater => hi = mid,
                Ordering::Equal => {
                    selectors::spend(work, 4)?;
                    return Ok(BackgroundList::Many(Arc::clone(&self.entries[mid])));
                }
            }
        }
        let growth = count == self.admitted_capacity;
        let capacity = if growth {
            self.admitted_capacity.saturating_mul(2).max(1)
        } else {
            self.admitted_capacity
        };
        let entry = size_of::<Arc<[T]>>();
        let moves = (count - lo)
            .saturating_mul(entry)
            .saturating_add(if growth {
                count.saturating_mul(entry)
            } else {
                0
            });
        selectors::spend(
            work,
            16usize
                .saturating_add(ITEM_WORDS * values.len())
                .saturating_add(moves),
        )?;
        selectors::spend(work, 4)?;
        let storage = arc_bytes::<T>(values.len()).saturating_add(if growth {
            capacity.saturating_mul(entry).saturating_add(64)
        } else {
            0
        });
        reserve_bytes(bytes, storage)?;
        if growth {
            self.entries
                .try_reserve_exact(capacity - count)
                .map_err(|_| Error::Limit)?;
            self.admitted_capacity = capacity;
        }
        let value: Arc<[T]> = Arc::from(values);
        self.entries.insert(lo, Arc::clone(&value));
        Ok(BackgroundList::Many(value))
    }
}

#[derive(Default)]
struct RecordPool {
    entries: Vec<Arc<BackgroundData>>,
    admitted_capacity: usize,
}
impl RecordPool {
    fn intern(
        &mut self,
        data: BackgroundData,
        work: &mut usize,
        bytes: &mut usize,
    ) -> Result<Arc<BackgroundData>, Error> {
        selectors::spend(work, 8 + RECORD_WORDS)?;
        if compare_data(&data, &BackgroundData::default()) == Ordering::Equal {
            selectors::spend(work, 4)?;
            return Ok(BackgroundData::initial());
        }
        let count = self.entries.len();
        selectors::spend(work, RECORD_WORDS.saturating_mul(levels(count) + 1))?;
        let (mut lo, mut hi) = (0, count);
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            match compare_data(&self.entries[mid], &data) {
                Ordering::Less => lo = mid + 1,
                Ordering::Greater => hi = mid,
                Ordering::Equal => {
                    selectors::spend(work, 4)?;
                    return Ok(Arc::clone(&self.entries[mid]));
                }
            }
        }
        let growth = count == self.admitted_capacity;
        let capacity = if growth {
            self.admitted_capacity.saturating_mul(2).max(1)
        } else {
            self.admitted_capacity
        };
        let entry = size_of::<Arc<BackgroundData>>();
        let moves = (count - lo)
            .saturating_mul(entry)
            .saturating_add(if growth {
                count.saturating_mul(entry)
            } else {
                0
            });
        selectors::spend(
            work,
            16usize.saturating_add(RECORD_WORDS).saturating_add(moves),
        )?;
        selectors::spend(work, 4)?;
        reserve_bytes(
            bytes,
            arc_bytes::<BackgroundData>(1).saturating_add(if growth {
                capacity.saturating_mul(entry).saturating_add(64)
            } else {
                0
            }),
        )?;
        if growth {
            self.entries
                .try_reserve_exact(capacity - count)
                .map_err(|_| Error::Limit)?;
            self.admitted_capacity = capacity;
        }
        let value = Arc::new(data);
        self.entries.insert(lo, Arc::clone(&value));
        Ok(value)
    }
}

#[derive(Default)]
pub(in crate::css) struct BackgroundStorage {
    images: image::ImagePool,
    image_lists: ListPool<Option<BackgroundImage>>,
    sizes: ListPool<BackgroundSize>,
    positions: ListPool<BackgroundPosition>,
    repeats: ListPool<[bool; 2]>,
    origins: ListPool<BackgroundBox>,
    clips: ListPool<BackgroundBox>,
    records: RecordPool,
    charged: usize,
}

macro_rules! setter {
    ($name:ident, $pool:ident, $field:ident, $ty:ty) => {
        pub(in crate::css) fn $name(
            &mut self,
            builder: &mut BackgroundBuilder,
            values: &[$ty],
            work: &mut usize,
        ) -> Result<(), Error> {
            let free = MAX_BYTES
                .saturating_sub(self.images.bytes())
                .saturating_sub(self.charged);
            let mut left = free;
            let result = self.$pool.intern(values, work, &mut left);
            self.charged += free - left;
            builder.data.$field = result?;
            Ok(())
        }
    };
}
impl BackgroundStorage {
    setter!(set_images, image_lists, images, Option<BackgroundImage>);
    setter!(set_sizes, sizes, sizes, BackgroundSize);
    setter!(set_positions, positions, positions, BackgroundPosition);
    setter!(set_repeats, repeats, repeats, [bool; 2]);
    setter!(set_origins, origins, origins, BackgroundBox);
    setter!(set_clips, clips, clips, BackgroundBox);
    pub(in crate::css) fn gradient(
        &mut self,
        parsed: &gradient::ParsedGradient,
        work: &mut usize,
    ) -> Result<Option<Arc<LinearGradient>>, Error> {
        self.images
            .gradient_with_reserved(parsed, work, self.charged)
    }
    pub(in crate::css) fn url(
        &mut self,
        raw: &str,
        origin: WinningSource,
        rules: &[Rule],
        document_base: &::url::Url,
        work: &mut usize,
    ) -> Result<Option<Arc<url::ResolvedUrl>>, Error> {
        self.images
            .url_with_reserved(raw, origin, rules, document_base, work, self.charged)
    }
    pub(in crate::css) fn finish(
        &mut self,
        builder: BackgroundBuilder,
        work: &mut usize,
    ) -> Result<Option<Arc<BackgroundData>>, Error> {
        let free = MAX_BYTES
            .saturating_sub(self.images.bytes())
            .saturating_sub(self.charged);
        let mut left = free;
        let result = self.records.intern(builder.data, work, &mut left);
        self.charged += free - left;
        result.map(Some)
    }
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
