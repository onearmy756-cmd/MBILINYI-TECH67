// Prevents a console window from opening alongside the app on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! The desktop shell. It opens the live platform in a native window, so the
//! desktop app and the website are never two different products — an update to
//! the platform is an update to every installed copy.

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("the Mbilinyi Tech desktop window failed to start");
}
