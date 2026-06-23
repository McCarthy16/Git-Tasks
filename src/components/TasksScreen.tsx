import type { Dispatch, Project, Task, WorkspaceView } from "../types";
import { basename } from "../path";
import { Screen } from "./Screen";
import { AddRow } from "./AddRow";
import { List } from "./List";
import { Row } from "./Row";

/** Screen 3: the tasks within the open project. The first crumb returns to the project list. */
export function TasksScreen({
  workspace,
  project,
  tasks,
  dispatch,
  error,
}: {
  workspace: WorkspaceView;
  project: Project;
  tasks: Task[];
  dispatch: Dispatch;
  error: string | null;
}) {
  return (
    <Screen
      crumbs={[
        {
          label: basename(workspace.root),
          onClick: () => dispatch({ type: "close_project" }),
        },
        { label: project.name },
      ]}
      error={error}
    >
      <AddRow
        placeholder="New task…"
        onSubmit={(name) => dispatch({ type: "create_task", name })}
      />
      <List>
        {tasks.map((task) => (
          <Row key={task.id} name={task.name} id={task.id} />
        ))}
      </List>
    </Screen>
  );
}
