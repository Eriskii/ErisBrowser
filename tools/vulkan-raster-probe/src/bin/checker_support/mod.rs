//! Shared bounded checker capture helpers, extracted without semantic changes.
use eris::{
    graphics::{Color, DrawCommand, RasterImage, Rect},
    page::TaskState,
    worker::Snapshot,
};
use eris_vulkan_raster_prototype::{Frame, Result};
use std::{
    fmt::Write as _,
    fs,
    io::Read,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::Instant,
};

const MAX_SNAPSHOT_DUMP: usize = 65_536;
const MAX_OWN_TASKS: usize = 64;

pub(super) fn reserve<T>(count: usize) -> Result<Vec<T>> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| "bounded checker allocation")?;
    Ok(result)
}

pub(super) fn check_capture_deadline(deadline: Instant) -> Result<()> {
    if Instant::now() >= deadline {
        Err("capture deadline".into())
    } else {
        Ok(())
    }
}

// This is a refusal check, not a reaper. The outer subreaper owns cleanup.
// Every task is checked because Linux children lists are per-thread.
pub(super) fn ensure_no_children() -> Result<usize> {
    let mut tasks = 0usize;
    for entry in fs::read_dir("/proc/self/task").map_err(|e| format!("own tasks: {e}"))? {
        tasks += 1;
        if tasks > MAX_OWN_TASKS {
            return Err("own task inventory limit".into());
        }
        let entry = entry.map_err(|e| format!("own task entry: {e}"))?;
        let mut file = fs::File::open(entry.path().join("children"))
            .map_err(|e| format!("own task children: {e}"))?;
        let mut bytes = [0u8; 4097];
        let mut length = 0;
        loop {
            let count = file
                .read(&mut bytes[length..])
                .map_err(|e| format!("own child inventory: {e}"))?;
            if count == 0 {
                break;
            }
            length += count;
            if length > 4096 {
                return Err("own child inventory exceeds bound".into());
            }
        }
        if bytes[..length].iter().any(|b| !b.is_ascii_whitespace()) {
            return Err("capture has unresolved children".into());
        }
    }
    if tasks == 0 {
        return Err("missing own task inventory".into());
    }
    Ok(tasks)
}

pub(super) fn file_address(path: &Path) -> Result<String> {
    let bytes = path.as_os_str().as_bytes();
    if !path.is_absolute() || bytes.len() > 4096 {
        return Err("fixture path bound".into());
    }
    let mut address = String::new();
    address
        .try_reserve_exact(7 + bytes.len() * 3)
        .map_err(|_| "fixture address allocation")?;
    address.push_str("file://");
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || b"/-._~".contains(&byte) {
            address.push(char::from(byte));
        } else {
            write!(address, "%{byte:02X}").map_err(|_| "fixture address formatting")?;
        }
    }
    Ok(address)
}

pub(super) fn fixture_path(directory: &Path, name: &str) -> Result<PathBuf> {
    let relative = Path::new(name);
    if name.len() > 4096
        || relative
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("fixture relative path".into());
    }
    let path = directory
        .join(relative)
        .canonicalize()
        .map_err(|e| format!("fixture path: {e}"))?;
    if !path.starts_with(directory) || !path.is_file() {
        return Err("fixture outside supplied directory".into());
    }
    Ok(path)
}

