use binrw::{binrw, io::Cursor, BinRead};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str::FromStr;

use super::shared::RES_TYPES;

#[binrw]
#[br(little)]
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
struct BinaryLocalizedString {
    language_id: u32,
    #[br(count=language_id)]
    string: Vec<u8>,
}

impl BinaryLocalizedString {
    pub fn resolve_string(&self) -> String {
        let mut string: String =
            String::from_utf8_lossy(&self.string[0..(self.string.len() - 1)]).into_owned();

        string.split_off(8 + string.len())
    }
}

pub struct Erf {
    erf_filename: String,
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

impl Erf {
    pub fn new(self) -> Self {
        self
    }

    fn open_file(&self, filename: &str) -> BufReader<File> {
        let path = PathBuf::from_str(filename).expect("Path not found.");

        let file = File::open(path).expect("Could not open file.");

        BufReader::new(file)
    }

    pub fn get_resource_id_by_name(self, resource_name: &str) -> Option<String> {
        todo!()
    }

    fn build_localized_string_list(
        &self,
        buffer: &mut BufReader<File>,
        offset: u32,
        buffer_size: u32,
        loop_count: u32,
    ) -> Vec<BinaryLocalizedString> {
        buffer.seek(SeekFrom::Start(offset as u64)).unwrap();

        let mut internal_buffer = vec![0; buffer_size as usize];
        buffer.read_exact(&mut internal_buffer).unwrap();
        let mut temp_buffer = Cursor::new(internal_buffer);

        let mut localised_strings: Vec<BinaryLocalizedString> = Vec::new();
        for _ in 0..loop_count {
            // Get the two u32 elements for langugage id and string
            let localised_ = BinaryLocalizedString::read(&mut temp_buffer).unwrap();

            localised_strings.push(localised_);
        }
        localised_strings
    }

    pub fn read_erf(&self, erf_filename: &str) {
        let buffer = &mut self.open_file(erf_filename);

        let mut sig_buffer = [0u8; 4];
        buffer.read_exact(&mut sig_buffer).unwrap();
        let mut version_buffer = [0u8; 4];
        buffer.read_exact(&mut version_buffer).unwrap();

        let mut inner_erf_buffer = [0u8; 36];
        buffer.read_exact(&mut inner_erf_buffer).unwrap();

        let mut inner_erf_buffer = Cursor::new(inner_erf_buffer);

        let inner_erf_data = InnerErfData::read(&mut inner_erf_buffer).unwrap();

        let localized_strings = &self.build_localized_string_list(
            buffer,
            inner_erf_data.offset_to_localized_string,
            inner_erf_data.localized_string_size,
            inner_erf_data.localized_string_count,
        );
    }
}
