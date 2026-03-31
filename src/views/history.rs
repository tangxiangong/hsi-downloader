use yushi_core::{CompletedTask, DownloadHistory};

pub fn search_history(history: &DownloadHistory, query: &str) -> Vec<CompletedTask> {
    if query.trim().is_empty() {
        history.get_all().to_vec()
    } else {
        history.search(query)
    }
}
