import os
import pathlib
import struct
from collections import defaultdict

RESOURCE_TYPES = {
    0x0000: "res",  # Misc. GFF resources
    0x0001: "bmp",  # Microsoft Windows Bitmap
    0x0002: "mve",
    0x0003: "tga",  # Targa Graphics Format
    0x0004: "wav",  # Wave
    0x0006: "plt",  # Bioware Packed Layer Texture
    0x0007: "ini",  # Windows INI
    0x0008: "mp3",  # MP3
    0x0009: "mpg",  # MPEG
    0x000A: "txt",  # Text file
    0x000B: "wma",  # Windows Media audio?
    0x000C: "wmv",  # Windows Media video?
    0x000D: "xmv",
    0x07D0: "plh",
    0x07D1: "tex",
    0x07D2: "mdl",  # Model
    0x07D3: "thg",
    0x07D5: "fnt",  # Font
    0x07D7: "lua",
    0x07D8: "slt",
    0x07D9: "nss",  # NWScript source code
    0x07DA: "ncs",  # NWScript bytecode
    0x07DB: "mod",  # Module
    0x07DC: "are",  # Area (GFF)
    0x07DD: "set",  # Tileset (unused in KOTOR?)
    0x07DE: "ifo",  # Module information
    0x07DF: "bic",  # Character sheet (unused)
    0x07E0: "wok",  # walk-mesh
    0x07E1: "2da",  # 2-dimensional array
    0x07E2: "tlk",  # conversation file
    0x07E6: "txi",  # Texture information
    0x07E7: "git",  # Dynamic area information, game instance file, all area and objects that are scriptable
    0x07E8: "bti",
    0x07E9: "uti",  # item blueprint
    0x07EA: "btc",
    0x07EB: "utc",  # Creature blueprint
    0x07ED: "dlg",  # Dialogue
    0x07EE: "itp",  # tile blueprint pallet file
    0x07EF: "btt",
    0x07F0: "utt",  # trigger blueprint
    0x07F1: "dds",  # compressed texture file
    0x07F2: "bts",
    0x07F3: "uts",  # sound blueprint
    0x07F4: "ltr",  # letter combo probability info
    0x07F5: "gff",  # Generic File Format
    0x07F6: "fac",  # faction file
    0x07F7: "bte",
    0x07F8: "ute",  # encounter blueprint
    0x07F9: "btd",
    0x07FA: "utd",  # door blueprint
    0x07FB: "btp",
    0x07FC: "utp",  # placeable object blueprint
    0x07FD: "dft",  # default values file (text-ini)
    0x07FE: "gic",  # game instance comments
    0x07FF: "gui",  # GUI definition (GFF)
    0x0800: "css",
    0x0801: "ccs",
    0x0802: "btm",
    0x0803: "utm",  # store merchant blueprint
    0x0804: "dwk",  # door walkmesh
    0x0805: "pwk",  # placeable object walkmesh
    0x0806: "btg",
    0x0807: "utg",
    0x0808: "jrl",  # Journal
    0x0809: "sav",  # Saved game (ERF)
    0x080A: "utw",  # waypoint blueprint
    0x080B: "4pc",
    0x080C: "ssf",  # sound set file
    0x080D: "hak",  # Hak pak (unused)
    0x080E: "nwm",
    0x080F: "bik",  # movie file (bik format)
    0x0810: "ndb",  # script debugger file
    0x0811: "ptm",  # plot manager/plot instance
    0x0812: "ptt",  # plot wizard blueprint
    0x0BB8: "lyt",
    0x0BB9: "vis",
    0x0BBA: "rim",  # See RIM File Format
    0x0BBB: "pth",  # Path information? (GFF)
    0x0BBC: "lip",
    0x0BBD: "bwm",
    0x0BBE: "txb",
    0x0BBF: "tpc",  # Texture
    0x0BC0: "mdx",
    0x0BC1: "rsv",
    0x0BC2: "sig",
    0x0BC3: "xbx",
    0x270D: "erf",  # Encapsulated Resource Format
    0x270E: "bif",
    0x270F: "key",
}


def new(
    invocant,
    registered_path: pathlib.Path | None,
    bif_ix_filter: int | None = None,
    bif_type_filter: int | None = None,
):
    if registered_path is None:
        try:
            """
                        my  $kotor_key= new Win32::TieRegistry "LMachine/Software/Bioware/SW/Kotor",             #read registry
                            {Access=>Win32::TieRegistry::KEY_READ, Delimiter=>"/"};
            $registered_path= $kotor_key->GetValue("Path")

            """
            # TODO
            registered_path = pathlib.Path.cwd()
        except:
            return

    key_file = registered_path / "chitin.key"

    if not key_file.exists():
        return

    with key_file.open(mode="rb") as key:
        filetype = key.read(4)

        if filetype != b"KEY ":
            return

        key.seek(8, os.SEEK_SET)

        _, keycount, offset_filetable, offset_keytable = struct.unpack(
            "<4L", key.read(16)
        )

        r = defaultdict(str)

        SIZE = 22
        for offset_index in range(keycount):
            key.seek(offset_keytable + offset_index * SIZE, os.SEEK_SET)
            resref, restype, resid = struct.unpack("<16sHL", key.read(16))

            bif_index = resid >> 20

            if bif_ix_filter is not None and bif_index != bif_ix_filter:
                continue

            if (
                bif_type_filter is not None
                and RESOURCE_TYPES[restype] != bif_type_filter
            ):
                continue

            index_in_bif = resid - (bif_index << 20)
            r[RESOURCE_TYPES[restype]] += f" {resref}.{RESOURCE_TYPES[restype]}"

            key.seek(offset_filetable + bif_index * 12, os.SEEK_SET)
            bif_size, bif_name_offset, bif_name_size = struct.unpack(
                "<LLH", key.read(10)
            )

            key.seek(bif_name_offset, os.SEEK_SET)
            name = key.read(bif_name_size)

            # TODO
            """
            $bifhash->{$bif_name}{Resources}{"$resref.$res_types{$restype}"}=
                {'Ix'=>$index_in_bif,
                 'Type_ID'=>$restype,
                  'ID'=>$resid};
            # if ($resref eq "feat") { print "Resource $index_in_bif" . ":\n  Name: $resref.$res_types{$restype}\n  ID: $resid\n\n"; }
            $bifhash->{$bif_name}{Bif_Ix}=$bif_index;
            """

    # TODO
    """
    my $class=ref($invocant)||$invocant;
    my $self={ 'path'=>$registered_path, 'BIFs'=>$bifhash, 'Array'=>\%r};
    bless $self,$class;
    return $self;
    """
