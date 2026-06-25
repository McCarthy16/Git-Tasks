import { useState } from "react";
import type { Dispatch, Project, WorkspaceView } from "../types";
import { basename } from "../path";
import { Screen } from "./Screen";
import { AddRow } from "./AddRow";
import { List } from "./List";
import { Row } from "./Row";
import { ContextMenu } from "./ContextMenu";
import { ConfirmDialog } from "./ConfirmDialog";
import { RenameDialog } from "./RenameDialog";

type MenuState = { name: string; id: string; x: number; y: number };
type DialogState =
  | { kind: "rename"; id: string; name: string }
  | { kind: "remove"; id: string; name: string };

/** Screen 2: the projects in the open workspace. Selecting one opens its tasks. */
export function ProjectsScreen({
  workspace,
  projects,
  dispatch,
  error,
}: {
  workspace: WorkspaceView;
  projects: Project[];
  dispatch: Dispatch;
  error: string | null;
}) {
  const [menu, setMenu] = useState<MenuState | null>(null);
  const [dialog, setDialog] = useState<DialogState | null>(null);

  return (
    <Screen
      crumbs={[
        {
          label: basename(workspace.root),
          onClick: () => dispatch({ type: "close_workspace" }),
        },
      ]}
      error={error}
    >
      <AddRow
        placeholder="New project…"
        onSubmit={(name) => dispatch({ type: "create_project", name })}
      />
      <List>
        {projects.map((project) => (
          <Row
            key={project.id}
            name={project.name}
            id={project.id}
            onClick={() =>
              dispatch({ type: "open_project", project_id: project.id })
            }
            onContextMenu={(e) => {
              e.preventDefault();
              setMenu({ name: project.name, id: project.id, x: e.clientX, y: e.clientY });
            }}
          />
        ))}
      </List>

      {menu && (
        <ContextMenu
          name={menu.name}
          id={menu.id}
          x={menu.x}
          y={menu.y}
          onClose={() => setMenu(null)}
          onRename={() => setDialog({ kind: "rename", id: menu.id, name: menu.name })}
          onRemove={() => setDialog({ kind: "remove", id: menu.id, name: menu.name })}
        />
      )}

      {dialog?.kind === "rename" && (
        <RenameDialog
          name={dialog.name}
          onConfirm={(newName) => {
            dispatch({ type: "rename_project", project_id: dialog.id, new_name: newName });
            setDialog(null);
          }}
          onCancel={() => setDialog(null)}
        />
      )}

      {dialog?.kind === "remove" && (
        <ConfirmDialog
          name={dialog.name}
          message="Remove this project? This cannot be undone."
          onConfirm={() => {
            dispatch({ type: "archive_project", project_id: dialog.id });
            setDialog(null);
          }}
          onCancel={() => setDialog(null)}
        />
      )}
    </Screen>
  );
}
