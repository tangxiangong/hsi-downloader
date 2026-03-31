use crate::{Error, Result};
use fs_err::tokio as fs;
use std::path::{Path, PathBuf};

const APP_DIR: &str = "yushi";
const LEGACY_TAURI_IDENTIFIER: &str = "com.tangxiangong.YuShi";

pub fn storage_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| Error::PathError("unable to resolve config directory".into()))?;
    Ok(config_dir.join(APP_DIR))
}

pub fn config_path() -> Result<PathBuf> {
    Ok(storage_dir()?.join("config.json"))
}

pub fn history_path() -> Result<PathBuf> {
    Ok(storage_dir()?.join("history.json"))
}

pub fn queue_state_path() -> Result<PathBuf> {
    Ok(storage_dir()?.join("queue.json"))
}

pub async fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

pub async fn migrate_legacy_file(target: &Path) -> Result<()> {
    if target.exists() {
        return Ok(());
    }

    let Some(file_name) = target.file_name().and_then(|name| name.to_str()) else {
        return Ok(());
    };

    for candidate in legacy_candidates(file_name) {
        if candidate.exists() {
            ensure_parent_dir(target).await?;
            fs::copy(&candidate, target).await?;
            break;
        }
    }

    Ok(())
}

fn legacy_candidates(file_name: &str) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(dir) = dirs::data_local_dir() {
        candidates.push(dir.join(LEGACY_TAURI_IDENTIFIER).join(file_name));
    }

    if let Some(dir) = dirs::data_dir() {
        candidates.push(dir.join(LEGACY_TAURI_IDENTIFIER).join(file_name));
    }

    candidates
}
