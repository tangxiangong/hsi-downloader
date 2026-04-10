use crate::{Error, Result, types::Config};
use crate::{storage, utils::parse_speed_limit};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// BitTorrent 相关配置
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BtConfig {
    /// 是否启用 DHT（默认 true）
    pub dht_enabled: bool,
    /// 上传限速（字节/秒），None 表示不限制
    pub upload_limit: Option<u64>,
    /// 做种目标比例（如 2.0），达到后停止做种
    pub seed_ratio: Option<f64>,
    /// BT 监听端口，None 表示随机
    pub listen_port: Option<u16>,
}

impl Default for BtConfig {
    fn default() -> Self {
        Self {
            dht_enabled: true,
            upload_limit: None,
            seed_ratio: None,
            listen_port: None,
        }
    }
}

/// Shared application configuration persisted for CLI, TUI, and GUI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// 默认下载路径
    pub default_download_path: PathBuf,
    /// 每个任务的最大并发下载连接数
    pub max_concurrent_downloads: usize,
    /// 队列中同时运行的最大任务数
    pub max_concurrent_tasks: usize,
    /// 分块大小（字节）
    pub chunk_size: u64,
    /// 连接超时（秒）
    pub timeout: u64,
    /// 用户代理
    pub user_agent: String,
    /// 默认代理 URL，支持 http/https/socks5/socks5h 及 URL 内嵌认证信息
    #[serde(default)]
    pub proxy: Option<String>,
    /// 默认任务限速（字节/秒）
    #[serde(default)]
    pub speed_limit: Option<u64>,
    /// 主题设置 (light, dark, system)
    pub theme: AppTheme,
    /// BitTorrent 相关配置
    #[serde(default)]
    pub bt: BtConfig,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AppTheme {
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "system")]
    #[default]
    System,
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AppTheme::Light => "light",
                AppTheme::Dark => "dark",
                AppTheme::System => "system",
            }
        )
    }
}

impl std::str::FromStr for AppTheme {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "light" => Ok(AppTheme::Light),
            "dark" => Ok(AppTheme::Dark),
            "system" => Ok(AppTheme::System),
            _ => Err(Error::ConfigError(format!("invalid theme: {s}"))),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_path = dirs::download_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        Self {
            default_download_path: default_path,
            max_concurrent_downloads: 4,
            max_concurrent_tasks: 2,
            chunk_size: 10 * 1024 * 1024,
            timeout: 30,
            user_agent: "Hsi/1.0".to_string(),
            proxy: None,
            speed_limit: None,
            theme: AppTheme::default(),
            bt: BtConfig::default(),
        }
    }
}

impl AppConfig {
    pub async fn load(path: &Path) -> Result<Self> {
        storage::migrate_legacy_file(path).await?;

        if path.exists() {
            let content = fs_err::tokio::read_to_string(path).await?;
            let config = Self::parse_compat(&content)?;
            config.validate()?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        storage::ensure_parent_dir(path).await?;
        let content = serde_json::to_string_pretty(self)?;
        fs_err::tokio::write(path, content).await?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.max_concurrent_downloads == 0 {
            return Err(Error::ConfigError(
                "max_concurrent_downloads must be greater than 0".into(),
            ));
        }
        if self.max_concurrent_tasks == 0 {
            return Err(Error::ConfigError(
                "max_concurrent_tasks must be greater than 0".into(),
            ));
        }
        if self.chunk_size == 0 {
            return Err(Error::ConfigError(
                "chunk_size must be greater than 0".into(),
            ));
        }
        if self.timeout == 0 {
            return Err(Error::ConfigError("timeout must be greater than 0".into()));
        }
        if let Some(limit) = self.speed_limit
            && limit == 0
        {
            return Err(Error::ConfigError(
                "speed_limit must be greater than 0".into(),
            ));
        }
        if let Some(proxy) = &self.proxy {
            reqwest::Proxy::all(proxy.as_str())
                .map_err(|err| Error::ConfigError(format!("invalid proxy: {err}")))?;
        }

        Ok(())
    }

    pub fn downloader_config(&self) -> Config {
        Config {
            max_concurrent: self.max_concurrent_downloads,
            chunk_size: self.chunk_size,
            speed_limit: self.speed_limit,
            headers: Default::default(),
            proxy: self.proxy.clone(),
            timeout: self.timeout,
            user_agent: Some(self.user_agent.clone()),
        }
    }

    fn parse_compat(content: &str) -> Result<Self> {
        serde_json::from_str::<Self>(content).or_else(|_| {
            serde_json::from_str::<LegacyCliConfig>(content)
                .map(Self::from)
                .map_err(Into::into)
        })
    }
}

#[derive(Debug, Deserialize)]
struct LegacyCliConfig {
    default_connections: usize,
    default_max_tasks: usize,
    default_output_dir: PathBuf,
    user_agent: Option<String>,
    proxy: Option<String>,
    speed_limit: Option<String>,
}

impl From<LegacyCliConfig> for AppConfig {
    fn from(value: LegacyCliConfig) -> Self {
        Self {
            default_download_path: value.default_output_dir,
            max_concurrent_downloads: value.default_connections,
            max_concurrent_tasks: value.default_max_tasks,
            user_agent: value
                .user_agent
                .unwrap_or_else(|| Self::default().user_agent),
            proxy: value.proxy,
            speed_limit: value
                .speed_limit
                .and_then(|value| parse_speed_limit(&value)),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, AppTheme, BtConfig};
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_file(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("hsi-core-{name}-{nonce}.json"))
    }

    #[tokio::test]
    async fn validates_config_values() {
        let mut config = AppConfig {
            max_concurrent_downloads: 0,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());

        config.speed_limit = Some(0);
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn roundtrip_save_and_load() {
        let path = temp_file("config-roundtrip");
        let config = AppConfig {
            theme: AppTheme::Dark,
            proxy: Some("socks5://user:pass@127.0.0.1:1080".into()),
            speed_limit: Some(2 * 1024 * 1024),
            ..AppConfig::default()
        };

        config.save(&path).await.expect("save config");
        let loaded = AppConfig::load(&path).await.expect("load config");

        assert_eq!(loaded, config);
        let _ = fs_err::tokio::remove_file(path).await;
    }

    #[test]
    fn bt_config_default() {
        let config = BtConfig::default();
        assert!(config.dht_enabled);
        assert!(config.upload_limit.is_none());
        assert!(config.seed_ratio.is_none());
        assert!(config.listen_port.is_none());
    }

    #[test]
    fn bt_config_serde_roundtrip() {
        let config = BtConfig {
            dht_enabled: false,
            upload_limit: Some(1024 * 1024),
            seed_ratio: Some(2.0),
            listen_port: Some(6881),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: BtConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}
