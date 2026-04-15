use crate::{
    cli::{QueueArgs, QueueCommands},
    config::ConfigStore,
    ui::{ProgressManager, format_size, print_info, print_success, short_task_id},
};
use anyhow::{Result, anyhow};
use console::style;
use hsi_core::{
    ChecksumType, DownloadSource, DownloaderEvent, Priority, ProgressEvent, TaskEvent, TaskStatus,
    VerificationEvent, parse_speed_limit, types::AddTaskOptions,
};
use std::path::PathBuf;

pub async fn execute(args: QueueArgs) -> Result<()> {
    match args.command {
        QueueCommands::Add {
            url,
            output,
            priority,
            md5,
            sha256,
            speed_limit,
            select_files,
        } => {
            add_task(
                url,
                output,
                priority,
                md5,
                sha256,
                speed_limit,
                select_files,
            )
            .await
        }
        QueueCommands::List => list_tasks().await,
        QueueCommands::Start {
            max_tasks,
            connections,
        } => start_queue(max_tasks, connections).await,
        QueueCommands::Pause { task_id } => pause_task(task_id).await,
        QueueCommands::Resume { task_id } => resume_task(task_id).await,
        QueueCommands::Cancel { task_id } => cancel_task(task_id).await,
        QueueCommands::Remove { task_id } => remove_task(task_id).await,
        QueueCommands::Clear => clear_completed().await,
    }
}

async fn add_task(
    url: String,
    output: PathBuf,
    priority_str: String,
    md5: Option<String>,
    sha256: Option<String>,
    speed_limit: Option<String>,
    select_files: Option<String>,
) -> Result<()> {
    let config = ConfigStore::load().await?;
    let output = if output.is_absolute() {
        output
    } else {
        config.default_download_path.join(output)
    };
    let (queue, _) = ConfigStore::build_queue(&config, None, Some(1)).await?;

    queue.load_queue_from_state().await?;

    let priority = match priority_str.to_lowercase().as_str() {
        "low" => Priority::Low,
        "normal" => Priority::Normal,
        "high" => Priority::High,
        _ => return Err(anyhow!("无效的优先级: {}", priority_str)),
    };

    let checksum = if let Some(hash) = md5 {
        Some(ChecksumType::Md5(hash))
    } else {
        sha256.map(ChecksumType::Sha256)
    };

    let speed_limit = match speed_limit {
        Some(limit) => {
            Some(parse_speed_limit(&limit).ok_or_else(|| anyhow!("无效的速度限制: {}", limit))?)
        }
        None => None,
    };

    let selected_files = select_files.map(|s| {
        s.split(',')
            .filter_map(|idx| idx.trim().parse::<usize>().ok())
            .collect::<Vec<_>>()
    });

    let task_id = queue
        .add_task_with_options(AddTaskOptions {
            url: url.clone(),
            dest: output.clone(),
            priority: Some(priority),
            checksum,
            speed_limit,
            auto_rename_on_conflict: true,
            selected_files,
            headers: None,
            config: Default::default(),
        })
        .await?;

    print_success("任务已添加到队列");
    println!("  任务 ID: {}", style(&task_id).cyan());
    println!("  URL: {}", url);
    println!("  输出: {}", output.display());
    println!("  优先级: {:?}", priority);
    if let Some(limit) = speed_limit {
        println!("  限速: {}/s", format_size(limit));
    }

    Ok(())
}

async fn list_tasks() -> Result<()> {
    let config = ConfigStore::load().await?;
    let (queue, _) = ConfigStore::build_queue(&config, None, Some(1)).await?;

    queue.load_queue_from_state().await?;
    let tasks = queue.get_all_tasks().await;

    if tasks.is_empty() {
        print_info("队列为空");
        return Ok(());
    }

    println!("\n{}", style("下载队列").bold().underlined());
    println!();

    for task in tasks {
        let status_str = match task.status {
            TaskStatus::Pending => style("等待中").yellow(),
            TaskStatus::Downloading => style("下载中").green(),
            TaskStatus::Paused => style("已暂停").blue(),
            TaskStatus::Completed => style("已完成").green(),
            TaskStatus::Failed => style("失败").red(),
            TaskStatus::Cancelled => style("已取消").red(),
        };

        let source_label = match &task.source {
            DownloadSource::BitTorrent { .. } => "[BT]",
            DownloadSource::Http { .. } => "[HTTP]",
        };

        println!(
            "{} {} {}",
            style("●").bold(),
            status_str,
            style(source_label).dim()
        );
        println!("  ID: {}", style(&task.id[..16]).cyan());
        println!("  URL: {}", task.url);
        println!("  输出: {}", task.dest.display());
        println!("  优先级: {:?}", task.priority);
        if let Some(limit) = task.speed_limit {
            println!("  限速: {}/s", format_size(limit));
        }

        if task.total_size > 0 {
            let progress = (task.downloaded as f64 / task.total_size as f64) * 100.0;
            println!(
                "  进度: {:.1}% ({} / {})",
                progress,
                format_size(task.downloaded),
                format_size(task.total_size)
            );
        } else {
            println!("  进度: {} (流式下载)", format_size(task.downloaded));
        }

        if task.speed > 0 {
            println!("  速度: {}/s", format_size(task.speed));
        }

        if let Some(eta) = task.eta {
            println!("  剩余时间: {}s", eta);
        }

        if let Some(error) = &task.error {
            println!("  {}: {}", style("错误").red(), error);
        }

        if let Some(bt) = &task.bt_info {
            println!(
                "  BT: {}P · ↑{}/s · 已上传 {}",
                bt.peers,
                format_size(bt.upload_speed),
                format_size(bt.uploaded)
            );
        }

        println!();
    }

    Ok(())
}

