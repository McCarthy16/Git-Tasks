import { useEffect, useRef, useState } from "react";
import type { Dispatch, Project, Status, Task, TaskEventEntry, View, WorkspaceView } from "../types";
import { basename } from "../path";
import { formatTime } from "../time";
import { Screen } from "./Screen";
import { MarkdownEditor } from "./MarkdownEditor";
import { GhostTextField } from "./GhostTextField";
import { DescriptionSnapshotDialog } from "./DescriptionSnapshotDialog";
import "./TaskDetailScreen.css";

const AUTOSAVE_DELAY_MS = 600;

/** Screen 4: a single task — editable title, description, and event timeline. */
export function TaskDetailScreen({
  workspace,
  project,
  task,
  events,
  statuses,
  dispatch,
  error,
}: {
  workspace: WorkspaceView;
  project: Project;
  task: Task;
  events: TaskEventEntry[];
  statuses: Status[];
  dispatch: Dispatch;
  error: string | null;
}) {
  const sessionEventIds = useRef<Map<string, string>>(new Map());
  const saveTimer = useRef<number | null>(null);
  const [snapshotEvent, setSnapshotEvent] = useState<TaskEventEntry | null>(null);

  useEffect(() => {
    return () => {
      if (saveTimer.current !== null) clearTimeout(saveTimer.current);
    };
  }, []);

  async function handleTitleCommit(name: string) {
    const sessionEventId = sessionEventIds.current.get("rename");
    if (sessionEventId) {
      dispatch({ type: "rename_task_in_place", task_id: task.id, event_id: sessionEventId, new_name: name });
    } else {
      const newView = await dispatch({ type: "rename_task", task_id: task.id, new_name: name });
      recordSessionEvent(newView, "renamed", "rename");
    }
  }

  function handleDescriptionChange(markdown: string) {
    if (saveTimer.current !== null) clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(async () => {
      const sessionEventId = sessionEventIds.current.get("description");
      if (sessionEventId) {
        dispatch({ type: "update_task_description_in_place", task_id: task.id, event_id: sessionEventId, description: markdown });
      } else {
        const newView = await dispatch({ type: "update_task_description", task_id: task.id, description: markdown });
        recordSessionEvent(newView, "description_updated", "description");
      }
    }, AUTOSAVE_DELAY_MS);
  }

  function recordSessionEvent(view: View | null, kind: string, key: string) {
    if (view?.screen !== "task_detail") return;
    const match = [...view.events].reverse().find((e) => e.kind === kind);
    if (match) sessionEventIds.current.set(key, match.id);
  }

  return (
    <Screen
      crumbs={[
        { label: basename(workspace.root), onClick: () => dispatch({ type: "close_project" }) },
        { label: project.name, onClick: () => dispatch({ type: "close_task_detail" }) },
        { label: task.name },
      ]}
      error={error}
    >
      <div className="task-detail">
        <div className="task-detail__content">
          <div className="task-detail__title-row">
            <GhostTextField
              className="task-detail__title"
              value={task.name}
              onCommit={handleTitleCommit}
            />
            {task.status_id && (
              <span className="row__chip">
                {statuses.find((s) => s.id === task.status_id)?.name}
              </span>
            )}
          </div>
          <MarkdownEditor
            content={task.description}
            placeholder="Add a description…"
            onChange={handleDescriptionChange}
          />
        </div>

        <hr className="task-detail__divider" />

        <div className="task-detail__timeline">
          <p className="task-detail__timeline-heading">History</p>
          <ol className="timeline" aria-label="Task history">
            {events.map((event, i) => {
              const isClickable = event.kind === "description_updated" && event.detail != null;
              return (
                <li key={event.id} className="timeline__entry">
                  <div className="timeline__spine">
                    <div className="timeline__dot" data-kind={event.kind} />
                    {i < events.length - 1 && <div className="timeline__line" />}
                  </div>
                  {isClickable ? (
                    <button
                      className="timeline__body timeline__body--btn"
                      onClick={() => setSnapshotEvent(event)}
                    >
                      <span className="timeline__label">
                        {eventLabel(event)}
                        <span className="timeline__peek"> — view</span>
                      </span>
                      {event.created_at_millis != null && (
                        <span className="timeline__time">{formatTime(event.created_at_millis)}</span>
                      )}
                    </button>
                  ) : (
                    <div className="timeline__body">
                      <span className="timeline__label">{eventLabel(event)}</span>
                      {event.created_at_millis != null && (
                        <span className="timeline__time">{formatTime(event.created_at_millis)}</span>
                      )}
                    </div>
                  )}
                </li>
              );
            })}
          </ol>
        </div>
      </div>

      {snapshotEvent && (
        <DescriptionSnapshotDialog
          event={snapshotEvent}
          onClose={() => setSnapshotEvent(null)}
        />
      )}
    </Screen>
  );
}

function eventLabel(event: TaskEventEntry): string {
  switch (event.kind) {
    case "created":      return "Created";
    case "renamed":      return event.detail ? `Renamed to "${event.detail}"` : "Renamed";
    case "moved":        return "Moved to another project";
    case "closed":       return "Closed";
    case "reopened":     return "Reopened";
    case "description_updated": return "Description updated";
    case "status_changed":      return event.detail ? `Status changed to ${event.detail}` : "Status cleared";
    default:             return event.kind;
  }
}
