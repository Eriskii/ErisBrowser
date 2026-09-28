use crate::edit::Selection;
use eris::{
    dom::{Namespace, NodeId},
    graphics::{Canvas, Color, DrawCommand, Fonts, Rect},
    layout::HitAction,
    page::Navigation,
    worker::{Command, Snapshot, WorkerClient},
};
use std::{
    collections::VecDeque,
    num::NonZeroU32,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Instant,
};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::{Key, ModifiersState, NamedKey},
    window::{CursorIcon, Window, WindowId},
};

const TOOLBAR: f32 = 76.0;
const STATUS: f32 = 25.0;

/// Hit regions have already been clipped by layout. Propagating their visible
/// area to ancestors also covers inline links whose text owns the actual hits.
/// Do not clip to the viewport: a control below the fold remains focusable.
fn visible_layout_nodes(snapshot: &Snapshot) -> Vec<bool> {
    let mut visible = vec![false; snapshot.document.nodes.len()];
    let mut default_summaries = vec![false; snapshot.document.nodes.len()];
    for hit in &snapshot.layout.hit_regions {
        if hit.rect.width <= 0.0 || hit.rect.height <= 0.0 {
            continue;
        }
        if hit.action == HitAction::DefaultSummary {
            default_summaries[hit.node] = true;
        }
        let mut current = Some(hit.node);
        for _ in 0..eris::dom::MAX_DEPTH {
            let Some(node) = current else {
                break;
            };
            let Some(mark) = visible.get_mut(node) else {
                break;
            };
            if *mark {
                break;
            }
            *mark = true;
            current = snapshot.document.nodes[node].parent;
        }
    }
    for (node, mark) in visible.iter_mut().enumerate() {
        if snapshot.document.namespace(node) == Some(Namespace::Html)
            && snapshot.document.tag(node) == Some("details")
            && snapshot.document.first_summary(node).is_none()
        {
            *mark = default_summaries[node];
        }
    }
    visible
}
enum Event {
    Ready,
    Failed {
        generation: u64,
        error: String,
    },
    Navigate {
        generation: u64,
        navigation: Navigation,
    },
}
enum Request {
    Load {
        generation: u64,
        navigation: Navigation,
    },
    Resize {
        width: f32,
        height: f32,
    },
    Click {
        generation: u64,
        node: NodeId,
    },
    DefaultSummary {
        generation: u64,
        node: NodeId,
    },
    Edit {
        generation: u64,
        sequence: u64,
        node: NodeId,
        value: String,
    },
    Fragment {
        generation: u64,
        address: String,
    },
    Stop,
}

const MAX_PENDING_REQUESTS: usize = 64;

#[derive(Default)]
struct RequestState {
    pending: VecDeque<Request>,
    stopped: bool,
}

/// Keep UI bursts bounded while retaining the newest navigation and edits.
#[derive(Default)]
struct RequestQueue {
    state: Mutex<RequestState>,
    ready: Condvar,
}

impl RequestQueue {
    fn send(&self, request: Request) -> Result<(), ()> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if state.stopped {
            return Err(());
        }
        if matches!(request, Request::Stop) {
            state.pending.clear();
            state.stopped = true;
            self.ready.notify_one();
            return Ok(());
        }
        if matches!(request, Request::Load { .. }) {
            // Resizing precedes the initial Load; preserve its latest dimensions.
            let resize = state
                .pending
                .iter()
                .rev()
                .find_map(|pending| match pending {
                    Request::Resize { width, height } => Some((*width, *height)),
                    _ => None,
                });
            state.pending.clear();
            if let Some((width, height)) = resize {
                state.pending.push_back(Request::Resize { width, height });
            }
        } else {
            match (&request, state.pending.back_mut()) {
                (
                    Request::Resize { width, height },
                    Some(Request::Resize {
                        width: old_width,
                        height: old_height,
                    }),
                ) => {
                    *old_width = *width;
                    *old_height = *height;
                    return Ok(());
                }
                (
                    Request::Edit {
                        generation,
                        sequence,
                        node,
                        value,
                    },
                    Some(Request::Edit {
                        generation: old_generation,
                        sequence: old_sequence,
                        node: old_node,
                        value: old_value,
                    }),
                ) if generation == old_generation && node == old_node => {
                    if sequence >= old_sequence {
                        old_value.clone_from(value);
                        *old_sequence = *sequence;
                    }
                    return Ok(());
                }
                _ => {}
            }
        }
        if state.pending.len() >= MAX_PENDING_REQUESTS {
            let evict = state
                .pending
                .iter()
                .position(|pending| {
                    !matches!(pending, Request::Load { .. } | Request::Resize { .. })
                })
                .or_else(|| {
                    state
                        .pending
                        .iter()
                        .position(|pending| !matches!(pending, Request::Load { .. }))
                });
            if let Some(index) = evict {
                state.pending.remove(index);
            } else {
                return Err(());
            }
        }
        state.pending.push_back(request);
        self.ready.notify_one();
        Ok(())
    }

    fn recv(&self) -> Option<Request> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        loop {
            if let Some(request) = state.pending.pop_front() {
                return Some(request);
            }
            if state.stopped {
                return None;
            }
            state = self
                .ready
                .wait(state)
                .unwrap_or_else(|error| error.into_inner());
        }
    }

    fn has_pending(&self) -> bool {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state.stopped || !state.pending.is_empty()
    }

    fn cancelled(&self, generation: u64, current: &AtomicU64) -> bool {
        generation != current.load(Ordering::Relaxed)
            || self
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .stopped
    }
}

/// At most one full document snapshot can wait for the UI. The event loop
/// receives a small wake-up signal only when this slot changes from empty.
struct Latest<T> {
    value: Mutex<Option<T>>,
}

impl<T> Default for Latest<T> {
    fn default() -> Self {
        Self {
            value: Mutex::new(None),
        }
    }
}

impl<T> Latest<T> {
    fn publish(&self, value: T) -> bool {
        let previous = self
            .value
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .replace(value);
        previous.is_none()
    }

    fn take(&self) -> Option<T> {
        self.value
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take()
    }
}

/// The bridge owns IPC and the request queue; all page work runs in a child.
fn worker(
    proxy: EventLoopProxy<Event>,
    requests: Arc<RequestQueue>,
    ready_snapshot: Arc<Latest<Snapshot>>,
    current: Arc<AtomicU64>,
    scripts: bool,
) {
    let mut client: Option<WorkerClient> = None;
    let mut generation = 0;
    let mut edit_sequence = 0;
    let mut width = 1180.0;
    let mut height = 739.0;
    while let Some(request) = requests.recv() {
        let command = match request {
            Request::Stop => break,
            Request::Load {
                generation: id,
                navigation,
            } => {
                if requests.cancelled(id, &current) {
                    continue;
                }
                // A new document always gets a fresh sandbox and process.
                drop(client.take());
                generation = id;
                edit_sequence = 0;
                match WorkerClient::spawn(scripts, &navigation, generation) {
                    Ok(worker) => client = Some(worker),
                    Err(error) => {
                        if !requests.cancelled(generation, &current) {
                            let _ = proxy.send_event(Event::Failed { generation, error });
                        }
                        continue;
                    }
                }
                Some(Command::Load { navigation })
            }
            Request::Resize {
                width: w,
                height: h,
            } => {
                width = w;
                height = h;
                None
            }
            Request::Click {
                generation: id,
                node,
            } => {
                if id != generation {
                    continue;
                }
                Some(Command::Click { node })
            }
            Request::DefaultSummary {
                generation: id,
                node,
            } => {
                if id != generation {
                    continue;
                }
                Some(Command::DefaultSummary { node })
            }
            Request::Edit {
                generation: id,
                sequence,
                node,
                value,
            } => {
                if id != generation || sequence <= edit_sequence {
                    continue;
                }
                edit_sequence = sequence;
                Some(Command::Edit {
                    sequence,
                    node,
                    value,
                })
            }
            Request::Fragment {
                generation: id,
                address,
            } => {
                if id != generation {
                    continue;
                }
                Some(Command::Fragment { address })
            }
        };
        let Some(active) = client.as_mut() else {
            continue;
        };
        let result = (|| -> Result<Option<Snapshot>, String> {
            if let Some(command) = command {
                let reply =
                    active.exchange(command, || requests.cancelled(generation, &current))?;
                if !requests.cancelled(generation, &current)
                    && let Some(navigation) = reply.navigation
                {
                    let _ = proxy.send_event(Event::Navigate {
                        generation,
                        navigation,
                    });
                }
            }
            // Intermediate edits are acknowledged in the next rendered snapshot;
            // they do not create a full-document IPC backlog.
            if requests.cancelled(generation, &current) || requests.has_pending() {
                return Ok(None);
            }
            let reply = active.exchange(Command::Render { width, height }, || {
                requests.cancelled(generation, &current)
            })?;
            if let Some(navigation) = reply.navigation {
                let _ = proxy.send_event(Event::Navigate {
                    generation,
                    navigation,
                });
            }
            let snapshot = reply
                .snapshot
                .ok_or("Page process returned no rendered snapshot")?;
            if snapshot.generation != generation || snapshot.processed_edit_sequence > edit_sequence
            {
                return Err("Page process returned an inconsistent document generation or edit acknowledgement".into());
            }
            Ok(Some(snapshot))
        })();
        match result {
            Ok(Some(snapshot))
                if !requests.cancelled(generation, &current) && !requests.has_pending() =>
            {
                if ready_snapshot.publish(snapshot) && proxy.send_event(Event::Ready).is_err() {
                    break;
                }
            }
            Ok(_) => {
                if requests.cancelled(generation, &current) {
                    drop(client.take());
                }
            }
            Err(error) => {
                drop(client.take());
                if !requests.cancelled(generation, &current)
                    && proxy
                        .send_event(Event::Failed { generation, error })
                        .is_err()
                {
                    break;
                }
            }
        }
    }
    // WorkerClient::drop kills and reaps the child, including after Stop.
}

