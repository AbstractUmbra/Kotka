use binrw::{binrw, BinRead, BinWrite, BinWriterExt};
use eos::DateTime;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::str::FromStr;

use super::shared::RES_TYPES;

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
#[br(little)]
#[derive(Debug, Eq, PartialEq)]
pub struct ErfMetadata {
    localized_string_count: u32,
    localized_string_size: u32,
    entry_count: u32,
    offset_to_localized_string: u32,
    offset_to_key_list: u32,
    offset_to_resource_list: u32,
    build_year: u32,
    build_day: u32,
    description_str_ref: u32,
}

#[binrw]
#[brw(little)]
#[derive(Default, Debug, Eq, PartialEq)]
struct ResourceMetadata {
    offset: u32,
    size: u32,
    #[brw(ignore)]
    new_offset: Option<u32>,
    #[brw(ignore)]
    new_size: Option<u32>,
}

#[binrw]
#[brw(little, import(version: [u8; 4], resource_offset: u32))]
#[derive(Default, Debug, Eq, PartialEq)]
pub struct ErfResource {
    #[br(temp, calc = *version.last().unwrap())]
    #[bw(ignore)]
    version_char: u8,

    #[br(count=if version_char == 48 { 16 } else { 32 }, try_map = |x| String::from_utf8(x).map(|s| s.trim_end_matches('\0').to_owned()))]
    #[bw(map = |s| s.as_bytes().to_vec(), pad_size_to = if version == [0x86, 0x49, 0x46, 0x48] { 16 } else { 32 })]
    reference: String,

    id: u32,
    r#type: u32,
    #[br(seek_before = SeekFrom::Start(resource_offset as u64 + ((id as u64 + 1u64) * 8u64)), restore_position)]
    #[bw(seek_before = SeekFrom::Start(resource_offset as u64 + ((*id as u64 + 1u64) * 8u64)), restore_position)]
    metadata: ResourceMetadata,
    #[br(ignore)]
    data: Option<Vec<u8>>,
    #[brw(ignore)]
    new_data: Option<Vec<u8>>,
    #[brw(ignore)]
    is_new: bool,
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
    metadata: ErfMetadata,
    #[br(seek_before = SeekFrom::Start(metadata.offset_to_localized_string as u64), count=metadata.localized_string_count)]
    localised_strings: Vec<LocalizedString>,
    #[br(seek_before = SeekFrom::Start(metadata.offset_to_key_list as u64), count=metadata.entry_count, args { inner: (version, metadata.offset_to_resource_list) })]
    #[bw(args(*version, metadata.offset_to_key_list))]
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

    pub fn load_file(&mut self, filename: &str) -> &mut Erf<'a> {
        let mut file = Self::open_file(filename).unwrap();

        for resource in self.resources.iter_mut() {
            file.seek(SeekFrom::Start(resource.metadata.offset as u64))
                .unwrap();
            let mut buf = vec![0u8; resource.metadata.size as usize];
            file.read_exact(&mut buf).unwrap();

            resource.data = Some(buf);
            resource.is_new = false
        }

        return self;
    }

    fn recalculate_sizing(&mut self) {
        self.metadata.localized_string_count = self.localised_strings.len() as u32;
        let mut total_string_size: u32 = 0;

        for string in &self.localised_strings {
            total_string_size += 8;
            total_string_size += string.string.len() as u32;
        }

        self.metadata.localized_string_size = total_string_size;
        self.metadata.offset_to_key_list =
            self.metadata.offset_to_localized_string + self.metadata.localized_string_size;

        self.metadata.entry_count = self.resources.len() as u32;

        if self.version.last().unwrap() == &48 {
            // Version v1.0
            self.metadata.offset_to_resource_list =
                self.metadata.offset_to_key_list + (24 * self.metadata.entry_count)
        } else if self.version.last().unwrap() == &49 {
            // Version v1.1
            self.metadata.offset_to_resource_list =
                self.metadata.offset_to_key_list + (40 * self.metadata.entry_count)
        }

        let offset_to_resource_data =
            self.metadata.offset_to_resource_list + (8 * self.metadata.entry_count);

        let mut previous_resource_offset = offset_to_resource_data;
        let mut previous_resource_length = 0;
        let mut previous_resource_offset_memory = offset_to_resource_data;
        let mut previous_resource_length_memory = 0;

        for resource in &mut self.resources {
            resource.metadata.offset = previous_resource_offset + previous_resource_length;
            resource.metadata.new_offset =
                Some(previous_resource_offset_memory + previous_resource_length_memory);

            let resource_data_length = resource
                .data
                .as_mut()
                .expect("No data present in the current ERF file.")
                .len() as u32;

            if resource_data_length > 0 {
                resource.metadata.size = resource_data_length;
                resource.metadata.new_size = Some(resource_data_length);
            }

            let new_resource_data_length = resource
                .new_data
                .as_mut()
                .expect("No new data found on this ERF resource.")
                .len() as u32;

            if new_resource_data_length > 0 {
                resource.metadata.new_size = Some(new_resource_data_length);
            }

            previous_resource_offset = resource.metadata.offset;
            previous_resource_offset_memory = resource
                .metadata
                .new_offset
                .expect("No new offset set just yet.");
            previous_resource_length = resource.metadata.size;
            previous_resource_length_memory = resource
                .metadata
                .new_size
                .expect("No new size set just yet.");
        }
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

    pub fn export(
        &mut self,
        resource: &mut ErfResource,
        output_path: &mut PathBuf,
    ) -> Result<(), binrw::Error> {
        output_path.push(&resource.reference);
        let mut output_file = std::fs::File::create(output_path).unwrap();

        if resource.data.is_some() {
            output_file.write_le(&resource.data)
        } else {
            let mut input_file = std::fs::File::open(&self.filename).unwrap();
            input_file
                .seek(SeekFrom::Start(resource.metadata.offset as u64))
                .unwrap();

            let mut buf = vec![0u8; resource.metadata.size as usize];
            input_file
                .read_exact(&mut buf)
                .expect("Unable to read from the input file.");
            resource.data = Some(buf);

            self.write_erf_data(&mut output_file, true)?;

            Ok(())
        }
    }

    pub fn write_erf_data(
        &mut self,
        output_file: &mut File,
        update_build: bool,
    ) -> Result<(), binrw::Error> {
        if update_build == true {
            let now = DateTime::now().unwrap();

            self.metadata.build_year = now.year() as u32;
            self.metadata.build_day = now.day() as u32;
        }

        self.recalculate_sizing();

        self.write(output_file)
    }
}
