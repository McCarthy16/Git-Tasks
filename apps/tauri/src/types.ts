/**
 * These types mirror the Rust app layer. The frontend is a thin renderer: the
 * backend owns all routing and state, sends a `View` to draw, and accepts an
 * `Action` for every interaction.
 */

export type Project = {
  id: string;
  name: string;
  created_at_millis: number | null;
};

export type Status = {
  id: string;
  name: string;
  kind: "unstarted" | "started" | "complete" | "canceled";
  description: string | null;
  removed: boolean;
  created_at_millis: number | null;
};

export type Task = {
  id: string;
  project_id: string;
  name: string;
  description: string;
  /** null means no status assigned */
  status_id: string | null;
  closed: boolean;
  created_at_millis: number | null;
};

export type TaskEventEntry = {
  id: string;
  kind: string;
  created_at_millis: number | null;
  detail: string | null;
};

export type WorkspaceView = {
  root: string;
  tasks_dir: string;
};

/** The screen to render — mirrors the Rust `View` enum (tagged by `screen`). */
export type View =
  | { screen: "select_repo"; recent_workspaces: string[] }
  | { screen: "projects"; workspace: WorkspaceView; projects: Project[] }
  | {
      screen: "tasks";
      workspace: WorkspaceView;
      project: Project;
      projects: Project[];
      tasks: Task[];
      statuses: Status[];
    }
  | {
      screen: "task_detail";
      workspace: WorkspaceView;
      project: Project;
      projects: Project[];
      task: Task;
      events: TaskEventEntry[];
      statuses: Status[];
    };

/** An intent to send back — mirrors the Rust `Action` enum (tagged by `type`). */
export type Action =
  | { type: "pick_workspace" }
  | { type: "open_workspace"; path: string }
  | { type: "close_workspace" }
  | { type: "open_project"; project_id: string }
  | { type: "close_project" }
  | { type: "create_project"; name: string }
  | { type: "rename_project"; project_id: string; new_name: string }
  | { type: "archive_project"; project_id: string }
  | { type: "create_task"; name: string }
  | { type: "rename_task"; task_id: string; new_name: string }
  | { type: "move_task"; task_id: string; project_id: string }
  | { type: "close_task"; task_id: string }
  | { type: "open_task"; task_id: string }
  | { type: "close_task_detail" }
  | { type: "update_task_description"; task_id: string; description: string }
  | { type: "update_task_description_in_place"; task_id: string; event_id: string; description: string }
  | { type: "rename_task_in_place"; task_id: string; event_id: string; new_name: string }
  | { type: "set_task_status"; task_id: string; status_id: string | null };

/** Dispatch an action, apply the returned view, and return it (null on error). */
export type Dispatch = (action: Action) => Promise<View | null>;
