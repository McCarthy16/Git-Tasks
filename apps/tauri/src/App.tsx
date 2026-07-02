import { useView } from "./useView";
import { SelectRepo } from "./components/SelectRepo";
import { ProjectsScreen } from "./components/ProjectsScreen";
import { TasksScreen } from "./components/TasksScreen";
import { TaskDetailScreen } from "./components/TaskDetailScreen";
import "./App.css";

function App() {
  const { view, error, dispatch } = useView();

  if (!view) return <div className="app" />;

  return (
    <div className="app">
      {view.screen === "select_repo" && (
        <SelectRepo
          dispatch={dispatch}
          recentWorkspaces={view.recent_workspaces}
        />
      )}
      {view.screen === "projects" && (
        <ProjectsScreen
          workspace={view.workspace}
          projects={view.projects}
          dispatch={dispatch}
          error={error}
        />
      )}
      {view.screen === "tasks" && (
        <TasksScreen
          workspace={view.workspace}
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
          workspace={view.workspace}
          project={view.project}
          task={view.task}
          events={view.events}
          statuses={view.statuses}
          dispatch={dispatch}
          error={error}
        />
      )}
    </div>
  );
}

export default App;
