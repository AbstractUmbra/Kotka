use binrw::BinRead;
use phf::phf_map;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winreg::enums::HKEY_LOCAL_MACHINE;
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub static RES_TYPES: phf::Map<u16, &'static str> = phf_map! {
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

#[binrw::parser(reader)]
pub fn parse_padded_string<const SIZE: usize, T: for<'a> From<&'a str>>() -> binrw::BinResult<T> {
    let pos = reader.stream_position()?;
    <[u8; SIZE]>::read(reader).and_then(|bytes| {
        std::str::from_utf8(&bytes)
            .map(|s| s.trim_end_matches('\0').into())
            .map_err(|err| binrw::Error::Custom {
                pos,
                err: Box::new(err),
            })
    })
}

#[cfg(target_os = "windows")]
pub fn resolve_windows_registry_key() -> Option<PathBuf> {
    use std::str::FromStr;

    let path: Option<String> = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE//Bioware//SW//Kotor")
        .ok()?
        .get_value("Path")
        .ok();

    match path {
        Some(path) => PathBuf::from_str(path.as_ref()).ok(),
        None => None,
    }
}

#[cfg(not(target_os = "windows"))]
pub fn resolve_windows_registry_key() -> Option<PathBuf> {
    None
}
