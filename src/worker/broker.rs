//! Resource access lives outside the renderer. The broker's authority comes
//! from the navigation authorized by the UI, never a claimed initiator URL.
use super::{channel::Channel, codec, sandbox};
use crate::{
    net::{self, Fetcher, Resource, ResourceKind},
    page::Navigation,
};
use std::{
    io,
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

pub(super) struct FetchRequest {
    pub url: Url,
    pub kind: ResourceKind,
    pub form_body: Option<String>,
}
pub(super) struct BrokerInit {
    pub navigation: Navigation,
    pub root: Option<PathBuf>,
}
pub(super) struct BrokerClient {
    channel: Channel,
}
impl BrokerClient {
    pub(super) fn spawn(
        executable: &Path,
        init: &BrokerInit,
        cancel: impl Fn() -> bool,
    ) -> Result<Self, String> {
        let mut channel = Channel::spawn(executable, "--resource-broker")?;
        let response = channel.exchange(
            codec::encode_broker_init(init)?,
            Duration::from_secs(5),
            cancel,
            |_| Ok(None),
        )?;
        let reply = codec::decode_reply(&response)?;
        if reply.snapshot.is_some() || reply.navigation.is_some() {
            return Err("unexpected broker startup reply".into());
        }
        Ok(Self { channel })
    }
    pub(super) fn fetch(
        &mut self,
        request: &FetchRequest,
        cancel: impl Fn() -> bool,
    ) -> Result<Resource, String> {
        let bytes = self.channel.exchange_bounded(
            codec::encode_fetch_request(request)?,
            Duration::from_secs(40),
            // URL (16 MiB), body (8 MiB), bounded headers and framing.
            26 * 1024 * 1024,
            cancel,
            |_| Ok(None),
        )?;
        let resource = codec::decode_resource(&bytes)??;
        if resource.decoded_image.is_some() {
            return Err("broker sent unexpected decoded pixels".into());
        }
        Ok(resource)
    }
    pub(super) fn pid(&self) -> u32 {
        self.channel.pid()
    }
}

struct Policy {
    navigation: Navigation,
    initial: Url,
    committed: Option<Url>,
    attempted_document: bool,
    csp: bool,
    requests: usize,
    fetcher: Fetcher,
}
impl Policy {
    fn new(init: BrokerInit) -> Result<Self, String> {
        let initial = Url::parse(&init.navigation.address).map_err(|e| e.to_string())?;
        super::validate_address(&initial)?;
        Ok(Self {
            navigation: init.navigation,
            initial,
            committed: None,
            attempted_document: false,
            csp: false,
            requests: 0,
            fetcher: Fetcher::new(init.root),
        })
    }
    fn fetch(&mut self, request: FetchRequest) -> Result<Resource, String> {
        self.requests += 1;
        if self.requests > net::MAX_RESOURCES {
            return Err("broker request budget exceeded".into());
        }
        if request.kind == ResourceKind::Document {
            if self.attempted_document {
                return Err("duplicate document request".into());
            }
            self.attempted_document = true;
            if request.url != self.initial || request.form_body != self.navigation.form_body {
                return Err("document request differs from authorized navigation".into());
            }
            if matches!(self.initial.scheme(), "eris" | "about") {
                return Err("built-in pages do not use the resource broker".into());
            }
            let response = self
                .fetcher
                .fetch_document(&self.initial, self.navigation.form_body.as_deref())?;
            self.csp = response.headers.contains_key("content-security-policy");
            self.committed = Some(response.url.clone());
            Ok(response)
        } else {
            if request.form_body.is_some() {
                return Err("subresources cannot submit forms".into());
            }
            let committed = self
                .committed
                .as_ref()
                .ok_or("subresource requested before the authorized document")?;
            if self.csp {
                return Err("document CSP blocks external resources in the current policy".into());
            }
            self.fetcher
                .fetch(&request.url, Some(committed), request.kind)
        }
    }
}

pub(super) fn fetch_via_parent(
    url: &Url,
    _initiator: Option<&Url>,
    kind: ResourceKind,
    form_body: Option<&str>,
) -> Result<Resource, String> {
    let request = FetchRequest {
        url: url.clone(),
        kind,
        form_body: form_body.map(str::to_owned),
    };
    codec::write_frame(
        &mut io::stdout().lock(),
        &codec::encode_fetch_request(&request)?,
    )?;
    let bytes = codec::read_frame(&mut io::stdin().lock(), codec::MAX_FRAME)?;
    codec::decode_resource(&bytes)?
}

pub fn serve() -> Result<(), String> {
    let init = codec::decode_broker_init(&codec::read_frame(
        &mut io::stdin().lock(),
        codec::MAX_REQUEST,
    )?)?;
    if let Err(error) = sandbox::check_inherited_descriptors()
        .and_then(|()| sandbox::restrict_broker(init.root.as_deref()))
    {
        codec::write_frame(&mut io::stdout().lock(), &codec::encode_error(&error)?)?;
        return Err(error);
    }
    net::use_synchronous_dns_for_broker();
    let mut policy = Policy::new(init)?;
    codec::write_frame(
        &mut io::stdout().lock(),
        &codec::encode_reply(&super::Reply::empty())?,
    )?;
    loop {
        let bytes = codec::read_frame(&mut io::stdin().lock(), codec::MAX_REQUEST)?;
        let request = codec::decode_fetch_request(&bytes)?;
        let result = policy.fetch(request);
        codec::write_frame(&mut io::stdout().lock(), &codec::encode_resource(&result)?)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy() -> Policy {
        Policy::new(BrokerInit {
            navigation: Navigation::get("data:text/html,authorized"),
            root: None,
        })
        .unwrap()
    }
    fn request(url: &str, kind: ResourceKind) -> FetchRequest {
        FetchRequest {
            url: Url::parse(url).unwrap(),
            kind,
            form_body: None,
        }
    }
    #[test]
    fn broker_requires_the_exact_authorized_document_and_committed_initiator() {
        let mut p = policy();
        assert!(
            p.fetch(request("file:///etc/passwd", ResourceKind::Image))
                .is_err()
        );
        assert!(
            p.fetch(request("data:text/html,other", ResourceKind::Document))
                .is_err()
        );
        assert!(
            p.fetch(request("data:text/html,authorized", ResourceKind::Document))
                .is_err()
        );
        let mut p = policy();
        assert_eq!(
            p.fetch(request("data:text/html,authorized", ResourceKind::Document))
                .unwrap()
                .bytes,
            b"authorized"
        );
        assert!(
            p.fetch(request("data:text/html,authorized", ResourceKind::Document))
                .is_err()
        );
        assert!(
            p.fetch(request("file:///etc/passwd", ResourceKind::Image))
                .is_err()
        );
        assert!(
            p.fetch(request("data:text/javascript,42", ResourceKind::Script))
                .is_err()
        );
        assert!(
            p.fetch(request(
                "https://untrusted.invalid/script.js",
                ResourceKind::Script
            ))
            .is_err()
        );
    }
    #[test]
    fn broker_csp_and_request_caps_apply_before_fetching() {
        let mut p = policy();
        p.fetch(request("data:text/html,authorized", ResourceKind::Document))
            .unwrap();
        p.csp = true;
        assert!(
            p.fetch(request("data:image/svg+xml,svg", ResourceKind::Image))
                .unwrap_err()
                .contains("CSP")
        );
        p.csp = false;
        p.requests = net::MAX_RESOURCES;
        assert!(
            p.fetch(request("data:image/svg+xml,svg", ResourceKind::Image))
                .unwrap_err()
                .contains("budget")
        );
    }
}
