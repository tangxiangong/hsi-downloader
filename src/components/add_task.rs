use anyhow::{Result, anyhow};
use std::path::PathBuf;
use yushi_core::{AppConfig, YuShi, parse_speed_limit};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddTaskDraft {
    pub url: String,
    pub destination_input: String,
    pub speed_limit: Option<u64>,
}

impl AddTaskDraft {
    pub fn parse(url: &str, destination_input: &str, speed_limit_input: &str) -> Result<Self> {
        let url = url.trim();
        if url.is_empty() {
            return Err(anyhow!("URL is required"));
        }

        let speed_limit = parse_optional_speed_limit(speed_limit_input)?;

        Ok(Self {
            url: url.to_string(),
            destination_input: destination_input.trim().to_string(),
            speed_limit,
        })
    }

    pub async fn resolve_destination(&self, queue: &YuShi, config: &AppConfig) -> PathBuf {
        if self.destination_input.is_empty() {
            queue
                .infer_destination_in_dir(&self.url, config.default_download_path.clone())
                .await
        } else {
            PathBuf::from(&self.destination_input)
        }
    }
}

pub fn parse_optional_speed_limit(speed_limit_input: &str) -> Result<Option<u64>> {
    if speed_limit_input.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(
            parse_speed_limit(speed_limit_input.trim())
                .ok_or_else(|| anyhow!("Invalid speed limit: {}", speed_limit_input.trim()))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{AddTaskDraft, parse_optional_speed_limit};

    #[test]
    fn rejects_empty_url() {
        assert!(AddTaskDraft::parse("", "", "").is_err());
    }

    #[test]
    fn parses_speed_limit() {
        let draft =
            AddTaskDraft::parse("https://example.com/file.bin", "", "2M").expect("parse draft");

        assert_eq!(draft.url, "https://example.com/file.bin");
        assert_eq!(draft.speed_limit, Some(2 * 1024 * 1024));
    }

    #[test]
    fn preserves_explicit_destination() {
        let draft = AddTaskDraft::parse("https://example.com/file.bin", "/tmp/output.bin", "")
            .expect("parse draft");

        assert_eq!(draft.destination_input, "/tmp/output.bin");
        assert_eq!(draft.speed_limit, None);
    }

    #[test]
    fn parses_optional_speed_limit() {
        assert_eq!(parse_optional_speed_limit("").expect("empty"), None);
        assert_eq!(
            parse_optional_speed_limit("512K").expect("speed"),
            Some(512 * 1024)
        );
    }
}
