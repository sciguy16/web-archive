use crate::parsing::ResourceMap;
use std::io;
use std::path::Path;

#[derive(Debug)]
pub struct PageArchive {
    pub content: String,
    pub resource_map: ResourceMap,
}

impl PageArchive {
    pub fn write_to_disk<P: AsRef<Path>>(
        &self,
        _output_dir: &P,
    ) -> Result<(), io::Error> {
        todo!()
    }
}
