import { type Component, createSignal, For, Show } from "solid-js";
import FilterTabs from "../components/FilterTabs";
import TaskCard from "../components/TaskCard";
import AddTaskDialog from "../components/AddTaskDialog";
import { filteredTasks, filter, setFilter } from "../stores/task-store";

const TasksPage: Component = () => {
  const [showAddDialog, setShowAddDialog] = createSignal(false);

  return (
    <div>
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-2xl font-bold">任务</h2>
        <button class="btn btn-primary btn-sm" onClick={() => setShowAddDialog(true)}>
          添加任务
        </button>
      </div>

      <FilterTabs current={filter()} onChange={setFilter} />

      <div class="mt-4 space-y-3">
        <For each={filteredTasks()} fallback={
          <div class="text-center text-base-content/50 py-12">暂无任务</div>
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
