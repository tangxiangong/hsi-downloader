import { type Component, createSignal, For, Show } from "solid-js";
import FilterTabs from "../components/FilterTabs";
import TaskCard from "../components/TaskCard";
import AddTaskDialog from "../components/AddTaskDialog";
import {
  filteredTasks,
  filter,
  setFilter,
  taskCounts,
} from "../stores/task-store";

const TasksPage: Component = () => {
  const [showAddDialog, setShowAddDialog] = createSignal(false);

  return (
    <div>
      <div class="flex items-center justify-between mb-5">
        <div>
          <h2 class="text-xl font-bold">任务</h2>
          <p class="text-xs text-base-content/40 mt-0.5">
            {taskCounts().all} 个任务
          </p>
        </div>
        <button
          class="btn btn-primary btn-sm"
          onClick={() => setShowAddDialog(true)}
        >
          + 添加任务
        </button>
      </div>

      <FilterTabs current={filter()} onChange={setFilter} />

      <div class="mt-4 space-y-2">
        <For
          each={filteredTasks()}
          fallback={
            <div class="text-center text-base-content/30 py-16">
              <div class="text-4xl mb-3">↓</div>
              <p class="text-sm">暂无任务</p>
              <p class="text-xs mt-1">
                点击「添加任务」开始下载
              </p>
            </div>
          }
        >
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