pub fn run(
    address: String,
    scripts: bool,
    exit_after: Option<f64>,
    capture: Option<PathBuf>,
) -> Result<(), String> {
    let event_loop = EventLoop::<Event>::with_user_event()
        .build()
        .map_err(|e| e.to_string())?;
    let tx = Arc::new(RequestQueue::default());
    let rx = tx.clone();
    let ready_snapshot = Arc::new(Latest::default());
    let worker_snapshot = ready_snapshot.clone();
    let current = Arc::new(AtomicU64::new(0));
    let proxy = event_loop.create_proxy();
    let generation = current.clone();
    let bridge = thread::Builder::new()
        .name("eris-page-ipc".into())
        .spawn(move || worker(proxy, rx, worker_snapshot, generation, scripts))
        .map_err(|e| e.to_string())?;
    let mut browser = Browser {
        window: None,
        surface: None,
        fonts: Fonts::new(),
        snapshot: None,
        visible_nodes: Vec::new(),
        ready_snapshot,
        tx,
        current,
        address: address.clone(),
        address_focused: false,
        selection: Selection::default(),
        focused: None,
        input_value: String::new(),
        edit_sequence: 0,
        history: Vec::new(),
        history_index: 0,
        scroll: 0.0,
        zoom: 1.0,
        cursor: (0.0, 0.0),
        modifiers: ModifiersState::default(),
        loading: false,
        status: String::new(),
        hover_clickable: false,
        initial: Some(address),
        started: Instant::now(),
        exit_after,
        capture,
        startup_error: None,
        worker_error: None,
        clipboard: None,
        applied_fragment_generation: 0,
    };
    let result = event_loop.run_app(&mut browser).map_err(|e| e.to_string());
    let _ = browser.tx.send(Request::Stop);
    let bridge_result = bridge
        .join()
        .map_err(|_| "Page IPC bridge stopped unexpectedly".to_owned());
    if let Some(error) = browser.startup_error {
        Err(error)
    } else if let Some(error) = browser.worker_error {
        Err(format!("Page process failed: {error}"))
    } else {
        result.and(bridge_result)
    }
}
struct Browser {
    window: Option<Arc<Window>>,
    surface: Option<softbuffer::Surface<Arc<Window>, Arc<Window>>>,
    fonts: Fonts,
    snapshot: Option<Snapshot>,
    visible_nodes: Vec<bool>,
    ready_snapshot: Arc<Latest<Snapshot>>,
    tx: Arc<RequestQueue>,
    current: Arc<AtomicU64>,
    address: String,
    address_focused: bool,
    selection: Selection,
    focused: Option<NodeId>,
    input_value: String,
    edit_sequence: u64,
    history: Vec<String>,
    history_index: usize,
    scroll: f32,
    zoom: f32,
    cursor: (f32, f32),
    modifiers: ModifiersState,
    loading: bool,
    status: String,
    hover_clickable: bool,
    initial: Option<String>,
    started: Instant,
    exit_after: Option<f64>,
    capture: Option<PathBuf>,
    startup_error: Option<String>,
    worker_error: Option<String>,
    clipboard: Option<arboard::Clipboard>,
    applied_fragment_generation: u64,
}
impl Browser {
    fn generation(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }
    fn redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
    fn viewport(&self) -> (f32, f32) {
        self.window
            .as_ref()
            .map(|w| {
                let s = w.inner_size();
                (
                    s.width as f32 / self.zoom,
                    (s.height as f32 - TOOLBAR - STATUS).max(1.0) / self.zoom,
                )
            })
            .unwrap_or((1180.0, 739.0))
    }
    fn resize(&self) {
        let (width, height) = self.viewport();
        let _ = self.tx.send(Request::Resize { width, height });
    }
    fn navigate(&mut self, address: String, record: bool) {
        self.navigate_impl(Navigation::get(address), record, false);
    }
    fn navigate_request(&mut self, navigation: Navigation, record: bool) {
        self.navigate_impl(navigation, record, true);
    }
    fn record_address(&mut self, address: &str) {
        if !self.history.is_empty() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(address.to_owned());
        self.history_index = self.history.len() - 1;
    }
    fn navigate_impl(&mut self, navigation: Navigation, record: bool, allow_fragment: bool) {
        let address = navigation.address.clone();
        if allow_fragment
            && !self.loading
            && navigation.form_body.is_none()
            && let (Some(snapshot), Ok(target)) = (&self.snapshot, url::Url::parse(&address))
            && snapshot.generation == self.generation()
            && let Ok(existing_url) = url::Url::parse(&snapshot.url)
        {
            let mut without = target.clone();
            without.set_fragment(None);
            let mut existing = existing_url.clone();
            existing.set_fragment(None);
            if existing == without
                && (target.fragment().is_some() || existing_url.fragment().is_some())
            {
                if record {
                    self.record_address(&address);
                }
                self.address = address.clone();
                self.address_focused = false;
                self.focused = None;
                self.selection.end(&self.address);
                if let Some(snapshot) = &mut self.snapshot {
                    snapshot.navigate_fragment(target.clone());
                }
                self.jump_to(target.fragment().unwrap_or(""));
                let _ = self.tx.send(Request::Fragment {
                    generation: self.generation(),
                    address,
                });
                return;
            }
        }
        if record {
            self.record_address(&address);
        }
        self.address = address.clone();
        self.selection.end(&self.address);
        self.address_focused = false;
        self.focused = None;
        self.scroll = 0.0;
        self.loading = true;
        self.worker_error = None;
        self.edit_sequence = 0;
        self.status = "Loading…".into();
        let generation = self.current.fetch_add(1, Ordering::Relaxed) + 1;
        let _ = self.tx.send(Request::Load {
            generation,
            navigation,
        });
        self.redraw();
    }
    fn back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.navigate_request(
                Navigation::get(self.history[self.history_index].clone()),
                false,
            );
        }
    }
    fn forward(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.navigate_request(
                Navigation::get(self.history[self.history_index].clone()),
                false,
            );
        }
    }
    fn jump_to(&mut self, fragment: &str) {
        if fragment.is_empty() {
            self.scroll = 0.0;
            self.redraw();
            return;
        }
        if let Some(s) = &self.snapshot
            && let Some(id) = eris::page::find_fragment(&s.document, fragment)
            && let Some(hit) = s.layout.hit_regions.iter().find(|r| r.node == id)
            && !hit.fixed
        {
            self.scroll = hit.rect.y * self.zoom;
            self.clamp_scroll();
            self.redraw();
        }
    }
    fn clamp_scroll(&mut self) {
        let h = self.viewport().1 * self.zoom;
        let max = self
            .snapshot
            .as_ref()
            .map(|s| (s.layout.content_height * self.zoom - h).max(0.0))
            .unwrap_or(0.0);
        self.scroll = self.scroll.clamp(0.0, max);
    }
    fn hit(&self) -> Option<NodeId> {
        self.hit_region().map(|hit| hit.node)
    }
    fn hit_region(&self) -> Option<&eris::layout::HitRegion> {
        let (x, y) = self.cursor;
        if y < TOOLBAR
            || self
                .window
                .as_ref()
                .is_some_and(|w| y >= w.inner_size().height as f32 - STATUS)
        {
            return None;
        }
        let snapshot = self.snapshot.as_ref()?;
        if snapshot.generation != self.generation() {
            return None;
        }
        snapshot.layout.hit_regions.iter().rev().find(|h| {
            h.rect.contains(
                x / self.zoom,
                (y - TOOLBAR + if h.fixed { 0.0 } else { self.scroll }) / self.zoom,
            )
        })
    }
    fn ancestor_with_tag(&self, mut id: NodeId, tags: &[&str]) -> Option<NodeId> {
        let snapshot = self
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.generation == self.generation())?;
        let doc = &snapshot.document;
        if doc.interaction_blocked(id) {
            return None;
        }
        for _ in 0..eris::dom::MAX_DEPTH {
            if doc.namespace(id) == Some(Namespace::Html)
                && doc.tag(id).is_some_and(|tag| tags.contains(&tag))
            {
                return Some(id);
            }
            id = doc.nodes.get(id)?.parent?;
        }
        None
    }
    fn focus_input(&mut self, node: NodeId) {
        let Some(s) = self
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.generation == self.generation())
        else {
            return;
        };
        if !self.visible_nodes.get(node).copied().unwrap_or(false)
            || !s.document.can_focus_control(node)
        {
            return;
        }
        self.focused = Some(node);
        self.address_focused = false;
        self.input_value = if s.document.tag(node) == Some("textarea") {
            s.document.text_content(node)
        } else {
            s.document.attr(node, "value").unwrap_or("").into()
        };
        self.selection.end(&self.input_value);
        if let Some(w) = &self.window {
            w.set_ime_allowed(s.document.can_edit_control(node));
        }
        self.redraw();
    }
    fn click(&mut self) {
        let (x, y) = self.cursor;
        if y < TOOLBAR {
            self.focused = None;
            match x {
                x if x < 48.0 => self.back(),
                x if x < 88.0 => self.forward(),
                x if x < 130.0 => self.navigate(self.address.clone(), false),
                x if x < 175.0 => self.navigate("eris:home".into(), true),
                _ => {
                    self.address_focused = true;
                    self.selection.all(&self.address);
                    if let Some(w) = &self.window {
                        w.set_ime_allowed(true);
                    }
                    self.redraw();
                }
            }
            return;
        }
        self.address_focused = false;
        self.focused = None;
        if let Some((node, action)) = self.hit_region().map(|hit| (hit.node, hit.action)) {
            let focus = if action == HitAction::DefaultSummary {
                Some(node)
            } else {
                self.ancestor_with_tag(node, &["summary", "input", "textarea", "button", "a"])
            };
            if let Some(focus) = focus {
                self.focus_input(focus);
            }
            if let Some(input) = self.ancestor_with_tag(node, &["input", "textarea"]) {
                let kind = self
                    .snapshot
                    .as_ref()
                    .and_then(|s| s.document.attr(input, "type"))
                    .unwrap_or("text")
                    .to_ascii_lowercase();
                if !matches!(
                    kind.as_str(),
                    "checkbox" | "radio" | "submit" | "button" | "reset" | "hidden" | "file"
                ) {
                    self.focus_input(input);
                }
            }
            let request = if action == HitAction::DefaultSummary {
                Request::DefaultSummary {
                    generation: self.generation(),
                    node,
                }
            } else {
                Request::Click {
                    generation: self.generation(),
                    node,
                }
            };
            let _ = self.tx.send(request);
        }
        self.redraw();
    }
    fn insert_text(&mut self, text: &str) {
        if self.address_focused {
            let text: String = text.chars().filter(|c| !c.is_control()).collect();
            self.selection.replace(&mut self.address, &text, 8192);
        } else if let Some(node) = self.focused {
            if !self.editable(node) {
                return;
            }
            let multiline = self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.document.tag(node) == Some("textarea"));
            let text: String = text
                .chars()
                .filter(|c| !c.is_control() || (multiline && *c == '\n'))
                .collect();
            if self.selection.replace(&mut self.input_value, &text, 65_536) {
                self.send_edit(node);
            }
        }
        self.redraw();
    }
    fn send_edit(&mut self, node: NodeId) {
        if !self.editable(node) || self.input_value.len() > 65_536 {
            return;
        }
        let Some(sequence) = self.edit_sequence.checked_add(1) else {
            return;
        };
        if self
            .tx
            .send(Request::Edit {
                generation: self.generation(),
                sequence,
                node,
                value: self.input_value.clone(),
            })
            .is_ok()
        {
            self.edit_sequence = sequence;
        }
    }
    fn erase_text(&mut self, backward: bool) {
        if self.address_focused {
            self.selection.erase(&mut self.address, backward);
        } else if let Some(node) = self.focused {
            if !self.editable(node) {
                return;
            }
            if self.selection.erase(&mut self.input_value, backward) {
                self.send_edit(node);
            }
        }
        self.redraw();
    }
    fn has_text_focus(&self) -> bool {
        self.address_focused || self.focused.is_some_and(|n| self.editable(n))
    }
    fn move_text_cursor(&mut self, right: bool, edge: bool) {
        let text = if self.address_focused {
            &self.address
        } else {
            &self.input_value
        };
        if edge {
            self.selection.edge(text, right, self.modifiers.shift_key());
        } else {
            self.selection
                .horizontal(text, right, self.modifiers.shift_key());
        }
        self.redraw();
    }
    fn editable(&self, node: NodeId) -> bool {
        self.snapshot
            .as_ref()
            .filter(|snapshot| snapshot.generation == self.generation())
            .is_some_and(|s| {
                self.visible_nodes.get(node).copied().unwrap_or(false)
                    && s.document.can_edit_control(node)
            })
    }
    fn activate_focused(&self, node: NodeId) {
        let fallback = self.snapshot.as_ref().is_some_and(|snapshot| {
            snapshot.generation == self.generation()
                && snapshot.layout.hit_regions.iter().any(|hit| {
                    hit.node == node
                        && hit.action == HitAction::DefaultSummary
                        && hit.rect.width > 0.0
                        && hit.rect.height > 0.0
                })
        });
        let request = if fallback {
            Request::DefaultSummary {
                generation: self.generation(),
                node,
            }
        } else {
            Request::Click {
                generation: self.generation(),
                node,
            }
        };
        let _ = self.tx.send(request);
    }
    fn password_focused(&self) -> bool {
        !self.address_focused
            && self.focused.is_some_and(|node| {
                self.snapshot.as_ref().is_some_and(|snapshot| {
                    snapshot.generation == self.generation()
                        && snapshot.document.namespace(node) == Some(Namespace::Html)
                        && snapshot.document.tag(node) == Some("input")
                        && snapshot.document.can_focus_control(node)
                        && snapshot
                            .document
                            .attr(node, "type")
                            .is_some_and(|kind| kind.eq_ignore_ascii_case("password"))
                })
            })
    }
    fn clear_stale_focus(&mut self, incoming_generation: u64) {
        if self
            .snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot.generation != incoming_generation)
        {
            self.focused = None;
            self.input_value.clear();
            self.edit_sequence = 0;
            if !self.address_focused {
                self.selection = Selection::default();
            }
        }
    }
    fn reconcile_input(&mut self, snapshot: &Snapshot) {
        if self.address_focused || snapshot.generation != self.generation() {
            return;
        }
        let Some(node) = self.focused else {
            return;
        };
        if !self.visible_nodes.get(node).copied().unwrap_or(false)
            || !snapshot.document.can_focus_control(node)
            || self.editable(node) && !snapshot.document.can_edit_control(node)
        {
            self.focused = None;
            self.input_value.clear();
            self.selection = Selection::default();
            if let Some(window) = &self.window {
                window.set_ime_allowed(false);
            }
            return;
        }
        if snapshot.processed_edit_sequence < self.edit_sequence {
            return;
        }
        let value = match snapshot.document.tag(node) {
            Some("textarea") => snapshot.document.text_content(node),
            Some("input") => snapshot
                .document
                .attr(node, "value")
                .unwrap_or("")
                .to_owned(),
            _ => return,
        };
        if value != self.input_value {
            let at_end =
                self.selection.collapsed() && self.selection.caret == self.input_value.len();
            self.input_value = value;
            if at_end {
                self.selection.end(&self.input_value);
            } else {
                self.selection.normalize(&self.input_value);
            }
        }
    }
    fn accept_snapshot(&mut self, mut snapshot: Snapshot) {
        if self.worker_error.is_some()
            || snapshot.generation != self.generation()
            || self.snapshot.as_ref().is_some_and(|old| {
                old.generation == snapshot.generation
                    && old.processed_edit_sequence > snapshot.processed_edit_sequence
            })
        {
            return;
        }
        // A fragment navigation is immediate in the UI. A worker snapshot made
        // before that request must not restore the previous fragment.
        if let Some(old) = &self.snapshot
            && old.generation == snapshot.generation
        {
            snapshot.url.clone_from(&old.url);
        }
        let first_snapshot = self.applied_fragment_generation != snapshot.generation;
        let loaded_fragment = if first_snapshot {
            self.applied_fragment_generation = snapshot.generation;
            url::Url::parse(&snapshot.url)
                .ok()
                .and_then(|url| url.fragment().map(str::to_owned))
        } else {
            None
        };
        self.loading = false;
        self.status.clear();
        if let Some(window) = &self.window {
            window.set_title(&format!("{} — Eris", snapshot.title));
        }
        if !self.address_focused && snapshot.url != "eris:error" {
            self.address = snapshot.url.clone();
        }
        for diagnostic in &snapshot.diagnostics {
            eprintln!("[page] {diagnostic}");
        }
        self.clear_stale_focus(snapshot.generation);
        self.visible_nodes = visible_layout_nodes(&snapshot);
        self.reconcile_input(&snapshot);
        self.snapshot = Some(snapshot);
        self.clamp_scroll();
        if let Some(fragment) = loaded_fragment {
            self.jump_to(&fragment);
        }
        self.redraw();
    }

    fn accept_failure(&mut self, generation: u64, error: String) {
        if generation != self.generation() {
            return;
        }
        self.loading = false;
        self.snapshot = None;
        self.visible_nodes.clear();
        self.focused = None;
        self.input_value.clear();
        self.scroll = 0.0;
        self.hover_clickable = false;
        if !self.address_focused {
            self.selection = Selection::default();
        }
        self.worker_error = Some(
            error
                .chars()
                .map(|c| if c.is_control() { ' ' } else { c })
                .take(1024)
                .collect(),
        );
        self.status = "Page process stopped".into();
        if let Some(window) = &self.window {
            window.set_title("Page process stopped — Eris");
            window.set_cursor(CursorIcon::Default);
        }
        self.redraw();
    }
    fn clipboard_action(&mut self, action: &str) {
        if !self.has_text_focus() {
            return;
        }
        if action != "v" && self.password_focused() {
            return;
        }
        if self.clipboard.is_none() {
            match arboard::Clipboard::new() {
                Ok(clipboard) => self.clipboard = Some(clipboard),
                Err(error) => {
                    self.status = format!("Clipboard unavailable: {error}");
                    self.redraw();
                    return;
                }
            }
        }
        if action == "v" {
            match self.clipboard.as_mut().unwrap().get_text() {
                Ok(text) if text.len() <= 65_536 => self.insert_text(&text),
                Ok(_) => self.status = "Clipboard text exceeds field limit".into(),
                Err(error) => self.status = format!("Paste failed: {error}"),
            }
        } else {
            let text = if self.address_focused {
                &self.address
            } else {
                &self.input_value
            };
            self.selection.normalize(text);
            if self.selection.collapsed() {
                return;
            }
            let selected = text[self.selection.range()].to_owned();
            match self.clipboard.as_mut().unwrap().set_text(selected) {
                Ok(()) if action == "x" => self.erase_text(true),
                Ok(()) => {}
                Err(error) => self.status = format!("Copy failed: {error}"),
            }
        }
        self.redraw();
    }
    fn key(&mut self, key: Key, text: Option<&str>) {
        let ctrl = self.modifiers.control_key() || self.modifiers.super_key();
        if ctrl && let Key::Character(ref ch) = key {
            match ch.to_lowercase().as_str() {
                "l" => {
                    self.address_focused = true;
                    self.selection.all(&self.address);
                    self.focused = None;
                    self.redraw();
                    return;
                }
                "r" => {
                    self.navigate(self.address.clone(), false);
                    return;
                }
                action @ ("c" | "v" | "x") => {
                    self.clipboard_action(action);
                    return;
                }
                "a" if self.address_focused || self.focused.is_some() => {
                    if self.address_focused {
                        self.selection.all(&self.address);
                    } else {
                        self.selection.all(&self.input_value);
                    }
                    self.redraw();
                    return;
                }
                "+" | "=" => {
                    self.zoom = (self.zoom + 0.1).min(3.0);
                    self.resize();
                    return;
                }
                "-" => {
                    self.zoom = (self.zoom - 0.1).max(0.5);
                    self.resize();
                    return;
                }
                "0" => {
                    self.zoom = 1.0;
                    self.resize();
                    return;
                }
                _ => {}
            }
        }
        if self.modifiers.alt_key() {
            match key {
                Key::Named(NamedKey::ArrowLeft) => {
                    self.back();
                    return;
                }
                Key::Named(NamedKey::ArrowRight) => {
                    self.forward();
                    return;
                }
                _ => {}
            }
        }
        match key {
            Key::Named(NamedKey::Enter) if self.address_focused => {
                self.navigate(self.address.clone(), true)
            }
            Key::Named(NamedKey::Enter) if self.focused.is_some() => {
                let node = self.focused.unwrap();
                if !self.editable(node) {
                    self.activate_focused(node);
                    return;
                }
                if self
                    .snapshot
                    .as_ref()
                    .is_some_and(|s| s.document.tag(node) == Some("textarea"))
                {
                    self.insert_text("\n");
                } else if let Some(form) = self.ancestor_with_tag(node, &["form"]) {
                    let _ = self.tx.send(Request::Click {
                        generation: self.generation(),
                        node: form,
                    });
                }
            }
            Key::Named(NamedKey::Backspace) if self.has_text_focus() => self.erase_text(true),
            Key::Named(NamedKey::Delete) if self.has_text_focus() => self.erase_text(false),
            Key::Named(NamedKey::ArrowLeft) if self.has_text_focus() => {
                self.move_text_cursor(false, false)
            }
            Key::Named(NamedKey::ArrowRight) if self.has_text_focus() => {
                self.move_text_cursor(true, false)
            }
            Key::Named(NamedKey::Home) if self.has_text_focus() => {
                self.move_text_cursor(false, true)
            }
            Key::Named(NamedKey::End) if self.has_text_focus() => self.move_text_cursor(true, true),
            Key::Named(NamedKey::Space) => {
                if self.has_text_focus() {
                    self.insert_text(" ");
                } else if let Some(node) = self.focused {
                    self.activate_focused(node);
                } else {
                    self.scroll += self.viewport().1
                        * self.zoom
                        * 0.85
                        * if self.modifiers.shift_key() {
                            -1.0
                        } else {
                            1.0
                        };
                    self.clamp_scroll();
                    self.redraw();
                }
            }
            Key::Named(NamedKey::Escape) => {
                self.address_focused = false;
                self.focused = None;
                self.selection = Selection::default();
                self.redraw();
            }
            Key::Named(NamedKey::F5) => self.navigate(self.address.clone(), false),
            Key::Named(NamedKey::ArrowDown) if self.focused.is_none() => {
                self.scroll += 42.0;
                self.clamp_scroll();
                self.redraw();
            }
            Key::Named(NamedKey::ArrowUp) if self.focused.is_none() => {
                self.scroll -= 42.0;
                self.clamp_scroll();
                self.redraw();
            }
            Key::Named(NamedKey::PageDown) => {
                self.scroll += self.viewport().1 * self.zoom * 0.85;
                self.clamp_scroll();
                self.redraw();
            }
            Key::Named(NamedKey::PageUp) => {
                self.scroll -= self.viewport().1 * self.zoom * 0.85;
                self.clamp_scroll();
                self.redraw();
            }
            Key::Named(NamedKey::Home) if !self.address_focused && self.focused.is_none() => {
                self.scroll = 0.0;
                self.redraw();
            }
            Key::Named(NamedKey::End) if !self.address_focused && self.focused.is_none() => {
                self.scroll = f32::MAX;
                self.clamp_scroll();
                self.redraw();
            }
            Key::Named(NamedKey::Tab) => self.tab_focus(),
            Key::Character(_) if !ctrl && !self.modifiers.alt_key() => {
                if let Some(text) = text {
                    self.insert_text(text);
                }
            }
            _ => {}
        }
    }
    fn tab_focus(&mut self) {
        let Some(s) = self
            .snapshot
            .as_ref()
            .filter(|snapshot| snapshot.generation == self.generation())
        else {
            return;
        };
        let inputs = s
            .document
            .query_selector_all("input, textarea, button, a[href], summary, details")
            .into_iter()
            .filter(|&n| {
                self.visible_nodes.get(n).copied().unwrap_or(false)
                    && s.document.can_focus_control(n)
            })
            .collect::<Vec<_>>();
        if inputs.is_empty() {
            return;
        }
        let position = self
            .focused
            .and_then(|id| inputs.iter().position(|n| *n == id));
        let next = if self.modifiers.shift_key() {
            position
                .map(|p| (p + inputs.len() - 1) % inputs.len())
                .unwrap_or(inputs.len() - 1)
        } else {
            position.map(|p| (p + 1) % inputs.len()).unwrap_or(0)
        };
        self.focus_input(inputs[next]);
    }
    fn draw(&mut self) -> Result<(), String> {
        let Some(window) = &self.window else {
            return Ok(());
        };
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Ok(());
        }
        let mut canvas = Canvas::new(size.width, size.height)?;
        canvas.clear(Color::WHITE);
        let viewport = Rect {
            x: 0.0,
            y: TOOLBAR,
            width: size.width as f32,
            height: (size.height as f32 - TOOLBAR - STATUS).max(0.0),
        };
        canvas.set_clip(viewport);
        if let Some(snapshot) = &self.snapshot {
            if (self.zoom - 1.0).abs() < 0.001 {
                canvas.paint_with_viewport(
                    &snapshot.layout.commands,
                    &self.fonts,
                    &snapshot.images,
                    (0.0, TOOLBAR - self.scroll),
                    (0.0, TOOLBAR),
                );
            } else {
                let commands = snapshot
                    .layout
                    .commands
                    .iter()
                    .map(|c| scaled_command(c, self.zoom))
                    .collect::<Vec<_>>();
                canvas.paint_with_viewport(
                    &commands,
                    &self.fonts,
                    &snapshot.images,
                    (0.0, TOOLBAR - self.scroll),
                    (0.0, TOOLBAR),
                );
            }
            if let Some(node) = self.focused
                && let Some(hit) = snapshot.layout.hit_regions.iter().find(|h| h.node == node)
            {
                let r = Rect {
                    x: hit.rect.x * self.zoom,
                    y: hit.rect.y * self.zoom + TOOLBAR - if hit.fixed { 0.0 } else { self.scroll },
                    width: hit.rect.width * self.zoom,
                    height: hit.rect.height * self.zoom,
                };
                let blue = Color::rgb(67, 134, 231);
                canvas.rect(Rect { height: 2.0, ..r }, blue, 0.0);
                canvas.rect(
                    Rect {
                        y: r.y + r.height - 2.0,
                        height: 2.0,
                        ..r
                    },
                    blue,
                    0.0,
                );
                canvas.rect(Rect { width: 2.0, ..r }, blue, 0.0);
                canvas.rect(
                    Rect {
                        x: r.x + r.width - 2.0,
                        width: 2.0,
                        ..r
                    },
                    blue,
                    0.0,
                );
            }
            let total = snapshot.layout.content_height * self.zoom;
            if total > viewport.height {
                let h = (viewport.height * viewport.height / total).max(30.0);
                let y = TOOLBAR
                    + (viewport.height - h) * self.scroll / (total - viewport.height).max(1.0);
                canvas.rect(
                    Rect {
                        x: size.width as f32 - 7.0,
                        y,
                        width: 5.0,
                        height: h,
                    },
                    Color::rgba(123, 133, 154, 180),
                    3.0,
                );
            }
        }
        if let Some(error) = &self.worker_error {
            paint_process_error(&mut canvas, &self.fonts, viewport, error);
        }
        canvas.set_clip(Rect {
            x: 0.0,
            y: 0.0,
            width: size.width as f32,
            height: size.height as f32,
        });
        canvas.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                width: size.width as f32,
                height: TOOLBAR,
            },
            Color::rgb(22, 26, 36),
            0.0,
        );
        canvas.text(
            &self.fonts,
            17.0,
            9.0,
            "ERIS",
            11.0,
            Color::rgb(241, 191, 101),
            true,
            false,
            false,
        );
        let title = self
            .snapshot
            .as_ref()
            .map(|s| s.title.as_str())
            .unwrap_or("A browser of its own");
        canvas.text(
            &self.fonts,
            66.0,
            8.0,
            title,
            12.0,
            Color::rgb(167, 180, 203),
            false,
            false,
            false,
        );
        for (x, label) in [(20.0, "←"), (60.0, "→"), (101.0, "↻"), (143.0, "⌂")] {
            canvas.text(
                &self.fonts,
                x,
                35.0,
                label,
                23.0,
                Color::rgb(209, 216, 232),
                false,
                false,
                false,
            );
        }
        let bar = Rect {
            x: 181.0,
            y: 31.0,
            width: (size.width as f32 - 196.0).max(20.0),
            height: 34.0,
        };
        canvas.rect(
            bar,
            if self.address_focused {
                Color::rgb(57, 65, 87)
            } else {
                Color::rgb(38, 45, 61)
            },
            7.0,
        );
        canvas.set_clip(Rect {
            x: bar.x + 9.0,
            y: bar.y,
            width: bar.width - 18.0,
            height: bar.height,
        });
        self.selection.normalize(if self.address_focused {
            &self.address
        } else {
            &self.input_value
        });
        let caret_width = if self.address_focused {
            self.fonts.measure(
                &self.address[..self.selection.caret],
                14.0,
                false,
                false,
                false,
            )
        } else {
            0.0
        };
        let offset = if self.address_focused {
            (caret_width - (bar.width - 28.0)).max(0.0)
        } else {
            0.0
        };
        let text_x = bar.x + 11.0 - offset;
        if self.address_focused && !self.selection.collapsed() {
            let selected = self.selection.range();
            let before =
                self.fonts
                    .measure(&self.address[..selected.start], 14.0, false, false, false);
            let selected_width =
                self.fonts
                    .measure(&self.address[selected], 14.0, false, false, false);
            canvas.rect(
                Rect {
                    x: text_x + before,
                    y: bar.y + 7.0,
                    width: selected_width,
                    height: 20.0,
                },
                Color::rgb(55, 93, 155),
                0.0,
            );
        }
        canvas.text(
            &self.fonts,
            text_x,
            bar.y + 7.0,
            &self.address,
            14.0,
            Color::rgb(223, 230, 242),
            false,
            false,
            false,
        );
        if self.address_focused && self.selection.collapsed() {
            canvas.rect(
                Rect {
                    x: text_x + caret_width,
                    y: bar.y + 8.0,
                    width: 1.0,
                    height: 18.0,
                },
                Color::WHITE,
                0.0,
            );
        }
        canvas.set_clip(Rect {
            x: 0.0,
            y: 0.0,
            width: size.width as f32,
            height: size.height as f32,
        });
        let status_y = size.height as f32 - STATUS;
        canvas.rect(
            Rect {
                x: 0.0,
                y: status_y,
                width: size.width as f32,
                height: STATUS,
            },
            Color::rgb(22, 26, 36),
            0.0,
        );
        let status = if self.loading {
            "Loading…".to_owned()
        } else if self.worker_error.is_some() {
            "Page process stopped".to_owned()
        } else if canvas.exhausted() {
            "Page painting limited: rendering work budget exceeded".to_owned()
        } else if !self.status.is_empty() {
            self.status.clone()
        } else if let Some(s) = &self.snapshot {
            format!(
                "{} nodes · {} draw commands · {:.1} ms load · {} notices · {:.0}%",
                s.document.nodes.len(),
                s.layout.commands.len(),
                s.load_ms,
                s.diagnostics.len(),
                self.zoom * 100.0
            )
        } else {
            "Ready".into()
        };
        canvas.text(
            &self.fonts,
            12.0,
            status_y + 5.0,
            &status,
            11.0,
            Color::rgb(157, 173, 197),
            false,
            false,
            false,
        );
        if (self
            .snapshot
            .as_ref()
            .is_some_and(|s| s.generation == self.generation())
            || self.worker_error.is_some())
            && !self.loading
            && let Some(path) = self.capture.take()
        {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            canvas.save(&path)?;
        }
        if let Some(surface) = &mut self.surface {
            surface
                .resize(
                    NonZeroU32::new(size.width).unwrap(),
                    NonZeroU32::new(size.height).unwrap(),
                )
                .map_err(|e| e.to_string())?;
            let mut buffer = surface.buffer_mut().map_err(|e| e.to_string())?;
            buffer.copy_from_slice(&canvas.pixels);
            buffer.present().map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
impl ApplicationHandler<Event> for Browser {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Eris Browser")
            .with_inner_size(LogicalSize::new(1180.0, 880.0))
            .with_min_inner_size(LogicalSize::new(400.0, 250.0));
        let result = (|| -> Result<(), String> {
            let window = Arc::new(
                event_loop
                    .create_window(attributes)
                    .map_err(|e| e.to_string())?,
            );
            let context = softbuffer::Context::new(window.clone()).map_err(|e| e.to_string())?;
            let surface =
                softbuffer::Surface::new(&context, window.clone()).map_err(|e| e.to_string())?;
            window.set_ime_allowed(true);
            self.surface = Some(surface);
            self.window = Some(window);
            Ok(())
        })();
        if let Err(e) = result {
            eprintln!("Unable to open browser window: {e}");
            self.startup_error = Some(e);
            event_loop.exit();
            return;
        }
        self.resize();
        if let Some(address) = self.initial.take() {
            self.navigate(address, true);
        }
    }
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Event) {
        match event {
            Event::Ready => {
                let Some(snapshot) = self.ready_snapshot.take() else {
                    return;
                };
                self.accept_snapshot(snapshot);
            }
            Event::Failed { generation, error } => self.accept_failure(generation, error),
            Event::Navigate {
                generation,
                navigation,
            } if generation == self.generation() => self.navigate_request(navigation, true),
            _ => {}
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                self.resize();
                self.redraw();
            }
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.draw() {
                    eprintln!("paint: {error}");
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers.state(),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                self.key(event.logical_key, event.text.as_deref())
            }
            WindowEvent::Ime(Ime::Commit(text)) => self.insert_text(&text),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => self.click(),
            WindowEvent::MouseWheel { delta, .. } => {
                self.scroll -= match delta {
                    MouseScrollDelta::LineDelta(_, dy) => dy * 46.0,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                self.clamp_scroll();
                self.redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as f32, position.y as f32);
                let node = self.hit();
                let link = node.and_then(|n| self.ancestor_with_tag(n, &["a"]));
                let status = link
                    .and_then(|n| {
                        self.snapshot
                            .as_ref()?
                            .document
                            .attr(n, "href")
                            .map(|href| href.chars().take(8192).collect::<String>())
                    })
                    .unwrap_or_default();
                let clickable = link.is_some()
                    || node
                        .and_then(|n| self.ancestor_with_tag(n, &["button", "summary"]))
                        .is_some()
                    || self
                        .hit_region()
                        .is_some_and(|hit| hit.action == HitAction::DefaultSummary);
                if clickable != self.hover_clickable {
                    self.hover_clickable = clickable;
                    if let Some(w) = &self.window {
                        w.set_cursor(if clickable {
                            CursorIcon::Pointer
                        } else {
                            CursorIcon::Default
                        });
                    }
                }
                if status != self.status {
                    self.status = status;
                    self.redraw();
                }
            }
            _ => {}
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(seconds) = self.exit_after {
            if self.started.elapsed().as_secs_f64() >= seconds {
                event_loop.exit();
            } else {
                event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                    Instant::now() + std::time::Duration::from_millis(100),
                ));
            }
        }
    }
}
/// A process error is trusted browser chrome. No HTML parsing, scripting, or
/// page layout is performed in the native UI to present it.
fn paint_process_error(canvas: &mut Canvas, fonts: &Fonts, viewport: Rect, error: &str) {
    let x = viewport.x + 28.0;
    let mut y = viewport.y + 32.0;
    canvas.text(
        fonts,
        x,
        y,
        "Page process stopped",
        24.0,
        Color::rgb(160, 48, 43),
        true,
        false,
        false,
    );
    y += 42.0;
    let width = (viewport.width - 56.0).max(1.0);
    let mut line = String::new();
    for word in error.split_whitespace().take(128) {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty() && fonts.measure(&candidate, 15.0, false, false, false) > width {
            canvas.text(
                fonts,
                x,
                y,
                &line,
                15.0,
                Color::rgb(48, 52, 60),
                false,
                false,
                false,
            );
            y += 21.0;
            line = word.to_owned();
        } else {
            line = candidate;
        }
        if y > viewport.y + viewport.height - 50.0 {
            break;
        }
    }
    if !line.is_empty() {
        canvas.text(
            fonts,
            x,
            y,
            &line,
            15.0,
            Color::rgb(48, 52, 60),
            false,
            false,
            false,
        );
        y += 35.0;
    }
    canvas.text(
        fonts,
        x,
        y,
        "Reload the page or enter another address.",
        14.0,
        Color::rgb(94, 102, 119),
        false,
        false,
        false,
    );
}