// Canonical capture format EWB1, all integers little-endian; strings are u32
// byte length + exact UTF-8. No field is normalized or removed.
// Header: magic, frame width/height u32, generation/edit sequence u64, task u8,
// content-height f32 bits, load-ms f64 bits, title string, URL string.
// Commands: u32 count, then tags 0 clip(rect), 1 pop-clip, 2 fixed, 3 pop-fixed,
// 4 opacity(f32), 5 pop-opacity, 6 rect(rect,RGBA,radius),
// 7 text(x,y,size,RGBA,bold,italic,mono,text), 8 image(rect,key),
// 9 line(x1,y1,x2,y2,width,RGBA). Rect is x/y/width/height f32 bits.
// Images: u32 key count, sorted (key string, u32 source ID); then u32 source
// count and each (width,height,RGBA byte count,exact bytes). Arc identity gets
// first sorted-key IDs, never pointer addresses. Finally u32 diagnostic count
// followed by each exact string. Snapshot DOM/hit-regions are not serialized.
struct Dump {
    bytes: Option<Vec<u8>>,
    len: usize,
}
impl Dump {
    fn raw(&mut self, bytes: &[u8]) -> Result<()> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .ok_or("capture size overflow")?;
        if self.len > MAX_SNAPSHOT_DUMP {
            return Err("worker capture record too large".into());
        }
        if let Some(out) = &mut self.bytes {
            if self.len > out.capacity() {
                return Err("capture reservation mismatch".into());
            }
            out.extend_from_slice(bytes);
        }
        Ok(())
    }
    fn count(&mut self, n: usize) -> Result<()> {
        self.raw(
            &u32::try_from(n)
                .map_err(|_| "capture count overflow")?
                .to_le_bytes(),
        )
    }
    fn string(&mut self, s: &str) -> Result<()> {
        self.count(s.len())?;
        self.raw(s.as_bytes())
    }
    fn scalar(&mut self, v: f32) -> Result<()> {
        self.raw(&v.to_bits().to_le_bytes())
    }
    fn rect(&mut self, rect: Rect) -> Result<()> {
        for v in [rect.x, rect.y, rect.width, rect.height] {
            self.scalar(v)?;
        }
        Ok(())
    }
    fn color(&mut self, color: Color) -> Result<()> {
        self.raw(&[color.r, color.g, color.b, color.a])
    }
}

pub(super) fn dump_snapshot(snapshot: &Snapshot, frame: Frame) -> Result<Vec<u8>> {
    if snapshot.layout.commands.len() > 256
        || snapshot.images.len() > 256
        || snapshot.images.capacity() > 512
        || snapshot.diagnostics.len() > 256
    {
        return Err("worker capture inventory exceeds fixture contract".into());
    }
    let mut key_bytes = 0usize;
    for key in snapshot.images.keys() {
        key_bytes = key_bytes
            .checked_add(key.len())
            .ok_or("capture key overflow")?;
        if key.len() > 4096 || key_bytes > 65_536 {
            return Err("worker capture key bound".into());
        }
    }
    let mut entries = reserve(snapshot.images.len())?;
    entries.extend(snapshot.images.iter());
    // At most 256 entries and 64 KiB of key bytes, bounded before sorting.
    for i in 1..entries.len() {
        let mut j = i;
        while j > 0 && entries[j].0 < entries[j - 1].0 {
            entries.swap(j, j - 1);
            j -= 1;
        }
    }
    let mut sources: Vec<&Arc<RasterImage>> = reserve(entries.len())?;
    let mut ids = reserve(entries.len())?;
    for (_, image) in &entries {
        let id = if let Some(id) = sources.iter().position(|old| Arc::ptr_eq(old, image)) {
            id
        } else {
            sources.push(image);
            sources.len() - 1
        };
        ids.push(id);
    }
    let serialize = |out: &mut Dump| -> Result<()> {
        out.raw(b"EWB1")?;
        out.raw(&frame.width.to_le_bytes())?;
        out.raw(&frame.height.to_le_bytes())?;
        out.raw(&snapshot.generation.to_le_bytes())?;
        out.raw(&snapshot.processed_edit_sequence.to_le_bytes())?;
        out.raw(&[match snapshot.task_state {
            TaskState::Idle => 0,
            TaskState::Pending => 1,
            TaskState::Suspended => 2,
        }])?;
        out.scalar(snapshot.layout.content_height)?;
        out.raw(&snapshot.load_ms.to_bits().to_le_bytes())?;
        out.string(&snapshot.title)?;
        out.string(&snapshot.url)?;
        out.count(snapshot.layout.commands.len())?;
        for command in &snapshot.layout.commands {
            match command {
                DrawCommand::PushClip { rect } => {
                    out.raw(&[0])?;
                    out.rect(*rect)?;
                }
                DrawCommand::PopClip => out.raw(&[1])?,
                DrawCommand::PushFixed => out.raw(&[2])?,
                DrawCommand::PopFixed => out.raw(&[3])?,
                DrawCommand::PushOpacity { opacity } => {
                    out.raw(&[4])?;
                    out.scalar(*opacity)?;
                }
                DrawCommand::PopOpacity => out.raw(&[5])?,
                DrawCommand::Rect {
                    rect,
                    color,
                    radius,
                } => {
                    out.raw(&[6])?;
                    out.rect(*rect)?;
                    out.color(*color)?;
                    out.scalar(*radius)?;
                }
                DrawCommand::Text {
                    x,
                    y,
                    text,
                    size,
                    color,
                    bold,
                    italic,
                    monospace,
                } => {
                    out.raw(&[7])?;
                    for v in [*x, *y, *size] {
                        out.scalar(v)?;
                    }
                    out.color(*color)?;
                    out.raw(&[u8::from(*bold), u8::from(*italic), u8::from(*monospace)])?;
                    out.string(text)?;
                }
                DrawCommand::Image { rect, key } => {
                    out.raw(&[8])?;
                    out.rect(*rect)?;
                    out.string(key)?;
                }
                DrawCommand::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    color,
                    width,
                } => {
                    out.raw(&[9])?;
                    for v in [*x1, *y1, *x2, *y2, *width] {
                        out.scalar(v)?;
                    }
                    out.color(*color)?;
                }
            }
        }
        out.count(entries.len())?;
        for ((key, _), id) in entries.iter().zip(&ids) {
            out.string(key)?;
            out.count(*id)?;
        }
        out.count(sources.len())?;
        for image in &sources {
            out.raw(&image.width.to_le_bytes())?;
            out.raw(&image.height.to_le_bytes())?;
            out.count(image.rgba.len())?;
            out.raw(&image.rgba)?;
        }
        out.count(snapshot.diagnostics.len())?;
        for diagnostic in &snapshot.diagnostics {
            out.string(diagnostic)?;
        }
        Ok(())
    };
    let mut measure = Dump {
        bytes: None,
        len: 0,
    };
    serialize(&mut measure)?;
    let mut output = Dump {
        bytes: Some(reserve(measure.len)?),
        len: 0,
    };
    serialize(&mut output)?;
    if output.len != measure.len {
        return Err("capture length changed".into());
    }
    output.bytes.ok_or_else(|| "missing capture bytes".into())
}

