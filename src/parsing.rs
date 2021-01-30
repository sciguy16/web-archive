use crate::error::Error;
use bytes::Bytes;
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, NodeData};
use std::collections::HashMap;
use url::Url;

pub(crate) fn parse_resource_urls(
    url_base: &Url,
    page: &str,
) -> Result<Vec<ResourceUrl>, Error> {
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

    Ok(resource_urls)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceUrl {
    Javascript(Url),
    Css(Url),
    Image(Url),
}

impl ResourceUrl {
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

pub type ResourceMap = HashMap<Url, Resource>;

#[derive(Debug, PartialEq, Eq)]
pub enum Resource {
    Javascript(String),
    Css(String),
    Image(ImageResource),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ImageResource {
    pub data: Bytes,
    pub mimetype: String,
}

impl ImageResource {
    pub fn to_data_uri(&self) -> String {
        let encoded = base64::encode(&self.data);
        format!("data:{};base64,{}", self.mimetype, encoded)
    }
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

        let resource_urls = parse_resource_urls(&u(), &html).unwrap();

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

        let resource_urls = parse_resource_urls(&u(), &html).unwrap();

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

        let resource_urls = parse_resource_urls(&u(), &html).unwrap();

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

        let resource_urls = parse_resource_urls(&u(), &html).unwrap();

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
        let resource_urls = parse_resource_urls(&u, &html).unwrap();
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

        let resource_urls = parse_resource_urls(&u(), &html).unwrap();

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

        let resource_urls = parse_resource_urls(&u(), &html).unwrap();
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
}
