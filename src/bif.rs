use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use binrw::{binrw, io::Cursor, BinRead};
use std::fs::File;

use super::shared::RES_TYPES;

#[derive(PartialEq, Debug)]
#[binrw]
#[br(little)]
struct BinaryHeaders {
    bif_count: u32,
    key_count: u32,
    offset_filetable: u32,
    offset_keytable: u32,
}

#[derive(PartialEq, Debug)]
#[binrw]
#[br(little)]
struct BinaryResourceReference {
    resolved: [u8; 16],
}

#[derive(PartialEq, Debug)]
#[binrw]
#[br(little)]
struct BinaryResourceData {
    reference: BinaryResourceReference,
    type_id: u16,
    id: u32,
}
impl BinaryResourceData {
    pub fn name(&self) -> String {
        std::str::from_utf8(&self.reference.resolved)
            .expect("Enable to read the data from the reference.")
            .trim_matches('\x00')
            .to_owned()
    }
}

#[derive(PartialEq, Debug)]
#[binrw]
#[br(little)]
struct BinaryBIFData {
    size: u32,
    name_offset: u32,
    name_size: u16,
}

#[derive(PartialEq, Debug)]
#[binrw]
#[br(little)]
struct BinaryExtractedResource {
    offset: u32,
    size: u32,
}

#[derive(Debug)]
pub struct BIFResource<'a> {
    idx: u32,
    type_id: u16,
    resource_type: &'a &'a str,
}

#[derive(Debug)]
pub struct BIF<'a> {
    path: PathBuf,
    bifs: HashMap<String, HashMap<String, HashMap<String, BIFResource<'a>>>>,
    array: HashMap<&'a &'a str, Vec<String>>,
}

