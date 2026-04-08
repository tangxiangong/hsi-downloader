import type { Component } from "solid-js";

export type Page = "tasks" | "history" | "settings";

interface SidebarProps {
  current: Page;
  onChange: (page: Page) => void;
}

const Sidebar: Component<SidebarProps> = (props) => {
  const items: { page: Page; icon: string; label: string }[] = [
    { page: "tasks", icon: "\u2193", label: "\u4efb\u52a1" },
    { page: "history", icon: "\u2630", label: "\u5386\u53f2" },
    { page: "settings", icon: "\u2699", label: "\u8bbe\u7f6e" },
  ];

  return (
    <aside class="w-[60px] bg-base-100 border-r border-base-300 flex flex-col items-center h-screen py-4 gap-2">
      {/* App Logo */}
      <img src="/yushi.png" alt="驭时" class="w-9 h-9 rounded-lg mb-4" />

      {/* Navigation */}
      <nav class="flex flex-col items-center gap-1 flex-1">
        {items.map((item) => (
          <div class="tooltip tooltip-right" data-tip={item.label}>
            <button
              class={`w-10 h-10 rounded-lg flex items-center justify-center text-lg transition-colors ${
                props.current === item.page
                  ? "bg-primary/15 text-primary"
                  : "text-base-content/50 hover:text-base-content hover:bg-base-300"
              }`}
              onClick={() => props.onChange(item.page)}
            >
              {item.icon}
            </button>
          </div>
        ))}
      </nav>

      {/* Version */}
      <div class="text-[9px] text-base-content/30 writing-vertical-rl">
        v0.1.0
      </div>
    </aside>
  );
};

export default Sidebar;