fn scaled_command(command: &DrawCommand, z: f32) -> DrawCommand {
    let rect = |r: &Rect| Rect {
        x: r.x * z,
        y: r.y * z,
        width: r.width * z,
        height: r.height * z,
    };
    match command {
        DrawCommand::PushClip { rect: r } => DrawCommand::PushClip { rect: rect(r) },
        DrawCommand::PopClip => DrawCommand::PopClip,
        DrawCommand::PushFixed => DrawCommand::PushFixed,
        DrawCommand::PopFixed => DrawCommand::PopFixed,
        DrawCommand::PushOpacity { opacity } => DrawCommand::PushOpacity { opacity: *opacity },
        DrawCommand::PopOpacity => DrawCommand::PopOpacity,
        DrawCommand::Rect {
            rect: r,
            color,
            radius,
        } => DrawCommand::Rect {
            rect: rect(r),
            color: *color,
            radius: radius * z,
        },
        DrawCommand::Text {
            x,
            y,
            text,
            size,
            color,
            bold,
            italic,
            monospace,
        } => DrawCommand::Text {
            x: x * z,
            y: y * z,
            text: text.clone(),
            size: size * z,
            color: *color,
            bold: *bold,
            italic: *italic,
            monospace: *monospace,
        },
        DrawCommand::Image { rect: r, key } => DrawCommand::Image {
            rect: rect(r),
            key: key.clone(),
        },
        DrawCommand::Line {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        } => DrawCommand::Line {
            x1: x1 * z,
            y1: y1 * z,
            x2: x2 * z,
            y2: y2 * z,
            color: *color,
            width: width * z,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eris::{
        dom::Document, graphics::ImageStore, layout::LayoutResult, page::Page, worker::apply_edit,
    };

    #[test]
    fn fixed_hit_regions_keep_viewport_coordinates_after_scroll_and_zoom() {
        let mut browser =
            editing_browser("<button id=f>fixed</button><button id=n>normal</button>");
        let snapshot = browser.snapshot.as_mut().unwrap();
        let fixed = snapshot.document.query_selector("#f").unwrap();
        let normal = snapshot.document.query_selector("#n").unwrap();
        let rect = Rect {
            x: 20.0,
            y: 20.0,
            width: 100.0,
            height: 30.0,
        };
        snapshot.layout.hit_regions = vec![
            eris::layout::HitRegion {
                node: normal,
                action: HitAction::Node,
                rect,
                fixed: false,
            },
            eris::layout::HitRegion {
                node: fixed,
                action: HitAction::Node,
                rect,
                fixed: true,
            },
        ];
        browser.scroll = 800.0;
        browser.zoom = 2.0;
        browser.cursor = (60.0, TOOLBAR + 60.0);
        assert_eq!(browser.hit(), Some(fixed));
        browser.cursor = (60.0, TOOLBAR - 1.0);
        assert_eq!(browser.hit(), None);
        browser.snapshot.as_mut().unwrap().layout.hit_regions.pop();
        browser.cursor = (60.0, TOOLBAR + 60.0);
        assert_eq!(browser.hit(), None);
        browser.scroll = 0.0;
        assert_eq!(browser.hit(), Some(normal));
    }

    fn document_layout(document: &Document, fonts: &Fonts) -> LayoutResult {
        let styles = eris::css::compute_styles(document, &document.stylesheets(), 1180.0, 739.0);
        eris::layout::layout(document, &styles, 1180.0, 739.0, fonts)
    }

    fn editing_browser(html: &str) -> Browser {
        let fonts = Fonts::new();
        let document = Document::parse(html);
        let snapshot = Snapshot {
            generation: 1,
            processed_edit_sequence: 0,
            layout: document_layout(&document, &fonts),
            images: ImageStore::new(),
            document,
            title: "Pure editing test".into(),
            url: "https://old.example/".into(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        };
        let visible_nodes = visible_layout_nodes(&snapshot);
        Browser {
            window: None,
            surface: None,
            fonts,
            snapshot: Some(snapshot),
            visible_nodes,
            ready_snapshot: Arc::new(Latest::default()),
            tx: Arc::new(RequestQueue::default()),
            current: Arc::new(AtomicU64::new(1)),
            address: "https://old.example/".into(),
            address_focused: false,
            selection: Selection::default(),
            focused: None,
            input_value: String::new(),
            edit_sequence: 0,
            history: Vec::new(),
            history_index: 0,
            scroll: 0.0,
            zoom: 1.0,
            cursor: (0.0, 0.0),
            modifiers: ModifiersState::default(),
            loading: false,
            status: String::new(),
            hover_clickable: false,
            initial: None,
            started: Instant::now(),
            exit_after: None,
            capture: None,
            startup_error: None,
            worker_error: None,
            clipboard: None,
            applied_fragment_generation: 0,
        }
    }

    #[test]
    fn details_native_focus_and_activation_distinguish_real_and_default_summaries() {
        let mut browser = editing_browser(
            "<style>body{margin:0}</style><details><summary id=s><b id=label>Show</b></summary><input id=closed></details><details id=fallback></details><button id=last>End</button>",
        );
        let s = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#s")
            .unwrap();
        let fallback = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#fallback")
            .unwrap();
        browser.tab_focus();
        assert_eq!(browser.focused, Some(s));
        browser.key(Key::Named(NamedKey::Enter), None);
        assert!(matches!(browser.tx.recv(),Some(Request::Click{node,..}) if node==s));
        browser.tab_focus();
        assert_eq!(browser.focused, Some(fallback));
        browser.key(Key::Named(NamedKey::Space), None);
        assert!(
            matches!(browser.tx.recv(),Some(Request::DefaultSummary{node,..}) if node==fallback)
        );
        let header = browser
            .snapshot
            .as_ref()
            .unwrap()
            .layout
            .hit_regions
            .iter()
            .find(|hit| hit.action == HitAction::DefaultSummary)
            .unwrap()
            .rect;
        browser.cursor = (header.x + 1.0, TOOLBAR + header.y + 1.0);
        browser.click();
        assert_eq!(browser.focused, Some(fallback));
        assert!(
            matches!(browser.tx.recv(),Some(Request::DefaultSummary{node,..}) if node==fallback)
        );
        let mut clipped = editing_browser(
            "<style>details{height:0;overflow:hidden;border:2px solid}</style><details id=d></details><button id=b>end</button>",
        );
        clipped.tab_focus();
        assert_eq!(
            clipped.focused,
            clipped
                .snapshot
                .as_ref()
                .unwrap()
                .document
                .query_selector("#b")
        );
    }

    #[test]
    fn details_closing_revokes_native_focus_before_an_outstanding_edit_ack() {
        let mut browser = editing_browser(
            "<details id=d open><summary>Show</summary><input id=field value=old></details>",
        );
        let field = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.focus_input(field);
        browser.insert_text("pending");
        let mut document = browser.snapshot.as_ref().unwrap().document.clone();
        let details = document.query_selector("#d").unwrap();
        document.remove_attr(details, "open");
        let snapshot = acknowledgement(&browser, 0, document);
        browser.accept_snapshot(snapshot);
        assert!(browser.focused.is_none());
        assert!(!browser.editable(field));
    }

    fn acknowledgement(browser: &Browser, sequence: u64, document: Document) -> Snapshot {
        let old = browser.snapshot.as_ref().unwrap();
        Snapshot {
            generation: old.generation,
            processed_edit_sequence: sequence,
            layout: document_layout(&document, &browser.fonts),
            images: ImageStore::new(),
            document,
            title: old.title.clone(),
            url: old.url.clone(),
            diagnostics: Vec::new(),
            load_ms: 0.0,
        }
    }

    #[test]
    fn page_process_failure_discards_old_content_and_cannot_be_undone_by_a_queued_snapshot() {
        let mut browser = editing_browser("<input id=field value=old>");
        let old = acknowledgement(
            &browser,
            0,
            browser.snapshot.as_ref().unwrap().document.clone(),
        );
        browser.loading = true;
        browser.accept_failure(0, "stale process failed".into());
        assert!(browser.loading);
        assert!(browser.snapshot.is_some());
        browser.accept_failure(1, "process exited\nwithout a reply".into());
        assert!(!browser.loading);
        assert!(browser.snapshot.is_none());
        assert!(browser.focused.is_none());
        assert!(
            browser
                .worker_error
                .as_ref()
                .unwrap()
                .chars()
                .all(|c| !c.is_control())
        );
        browser.accept_snapshot(old);
        assert!(browser.snapshot.is_none());
        browser.navigate("eris:home".into(), true);
        assert!(browser.worker_error.is_none());
        assert!(browser.loading);
        assert_eq!(browser.generation(), 2);
    }

    #[test]
    fn ipc_cancellation_ignores_pending_edits_but_honors_navigation_and_shutdown() {
        let queue = RequestQueue::default();
        let current = AtomicU64::new(7);
        queue
            .send(Request::Edit {
                generation: 7,
                sequence: 1,
                node: 0,
                value: "a".into(),
            })
            .unwrap();
        assert!(!queue.cancelled(7, &current));
        current.store(8, Ordering::Relaxed);
        assert!(queue.cancelled(7, &current));
        assert!(!queue.cancelled(8, &current));
        queue.send(Request::Stop).unwrap();
        assert!(queue.cancelled(8, &current));
    }

    #[test]
    fn only_newest_edit_acknowledgement_reconciles_the_focused_value() {
        let mut browser = editing_browser("<input id=field>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.focus_input(node);
        browser.insert_text("a");
        browser.insert_text("é");
        assert_eq!(browser.edit_sequence, 2);
        let mut document = browser.snapshot.as_ref().unwrap().document.clone();
        document.set_attr(node, "value", "A");
        browser.accept_snapshot(acknowledgement(&browser, 1, document.clone()));
        assert_eq!(browser.input_value, "aé");
        assert_eq!(browser.selection.caret, "aé".len());
        document.set_attr(node, "value", "AÉ");
        browser.accept_snapshot(acknowledgement(&browser, 2, document.clone()));
        assert_eq!(browser.input_value, "AÉ");
        assert_eq!(browser.selection.caret, "AÉ".len());
        document.set_attr(node, "value", "stale");
        browser.accept_snapshot(acknowledgement(&browser, 1, document));
        assert_eq!(browser.input_value, "AÉ");
        assert_eq!(
            browser
                .snapshot
                .as_ref()
                .unwrap()
                .document
                .attr(node, "value"),
            Some("AÉ")
        );
        assert!(
            matches!(browser.tx.recv(),Some(Request::Edit { sequence:2,value,.. }) if value=="aé")
        );
    }

    #[test]
    fn script_input_changes_feed_the_next_local_edit_after_acknowledgement() {
        let html = "<input id=field><script>document.querySelector('#field').addEventListener('input', event => { event.target.value = event.target.value.toUpperCase(); });</script>";
        let mut browser = editing_browser(html);
        let mut page =
            Page::from_html(url::Url::parse("https://old.example/").unwrap(), html, true);
        assert!(page.diagnostics.is_empty(), "{:?}", page.diagnostics);
        let node = page.document.query_selector("#field").unwrap();
        browser.focus_input(node);
        for (insert, queued, result) in [("a", "a", "A"), ("é", "Aé", "AÉ")] {
            browser.insert_text(insert);
            let Some(Request::Edit {
                sequence,
                node: edited,
                value,
                ..
            }) = browser.tx.recv()
            else {
                panic!("expected queued edit");
            };
            assert_eq!(value, queued);
            apply_edit(&mut page, edited, &value);
            assert_eq!(page.document.attr(node, "value"), Some(result));
            browser.accept_snapshot(acknowledgement(&browser, sequence, page.document.clone()));
            assert_eq!(browser.input_value, result);
            assert_eq!(browser.selection.caret, result.len());
        }
        assert!(browser.clipboard.is_none());
    }

    #[test]
    fn boundary_deletions_and_unchanged_replacements_do_not_queue_input_events() {
        let mut browser = editing_browser("<input id=field value=é>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.focus_input(node);
        browser.erase_text(false);
        browser.selection.edge(&browser.input_value, false, false);
        browser.erase_text(true);
        browser.insert_text("");
        browser.selection.all(&browser.input_value);
        browser.insert_text("é");
        assert!(!browser.tx.has_pending());
        assert_eq!(browser.edit_sequence, 0);
        assert_eq!(browser.input_value, "é");
        browser.erase_text(true);
        assert!(
            matches!(browser.tx.recv(),Some(Request::Edit { sequence:1,value,.. }) if value.is_empty())
        );
    }

    #[test]
    fn fragment_navigation_updates_history_and_ignores_pre_fragment_snapshots() {
        let mut browser = editing_browser("<p id=target>Target</p>");
        let original = "https://old.example/path?q=1";
        let target = "https://old.example/path?q=1#target";
        browser.address = original.into();
        browser.snapshot.as_mut().unwrap().url = original.into();
        browser.history = vec![original.into()];
        let stale = acknowledgement(
            &browser,
            0,
            browser.snapshot.as_ref().unwrap().document.clone(),
        );
        browser.navigate_request(Navigation::get(target), true);
        assert_eq!(browser.generation(), 1);
        assert!(!browser.loading);
        assert_eq!(browser.address, target);
        assert_eq!(browser.snapshot.as_ref().unwrap().url, target);
        assert_eq!(
            browser.snapshot.as_ref().unwrap().document.url().as_str(),
            target
        );
        assert_eq!(
            browser
                .snapshot
                .as_ref()
                .unwrap()
                .document
                .base_url()
                .as_str(),
            target
        );
        assert_eq!(browser.history, [original, target]);
        assert_eq!(browser.history_index, 1);
        assert!(
            matches!(browser.tx.recv(),Some(Request::Fragment { generation:1,address }) if address==target)
        );
        browser.accept_snapshot(stale);
        assert_eq!(browser.address, target);
        assert_eq!(browser.snapshot.as_ref().unwrap().url, target);
        assert_eq!(browser.history, [original, target]);
    }

    #[test]
    fn reload_with_fragment_loads_again_and_resets_edit_sequence() {
        let mut browser = editing_browser("<input id=field>");
        let address = "https://old.example/path#target";
        browser.snapshot.as_mut().unwrap().url = address.into();
        browser.address = address.into();
        browser.edit_sequence = 7;
        browser.navigate(address.into(), false);
        assert!(browser.loading);
        assert_eq!(browser.generation(), 2);
        assert_eq!(browser.edit_sequence, 0);
        assert!(
            matches!(browser.tx.recv(),Some(Request::Load { generation:2,navigation }) if navigation.address==address)
        );
    }

    #[test]
    fn fragment_back_and_forward_preserve_path_and_query() {
        let mut browser = editing_browser("<p id=first>First</p><p id=second>Second</p>");
        let first = "https://old.example/a/path?q=1#first";
        let second = "https://old.example/a/path?q=1#second";
        browser.address = second.into();
        browser.snapshot.as_mut().unwrap().url = second.into();
        browser.history = vec![first.into(), second.into()];
        browser.history_index = 1;
        browser.back();
        assert_eq!(browser.address, first);
        assert_eq!(browser.history_index, 0);
        assert_eq!(browser.generation(), 1);
        assert!(
            matches!(browser.tx.recv(),Some(Request::Fragment { address,.. }) if address==first)
        );
        browser.forward();
        assert_eq!(browser.address, second);
        assert_eq!(browser.history_index, 1);
        assert_eq!(browser.generation(), 1);
        assert!(
            matches!(browser.tx.recv(),Some(Request::Fragment { address,.. }) if address==second)
        );
        assert_eq!(browser.history, [first, second]);
    }

    #[test]
    fn stale_generation_cannot_shortcut_navigation_or_replace_newer_state() {
        let mut browser = editing_browser("<input id=field>");
        let stale = acknowledgement(
            &browser,
            3,
            browser.snapshot.as_ref().unwrap().document.clone(),
        );
        browser.current.store(2, Ordering::Relaxed);
        browser.navigate_request(Navigation::get("https://old.example/#target"), true);
        assert_eq!(browser.generation(), 3);
        assert!(matches!(
            browser.tx.recv(),
            Some(Request::Load { generation: 3, .. })
        ));
        browser.accept_snapshot(stale);
        assert_eq!(browser.address, "https://old.example/#target");
        assert!(browser.loading);
        assert_eq!(browser.edit_sequence, 0);
        assert_eq!(browser.history, ["https://old.example/#target"]);
    }

    #[test]
    fn stale_snapshots_cannot_focus_or_queue_edits_for_a_new_page() {
        let mut browser = editing_browser("<input id=field value=old>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.focus_input(node);
        assert!(browser.editable(node));
        browser.current.store(2, Ordering::Relaxed);
        assert!(!browser.editable(node));
        assert!(!browser.has_text_focus());
        browser.send_edit(node);
        browser.insert_text("must not reach another page");
        assert!(!browser.tx.has_pending());
        browser.focused = None;
        browser.focus_input(node);
        assert!(browser.focused.is_none());
        browser.tab_focus();
        assert!(browser.focused.is_none());
        assert!(browser.clipboard.is_none());
    }

    #[test]
    fn current_page_edits_are_bounded_and_keep_their_generation() {
        let mut browser = editing_browser("<input id=field value=é>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.focus_input(node);
        browser.insert_text("🦀");
        assert!(
            matches!(browser.tx.recv(),Some(Request::Edit { generation:1,sequence:1,node:edited,value }) if edited==node && value=="é🦀")
        );
        browser.input_value = "x".repeat(65_537);
        browser.send_edit(node);
        assert!(!browser.tx.has_pending());
    }

    #[test]
    fn input_type_checks_are_case_insensitive_without_opening_clipboard() {
        let mut browser = editing_browser(
            "<input id=hidden type=HIDDEN><input id=file type=FILE><input id=check type=CHECKBOX><input id=password type=PaSsWoRd value=test-secret>",
        );
        for name in ["hidden", "file", "check"] {
            let node = browser
                .snapshot
                .as_ref()
                .unwrap()
                .document
                .query_selector(&format!("#{name}"))
                .unwrap();
            assert!(!browser.editable(node));
        }
        let password = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#password")
            .unwrap();
        browser.focus_input(password);
        assert!(browser.editable(password));
        assert!(browser.password_focused());
        browser.address_focused = true;
        assert!(!browser.password_focused());
        assert!(browser.clipboard.is_none());
    }

    #[test]
    fn native_focus_ignores_foreign_template_inert_and_disabled_controls() {
        let mut browser = editing_browser(
            "<svg><input id=foreign value=svg /></svg><math><input id=math value=math /></math><template><input id=template></template><div inert><input id=inert></div><fieldset disabled><legend><input id=legend value=allowed></legend><input id=blocked><legend><input id=later></legend></fieldset><input id=normal value=normal>",
        );
        for name in ["foreign", "math", "template", "inert", "blocked", "later"] {
            let document = &browser.snapshot.as_ref().unwrap().document;
            let node = if name == "template" {
                assert!(document.query_selector("#template").is_none());
                let template = document.query_selector("template").unwrap();
                let contents = document.template_contents(template).unwrap();
                document.query_selector_from(contents, "#template").unwrap()
            } else {
                document.query_selector(&format!("#{name}")).unwrap()
            };
            assert!(!browser.editable(node), "{name}");
            browser.focus_input(node);
            assert!(browser.focused.is_none(), "{name}");
            browser.send_edit(node);
        }
        assert!(!browser.tx.has_pending());
        let legend = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#legend")
            .unwrap();
        let normal = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#normal")
            .unwrap();
        browser.tab_focus();
        assert_eq!(browser.focused, Some(legend));
        assert!(browser.editable(legend));
        browser.tab_focus();
        assert_eq!(browser.focused, Some(normal));
        assert!(browser.clipboard.is_none());
    }

    #[test]
    fn tab_focus_uses_clipped_geometry_keeps_inline_links_and_offscreen_controls() {
        let mut browser = editing_browser(
            "<style>body{margin:0}#hidden,#hidden-parent{display:none}#clip{height:0;overflow:hidden}#partial-parent{height:10px;overflow:clip}#offscreen{position:absolute;top:2000px}</style><input id=first><input id=hidden><div id=hidden-parent><textarea id=descendant></textarea></div><div id=clip><input id=clipped></div><div id=partial-parent><input id=partial></div><a id=link href='#target'><span><em>inline link</em></span></a><input id=offscreen><button id=last>last</button>",
        );
        let node = |browser: &Browser, selector| {
            browser
                .snapshot
                .as_ref()
                .unwrap()
                .document
                .query_selector(selector)
                .unwrap()
        };
        for selector in ["#hidden", "#descendant", "#clipped"] {
            let id = node(&browser, selector);
            assert!(
                browser
                    .snapshot
                    .as_ref()
                    .unwrap()
                    .document
                    .can_focus_control(id)
            );
            browser.focus_input(id);
            assert!(browser.focused.is_none(), "{selector}");
            assert!(!browser.editable(id), "{selector}");
            browser.send_edit(id);
        }
        assert!(!browser.tx.has_pending());
        let link = node(&browser, "#link");
        assert!(
            !browser
                .snapshot
                .as_ref()
                .unwrap()
                .layout
                .hit_regions
                .iter()
                .any(|h| h.node == link)
        );
        for selector in [
            "#first",
            "#partial",
            "#link",
            "#offscreen",
            "#last",
            "#first",
        ] {
            browser.tab_focus();
            assert_eq!(
                browser.focused,
                Some(node(&browser, selector)),
                "{selector}"
            );
        }
        browser.scroll = 1000.0;
        browser.modifiers = ModifiersState::SHIFT;
        browser.tab_focus();
        assert_eq!(browser.focused, Some(node(&browser, "#last")));
        browser.tab_focus();
        assert_eq!(browser.focused, Some(node(&browser, "#offscreen")));
        assert_eq!(
            browser.scroll, 1000.0,
            "visibility is document geometry, not viewport intersection"
        );
    }

    #[test]
    fn pointer_and_tab_admit_the_same_partly_clipped_input_geometry() {
        let mut browser = editing_browser(
            "<style>body{margin:0}main{width:80px;height:10px;overflow:hidden}input{width:120px;height:30px}</style><main><input id=field value=visible></main>",
        );
        let snapshot = browser.snapshot.as_ref().unwrap();
        let node = snapshot.document.query_selector("#field").unwrap();
        let hit = snapshot
            .layout
            .hit_regions
            .iter()
            .find(|h| h.node == node)
            .unwrap()
            .rect;
        assert_eq!((hit.width, hit.height), (80.0, 10.0));
        browser.tab_focus();
        assert_eq!(browser.focused, Some(node));
        browser.focused = None;
        browser.cursor = (hit.x + hit.width / 2.0, TOOLBAR + hit.y + hit.height / 2.0);
        browser.click();
        assert_eq!(browser.focused, Some(node));
        browser.focused = None;
        browser.cursor = (hit.x + hit.width + 1.0, TOOLBAR + hit.y + hit.height / 2.0);
        browser.click();
        assert!(browser.focused.is_none());
    }

    #[test]
    fn accepted_hidden_geometry_revokes_pending_native_edits_before_acknowledgement() {
        let mut browser = editing_browser("<input id=field value=base>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.focus_input(node);
        browser.insert_text("-pending");
        assert!(matches!(
            browser.tx.recv(),
            Some(Request::Edit { sequence: 1, .. })
        ));
        let mut document = browser.snapshot.as_ref().unwrap().document.clone();
        document.set_attr(node, "style", "display:none");
        browser.accept_snapshot(acknowledgement(&browser, 0, document.clone()));
        assert!(browser.focused.is_none());
        assert!(browser.input_value.is_empty());
        assert!(!browser.editable(node));
        assert_eq!(browser.edit_sequence, 1);
        browser.insert_text("blocked");
        browser.send_edit(node);
        assert!(!browser.tx.has_pending());
        document.set_attr(node, "style", "display:block");
        document.set_attr(node, "value", "acknowledged");
        browser.accept_snapshot(acknowledgement(&browser, 1, document));
        browser.tab_focus();
        assert_eq!(browser.focused, Some(node));
        assert_eq!(browser.input_value, "acknowledged");
    }

    #[test]
    fn obsolete_hidden_snapshot_cannot_replace_accepted_focus_geometry() {
        let mut browser = editing_browser("<input id=field value=current>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.snapshot.as_mut().unwrap().processed_edit_sequence = 2;
        browser.edit_sequence = 2;
        browser.focus_input(node);
        let visible = browser.visible_nodes.clone();
        let mut document = browser.snapshot.as_ref().unwrap().document.clone();
        document.set_attr(node, "style", "display:none");
        browser.accept_snapshot(acknowledgement(&browser, 1, document.clone()));
        assert_eq!(browser.visible_nodes, visible);
        assert_eq!(browser.focused, Some(node));
        assert!(browser.editable(node));
        let mut stale_generation = acknowledgement(&browser, 3, document);
        stale_generation.generation = 0;
        browser.accept_snapshot(stale_generation);
        assert_eq!(browser.visible_nodes, visible);
        assert_eq!(browser.focused, Some(node));
        assert_eq!(browser.input_value, "current");
    }

    #[test]
    fn readonly_range_and_color_controls_cannot_queue_native_text_edits() {
        let mut browser = editing_browser(
            "<input id=readonly readonly value=kept><textarea id=textarea readonly>kept</textarea><input id=range type=RANGE value=5><input id=color type=COLOR value=red>",
        );
        for name in ["readonly", "textarea", "range", "color"] {
            let node = browser
                .snapshot
                .as_ref()
                .unwrap()
                .document
                .query_selector(&format!("#{name}"))
                .unwrap();
            browser.focus_input(node);
            assert_eq!(browser.focused, Some(node));
            let initial = browser.input_value.clone();
            assert!(!browser.editable(node));
            assert!(!browser.has_text_focus());
            browser.insert_text("blocked");
            browser.erase_text(true);
            browser.send_edit(node);
            assert_eq!(browser.input_value, initial);
            assert!(!browser.tx.has_pending());
        }
        assert!(browser.clipboard.is_none());
    }

    #[test]
    fn accepted_snapshot_revokes_editing_before_ack_when_control_policy_changes() {
        for change in ["readonly", "disabled", "inert", "detached", "foreign"] {
            let mut browser = editing_browser("<input id=field value=base>");
            let node = browser
                .snapshot
                .as_ref()
                .unwrap()
                .document
                .query_selector("#field")
                .unwrap();
            browser.focus_input(node);
            browser.insert_text("-pending");
            assert!(matches!(
                browser.tx.recv(),
                Some(Request::Edit { sequence: 1, .. })
            ));
            let mut document = browser.snapshot.as_ref().unwrap().document.clone();
            match change {
                "detached" => document.remove_child(document.nodes[node].parent.unwrap(), node),
                "foreign" => {
                    let eris::dom::NodeKind::Element(element) = &mut document.nodes[node].kind
                    else {
                        unreachable!()
                    };
                    element.namespace = Namespace::Svg;
                }
                attribute => document.set_attr(node, attribute, ""),
            }
            let snapshot = acknowledgement(&browser, 0, document);
            browser.accept_snapshot(snapshot);
            assert!(browser.focused.is_none(), "{change}");
            assert!(browser.input_value.is_empty(), "{change}");
            assert!(!browser.editable(node), "{change}");
            browser.insert_text("must not queue");
            browser.send_edit(node);
            assert!(!browser.tx.has_pending(), "{change}");
            assert!(browser.clipboard.is_none());
        }
    }

    #[test]
    fn obsolete_snapshot_policy_cannot_revoke_current_control_focus() {
        let mut browser = editing_browser("<input id=field value=current>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.snapshot.as_mut().unwrap().processed_edit_sequence = 2;
        browser.edit_sequence = 2;
        browser.focus_input(node);
        let mut document = browser.snapshot.as_ref().unwrap().document.clone();
        document.set_attr(node, "readonly", "");
        let snapshot = acknowledgement(&browser, 1, document);
        browser.accept_snapshot(snapshot);
        assert_eq!(browser.focused, Some(node));
        assert!(browser.editable(node));
        assert_eq!(browser.input_value, "current");
        assert!(browser.clipboard.is_none());
    }

    #[test]
    fn new_document_snapshot_clears_field_state_but_preserves_address_editing() {
        let mut browser = editing_browser("<input id=field value=old>");
        let node = browser
            .snapshot
            .as_ref()
            .unwrap()
            .document
            .query_selector("#field")
            .unwrap();
        browser.focus_input(node);
        browser.clear_stale_focus(1);
        assert_eq!(browser.focused, Some(node));
        browser.clear_stale_focus(2);
        assert!(browser.focused.is_none());
        assert!(browser.input_value.is_empty());
        assert_eq!(browser.selection.range(), 0..0);
        browser.address_focused = true;
        browser.selection.all(&browser.address);
        let selected = browser.selection.range();
        browser.clear_stale_focus(2);
        assert_eq!(browser.selection.range(), selected);
    }

    #[test]
    fn adjacent_resize_and_edit_requests_coalesce_without_crossing_clicks() {
        let queue = RequestQueue::default();
        queue
            .send(Request::Resize {
                width: 800.0,
                height: 600.0,
            })
            .unwrap();
        queue
            .send(Request::Resize {
                width: 1024.0,
                height: 700.0,
            })
            .unwrap();
        queue
            .send(Request::Edit {
                generation: 1,
                sequence: 1,
                node: 5,
                value: "a".into(),
            })
            .unwrap();
        queue
            .send(Request::Edit {
                generation: 1,
                sequence: 2,
                node: 5,
                value: "ab".into(),
            })
            .unwrap();
        queue
            .send(Request::Click {
                generation: 1,
                node: 7,
            })
            .unwrap();
        queue
            .send(Request::Edit {
                generation: 1,
                sequence: 3,
                node: 5,
                value: "abc".into(),
            })
            .unwrap();
        assert_eq!(queue.state.lock().unwrap().pending.len(), 4);
        assert!(matches!(
            queue.recv(),
            Some(Request::Resize {
                width: 1024.0,
                height: 700.0
            })
        ));
        assert!(
            matches!(queue.recv(), Some(Request::Edit { sequence:2,value, .. }) if value == "ab")
        );
        assert!(matches!(queue.recv(), Some(Request::Click { node: 7, .. })));
        assert!(
            matches!(queue.recv(), Some(Request::Edit { sequence:3,value, .. }) if value == "abc")
        );
    }

    #[test]
    fn latest_navigation_supersedes_pending_work_but_keeps_viewport() {
        let queue = RequestQueue::default();
        queue
            .send(Request::Resize {
                width: 1300.0,
                height: 900.0,
            })
            .unwrap();
        queue
            .send(Request::Edit {
                generation: 1,
                sequence: 1,
                node: 3,
                value: "old".into(),
            })
            .unwrap();
        queue
            .send(Request::Load {
                generation: 2,
                navigation: Navigation::get("https://old.example/"),
            })
            .unwrap();
        queue
            .send(Request::Load {
                generation: 3,
                navigation: Navigation::get("https://latest.example/"),
            })
            .unwrap();
        assert_eq!(queue.state.lock().unwrap().pending.len(), 2);
        assert!(matches!(
            queue.recv(),
            Some(Request::Resize { width: 1300.0, .. })
        ));
        assert!(
            matches!(queue.recv(), Some(Request::Load { generation: 3, navigation }) if navigation.address == "https://latest.example/")
        );
    }

    #[test]
    fn event_flood_stays_bounded_and_retains_navigation() {
        let queue = RequestQueue::default();
        queue
            .send(Request::Resize {
                width: 800.0,
                height: 600.0,
            })
            .unwrap();
        queue
            .send(Request::Load {
                generation: 7,
                navigation: Navigation::get("https://latest.example/"),
            })
            .unwrap();
        for node in 0..1000 {
            queue
                .send(Request::Click {
                    generation: 7,
                    node,
                })
                .unwrap();
        }
        let state = queue.state.lock().unwrap();
        assert_eq!(state.pending.len(), MAX_PENDING_REQUESTS);
        assert!(
            state
                .pending
                .iter()
                .any(|request| matches!(request, Request::Load { generation: 7, .. }))
        );
        assert!(
            state
                .pending
                .iter()
                .any(|request| matches!(request, Request::Resize { .. }))
        );
        assert!(matches!(
            state.pending.back(),
            Some(Request::Click { node: 999, .. })
        ));
    }

    #[test]
    fn stop_wakes_worker_and_rejects_later_requests() {
        let queue = Arc::new(RequestQueue::default());
        let receiver = queue.clone();
        let worker = thread::spawn(move || receiver.recv().is_none());
        queue.send(Request::Stop).unwrap();
        assert!(worker.join().unwrap());
        assert!(
            queue
                .send(Request::Resize {
                    width: 1.0,
                    height: 1.0
                })
                .is_err()
        );
    }

    #[test]
    fn snapshots_keep_only_latest_value_and_one_wakeup() {
        let latest = Latest::default();
        assert!(latest.publish(1));
        assert!(!latest.publish(2));
        assert!(!latest.publish(3));
        assert_eq!(latest.take(), Some(3));
        assert_eq!(latest.take(), None);
        assert!(latest.publish(4));
        assert_eq!(latest.take(), Some(4));
    }
}
