mod commands;
mod orchestrator;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::convert::get_files_info,
            commands::convert::get_output_formats,
            commands::convert::convert_files,
            commands::convert::scan_directory,
            commands::convert::open_path,
            commands::convert::get_available_features,
            commands::context_menu::register_context_menu,
            commands::context_menu::unregister_context_menu,
            commands::context_menu::is_context_menu_registered,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
