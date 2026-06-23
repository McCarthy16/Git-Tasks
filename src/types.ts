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

export type Task = {
  id: string;
  project_id: string;
  name: string;
  created_at_millis: number | null;
};

export type WorkspaceView = {
  root: string;
  tasks_dir: string;
};

/** The screen to render — mirrors the Rust `View` enum (tagged by `screen`). */
export type View =
  | { screen: "select_repo" }
  | { screen: "projects"; workspace: WorkspaceView; projects: Project[] }
  | {
      screen: "tasks";
      workspace: WorkspaceView;
      project: Project;
      tasks: Task[];
    };

/** An intent to send back — mirrors the Rust `Action` enum (tagged by `type`). */
export type Action =
  | { type: "pick_workspace" }
  | { type: "close_workspace" }
  | { type: "open_project"; project_id: string }
  | { type: "close_project" }
  | { type: "create_project"; name: string }
  | { type: "create_task"; name: string };

/** Dispatch an action and apply the returned view. */
export type Dispatch = (action: Action) => Promise<void>;
