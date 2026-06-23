import type { Dispatch, Project, WorkspaceView } from "../types";
import { basename } from "../path";
import { Screen } from "./Screen";
import { AddRow } from "./AddRow";
import { List } from "./List";
import { Row } from "./Row";

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
          />
        ))}
      </List>
    </Screen>
  );
}
