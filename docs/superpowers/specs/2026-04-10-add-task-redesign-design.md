# AddTaskDialog 改造 + 分块进度条 Bug 修复

**日期:** 2026-04-10
**范围:** 前端 AddTaskDialog 组件改造、后端 AddTaskOptions 扩展、分块进度条 bug 修复

## 1. 问题描述

1. **添加任务时无法区分 HTTP 下载和磁力下载** — 当前 AddTaskDialog 是单一表单，虽然后端已有 `DownloadSource::Http` 和 `DownloadSource::BitTorrent` 的区分，但前端没有体现。
2. **每个任务无法单独配置** — 后端 `AddTaskOptions` 已支持 `priority`、`speed_limit`、`checksum` 等字段，但前端未暴露这些配置项。
3. **分块进度条 bug** — HTTP 分块下载任务的分块进度条仅在第一次暂停后才正常显示，首次下载时不显示。

## 2. AddTaskDialog 改造

### 2.1 UI 结构：Tab 切换 + 分层配置

对话框顶部使用 DaisyUI `tabs` 组件提供 **HTTP 下载** / **磁力下载** 两个 Tab。

**共享区域（始终显示）：**
- URL 输入框
- 保存位置 + 浏览按钮
- 优先级选择（Low / Normal / High），默认 Normal
- 速度限制输入（可选，单位 KB/s）

**高级选项（折叠区，默认收起）：**
- **HTTP Tab：**
  - 校验和验证：类型选择（MD5 / SHA256）+ 值输入
  - 自定义 Headers：动态 key-value 编辑器（添加/删除行）
- **磁力 Tab：**
  - 文件选择列表（已有逻辑迁移到此处）

### 2.2 自动检测行为

输入 URL 时自动调用 `isBtUrl()` 检测类型并切换到对应 Tab。用户也可手动切换 Tab。切换 Tab 时重置该 Tab 特有的状态（如 torrentFiles）。

### 2.3 提交数据流

构造完整 `AddTaskOptions` 提交：

```typescript
interface AddTaskOptions {
  url: string;
  dest: string;
  checksum?: ChecksumType;        // 新增 UI 入口
  priority?: TaskPriority;        // 新增 UI 入口
  speed_limit?: number;           // 新增 UI 入口
  auto_rename_on_conflict?: boolean;
  selected_files?: number[];
  headers?: Record<string, string>; // 新增字段
}
```

## 3. 后端变更

### 3.1 AddTaskOptions 扩展

在 `hsi-core/src/types.rs` 的 `AddTaskOptions` 添加：

```rust
pub headers: Option<HashMap<String, String>>,
```

在 `hsi-core/src/downloader.rs` 的 `add_task_with_options` 中将 headers 传递到 Task 构造。

### 3.2 前端类型同步

`src-ui/src/lib/types.ts` 的 `AddTaskOptions` 添加 `headers?: Record<string, string>`。

## 4. 分块进度条 Bug 修复

### 4.1 根因

`download_internal`（`downloader.rs:406`）通过内部 channel 发送 `ProgressEvent::Initialized` 事件，task_id 为硬编码的 `"internal"`。

后端 progress listener（`downloader.rs:1080-1097`）收到此事件后：
- 正确更新了内部 Task HashMap 中的 `chunk_progress`（使用真实 task_id）
- **但没有将 `Initialized` 事件转发到 queue event channel**

因此前端的 `Initialized` 事件处理器（`task-store.ts:172-178`）永远收不到这个事件，`chunk_progress` 始终为 `null`。

暂停后恢复正常的原因：暂停触发 `refreshTasks()` → `getTasks()` 从后端 HashMap 读取当前状态，此时 `chunk_progress` 已在后端被更新。

### 4.2 修复方案

在 progress listener 的 `Initialized` 分支末尾，将事件用真实 task_id 转发到 queue event channel：

```rust
// downloader.rs, ProgressEvent::Initialized handler 末尾添加
let _ = queue_event_tx_clone
    .send(DownloaderEvent::Progress(ProgressEvent::Initialized {
        task_id: task_id_clone.clone(),
        total_size,
        chunks: (!chunk_progress.is_empty())
            .then_some(chunk_progress.clone()),
    }))
    .await;
```

## 5. 涉及文件

| 文件 | 变更 |
|------|------|
| `hsi-core/src/types.rs` | `AddTaskOptions` 添加 `headers` 字段 |
| `hsi-core/src/downloader.rs` | `add_task_with_options` 传递 headers；progress listener 转发 Initialized 事件 |
| `src-ui/src/lib/types.ts` | `AddTaskOptions` 添加 `headers` 字段 |
| `src-ui/src/components/AddTaskDialog.tsx` | 完全重写：Tab 切换 + 分层配置表单 |
| `src-ui/src/stores/task-store.ts` | 无需修改（已有 Initialized 处理器） |

## 6. 不做的事

- 不改全局配置页面
- 不添加每任务代理（proxy）设置
- 不修改 CLI TUI 的添加任务对话框
- 不修改任务列表/详情页的显示
