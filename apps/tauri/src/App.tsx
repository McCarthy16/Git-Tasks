import { useView } from "./useView";
import { basename } from "./path";
import { SelectRepo } from "./components/SelectRepo";
import { Sidebar } from "./components/Sidebar";
import { Header, type Crumb } from "./components/Header";
import { ProjectsScreen } from "./components/ProjectsScreen";
import { TasksScreen } from "./components/TasksScreen";
import { TaskDetailScreen } from "./components/TaskDetailScreen";
import "./App.css";

function App() {
  const { view, error, dispatch } = useView();

  if (!view) return <div className="app" />;

  // No workspace open: the full-screen repo picker, no shell.
  if (view.screen === "select_repo") {
    return (
      <div className="app">
        <SelectRepo dispatch={dispatch} recentWorkspaces={view.recent_workspaces} />
      </div>
    );
  }

  // <repo> / <project> / <task> — every crumb but the last navigates up.
  const crumbs: Crumb[] = [
    {
      label: basename(view.workspace.root),
      onClick:
        view.screen === "projects"
          ? undefined
          : () => dispatch({ type: "close_project" }),
    },
  ];
  if (view.screen === "tasks" || view.screen === "task_detail") {
    crumbs.push({
      label: view.project.name,
      onClick:
        view.screen === "task_detail"
          ? () => dispatch({ type: "close_task_detail" })
          : undefined,
    });
  }
  if (view.screen === "task_detail") {
    crumbs.push({ label: view.task.name });
  }

  return (
    <div className="app shell">
      <Sidebar
        workspace={view.workspace}
        projects={view.projects}
        activeProjectId={view.screen === "projects" ? null : view.project.id}
        dispatch={dispatch}
      />
      <main className="shell__main">
        <Header crumbs={crumbs} />
        {view.screen === "projects" && (
          <ProjectsScreen projects={view.projects} dispatch={dispatch} error={error} />
        )}
        {view.screen === "tasks" && (
          <TasksScreen
            project={view.project}
            projects={view.projects}
            tasks={view.tasks}
            statuses={view.statuses}
            dispatch={dispatch}
            error={error}
          />
        )}
        {view.screen === "task_detail" && (
          <TaskDetailScreen
            key={view.task.id}
            task={view.task}
            events={view.events}
            statuses={view.statuses}
            dispatch={dispatch}
            error={error}
          />
        )}
      </main>
    </div>
  );
}

export default App;
