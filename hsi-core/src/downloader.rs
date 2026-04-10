use crate::{
    Error, Result,
    bt::{BtEngine, detect_source, spawn_bt_progress_poller},
    config::BtConfig,
    state::{ChunkState, DownloadState, QueueState, current_timestamp},
    storage,
    types::{
        AddTaskOptions, BtTaskInfo, ChunkProgressInfo, CompletionCallback, Config, DownloadSource,
        DownloaderEvent, ProgressEvent, Task, TaskEvent, TaskPriority, TaskStatus, TorrentFileInfo,
        VerificationEvent,
    },
    utils::{
        SpeedCalculator, SpeedLimiter, auto_rename, infer_filename_from_content_disposition,
        infer_filename_from_url, verify_file,
    },
};
use fs_err::tokio as fs;
use futures::StreamExt;
use reqwest::{
    Client, Proxy,
    StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, RANGE, USER_AGENT},
};
use std::{
    collections::HashMap,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncSeekExt, AsyncWriteExt, SeekFrom},
    runtime::Handle,
    sync::{Mutex, RwLock, Semaphore, mpsc},
    task::JoinHandle,
    task::block_in_place,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Hsi {
    client: Arc<std::sync::RwLock<Client>>,
    config: Arc<std::sync::RwLock<Config>>,
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    active_downloads: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    max_concurrent_tasks: Arc<AtomicUsize>,
    queue_state_path: PathBuf,
    queue_state_save_tx: mpsc::Sender<QueueStateSaveSignal>,
    queue_state_write_lock: Arc<Mutex<()>>,
    queue_event_tx: mpsc::Sender<DownloaderEvent>,
    on_complete: Option<CompletionCallback>,
    bt_engine: Arc<RwLock<Option<Arc<BtEngine>>>>,
    bt_config: Arc<std::sync::RwLock<BtConfig>>,
}

const STATE_SAVE_INTERVAL: Duration = Duration::from_millis(750);
const STATE_SAVE_BYTES_THRESHOLD: u64 = 512 * 1024;
const QUEUE_STATE_SAVE_DEBOUNCE: Duration = Duration::from_millis(150);
const MAX_RETRIES: u32 = 5;
const MAX_RETRY_BACKOFF: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadPlan {
    Chunked { total_size: u64, resumed_bytes: u64 },
    Streaming { total_size: Option<u64>, resumed_bytes: u64 },
}

#[derive(Debug, Clone, Copy)]
enum QueueStateSaveSignal {
    Save,
}

impl std::fmt::Debug for Hsi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = self
            .config
            .read()
            .unwrap_or_else(|err| err.into_inner())
            .clone();

        f.debug_struct("Hsi")
            .field("config", &config)
            .field(
                "max_concurrent_tasks",
                &self.max_concurrent_tasks.load(Ordering::Relaxed),
            )
            .field("queue_state_path", &self.queue_state_path)
            .field("has_on_complete", &self.on_complete.is_some())
            .finish()
    }
}

impl Hsi {
    async fn sync_chunk_state(
        file: &mut fs::File,
        state_lock: &Arc<tokio::sync::RwLock<DownloadState>>,
        state_file: &Path,
    ) -> Result<()> {
        // Persist file contents before advancing the resumable state file.
        file.flush().await?;
        file.sync_data().await?;

        let state_snapshot = { state_lock.read().await.clone() };
        state_snapshot.save(state_file).await?;
        Ok(())
    }

    fn spawn_queue_state_save_worker(
        tasks: Arc<RwLock<HashMap<String, Task>>>,
        queue_state_path: PathBuf,
        queue_state_write_lock: Arc<Mutex<()>>,
        mut queue_state_save_rx: mpsc::Receiver<QueueStateSaveSignal>,
    ) {
        tokio::spawn(async move {
            let mut channel_closed = false;

            while !channel_closed {
                match queue_state_save_rx.recv().await {
                    Some(QueueStateSaveSignal::Save) => {}
                    None => break,
                }

                loop {
                    match tokio::time::timeout(
                        QUEUE_STATE_SAVE_DEBOUNCE,
                        queue_state_save_rx.recv(),
                    )
                    .await
                    {
                        Ok(Some(QueueStateSaveSignal::Save)) => continue,
                        Ok(None) => {
                            channel_closed = true;
                            break;
                        }
                        Err(_) => break,
                    }
                }

                let _ = Self::write_queue_state_snapshot_from_tasks(
                    &tasks,
                    &queue_state_path,
                    &queue_state_write_lock,
                )
                .await;
            }
        });
    }

    async fn write_queue_state_snapshot_from_tasks(
        tasks: &Arc<RwLock<HashMap<String, Task>>>,
        queue_state_path: &Path,
        queue_state_write_lock: &Arc<Mutex<()>>,
    ) -> Result<()> {
        let task_list: Vec<Task> = {
            let tasks = tasks.read().await;
            tasks.values().cloned().collect()
        };

        let state = QueueState {
            version: "1.0".to_string(),
            tasks: task_list,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };

        let _guard = queue_state_write_lock.lock().await;
        state.save(queue_state_path).await?;
        Ok(())
    }

    async fn write_queue_state_snapshot(&self) -> Result<()> {
        Self::write_queue_state_snapshot_from_tasks(
            &self.tasks,
            &self.queue_state_path,
            &self.queue_state_write_lock,
        )
        .await
    }

    async fn finalize_task(
        &self,
        task_id: String,
        verify_result: Result<()>,
        on_complete: Option<CompletionCallback>,
    ) {
        let callback_result = match &verify_result {
            Ok(_) => Ok(()),
            Err(error) => Err(error.to_string()),
        };

        let completion_event = {
            let mut tasks = self.tasks.write().await;
            let Some(task) = tasks.get_mut(&task_id) else {
                return;
            };

            match &verify_result {
                Ok(_) => {
                    task.status = TaskStatus::Completed;
                    task.error = None;
                    task.speed = 0;
                    task.eta = Some(0);
                    DownloaderEvent::Task(TaskEvent::Completed {
                        task_id: task_id.clone(),
                    })
                }
                Err(error) => {
                    let error_message = error.to_string();
                    task.status = TaskStatus::Failed;
                    task.error = Some(error_message.clone());
                    task.speed = 0;
                    task.eta = None;
                    DownloaderEvent::Task(TaskEvent::Failed {
                        task_id: task_id.clone(),
                        error: error_message,
                    })
                }
            }
        };

        let _ = self.write_queue_state_snapshot().await;

        if let Some(callback) = on_complete {
            callback(task_id.clone(), callback_result).await;
        }

        let _ = self.queue_event_tx.send(completion_event).await;
    }

