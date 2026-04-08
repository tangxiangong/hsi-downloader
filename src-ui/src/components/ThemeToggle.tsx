import type { Component } from "solid-js";
import type { AppTheme } from "../lib/types";

interface ThemeToggleProps {
  value: AppTheme;
  onChange: (theme: AppTheme) => void;
}

const ThemeToggle: Component<ThemeToggleProps> = (props) => {
  const options: { value: AppTheme; label: string }[] = [
    { value: "light", label: "浅色" },
    { value: "dark", label: "深色" },
    { value: "system", label: "跟随系统" },
  ];

  return (
    <div class="flex gap-2">
      {options.map((opt) => (
        <button
          class={`btn btn-sm ${props.value === opt.value ? "btn-primary" : "btn-ghost"}`}
          onClick={() => props.onChange(opt.value)}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
};

export default ThemeToggle;
