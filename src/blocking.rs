//! ### Blocking
//!
//! This is the blocking API
//!
//! ```no_run
//! use web_archive::blocking;
//!
//! // Fetch page and all its resources
//! let archive = blocking::archive("http://example.com").unwrap();
//!
//! // Embed the resources into the page
//! let page = archive.embed_resources();
//! println!("{}", page);
//!
//! ```

use crate::error::Error;
use crate::page_archive::PageArchive;
use crate::parsing::{
    parse_resource_urls, ImageResource, Resource, ResourceMap, ResourceUrl,
};
use reqwest::StatusCode;
use std::convert::TryInto;
use std::fmt::Display;
use url::Url;

/// The blocking archive function.
///
/// Takes in a URL and attempts to download the page and its resources.
/// Network errors get wrapped in [`Error`] and returned as the `Err`
/// case.
pub fn archive<U>(url: U) -> Result<PageArchive, Error>
where
    U: TryInto<Url>,
    <U as TryInto<Url>>::Error: Display,
{
    let url: Url = url
        .try_into()
        .map_err(|e| Error::ParseError(format!("{}", e)))?;

    // Initialise client
    let client = reqwest::blocking::Client::new();

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
                resource_map.insert(
                    u,
                    Resource::Image(ImageResource {
                        data: response.bytes()?,
                        mimetype: String::new(),
                    }),
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

        let res = archive(u);
        assert!(res.is_err());

        if let Err(Error::ParseError(_err)) = res {
            // Okay, it's a parse error
        } else {
            panic!("Expected parse error");
        }
    }
}
