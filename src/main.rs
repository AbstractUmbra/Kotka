#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod bif;
pub mod erf;
mod pathfinder;
mod shared;

use bif::BIF;
use rfd::FileDialog;

fn main() {
    let installation_path = shared::resolve_windows_registry_key()
        .or(FileDialog::new().pick_folder())
        .unwrap();

    println!("{:#?}", installation_path);
}
