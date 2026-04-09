use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "yushi")]
#[command(about = "YuShi - 高性能多线程下载器", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 下载单个文件
    Download(DownloadArgs),
    /// 管理下载队列
    Queue(QueueArgs),
    /// 配置管理
    Config(ConfigArgs),
    /// 启动 TUI 界面
    #[cfg(feature = "tui")]
    Tui,
}

#[derive(Parser)]
pub struct DownloadArgs {
    /// 下载 URL
    #[arg(value_name = "URL")]
    pub url: String,

    /// 输出文件路径
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// 并发连接数
    #[arg(short = 'n', long)]
    pub connections: Option<usize>,

    /// 速度限制 (例如: 1M, 500K)
    #[arg(short = 'l', long)]
    pub speed_limit: Option<String>,

    /// 自定义 User-Agent
    #[arg(short = 'A', long)]
    pub user_agent: Option<String>,

    /// 代理 URL，支持 http/https/socks5/socks5h 和 URL 内嵌认证
    #[arg(short = 'x', long)]
    pub proxy: Option<String>,

    /// 自定义 HTTP 头 (格式: "Key: Value")
    #[arg(short = 'H', long)]
    pub header: Vec<String>,

    /// MD5 校验和
    #[arg(long)]
    pub md5: Option<String>,

    /// SHA256 校验和
    #[arg(long)]
    pub sha256: Option<String>,

    /// 静默模式（不显示进度）
    #[arg(short = 'q', long)]
    pub quiet: bool,
}

#[derive(Parser)]
pub struct QueueArgs {
    #[command(subcommand)]
    pub command: QueueCommands,
}

#[derive(Subcommand)]
pub enum QueueCommands {
    /// 添加下载任务到队列
    Add {
        /// 下载 URL
        url: String,
        /// 输出文件路径
        #[arg(short, long)]
        output: PathBuf,
        /// 优先级 (low, normal, high)
        #[arg(short, long, default_value = "normal")]
        priority: String,
        /// MD5 校验和
        #[arg(long)]
        md5: Option<String>,
        /// SHA256 校验和
        #[arg(long)]
        sha256: Option<String>,
        /// 任务限速 (例如: 1M, 500K)
        #[arg(short = 'l', long)]
        speed_limit: Option<String>,
        /// 选择下载的文件索引（逗号分隔，用于种子下载）
        #[arg(long)]
        select_files: Option<String>,
    },
    /// 列出所有任务
    List,
    /// 启动队列处理
    Start {
        /// 最大并发任务数
        #[arg(short = 'n', long)]
        max_tasks: Option<usize>,
        /// 每个任务的并发连接数
        #[arg(short = 'c', long)]
        connections: Option<usize>,
    },
    /// 暂停任务
    Pause {
        /// 任务 ID
        task_id: String,
    },
    /// 恢复任务
    Resume {
        /// 任务 ID
        task_id: String,
    },
    /// 取消任务
    Cancel {
        /// 任务 ID
        task_id: String,
    },
    /// 移除任务
    Remove {
        /// 任务 ID
        task_id: String,
    },
    /// 清空已完成任务
    Clear,
}

#[derive(Parser)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// 显示当前配置
    Show,
    /// 设置配置项
    Set {
        /// 配置键
        key: String,
        /// 配置值
        value: String,
    },
    /// 重置配置
    Reset,
}
