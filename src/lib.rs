#![warn(missing_docs)]
#![forbid(unsafe_code)]

//! The purpose of this crate is to download a web page, then download
//! its linked image, Javascript, and CSS resources and embed them in
//! the HTML.
//!
//! Both async and blocking APIs are provided, making use of `reqwest`'s
//! support for both. The blocking APIs are enabled with the `blocking`
// Copyright 2021 David Young
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! feature.
//!
//! ## Examples
//!
//! ### Async
//!
//! ```no_run
//! use web_archive::archive;
//!
//! # async fn archive_async() {
//! // Fetch page and all its resources
//! let archive = archive("http://example.com", Default::default())
//!     .await
//!     .unwrap();
//!
//! // Embed the resources into the page
//! let page = archive.embed_resources();
//! println!("{}", page);
//! # }
//!
//! ```
//!
//! ### Blocking
//!
//! ```no_run
//! use web_archive::blocking;
//!
//! // Fetch page and all its resources
//! let archive =
//!     blocking::archive("http://example.com", Default::default()).unwrap();
//!
//! // Embed the resources into the page
//! let page = archive.embed_resources();
//! println!("{}", page);
//!
//! ```
//!
//! ### Ignore certificate errors (dangerous!)
//!
//! ```no_run
//! use web_archive::{archive, ArchiveOptions};
//!
//! # async fn archive_async() {
//! // Fetch page and all its resources
//! let archive_options = ArchiveOptions {
//!     accept_invalid_certificates: true,
//!     ..Default::default()
//! };
//! let archive = archive("http://example.com", archive_options)
//!     .await
//!     .unwrap();
//!
//! // Embed the resources into the page
//! let page = archive.embed_resources();
//! println!("{}", page);
//! # }
//!
//! ```

pub use error::Error;
pub use page_archive::PageArchive;
use parsing::{mimetype_from_response, parse_resource_urls};
pub use parsing::{ImageResource, Resource, ResourceMap, ResourceUrl};
use reqwest::StatusCode;
use std::convert::TryInto;
use std::fmt::Display;
use url::Url;

pub mod error;
pub mod page_archive;
pub mod parsing;

#[cfg(feature = "blocking")]
pub mod blocking;

/// The async archive function.
///
/// Takes in a URL and attempts to download the page and its resources.
/// Network errors get wrapped in [`Error`] and returned as the `Err`
/// case.
pub async fn archive<U>(
    url: U,
    options: ArchiveOptions,
) -> Result<PageArchive, Error>
where
    U: TryInto<Url>,
    <U as TryInto<Url>>::Error: Display,
{
    let url: Url = url
        .try_into()
        .map_err(|e| Error::ParseError(format!("{}", e)))?;

    // Initialise client
    let client = reqwest::Client::builder()
    	.use_native_tls()
    	.danger_accept_invalid_certs(options.accept_invalid_certificates)
    	.danger_accept_invalid_hostnames(options.accept_invalid_certificates)
    	.build()?;

    // Fetch the page contents
    let content = client.get(url.clone()).send().await?.text().await?;

    // Determine the resources that the page needs
    let resource_urls = parse_resource_urls(&url, &content);

    // Download them
    let mut resource_map = ResourceMap::new();
    for resource_url in resource_urls {
        use ResourceUrl::*;

        let response = client.get(resource_url.url().clone()).send().await?;
        if response.status() != StatusCode::OK {
            // Skip any errors
            continue;
        }
        match resource_url {
            Image(u) => {
                // Get mimetype of image
                let data = response.bytes().await?;
                let mimetype = mimetype_from_response(&data, &u);
                resource_map.insert(
                    u,
                    Resource::Image(ImageResource { data, mimetype }),
                );
            }
            Css(u) => {
                resource_map.insert(u, Resource::Css(response.text().await?));
            }
            Javascript(u) => {
                resource_map
                    .insert(u, Resource::Javascript(response.text().await?));
            }
        }
    }

    Ok(PageArchive {
        url,
        content,
        resource_map,
    })
}

/// Configuration options to control aspects of the archiving behaviour.
pub struct ArchiveOptions {
    /// Accept invalid certificates or certificates that do not match
    /// the requested hostname. For example, performing an HTTPS request
    /// against an IP address will more than likely result in a hostname
    /// mismatch.
    ///
    /// Corresponds to [`reqwest::ClientBuilder::danger_accept_invalid_certs`]
    /// and [`reqwest::ClientBuilder::danger_accept_invalid_hostnames`].
    ///
    /// Default: `false`
    pub accept_invalid_certificates: bool,
}

impl Default for ArchiveOptions {
    fn default() -> Self {
        Self {
            accept_invalid_certificates: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn parse_invalid_url_async() {
        let u = "this~is~not~a~url";

        let res = block_on(archive(u, Default::default()));
        assert!(res.is_err());

        if let Err(Error::ParseError(_err)) = res {
            // Okay, it's a parse error
        } else {
            panic!("Expected parse error");
        }
    }
}
