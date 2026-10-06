// Mencegah jendela konsol tambahan di Windows saat build rilis.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ziyadah_lib::run()
}
