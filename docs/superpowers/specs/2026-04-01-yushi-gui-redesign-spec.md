# YuShi GUI Redesign Specification

## 1. Overview
The goal of this project is to redesign and beautify the graphical user interface (GUI) of YuShi, a pure Rust async download manager built with `gpui`. The redesign will shift the application from a basic functional layout to a modern, polished, and user-friendly experience.

## 2. Design Direction
The chosen design language is **Modern Minimal** with a **Floating Card Layout**.
*   **Style:** Clean lines, ample whitespace, low-saturation backgrounds, and a focus on content.
*   **Layout:** A "floating" structure where the sidebar and main content areas appear as distinct, elevated cards over a subtle background.
*   **Accent Color:** **Classic Blue**. Used for primary actions, progress bars, and active states to convey reliability and professionalism.

## 3. UI Components & Layout

### 3.1 Global Layout (`src/views/layout.rs`)
*   **Background:** The root window background will be a subtle off-white/light-gray (in light mode) or dark-gray (in dark mode).
*   **Cards:** The Sidebar and the Main Content Panel will be rendered as rounded rectangles (e.g., `rounded-xl` or `rounded-2xl`) with a solid background color (white/dark-gray) and a soft drop shadow (`shadow-md` or `shadow-lg`).
*   **Spacing:** Generous padding around the cards to emphasize the floating effect.

### 3.2 Navigation Sidebar (`src/components/nav_sidebar.rs`)
*   Will become a floating card on the left.
*   Navigation items will have rounded selection states.
*   Active items will use the Classic Blue accent color (either as text color, background tint, or a subtle indicator line).

### 3.3 Task List & Task Cards (`src/views/task_list.rs`, `src/components/task_card.rs`)
*   **Card View:** Each download task will be a distinct, spacious card (`Spacious Cards` approach).
*   **Typography:**
    *   Filename: Large, semi-bold font.
    *   Metadata (speed, ETA, size): Smaller, muted text color.
*   **Progress Bar:** Thicker, more prominent progress bar using the Classic Blue accent color.
*   **Actions:** Pause/Resume/Cancel buttons will be refined. Consider showing them only on hover to reduce visual clutter, or styling them as subtle icon buttons.

### 3.4 Add Task Dialog (`src/views/dialogs.rs`)
*   **Interaction:** Retain the **Modal Dialog** approach.
*   **Styling:** The dialog itself will be a rounded card with a shadow. Input fields will have clear borders, focus states (Classic Blue ring), and adequate padding. The primary "Add" button will be a solid Classic Blue button.

### 3.5 Settings Page (`src/components/settings_form.rs`)
*   Will be housed within the main content floating card.
*   Input fields and toggles will be styled consistently with the modern minimal theme (rounded corners, clear focus states).

## 4. Implementation Strategy (GPUI)
*   **Colors:** Update `src/utils.rs` (or equivalent theme definition) to define the new background colors, card background colors, and the Classic Blue accent color.
*   **Styling:** Leverage `gpui`'s styling methods (`.bg()`, `.rounded_xl()`, `.shadow_md()`, `.p_4()`, etc.) to implement the floating cards and spacious layouts.
*   **Components:** Refactor existing components (`nav_sidebar`, `task_card`, `content_panel`) to wrap their contents in these styled containers.

## 5. Scope
This redesign focuses purely on the visual presentation and layout within the existing `gpui` framework. It does not involve changes to the core downloading logic (`yushi-core`) or the CLI (`yushi-cli`).