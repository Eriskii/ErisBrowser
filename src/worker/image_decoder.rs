//! A fresh, filesystem- and network-denied process for one cross-origin image.
//! Never reuse the process: an image decoder compromised by one origin must
//! not keep state and inspect another origin's encoded response on a later call.
use super::{channel::Channel, codec, sandbox};
use crate::{graphics::RasterImage, net::Resource, page};
use std::{collections::BTreeMap, io, path::Path, time::Duration};

pub(super) fn decode(
    executable: &Path,
    resource: &Resource,
    budget: usize,
    cancel: impl Fn() -> bool,
) -> Result<RasterImage, String> {
    if budget < 4 {
        return Err("decoded image budget exhausted".into());
    }
    let mut channel = Channel::spawn(executable, "--image-decoder")?;
    let response = channel.exchange_bounded(
        codec::encode_decoder_init()?,
        Duration::from_secs(5),
        8192,
        &cancel,
        |_| Ok(None),
    )?;
    let reply = codec::decode_reply(&response)?;
    if reply.snapshot.is_some() || reply.navigation.is_some() {
        return Err("unexpected image decoder startup reply".into());
    }
    let response = channel.exchange_bounded(
        codec::encode_image_request(&resource.content_type, &resource.bytes, budget)?,
        Duration::from_secs(10),
        budget.saturating_add(32).max(8192),
        cancel,
        |_| Ok(None),
    )?;
    // Channel::drop kills and reaps the process on every success/error path.
    codec::decode_image_result(&response, budget)?
}

pub(super) fn serve() -> Result<(), String> {
    codec::decode_decoder_init(&codec::read_frame(&mut io::stdin().lock(), 64)?)?;
    if let Err(error) =
        sandbox::check_inherited_descriptors().and_then(|()| sandbox::restrict_renderer())
    {
        codec::write_frame(&mut io::stdout().lock(), &codec::encode_error(&error)?)?;
        return Err(error);
    }
    codec::write_frame(
        &mut io::stdout().lock(),
        &codec::encode_reply(&super::Reply::empty())?,
    )?;
    let request = codec::read_frame(
        &mut io::stdin().lock(),
        crate::net::MAX_RESOURCE_BYTES + 16384,
    )?;
    let (content_type, bytes, budget) = codec::decode_image_request(&request)?;
    let result = if content_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .eq_ignore_ascii_case("image/svg+xml")
    {
        let source = Resource {
            url: url::Url::parse("about:blank").expect("constant URL"),
            bytes: bytes.to_vec(),
            content_type,
            headers: BTreeMap::new(),
            status: 200,
            origin_clean: false,
            decoded_image: None,
        }
        .text();
        crate::svg::render_with_budget(&source, None, None, budget)
    } else {
        page::decode_image_with_budget(bytes, budget)
    };
    codec::write_frame(
        &mut io::stdout().lock(),
        &codec::encode_image_result(&result, budget)?,
    )
}
