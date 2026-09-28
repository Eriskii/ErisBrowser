//! Isolated page execution and a bounded, validated binary IPC boundary.
mod broker;
mod channel;
mod codec;
#[cfg(all(target_os = "linux", feature = "vulkan-presenter"))]
mod launcher;
#[cfg(all(target_os = "linux", feature = "vulkan-presenter"))]
pub use launcher::launch_worker;
mod image_decoder;
mod sandbox;
use crate::{
    dom::{Document, NodeId},
    graphics::{Fonts, ImageStore},
    layout::LayoutResult,
    net,
    page::{Navigation, Page, TaskState},
};
use std::{
    cell::Cell,
    io,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use url::Url;

pub struct Snapshot {
    pub generation: u64,
    pub processed_edit_sequence: u64,
    pub task_state: TaskState,
    pub layout: LayoutResult,
    pub images: ImageStore,
    pub document: Document,
    pub title: String,
    pub url: String,
    pub diagnostics: Vec<String>,
    pub load_ms: f64,
}
impl Snapshot {
    pub fn navigate_fragment(&mut self, target: Url) -> bool {
        let Ok(current) = Url::parse(&self.url) else {
            return false;
        };
        if !crate::page::same_document_url(&current, &target) {
            return false;
        }
        self.document.set_url(target.clone());
        self.url = target.into();
        true
    }
}
#[derive(Debug)]
pub enum Command {
    Load {
        navigation: Navigation,
    },
    Click {
        node: NodeId,
    },
    DefaultSummary {
        node: NodeId,
    },
    Edit {
        sequence: u64,
        node: NodeId,
        value: String,
    },
    Fragment {
        address: String,
    },
    Render {
        width: f32,
        height: f32,
    },
    RunTasks,
}
pub struct Reply {
    pub snapshot: Option<Snapshot>,
    pub navigation: Option<Navigation>,
}
impl Reply {
    fn empty() -> Self {
        Self {
            snapshot: None,
            navigation: None,
        }
    }
}
struct Init {
    scripts: bool,
    generation: u64,
}

pub struct WorkerClient {
    channel: channel::Channel,
    broker: Option<broker::BrokerClient>,
    executable: PathBuf,
    generation: u64,
    scripts: bool,
    root: Option<PathBuf>,
    initial: Url,
    authorized: Navigation,
    committed: Option<Url>,
    document_attempted: bool,
    document_failed: bool,
    requests: usize,
    loaded: bool,
    opaque_image_bytes: usize,
}
impl WorkerClient {
    pub fn spawn(scripts: bool, navigation: &Navigation, generation: u64) -> Result<Self, String> {
        Self::spawn_at(
            &std::env::current_exe().map_err(|e| e.to_string())?,
            scripts,
            navigation,
            generation,
        )
    }
    pub fn spawn_at(
        executable: &Path,
        scripts: bool,
        navigation: &Navigation,
        generation: u64,
    ) -> Result<Self, String> {
        let initial = net::parse_address(&navigation.address)?;
        validate_address(&initial)?;
        let root = if initial.scheme() == "file" {
            let path = initial
                .to_file_path()
                .map_err(|_| "invalid local document")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.is_file() {
                return Err("document must be a regular file".into());
            }
            Some(
                path.parent()
                    .ok_or("document has no parent directory")?
                    .to_owned(),
            )
        } else {
            None
        };
        let mut channel = channel::Channel::spawn(executable, "--page-worker")?;
        let bytes = channel.exchange(
            codec::encode_init(&Init {
                scripts,
                generation,
            })?,
            Duration::from_secs(5),
            || false,
            |_| Ok(None),
        )?;
        let reply = codec::decode_reply(&bytes)?;
        if reply.snapshot.is_some() || reply.navigation.is_some() {
            return Err("unexpected page worker startup reply".into());
        }
        let committed = matches!(initial.scheme(), "eris" | "about").then(|| initial.clone());
        let authorized = Navigation {
            address: initial.to_string(),
            form_body: navigation.form_body.clone(),
        };
        Ok(Self {
            channel,
            broker: None,
            executable: executable.to_owned(),
            generation,
            scripts,
            root,
            initial,
            authorized,
            committed,
            document_attempted: false,
            document_failed: false,
            requests: 0,
            loaded: false,
            opaque_image_bytes: 0,
        })
    }
    pub fn pid(&self) -> u32 {
        self.channel.pid()
    }
    pub fn broker_pid(&self) -> Option<u32> {
        self.broker.as_ref().map(broker::BrokerClient::pid)
    }
    pub fn exchange(
        &mut self,
        mut command: Command,
        cancel: impl Fn() -> bool,
    ) -> Result<Reply, String> {
        let loading = matches!(command, Command::Load { .. });
        let timeout = if let Command::Load { navigation } = &mut command {
            if self.loaded {
                return Err("each document load requires a new page worker".into());
            }
            let url = net::parse_address(&navigation.address)?;
            if url != self.initial || navigation.form_body != self.authorized.form_body {
                return Err("worker load differs from authorized document".into());
            }
            *navigation = self.authorized.clone();
            self.loaded = true;
            Duration::from_secs(40)
        } else {
            Duration::from_secs(15)
        };
        let started = Instant::now();
        let cancellation_observed = Cell::new(false);
        let cancelled = || {
            if cancel() || started.elapsed() > timeout {
                cancellation_observed.set(true);
            }
            cancellation_observed.get()
        };
        let encoded = codec::encode_command(&command)?;
        let Self {
            channel,
            broker,
            executable,
            root,
            initial,
            authorized,
            committed,
            document_attempted,
            document_failed,
            requests,
            opaque_image_bytes,
            ..
        } = self;
        let result = channel
            .exchange(encoded, timeout, cancelled, |bytes| {
                if !codec::is_fetch_request(bytes) {
                    return Ok(None);
                }
                *requests += 1;
                if *requests > net::MAX_RESOURCES {
                    return Err("renderer resource request budget exceeded".into());
                }
                let request = codec::decode_fetch_request(bytes)?;
                let document = request.kind == net::ResourceKind::Document;
                if document {
                    if !loading
                        || *document_attempted
                        || request.url != *initial
                        || request.form_body != authorized.form_body
                    {
                        return Err("unauthorized renderer document request".into());
                    }
                    *document_attempted = true;
                } else if committed.is_none() || request.form_body.is_some() {
                    return Err("resource request has no authorized document".into());
                }
                if broker.is_none() {
                    *broker = Some(broker::BrokerClient::spawn(
                        executable,
                        &broker::BrokerInit {
                            navigation: authorized.clone(),
                            root: root.clone(),
                        },
                        cancelled,
                    )?);
                }
                let mut response = broker
                    .as_mut()
                    .ok_or("resource broker unavailable")?
                    .fetch(&request, cancelled);
                if document {
                    match &response {
                        Ok(response) => {
                            validate_committed(initial, &response.url)?;
                            *committed = Some(response.url.clone());
                        }
                        Err(_) => *document_failed = true,
                    }
                }
                if request.kind == net::ResourceKind::Image {
                    response = response
                        .and_then(|resource| {
                            let source = committed
                                .as_ref()
                                .ok_or("image has no committed document")?;
                            let opaque = !resource.origin_clean
                                || !net::image_origin_clean(source, &request.url)
                                || !net::image_origin_clean(source, &resource.url);
                            if !opaque {
                                return Ok(resource);
                            }
                            let remaining = crate::page::MAX_DECODED_IMAGE_BYTES
                                .saturating_sub(*opaque_image_bytes);
                            let image =
                                image_decoder::decode(executable, &resource, remaining, cancelled)?;
                            *opaque_image_bytes += image.rgba.len();
                            // Only the requested address and pixels cross into the renderer.
                            // The final URL, raw body, response headers and status do not.
                            Ok(net::Resource {
                                url: request.url.clone(),
                                bytes: Vec::new(),
                                content_type: String::new(),
                                headers: Default::default(),
                                status: 200,
                                origin_clean: false,
                                decoded_image: Some(image),
                            })
                        })
                        .map_err(|_| "image loading or decoding failed".to_owned());
                }
                Ok(Some(codec::encode_resource(&response)?))
            })
            .and_then(|bytes| codec::decode_reply(&bytes));
        let mut reply = match result {
            Ok(reply) => reply,
            Err(error) => {
                self.channel.terminate();
                self.broker.take();
                return Err(error);
            }
        };
        let validation = (|| {
            if let Some(snapshot) = &mut reply.snapshot {
                validate_snapshot_metadata(snapshot, self.generation, self.scripts, &command)?;
                if !(self.document_failed && snapshot.url == "eris:error") {
                    let committed = self
                        .committed
                        .as_ref()
                        .ok_or("renderer has no committed document")?;
                    let mut expected = committed.clone();
                    expected.set_fragment(None);
                    let mut reported = Url::parse(&snapshot.url).map_err(|e| e.to_string())?;
                    reported.set_fragment(None);
                    if reported != expected {
                        return Err("renderer changed the broker-authorized document URL".into());
                    }
                }
                if let Some(pid) = self.broker_pid() {
                    snapshot.diagnostics.truncate(255);
                    snapshot.diagnostics.push(format!("Resource broker {pid}: committed URL and fetch policy enforced outside renderer"));
                }
            }
            if let Some(navigation) = &reply.navigation {
                if !matches!(command, Command::Click { .. }) {
                    return Err("unexpected page navigation".to_owned());
                }
                self.validate_destination(&navigation.address)?;
            }
            Ok(())
        })();
        if let Err(error) = validation {
            self.channel.terminate();
            self.broker.take();
            return Err(error);
        }
        Ok(reply)
    }
    fn validate_destination(&self, address: &str) -> Result<(), String> {
        let url = Url::parse(address).map_err(|e| e.to_string())?;
        validate_address(&url)?;
        if url.scheme() == "file" {
            let root = self
                .root
                .as_ref()
                .ok_or("remote worker requested a local file")?;
            let path = url
                .to_file_path()
                .map_err(|_| "invalid file URL")?
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if !path.starts_with(root) || !path.is_file() {
                return Err("worker navigation escaped its file scope".into());
            }
        }
        if self.committed.as_ref().unwrap_or(&self.initial).scheme() == "https"
            && url.scheme() == "http"
        {
            return Err("worker requested an HTTPS downgrade".into());
        }
        Ok(())
    }
}
fn validate_snapshot_metadata(
    snapshot: &Snapshot,
    generation: u64,
    scripts: bool,
    command: &Command,
) -> Result<(), String> {
    if snapshot.generation != generation || !matches!(command, Command::Render { .. }) {
        return Err("unexpected page snapshot".into());
    }
    if !scripts && snapshot.task_state != TaskState::Idle {
        return Err("page scheduled tasks without script authorization".into());
    }
    Ok(())
}
fn validate_committed(initial: &Url, committed: &Url) -> Result<(), String> {
    validate_address(committed)?;
    if matches!(initial.scheme(), "http" | "https") {
        if !matches!(committed.scheme(), "http" | "https")
            || initial.scheme() == "https" && committed.scheme() != "https"
        {
            return Err("broker returned an invalid document redirect".into());
        }
    } else if initial != committed {
        return Err("broker changed a non-network document URL".into());
    }
    Ok(())
}
pub fn serve_image_decoder() -> Result<(), String> {
    image_decoder::serve()
}

pub fn serve_resource_broker() -> Result<(), String> {
    broker::serve()
}
fn validate_address(url: &Url) -> Result<(), String> {
    if !url.username().is_empty() || url.password().is_some() {
        return Err("URL credentials are not allowed".into());
    }
    match url.scheme() {
        "http" | "https" | "file" | "data" => Ok(()),
        "about" if url.path() == "blank" => Ok(()),
        "eris" if url.path() == "home" => Ok(()),
        _ => Err("unsupported worker navigation scheme".into()),
    }
}

/// Internal entry point: this executes before any native UI initialization.
pub fn serve() -> Result<(), String> {
    let init = codec::decode_init(&codec::read_frame(
        &mut io::stdin().lock(),
        codec::MAX_REQUEST,
    )?)?;
    if let Err(error) =
        sandbox::check_inherited_descriptors().and_then(|()| sandbox::restrict_renderer())
    {
        codec::write_frame(&mut io::stdout().lock(), &codec::encode_error(&error)?)?;
        return Err(error);
    }
    net::use_page_fetch_bridge(broker::fetch_via_parent);
    codec::write_frame(
        &mut io::stdout().lock(),
        &codec::encode_reply(&Reply::empty())?,
    )?;
    let fonts = Fonts::new();
    let mut page: Option<Page> = None;
    let mut processed_edit_sequence = 0;
    loop {
        let bytes = codec::read_frame(&mut io::stdin().lock(), codec::MAX_REQUEST)?;
        let command = codec::decode_command(&bytes)?;
        let mut reply = Reply::empty();
        match command {
            Command::Load { navigation } => {
                if page.is_some() {
                    return Err("duplicate worker load".into());
                }
                page = Some(
                    Page::load_navigation(&navigation, init.scripts)
                        .unwrap_or_else(|error| Page::error(&navigation.address, &error)),
                );
            }
            Command::Click { node } => {
                reply.navigation = page.as_mut().ok_or("no page")?.click(node)
            }
            Command::DefaultSummary { node } => {
                let page = page.as_mut().ok_or("no page")?;
                if page.document.namespace(node) != Some(crate::dom::Namespace::Html)
                    || page.document.tag(node) != Some("details")
                    || page.document.first_summary(node).is_some()
                    || page.document.interaction_blocked(node)
                    || page.document.disclosure_hidden(node)
                {
                    return Err("invalid generated summary activation".into());
                }
                page.click_default_summary(node);
            }
            Command::Edit {
                sequence,
                node,
                value,
            } => {
                if sequence > processed_edit_sequence {
                    apply_edit(page.as_mut().ok_or("no page")?, node, &value);
                    processed_edit_sequence = sequence;
                }
            }
            Command::Fragment { address } => {
                let page = page.as_mut().ok_or("no page")?;
                if let Ok(target) = Url::parse(&address) {
                    page.navigate_fragment(target);
                }
            }
            Command::RunTasks => page.as_mut().ok_or("no page")?.run_pending_tasks(),
            Command::Render { width, height } => {
                if !width.is_finite()
                    || !height.is_finite()
                    || !(1.0..=8192.0).contains(&width)
                    || !(1.0..=8192.0).contains(&height)
                {
                    return Err("invalid viewport".into());
                }
                let page = page.as_mut().ok_or("no page")?;
                page.diagnostics.truncate(255);
                let mut diagnostics: Vec<String> = page
                    .diagnostics
                    .iter()
                    .map(|s| s.chars().take(2048).collect())
                    .collect();
                diagnostics.push(format!(
                    "Page process {}: Landlock ABI 6 and seccomp: no direct resource or socket access",
                    std::process::id()
                ));
                reply.snapshot = Some(Snapshot {
                    generation: init.generation,
                    processed_edit_sequence,
                    task_state: page.task_state(),
                    layout: page.layout(width, height, &fonts),
                    images: page.images.clone(),
                    document: page.document.clone(),
                    title: page.title().chars().take(512).collect(),
                    url: page.url.to_string(),
                    diagnostics,
                    load_ms: page.load_ms,
                });
            }
        }
        codec::write_frame(&mut io::stdout().lock(), &codec::encode_reply(&reply)?)?;
    }
}
pub fn apply_edit(page: &mut Page, node: NodeId, value: &str) {
    if value.len() > 65_536 || !page.can_edit_control(node) {
        return;
    }
    if page.document.tag(node) == Some("textarea") {
        page.document.set_text_content(node, value);
    } else {
        page.document.set_attr(node, "value", value);
    }
    if page.scripts_enabled
        && let Err(error) = page
            .runtime
            .dispatch_event(node, "input", &mut page.document)
    {
        page.diagnostics.push(format!("input: {error}"));
    }
    page.disclosure_checkpoint();
    if page.scripts_enabled {
        page.refresh_inline_svg();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forged_task_metadata_cannot_schedule_work_when_scripts_are_disabled() {
        let page = Page::from_html(
            Url::parse("about:blank").unwrap(),
            "<details open></details>",
            false,
        );
        let mut snapshot = Snapshot {
            generation: 11,
            processed_edit_sequence: 0,
            task_state: TaskState::Idle,
            layout: page.layout(320.0, 240.0, &Fonts::new()),
            images: page.images,
            document: page.document,
            title: String::new(),
            url: "about:blank".into(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        };
        let render = Command::Render {
            width: 320.0,
            height: 240.0,
        };
        assert!(validate_snapshot_metadata(&snapshot, 11, false, &render).is_ok());
        for forged in [TaskState::Pending, TaskState::Suspended] {
            snapshot.task_state = forged;
            assert!(
                validate_snapshot_metadata(&snapshot, 11, false, &render)
                    .unwrap_err()
                    .contains("without script authorization")
            );
            assert!(validate_snapshot_metadata(&snapshot, 11, true, &render).is_ok());
        }
        // Task execution returns an empty acknowledgement, never a snapshot.
        assert!(validate_snapshot_metadata(&snapshot, 11, true, &Command::RunTasks).is_err());
        assert!(validate_snapshot_metadata(&snapshot, 12, true, &render).is_err());
    }
    #[test]
    fn committed_urls_come_from_checked_network_redirects() {
        let initial = Url::parse("https://first.example/").unwrap();
        assert!(
            validate_committed(&initial, &Url::parse("https://second.example/new").unwrap())
                .is_ok()
        );
        for denied in [
            "http://first.example/",
            "file:///etc/passwd",
            "eris:home",
            "https://user:secret@example.com/",
        ] {
            assert!(validate_committed(&initial, &Url::parse(denied).unwrap()).is_err());
        }
        let initial = Url::parse("file:///tmp/authorized.html").unwrap();
        assert!(
            validate_committed(&initial, &Url::parse("file:///tmp/other.html").unwrap()).is_err()
        );
    }
}
