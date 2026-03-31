use crate::storage;
use crate::{Error, Result, types::Config};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Shared application configuration persisted for CLI, TUI, and GUI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    /// 主题设置 (light, dark, system)
    pub theme: String,
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
            user_agent: "YuShi/1.0".to_string(),
            theme: "system".to_string(),
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
        if !matches!(self.theme.as_str(), "light" | "dark" | "system") {
            return Err(Error::ConfigError(
                "theme must be one of: light, dark, system".into(),
            ));
        }
        Ok(())
    }

    pub fn downloader_config(&self) -> Config {
        Config {
            max_concurrent: self.max_concurrent_downloads,
            chunk_size: self.chunk_size,
            speed_limit: None,
            headers: Default::default(),
            proxy: None,
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
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppConfig;
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_file(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("yushi-core-{name}-{nonce}.json"))
    }

    #[tokio::test]
    async fn validates_config_values() {
        let mut config = AppConfig {
            max_concurrent_downloads: 0,
            ..AppConfig::default()
        };
        assert!(config.validate().is_err());

        config.max_concurrent_downloads = 1;
        config.theme = "neon".into();
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn roundtrip_save_and_load() {
        let path = temp_file("config-roundtrip");
        let config = AppConfig {
            theme: "dark".into(),
            ..AppConfig::default()
        };

        config.save(&path).await.expect("save config");
        let loaded = AppConfig::load(&path).await.expect("load config");

        assert_eq!(loaded, config);
        let _ = fs_err::tokio::remove_file(path).await;
    }

    #[tokio::test]
    async fn loads_legacy_cli_config() {
        let path = temp_file("config-legacy-cli");
        let legacy = r#"{
            "default_connections": 8,
            "default_max_tasks": 5,
            "default_output_dir": "/tmp/legacy",
            "user_agent": "Legacy/1.0",
            "proxy": "http://localhost:8080",
            "speed_limit": "1M"
        }"#;

        fs_err::tokio::write(&path, legacy)
            .await
            .expect("write legacy config");
        let loaded = AppConfig::load(&path).await.expect("load legacy config");

        assert_eq!(loaded.max_concurrent_downloads, 8);
        assert_eq!(loaded.max_concurrent_tasks, 5);
        assert_eq!(loaded.default_download_path, PathBuf::from("/tmp/legacy"));
        assert_eq!(loaded.user_agent, "Legacy/1.0");
        assert_eq!(loaded.theme, "system");

        let _ = fs_err::tokio::remove_file(path).await;
    }

    #[tokio::test]
    async fn loads_tauri_config_with_window_state() {
        let path = temp_file("config-legacy-tauri");
        let legacy = r#"{
            "default_download_path": "/tmp/downloads",
            "max_concurrent_downloads": 3,
            "max_concurrent_tasks": 2,
            "chunk_size": 1048576,
            "timeout": 15,
            "user_agent": "Tauri/1.0",
            "theme": "light",
            "window": {
              "width": 1200,
              "height": 800,
              "x": -1,
              "y": -1,
              "maximized": false,
              "sidebar_open": true
            }
        }"#;

        fs_err::tokio::write(&path, legacy)
            .await
            .expect("write tauri config");
        let loaded = AppConfig::load(&path).await.expect("load tauri config");

        assert_eq!(
            loaded.default_download_path,
            PathBuf::from("/tmp/downloads")
        );
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.user_agent, "Tauri/1.0");

        let _ = fs_err::tokio::remove_file(path).await;
    }
}
