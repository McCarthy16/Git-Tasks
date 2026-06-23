//! Tauri adapter: exposes the server-driven UI as two commands.
//!
//! The frontend never holds routing or domain state. It calls `view` once to
//! learn what to draw, then `dispatch` for every interaction — each returns the
//! freshly-rendered [`View`]. All navigation and data live in [`app::state`].

mod app;
mod error;
mod projects;
mod shared;
mod tasks;

use std::sync::Mutex;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use app::action::Action;
use app::state::AppState;
use app::view::View;

/// The single piece of managed state: the server-side UI state machine.
#[derive(Default)]
struct AppStateLock(Mutex<AppState>);

/// Render the current view (called once on boot).
#[tauri::command]
fn view(state: State<'_, AppStateLock>) -> Result<View, String> {
    state.0.lock().unwrap().render().map_err(|e| e.to_string())
}

/// Apply an action and return the freshly-rendered view.
///
/// This is `async` on purpose: `PickWorkspace` opens a native folder dialog via
/// `blocking_pick_folder`, which must NOT run on the main thread (it needs the
/// main thread's event loop to service the dialog, so calling it there
/// deadlocks/freezes the app). Async commands are scheduled off the main thread.
#[tauri::command]
async fn dispatch(
    app: AppHandle,
    state: State<'_, AppStateLock>,
    action: Action,
) -> Result<View, String> {
    // `PickWorkspace` needs the native folder dialog — a platform concern. We
    // resolve it here (outside the state lock, since it blocks on the user)
    // and hand the app layer only a plain path.
    let picked_folder = match &action {
        Action::PickWorkspace => match app.dialog().file().blocking_pick_folder() {
            Some(folder) => Some(folder.into_path().map_err(|e| e.to_string())?),
            // Cancelled: leave state untouched and report the current view.
            None => return state.0.lock().unwrap().render().map_err(|e| e.to_string()),
        },
        _ => None,
    };

    let mut app_state = state.0.lock().unwrap();
    match action {
        Action::PickWorkspace => app_state
            .open_workspace(picked_folder.expect("folder resolved above"))
            .map_err(|e| e.to_string())?,
        Action::CloseWorkspace => app_state.close_workspace(),
        Action::OpenProject { project_id } => app_state.open_project(project_id),
        Action::CloseProject => app_state.close_project(),
        Action::CreateProject { name } => {
            app_state.create_project(name).map_err(|e| e.to_string())?
        }
        Action::CreateTask { name } => app_state.create_task(name).map_err(|e| e.to_string())?,
    }
    app_state.render().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppStateLock::default())
        .invoke_handler(tauri::generate_handler![view, dispatch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
