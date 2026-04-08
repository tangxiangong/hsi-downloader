import type { Component } from "solid-js";

const App: Component = () => {
  return (
    <div class="flex h-screen">
      <aside class="w-56 bg-base-100 border-r border-base-300 p-4">
        <h1 class="text-xl font-bold mb-6">驭时</h1>
        <ul class="menu">
          <li><a class="active">任务</a></li>
          <li><a>历史</a></li>
          <li><a>设置</a></li>
        </ul>
      </aside>
      <main class="flex-1 p-6 overflow-y-auto">
        <p class="text-base-content">YuShi is running.</p>
      </main>
    </div>
  );
};

export default App;