impl BIF<'_> {
    pub fn new(
        installation_path: &mut PathBuf,
        bif_ix_filter: Option<u32>,
        bif_type_filter: &mut Option<String>,
    ) -> Option<Self> {
        // If we don't pass a path, e.g. linux or custom windows install
        // then we wanna return here, since it likely isn't resolved by the reg key.
        installation_path.push("chitin.key");

        let file =
            File::open(&installation_path).expect("Chitin Key not found or could not be opened.");

        let mut buffer = BIF::validate_chitin_key(&file)?;

        let chitin_headers = BIF::parse_chitin_key_headers(&mut buffer);

        let bif = BIF::parse_chitin_key_body(
            &mut buffer,
            &chitin_headers,
            bif_ix_filter,
            bif_type_filter,
            installation_path,
        );

        Some(bif)
    }

    fn parse_chitin_key_headers(file_buffer: &mut BufReader<&File>) -> BinaryHeaders {
        // move the buffer to the next header
        file_buffer.seek(SeekFrom::Start(8)).ok();

        let mut header_packed = [0u8; 16];
        file_buffer.read_exact(&mut header_packed).unwrap();
        let mut header_packed = Cursor::new(header_packed);

        // Read the headers of the chitin key for the necessary data.
        BinaryHeaders::read(&mut header_packed).unwrap()
    }

    fn validate_chitin_key(file: &File) -> Option<BufReader<&File>> {
        let mut header_packed = [0u8; 4];
        let mut key_reader = BufReader::new(file);
        key_reader.read_exact(&mut header_packed).unwrap();

        let header = String::from_utf8_lossy(&header_packed);
        if header != "KEY " {
            return None;
        }

        Some(key_reader)
    }

    fn parse_chitin_key_body<'a, 'c>(
        file_buffer: &'c mut BufReader<&'c File>,
        headers: &'c BinaryHeaders,
        bif_ix_filter: Option<u32>,
        bif_type_filter: &'c mut Option<String>,
        registered_path: &mut PathBuf,
    ) -> BIF<'a> {
        let mut array: HashMap<&&str, Vec<String>> = HashMap::new();
        let mut bif_hash: HashMap<String, HashMap<String, HashMap<String, BIFResource>>> =
            HashMap::new();

        let mut top_level_bif_hash: HashMap<String, HashMap<String, BIFResource>> = HashMap::new();
        top_level_bif_hash.insert("resources".to_owned(), HashMap::new());

        for idx in 0..headers.key_count {
            let mut key_bytes = [0; 22];
            file_buffer
                .seek(SeekFrom::Start(
                    (headers.offset_keytable + (idx * 22)).into(),
                ))
                .unwrap();
            file_buffer.read_exact(&mut key_bytes).unwrap();
            let mut key_bytes = Cursor::new(key_bytes);

            let resource = BinaryResourceData::read(&mut key_bytes).unwrap();

            let bif_index: u32 = resource.id >> 20;

            let resource_type = RES_TYPES.get(&resource.type_id).unwrap();

            if let Some(bif_ix_filter) = bif_ix_filter {
                if bif_index != bif_ix_filter {
                    continue;
                }
            }
            if let Some(bif_type_filter) = bif_type_filter {
                if *resource_type != bif_type_filter.as_mut() {
                    continue;
                }
            }

            let resource_format = format!("{}.{}", resource.name(), resource_type);
            array
                .entry(resource_type)
                .or_default()
                .push(resource_format.to_owned());

            let bif_index_plus_offset: u32 = bif_index * 12;
            file_buffer
                .seek(SeekFrom::Start(
                    (headers.offset_filetable + bif_index_plus_offset).into(),
                ))
                .unwrap();

            let index_in_bif = resource.id - (bif_index << 20);

            let mut bif_data = [0; 10];
            file_buffer.read_exact(&mut bif_data).unwrap();
            let mut bif_data = Cursor::new(bif_data);
            let inner_bif = BinaryBIFData::read(&mut bif_data).unwrap();

            file_buffer
                .seek(SeekFrom::Start(inner_bif.name_offset as u64))
                .unwrap();
            let mut bif_name_packed = vec![0; inner_bif.name_size as usize];
            file_buffer.read_exact(&mut bif_name_packed).unwrap();
            let bif_name = String::from_utf8_lossy(&bif_name_packed)
                .trim_matches('\x00')
                .to_owned();

            let resource = BIFResource {
                idx: index_in_bif,
                type_id: resource.type_id,
                resource_type,
            };

            // bifhash = {bif_name: {"resources": {resource_name_and_ext: Resource}}}
            bif_hash
                .entry(bif_name)
                .or_default()
                .entry("resource".to_owned())
                .or_default()
                .insert(resource_format, resource);
        }
        BIF {
            path: registered_path.to_owned(),
            bifs: bif_hash,
            array,
        }
    }

    fn open_bif_file(&mut self, bif_name: &str) -> BufReader<File> {
        let path = &mut self.path;
        path.push(bif_name);

        let resource_file = File::open(path).expect("Cannot open the located file.");

        BufReader::new(resource_file)
    }

    fn open_resource_file(&self, bif_name: &str, resource_name: String) -> &BIFResource<'_> {
        let bif_entry = &self.bifs.get(bif_name).unwrap();
        let resource_entry = bif_entry.get("resources").unwrap();
        resource_entry.get(&resource_name).unwrap()
    }

    pub fn extract_resource(&mut self, bif_name: &str, resource_name: String) -> File {
        let mut resource_buf = self.open_bif_file(bif_name);
        let resource = self.open_resource_file(bif_name, resource_name);

        resource_buf
            .seek(SeekFrom::Start((24 + (16 * resource.idx)).into()))
            .expect("Cannot seem to read the resource at this location.");

        let mut temp_resource = [0; 8];
        resource_buf
            .read_exact(&mut temp_resource)
            .expect("Couldn't read into the temporary file buffer.");
        let mut temp_resource = Cursor::new(temp_resource);
        let temp_resource = BinaryExtractedResource::read(&mut temp_resource).unwrap();

        resource_buf
            .seek(SeekFrom::Start(temp_resource.offset as u64))
            .unwrap();

        let mut resource = vec![0; temp_resource.size as usize];
        resource_buf.read_exact(&mut resource).unwrap();

        let mut file = tempfile::tempfile().expect("Unable to create a temporary file.");

        file.write_all(&resource as &[u8])
            .expect("Unable to write the temporary resource to the temp file.");

        file
    }

    pub fn get_resource(&mut self, bif_name: &str, resource_name: String) -> Vec<u8> {
        let mut bif_reader = self.open_bif_file(bif_name);
        let resource = self.open_resource_file(bif_name, resource_name);

        bif_reader
            .seek(SeekFrom::Start((24 + (16 * resource.idx)).into()))
            .expect("Cannot seem to read the resource at this location.");

        let mut resource_data = [0; 8];
        bif_reader
            .read_exact(&mut resource_data)
            .expect("Unable to read the resource data.");
        let mut resource_data = Cursor::new(resource_data);

        let resource = BinaryExtractedResource::read(&mut resource_data).unwrap();

        bif_reader
            .seek(SeekFrom::Start(resource.offset as u64))
            .expect("Unable to seek the resource file.");

        let mut resource_data = vec![0; resource.size as usize];
        bif_reader
            .read_exact(&mut resource_data)
            .expect("Unable to read the resource data.");

        resource_data.to_owned()
    }
}
