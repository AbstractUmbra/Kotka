use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::str::FromStr;

use byte_struct::*;
use phf::phf_map;
use std::fs::File;
use tempfile::tempfile;

#[cfg(target_os = "windows")]
use winreg::enums::HKEY_LOCAL_MACHINE;
#[cfg(target_os = "windows")]
use winreg::RegKey;

static RES_TYPES: phf::Map<u16, &'static str> = phf_map! {
    0x0000u16 => "res", 	// Misc. GFF resources
    0x0001u16 => "bmp", 	// Microsoft Windows Bitmap
    0x0002u16 => "mve",
    0x0003u16 => "tga", 	// Targa Graphics Format
    0x0004u16 => "wav", 	// Wave
    0x0006u16 => "plt", 	// Bioware Packed Layer Texture
    0x0007u16 => "ini", 	// Windows INI
    0x0008u16 => "mp3", 	// MP3
    0x0009u16 => "mpg", 	// MPEG
    0x000Au16 => "txt", 	// Text file
    0x000Bu16 => "wma", 	// Windows Media audio?
    0x000Cu16 => "wmv", 	// Windows Media video?
    0x000Du16 => "xmv",
    0x07D0u16 => "plh",
    0x07D1u16 => "tex",
    0x07D2u16 => "mdl", 	// Model
    0x07D3u16 => "thg",
    0x07D5u16 => "fnt", 	// Font
    0x07D7u16 => "lua",
    0x07D8u16 => "slt",
    0x07D9u16 => "nss", 	// NWScript source code
    0x07DAu16 => "ncs", 	// NWScript bytecode
    0x07DBu16 => "mod", 	// Module
    0x07DCu16 => "are", 	// Area (GFF)
    0x07DDu16 => "set", 	// Tileset (unused in KOTOR?)
    0x07DEu16 => "ifo", 	// Module information
    0x07DFu16 => "bic", 	// Character sheet (unused)
    0x07E0u16 => "wok", 	//  walk-mesh
    0x07E1u16 => "2da", 	// 2-dimensional array
    0x07E2u16 => "tlk", 	// conversation file
    0x07E6u16 => "txi", 	// Texture information
    0x07E7u16 => "git", 	// Dynamic area information, game instance file, all area and objects that are scriptable
    0x07E8u16 => "bti",
    0x07E9u16 => "uti", 	// item blueprint
    0x07EAu16 => "btc",
    0x07EBu16 => "utc", 	// Creature blueprint
    0x07EDu16 => "dlg", 	// Dialogue
    0x07EEu16 => "itp", 	// tile blueprint pallet file
    0x07EFu16 => "btt",
    0x07F0u16 => "utt", 	// trigger blueprint
    0x07F1u16 => "dds", 	// compressed texture file
    0x07F2u16 => "bts",
    0x07F3u16 => "uts", 	// sound blueprint
    0x07F4u16 => "ltr", 	// letter combo probability info
    0x07F5u16 => "gff", 	// Generic File Format
    0x07F6u16 => "fac", 	// faction file
    0x07F7u16 => "bte",
    0x07F8u16 => "ute", 	// encounter blueprint
    0x07F9u16 => "btd",
    0x07FAu16 => "utd", 	// door blueprint
    0x07FBu16 => "btp",
    0x07FCu16 => "utp", 	// placeable object blueprint
    0x07FDu16 => "dft", 	// default values file (text-ini)
    0x07FEu16 => "gic", 	// game instance comments
    0x07FFu16 => "gui", 	// GUI definition (GFF)
    0x0800u16 => "css",
    0x0801u16 => "ccs",
    0x0802u16 => "btm",
    0x0803u16 => "utm", 	// store merchant blueprint
    0x0804u16 => "dwk", 	// door walkmesh
    0x0805u16 => "pwk", 	// placeable object walkmesh
    0x0806u16 => "btg",
    0x0807u16 => "utg",
    0x0808u16 => "jrl", 	// Journal
    0x0809u16 => "sav", 	// Saved game (ERF)
    0x080Au16 => "utw", 	// waypoint blueprint
    0x080Bu16 => "4pc",
    0x080Cu16 => "ssf", 	// sound set file
    0x080Du16 => "hak", 	// Hak pak (unused)
    0x080Eu16 => "nwm",
    0x080Fu16 => "bik", 	// movie file (bik format)
    0x0810u16 => "ndb",     // script debugger file
    0x0811u16 => "ptm",     // plot manager/plot instance
    0x0812u16 => "ptt",     // plot wizard blueprint
    0x0BB8u16 => "lyt",
    0x0BB9u16 => "vis",
    0x0BBAu16 => "rim", 	// See RIM File Format
    0x0BBBu16 => "pth", 	// Path information? (GFF)
    0x0BBCu16 => "lip",
    0x0BBDu16 => "bwm",
    0x0BBEu16 => "txb",
    0x0BBFu16 => "tpc", 	// Texture
    0x0BC0u16 => "mdx",
    0x0BC1u16 => "rsv",
    0x0BC2u16 => "sig",
    0x0BC3u16 => "xbx",
    0x270Du16 => "erf", 	// Encapsulated Resource Format
    0x270Eu16 => "bif",
    0x270Fu16 => "key"
};

