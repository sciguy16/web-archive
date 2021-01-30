//! Module for the core archiving functionality

use crate::parsing::{Resource, ResourceMap};
use html5ever::{interface::QualName, local_name, namespace_url, ns};
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, NodeData, NodeRef};
use std::io;
use std::path::Path;
use url::Url;

/// Intermediate struct storing the downloaded resources
#[derive(Debug)]
pub struct PageArchive {
    /// Base URL of the page being archived
    pub url: Url,
    /// The content/body of the page
    pub content: String,
    /// A mapping of resource URLs to the downloaded resources
    pub resource_map: ResourceMap,
}

impl PageArchive {
    /// Searches `img`, `link`, and `script` tags in the page body and
    /// substitutes in the downloaded content.
    ///
    /// * Images are base-64 encoded and inserted as `data:` URIs
    /// * Stylesheets are inserted as inline `<style>` tags, replacing
    ///   the `<link>` tags they originated from
    /// * Scripts are inserted into their originating `<script>` tags
    ///   and the original `src` attribute is deleted.
    pub fn embed_resources(&self) -> String {
        // Parse DOM again, and substitute in the downloaded resources

        let document = parse_html().one(self.content.as_str());

        // Replace images
        for element in document.select("img").unwrap() {
            let node = element.as_node();
            if let NodeData::Element(data) = node.data() {
                // node is an 'element'
                let mut attr = data.attributes.borrow_mut();
                if let Some(u) = attr.get_mut("src") {
                    // has a src attribute
                    if let Ok(url) = self.url.join(u) {
                        // The url parses correctly
                        if let Some(Resource::Image(image_data)) =
                            self.resource_map.get(&url)
                        {
                            // We have a stored copy of this resource
                            *u = image_data.to_data_uri();
                        }
                    }
                }
            }
        }

        // Replace CSS
        for element in document.select("link").unwrap() {
            let node = element.as_node();

            // Create a place to store the css data reference so that
            // the horribly nested borrows can be dropped before we
            // replace the `<link>` element with a `<style>`.
            let mut css_data: Option<&String> = None;

            if let NodeData::Element(data) = node.data() {
                // node is an 'element'
                let attr = data.attributes.borrow();
                if Some("stylesheet") == attr.get("rel") {
                    // rel="stylesheet"
                    if let Some(u) = attr.get("href") {
                        // href="style.css"
                        if let Ok(u) = self.url.join(u) {
                            // href parses properly
                            if let Some(Resource::Css(css)) =
                                self.resource_map.get(&u)
                            {
                                // we have a stored copy of the CSS
                                css_data = Some(css);
                            }
                        }
                    }
                }
            }

            if let Some(css) = css_data {
                // CSS data was successfully retrieved by the above steps,
                // so now:
                // * locate the `<link>`'s parent
                // * create a new `<style>` tag containg the CSS
                // * attach it to the parent
                // * delete the original `<link>` tag

                if let Some(parent) = node.parent() {
                    // This probably won't ever fail, but if it does then
                    // ignore it
                    let style = NodeRef::new_element(
                        QualName::new(None, ns!(html), local_name!("style")),
                        None,
                    );
                    style.append(NodeRef::new_text(css));
                    parent.append(style);

                    // Remove the original `<link>` tag
                    node.detach();
                }
            }
        }

        // Replace scripts
        for element in document.select("script").unwrap() {
            let node = element.as_node();
            if let NodeData::Element(data) = node.data() {
                // node is an 'element'
                let mut attr = data.attributes.borrow_mut();
                if let Some(u) = attr.get_mut("src") {
                    // has a src attribute
                    if let Ok(url) = self.url.join(u) {
                        // The url parses correctly
                        if let Some(Resource::Javascript(script_text)) =
                            self.resource_map.get(&url)
                        {
                            // We have a stored copy of this resource
                            node.append(NodeRef::new_text(script_text));
                        }
                    }
                }
                // Remove the original 'src' attribute - doesn't matter
                // whether we managed to archive it or not because
                // external resources won't be reachable from the archived
                // page
                let _ = attr.remove("src");
            }
        }

        document.to_string()
    }

    /// NOT YET IMPLEMENTED
    ///
    /// Write the downloaded resources to disk in the directory specified
    pub fn write_to_disk<P: AsRef<Path>>(
        &self,
        _output_dir: &P,
    ) -> Result<(), io::Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use bytes::Bytes;

    #[test]
    fn test_single_css() {
        let content = r#"
		<html>
			<head>
				<link rel="stylesheet" href="style.css" />
			</head>
			<body></body>
		</html>
		"#
        .to_string();
        let url = Url::parse("http://example.com").unwrap();
        let mut resource_map = ResourceMap::new();
        resource_map.insert(
            url.join("style.css").unwrap(),
            Resource::Css(
                r#"
					body { background-color: blue; }
				"#
                .to_string(),
            ),
        );
        let archive = PageArchive {
            url,
            content,
            resource_map,
        };

        let output = archive.embed_resources();
        assert_eq!(
            output.replace("\t", "").replace("\n", ""),
            r#"
		<html>
			<head>
				<style>
					body { background-color: blue; }
				</style>
			</head>
			<body></body>
		</html>
		"#
            .to_string()
            .replace("\t", "")
            .replace("\n", "")
        );
    }

    #[test]
    fn test_single_image() {
        let content = r#"
		<html>
			<head></head>
			<body>
				<img src="rustacean.png" />
			</body>
		</html>
		"#
        .to_string();
        let url = Url::parse("http://example.com").unwrap();
        let mut resource_map = ResourceMap::new();
        resource_map.insert(
            url.join("rustacean.png").unwrap(),
            Resource::Image(ImageResource {
                data: Bytes::from(
                    include_bytes!(
                        "../dynamic_tests/resources/rustacean-flat-happy.png"
                    )
                    .to_vec(),
                ),
                mimetype: "image/png".to_string(),
            }),
        );
        let archive = PageArchive {
            url,
            content,
            resource_map,
        };

        let output = archive.embed_resources();
        println!("{}", output);
        // base64 < dynamic_tests/resources/rustacean-flat-happy.png
        assert!(output.contains(
            r#"<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAB"#
        ));
        // chunk from middle of image
        assert!(output.contains("gfuBxu3QDwEsoDXx5J5KCU+2/DF2JAQAoDHV"))
    }

    #[test]
    fn test_single_js() {
        let content = r#"
		<html>
			<head>
				<script src="script.js"></script>
			</head>
			<body></body>
		</html>
		"#
        .to_string();
        let url = Url::parse("http://example.com").unwrap();
        let mut resource_map = ResourceMap::new();
        resource_map.insert(
            url.join("script.js").unwrap(),
            Resource::Javascript(
                r#"
					function do_stuff() {
						console.log("Hello!");
					}
				"#
                .to_string(),
            ),
        );
        let archive = PageArchive {
            url,
            content,
            resource_map,
        };

        let output = archive.embed_resources();
        assert_eq!(
            output.replace("\t", "").replace("\n", ""),
            r#"
		<html><head>
				<script>
					function do_stuff() {
						console.log("Hello!");
					}
				</script>
			</head>
			<body></body>
		</html>
		"#
            .to_string()
            .replace("\t", "")
            .replace("\n", "")
        );
    }
}
