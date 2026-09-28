//! Bounded resource loading with explicit URL and local-file policy.
use base64::Engine;
use std::{
    cell::Cell,
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use url::Url;

pub const MAX_RESOURCE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_PAGE_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_RESOURCES: usize = 48;
pub const MAX_FORM_BODY_BYTES: usize = 1024 * 1024;

thread_local! {
    static PAGE_FETCH_BRIDGE: Cell<Option<FetchBridge>> = const { Cell::new(None) };
    static SYNCHRONOUS_BROKER_DNS: Cell<bool> = const { Cell::new(false) };
}

pub(crate) type FetchBridge =
    fn(&Url, Option<&Url>, ResourceKind, Option<&str>) -> Result<Resource, String>;
/// Install a pipe-only fetch path before executing untrusted page code.
pub(crate) fn use_page_fetch_bridge(bridge: FetchBridge) {
    PAGE_FETCH_BRIDGE.set(Some(bridge));
}

/// The confined resource broker cannot create the timeout thread used by ureq's
/// default resolver. Its parent enforces cancellation and a process deadline,
/// including time spent in synchronous system DNS. Other threads keep ureq's
/// ordinary resolver timeout behavior.
pub(crate) fn use_synchronous_dns_for_broker() {
    SYNCHRONOUS_BROKER_DNS.set(true);
}

#[derive(Debug)]
struct BrokerResolver;

impl ureq::unversioned::resolver::Resolver for BrokerResolver {
    fn resolve(
        &self,
        uri: &ureq::http::Uri,
        config: &ureq::config::Config,
        timeout: ureq::unversioned::transport::NextTimeout,
    ) -> Result<ureq::unversioned::resolver::ResolvedSocketAddrs, ureq::Error> {
        ureq::unversioned::resolver::DefaultResolver::default().resolve(
            uri,
            config,
            ureq::unversioned::transport::NextTimeout {
                after: ureq::unversioned::transport::time::Duration::NotHappening,
                reason: timeout.reason,
            },
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Document,
    Style,
    Script,
    Image,
}
#[derive(Debug)]
pub struct Resource {
    pub url: Url,
    pub bytes: Vec<u8>,
    pub content_type: String,
    pub headers: BTreeMap<String, String>,
    pub status: u16,
    /// False if any network redirect crosses the document origin.
    pub origin_clean: bool,
    /// Pixels from the isolated decoder; opaque responses have no body or headers.
    pub decoded_image: Option<crate::graphics::RasterImage>,
}
impl Resource {
    pub fn text(&self) -> String {
        let label = self.content_type.split(';').skip(1).find_map(|p| {
            let (key, value) = p.trim().split_once('=')?;
            key.eq_ignore_ascii_case("charset")
                .then(|| value.trim_matches(['\'', '"', ' ']))
        });
        let encoding = encoding_rs::Encoding::for_bom(&self.bytes)
            .map(|value| value.0)
            .or_else(|| label.and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes())));
        if let Some(e) = encoding {
            e.decode(&self.bytes).0.into_owned()
        } else {
            String::from_utf8_lossy(&self.bytes).into_owned()
        }
    }
}

pub struct Fetcher {
    agent: Option<ureq::Agent>,
    bridge: Option<FetchBridge>,
    local_root: Option<PathBuf>,
    total: usize,
    count: usize,
    started: Instant,
}
impl Default for Fetcher {
    fn default() -> Self {
        Self::new(None)
    }
}
impl Fetcher {
    pub fn new(local_root: Option<PathBuf>) -> Self {
        let bridge = PAGE_FETCH_BRIDGE.get();
        let agent = bridge.is_none().then(|| {
            let config = ureq::Agent::config_builder()
                .max_redirects(0)
                .http_status_as_error(false)
                .timeout_global(Some(Duration::from_secs(12)))
                .user_agent("ErisBrowser/0.1 (independent experimental engine)")
                .build();
            if SYNCHRONOUS_BROKER_DNS.get() {
                ureq::Agent::with_parts(
                    config,
                    ureq::unversioned::transport::DefaultConnector::default(),
                    BrokerResolver,
                )
            } else {
                config.into()
            }
        });
        Self {
            agent,
            bridge,
            local_root,
            total: 0,
            count: 0,
            started: Instant::now(),
        }
    }
    pub fn for_document(url: &Url) -> Self {
        if PAGE_FETCH_BRIDGE.get().is_some() {
            return Self::new(None);
        }
        let root = url
            .to_file_path()
            .ok()
            .and_then(|p| p.canonicalize().ok())
            .and_then(|p| p.parent().map(Path::to_path_buf));
        Self::new(root)
    }
    pub fn fetch(
        &mut self,
        url: &Url,
        initiator: Option<&Url>,
        kind: ResourceKind,
    ) -> Result<Resource, String> {
        self.fetch_request(url, initiator, kind, None)
    }
    /// Fetch a top-level document, optionally submitting a bounded URL-encoded form.
    pub fn fetch_document(
        &mut self,
        url: &Url,
        form_body: Option<&str>,
    ) -> Result<Resource, String> {
        if let Some(body) = form_body {
            if body.len() > MAX_FORM_BODY_BYTES {
                return Err("form submission byte budget exceeded".into());
            }
            if !matches!(url.scheme(), "http" | "https") {
                return Err("POST form submissions require HTTP or HTTPS".into());
            }
        }
        self.fetch_request(url, None, ResourceKind::Document, form_body)
    }
    fn fetch_request(
        &mut self,
        url: &Url,
        initiator: Option<&Url>,
        kind: ResourceKind,
        mut form_body: Option<&str>,
    ) -> Result<Resource, String> {
        if let Some(bridge) = self.bridge {
            return bridge(url, initiator, kind, form_body);
        }
        self.count += 1;
        if self.count > MAX_RESOURCES || self.started.elapsed() > Duration::from_secs(30) {
            return Err("page resource or time budget exceeded".into());
        }
        let mut target = url.clone();
        let mut origin_clean = true;
        for _ in 0..=8 {
            let remaining_time = Duration::from_secs(30)
                .checked_sub(self.started.elapsed())
                .filter(|time| !time.is_zero())
                .ok_or("page time budget exceeded")?;
            let remaining_bytes = MAX_PAGE_BYTES.saturating_sub(self.total);
            self.validate(&target, initiator, kind)?;
            origin_clean &= initiator.is_none_or(|source| image_origin_clean(source, &target));
            let mut resource = match target.scheme() {
                "http" | "https" => {
                    let timeout = Some(remaining_time.min(Duration::from_secs(12)));
                    let mut request_url = target.clone();
                    request_url.set_fragment(None);
                    let agent = self
                        .agent
                        .as_ref()
                        .ok_or("direct network access unavailable")?;
                    let mut response = if let Some(body) = form_body {
                        agent
                            .post(request_url.as_str())
                            .header("Content-Type", "application/x-www-form-urlencoded")
                            .config()
                            .timeout_global(timeout)
                            .build()
                            .send(body.as_bytes())
                    } else {
                        agent
                            .get(request_url.as_str())
                            .config()
                            .timeout_global(timeout)
                            .build()
                            .call()
                    }
                    .map_err(|error| error.to_string())?;
                    let status = response.status().as_u16();
                    if matches!(status, 301 | 302 | 303 | 307 | 308) {
                        let next = response
                            .headers()
                            .get("location")
                            .and_then(|v| v.to_str().ok())
                            .ok_or("redirect has no valid Location")?;
                        let next = target.join(next).map_err(|e| e.to_string())?;
                        if target.scheme() == "https" && next.scheme() != "https" {
                            return Err("HTTPS downgrade redirect blocked".into());
                        }
                        if !matches!(next.scheme(), "http" | "https") {
                            return Err("HTTP redirects require an HTTP or HTTPS target".into());
                        }
                        if matches!(status, 301..=303) {
                            form_body = None;
                        }
                        target = next;
                        continue;
                    }
                    let headers = response
                        .headers()
                        .iter()
                        .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_owned())))
                        .collect::<BTreeMap<_, _>>();
                    let content_type = headers.get("content-type").cloned().unwrap_or_default();
                    if let Some(length) = headers
                        .get("content-length")
                        .and_then(|v| v.parse::<usize>().ok())
                        && length > MAX_RESOURCE_BYTES
                    {
                        return Err("response exceeds resource byte budget".into());
                    }
                    // Limit the decompressed stream too; Content-Length alone is insufficient.
                    let bytes = read_bounded(
                        response
                            .body_mut()
                            .with_config()
                            .limit((MAX_RESOURCE_BYTES as u64).saturating_add(1))
                            .reader(),
                        MAX_RESOURCE_BYTES.min(remaining_bytes),
                    )?;
                    Resource {
                        url: target.clone(),
                        bytes,
                        content_type,
                        headers,
                        status,
                        origin_clean,
                        decoded_image: None,
                    }
                }
                "file" => {
                    let path = target
                        .to_file_path()
                        .map_err(|_| "invalid file URL")?
                        .canonicalize()
                        .map_err(|e| e.to_string())?;
                    if !self
                        .local_root
                        .as_ref()
                        .is_some_and(|root| path.starts_with(root))
                    {
                        return Err("file escapes the explicitly opened document directory".into());
                    }
                    if !path.is_file() {
                        return Err("only regular local files can be opened".into());
                    }
                    let bytes = read_bounded(
                        File::open(&path).map_err(|e| e.to_string())?,
                        MAX_RESOURCE_BYTES.min(remaining_bytes),
                    )?;
                    Resource {
                        url: target.clone(),
                        bytes,
                        content_type: mime_for_path(&path).into(),
                        headers: BTreeMap::new(),
                        status: 200,
                        origin_clean,
                        decoded_image: None,
                    }
                }
                "data" => decode_data_url(&target)?,
                _ => return Err("unsupported URL scheme".into()),
            };
            let next_total = self
                .total
                .checked_add(resource.bytes.len())
                .ok_or("resource size overflow")?;
            if next_total > MAX_PAGE_BYTES {
                return Err("page byte budget exceeded".into());
            }
            self.total = next_total;
            if kind != ResourceKind::Document && !(200..300).contains(&resource.status) {
                return Err(format!("subresource returned HTTP {}", resource.status));
            }
            if kind == ResourceKind::Style
                && !resource
                    .content_type
                    .split(';')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .eq_ignore_ascii_case("text/css")
            {
                return Err("stylesheet has an unsupported MIME type".into());
            }
            if kind == ResourceKind::Script && !is_javascript_mime(&resource.content_type) {
                return Err("script has an unsupported MIME type".into());
            }
            resource.origin_clean = origin_clean;
            resource.url = target;
            return Ok(resource);
        }
        Err("redirect limit exceeded".into())
    }
    fn validate(
        &self,
        url: &Url,
        initiator: Option<&Url>,
        kind: ResourceKind,
    ) -> Result<(), String> {
        if url.as_str().len() > MAX_RESOURCE_BYTES * 2 {
            return Err("URL exceeds size limit".into());
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err("URLs containing credentials are blocked".into());
        }
        if !matches!(url.scheme(), "https" | "http" | "file" | "data") {
            return Err(format!("unsupported scheme: {}", url.scheme()));
        }
        if let Some(source) = initiator {
            if url.scheme() == "file" && source.scheme() != "file" {
                return Err("remote documents cannot load local files".into());
            }
            if source.scheme() == "https" && url.scheme() == "http" {
                return Err("mixed content blocked".into());
            }
            // Until full CORS/CSP/credentials handling exists, active subresources are same-origin.
            if matches!(kind, ResourceKind::Script | ResourceKind::Style)
                && matches!(url.scheme(), "http" | "https")
                && source.origin() != url.origin()
            {
                return Err("cross-origin active subresource blocked".into());
            }
            if url.scheme() == "data" && kind != ResourceKind::Image {
                return Err("data URLs are permitted only for images in documents".into());
            }
        }
        Ok(())
    }
}

