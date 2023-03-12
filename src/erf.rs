use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str::FromStr;

use super::shared::RES_TYPES;
use byte_struct::*;

#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
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

#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
struct BinaryLocalizedString {
    language_id: u32,
    string: u32,
}

struct LocalizedString {
    language_id: u32,
    string: String,
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

    fn build_localized_string_list<'a>(
        &self,
        buffer: &mut BufReader<File>,
        offset: u32,
        buffer_size: u32,
        loop_count: u32,
    ) -> Vec<LocalizedString> {
        buffer.seek(SeekFrom::Start(offset as u64)).unwrap();

        let mut temp_buffer = vec![0; buffer_size as usize];
        buffer.read_exact(&mut temp_buffer).unwrap();

        let mut localised_strings: Vec<LocalizedString> = Vec::new();
        for _ in 0..loop_count {
            // Get the two u32 elements for langugage id and string
            let localised_ = BinaryLocalizedString::read_bytes(&temp_buffer);

            // then we get the string value which is based on the size of the string
            let mut string_buffer = vec![0u8; localised_.string as usize];
            buffer.read_exact(&mut string_buffer).unwrap();
            let string_ = String::from_utf8_lossy(&string_buffer);

            let len = 8 + string_.len();
            let (left, _) = string_.split_at(len);

            let localised_string = LocalizedString {
                language_id: localised_.language_id,
                string: left.to_owned(),
            };

            localised_strings.push(localised_string);
        }
        localised_strings
    }

    pub fn read_erf(&self, erf_filename: &str) {
        let mut buffer = &mut self.open_file(erf_filename);

        let mut sig_buffer = [0u8; 4];
        buffer.read_exact(&mut sig_buffer).unwrap();
        let mut version_buffer = [0u8; 4];
        buffer.read_exact(&mut version_buffer).unwrap();

        let mut inner_erf_buffer = [0u8; 36];
        buffer.read_exact(&mut inner_erf_buffer).unwrap();

        let inner_erf_data = InnerErfData::read_bytes(&inner_erf_buffer);

        let localized_strings = &self.build_localized_string_list(
            &mut buffer,
            inner_erf_data.offset_to_localized_string,
            inner_erf_data.localized_string_size,
            inner_erf_data.localized_string_count,
        );
    }
}
