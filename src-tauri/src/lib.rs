use tauri::Manager;

mod commands;
mod claude;
mod context;
mod kanban;
mod mcp;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // Initialize database
            tauri::async_runtime::spawn(async move {
                if let Err(e) = kanban::database::init_db().await {
                    eprintln!("Failed to initialize database: {}", e);
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::context_commands::load_context,
            commands::context_commands::save_context,
            commands::context_commands::generate_claude_md,
            commands::context_commands::list_md_files,
            commands::claude_commands::start_claude_session,
            commands::claude_commands::execute_task,
            commands::claude_commands::get_session_status,
            commands::kanban_commands::get_tasks,
            commands::kanban_commands::create_task,
            commands::kanban_commands::update_task,
            commands::kanban_commands::delete_task,
            commands::kanban_commands::sync_with_markdown,
            commands::mcp_commands::list_servers,
            commands::mcp_commands::toggle_server,
            commands::mcp_commands::create_hook,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
