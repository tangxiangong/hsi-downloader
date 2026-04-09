use crate::{
    cli::DownloadArgs,
    config::ConfigStore,
    ui::{format_size, print_error, print_info, print_success},
};
use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc;
use yushi_core::{ChecksumType, ProgressEvent, YuShi, parse_speed_limit};

pub async fn execute(args: DownloadArgs) -> Result<()> {
    let app_config = ConfigStore::load().await?;
    let mut config = app_config.downloader_config();
    if let Some(connections) = args.connections {
        config.max_concurrent = connections;
    }

    if let Some(limit_str) = &args.speed_limit {
        let limit =
            parse_speed_limit(limit_str).ok_or_else(|| anyhow!("无效的速度限制: {}", limit_str))?;
        config.speed_limit = Some(limit);
        print_info(&format!("速度限制: {}/s", format_size(limit)));
    }

    if let Some(ua) = &args.user_agent {
        config.user_agent = Some(ua.clone());
    }

    if let Some(proxy) = &args.proxy {
        config.proxy = Some(proxy.clone());
        print_info(&format!("使用代理: {}", proxy));
    }

    for header in &args.header {
        if let Some((key, value)) = header.split_once(':') {
            config
                .headers
                .insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    let temp_dir = std::env::temp_dir();
    let queue_state_path = temp_dir.join(format!("yushi_temp_{}.json", std::process::id()));

    let (downloader, _) = YuShi::with_config(config, 1, queue_state_path.clone());
    let output = if let Some(path) = args.output {
        if path.is_absolute() {
            path
        } else {
            app_config.default_download_path.join(path)
        }
    } else {
        downloader
            .infer_destination_in_dir(&args.url, app_config.default_download_path.clone())
            .await
    };
    let (tx, mut rx) = mpsc::channel(1024);

    print_info(&format!("下载: {}", args.url));
    print_info(&format!("保存到: {}", output.display()));

    let quiet = args.quiet;
    let progress_handle = tokio::spawn(async move {
        let mut pb: Option<ProgressBar> = None;
        let mut downloaded = 0u64;

        while let Some(event) = rx.recv().await {
            match event {
                ProgressEvent::Initialized { total_size, .. } => {
                    if !quiet {
                        if let Some(size) = total_size {
                            let bar = ProgressBar::new(size);
                            bar.set_style(
                                ProgressStyle::default_bar()
                                    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                                    .expect("valid progress bar template")
                                    .progress_chars("#>-"),
                            );
                            pb = Some(bar);
                        } else {
                            let bar = ProgressBar::new_spinner();
                            bar.set_style(
                                ProgressStyle::default_spinner()
                                    .template("{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec}) - 流式下载")
                                    .expect("valid spinner template"),
                            );
                            pb = Some(bar);
                        }
                    }
                }
                ProgressEvent::Updated {
                    downloaded: current,
                    ..
                }
                | ProgressEvent::StreamProgress {
                    downloaded: current,
                    ..
                }
                | ProgressEvent::StreamDownloading {
                    downloaded: current,
                } => {
                    downloaded = current;
                    if let Some(ref bar) = pb {
                        bar.set_position(downloaded);
                    }
                }
                ProgressEvent::ChunkProgress { delta, .. }
                | ProgressEvent::ChunkDownloading { delta, .. } => {
                    downloaded += delta;
                    if let Some(ref bar) = pb {
                        bar.set_position(downloaded);
                    }
                }
                ProgressEvent::Finished { .. } => {
                    if let Some(bar) = pb.take() {
                        bar.finish_with_message("下载完成");
                    }
                }
                ProgressEvent::Failed { error, .. } => {
                    if let Some(bar) = pb.take() {
                        bar.finish_with_message(format!("下载失败: {}", error));
                    }
                }
                ProgressEvent::BtStatus { .. } => {
                    // BT status updates are informational, no progress bar action needed
                }
            }
        }
    });

    let output_str = output
        .to_str()
        .ok_or_else(|| anyhow!("输出路径包含无效 UTF-8"))?;
    let result = downloader.download(&args.url, output_str, Some(tx)).await;

    let _ = std::fs::remove_file(queue_state_path);
    progress_handle.await?;

    match result {
        Ok(_) => {
            if let Some(md5) = args.md5 {
                print_info("验证 MD5...");
                let checksum = ChecksumType::Md5(md5);
                match yushi_core::verify_file(&output, &checksum).await {
                    Ok(true) => print_success("MD5 校验通过"),
                    Ok(false) => {
                        print_error("MD5 校验失败");
                        return Err(anyhow!("MD5 校验失败"));
                    }
                    Err(err) => {
                        print_error(&format!("MD5 校验错误: {}", err));
                        return Err(err.into());
                    }
                }
            }

            if let Some(sha256) = args.sha256 {
                print_info("验证 SHA256...");
                let checksum = ChecksumType::Sha256(sha256);
                match yushi_core::verify_file(&output, &checksum).await {
                    Ok(true) => print_success("SHA256 校验通过"),
                    Ok(false) => {
                        print_error("SHA256 校验失败");
                        return Err(anyhow!("SHA256 校验失败"));
                    }
                    Err(err) => {
                        print_error(&format!("SHA256 校验错误: {}", err));
                        return Err(err.into());
                    }
                }
            }

            print_success(&format!("文件已保存到: {}", output.display()));
            Ok(())
        }
        Err(err) => {
            print_error(&format!("下载失败: {}", err));
            Err(err.into())
        }
    }
}
