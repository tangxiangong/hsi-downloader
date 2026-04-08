import type { Component } from "solid-js";

export type Page = "tasks" | "history" | "settings";

interface SidebarProps {
  current: Page;
  onChange: (page: Page) => void;
}

const Sidebar: Component<SidebarProps> = (props) => {
  const items: { page: Page; label: string }[] = [
    { page: "tasks", label: "任务" },
    { page: "history", label: "历史" },
    { page: "settings", label: "设置" },
  ];

  return (
    <aside class="w-56 bg-base-100 border-r border-base-300 flex flex-col h-screen">
      <div class="p-4 border-b border-base-300">
        <h1 class="text-xl font-bold">驭时</h1>
        <p class="text-xs text-base-content/50 mt-1">YuShi Download Manager</p>
      </div>
      <ul class="menu flex-1 p-2">
        {items.map((item) => (
          <li>
            <a
              class={props.current === item.page ? "active" : ""}
              onClick={() => props.onChange(item.page)}
            >
              {item.label}
            </a>
          </li>
        ))}
      </ul>
      <div class="p-4 text-xs text-base-content/40">
        v0.1.0
      </div>
    </aside>
  );
};

export default Sidebar;
