use yushi_core::DownloadTask;

pub fn filter_tasks(
    tasks: &[DownloadTask],
    view: crate::app_state::ViewKind,
) -> Vec<&DownloadTask> {
    let mut tasks = tasks
        .iter()
        .filter(|task| match view {
            crate::app_state::ViewKind::AllTasks => true,
            crate::app_state::ViewKind::Downloading => {
                matches!(
                    task.status,
                    yushi_core::TaskStatus::Pending | yushi_core::TaskStatus::Downloading
                )
            }
            crate::app_state::ViewKind::Completed => {
                task.status == yushi_core::TaskStatus::Completed
            }
            crate::app_state::ViewKind::History | crate::app_state::ViewKind::Settings => false,
        })
        .collect::<Vec<_>>();

    tasks.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    tasks
}
