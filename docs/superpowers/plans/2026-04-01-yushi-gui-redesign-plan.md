# YuShi GUI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the "Modern Minimal" and "Floating Card" redesign for the YuShi GUI.
**Architecture:** We will update the GPUI styling in `src/utils.rs` to define the new color palette (Classic Blue accent, off-white/dark-gray backgrounds) and then refactor the layout (`src/views/layout.rs`), sidebar (`src/components/nav_sidebar.rs`), and task cards (`src/components/task_card.rs`) to use rounded corners, shadows, and increased padding to create the floating card effect.
**Tech Stack:** Rust, `gpui`, `gpui_component`

---

### Task 1: Update Color Palette and Theme Utilities

**Files:**
- Modify: `src/utils.rs:9-71`

- [ ] **Step 1: Update background and panel colors**
Modify `app_background`, `panel_color`, and `card_color` to create a softer, lower-contrast base for the floating cards.

```rust
pub fn app_background(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.10, 0.08, 1.0) // Darker, less saturated background
    } else {
        hsla(0.60, 0.05, 0.96, 1.0) // Very light gray/off-white background
    }
}

pub fn panel_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.62, 0.12, 0.14, 1.0) // Slightly lighter than background for cards
    } else {
        white() // Pure white for cards in light mode
    }
}

pub fn card_color(cx: &App) -> Hsla {
    panel_color(cx) // Keep cards and panels the same color for consistency
}
```

- [ ] **Step 2: Update primary accent color to Classic Blue**
Modify `primary_color` to use a classic, reliable blue.

```rust
pub fn primary_color(cx: &App) -> Hsla {
    if cx.theme().is_dark() {
        hsla(0.58, 0.80, 0.60, 1.0) // Bright Classic Blue for dark mode
    } else {
        hsla(0.60, 0.85, 0.50, 1.0) // Deep Classic Blue for light mode
    }
}
```

- [ ] **Step 3: Commit**
```bash
cargo fmt
git add src/utils.rs
git commit -m "style: update color palette for modern minimal design"
```

### Task 2: Implement Floating Sidebar

**Files:**
- Modify: `src/components/nav_sidebar.rs:9-109`

- [ ] **Step 1: Refactor `nav_sidebar` to be a floating card**
Remove the right border and full height. Add margin, rounded corners, and a shadow.

```rust
pub fn nav_sidebar(current_view: ViewKind, stats: &ViewStats, cx: &mut Context<YuShiGUI>) -> Div {
    v_flex()
        .w(px(240.))
        // Remove h_full(), add margin to float it
        .my_4()
        .ml_4()
        .flex_shrink_0()
        .justify_between()
        .bg(utils::panel_color(cx))
        // Remove border_r_1() and border_color()
        // Add rounded corners and shadow
        .rounded_xl()
        .shadow_md()
        .p_4()
        .child(
            v_flex()
                .gap_4()
                .child(
                    v_flex()
                        .gap_2()
                        .child(
                            div()
                                .font_semibold()
                                .text_color(utils::text_color(cx))
                                .child("导航"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(utils::muted_text_color(cx))
                                .child("任务、历史和设置都在这里。"),
                        ),
                )
                .child(nav_item(
                    "All Tasks",
                    IconName::LayoutDashboard,
                    current_view == ViewKind::AllTasks,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::AllTasks, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "Downloading",
                    IconName::ArrowDown,
                    current_view == ViewKind::Downloading,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::Downloading, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "Completed",
                    IconName::CircleCheck,
                    current_view == ViewKind::Completed,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::Completed, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "History",
                    IconName::BookOpen,
                    current_view == ViewKind::History,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::History, window, cx);
                    }),
                    cx,
                ))
                .child(nav_item(
                    "Settings",
                    IconName::Settings,
                    current_view == ViewKind::Settings,
                    cx.listener(|view, _, window, cx| {
                        view.set_view(ViewKind::Settings, window, cx);
                    }),
                    cx,
                )),
        )
        .child(
            v_flex()
                .gap_1()
                .p_3()
                .rounded_lg()
                .bg(utils::app_background(cx)) // Make inner card contrast with panel
                // Remove border
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(utils::text_color(cx))
                        .child("当前概览"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(utils::muted_text_color(cx))
                        .child(format!(
                            "共 {} 个任务，{} 条历史",
                            stats.total_tasks, stats.history_items
                        )),
                ),
        )
}
```

- [ ] **Step 2: Commit**
```bash
cargo fmt
git add src/components/nav_sidebar.rs
git commit -m "style: implement floating sidebar card"
```

### Task 3: Implement Floating Content Panel

**Files:**
- Modify: `src/components/content_panel.rs:5-41`
- Modify: `src/views/layout.rs:44-64`

- [ ] **Step 1: Refactor `content_panel` to be a floating card**
Add margin, rounded corners, and shadow to match the sidebar.

```rust
// In src/components/content_panel.rs
pub fn content_panel(
    current_view: ViewKind,
    content: AnyElement,
    status_message: Option<String>,
    cx: &App,
) -> Div {
    let panel = v_flex()
        .size_full()
        .p_6() // Increase padding
        .gap_6() // Increase gap
        .bg(utils::panel_color(cx))
        .rounded_xl()
        .shadow_md()
        .text_color(utils::text_color(cx))
        .child(
            div()
                .pb_4() // Increase padding bottom
                .border_b_1()
                .border_color(utils::border_color(cx))
                .child(
                    div()
                        .text_2xl() // Make title larger
                        .font_semibold()
                        .child(view_title(current_view))
                )
                .child(
                    div()
                        .mt_1()
                        .text_sm()
                        .text_color(utils::muted_text_color(cx))
                        .child(view_description(current_view)),
                ),
        )
        // Note: we removed the separate description child since we moved it into the header block above
        .child(content);

    match status_message {
        Some(message) => panel.child(
            div()
                .mt_auto() // Push to bottom
                .text_sm()
                .text_color(utils::muted_text_color(cx))
                .child(message),
        ),
        None => panel,
    }
}
```