    /// 创建新的下载器实例
    ///
    /// # 参数
    /// * `max_concurrent_downloads` - 每个任务的最大并发下载连接数
    /// * `max_concurrent_tasks` - 队列中同时运行的最大任务数
    /// * `queue_state_path` - 队列状态持久化文件路径
    ///
    /// # 返回
    /// 返回下载器实例和队列事件接收器
    pub fn new(
        max_concurrent_downloads: usize,
        max_concurrent_tasks: usize,
        queue_state_path: PathBuf,
    ) -> (Self, mpsc::Receiver<DownloaderEvent>) {
        let config = Config {
            max_concurrent: max_concurrent_downloads,
            ..Default::default()
        };
        Self::with_config(
            config,
            max_concurrent_tasks,
            queue_state_path,
            BtConfig::default(),
        )
    }

    /// 使用自定义配置创建下载器
    ///
    /// # 参数
    /// * `config` - 下载配置
    /// * `max_concurrent_tasks` - 队列中同时运行的最大任务数
    /// * `queue_state_path` - 队列状态持久化文件路径
    ///
    /// # 返回
    /// 返回下载器实例和队列事件接收器
    pub fn with_config(
        config: Config,
        max_concurrent_tasks: usize,
        queue_state_path: PathBuf,
        bt_config: BtConfig,
    ) -> (Self, mpsc::Receiver<DownloaderEvent>) {
        let (event_tx, event_rx) = mpsc::channel(1024);
        let (queue_state_save_tx, queue_state_save_rx) = mpsc::channel(64);
        let client = Self::build_client(&config).expect("failed to build reqwest client");
        let tasks = Arc::new(RwLock::new(HashMap::new()));
        let queue_state_write_lock = Arc::new(Mutex::new(()));

        let downloader = Self {
            client: Arc::new(std::sync::RwLock::new(client)),
            config: Arc::new(std::sync::RwLock::new(config)),
            tasks: Arc::clone(&tasks),
            active_downloads: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent_tasks: Arc::new(AtomicUsize::new(max_concurrent_tasks)),
            queue_state_path: queue_state_path.clone(),
            queue_state_save_tx,
            queue_state_write_lock: Arc::clone(&queue_state_write_lock),
            queue_event_tx: event_tx,
            on_complete: None,
            bt_engine: Arc::new(RwLock::new(None)),
            bt_config: Arc::new(std::sync::RwLock::new(bt_config)),
        };

        Self::spawn_queue_state_save_worker(
            tasks,
            queue_state_path,
            queue_state_write_lock,
            queue_state_save_rx,
        );
        (downloader, event_rx)
    }

    /// 设置下载完成回调
    pub fn set_on_complete<F, Fut>(&mut self, callback: F)
    where
        F: Fn(String, std::result::Result<(), String>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_complete = Some(Arc::new(move |task_id, result| {
            Box::pin(callback(task_id, result))
        }));
    }

    fn build_client(config: &Config) -> std::result::Result<Client, reqwest::Error> {
        let mut builder = Client::builder()
            .tcp_keepalive(Duration::from_secs(60))
            .timeout(Duration::from_secs(config.timeout));

        if let Some(proxy_url) = &config.proxy
            && let Ok(proxy) = Proxy::all(proxy_url)
        {
            builder = builder.proxy(proxy);
        }

        builder.build()
    }

    fn runtime_config(&self) -> Config {
        self.config
            .read()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }

    fn http_client(&self) -> Client {
        self.client
            .read()
            .unwrap_or_else(|err| err.into_inner())
            .clone()
    }

    pub async fn apply_runtime_config(
        &self,
        config: Config,
        max_concurrent_tasks: usize,
    ) -> Result<()> {
        let client = Self::build_client(&config)?;

        *self.client.write().unwrap_or_else(|err| err.into_inner()) = client;
        *self.config.write().unwrap_or_else(|err| err.into_inner()) = config;
        self.max_concurrent_tasks
            .store(max_concurrent_tasks, Ordering::Relaxed);

        self.process_queue().await?;
        Ok(())
    }

    async fn ensure_bt_engine(&self) -> Result<Arc<BtEngine>> {
        {
            let engine = self.bt_engine.read().await;
            if let Some(ref e) = *engine {
                return Ok(Arc::clone(e));
            }
        }
        let bt_config = self
            .bt_config
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let output_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let engine = Arc::new(BtEngine::new(output_dir, &bt_config).await?);
        let mut guard = self.bt_engine.write().await;
        *guard = Some(Arc::clone(&engine));
        Ok(engine)
    }

    pub async fn infer_destination_in_dir(&self, url: &str, directory: PathBuf) -> PathBuf {
        let file_name = self
            .infer_remote_filename(url)
            .await
            .or_else(|| infer_filename_from_url(url))
            .unwrap_or_else(|| "download".to_string());

        directory.join(file_name)
    }

    async fn infer_remote_filename(&self, url: &str) -> Option<String> {
        let response = self.http_client().head(url).send().await.ok()?;

        response
            .headers()
            .get(CONTENT_DISPOSITION)
            .and_then(|value| value.to_str().ok())
            .and_then(infer_filename_from_content_disposition)
            .or_else(|| infer_filename_from_url(response.url().as_str()))
    }

