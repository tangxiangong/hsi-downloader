import { type Component, createSignal, For, Show } from "solid-js";
import FilterTabs from "../components/FilterTabs";
import TaskCard from "../components/TaskCard";
import AddTaskDialog from "../components/AddTaskDialog";
import { filteredTasks, filter, setFilter, taskCounts } from "../stores/task-store";

const TasksPage: Component = () => {
  const [showAddDialog, setShowAddDialog] = createSignal(false);

  return (
    <div>
      <div class="flex items-center justify-between mb-5">
        <div>
          <h2 class="text-xl font-bold">{"\u4efb\u52a1"}</h2>
          <p class="text-xs text-base-content/40 mt-0.5">{taskCounts().all} {"\u4e2a\u4efb\u52a1"}</p>
        </div>
        <button class="btn btn-primary btn-sm" onClick={() => setShowAddDialog(true)}>
          + {"\u6dfb\u52a0\u4efb\u52a1"}
        </button>
      </div>

      <FilterTabs current={filter()} onChange={setFilter} />

      <div class="mt-4 space-y-2">
        <For each={filteredTasks()} fallback={
          <div class="text-center text-base-content/30 py-16">
            <div class="text-4xl mb-3">{"\u2193"}</div>
            <p class="text-sm">{"\u6682\u65e0\u4efb\u52a1"}</p>
            <p class="text-xs mt-1">{"\u70b9\u51fb\u300c\u6dfb\u52a0\u4efb\u52a1\u300d\u5f00\u59cb\u4e0b\u8f7d"}</p>
          </div>
        }>
          {(task) => <TaskCard task={task} />}
        </For>
      </div>

      <Show when={showAddDialog()}>
        <AddTaskDialog onClose={() => setShowAddDialog(false)} />
      </Show>
    </div>
  );
};

export default TasksPage;
