use crate::error::Error;
use bytes::Bytes;
use html5ever::tendril::{Tendril, TendrilSink};
use html5ever::{parse_document, ParseOpts};
use markup5ever::{local_name, Namespace, QualName};
use markup5ever_rcdom::{Handle, NodeData, RcDom};
use std::collections::HashMap;
use url::Url;

pub(crate) fn parse_resource_urls(
    url_base: &Url,
    page: &str,
) -> Result<Vec<ResourceUrl>, Error> {
    let mut buf = page.as_bytes();

    let parse_opts: ParseOpts = Default::default();

    let parsed = parse_document(RcDom::default(), parse_opts)
        .from_utf8()
        .read_from(&mut buf)?;

    // Recursively walk the DOM, collecting any supported resource URLs
    let resource_urls = walk_dom(&url_base, &parsed.document);

    Ok(resource_urls)
}

fn walk_dom(url_base: &Url, node: &Handle) -> Vec<ResourceUrl> {
    // prepare a vec to collect the data
    let mut resource_urls = Vec::new();

    // Determine what type of node it is
    match &node.data {
        NodeData::Element { name, attrs, .. } => match name.local {
            local_name!("img") => {
                // <img src="/images/fun.png" />
                for attr in attrs.borrow().iter() {
                    let src = QualName::new(
                        None,
                        Namespace::from(""),
                        local_name!("src"),
                    );
                    if attr.name == src {
                        // "join" just sets the default base URL to be
                        // `url_base`. If `attr.value` is a fully
                        // qualified URL then that will override the
                        // base
                        if let Ok(u) = url_base.join(&attr.value) {
                            // Only save URLs that parse properly
                            resource_urls.push(ResourceUrl::Image(u));
                        }
                    }
                }
            }
            local_name!("script") => {
                // <script language="javascript" src="/js.js"></script>
                for attr in attrs.borrow().iter() {
                    let src = QualName::new(
                        None,
                        Namespace::from(""),
                        local_name!("src"),
                    );
                    if attr.name == src {
                        // "join" just sets the default base URL to be
                        // `url_base`. If `attr.value` is a fully
                        // qualified URL then that will override the
                        // base
                        if let Ok(u) = url_base.join(&attr.value) {
                            // Only save URLs that parse properly
                            resource_urls.push(ResourceUrl::Javascript(u));
                        }
                    }
                }
            }
            local_name!("link") => {
                // <link rel="stylesheet" type="text/css" href="/style.css" />
                // Probably need to check that `rel == stylesheet` before
                // committing to storing the URL
                let mut is_stylesheet = false;
                let mut url: Option<Url> = None;
                for attr in attrs.borrow().iter() {
                    let rel = QualName::new(
                        None,
                        Namespace::from(""),
                        local_name!("rel"),
                    );
                    let href: QualName = QualName::new(
                        None,
                        Namespace::from(""),
                        local_name!("href"),
                    );
                    if attr.name == href {
                        // "join" just sets the default base URL to be
                        // `url_base`. If `attr.value` is a fully
                        // qualified URL then that will override the
                        // base
                        if let Ok(u) = url_base.join(&attr.value) {
                            url = Some(u);
                        }
                    } else if attr.name == rel {
                        if attr.value == Tendril::from("stylesheet") {
                            is_stylesheet = true;
                        }
                    }
                }

                if is_stylesheet {
                    if let Some(u) = url {
                        resource_urls.push(ResourceUrl::Css(u));
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }

    for child in
        node.children
            .borrow()
            .iter()
            .filter(|child| match child.data {
                NodeData::Text { .. } | NodeData::Element { .. } => true,
                _ => false,
            })
    {
        resource_urls.append(&mut walk_dom(&url_base, &child));
    }

    resource_urls
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceUrl {
    Javascript(Url),
    Css(Url),
    Image(Url),
}

pub type ResourceMap = HashMap<Url, Resource>;

#[derive(Debug)]
pub enum Resource {
    Javascript(String),
    Css(String),
    Image(Bytes),
}

#[cfg(test)]
mod test {
    use super::*;

    fn u() -> Url {
        Url::parse("http://example.com").unwrap()
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

        assert_eq!(resource_urls.len(), 5);
        assert_eq!(
            resource_urls[0],
            ResourceUrl::Javascript(
                Url::parse("http://example.com/js.js").unwrap()
            )
        );
        assert_eq!(
            resource_urls[1],
            ResourceUrl::Css(Url::parse("http://example.com/1.css").unwrap())
        );
        assert_eq!(
            resource_urls[2],
            ResourceUrl::Image(Url::parse("http://example.com/1.png").unwrap())
        );
        assert_eq!(
            resource_urls[3],
            ResourceUrl::Javascript(
                Url::parse("http://example.com/2.js").unwrap()
            )
        );
        assert_eq!(
            resource_urls[4],
            ResourceUrl::Image(
                Url::parse("http://example.com/2.tiff").unwrap()
            )
        );
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

        assert_eq!(resource_urls.len(), 3);
        assert_eq!(
            resource_urls[0],
            ResourceUrl::Image(
                Url::parse("http://example.com/one/two/images/fun.png")
                    .unwrap()
            )
        );
        assert_eq!(
            resource_urls[1],
            ResourceUrl::Image(
                Url::parse("http://example.com/absolute_path.jpg").unwrap()
            )
        );
        assert_eq!(
            resource_urls[2],
            ResourceUrl::Image(
                Url::parse(
                    "https://www.rust-lang.org/static/images/rust-logo-blk.svg"
                )
                .unwrap()
            )
        );
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

        assert_eq!(resource_urls.len(), 2);
        assert_eq!(
            resource_urls[0],
            ResourceUrl::Javascript(
                Url::parse("http://example.com/js.js").unwrap()
            )
        );
        assert_eq!(
            resource_urls[1],
            ResourceUrl::Image(Url::parse("http://example.com/a.jpg").unwrap())
        );
    }
}