    /// 简单下载文件（单文件下载的便捷方法）
    ///
    /// # 参数
    /// * `url` - 下载 URL
    /// * `dest` - 目标文件路径
    /// * `event_tx` - 进度事件发送器（可选）
    pub async fn download(
        &self,
        url: &str,
        dest: &str,
        event_tx: Option<mpsc::Sender<ProgressEvent>>,
    ) -> Result<()> {
        // 添加任务到队列
        let task_id = self.add_task(url.to_string(), PathBuf::from(dest)).await?;

        // 等待任务完成
        loop {
            let task = self.get_task(&task_id).await;
            if let Some(task) = task {
                match task.status {
                    TaskStatus::Completed => return Ok(()),
                    TaskStatus::Failed => {
                        return Err(Error::TaskFailed(
                            task.error.unwrap_or_else(|| "Unknown error".to_string()),
                        ));
                    }
                    TaskStatus::Cancelled => {
                        return Err(Error::TaskCancelled);
                    }
                    _ => {
                        // 如果提供了进度事件发送器，发送进度更新
                        if let Some(tx) = &event_tx {
                            if task.total_size > 0 {
                                tx.send(ProgressEvent::Updated {
                                    task_id: task.id.clone(),
                                    downloaded: task.downloaded,
                                    total: task.total_size,
                                    speed: task.speed,
                                    eta: task.eta,
                                })
                                .await?;
                            } else {
                                tx.send(ProgressEvent::StreamDownloading {
                                    downloaded: task.downloaded,
                                })
                                .await?;
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            } else {
                return Err(Error::TaskNotFound);
            }
        }
    }

    /// 内部下载方法（由队列任务调用）
    ///
    /// # 参数
    /// * `url` - 下载 URL
    /// * `dest` - 目标文件路径
    /// * `event_tx` - 进度事件发送器
    async fn download_internal(
        &self,
        url: &str,
        dest: &str,
        event_tx: mpsc::Sender<ProgressEvent>,
        speed_limit: Option<u64>,
    ) -> Result<()> {
        let dest_path = PathBuf::from(dest);
        let state_path = storage::migrate_download_state_file(&dest_path).await?;

        let state = self
            .get_or_create_state(url, &dest_path, &state_path)
            .await?;
        let state = Arc::new(RwLock::new(state));

        let (total_size, is_streaming) = {
            let s = state.read().await;
            (s.total_size, s.is_streaming)
        };

        let chunk_progress = if is_streaming {
            None
        } else {
            let state_snapshot = state.read().await;
            Some(
                state_snapshot
                    .chunks
                    .iter()
                    .map(|chunk| ChunkProgressInfo {
                        index: chunk.index,
                        downloaded: chunk.current.saturating_sub(chunk.start),
                        size: chunk.end.saturating_sub(chunk.start) + 1,
                        complete: chunk.is_finished,
                    })
                    .collect(),
            )
        };

        event_tx
            .send(ProgressEvent::Initialized {
                task_id: "internal".to_string(),
                total_size,
                chunks: chunk_progress,
            })
            .await?;

        if is_streaming {
            self.download_streaming(state, url, &dest_path, &state_path, event_tx, speed_limit)
                .await
        } else {
            self.download_chunked(state, &dest_path, &state_path, event_tx, speed_limit)
                .await
        }
    }

    /// 流式下载（不需要 Content-Length）
    async fn download_streaming(
        &self,
        state: Arc<tokio::sync::RwLock<DownloadState>>,
        url: &str,
        dest: &Path,
        state_path: &Path,
        event_tx: mpsc::Sender<ProgressEvent>,
        speed_limit: Option<u64>,
    ) -> Result<()> {
        let client = self.http_client();
        let config = self.runtime_config();
        {
            let mut state = state.write().await;
            state.downloaded = 0;
        }
        let mut request = client.get(url);

        // 添加自定义头
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        // 添加 User-Agent
        if let Some(ua) = &config.user_agent {
            request = request.header(USER_AGENT, ua);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(Error::HttpError(response.status().to_string()));
        }

        let mut file = fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        let mut unsaved_bytes = 0u64;
        let mut last_state_save = Instant::now();
        let speed_limiter =
            speed_limit.map(|limit| Arc::new(RwLock::new(SpeedLimiter::new(limit))));

        while let Some(item) = stream.next().await {
            let chunk_data = item.map_err(|e| Error::StreamError(e.to_string()))?;
            file.write_all(&chunk_data).await?;

            let len = chunk_data.len() as u64;
            downloaded += len;

            if let Some(speed_limiter) = &speed_limiter {
                speed_limiter.write().await.wait(len).await;
            }

            {
                let mut state = state.write().await;
                state.downloaded = downloaded;
            }

            let _ = event_tx
                .send(ProgressEvent::StreamDownloading { downloaded })
                .await;

            unsaved_bytes += len;
            if unsaved_bytes >= STATE_SAVE_BYTES_THRESHOLD
                || last_state_save.elapsed() >= STATE_SAVE_INTERVAL
            {
                Self::sync_chunk_state(&mut file, &state, state_path).await?;
                unsaved_bytes = 0;
                last_state_save = Instant::now();
            }
        }

        Self::sync_chunk_state(&mut file, &state, state_path).await?;
        let _ = fs::remove_file(state_path).await;
        event_tx
            .send(ProgressEvent::Finished {
                task_id: "internal".to_string(),
            })
            .await?;
        Ok(())
    }

    /// 分块下载（需要 Content-Length）
    async fn download_chunked(
        &self,
        state: Arc<tokio::sync::RwLock<DownloadState>>,
        dest_path: &Path,
        state_path: &Path,
        event_tx: mpsc::Sender<ProgressEvent>,
        speed_limit: Option<u64>,
    ) -> Result<()> {
        let config = self.runtime_config();
        let client = self.http_client();
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        let speed_limiter =
            speed_limit.map(|limit| Arc::new(RwLock::new(SpeedLimiter::new(limit))));
        let mut workers = Vec::new();

        let (chunks_count, url) = {
            let s = state.read().await;
            (s.chunks.len(), s.url.clone())
        };

        for i in 0..chunks_count {
            let permit = semaphore.clone().acquire_owned().await?;
            let state_c = Arc::clone(&state);
            let client_c = client.clone();
            let url_c = url.clone();
            let dest_c = dest_path.to_path_buf();
            let state_file_c = state_path.to_path_buf();
            let tx_c = event_tx.clone();
            let speed_limiter_c = speed_limiter.clone();
            let headers = config.headers.clone();
            let user_agent = config.user_agent.clone();

            workers.push(tokio::spawn(async move {
                let res = Self::download_chunk(
                    i,
                    client_c,
                    &url_c,
                    &dest_c,
                    &state_file_c,
                    state_c,
                    tx_c,
                    speed_limiter_c,
                    headers,
                    user_agent,
                )
                .await;
                drop(permit);
                res
            }));
        }

        for worker in workers {
            worker.await??;
        }

        fs::remove_file(state_path).await?;
        event_tx
            .send(ProgressEvent::Finished {
                task_id: "internal".to_string(),
            })
            .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    /// 下载单个分块
    async fn download_chunk(
        index: usize,
        client: reqwest::Client,
        url: &str,
        dest: &Path,
        state_file: &Path,
        state_lock: Arc<tokio::sync::RwLock<DownloadState>>,
        tx: mpsc::Sender<ProgressEvent>,
        speed_limiter: Option<Arc<RwLock<SpeedLimiter>>>,
        headers: std::collections::HashMap<String, String>,
        user_agent: Option<String>,
    ) -> Result<()> {
        let mut retry_count = 0;

        loop {
            let (start_pos, end_pos) = {
                let s = state_lock.read().await;
                let chunk = &s.chunks[index];
                if chunk.is_finished {
                    return Ok(());
                }
                (chunk.current, chunk.end)
            };

            let mut request = client
                .get(url)
                .header(RANGE, format!("bytes={}-{}", start_pos, end_pos));

            // 添加自定义头
            for (key, value) in &headers {
                request = request.header(key, value);
            }

            // 添加 User-Agent
            if let Some(ua) = &user_agent {
                request = request.header(USER_AGENT, ua);
            }

            let res = request.send().await;

            match res {
                Ok(resp) if resp.status().is_success() => {
                    let download_result: Result<()> = async {
                        if !is_valid_chunk_response(
                            resp.status(),
                            resp.headers()
                                .get(CONTENT_RANGE)
                                .and_then(|value| value.to_str().ok()),
                            start_pos,
                            end_pos,
                        ) {
                            return Err(Error::HttpError(format!(
                                "Chunk {} received invalid partial response: status={} range={:?}",
                                index,
                                resp.status(),
                                resp.headers().get(CONTENT_RANGE)
                            )));
                        }

                        let mut file = fs::OpenOptions::new().write(true).open(&dest).await?;
                        file.seek(SeekFrom::Start(start_pos)).await?;

                        let mut stream = resp.bytes_stream();
                        let mut current_idx = start_pos;
                        let mut unsaved_bytes = 0u64;
                        let mut last_state_save = Instant::now();

                        while let Some(item) = stream.next().await {
                            let chunk_data = item.map_err(|e| Error::StreamError(e.to_string()))?;
                            let next_idx = current_idx.saturating_add(chunk_data.len() as u64);
                            if next_idx > end_pos.saturating_add(1) {
                                return Err(Error::HttpError(format!(
                                    "Chunk {} exceeded requested range {}-{}",
                                    index, start_pos, end_pos
                                )));
                            }
                            file.write_all(&chunk_data).await?;

                            let len = chunk_data.len() as u64;
                            current_idx = next_idx;

                            if let Some(speed_limiter) = &speed_limiter {
                                speed_limiter.write().await.wait(len).await;
                            }

                            {
                                let mut s = state_lock.write().await;
                                s.chunks[index].current = current_idx;
                            }

                            let _ = tx
                                .send(ProgressEvent::ChunkDownloading {
                                    chunk_index: index,
                                    delta: len,
                                })
                                .await;

                            unsaved_bytes += len;
                            if unsaved_bytes >= STATE_SAVE_BYTES_THRESHOLD
                                || last_state_save.elapsed() >= STATE_SAVE_INTERVAL
                            {
                                Self::sync_chunk_state(&mut file, &state_lock, state_file).await?;
                                unsaved_bytes = 0;
                                last_state_save = Instant::now();
                            }
                        }

                        if current_idx != end_pos.saturating_add(1) {
                            return Err(Error::HttpError(format!(
                                "Chunk {} ended early: expected {} bytes, got {}",
                                index,
                                end_pos.saturating_sub(start_pos).saturating_add(1),
                                current_idx.saturating_sub(start_pos)
                            )));
                        }

                        {
                            let mut s = state_lock.write().await;
                            s.chunks[index].current = current_idx;
                            s.chunks[index].is_finished = true;
                        }
                        Self::sync_chunk_state(&mut file, &state_lock, state_file).await?;
                        Ok(())
                    }
                    .await;

                    match download_result {
                        Ok(()) => return Ok(()),
                        Err(err) => {
                            retry_count += 1;
                            if retry_count > MAX_RETRIES {
                                return Err(err);
                            }
                            tokio::time::sleep(retry_delay(retry_count)).await;
                        }
                    }
                }
                Ok(resp) => {
                    retry_count += 1;
                    if retry_count > MAX_RETRIES {
                        return Err(Error::HttpError(format!(
                            "Chunk {} failed after {} retries: HTTP {}",
                            index,
                            MAX_RETRIES,
                            resp.status()
                        )));
                    }
                    tokio::time::sleep(retry_delay(retry_count)).await;
                }
                Err(err) => {
                    retry_count += 1;
                    if retry_count > MAX_RETRIES {
                        return Err(Error::ReqwestError(format!(
                            "Chunk {} failed after {} retries: {}",
                            index, MAX_RETRIES, err
                        )));
                    }
                    tokio::time::sleep(retry_delay(retry_count)).await;
                }
            }
        }
    }

    async fn probe_download_capability(&self, url: &str) -> Result<(Option<u64>, bool)> {
        let client = self.http_client();
        let config = self.runtime_config();
        let mut request = client.head(url);

        for (key, value) in &config.headers {
            request = request.header(key, value);
        }

        if let Some(ua) = &config.user_agent {
            request = request.header(USER_AGENT, ua);
        }

        let response = request.send().await?;
        let total_size = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok()?.parse::<u64>().ok());
        let supports_range = response
            .headers()
            .get("accept-ranges")
            .map(|v| v.to_str().unwrap_or("").contains("bytes"))
            .unwrap_or(false);

        Ok((total_size, supports_range))
    }

    async fn existing_downloaded_bytes(dest: &Path, fallback: u64) -> Result<u64> {
        match fs::metadata(dest).await {
            Ok(metadata) => Ok(metadata.len()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(fallback),
            Err(err) => Err(err.into()),
        }
    }

    async fn build_chunked_state(
        &self,
        url: &str,
        dest: &Path,
        state_path: &Path,
        total_size: u64,
        existing_downloaded: u64,
    ) -> Result<DownloadState> {
        let config = self.runtime_config();
        let mut options = fs::OpenOptions::new();
        options.create(true).write(true);
        let file = options.open(dest).await?;
        file.set_len(total_size).await?;

        let resumed_bytes = existing_downloaded.min(total_size);
        let mut chunks = Vec::new();
        let mut curr = 0;
        let mut idx = 0;

        while curr < total_size {
            let end = (curr + config.chunk_size - 1).min(total_size - 1);
            let current = resumed_bytes.clamp(curr, end + 1);

            chunks.push(ChunkState {
                index: idx,
                start: curr,
                end,
                current,
                is_finished: current > end,
            });

            curr += config.chunk_size;
            idx += 1;
        }

        let state = DownloadState {
            url: url.to_string(),
            total_size: Some(total_size),
            downloaded: resumed_bytes,
            chunks,
            is_streaming: false,
        };
        state.save(state_path).await?;
        Ok(state)
    }

    async fn build_streaming_state(
        &self,
        url: &str,
        state_path: &Path,
        total_size: Option<u64>,
        existing_downloaded: u64,
    ) -> Result<DownloadState> {
        let state = DownloadState {
            url: url.to_string(),
            total_size,
            downloaded: existing_downloaded,
            chunks: Vec::new(),
            is_streaming: true,
        };
        state.save(state_path).await?;
        Ok(state)
    }

    /// 获取或创建下载状态
    async fn get_or_create_state(
        &self,
        url: &str,
        dest: &Path,
        state_path: &Path,
    ) -> Result<DownloadState> {
        if let Some(state) = DownloadState::load(state_path).await?
            && state.url == url
        {
            if state.is_streaming {
                let existing_downloaded =
                    Self::existing_downloaded_bytes(dest, state.downloaded).await?;
                let (total_size, supports_range) = self.probe_download_capability(url).await?;
                return match choose_download_plan(
                    total_size.or(state.total_size),
                    supports_range,
                    existing_downloaded,
                ) {
                    DownloadPlan::Chunked {
                        total_size,
                        resumed_bytes,
                    } => {
                        self.build_chunked_state(url, dest, state_path, total_size, resumed_bytes)
                            .await
                    }
                    DownloadPlan::Streaming {
                        total_size,
                        resumed_bytes,
                    } => {
                        self.build_streaming_state(url, state_path, total_size, resumed_bytes)
                            .await
                    }
                };
            }

            return Ok(state);
        }

        let existing_downloaded = Self::existing_downloaded_bytes(dest, 0).await?;
        let (total_size_opt, supports_range) = self.probe_download_capability(url).await?;
        match choose_download_plan(total_size_opt, supports_range, existing_downloaded) {
            DownloadPlan::Chunked {
                total_size,
                resumed_bytes,
            } => {
                self.build_chunked_state(url, dest, state_path, total_size, resumed_bytes)
                    .await
            }
            DownloadPlan::Streaming {
                total_size,
                resumed_bytes,
            } => {
                self.build_streaming_state(url, state_path, total_size, resumed_bytes)
                    .await
            }
        }
    }

    // ==================== 队列管理方法 ====================

    /// 从持久化状态加载队列
    pub async fn load_queue_from_state(&self) -> Result<()> {
        if let Some(state) = QueueState::load(&self.queue_state_path).await? {
            let mut tasks = self.tasks.write().await;
            for mut task in state.tasks {
                normalize_loaded_task(&mut task);
                tasks.insert(task.id.clone(), task);
            }
        }
        Ok(())
    }

    /// 保存队列状态
    async fn save_queue_state(&self) -> Result<()> {
        self.queue_state_save_tx
            .send(QueueStateSaveSignal::Save)
            .await?;
        Ok(())
    }

    pub async fn persist_queue_state(&self) -> Result<()> {
        self.write_queue_state_snapshot().await
    }

    pub async fn start_pending_tasks(&self) -> Result<()> {
        self.process_queue().await
    }

    /// 添加下载任务到队列
    ///
    /// # 参数
    /// * `url` - 下载 URL
    /// * `dest` - 目标文件路径
    ///
    /// # 返回
    /// 返回任务 ID
    pub async fn add_task(&self, url: String, dest: PathBuf) -> Result<String> {
        let speed_limit = self.runtime_config().speed_limit;
        self.add_task_with_options(AddTaskOptions {
            url,
            dest,
            priority: Some(TaskPriority::Normal),
            checksum: None,
            speed_limit,
            auto_rename_on_conflict: false,
            selected_files: None,
            headers: None,
        })
        .await
    }

    /// 添加下载任务到队列（带选项）
    ///
    /// # 参数
    /// * `url` - 下载 URL
    /// * `dest` - 目标文件路径
    /// * `priority` - 任务优先级
    /// * `checksum` - 文件校验（可选）
    /// * `auto_rename_on_conflict` - 是否自动重命名冲突文件
    ///
    /// # 返回
    /// 返回任务 ID
    pub async fn add_task_with_options(&self, mut options: AddTaskOptions) -> Result<String> {
        let speed_limit = options.speed_limit.or(self.runtime_config().speed_limit);

        // 自动重命名
        if options.auto_rename_on_conflict && options.dest.exists() {
            options.dest = auto_rename(&options.dest);
        }

        let task_id = Uuid::new_v4().to_string();

        let source = detect_source(&options.url);
        let task = Task {
            id: task_id.clone(),
            url: options.url.clone(),
            dest: options.dest.clone(),
            status: TaskStatus::Pending,
            total_size: 0,
            downloaded: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            error: None,
            priority: options.priority.unwrap_or(TaskPriority::Normal),
            speed: 0,
            eta: None,
            headers: options.headers.clone().unwrap_or_default(),
            checksum: options.checksum.clone(),
            speed_limit,
            source,
            bt_info: options.selected_files.clone().map(|files| BtTaskInfo {
                selected_files: Some(files),
                ..Default::default()
            }),
            chunk_progress: None,
        };

        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task);
        }

        self.save_queue_state().await?;
        let _ = self
            .queue_event_tx
            .send(DownloaderEvent::Task(TaskEvent::Added {
                task_id: task_id.clone(),
            }))
            .await;

        // 尝试启动任务
        self.process_queue().await?;

        Ok(task_id)
    }

    /// 列出种子中的文件信息（用于选择性下载）
    pub async fn list_torrent_files(&self, uri: &str) -> Result<Vec<TorrentFileInfo>> {
        let engine = self.ensure_bt_engine().await?;
        engine.list_torrent_files(uri).await
    }

    /// 处理队列，启动待处理的任务（按优先级排序）
    async fn process_queue(&self) -> Result<()> {
        let active_count = self.active_downloads.read().await.len();
        let max_concurrent_tasks = self.max_concurrent_tasks.load(Ordering::Relaxed);
        if active_count >= max_concurrent_tasks {
            return Ok(());
        }

        let mut pending_tasks: Vec<(String, TaskPriority)> = {
            let tasks = self.tasks.read().await;
            tasks
                .values()
                .filter(|t| t.status == TaskStatus::Pending)
                .map(|t| (t.id.clone(), t.priority))
                .collect()
        };

        // 按优先级排序（高优先级在前）
        pending_tasks.sort_by(|a, b| b.1.cmp(&a.1));

        let task_ids: Vec<String> = pending_tasks
            .into_iter()
            .take(max_concurrent_tasks - active_count)
            .map(|(task_id, _)| task_id)
            .collect();

        for task_id in task_ids {
            self.start_queue_task(&task_id).await?;
        }

        Ok(())
    }

    /// 启动单个队列任务
    async fn start_queue_task(&self, task_id: &str) -> Result<()> {
        let task = {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(Error::TaskNotFound)?;

            if task.status != TaskStatus::Pending && task.status != TaskStatus::Paused {
                return Ok(());
            }

            task.status = TaskStatus::Downloading;
            task.clone()
        };

        self.save_queue_state().await?;
        let _ = self
            .queue_event_tx
            .send(DownloaderEvent::Task(TaskEvent::Started {
                task_id: task_id.to_string(),
            }))
            .await;

        let downloader = self.clone();
        let tasks = Arc::clone(&self.tasks);
        let active_downloads = Arc::clone(&self.active_downloads);
        let queue_event_tx = self.queue_event_tx.clone();
        let task_id_owned = task_id.to_string();
        let on_complete = self.on_complete.clone();
        // 记录暂停前已下载的字节数，用于断点续传时正确累加进度
        let initial_downloaded = task.downloaded;

        let handle = tokio::spawn(async move {
            let (tx, mut rx) = mpsc::channel(1024);
            let task_id_clone = task_id_owned.clone();
            let queue_event_tx_clone = queue_event_tx.clone();
            let tasks_clone = Arc::clone(&tasks);

            // 进度监听器
            tokio::spawn(async move {
                let mut total = 0u64;
                let mut chunk_progress = Vec::<ChunkProgressInfo>::new();
                // 本次会话新增的字节数（从 0 开始），用于速度计算
                // 历史已下载字节通过 initial_downloaded 偏移量补偿
                let mut session_downloaded = 0u64;
                let mut speed_calc = SpeedCalculator::new();

                while let Some(event) = rx.recv().await {
                    match event {
                        ProgressEvent::Initialized {
                            total_size, chunks, ..
                        } => {
                            if let Some(size) = total_size {
                                total = size;
                            }
                            if let Some(chunks) = chunks {
                                chunk_progress = chunks;
                            }
                            {
                                let mut tasks = tasks_clone.write().await;
                                if let Some(task) = tasks.get_mut(&task_id_clone) {
                                    task.total_size = total_size.unwrap_or(0);
                                    task.chunk_progress = (!chunk_progress.is_empty())
                                        .then_some(chunk_progress.clone());
                                }
                            }
                            // 将 Initialized 事件转发到前端，使分块进度条在首次下载时即可显示
                            let _ = queue_event_tx_clone
                                .send(DownloaderEvent::Progress(ProgressEvent::Initialized {
                                    task_id: task_id_clone.clone(),
                                    total_size,
                                    chunks: (!chunk_progress.is_empty())
                                        .then_some(chunk_progress.clone()),
                                }))
                                .await;
                        }
                        ProgressEvent::ChunkDownloading { chunk_index, delta } => {
                            session_downloaded += delta;
                            // 断点续传：加上历史偏移量，得到文件维度的真实进度
                            let total_downloaded = initial_downloaded + session_downloaded;

                            // 速度统计基于本次会话字节，避免历史字节拉高初始速度
                            let speed = speed_calc.update(session_downloaded);
                            let eta = if total > 0 {
                                speed_calc.calculate_eta(total_downloaded, total)
                            } else {
                                None
                            };

                            {
                                let mut tasks = tasks_clone.write().await;
                                if let Some(task) = tasks.get_mut(&task_id_clone) {
                                    task.downloaded = total_downloaded;
                                    task.speed = speed;
                                    task.eta = eta;
                                    if let Some(chunk) = chunk_progress.get_mut(chunk_index) {
                                        chunk.downloaded =
                                            (chunk.downloaded + delta).min(chunk.size);
                                        chunk.complete = chunk.downloaded >= chunk.size;
                                    }
                                    task.chunk_progress = (!chunk_progress.is_empty())
                                        .then_some(chunk_progress.clone());
                                }
                            }

                            if let Some(chunk) = chunk_progress.get(chunk_index) {
                                let _ = queue_event_tx_clone
                                    .send(DownloaderEvent::Progress(ProgressEvent::ChunkProgress {
                                        task_id: task_id_clone.clone(),
                                        chunk_index,
                                        downloaded: chunk.downloaded,
                                        size: chunk.size,
                                        complete: chunk.complete,
                                    }))
                                    .await;
                            }

                            let _ = queue_event_tx_clone
                                .send(DownloaderEvent::Progress(ProgressEvent::Updated {
                                    task_id: task_id_clone.clone(),
                                    downloaded: total_downloaded,
                                    total,
                                    speed,
                                    eta,
                                }))
                                .await;
                        }
                        ProgressEvent::StreamDownloading {
                            downloaded: stream_downloaded,
                        } => {
                            let session_downloaded =
                                stream_downloaded.saturating_sub(initial_downloaded);
                            let speed = speed_calc.update(session_downloaded);

                            {
                                let mut tasks = tasks_clone.write().await;
                                if let Some(task) = tasks.get_mut(&task_id_clone) {
                                    task.downloaded = stream_downloaded;
                                    task.speed = speed;
                                    task.eta = None; // 流式下载无法预估剩余时间
                                    task.chunk_progress = None;
                                }
                            }

                            let _ = queue_event_tx_clone
                                .send(DownloaderEvent::Progress(ProgressEvent::Updated {
                                    task_id: task_id_clone.clone(),
                                    downloaded: stream_downloaded,
                                    total: 0, // 流式下载时 total 为 0
                                    speed,
                                    eta: None,
                                }))
                                .await;
                        }
                        ProgressEvent::Finished { .. } => {}
                        ProgressEvent::Failed { .. } => {}
                        ProgressEvent::Updated { .. } => {}
                        ProgressEvent::ChunkProgress { .. } => {}
                        ProgressEvent::StreamProgress { .. } => {}
                        ProgressEvent::BtStatus { .. } => {}
                    }
                }
            });

            // 执行下载
            let result = match &task.source {
                DownloadSource::BitTorrent { uri } => {
                    let uri = uri.clone();
                    let selected_files =
                        task.bt_info.as_ref().and_then(|b| b.selected_files.clone());

                    async {
                        let engine = downloader.ensure_bt_engine().await?;
                        // BT 任务的 dest 是输出目录（不是文件路径）
                        let output_folder = Some(task.dest.to_string_lossy().to_string());

                        let (total_size, _name) = engine
                            .add_torrent(&task_id_owned, &uri, output_folder, selected_files)
                            .await?;

                        if let Some(total) = total_size {
                            let mut tasks_guard = tasks.write().await;
                            if let Some(t) = tasks_guard.get_mut(&task_id_owned) {
                                t.total_size = total;
                            }
                        }

                        let seed_ratio = downloader
                            .bt_config
                            .read()
                            .unwrap_or_else(|e| e.into_inner())
                            .seed_ratio;

                        let poller = spawn_bt_progress_poller(
                            engine,
                            task_id_owned.clone(),
                            queue_event_tx.clone(),
                            Arc::clone(&tasks),
                            seed_ratio,
                        );
                        let _ = poller.await;
                        Ok(())
                    }
                    .await
                }
                DownloadSource::Http { .. } => {
                    downloader
                        .download_internal(
                            &task.url,
                            task.dest.to_str().unwrap(),
                            tx,
                            task.speed_limit,
                        )
                        .await
                }
            };

            // 文件校验
            let checksum = task.checksum.clone();
            let dest_path = task.dest.clone();
            let verify_result = if result.is_ok() {
                if let Some(checksum_value) = checksum {
                    let _ = queue_event_tx
                        .send(DownloaderEvent::Verification(VerificationEvent::Started {
                            task_id: task_id_owned.clone(),
                        }))
                        .await;

                    match verify_file(&dest_path, &checksum_value).await {
                        Ok(success) => {
                            let _ = queue_event_tx
                                .send(DownloaderEvent::Verification(
                                    VerificationEvent::Completed {
                                        task_id: task_id_owned.clone(),
                                        success,
                                    },
                                ))
                                .await;
                            if success {
                                Ok(())
                            } else {
                                Err(Error::ChecksumVerificationFailed)
                            }
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    result
                }
            } else {
                result
            };

            downloader
                .finalize_task(task_id_owned.clone(), verify_result, on_complete)
                .await;

            // 从活动下载中移除
            active_downloads.write().await.remove(&task_id_owned);
            let refiller = downloader.clone();
            block_in_place(move || {
                let _ = Handle::current().block_on(async move { refiller.process_queue().await });
            });
        });

        self.active_downloads
            .write()
            .await
            .insert(task_id.to_string(), handle);

        Ok(())
    }

    /// 暂停任务
    pub async fn pause_task(&self, task_id: &str) -> Result<()> {
        let is_bt = {
            let tasks = self.tasks.read().await;
            let task = tasks.get(task_id).ok_or(Error::TaskNotFound)?;
            matches!(task.source, DownloadSource::BitTorrent { .. })
        };

        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(task_id).ok_or(Error::TaskNotFound)?;

        if task.status == TaskStatus::Downloading {
            if is_bt {
                if let Some(engine) = self.bt_engine.read().await.as_ref() {
                    engine.pause(task_id).await?;
                }
            } else {
                let mut active = self.active_downloads.write().await;
                if let Some(handle) = active.remove(task_id) {
                    handle.abort();
                }
            }

            task.status = TaskStatus::Paused;
            drop(tasks);

            self.save_queue_state().await?;
            let _ = self
                .queue_event_tx
                .send(DownloaderEvent::Task(TaskEvent::Paused {
                    task_id: task_id.to_string(),
                }))
                .await;

            if !is_bt {
                self.process_queue().await?;
            }
        }

        Ok(())
    }

    /// 恢复任务
    pub async fn resume_task(&self, task_id: &str) -> Result<()> {
        let is_bt = {
            let tasks = self.tasks.read().await;
            let task = tasks.get(task_id).ok_or(Error::TaskNotFound)?;
            matches!(task.source, DownloadSource::BitTorrent { .. })
        };

        {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(Error::TaskNotFound)?;

            if task.status == TaskStatus::Paused {
                if is_bt {
                    if let Some(engine) = self.bt_engine.read().await.as_ref() {
                        engine.resume(task_id).await?;
                    }
                    task.status = TaskStatus::Downloading;
                } else {
                    task.status = TaskStatus::Pending;
                }
                drop(tasks);

                self.save_queue_state().await?;
                let _ = self
                    .queue_event_tx
                    .send(DownloaderEvent::Task(TaskEvent::Resumed {
                        task_id: task_id.to_string(),
                    }))
                    .await;
            }
        }

        if !is_bt {
            self.process_queue().await?;
        }
        Ok(())
    }

    /// 重试失败的任务
    pub async fn retry_task(&self, task_id: &str) -> Result<()> {
        {
            let mut tasks = self.tasks.write().await;
            let task = tasks.get_mut(task_id).ok_or(Error::TaskNotFound)?;

            if task.status != TaskStatus::Failed {
                return Err(Error::InternalError(
                    "only failed tasks can be retried".into(),
                ));
            }

            task.status = TaskStatus::Pending;
            task.error = None;
            task.speed = 0;
            task.eta = None;
        }

        self.save_queue_state().await?;
        let _ = self
            .queue_event_tx
            .send(DownloaderEvent::Task(TaskEvent::Resumed {
                task_id: task_id.to_string(),
            }))
            .await;

        self.process_queue().await?;
        Ok(())
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let is_bt = {
            let tasks = self.tasks.read().await;
            tasks
                .get(task_id)
                .map(|t| matches!(t.source, DownloadSource::BitTorrent { .. }))
                .unwrap_or(false)
        };

        if is_bt {
            if let Some(engine) = self.bt_engine.read().await.as_ref() {
                let _ = engine.cancel(task_id, true).await;
            }
        } else {
            let mut active = self.active_downloads.write().await;
            if let Some(handle) = active.remove(task_id) {
                handle.abort();
            }
            drop(active);
        }

        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = TaskStatus::Cancelled;
            if !is_bt {
                let _ = fs::remove_file(&task.dest).await;
                if let Ok(state_path) = storage::download_state_path(&task.dest) {
                    let _ = fs::remove_file(state_path).await;
                }
                let _ = fs::remove_file(task.dest.with_extension("json")).await;
            }
        }
        drop(tasks);

        self.save_queue_state().await?;
        let _ = self
            .queue_event_tx
            .send(DownloaderEvent::Task(TaskEvent::Cancelled {
                task_id: task_id.to_string(),
            }))
            .await;

        self.process_queue().await?;

        Ok(())
    }

    /// 移除已完成或已取消的任务
    pub async fn remove_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get(task_id)
            && (task.status == TaskStatus::Completed
                || task.status == TaskStatus::Cancelled
                || task.status == TaskStatus::Failed)
        {
            tasks.remove(task_id);
            drop(tasks);
            self.save_queue_state().await?;
            return Ok(());
        }
        Err(Error::CannotRemoveTaskInCurrentStatus)
    }

    /// 删除任务对应的本地文件，并将该任务从队列中移除
    pub async fn remove_task_with_file(&self, task_id: &str) -> Result<()> {
        let task = {
            let tasks = self.tasks.read().await;
            let Some(task) = tasks.get(task_id) else {
                return Err(Error::TaskNotFound);
            };

            if !(task.status == TaskStatus::Completed
                || task.status == TaskStatus::Cancelled
                || task.status == TaskStatus::Failed)
            {
                return Err(Error::CannotRemoveTaskInCurrentStatus);
            }

            task.clone()
        };

        remove_file_if_exists(&task.dest).await?;
        if let Ok(state_path) = storage::download_state_path(&task.dest) {
            remove_file_if_exists(&state_path).await?;
        }
        remove_file_if_exists(&task.dest.with_extension("json")).await?;
        self.remove_task(task_id).await
    }

    /// 获取所有任务
    pub async fn get_all_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// 获取单个任务
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// 清空所有已完成的任务
    pub async fn clear_completed(&self) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.retain(|_, task| task.status != TaskStatus::Completed);
        drop(tasks);
        self.save_queue_state().await?;
        Ok(())
    }
}

async fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::metadata(path).await {
        Ok(metadata) if metadata.is_dir() => match fs::remove_dir_all(path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        },
        Ok(_) => match fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        },
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn retry_delay(retry_count: u32) -> Duration {
    let seconds = 2u64.saturating_pow(retry_count.saturating_sub(1));
    Duration::from_secs(seconds).min(MAX_RETRY_BACKOFF)
}

fn normalize_loaded_task(task: &mut Task) {
    if matches!(task.status, TaskStatus::Downloading) {
        task.status = TaskStatus::Pending;
        task.error = None;
    }

    if matches!(task.status, TaskStatus::Pending | TaskStatus::Paused) {
        task.speed = 0;
        task.eta = None;
    }
}

fn choose_download_plan(
    total_size: Option<u64>,
    supports_range: bool,
    existing_downloaded: u64,
) -> DownloadPlan {
    if supports_range && let Some(total_size) = total_size {
        return DownloadPlan::Chunked {
            total_size,
            resumed_bytes: existing_downloaded.min(total_size),
        };
    }

    DownloadPlan::Streaming {
        total_size,
        resumed_bytes: 0,
    }
}

fn is_valid_chunk_response(
    status: StatusCode,
    content_range: Option<&str>,
    start_pos: u64,
    end_pos: u64,
) -> bool {
    if status != StatusCode::PARTIAL_CONTENT {
        return false;
    }

    let Some(content_range) = content_range else {
        return false;
    };

    content_range.starts_with(&format!("bytes {start_pos}-{end_pos}/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;
    use std::{collections::HashMap, time::{SystemTime, UNIX_EPOCH}};

    #[test]
    fn retry_delay_uses_exponential_backoff_with_cap() {
        assert_eq!(retry_delay(1), Duration::from_secs(1));
        assert_eq!(retry_delay(2), Duration::from_secs(2));
        assert_eq!(retry_delay(3), Duration::from_secs(4));
        assert_eq!(retry_delay(10), MAX_RETRY_BACKOFF);
    }

    fn temp_file(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("hsi-core-{name}-{nonce}.bin"))
    }

    #[tokio::test]
    async fn chunked_state_reuses_existing_prefix_from_streaming_resume() {
        let dest = temp_file("chunk-resume");
        let state_path = temp_file("chunk-resume-state");
        fs::write(&dest, vec![1u8; 250])
            .await
            .expect("write partial file");

        let config = Config {
            chunk_size: 100,
            ..Default::default()
        };

        let (downloader, _) =
            Hsi::with_config(config, 1, temp_file("queue-state"), BtConfig::default());
        let state = downloader
            .build_chunked_state(
                "https://example.com/file.bin",
                &dest,
                &state_path,
                1000,
                250,
            )
            .await
            .expect("build chunked state");

        assert!(!state.is_streaming);
        assert_eq!(state.downloaded, 250);
        assert_eq!(state.chunks[0].current, state.chunks[0].end + 1);
        assert!(state.chunks[0].is_finished);
        assert_eq!(state.chunks[1].current, state.chunks[1].end + 1);
        assert!(state.chunks[1].is_finished);
        assert_eq!(state.chunks[2].current, 250);
        assert!(!state.chunks[2].is_finished);

        let metadata = fs::metadata(&dest).await.expect("stat resumed file");
        assert_eq!(metadata.len(), 1000);

        let _ = fs::remove_file(&dest).await;
        let _ = fs::remove_file(&state_path).await;
    }

    #[tokio::test]
    async fn load_queue_from_state_requeues_inflight_tasks_as_pending() {
        let queue_path = temp_file("queue-recovery");
        let task_id = "recover-me".to_string();
        let state = QueueState {
            version: "1.0".to_string(),
            tasks: vec![Task {
                id: task_id.clone(),
                url: "https://example.com/file.bin".to_string(),
                dest: temp_file("recover-dest"),
                status: TaskStatus::Downloading,
                total_size: 100,
                downloaded: 40,
                created_at: 1,
                error: None,
                priority: TaskPriority::Normal,
                speed: 1024,
                eta: Some(8),
                headers: HashMap::new(),
                checksum: None,
                speed_limit: None,
                source: DownloadSource::Http {
                    url: "https://example.com/file.bin".to_string(),
                },
                bt_info: None,
                chunk_progress: None,
            }],
            created_at: 1,
            updated_at: 1,
        };
        state.save(&queue_path).await.expect("save queue state");

        let (downloader, _) = Hsi::with_config(Config::default(), 1, queue_path.clone(), BtConfig::default());
        downloader
            .load_queue_from_state()
            .await
            .expect("load queue state");

        let task = downloader
            .get_task(&task_id)
            .await
            .expect("restored task should exist");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.speed, 0);
        assert_eq!(task.eta, None);

        let _ = fs::remove_file(queue_path).await;
        let _ = remove_file_if_exists(&task.dest).await;
    }

    #[test]
    fn chunked_download_rejects_responses_without_partial_content_metadata() {
        assert!(!is_valid_chunk_response(
            StatusCode::OK,
            None,
            0,
            3
        ));
        assert!(!is_valid_chunk_response(
            StatusCode::PARTIAL_CONTENT,
            Some("bytes 0-7/8"),
            0,
            3
        ));
        assert!(is_valid_chunk_response(
            StatusCode::PARTIAL_CONTENT,
            Some("bytes 0-3/8"),
            0,
            3
        ));
    }

    #[test]
    fn streaming_plan_never_preserves_partial_bytes() {
        let streaming = choose_download_plan(Some(8), false, 4);
        assert_eq!(
            streaming,
            DownloadPlan::Streaming {
                total_size: Some(8),
                resumed_bytes: 0,
            }
        );

        let chunked = choose_download_plan(Some(8), true, 4);
        assert_eq!(
            chunked,
            DownloadPlan::Chunked {
                total_size: 8,
                resumed_bytes: 4,
            }
        );
    }

    #[tokio::test]
    async fn completion_callback_finishes_before_completed_event_is_emitted() {
        let queue_path = temp_file("completion-order-queue");
        let (mut downloader, mut event_rx) =
            Hsi::with_config(Config::default(), 1, queue_path.clone(), BtConfig::default());
        let (callback_tx, mut callback_rx) = mpsc::channel(1);
        let dest = temp_file("completion-order-output");

        downloader.set_on_complete(move |task_id, result| {
            let callback_tx = callback_tx.clone();
            async move {
                assert!(result.is_ok());
                tokio::time::sleep(Duration::from_millis(100)).await;
                let _ = callback_tx.send(task_id).await;
            }
        });

        let task_id = "complete-me".to_string();
        downloader.tasks.write().await.insert(
            task_id.clone(),
            Task {
                id: task_id.clone(),
                url: "https://example.com/done.bin".to_string(),
                dest: dest.clone(),
                status: TaskStatus::Downloading,
                total_size: 4,
                downloaded: 4,
                created_at: 1,
                error: None,
                priority: TaskPriority::Normal,
                speed: 0,
                eta: None,
                headers: HashMap::new(),
                checksum: None,
                speed_limit: None,
                source: DownloadSource::Http {
                    url: "https://example.com/done.bin".to_string(),
                },
                bt_info: None,
                chunk_progress: None,
            },
        );

        downloader
            .finalize_task(task_id.clone(), Ok(()), downloader.on_complete.clone())
            .await;

        let completed_id = loop {
            let event = event_rx.recv().await.expect("event should be emitted");
            if let DownloaderEvent::Task(TaskEvent::Completed { task_id }) = event {
                break task_id;
            }
        };

        assert_eq!(completed_id, task_id);
        assert_eq!(
            callback_rx.try_recv().expect("callback should have completed before event"),
            completed_id
        );

        let _ = fs::remove_file(&dest).await;
        let _ = fs::remove_file(&queue_path).await;
    }

    #[tokio::test]
    async fn remove_task_with_file_deletes_bittorrent_output_directory() {
        let output_dir = std::env::temp_dir().join(format!(
            "hsi-bt-output-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(output_dir.join("nested"))
            .await
            .expect("create bt output directory");
        let queue_path = temp_file("bt-remove-queue");
        let (downloader, _) =
            Hsi::with_config(Config::default(), 1, queue_path.clone(), BtConfig::default());
        let task_id = "bt-task".to_string();

        downloader.tasks.write().await.insert(
            task_id.clone(),
            Task {
                id: task_id.clone(),
                url: "magnet:?xt=urn:btih:deadbeef".to_string(),
                dest: output_dir.clone(),
                status: TaskStatus::Completed,
                total_size: 0,
                downloaded: 0,
                created_at: 1,
                error: None,
                priority: TaskPriority::Normal,
                speed: 0,
                eta: None,
                headers: HashMap::new(),
                checksum: None,
                speed_limit: None,
                source: DownloadSource::BitTorrent {
                    uri: "magnet:?xt=urn:btih:deadbeef".to_string(),
                },
                bt_info: None,
                chunk_progress: None,
            },
        );

        downloader
            .remove_task_with_file(&task_id)
            .await
            .expect("remove bt task with directory output");

        assert!(!output_dir.exists(), "BT output directory should be removed");
        assert!(downloader.get_task(&task_id).await.is_none());

        let _ = fs::remove_file(&queue_path).await;
    }
}
