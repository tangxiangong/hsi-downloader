import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { type Component, createSignal, onCleanup, onMount, Match, Switch } from "solid-js";
import Sidebar, { type Page } from "./components/Sidebar";
import TasksPage from "./pages/TasksPage";
import HistoryPage from "./pages/HistoryPage";
import SettingsPage from "./pages/SettingsPage";
import { loadConfig } from "./stores/config-store";
import { loadTasks, setupTaskEvents } from "./stores/task-store";
import { loadHistory } from "./stores/history-store";

interface NavigateMainPayload {
  page?: Page;
}

const App: Component = () => {
  const [page, setPage] = createSignal<Page>("tasks");

  onMount(async () => {
    await loadConfig();
    await loadTasks();
    await loadHistory();
    setupTaskEvents();

    const unlisten = await getCurrentWebviewWindow().listen<NavigateMainPayload>(
      "navigate-main",
      (event) => {
        setPage(event.payload.page ?? "tasks");
      },
    );

    onCleanup(() => {
      void unlisten();
    });
  });

  return (
    <div class="flex h-screen">
      <Sidebar current={page()} onChange={setPage} />
      <main class="flex-1 p-5 overflow-y-auto bg-base-200 main-bg">
        <Switch>
          <Match when={page() === "tasks"}>
            <TasksPage />
          </Match>
          <Match when={page() === "history"}>
            <HistoryPage />
          </Match>
          <Match when={page() === "settings"}>
            <SettingsPage />
          </Match>
        </Switch>
      </main>
    </div>
  );
};

export default App;