pub(super) fn print_snapshot(name: &str, bytes: &[u8]) -> Result<()> {
    let mut hex = String::new();
    hex.try_reserve_exact(bytes.len().checked_mul(2).ok_or("capture hex overflow")?)
        .map_err(|_| "capture hex allocation")?;
    for byte in bytes {
        write!(hex, "{byte:02x}").map_err(|_| "capture hex formatting")?;
    }
    println!("SNAPSHOT {name} bytes={} hex={hex}", bytes.len());
    Ok(())
}

pub(super) fn read_gpu_grant(input: &mut impl Read) -> Result<()> {
    const EXPECTED: &[u8] = b"GPU_READY\n";
    let mut token = [0u8; EXPECTED.len()];
    input
        .read_exact(&mut token)
        .map_err(|e| format!("GPU capture grant: {e}"))?;
    if token != EXPECTED {
        return Err("invalid GPU capture grant".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_grant_requires_exact_token_and_complete_input() {
        assert!(read_gpu_grant(&mut &b"GPU_READY\n"[..]).is_ok());
        assert!(read_gpu_grant(&mut &b"GPU_READY"[..]).is_err());
        assert!(read_gpu_grant(&mut &b"BAD_READY\n"[..]).is_err());
        assert!(read_gpu_grant(&mut &b""[..]).is_err());
    }

    #[test]
    fn fixture_file_address_encodes_reserved_and_utf8_bytes() {
        assert_eq!(
            file_address(Path::new("/tmp/a b#c%/é.html")).unwrap(),
            "file:///tmp/a%20b%23c%25/%C3%A9.html"
        );
        assert!(file_address(Path::new("relative.html")).is_err());
    }

    #[test]
    fn capture_measure_rejects_oversized_payload_before_output_allocation() {
        let source = [0u8; MAX_SNAPSHOT_DUMP + 1];
        let mut measure = Dump {
            bytes: None,
            len: 0,
        };
        assert!(measure.raw(&source).is_err());
        assert!(measure.bytes.is_none());
    }

    #[test]
    fn canonical_record_keeps_utf8_and_negative_zero_bits() {
        let serialize = |out: &mut Dump| -> Result<()> {
            out.string("\0é")?;
            out.scalar(-0.0)
        };
        let mut measure = Dump {
            bytes: None,
            len: 0,
        };
        serialize(&mut measure).unwrap();
        assert_eq!(measure.len, 11);
        let mut out = Dump {
            bytes: Some(reserve(measure.len).unwrap()),
            len: 0,
        };
        serialize(&mut out).unwrap();
        assert_eq!(
            out.bytes.unwrap(),
            [3, 0, 0, 0, 0, 0xc3, 0xa9, 0, 0, 0, 0x80]
        );
    }
}
