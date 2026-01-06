mod commands;

use commands::audio::RecordingState;
use ringbuf::HeapCons;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub notes_dir: Mutex<PathBuf>,
    pub selected_audio_device: Mutex<Option<String>>,
    pub selected_model: Mutex<String>,
    pub recording: Arc<Mutex<RecordingState>>,
    pub sample_consumer: Arc<Mutex<Option<Arc<Mutex<HeapCons<f32>>>>>>,
}

impl Default for AppState {
    fn default() -> Self {
        let default_dir = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("opnotes");

        Self {
            notes_dir: Mutex::new(default_dir),
            selected_audio_device: Mutex::new(None),
            selected_model: Mutex::new("small.en".to_string()),
            recording: Arc::new(Mutex::new(RecordingState::default())),
            sample_consumer: Arc::new(Mutex::new(None)),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .setup(|app| {
            // Ensure notes directory exists
            let state = app.state::<AppState>();
            let notes_dir = state.notes_dir.lock().unwrap().clone();
            if !notes_dir.exists() {
                std::fs::create_dir_all(&notes_dir).ok();
                // Create default inbox folder
                std::fs::create_dir_all(notes_dir.join("inbox")).ok();
            }

            // Open devtools in release builds for debugging
            #[cfg(feature = "devtools")]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Notes
            commands::notes::list_folders,
            commands::notes::list_notes,
            commands::notes::read_note,
            commands::notes::save_note,
            commands::notes::create_note,
            commands::notes::delete_note,
            commands::notes::create_folder,
            commands::notes::delete_folder,
            commands::notes::rename_folder,
            commands::notes::search_notes,
            // Settings
            commands::settings::get_settings,
            commands::settings::save_settings,
            // Audio
            commands::audio::list_audio_devices,
            commands::audio::get_selected_device,
            commands::audio::set_selected_device,
            commands::audio::start_recording,
            commands::audio::stop_recording,
            commands::audio::cancel_recording,
            commands::audio::is_recording,
            // Whisper
            commands::whisper::list_whisper_models,
            commands::whisper::get_model_status,
            commands::whisper::get_selected_model,
            commands::whisper::set_selected_model,
            commands::whisper::download_model,
            commands::whisper::transcribe,
            commands::whisper::delete_model,
            commands::whisper::check_whisper_cli,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
