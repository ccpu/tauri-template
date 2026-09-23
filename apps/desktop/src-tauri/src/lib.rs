//! The app itself. `main.rs` only calls [`run`], so the same entry point works
//! for the desktop binary and for the mobile targets.

mod commands;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Opens links and files with the OS default handler, so an external
        // link never navigates the app's own webview. What the frontend is
        // allowed to open is set in capabilities/default.json.
        .plugin(tauri_plugin_opener::init())
        // Asked for by the updater: a dialog to offer the update in, and
        // `process` for the restart that finishes installing it.
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        // Registered unconditionally, but it does nothing until
        // `plugins.updater.pubkey` in tauri.conf.json holds a key -- see
        // src/updater.rs and docs/updater.md.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::app_info
        ])
        .setup(|app| {
            // One-off startup work goes here: tray icons, single-instance
            // checks, background tasks. Returning an error aborts the launch.
            #[cfg(desktop)]
            if updater::is_configured(app.config()) {
                updater::check_on_startup(app.handle());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
