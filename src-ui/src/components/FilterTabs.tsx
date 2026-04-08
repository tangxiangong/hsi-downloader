import type { Component } from "solid-js";
import { type TaskFilter, taskCounts } from "../stores/task-store";

interface FilterTabsProps {
  current: TaskFilter;
  onChange: (filter: TaskFilter) => void;
}

const FilterTabs: Component<FilterTabsProps> = (props) => {
  const tabs: { filter: TaskFilter; label: string; countKey: TaskFilter }[] = [
    { filter: "all", label: "全部", countKey: "all" },
    { filter: "downloading", label: "下载中", countKey: "downloading" },
    { filter: "completed", label: "已完成", countKey: "completed" },
  ];

  return (
    <div role="tablist" class="tabs tabs-bordered">
      {tabs.map((tab) => (
        <a
          role="tab"
          class={`tab ${props.current === tab.filter ? "tab-active" : ""}`}
          onClick={() => props.onChange(tab.filter)}
        >
          {tab.label}
          <span class="badge badge-sm ml-2">{taskCounts()[tab.countKey]}</span>
        </a>
      ))}
    </div>
  );
};

export default FilterTabs;
