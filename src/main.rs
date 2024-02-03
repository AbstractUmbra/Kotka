#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod bif;
mod erf;
mod error;
mod shared;
mod tpc;
mod twoda;

use bif::Bif;
use erf::Erf;
use std::{path::PathBuf, str::FromStr};
// use tlk::Tlk;
pub use error::{Error, Result};
use tpc::Tpc;
use twoda::TwoDA;

fn main() {
    // let mut installation_path = shared::resolve_windows_registry_key()
    //     .or(FileDialog::new().pick_folder())
    //     .unwrap();

    // let mut installation_path = PathBuf::from_str("example_files/kotor2").unwrap();

    // let bif = Bif::new(&mut installation_path, None, None).unwrap();
    // println!("{:#?}", bif);

    let erf = Erf::new("example_files/kotor/patch.erf");
    println!("{:#?}", erf);

    // let tlk = Tlk::new("").unwrap();
    // println!("{:#?}", tlk);
}
