//! Tauri adapter: exposes the server-driven UI as two commands.
//!
//! The frontend never holds routing or domain state. It calls `view` once to
//! learn what to draw, then `dispatch` for every interaction — each returns the
//! freshly-rendered [`View`]. Navigation lives in [`app::state`]; the data
//! itself lives behind the tasks daemon, reached through [`daemon`].

mod app;
mod daemon;
mod error;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};
#[cfg(target_os = "macos")]
use tauri::Emitter;
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
    let picked_folder = match &action {
        Action::PickWorkspace => match app.dialog().file().blocking_pick_folder() {
            Some(folder) => Some(folder.into_path().map_err(|e| e.to_string())?),
            None => return state.0.lock().unwrap().render().map_err(|e| e.to_string()),
        },
        _ => None,
    };

    let is_workspace_open =
        matches!(&action, Action::PickWorkspace | Action::OpenWorkspace { .. });

    let mut app_state = state.0.lock().unwrap();
    match action {
        Action::PickWorkspace => app_state
            .open_workspace(picked_folder.expect("folder resolved above"))
            .map_err(|e| e.to_string())?,
        Action::OpenWorkspace { path } => app_state
            .open_workspace(PathBuf::from(path))
            .map_err(|e| e.to_string())?,
        Action::CloseWorkspace => app_state.close_workspace(),
        Action::OpenProject { project_id } => app_state.open_project(project_id),
        Action::CloseProject => app_state.close_project(),
        Action::CreateProject { name } => {
            app_state.create_project(name).map_err(|e| e.to_string())?
        }
        Action::RenameProject { project_id, new_name } => app_state
            .rename_project(project_id, new_name)
            .map_err(|e| e.to_string())?,
        Action::ArchiveProject { project_id } => app_state
            .archive_project(project_id)
            .map_err(|e| e.to_string())?,
        Action::RestoreProject { project_id } => app_state
            .restore_project(project_id)
            .map_err(|e| e.to_string())?,
        Action::CreateTask { name } => app_state.create_task(name).map_err(|e| e.to_string())?,
        Action::RenameTask { task_id, new_name } => app_state
            .rename_task(task_id, new_name)
            .map_err(|e| e.to_string())?,
        Action::MoveTask { task_id, project_id } => app_state
            .move_task(task_id, project_id)
            .map_err(|e| e.to_string())?,
        Action::CloseTask { task_id } => {
            app_state.close_task(task_id).map_err(|e| e.to_string())?
        }
        Action::ReopenTask { task_id } => {
            app_state.reopen_task(task_id).map_err(|e| e.to_string())?
        }
        Action::OpenTask { task_id } => app_state.open_task(task_id),
        Action::CloseTaskDetail => app_state.close_task_detail(),
        Action::UpdateTaskDescription { task_id, description } => app_state
            .update_task_description(task_id, description)
            .map_err(|e| e.to_string())?,
        Action::UpdateTaskDescriptionInPlace { task_id, event_id, description } => app_state
            .update_task_description_in_place(task_id, event_id, description)
            .map_err(|e| e.to_string())?,
        Action::RenameTaskInPlace { task_id, event_id, new_name } => app_state
            .rename_task_in_place(task_id, event_id, new_name)
            .map_err(|e| e.to_string())?,
        Action::SetTaskStatus { task_id, status_id } => app_state
            .set_task_status(task_id, status_id)
            .map_err(|e| e.to_string())?,
        Action::CreateStatus { name, kind, description } => app_state
            .create_status(name, kind, description)
            .map_err(|e| e.to_string())?,
        Action::RenameStatus { status_id, new_name } => app_state
            .rename_status(status_id, new_name)
            .map_err(|e| e.to_string())?,
        Action::UpdateStatusDescription { status_id, description } => app_state
            .update_status_description(status_id, description)
            .map_err(|e| e.to_string())?,
        Action::ChangeStatusKind { status_id, new_kind } => app_state
            .change_status_kind(status_id, new_kind)
            .map_err(|e| e.to_string())?,
        Action::RemoveStatus { status_id } => {
            app_state.remove_status(status_id).map_err(|e| e.to_string())?
        }
    }

    let recents = app_state.recent_workspaces_strings();
    let view = app_state.render().map_err(|e| e.to_string())?;
    drop(app_state);

    // Sync OS-level recents (dock menu / Jump List) after a workspace opens.
    if is_workspace_open {
        update_os_recents(&app, recents);
    }

    Ok(view)
}

