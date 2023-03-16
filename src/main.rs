#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod bif;
pub mod erf;
mod shared;

fn main() {
    println!("Hello!")
}
