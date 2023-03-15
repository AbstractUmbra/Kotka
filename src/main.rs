#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod bif;
pub mod erf;
mod pathfinder;
mod shared;
use std::{path::PathBuf, str::FromStr};

use crate::bif::BIF;
use crate::erf::Erf;
use rfd::FileDialog;

fn main() {
    // let mut installation_path = shared::resolve_windows_registry_key()
    //     .or(FileDialog::new().pick_folder())
    //     .unwrap();

    // println!("{:#?}", installation_path);
    let mut installation_path = PathBuf::from_str("example_files/kotor2").unwrap();

    let bif = BIF::new(&mut installation_path, None, &mut None).unwrap();
    println!("{:#?}", bif);

    // let erf = Erf::new("example_files/Game/Modules/001EBO_dlg.erf");

    // println!("{:#?}", erf);
}
