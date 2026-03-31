use yushi_core::DownloadTask;

pub fn progress_percent(task: &DownloadTask) -> f32 {
    if task.total_size == 0 {
        0.0
    } else {
        ((task.downloaded as f64 / task.total_size as f64) * 100.0).clamp(0.0, 100.0) as f32
    }
}