/// Local images are restricted to the authorized directory; data images have no
/// external response origin. Network image taint survives every redirect hop.
pub(crate) fn image_origin_clean(source: &Url, target: &Url) -> bool {
    match target.scheme() {
        "data" => true,
        "file" => source.scheme() == "file",
        "http" | "https" => source.origin() == target.origin(),
        _ => false,
    }
}

pub fn read_bounded(mut reader: impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .by_ref()
        .take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > limit {
        Err("resource exceeds byte budget".into())
    } else {
        Ok(bytes)
    }
}

pub fn is_javascript_mime(mime: &str) -> bool {
    matches!(
        mime.split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "text/javascript"
            | "application/javascript"
            | "application/ecmascript"
            | "text/ecmascript"
            | "application/x-javascript"
    )
}
pub fn parse_address(address: &str) -> Result<Url, String> {
    let address = address.trim();
    if address.len() > MAX_RESOURCE_BYTES * 2 {
        return Err("address exceeds size limit".into());
    }
    if let Ok(url) = Url::parse(address)
        && matches!(url.scheme(), "about" | "eris")
    {
        return Ok(url);
    }
    let path = Path::new(address);
    if path.exists()
        || address.starts_with('/')
        || address.starts_with("./")
        || address.starts_with("../")
    {
        let absolute = path.canonicalize().map_err(|e| e.to_string())?;
        return Url::from_file_path(absolute).map_err(|_| "invalid file path".into());
    }
    if !address.contains("://") && !address.starts_with("data:") {
        return Url::parse(&format!("https://{address}")).map_err(|e| e.to_string());
    }
    Url::parse(address).map_err(|e| e.to_string())
}
fn mime_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "text/javascript",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => "text/plain",
    }
}
fn decode_data_url(url: &Url) -> Result<Resource, String> {
    let mut without_fragment = url.clone();
    without_fragment.set_fragment(None);
    let raw = without_fragment
        .as_str()
        .strip_prefix("data:")
        .ok_or("invalid data URL")?;
    let (meta, data) = raw.split_once(',').ok_or("invalid data URL")?;
    let bytes = percent_decode(data)?;
    let bytes = if meta.ends_with(";base64") {
        base64::engine::general_purpose::STANDARD
            .decode(bytes)
            .map_err(|e| e.to_string())?
    } else {
        bytes
    };
    if bytes.len() > MAX_RESOURCE_BYTES {
        return Err("data URL exceeds byte budget".into());
    }
    let content_type = meta.strip_suffix(";base64").unwrap_or(meta);
    let content_type = if content_type.is_empty() {
        "text/plain;charset=US-ASCII"
    } else {
        content_type
    };
    Ok(Resource {
        url: url.clone(),
        bytes,
        content_type: content_type.into(),
        headers: BTreeMap::new(),
        status: 200,
        origin_clean: true,
        decoded_image: None,
    })
}
pub(crate) fn percent_decode(input: &str) -> Result<Vec<u8>, String> {
    percent_decode_with_limit(input, MAX_RESOURCE_BYTES)
}
fn percent_decode_with_limit(input: &str, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let input = input.as_bytes();
    let mut i = 0;
    while i < input.len() {
        if bytes.len() >= limit {
            return Err("data URL exceeds byte budget".into());
        }
        if input[i] == b'%' && i + 2 < input.len() {
            let h = (input[i + 1] as char).to_digit(16);
            let l = (input[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (h, l) {
                bytes.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        bytes.push(input[i]);
        i += 1;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejected_data_resource_does_not_poison_remaining_budget() {
        let mut f = Fetcher {
            total: MAX_PAGE_BYTES - 2,
            ..Fetcher::default()
        };
        let large = Url::parse("data:text/plain,123").unwrap();
        assert!(f.fetch(&large, None, ResourceKind::Document).is_err());
        assert_eq!(f.total, MAX_PAGE_BYTES - 2);
        let small = Url::parse("data:text/plain,12").unwrap();
        assert_eq!(
            f.fetch(&small, None, ResourceKind::Document).unwrap().bytes,
            b"12"
        );
        assert_eq!(f.total, MAX_PAGE_BYTES);
    }
    #[test]
    fn exhausted_page_deadline_stops_before_fetch() {
        let mut f = Fetcher {
            started: Instant::now() - Duration::from_secs(31),
            ..Fetcher::default()
        };
        assert!(
            f.fetch(
                &Url::parse("data:text/plain,ok").unwrap(),
                None,
                ResourceKind::Document
            )
            .is_err()
        );
    }
    #[test]
    fn policy_blocks_remote_files_and_mixed_content() {
        let f = Fetcher::default();
        let source = Url::parse("https://example.com/").unwrap();
        assert!(
            f.validate(
                &Url::parse("file:///etc/passwd").unwrap(),
                Some(&source),
                ResourceKind::Image
            )
            .is_err()
        );
        assert!(
            f.validate(
                &Url::parse("http://example.com/a.png").unwrap(),
                Some(&source),
                ResourceKind::Image
            )
            .is_err()
        );
        assert!(
            f.validate(
                &Url::parse("https://elsewhere.invalid/a.js").unwrap(),
                Some(&source),
                ResourceKind::Script
            )
            .is_err()
        );
    }
    #[test]
    fn bounded_reader_rejects_extra_byte() {
        assert!(read_bounded(&b"12345"[..], 4).is_err());
        assert_eq!(read_bounded(&b"1234"[..], 4).unwrap(), b"1234");
        assert_eq!(read_bounded(&b"ok"[..], usize::MAX).unwrap(), b"ok");
    }
    #[test]
    fn percent_encoded_runs_cannot_bypass_the_decoded_byte_limit() {
        assert_eq!(percent_decode_with_limit("%41%42%43", 3).unwrap(), b"ABC");
        assert!(percent_decode_with_limit("%41%42%43%44", 3).is_err());
        assert!(percent_decode_with_limit("abc%44", 3).is_err());
        assert_eq!(percent_decode_with_limit("", 0).unwrap(), b"");
    }
    #[test]
    fn internal_addresses_keep_fragments_queries_and_their_schemes() {
        for address in [
            "eris:home#section",
            "eris:home?theme=dark#section",
            "about:blank#target",
            "about:blank?x=1#target",
            "about:unsupported",
        ] {
            assert_eq!(parse_address(address).unwrap().as_str(), address);
        }
    }
    #[test]
    fn data_url_decoding() {
        let mut f = Fetcher::default();
        let r = f
            .fetch(
                &Url::parse("data:text/html,%3Ch1%3Ehi%3C%2Fh1%3E").unwrap(),
                None,
                ResourceKind::Document,
            )
            .unwrap();
        assert_eq!(r.text(), "<h1>hi</h1>");
        assert!(
            f.fetch(
                &Url::parse("javascript:alert(1)").unwrap(),
                None,
                ResourceKind::Document
            )
            .is_err()
        );
    }
    #[test]
    fn named_encoding() {
        let r = Resource {
            url: Url::parse("https://example.com").unwrap(),
            bytes: vec![0x63, 0x61, 0x66, 0xe9],
            content_type: "text/html; charset=windows-1252".into(),
            headers: BTreeMap::new(),
            status: 200,
            origin_clean: true,
            decoded_image: None,
        };
        assert_eq!(r.text(), "café");
    }
    #[test]
    fn byte_order_mark_precedes_transport_charset() {
        let r = Resource {
            url: Url::parse("https://example.com").unwrap(),
            bytes: b"\xef\xbb\xbfcaf\xc3\xa9".to_vec(),
            content_type: "text/html;charset=windows-1252".into(),
            headers: BTreeMap::new(),
            status: 200,
            origin_clean: true,
            decoded_image: None,
        };
        assert_eq!(r.text(), "café");
    }
    #[test]
    fn data_url_fragment_is_not_resource_content() {
        let r = decode_data_url(&Url::parse("data:text/plain,hello#section").unwrap()).unwrap();
        assert_eq!(r.text(), "hello");
    }
}
