import type { Component } from "solid-js";
import { type TaskFilter, taskCounts } from "../stores/task-store";

interface FilterTabsProps {
  current: TaskFilter;
  onChange: (filter: TaskFilter) => void;
}

const FilterTabs: Component<FilterTabsProps> = (props) => {
  const tabs: { filter: TaskFilter; label: string; countKey: TaskFilter }[] = [
    { filter: "all", label: "全部", countKey: "all" },
    {
      filter: "downloading",
      label: "下载中",
      countKey: "downloading",
    },
    { filter: "completed", label: "已完成", countKey: "completed" },
  ];

  return (
    <div class="flex gap-1 bg-base-300 rounded-lg p-1">
      {tabs.map((tab) => (
        <button
          class={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
            props.current === tab.filter
              ? "bg-base-100 text-base-content shadow-sm"
              : "text-base-content/50 hover:text-base-content"
          }`}
          onClick={() => props.onChange(tab.filter)}
        >
          {tab.label}
          <span
            class={`ml-1.5 text-[10px] ${
              props.current === tab.filter
                ? "text-base-content/60"
                : "text-base-content/30"
            }`}
          >
            {taskCounts()[tab.countKey]}
          </span>
        </button>
      ))}
    </div>
  );
};

export default FilterTabs;
