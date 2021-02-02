// Copyright 2020 David Young
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! Module for the core parsing functionality

use bytes::Bytes;
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, NodeData};
use std::collections::HashMap;
use url::Url;

// https://github.com/Y2Z/monolith/blob/fa71f6a42c94df4c48d01819922afe1248eabad5/src/utils.rs#L13
const MAGIC: [(&[u8], &str); 18] = [
    // Image
    (b"GIF87a", "image/gif"),
    (b"GIF89a", "image/gif"),
    (b"\xFF\xD8\xFF", "image/jpeg"),
    (b"\x89PNG\x0D\x0A\x1A\x0A", "image/png"),
    (b"<svg ", "image/svg+xml"),
    (b"RIFF....WEBPVP8 ", "image/webp"),
    (b"\x00\x00\x01\x00", "image/x-icon"),
    // Audio
    (b"ID3", "audio/mpeg"),
    (b"\xFF\x0E", "audio/mpeg"),
    (b"\xFF\x0F", "audio/mpeg"),
    (b"OggS", "audio/ogg"),
    (b"RIFF....WAVEfmt ", "audio/wav"),
    (b"fLaC", "audio/x-flac"),
    // Video
    (b"RIFF....AVI LIST", "video/avi"),
    (b"....ftyp", "video/mp4"),
    (b"\x00\x00\x01\x0B", "video/mpeg"),
    (b"....moov", "video/quicktime"),
    (b"\x1A\x45\xDF\xA3", "video/webm"),
];

/// Search image, style, and script resources and store their URIs
pub(crate) fn parse_resource_urls(
    url_base: &Url,
    page: &str,
) -> Vec<ResourceUrl> {
    let document = parse_html().one(page);

    // Collect resource URLs for each element type
    let mut resource_urls = Vec::new();

    for element in document.select("img").unwrap() {
        let node = element.as_node();
        if let NodeData::Element(data) = node.data() {
            let attr = data.attributes.borrow();
            if let Some(u) = attr.get("src") {
                if let Ok(u) = url_base.join(u) {
                    resource_urls.push(ResourceUrl::Image(u));
                }
            }
        }
    }

    for element in document.select("link").unwrap() {
        let node = element.as_node();
        if let NodeData::Element(data) = node.data() {
            let attr = data.attributes.borrow();
            if Some("stylesheet") == attr.get("rel") {
                if let Some(u) = attr.get("href") {
                    if let Ok(u) = url_base.join(u) {
                        resource_urls.push(ResourceUrl::Css(u));
                    }
                }
            }
        }
    }

    for element in document.select("script").unwrap() {
        let node = element.as_node();
        if let NodeData::Element(data) = node.data() {
            let attr = data.attributes.borrow();
            if let Some(u) = attr.get("src") {
                if let Ok(u) = url_base.join(u) {
                    resource_urls.push(ResourceUrl::Javascript(u));
                }
            }
        }
    }

    // Dedup the URLs to avoid fetching the same one twice
    resource_urls.sort();
    resource_urls.dedup();

    resource_urls
}

/// Tag the resource URLs with the type of resource they correspond to
#[derive(Debug, PartialEq, Eq)]
pub enum ResourceUrl {
    /// Javascript files
    Javascript(Url),
    /// CSS files
    Css(Url),
    /// Image files
    Image(Url),
}

impl ResourceUrl {
    /// Returns a reference to the inner [`Url`]
    pub fn url(&self) -> &Url {
        use ResourceUrl::*;
        match self {
            Javascript(u) => &u,
            Css(u) => &u,
            Image(u) => &u,
        }
    }
}

impl PartialOrd for ResourceUrl {
    fn partial_cmp(&self, rhs: &ResourceUrl) -> Option<std::cmp::Ordering> {
        Some(self.url().cmp(rhs.url()))
    }
}

impl Ord for ResourceUrl {
    fn cmp(&self, rhs: &ResourceUrl) -> std::cmp::Ordering {
        self.url().cmp(rhs.url())
    }
}

