use crate::error::Error;
use crate::parsing::{Resource, ResourceMap};
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, NodeData, NodeRef};
use std::io;
use std::path::Path;
use url::Url;

#[derive(Debug)]
pub struct PageArchive {
    pub url: Url,
    pub content: String,
    pub resource_map: ResourceMap,
}

impl PageArchive {
    pub fn embed_resources(&self) -> Result<String, Error> {
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
                // Remove the original 'src' attribute
                let _ = attr.remove("src");
            }
        }

        Ok(document.to_string())
    }

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

    //#[test]
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

        let output = archive.embed_resources().unwrap();
        assert_eq!(output, "".to_string());
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

        let output = archive.embed_resources().unwrap();
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

        let output = archive.embed_resources().unwrap();
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
