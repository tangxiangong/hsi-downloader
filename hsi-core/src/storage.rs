use crate::{Error, Result};
use fs_err::tokio as fs;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::sleep;

const APP_DIR: &str = "hsi";
const LEGACY_TAURI_IDENTIFIER: &str = "com.tangxiangong.hsi.app";
const RESUME_DIR: &str = "resume";
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(50);
const LOCK_STALE_TIMEOUT: Duration = Duration::from_secs(30);

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

pub fn resume_state_dir() -> Result<PathBuf> {
    Ok(storage_dir()?.join(RESUME_DIR))
}

pub fn download_state_path(dest: &Path) -> Result<PathBuf> {
    let mut hasher = Sha256::new();
    hasher.update(dest.as_os_str().to_string_lossy().as_bytes());
    let digest = hasher.finalize();
    Ok(resume_state_dir()?.join(format!("{}.json", hex::encode(digest))))
}

pub async fn migrate_download_state_file(dest: &Path) -> Result<PathBuf> {
    let target = download_state_path(dest)?;
    let legacy = dest.with_extension("json");

    if target.exists() || !legacy.exists() {
        return Ok(target);
    }

    ensure_parent_dir(&target).await?;
    fs::copy(&legacy, &target).await?;
    let _ = fs::remove_file(&legacy).await;

    Ok(target)
}

pub async fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    Ok(())
}

pub async fn atomic_write_string(path: &Path, content: &str) -> Result<()> {
    ensure_parent_dir(path).await?;

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::PathError("unable to derive file name".into()))?;
    let temp_path = path.with_file_name(format!(".{file_name}.{nonce}.tmp"));

    fs::write(&temp_path, content).await?;
    fs::rename(&temp_path, path).await?;
    Ok(())
}

#[derive(Debug)]
pub struct FileLockGuard {
    lock_path: PathBuf,
}

impl Drop for FileLockGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

pub async fn acquire_file_lock(path: &Path) -> Result<FileLockGuard> {
    ensure_parent_dir(path).await?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::PathError("unable to derive lock file name".into()))?;
    let lock_path = path.with_file_name(format!("{file_name}.lock"));

    loop {
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                use std::io::Write;

                let _ = writeln!(file, "{}", std::process::id());
                return Ok(FileLockGuard { lock_path });
            }
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                if let Ok(metadata) = std::fs::metadata(&lock_path)
                    && let Ok(modified_at) = metadata.modified()
                    && modified_at.elapsed().unwrap_or_default() > LOCK_STALE_TIMEOUT
                {
                    let _ = std::fs::remove_file(&lock_path);
                    continue;
                }

                sleep(LOCK_RETRY_DELAY).await;
            }
            Err(err) => return Err(err.into()),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_state_path_is_stored_under_app_resume_dir() {
        let dest = PathBuf::from("/tmp/example/file.iso");
        let state_path = download_state_path(&dest).expect("derive state path");

        assert!(state_path.starts_with(resume_state_dir().expect("resume state dir")));
        assert_eq!(
            state_path.extension().and_then(|ext| ext.to_str()),
            Some("json")
        );
        assert!(!state_path.starts_with("/tmp/example"));
    }
}
