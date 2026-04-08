import type { Component } from "solid-js";
import type { AppTheme } from "../lib/types";

interface ThemeToggleProps {
  value: AppTheme;
  onChange: (theme: AppTheme) => void;
}

const ThemeToggle: Component<ThemeToggleProps> = (props) => {
  const options: { value: AppTheme; label: string }[] = [
    { value: "light", label: "\u6d45\u8272" },
    { value: "dark", label: "\u6df1\u8272" },
    { value: "system", label: "\u8ddf\u968f\u7cfb\u7edf" },
  ];

  return (
    <div class="flex gap-1 bg-base-300 rounded-lg p-1">
      {options.map((opt) => (
        <button
          class={`px-3 py-1.5 rounded-md text-xs font-medium transition-colors ${
            props.value === opt.value
              ? "bg-base-100 text-base-content shadow-sm"
              : "text-base-content/50 hover:text-base-content"
          }`}
          onClick={() => props.onChange(opt.value)}
        >
          {opt.label}
        </button>
      ))}
    </div>
  );
};

export default ThemeToggle;
