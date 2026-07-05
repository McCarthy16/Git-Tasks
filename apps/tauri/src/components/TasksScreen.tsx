import { useState } from "react";
import type { Dispatch, Project, Status, Task } from "../types";
import { Screen } from "./Screen";
import { AddRow } from "./AddRow";
import { List } from "./List";
import { Row } from "./Row";
import { ContextMenu } from "./ContextMenu";
import { ConfirmDialog } from "./ConfirmDialog";
import { RenameDialog } from "./RenameDialog";
import { MoveDialog } from "./MoveDialog";
import { ChangeStatusDialog } from "./ChangeStatusDialog";

type MenuState = { name: string; id: string; x: number; y: number; currentStatusId: string | null };
type DialogState =
  | { kind: "rename"; id: string; name: string }
  | { kind: "move"; id: string; name: string }
  | { kind: "change_status"; id: string; name: string; currentStatusId: string | null }
  | { kind: "remove"; id: string; name: string };

/** The tasks pane: the tasks within the open project. */
export function TasksScreen({
  project,
  projects,
  tasks,
  statuses,
  dispatch,
  error,
}: {
  project: Project;
  projects: Project[];
  tasks: Task[];
  statuses: Status[];
  dispatch: Dispatch;
  error: string | null;
}) {
  const [menu, setMenu] = useState<MenuState | null>(null);
  const [dialog, setDialog] = useState<DialogState | null>(null);

  return (
    <Screen error={error}>
      <AddRow
        placeholder="New task…"
        onSubmit={(name) => dispatch({ type: "create_task", name })}
      />
      <List>
        {tasks.map((task) => (
          <Row
            key={task.id}
            name={task.name}
            id={task.id}
            chip={statuses.find((s) => s.id === task.status_id)?.name ?? "None"}
            onChipClick={() =>
              setDialog({
                kind: "change_status",
                id: task.id,
                name: task.name,
                currentStatusId: task.status_id,
              })
            }
            onClick={() => dispatch({ type: "open_task", task_id: task.id })}
            onContextMenu={(e) => {
              e.preventDefault();
              setMenu({ name: task.name, id: task.id, x: e.clientX, y: e.clientY, currentStatusId: task.status_id });
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
          onMove={
            projects.filter((p) => p.id !== project.id).length > 0
              ? () => setDialog({ kind: "move", id: menu.id, name: menu.name })
              : undefined
          }
          onChangeStatus={() =>
            setDialog({
              kind: "change_status",
              id: menu.id,
              name: menu.name,
              currentStatusId: menu.currentStatusId,
            })
          }
          onRemove={() => setDialog({ kind: "remove", id: menu.id, name: menu.name })}
        />
      )}

      {dialog?.kind === "rename" && (
        <RenameDialog
          name={dialog.name}
          onConfirm={(newName) => {
            dispatch({ type: "rename_task", task_id: dialog.id, new_name: newName });
            setDialog(null);
          }}
          onCancel={() => setDialog(null)}
        />
      )}

      {dialog?.kind === "move" && (
        <MoveDialog
          name={dialog.name}
          currentProjectId={project.id}
          projects={projects}
          onConfirm={(projectId) => {
            dispatch({ type: "move_task", task_id: dialog.id, project_id: projectId });
            setDialog(null);
          }}
          onCancel={() => setDialog(null)}
        />
      )}

      {dialog?.kind === "change_status" && (
        <ChangeStatusDialog
          name={dialog.name}
          currentStatusId={dialog.currentStatusId}
          statuses={statuses}
          onConfirm={(statusId) => {
            dispatch({ type: "set_task_status", task_id: dialog.id, status_id: statusId });
            setDialog(null);
          }}
          onCancel={() => setDialog(null)}
        />
      )}

      {dialog?.kind === "remove" && (
        <ConfirmDialog
          name={dialog.name}
          message="Remove this task? This cannot be undone."
          onConfirm={() => {
            dispatch({ type: "close_task", task_id: dialog.id });
            setDialog(null);
          }}
          onCancel={() => setDialog(null)}
        />
      )}
    </Screen>
  );
}
