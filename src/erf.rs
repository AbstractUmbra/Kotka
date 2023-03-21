use binrw::{binrw, BinRead, BinReaderExt, BinWriterExt};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, SeekFrom};
use std::path::PathBuf;
use std::str::FromStr;

use super::shared::RES_TYPES;

#[binrw]
#[br(little)]
#[derive(Debug, Eq, PartialEq)]
pub struct InnerErfData {
    localized_string_count: u32,
    localized_string_size: u32,
    entry_count: u32,
    offset_to_localized_string: u32,
    offset_to_key_list: u32,
    offset_to_resource_list: u32,
    build_year: u32,
    build_name: u32,
    description_str_ref: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Eq, PartialEq)]
struct LocalizedString {
    language_id: u32,
    #[br(count=language_id)]
    string: Vec<u8>,
}

impl LocalizedString {
    pub fn resolve_string(&self) -> String {
        let mut string: String =
            String::from_utf8_lossy(&self.string[0..(self.string.len() - 1)]).into_owned();

        string.split_off(8 + string.len())
    }
}

#[binrw]
#[brw(little)]
#[derive(Default, Debug, Eq, PartialEq)]
struct InnerErfResource {}

#[binrw]
#[brw(little, import(version: [u8; 4], resource_offset: u32))]
#[derive(Default, Debug, Eq, PartialEq)]
struct ErfResource {
    #[br(temp, calc = *version.last().unwrap())]
    #[bw(ignore)]
    version_char: u8,

    #[br(count=if version_char == 48 { 16 } else { 32 }, try_map = String::from_utf8)]
    #[bw(map = |s| s.as_bytes().to_vec())]
    reference: String,

    id: u32,
    r#type: u32,
    #[br(seek_before = SeekFrom::Start(resource_offset as u64), restore_position)]
    offset: u32,
    #[br(restore_position)]
    size: u32,
    #[br(ignore)]
    data: Option<InnerErfResource>,
}

impl ErfResource {
    pub fn get_resource_type<'a>(&self) -> &'a str {
        RES_TYPES[&(self.r#type as u16)]
    }
}

#[binrw]
#[brw(little, magic = b"ERF ")]
#[derive(Debug, Eq, PartialEq)]
pub struct Erf<'a> {
    #[brw(ignore)]
    filename: &'a str,
    version: [u8; 4],
    metadata: InnerErfData,
    #[br(seek_before = SeekFrom::Start(metadata.offset_to_localized_string as u64), count=metadata.localized_string_count)]
    localised_strings: Vec<LocalizedString>,
    #[br(seek_before = SeekFrom::Start(metadata.offset_to_key_list as u64), count=metadata.entry_count, args { inner: (version, metadata.offset_to_resource_list) })]
    #[bw(args(*version, metadata.offset_to_resource_list))]
    resources: Vec<ErfResource>,
    // files: Vec<String> // This is actually just filename + file ext
}

impl<'a> Erf<'a> {
    pub fn new(erf_filename: &'a str) -> Self {
        let mut buffer = Self::open_file(erf_filename).unwrap();

        let mut self_return = Self::read(&mut buffer).unwrap();
        self_return.filename = erf_filename;

        return self_return;
    }

    fn open_file(filename: &str) -> Result<File, std::io::Error> {
        let path = PathBuf::from_str(filename).expect("Path not found.");

        OpenOptions::new().read(true).open(path)
    }

    pub fn get_resources_by_type(self, resource_type: &str) -> Vec<u32> {
        let mut resources: Vec<u32> = Vec::new();

        for key in self.resources.iter() {
            if key.get_resource_type() == resource_type {
                resources.push(key.id)
            }
        }

        resources
    }

    pub fn get_resource_id_by_name(&self, resource_name: &str) -> Option<u32> {
        self.resources
            .iter()
            .find(|key| format!("{}.{}", key.reference, key.get_resource_type()) == resource_name)
            .map(|key| key.id)
    }

    pub fn get_resource_id_by_type(&self, resource_type: &str) -> Option<u32> {
        self.resources
            .iter()
            .find(|key| key.get_resource_type() == resource_type)
            .map(|key| key.id)
    }

    fn get_resource_by_id(&self, resource_id: u32) -> Option<&ErfResource> {
        self.resources
            .iter()
            .find(|key| key.id == resource_id)
            .map(|key| key.to_owned())
    }

    // pub fn export(
    //     self,
    //     resource: &ErfResource,
    //     output_path: &mut PathBuf,
    // ) -> Result<(), binrw::Error> {
    //     output_path.push(&resource.reference);
    //     let mut output_file = std::fs::File::create(output_path).unwrap();

    //     if resource.data.is_some() {
    //         output_file.write_le(&resource.data)
    //     } else {
    //         let mut input_file = std::fs::File::open(&self.filename).unwrap();
    //         let mut buffer: Vec<u8> = vec![0u8; resource.size as usize];

    //         let data = input_file.read_le_args(args)
    //     }
    // }
}
