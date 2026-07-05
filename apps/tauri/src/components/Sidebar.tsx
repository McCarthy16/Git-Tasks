import { useState } from "react";
import type { Dispatch, Project, WorkspaceView } from "../types";
import { basename } from "../path";
import { CreateProjectDialog } from "./CreateProjectDialog";
import "./sidebar.css";

/**
 * The persistent navigation rail shown on every workspace screen: Home, the
 * project list, a "New project" CTA, and a footer that returns to the repo
 * picker. Selection state lives server-side — clicks just dispatch actions.
 */
export function Sidebar({
  workspace,
  projects,
  activeProjectId,
  dispatch,
}: {
  workspace: WorkspaceView;
  projects: Project[];
  /** The open project, or null when Home is active. */
  activeProjectId: string | null;
  dispatch: Dispatch;
}) {
  const [creating, setCreating] = useState(false);

  return (
    <aside className="sidebar">
      <nav className="sidebar__nav">
        <p className="sidebar__repo">{basename(workspace.root)}</p>
        <button
          className={
            "sidebar__item" + (activeProjectId === null ? " sidebar__item--active" : "")
          }
          onClick={() => dispatch({ type: "close_project" })}
        >
          <HomeIcon />
          <span className="sidebar__item-label">Home</span>
        </button>

        <p className="sidebar__section">Projects</p>
        {projects.map((project) => (
          <button
            key={project.id}
            className={
              "sidebar__item" +
              (project.id === activeProjectId ? " sidebar__item--active" : "")
            }
            onClick={() => dispatch({ type: "open_project", project_id: project.id })}
          >
            <span className="sidebar__dot" aria-hidden="true" />
            <span className="sidebar__item-label">{project.name}</span>
          </button>
        ))}
        <button className="sidebar__item sidebar__item--cta" onClick={() => setCreating(true)}>
          <span className="sidebar__plus" aria-hidden="true">
            +
          </span>
          <span className="sidebar__item-label">New project</span>
        </button>
      </nav>

      <footer className="sidebar__footer">
        <button
          className="sidebar__workspace"
          onClick={() => dispatch({ type: "close_workspace" })}
        >
          Change Repo
        </button>
      </footer>

      {creating && (
        <CreateProjectDialog
          onConfirm={(name) => {
            dispatch({ type: "create_project", name });
            setCreating(false);
          }}
          onCancel={() => setCreating(false)}
        />
      )}
    </aside>
  );
}

function HomeIcon() {
  return (
    <svg
      className="sidebar__icon"
      width="14"
      height="14"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d="M2.5 6.5 8 2l5.5 4.5V13a1 1 0 0 1-1 1h-9a1 1 0 0 1-1-1z" />
      <path d="M6.25 14v-4.5h3.5V14" />
    </svg>
  );
}
