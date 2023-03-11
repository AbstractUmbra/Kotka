use super::shared::RES_TYPES;
use byte_struct::*;

pub struct ErfData {}

#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
struct InnerErfData {
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

pub struct Erf {
    erf_filename: &'static str,
}

impl Erf {
    pub fn new(self) -> Self {
        self
    }

    pub fn get_resource_id_by_name(self, resource_name: &str) -> Option<String> {
        todo!()
    }

    pub fn read_erf(self, erf_filename: &str) -> Option<ErfData> {
        todo!()
    }
}
