// Copyright 2020 David Young
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! ### Blocking
//!
//! This is the blocking API
//!
//! ```no_run
//! use web_archive::blocking;
//!
//! // Fetch page and all its resources
//! let archive = blocking::archive("http://example.com", Default::default())
//!     .unwrap();
//!
//! // Embed the resources into the page
//! let page = archive.embed_resources();
//! println!("{}", page);
//!
//! ```

use crate::error::Error;
use crate::page_archive::PageArchive;
use crate::parsing::{
    mimetype_from_response, parse_resource_urls, ImageResource, Resource,
    ResourceMap, ResourceUrl,
};
use crate::ArchiveOptions;
use reqwest::{Proxy, StatusCode};
use std::convert::TryInto;
use std::fmt::Display;
use url::Url;

/// The blocking archive function.
///
/// Takes in a URL and attempts to download the page and its resources.
/// Network errors get wrapped in [`Error`] and returned as the `Err`
/// case.
pub fn archive<U>(url: U, options: ArchiveOptions) -> Result<PageArchive, Error>
where
    U: TryInto<Url>,
    <U as TryInto<Url>>::Error: Display,
{
    let url: Url = url
        .try_into()
        .map_err(|e| Error::ParseError(format!("{}", e)))?;

    // Initialise client
    let mut client = reqwest::blocking::Client::builder()
        .use_native_tls()
        .danger_accept_invalid_certs(options.accept_invalid_certificates)
        .danger_accept_invalid_hostnames(options.accept_invalid_certificates);
    if let Some(proxy) = options.proxy {
        client = client.proxy(Proxy::all(proxy)?);
    }
    let client = client.build()?;

    // Fetch the page contents
    let content = client.get(url.clone()).send()?.text()?;

    // Determine the resources that the page needs
    let resource_urls = parse_resource_urls(&url, &content);
    let mut resource_map = ResourceMap::new();

    // Download them
    for resource_url in resource_urls {
        use ResourceUrl::*;

        let response = client.get(resource_url.url().clone()).send()?;
        if response.status() != StatusCode::OK {
            // Skip any errors
            println!("Code: {}", response.status());
            continue;
        }
        match resource_url {
            Image(u) => {
                let data = response.bytes()?;
                let mimetype = mimetype_from_response(&data, &u);
                resource_map.insert(
                    u,
                    Resource::Image(ImageResource { data, mimetype }),
                );
            }
            Css(u) => {
                resource_map.insert(u, Resource::Css(response.text()?));
            }
            Javascript(u) => {
                resource_map.insert(u, Resource::Javascript(response.text()?));
            }
        }
    }

    Ok(PageArchive {
        url,
        content,
        resource_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_invalid_url_blocking() {
        let u = "this~is~not~a~url";

        let res = archive(u, Default::default());
        assert!(res.is_err());

        if let Err(Error::ParseError(_err)) = res {
            // Okay, it's a parse error
        } else {
            panic!("Expected parse error");
        }
    }
}
