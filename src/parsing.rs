use crate::error::Error;
use bytes::Bytes;
use html5ever::tendril::TendrilSink;
use html5ever::{parse_document, ParseOpts};
use markup5ever_rcdom::RcDom;
use std::collections::HashMap;
use url::Url;

pub(crate) fn parse_resource_urls(
    page: &str,
) -> Result<Vec<ResourceUrl>, Error> {
    let mut buf = page.as_bytes();

    let parse_opts: ParseOpts = Default::default();

    let parsed = parse_document(RcDom::default(), parse_opts)
        .from_utf8()
        .read_from(&mut buf);

    todo!()
}

pub enum ResourceUrl {
    Javascript(Url),
    Css(Url),
    Image(Url),
}

pub type ResourceMap = HashMap<Url, Resource>;

pub enum Resource {
    Javascript(String),
    Css(String),
    Image(Bytes),
}
