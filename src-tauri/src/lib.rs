pub mod commands;
pub mod pipeline;
pub mod recapper;
pub mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::archive::scan_archive,
            commands::archive::extract_zip,
            commands::toolkit::start_toolkit,
            commands::toolkit::cancel_toolkit,
            commands::recapper::start_recapper,
            commands::recapper::cancel_recapper,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::settings::reset_settings,
            commands::system::check_ffmpeg,
            commands::system::list_system_fonts,
            commands::system::cancel_job,
            commands::system::list_active_jobs,
            commands::system::check_offline_geodb,
            commands::system::download_offline_geodb,
            commands::system::set_active_geodb_tier,
            commands::system::delete_offline_geodb,
            commands::debug::export_debug_log,
            commands::debug::get_debug_logs,
            commands::debug::clear_debug_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BeReal Studio application");
}
