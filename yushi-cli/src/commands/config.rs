use crate::{
    cli::{ConfigArgs, ConfigCommands},
    config::ConfigStore,
    ui::{print_error, print_info, print_success},
};
use anyhow::Result;
use console::style;
use yushi_core::{AppConfig, parse_speed_limit};

pub async fn execute(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommands::Show => show_config().await,
        ConfigCommands::Set { key, value } => set_config(key, value).await,
        ConfigCommands::Reset => reset_config().await,
    }
}

async fn show_config() -> Result<()> {
    let config = ConfigStore::load().await?;

    println!("\n{}", style("当前配置").bold().underlined());
    println!();
    println!("  默认并发连接数: {}", config.max_concurrent_downloads);
    println!("  默认最大任务数: {}", config.max_concurrent_tasks);
    println!("  默认输出目录: {}", config.default_download_path.display());
    println!("  User-Agent: {}", config.user_agent);
    println!("  代理: {}", config.proxy.as_deref().unwrap_or("未设置"));
    println!("  超时: {} 秒", config.timeout);
    println!("  分块大小: {} 字节", config.chunk_size);
    println!(
        "  默认任务限速: {}",
        config
            .speed_limit
            .map(|limit| format!("{} B/s", limit))
            .unwrap_or_else(|| "不限速".to_string())
    );
    println!("  主题: {}", config.theme);

    println!();
    println!("配置文件: {}", ConfigStore::config_path()?.display());
    println!("历史文件: {}", ConfigStore::history_path()?.display());
    println!("队列文件: {}", ConfigStore::queue_state_path()?.display());

    Ok(())
}

async fn set_config(key: String, value: String) -> Result<()> {
    let mut config = ConfigStore::load().await?;

    match key.as_str() {
        "connections" => {
            config.max_concurrent_downloads = value.parse()?;
            print_success(&format!("默认并发连接数已设置为: {}", value));
        }
        "max_tasks" => {
            config.max_concurrent_tasks = value.parse()?;
            print_success(&format!("默认最大任务数已设置为: {}", value));
        }
        "output_dir" => {
            config.default_download_path = value.into();
            print_success(&format!(
                "默认输出目录已设置为: {}",
                config.default_download_path.display()
            ));
        }
        "user_agent" => {
            config.user_agent = value.clone();
            print_success(&format!("User-Agent 已设置为: {}", value));
        }
        "proxy" => {
            config.proxy = if value.trim().is_empty() {
                None
            } else {
                Some(value.clone())
            };
            print_success(&format!(
                "代理已设置为: {}",
                config.proxy.as_deref().unwrap_or("未设置")
            ));
        }
        "timeout" => {
            config.timeout = value.parse()?;
            print_success(&format!("超时已设置为: {} 秒", value));
        }
        "chunk_size" => {
            config.chunk_size = value.parse()?;
            print_success(&format!("分块大小已设置为: {} 字节", value));
        }
        "speed_limit" => {
            config.speed_limit = if value.trim().is_empty() {
                None
            } else {
                Some(
                    parse_speed_limit(&value)
                        .ok_or_else(|| anyhow::anyhow!("无效的速度限制: {}", value))?,
                )
            };
            print_success(&format!(
                "默认任务限速已设置为: {}",
                config
                    .speed_limit
                    .map(|limit| format!("{} B/s", limit))
                    .unwrap_or_else(|| "不限速".to_string())
            ));
        }
        "theme" => {
            config.theme = value.parse()?;
            print_success(&format!("主题已设置为: {}", value));
        }
        _ => {
            print_error(&format!("未知的配置项: {}", key));
            print_info(
                "可用的配置项: connections, max_tasks, output_dir, user_agent, proxy, timeout, chunk_size, speed_limit, theme",
            );
            return Ok(());
        }
    }

    config.validate()?;
    ConfigStore::save(&config).await?;
    Ok(())
}

async fn reset_config() -> Result<()> {
    let config = AppConfig::default();
    ConfigStore::save(&config).await?;
    print_success("配置已重置为默认值");
    Ok(())
}