// ---------------------------------------------------------------------------
// OS-level recent workspace integration
// ---------------------------------------------------------------------------

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn basename(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

#[allow(unused_variables)]
fn update_os_recents(app: &AppHandle, recents: Vec<String>) {
    #[cfg(target_os = "macos")]
    update_dock_menu(app, &recents);
    #[cfg(target_os = "windows")]
    update_jump_list(&recents);
}

// --- macOS: dock right-click menu ------------------------------------------

#[cfg(target_os = "macos")]
fn update_dock_menu(app: &AppHandle, recents: &[String]) {
    let _ = try_update_dock_menu(app, recents);
}

#[cfg(target_os = "macos")]
fn try_update_dock_menu(app: &AppHandle, recents: &[String]) -> tauri::Result<()> {
    use tauri::menu::{MenuBuilder, MenuItemBuilder};
    let mut builder = MenuBuilder::new(app);
    for path in recents {
        let id = format!("recent:{path}");
        let label = basename(path);
        if let Ok(item) = MenuItemBuilder::with_id(id, label).build(app) {
            builder = builder.item(&item);
        }
    }
    let menu = builder.build()?;
    app.set_dock_menu(Some(menu))?;
    Ok(())
}

// --- Windows: taskbar Jump List --------------------------------------------

#[cfg(target_os = "windows")]
fn update_jump_list(recents: &[String]) {
    let _ = try_update_jump_list(recents);
}

#[cfg(target_os = "windows")]
fn try_update_jump_list(recents: &[String]) -> windows::core::Result<()> {
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
        CoTaskMemAlloc,
    };
    use windows::Win32::System::Com::StructuredStorage::{PropVariantClear, PROPVARIANT};
    use windows::Win32::System::Ole::VT_LPWSTR;
    use windows::Win32::UI::Shell::{
        DestinationList, EnumerableObjectCollection, ICustomDestinationList, IObjectArray,
        IObjectCollection, IShellLinkW, ShellLink,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, PKEY_Title};
    use windows::core::{HSTRING, PCWSTR, PWSTR};

    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok();

        let list: ICustomDestinationList =
            CoCreateInstance(&DestinationList, None, CLSCTX_INPROC_SERVER)?;
        let mut slots = 0u32;
        let _removed: IObjectArray = list.BeginList(&mut slots)?;

        let collection: IObjectCollection =
            CoCreateInstance(&EnumerableObjectCollection, None, CLSCTX_INPROC_SERVER)?;

        let exe = std::env::current_exe().unwrap_or_default();
        let exe_hstr = HSTRING::from(exe.to_string_lossy().as_ref());

        for path in recents {
            let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

            link.SetPath(PCWSTR(exe_hstr.as_ptr()))?;
            let args_hstr = HSTRING::from(format!("--workspace {path}").as_str());
            link.SetArguments(PCWSTR(args_hstr.as_ptr()))?;
            link.SetIconLocation(PCWSTR(exe_hstr.as_ptr()), 0)?;

            // Set the jump list display title via IPropertyStore / PKEY_Title.
            let title = basename(path);
            let title_wide: Vec<u16> =
                title.encode_utf16().chain(std::iter::once(0)).collect();
            let buf = CoTaskMemAlloc(title_wide.len() * 2) as *mut u16;
            if !buf.is_null() {
                std::ptr::copy_nonoverlapping(title_wide.as_ptr(), buf, title_wide.len());
                let mut pv: PROPVARIANT = std::mem::zeroed();
                pv.Anonymous.Anonymous.vt = VT_LPWSTR;
                pv.Anonymous.Anonymous.Anonymous.pwszVal = PWSTR(buf);
                let store: IPropertyStore = link.cast()?;
                store.SetValue(&PKEY_Title, &pv)?;
                store.Commit()?;
                PropVariantClear(&mut pv)?; // frees the CoTaskMem buffer
            }

            collection.AddObject(&link)?;
        }

        let array: IObjectArray = collection.cast()?;
        list.AddUserTasks(&array)?;
        list.CommitList()?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let recents_file = app
                .path()
                .app_data_dir()
                .map(|d| d.join("recents.json"))
                .unwrap_or_else(|_| PathBuf::from("recents.json"));
            let mut state = AppState::with_recents_file(recents_file);

            // Windows Jump List items launch: tasks.exe --workspace /path
            // Any other launch: auto-open the most recent workspace.
            if let Some(path) = workspace_from_args() {
                let _ = state.open_workspace(path);
            } else {
                state.try_open_most_recent();
            }

            let recents = state.recent_workspaces_strings();
            app.manage(AppStateLock(Mutex::new(state)));

            // Set initial OS recent lists.
            let handle = app.handle().clone();
            update_os_recents(&handle, recents);

            // macOS dock menu clicks open the workspace in the current instance
            // and push "view-updated" so the UI re-fetches the current view.
            #[cfg(target_os = "macos")]
            app.on_menu_event({
                let handle = app.handle().clone();
                move |_src, event| {
                    let id = event.id().0.as_str();
                    if let Some(path) = id.strip_prefix("recent:") {
                        let (ok, recents) = {
                            let lock = handle.state::<AppStateLock>();
                            let mut s = lock.0.lock().unwrap();
                            let ok = s.open_workspace(PathBuf::from(path)).is_ok();
                            (ok, s.recent_workspaces_strings())
                        };
                        if ok {
                            let _ = handle.emit("view-updated", ());
                            update_dock_menu(&handle, &recents);
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![view, dispatch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Extract `--workspace /path` or `--workspace=/path` from argv.
/// Used when the app is launched by a Windows Jump List item.
fn workspace_from_args() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len().saturating_sub(1) {
        if args[i] == "--workspace" {
            return Some(PathBuf::from(&args[i + 1]));
        }
    }
    for arg in &args {
        if let Some(path) = arg.strip_prefix("--workspace=") {
            return Some(PathBuf::from(path));
        }
    }
    None
}
