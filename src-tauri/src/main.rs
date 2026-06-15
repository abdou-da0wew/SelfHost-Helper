#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    selfhost_helper_lib::run();
}
