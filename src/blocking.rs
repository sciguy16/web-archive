use crate::error::Error;
use crate::page_archive::PageArchive;
use crate::parsing::{parse_resource_urls, Resource, ResourceMap, ResourceUrl};
use std::convert::TryInto;
use std::fmt::Display;
use url::Url;

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
    let content = client.get(url).send()?.text()?;

    // Determine the resources that the page needs
    let resource_urls = parse_resource_urls(&content)?;
    let mut resource_map = ResourceMap::new();

    // Download them
    for resource_url in resource_urls {
        use ResourceUrl::*;
        match resource_url {
            Image(u) => {
                let content = client.get(u.clone()).send()?.bytes()?;
                resource_map.insert(u, Resource::Image(content));
            }
            Css(u) => {
                let content = client.get(u.clone()).send()?.text()?;
                resource_map.insert(u, Resource::Css(content));
            }
            Javascript(u) => {
                let content = client.get(u.clone()).send()?.text()?;
                resource_map.insert(u, Resource::Javascript(content));
            }
        }
    }

    Ok(PageArchive {
        content,
        resource_map,
    })
}