#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
pub struct Headers {
    bif_count: u32,
    key_count: u32,
    offset_filetable: u32,
    offset_keytable: u32,
}

#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
struct ResourceReference {
    resolved: [u8; 16],
}

#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
struct RawResource {
    reference: ResourceReference,
    type_id: u16,
    id: u32,
}
impl RawResource {
    pub fn name(&self) -> String {
        std::str::from_utf8(&self.reference.resolved)
            .unwrap()
            .trim_matches('\x00')
            .to_owned()
    }
}

#[derive(Debug)]
struct Resource<'a> {
    idx: u32,
    type_id: u16,
    resource_type: &'a &'a str,
}
#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
struct InnerBIFData {
    size: u32,
    name_offset: u32,
    name_size: u16,
}

#[derive(Debug)]
pub struct BIF<'a> {
    path: String,
    bifs: HashMap<String, HashMap<String, HashMap<String, Resource<'a>>>>,
    array: HashMap<&'a &'a str, Vec<String>>,
}

#[derive(ByteStruct, PartialEq, Debug)]
#[byte_struct_le]
struct ExtractedResource {
    offset: u32,
    size: u32,
}

impl BIF<'_> {
    pub fn new(
        installation_path: Option<String>,
        bif_ix_filter: Option<u32>,
        bif_type_filter: &mut Option<String>,
    ) -> Option<Self> {
        // If we don't pass a path, e.g. linux or custom windows install
        // then we wanna return here, since it likely isn't resolved by the reg key.
        let registered_path = installation_path.or(BIF::resolve_windows_registry_key())?;
        let mut path = PathBuf::from_str(&registered_path).expect("Installation path not found.");

        path.push("chitin.key");

        let file = File::open(path).expect("Chitin Key not found or could not be opened.");

        let mut buffer = BIF::validate_chitin_key(&file)?;

        let chitin_headers = BIF::parse_chitin_key_headers(&mut buffer);

        let bif = BIF::parse_chitin_key_body(
            &mut buffer,
            &chitin_headers,
            bif_ix_filter,
            bif_type_filter,
            registered_path,
        );

        Some(bif)
    }

    #[cfg(target_os = "windows")]
    fn resolve_windows_registry_key() -> Option<String> {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey("SOFTWARE//Bioware//SW//Kotor")
            .expect("The primary key does not exist.")
            .get_value("Path")
            .ok()
    }

    #[cfg(not(target_os = "windows"))]
    fn resolve_windows_registry_key() -> Option<String> {
        None
    }

    pub fn parse_chitin_key_headers(file_buffer: &mut BufReader<&File>) -> Headers {
        // move the buffer to the next header
        file_buffer.seek(SeekFrom::Start(8)).ok();

        let mut header_packed = [0u8; 16];
        file_buffer.read_exact(&mut header_packed).unwrap();

        // Read the headers of the chitin key for the necessary data.
        Headers::read_bytes(&header_packed)
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
        headers: &'c Headers,
        bif_ix_filter: Option<u32>,
        bif_type_filter: &'c mut Option<String>,
        registered_path: String,
    ) -> BIF<'a> {
        let mut array: HashMap<&&str, Vec<String>> = HashMap::new();
        let mut bif_hash: HashMap<String, HashMap<String, HashMap<String, Resource>>> =
            HashMap::new();

        let mut top_level_bif_hash: HashMap<String, HashMap<String, Resource>> = HashMap::new();
        top_level_bif_hash.insert("resources".to_owned(), HashMap::new());

        for idx in 0..headers.key_count {
            let mut key_bytes = [0; 22];
            file_buffer
                .seek(SeekFrom::Start(
                    (headers.offset_keytable + (idx * 22)).into(),
                ))
                .unwrap();
            file_buffer.read_exact(&mut key_bytes).unwrap();

            let resource = RawResource::read_bytes(&key_bytes);

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
            let inner_bif = InnerBIFData::read_bytes(&bif_data);

            file_buffer
                .seek(SeekFrom::Start(inner_bif.name_offset as u64))
                .unwrap();
            let mut bif_name_packed = vec![0; inner_bif.name_size as usize];
            file_buffer.read_exact(&mut bif_name_packed).unwrap();
            let bif_name = String::from_utf8_lossy(&bif_name_packed)
                .trim_matches('\x00')
                .to_owned();

            let resource = Resource {
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
            path: registered_path,
            bifs: bif_hash,
            array,
        }
    }

    fn open_bif_file(&self, bif_name: &str) -> BufReader<File> {
        let mut path =
            PathBuf::from_str(&self.path).expect("The data path we're loading doesn't exist.");
        path.push(bif_name);

        let resource_file = File::open(path).expect("Cannot open the located file.");

        BufReader::new(resource_file)
    }

    fn open_resource_file(&self, bif_name: &str, resource_name: String) -> &Resource<'_> {
        let bif_entry = &self.bifs.get(bif_name).unwrap();
        let resource_entry = bif_entry.get("resources").unwrap();
        resource_entry.get(&resource_name).unwrap()
    }

    fn extract_resource(self, bif_name: &str, resource_name: String) -> Result<(), std::io::Error> {
        let mut resource_buf = self.open_bif_file(bif_name);
        let resource = self.open_resource_file(bif_name, resource_name);

        resource_buf
            .seek(SeekFrom::Start((24 + (16 * resource.idx)).into()))
            .expect("Cannot seem to read the resource at this location.");

        let mut temp_resource = [0; 8];
        resource_buf
            .read_exact(&mut temp_resource)
            .expect("Couldn't read into the temporary file buffer.");
        let temp_resource = ExtractedResource::read_bytes(&temp_resource);

        resource_buf
            .seek(SeekFrom::Start(temp_resource.offset as u64))
            .unwrap();

        let mut resource = vec![0; temp_resource.size as usize];
        resource_buf.read_exact(&mut resource).unwrap();

        let mut file = tempfile::tempfile().expect("Unable to create a temporary file.");

        file.write_all(&resource as &[u8])
    }

    fn get_resource(self, bif_name: &str, resource_name: String) -> Vec<u8> {
        let mut bif_reader = self.open_bif_file(bif_name);
        let resource = self.open_resource_file(bif_name, resource_name);

        bif_reader
            .seek(SeekFrom::Start((24 + (16 * resource.idx)).into()))
            .expect("Cannot seem to read the resource at this location.");

        let mut resource_data = [0; 8];
        bif_reader
            .read_exact(&mut resource_data)
            .expect("Unable to read the resource data.");

        let resource = ExtractedResource::read_bytes(&resource_data);

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
