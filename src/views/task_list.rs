use yushi_core::DownloadTask;

pub fn filter_tasks(tasks: &[DownloadTask], view: crate::state::ViewKind) -> Vec<&DownloadTask> {
    let mut tasks = tasks
        .iter()
        .filter(|task| match view {
            crate::state::ViewKind::AllTasks => true,
            crate::state::ViewKind::Downloading => {
                matches!(
                    task.status,
                    yushi_core::TaskStatus::Pending | yushi_core::TaskStatus::Downloading
                )
            }
            crate::state::ViewKind::Completed => task.status == yushi_core::TaskStatus::Completed,
            crate::state::ViewKind::History | crate::state::ViewKind::Settings => false,
        })
        .collect::<Vec<_>>();

    tasks.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    tasks
}
