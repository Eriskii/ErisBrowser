//! Computed image interning. URL fees share the existing gradient admission.
use super::{Rule, SourceBase, WinningSource, gradient, url};
use crate::graphics::gradient::LinearGradient;
use crate::selectors::{self, Error};
use std::cmp::Ordering;
use std::mem::{align_of, size_of};
use std::sync::{Arc, atomic::AtomicUsize};

const MAX_IMAGE_BYTES: usize = gradient::MAX_RETAINED_GRADIENT_BYTES;

struct UrlEntry {
    source: u32,
    raw: String,
    value: Arc<url::ResolvedUrl>,
}
#[derive(Default)]
struct UrlImagePool {
    entries: Vec<UrlEntry>,
    admitted_capacity: usize,
    charged_bytes: usize,
}
#[derive(Default)]
pub(super) struct ImagePool {
    gradients: gradient::GradientPool,
    urls: UrlImagePool,
}

fn align_up(value: usize, alignment: usize) -> usize {
    value.saturating_add(alignment - 1) & !(alignment - 1)
}
fn arc_bytes() -> usize {
    let alignment = align_of::<url::ResolvedUrl>()
        .max(align_of::<AtomicUsize>())
        .max(2);
    align_up(
        align_up(2 * size_of::<AtomicUsize>(), align_of::<url::ResolvedUrl>())
            .saturating_add(size_of::<url::ResolvedUrl>()),
        alignment,
    )
    .saturating_add(32)
}
fn bytes(budget: &mut url::UrlBudget, cost: usize) -> Result<(), Error> {
    if cost > budget.bytes {
        budget.bytes = 0;
        return Err(Error::Limit);
    }
    budget.bytes -= cost;
    Ok(())
}

impl ImagePool {
    pub(super) fn gradient(
        &mut self,
        parsed: &gradient::ParsedGradient,
        work: &mut usize,
    ) -> Result<Option<Arc<LinearGradient>>, Error> {
        if self.urls.charged_bytes == 0 {
            self.gradients.intern(parsed, work)
        } else {
            self.gradients
                .intern_with_reserved(parsed, work, self.urls.charged_bytes)
        }
    }

    pub(super) fn url(
        &mut self,
        raw: &str,
        origin: WinningSource,
        rules: &[Rule],
        document_base: &::url::Url,
        work: &mut usize,
    ) -> Result<Option<Arc<url::ResolvedUrl>>, Error> {
        selectors::spend(work, 8)?;
        if raw.len() > 4096 {
            return Err(Error::Limit);
        }
        let document_source = SourceBase::Document;
        let base = match origin {
            WinningSource::Document => &document_source,
            WinningSource::Rule(index) => &rules.get(index as usize).ok_or(Error::Invalid)?.base,
        };
        let source = match (origin, base) {
            (_, SourceBase::Document) => u32::MAX,
            (WinningSource::Rule(index), SourceBase::Stylesheet(_)) => index,
            _ => return Err(Error::Invalid),
        };
        let (mut lo, mut hi) = (0, self.urls.entries.len());
        while lo < hi {
            selectors::spend(work, raw.len().saturating_add(6))?;
            let mid = lo + (hi - lo) / 2;
            let entry = &self.urls.entries[mid];
            match entry
                .source
                .cmp(&source)
                .then_with(|| entry.raw.as_str().cmp(raw))
            {
                Ordering::Less => lo = mid + 1,
                Ordering::Greater => hi = mid,
                Ordering::Equal => {
                    selectors::spend(work, 4)?;
                    return Ok(Some(Arc::clone(&entry.value)));
                }
            }
        }
        let count = self.urls.entries.len();
        let growth = count == self.urls.admitted_capacity;
        let capacity = if growth {
            self.urls.admitted_capacity.saturating_mul(2).max(1)
        } else {
            self.urls.admitted_capacity
        };
        let entry_bytes = size_of::<UrlEntry>();
        let move_bytes = (count - lo)
            .saturating_mul(entry_bytes)
            .saturating_add(if growth {
                count.saturating_mul(entry_bytes)
            } else {
                0
            });
        selectors::spend(
            work,
            16usize.saturating_add(raw.len()).saturating_add(move_bytes),
        )?;
        let free = MAX_IMAGE_BYTES
            .saturating_sub(self.gradients.bytes())
            .saturating_sub(self.urls.charged_bytes);
        let mut budget = url::UrlBudget {
            work: *work,
            bytes: free,
        };
        // Do not return through `?` outside this closure before budget writeback.
        let result = (|| {
            let storage = raw
                .len()
                .saturating_add(32)
                .saturating_add(arc_bytes())
                .saturating_add(if growth {
                    capacity.saturating_mul(entry_bytes).saturating_add(64)
                } else {
                    0
                });
            bytes(&mut budget, storage)?;
            let Some(decoded) = url::decode(raw, &mut budget).map_err(|_| Error::Limit)? else {
                return Ok(None);
            };
            let resolved = url::resolve(&decoded, base, document_base, &mut budget)
                .map_err(|_| Error::Limit)?;
            let mut key = String::new();
            key.try_reserve_exact(raw.len()).map_err(|_| Error::Limit)?;
            key.push_str(raw);
            if growth {
                self.urls
                    .entries
                    .try_reserve_exact(capacity - count)
                    .map_err(|_| Error::Limit)?;
                self.urls.admitted_capacity = capacity;
            }
            let value = Arc::new(resolved);
            self.urls.entries.insert(
                lo,
                UrlEntry {
                    source,
                    raw: key,
                    value: Arc::clone(&value),
                },
            );
            Ok(Some(value))
        })();
        *work = budget.work;
        self.urls.charged_bytes += free - budget.bytes;
        result
    }
}

pub(super) fn url_payload(token: selectors::Token<'_>) -> bool {
    matches!(token.kind, selectors::Kind::Url | selectors::Kind::String)
        || token.kind == selectors::Kind::Function && token.is_name("url")
}

/// Image-only normalization copies token payloads verbatim. In particular,
/// slash-star inside an unquoted URL is data, not an outer CSS comment.
pub(super) fn comments(value: &str, work: &mut usize) -> Result<String, Error> {
    selectors::spend(work, value.len().saturating_mul(3).saturating_add(8))?;
    let mut out = String::new();
    out.try_reserve_exact(value.len())
        .map_err(|_| Error::Limit)?;
    let mut at = 0;
    while at < value.len() {
        if value[at..].starts_with("/*") {
            let Some(end) = value[at + 2..].find("*/") else {
                return Err(Error::Invalid);
            };
            at += end + 4;
            out.push(' ');
        } else {
            let (_, count) = selectors::token(&value[at..]);
            out.push_str(&value[at..at + count]);
            at += count;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
