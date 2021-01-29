pub use error::Error;
pub use page_archive::PageArchive;
use parsing::{parse_resource_urls, Resource, ResourceMap, ResourceUrl};
use reqwest::StatusCode;
use std::convert::TryInto;
use std::fmt::Display;
use url::Url;

pub mod error;
pub mod page_archive;
pub mod parsing;

#[cfg(feature = "blocking")]
pub mod blocking;

pub async fn archive<U>(url: U) -> Result<PageArchive, Error>
where
    U: TryInto<Url>,
    <U as TryInto<Url>>::Error: Display,
{
    let url: Url = url
        .try_into()
        .map_err(|e| Error::ParseError(format!("{}", e)))?;

    // Initialise client
    let client = reqwest::Client::new();

    // Fetch the page contents
    let content = client.get(url.clone()).send().await?.text().await?;

    // Determine the resources that the page needs
    let resource_urls = parse_resource_urls(&url, &content)?;

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
                resource_map
                    .insert(u, Resource::Image(response.bytes().await?));
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
        content,
        resource_map,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn parse_invalid_url_async() {
        let u = "this~is~not~a~url";

        let res = block_on(archive(u));
        assert!(res.is_err());

        if let Err(Error::ParseError(_err)) = res {
            // Okay, it's a parse error
        } else {
            panic!("Expected parse error");
        }
    }
}