/// Newtype wrapper around [`HashMap`], mapping between resource URLs
/// and the downloaded file contents
pub type ResourceMap = HashMap<Url, Resource>;

/// Generic resource type
#[derive(Debug, PartialEq, Eq)]
pub enum Resource {
    /// Javascript is stored as a String
    Javascript(String),
    /// Stylesheets are stored as a String
    Css(String),
    /// Images are stored as an [`ImageResource`] to allow the mimetype
    /// metadata to be useful
    Image(ImageResource),
}

/// Data type representing an image
#[derive(Debug, PartialEq, Eq)]
pub struct ImageResource {
    /// Raw image data
    pub data: Bytes,
    /// Mime type of the image, e.g. `image/png`
    pub mimetype: String,
}

impl ImageResource {
    /// Encode the image data as base 64 and embed it into a `data:` URI,
    /// e.g. `data:image/png;base64,iVBORw0...`.
    pub fn to_data_uri(&self) -> String {
        let encoded = base64::encode(&self.data);
        format!("data:{};base64,{}", self.mimetype, encoded)
    }
}

// https://github.com/Y2Z/monolith/blob/fa71f6a42c94df4c48d01819922afe1248eabad5/src/utils.rs#L44
pub(crate) fn mimetype_from_response(data: &[u8], url: &Url) -> String {
    for item in MAGIC.iter() {
        if data.starts_with(item.0) {
            return item.1.to_string();
        }
    }

    if url.path().to_lowercase().ends_with(".svg") {
        return "image/svg+xml".to_string();
    }

    "".to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    fn u() -> Url {
        Url::parse("http://example.com").unwrap()
    }

    #[test]
    fn test_image_resouce_base_64() {
        let img = ImageResource {
            data: Bytes::from(
                include_bytes!(
                    "../dynamic_tests/resources/rustacean-flat-happy.png"
                )
                .to_vec(),
            ),
            mimetype: "image/png".to_string(),
        };

        let data_uri = img.to_data_uri();

        // base64 < dynamic_tests/resources/rustacean-flat-happy.png
        assert!(data_uri
            .starts_with("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAB"));
        assert!(data_uri.ends_with("Q/hkoEnAH1wAAAABJRU5ErkJggg=="));
    }

    #[test]
    fn test_image_tags() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <head></head>
            <body>
                <div id="content">
                    <img src="/images/fun.png" />
                </div>
            </body>
        </html>
        "#;

        let resource_urls = parse_resource_urls(&u(), &html);

        assert_eq!(resource_urls.len(), 1);
        assert_eq!(
            resource_urls[0],
            ResourceUrl::Image(
                Url::parse("http://example.com/images/fun.png").unwrap()
            )
        );
    }

    #[test]
    fn test_css_tags() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <link rel="stylesheet" type="text/css" href="/style.css" />
                <link rel="something_else" href="NOT_ALLOWED" />
            </head>
            <body>
                <div id="content">
                </div>
            </body>
        </html>
        "#;

        let resource_urls = parse_resource_urls(&u(), &html);

        assert_eq!(resource_urls.len(), 1);
        assert_eq!(
            resource_urls[0],
            ResourceUrl::Css(
                Url::parse("http://example.com/style.css").unwrap()
            )
        );
    }

    #[test]
    fn test_script_tags() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <script language="javascript" src="/js.js"></script>
            </head>
            <body>
                <div id="content">
                </div>
            </body>
        </html>
        "#;

        let resource_urls = parse_resource_urls(&u(), &html);

        assert_eq!(resource_urls.len(), 1);
        assert_eq!(
            resource_urls[0],
            ResourceUrl::Javascript(
                Url::parse("http://example.com/js.js").unwrap()
            )
        );
    }

    #[test]
    fn test_deep_nesting() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <script language="javascript" src="/js.js"></script>
                <link rel="stylesheet" href="1.css" type="text/css" />
            </head>
            <body>
                <div id="content">
                    <div><div><div>
                            <img src="1.png" />
                        </div></div>
                        <script src="2.js"></script>
                    </div>
                    <div><div>
                        <img src="2.tiff" />
                    </div></div>
                </div>
            </body>
        </html>
        "#;

        let resource_urls = parse_resource_urls(&u(), &html);

        let mut test_urls = vec![
            ResourceUrl::Javascript(
                Url::parse("http://example.com/js.js").unwrap(),
            ),
            ResourceUrl::Css(Url::parse("http://example.com/1.css").unwrap()),
            ResourceUrl::Image(Url::parse("http://example.com/1.png").unwrap()),
            ResourceUrl::Javascript(
                Url::parse("http://example.com/2.js").unwrap(),
            ),
            ResourceUrl::Image(
                Url::parse("http://example.com/2.tiff").unwrap(),
            ),
        ];
        test_urls.sort();

        assert_eq!(resource_urls.len(), 5);
        assert_eq!(resource_urls, test_urls,);
    }

    #[test]
    fn test_relative_paths() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <head></head>
            <body>
                <div id="content">
                    <img src="../../images/fun.png" />
                    <img src="/absolute_path.jpg" />
        <img src="https://www.rust-lang.org/static/images/rust-logo-blk.svg" />
                </div>
            </body>
        </html>
        "#;

        let u = Url::parse("http://example.com/one/two/three/four/").unwrap();
        let resource_urls = parse_resource_urls(&u, &html);
        let mut test_urls = vec![
            ResourceUrl::Image(
                Url::parse("http://example.com/one/two/images/fun.png")
                    .unwrap(),
            ),
            ResourceUrl::Image(
                Url::parse("http://example.com/absolute_path.jpg").unwrap(),
            ),
            ResourceUrl::Image(
                Url::parse(
                    "https://www.rust-lang.org/static/images/rust-logo-blk.svg",
                )
                .unwrap(),
            ),
        ];
        test_urls.sort();

        assert_eq!(resource_urls.len(), 3);
        assert_eq!(resource_urls, test_urls);
    }

    #[test]
    fn test_upper_case_tags() {
        let html = r#"
        <HTML>
            <HEAD>
                <SCRIPT LANGUAGE="javascript" SRC="/js.js"></SCRIPT>
            </HEAD>
            <BODY>
                <DIV ID="content">
                </DIV>
            </BODY>
        </HTML>
        "#;

        let resource_urls = parse_resource_urls(&u(), &html);

        assert_eq!(resource_urls.len(), 1);
        assert_eq!(
            resource_urls[0],
            ResourceUrl::Javascript(
                Url::parse("http://example.com/js.js").unwrap()
            )
        );
    }

    #[test]
    fn test_malformed_html() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <head>
                <script language="javascript" src="/js.js"></script>
            </head>
            <body>
                <div id="content">
                    <p>Closing paragraphs is for losers
                    <p><img src="a.jpg">
                </div>
            </body>
        </html>
        "#;

        let resource_urls = parse_resource_urls(&u(), &html);
        let mut test_urls = vec![
            ResourceUrl::Javascript(
                Url::parse("http://example.com/js.js").unwrap(),
            ),
            ResourceUrl::Image(Url::parse("http://example.com/a.jpg").unwrap()),
        ];
        test_urls.sort();

        assert_eq!(resource_urls.len(), 2);
        assert_eq!(resource_urls, test_urls);
    }

    #[test]
    fn test_mimetype_detection() {
        let data: &[u8] = include_bytes!(
            "../dynamic_tests/resources/rustacean-flat-happy.png"
        );
        let url = Url::parse("http://example.com/ferris.png").unwrap();
        let mimetype = mimetype_from_response(&data, &url);
        assert_eq!(mimetype, "image/png");

        let data: &[u8] =
            include_bytes!("../dynamic_tests/resources/rust-logo-blk.svg");
        let url = Url::parse("http://example.com/rust.svg").unwrap();
        let mimetype = mimetype_from_response(&data, &url);
        assert_eq!(mimetype, "image/svg+xml");
    }
}
