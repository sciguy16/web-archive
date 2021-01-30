use crate::error::Error;
use crate::parsing::ResourceMap;
use crate::parsing::SRC;
use html5ever::tendril::{Tendril, TendrilSink};
use html5ever::{parse_document, serialize, ParseOpts};
use lazy_static::lazy_static;
use markup5ever::{local_name, Namespace, QualName};
use markup5ever_rcdom::{Handle, NodeData, RcDom, SerializableHandle};

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
        // Parse the DOM again, then walk over it editing the image,
        // script, and link tags to embed their resources
        let mut buf = self.content.as_bytes();
        let parse_opts: ParseOpts = Default::default();

        let mut parsed = parse_document(RcDom::default(), parse_opts)
            .from_utf8()
            .read_from(&mut buf)?;

        walk_and_edit(&self.url, &mut parsed.document, &self.resource_map);

        let doc: SerializableHandle = parsed.document.into();
        let mut output = Vec::new(); //String::new().as_bytes_mut();
        serialize(&mut output, &doc, Default::default())?;

        Ok(String::from_utf8(output)?)
    }

    pub fn write_to_disk<P: AsRef<Path>>(
        &self,
        _output_dir: &P,
    ) -> Result<(), io::Error> {
        todo!()
    }
}

fn walk_and_edit(
    url_base: &Url,
    node: &mut Handle,
    resource_map: &ResourceMap,
) {/*
    match &node.data {
        NodeData::Element { name, attrs, .. } => match name.local {
            local_name!("img") => {
                // <img src="/images/fun.png" />
                for attr in attrs.borrow_mut().iter_mut() {
                    if attr.name == *SRC {
                        // "join" just sets the default base URL to be
                        // `url_base`. If `attr.value` is a fully
                        // qualified URL then that will override the
                        // base
                        if let Ok(u) = url_base.join(&attr.value) {
                            if let Some(data) = resource_map.get(&u) {
                                // Subsititute in the data URL
                                attr.value =
                                    "http://the_game.example.com".into();
                            }
                        }
                    }
                }
            }

            _ => { /* Other element names */ }
        },
        _ => { /* Other node types */ }
    }*/
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Resource;
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

    //#[test]
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
            Resource::Image(Bytes::from(
                include_bytes!(
                    "../dynamic_tests/resources/rustacean-flat-happy.png"
                )
                .to_vec(),
            )),
        );
        let archive = PageArchive {
            url,
            content,
            resource_map,
        };

        let output = archive.embed_resources().unwrap();
        assert_eq!(output, "".to_string());
    }
}