async fn start_queue(max_tasks: Option<usize>, connections: Option<usize>) -> Result<()> {
    let config = ConfigStore::load().await?;
    let (queue, mut event_rx) = ConfigStore::build_queue(&config, connections, max_tasks).await?;

    queue.load_queue_from_state().await?;

    let tasks = queue.get_all_tasks().await;
    let pending_count = tasks
        .iter()
        .filter(|task| task.status == TaskStatus::Pending)
        .count();

    if pending_count == 0 {
        print_info("没有待处理的任务");
        return Ok(());
    }

    print_info(&format!("启动队列处理 ({} 个待处理任务)", pending_count));
    print_info(&format!(
        "最大并发任务: {}",
        max_tasks.unwrap_or(config.max_concurrent_tasks)
    ));
    print_info(&format!(
        "每任务连接数: {}",
        connections.unwrap_or(config.max_concurrent_downloads)
    ));
    println!();

    queue.start_pending_tasks().await?;

    let progress_mgr = ProgressManager::new();
    let event_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                DownloaderEvent::Task(TaskEvent::Started { task_id }) => {
                    println!("🚀 开始: {}", &short_task_id(&task_id));
                }
                DownloaderEvent::Progress(ProgressEvent::Updated {
                    task_id,
                    downloaded,
                    total,
                    speed,
                    ..
                }) => {
                    progress_mgr
                        .update_progress(&task_id, downloaded, speed)
                        .await;
                    if total > 0 && downloaded == 0 {
                        progress_mgr.add_task(task_id, Some(total)).await;
                    } else if total == 0 && downloaded == 0 {
                        progress_mgr.add_task(task_id, None).await;
                    }
                }
                DownloaderEvent::Task(TaskEvent::Completed { task_id }) => {
                    progress_mgr.finish_task(&task_id, true).await;
                }
                DownloaderEvent::Task(TaskEvent::Failed { task_id, error }) => {
                    progress_mgr.finish_task(&task_id, false).await;
                    eprintln!("❌ 失败 {}: {}", &short_task_id(&task_id), error);
                }
                DownloaderEvent::Verification(VerificationEvent::Started { task_id }) => {
                    println!("🔍 校验: {}", &short_task_id(&task_id));
                }
                DownloaderEvent::Verification(VerificationEvent::Completed {
                    task_id,
                    success,
                }) => {
                    if success {
                        println!("✅ 校验通过: {}", &short_task_id(&task_id));
                    } else {
                        println!("❌ 校验失败: {}", &short_task_id(&task_id));
                    }
                }
                _ => {}
            }
        }
    });

    tokio::signal::ctrl_c().await?;
    println!("\n\n收到中断信号，正在停止...");

    event_handle.abort();
    print_success("队列已停止");

    Ok(())
}

async fn pause_task(task_id: String) -> Result<()> {
    let config = ConfigStore::load().await?;
    let (queue, _) = ConfigStore::build_queue(&config, None, Some(1)).await?;

    queue.load_queue_from_state().await?;
    queue.pause_task(&task_id).await?;

    print_success(&format!("任务已暂停: {}", &task_id[..16]));
    Ok(())
}

async fn resume_task(task_id: String) -> Result<()> {
    let config = ConfigStore::load().await?;
    let (queue, _) = ConfigStore::build_queue(&config, None, Some(1)).await?;

    queue.load_queue_from_state().await?;
    queue.resume_task(&task_id).await?;

    print_success(&format!("任务已恢复: {}", &task_id[..16]));
    Ok(())
}

async fn cancel_task(task_id: String) -> Result<()> {
    let config = ConfigStore::load().await?;
    let (queue, _) = ConfigStore::build_queue(&config, None, Some(1)).await?;

    queue.load_queue_from_state().await?;
    queue.cancel_task(&task_id).await?;

    print_success(&format!("任务已取消: {}", &task_id[..16]));
    Ok(())
}

async fn remove_task(task_id: String) -> Result<()> {
    let config = ConfigStore::load().await?;
    let (queue, _) = ConfigStore::build_queue(&config, None, Some(1)).await?;

    queue.load_queue_from_state().await?;
    queue.remove_task(&task_id).await?;

    print_success(&format!("任务已移除: {}", &task_id[..16]));
    Ok(())
}

async fn clear_completed() -> Result<()> {
    let config = ConfigStore::load().await?;
    let (queue, _) = ConfigStore::build_queue(&config, None, Some(1)).await?;

    queue.load_queue_from_state().await?;
    queue.clear_completed().await?;

    print_success("已清空所有已完成任务");
    Ok(())
}
