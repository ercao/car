mod command;

use command::connect;
use tauri::Manager;

pub fn entry() {
  let mut builder =
    tauri::Builder::default().plugin(tauri_plugin_shell::init()).invoke_handler(tauri::generate_handler![connect]);

  #[cfg(debug_assertions)]
  {
    builder = builder.plugin(tauri_plugin_devtools::init());
  }

  builder
    .setup(|app| {
      #[cfg(debug_assertions)]
      {
        let window = app.get_webview_window("main").unwrap();
        window.open_devtools();
        window.close_devtools();
      }

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  entry()
}
