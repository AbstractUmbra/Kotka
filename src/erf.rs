use binrw::{binread, binrw, BinRead};
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
#[brw(little, import(version: [u8; 4]))]
#[derive(Debug, Eq, PartialEq)]
struct ErfKey {
    #[br(temp, calc = *version.last().unwrap())]
    #[bw(ignore)]
    version_char: u8,
    #[br(seek_before = SeekFrom::Start(if version_char == 48 { 24u64 } else { 40u64 }), count=if version_char == 48 { 16 } else { 32 })]
    resref: Vec<u8>,
}

#[binrw]
#[brw(little, magic = b"ERF ")]
#[derive(Debug, Eq, PartialEq)]
pub struct Erf {
    version: [u8; 4],
    metadata: InnerErfData,
    #[br(seek_before = SeekFrom::Start(metadata.offset_to_localized_string as u64), count=metadata.localized_string_count)]
    localised_strings: Vec<LocalizedString>,
    #[br(seek_before = SeekFrom::Start(metadata.offset_to_key_list as u64), count=metadata.entry_count, args { inner: (version,) })]
    #[bw(args(*version))]
    keys: Vec<ErfKey>,
}

impl Erf {
    pub fn new(erf_filename: &str) -> Result<Self, binrw::Error> {
        let mut buffer = Self::open_file(erf_filename)?;

        Self::read(&mut buffer)
    }

    fn open_file(filename: &str) -> Result<File, std::io::Error> {
        let path = PathBuf::from_str(filename).expect("Path not found.");

        OpenOptions::new().read(true).open(path)
    }

    pub fn get_resource_id_by_name(self, resource_name: &str) -> Option<String> {
        todo!()
    }
}
