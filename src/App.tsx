import { useView } from "./useView";
import { SelectRepo } from "./components/SelectRepo";
import { ProjectsScreen } from "./components/ProjectsScreen";
import { TasksScreen } from "./components/TasksScreen";
import "./App.css";

function App() {
  const { view, error, dispatch } = useView();

  if (!view) return <div className="app" />;

  return (
    <div className="app">
      {view.screen === "select_repo" && <SelectRepo dispatch={dispatch} />}
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
          tasks={view.tasks}
          dispatch={dispatch}
          error={error}
        />
      )}
    </div>
  );
}

export default App;
