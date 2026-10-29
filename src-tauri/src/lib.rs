#[cfg(target_os = "android")]
mod android;
#[cfg(not(target_os = "android"))]
mod desktop;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "android")]
    {
        tauri::Builder::default()
            .plugin(android::plugin())
            .invoke_handler(tauri::generate_handler![
                android::dependencies,
                android::latest_release,
                android::preview,
                android::choose_directory,
                android::open_directory,
                android::start_download,
                android::cancel_download
            ])
            .run(tauri::generate_context!())
            .expect("error running Downpour on Android");
    }
    #[cfg(not(target_os = "android"))]
    desktop::run();
}