- [ ] **Step 2: Adjust global layout spacing**
Ensure the content panel has margins so it floats.

```rust
// In src/views/layout.rs
        div()
            .size_full()
            .bg(utils::app_background(cx))
            .text_color(utils::text_color(cx))
            .child(
                v_flex()
                    .size_full()
                    .child(TitleBar::new().child(header(cx)))
                    .child(
                        h_flex().size_full().child(sidebar).child(
                            v_flex()
                                .size_full()
                                .p_4() // Add padding around the right side content
                                .gap_4()
                                .child(summary_row)
                                .child(content_panel),
                        ),
                    ),
            )
```

- [ ] **Step 3: Commit**
```bash
cargo fmt
git add src/components/content_panel.rs src/views/layout.rs
git commit -m "style: implement floating content panel and adjust layout"
```

### Task 4: Redesign Task Cards (Spacious Cards)

**Files:**
- Modify: `src/components/task_card.rs:11-105`

- [ ] **Step 1: Refactor `task_card` for a spacious layout**
Increase padding, enlarge the filename, and adjust the layout of metadata and actions.

```rust
pub fn task_card(task: DownloadTask, cx: &mut Context<YuShiGUI>) -> Div {
    let task_id: SharedString = task.id.clone().into();
    let actions = task_actions(task.status);

    v_flex()
        .gap_4() // Increase gap
        .p_5() // Increase padding
        // Remove border, rely on background and shadow
        .rounded_xl()
        .bg(utils::app_background(cx)) // Use app background to contrast with the panel
        .shadow_sm()
        .text_color(utils::text_color(cx))
        .child(
            h_flex()
                .justify_between()
                .items_start()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div().text_lg().font_semibold().child( // Larger filename
                                task.url
                                    .split('/')
                                    .next_back()
                                    .unwrap_or(task.url.as_str())
                                    .to_string(),
                            ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(utils::muted_text_color(cx))
                                .child(task.url.clone()),
                        ),
                )
                .child(utils::status_badge(task.status, cx)),
        )
        .child(
            // Wrap progress bar to make it thicker (gpui_component Progress might have fixed height, but we can try to wrap it or just rely on its default)
            div().py_2().child(Progress::new("progress").value(progress_percent(&task)))
        )
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            div().text_sm().font_medium().child(format!(
                                "{} / {}  ·  {} /s",
                                utils::format_bytes(task.downloaded),
                                utils::format_bytes(task.total_size),
                                utils::format_bytes(task.speed),
                            ))
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(utils::muted_text_color(cx))
                                .child(match task.speed_limit {
                                    Some(limit) => format!("限速 {}/s · 保存到 {}", utils::format_bytes(limit), task.dest.display()),
                                    None => format!("不限速 · 保存到 {}", task.dest.display()),
                                }),
                        )
                )
                .child(
                    h_flex()
                        .gap_2()
                        .children(actions.into_iter().enumerate().map(|(index, action)| {
                            let button_id = SharedString::from(format!(
                                "task-{}-{}-{}",
                                task.id,
                                index,
                                action.id_suffix()
                            ));
                            let task_id = task_id.clone();
                            let destination = task.dest.display().to_string();
                            let button_style = if action.is_primary() {
                                utils::button_style(utils::primary_color(cx), gpui_component::white(), cx)
                            } else {
                                utils::button_style(utils::panel_color(cx), utils::text_color(cx), cx)
                            };

                            Button::new(button_id)
                                .custom(button_style)
                                .label(action.button_label())
                                .on_click(cx.listener(move |view, _, window, cx| {
                                    if action == TaskAction::DeleteFile {
                                        view.open_task_delete_file_dialog(
                                            task_id.clone(),
                                            destination.clone(),
                                            window,
                                            cx,
                                        );
                                    } else {
                                        view.run_task_action(task_id.clone(), action, window, cx);
                                    }
                                }))
                        })),
                )
        )
}
```

- [ ] **Step 2: Commit**
```bash
cargo fmt
git add src/components/task_card.rs
git commit -m "style: redesign task cards to be spacious and modern"
```

### Task 5: Refine Dialog Styling

**Files:**
- Modify: `src/views/dialogs.rs`

- [ ] **Step 1: Check and refine dialog styling**
The `gpui_component` `Dialog` might have its own styling, but we should ensure the content we pass in looks good.
In `open_add_task_dialog`, add some padding to the `v_flex` container.

```rust
// In src/views/dialogs.rs, inside open_add_task_dialog
                .child(
                    v_flex()
                        .p_2() // Add padding
                        .gap_4() // Increase gap
                        .child(div().text_sm().text_color(utils::muted_text_color(cx)).child("Use the default download directory by leaving destination blank."))
                        .child(Input::new(&add_url_input))
                        .child(Input::new(&add_dest_input))
                        .child(Input::new(&add_speed_input)),
                )
```

- [ ] **Step 2: Commit**
```bash
cargo fmt
git add src/views/dialogs.rs
git commit -m "style: refine add task dialog spacing"
```

### Task 6: Final Verification

**Files:**
- None

- [ ] **Step 1: Run the application**
Run `cargo run` to launch the GUI and visually inspect the changes. Ensure the floating cards look correct in both light and dark modes, and the Classic Blue accent color is applied.

- [ ] **Step 2: Check tests**
Run `cargo test --workspace` to ensure no logic was broken during the UI refactoring.

- [ ] **Step 3: Commit**
```bash
git commit --allow-empty -m "chore: verify GUI redesign implementation"
```
