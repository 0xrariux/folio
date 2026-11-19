//! Private recovery snapshots and verified, rehearsed portable backups.
use super::Store;
use crate::{
    security::{self, Key, VaultConfig},
    workspace::{self, Notebook},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackupProof {
    pub schema: u32,
    pub sha256: String,
    pub size: u64,
    pub storage_contract: String,
    pub restored_and_reopened: bool,
    pub verified_by_version: String,
    pub verified_by_revision: String,
    pub schema_version: i64,
}
impl BackupProof {
    pub fn path(backup: &Path) -> PathBuf {
        backup.with_extension("verification.json")
    }
    pub fn verify(backup: &Path) -> Result<Self> {
        let proof: Self =
            serde_json::from_slice(&workspace::read_limited(&Self::path(backup), 4096)?)
                .context("The backup has no valid restoration proof")?;
        ensure!(
            proof.schema == 1
                && proof.restored_and_reopened
                && proof.size > 0
                && proof.size <= workspace::MAX_BACKUP
                && proof.sha256.len() == 64
                && proof.schema_version == super::SCHEMA_VERSION
                && proof.verified_by_version == crate::updates::VERSION
                && proof.verified_by_revision == crate::updates::REVISION
                && proof.storage_contract == crate::updates::config().storage_contract,
            "Backup verification is incompatible or incomplete"
        );
        ensure!(
            backup.metadata()?.len() == proof.size
                && crate::updates::hash_file(backup)? == proof.sha256,
            "Pre-update backup changed or is damaged; update cancelled"
        );
        Ok(proof)
    }
}
/// Called while the profile worker owns its lock, before the first migration write.
/// Database and committed WAL bytes are kept together; never checkpoint to make this copy.
pub(super) fn snapshot(path: &Path, reason: &str) -> Result<PathBuf> {
    let folder = path.with_extension("recovery");
    fs::create_dir_all(&folder)?;
    let staging = tempfile::Builder::new()
        .prefix("before-migration-")
        .tempdir_in(&folder)?;
    let mut files = Vec::new();
    let mut inputs = vec![
        (path.to_owned(), "database"),
        (
            PathBuf::from(format!("{}-wal", path.display())),
            "database-wal",
        ),
    ];
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let config = path.with_file_name(format!(
        "{}.security.json",
        stem.strip_suffix(".encrypted").unwrap_or(stem)
    ));
    inputs.push((config, "security.json"));
    for (source, name) in inputs {
        if !source.exists() {
            continue;
        }
        ensure!(source.is_file(), "Invalid recovery source");
        let target = staging.path().join(name);
        fs::copy(&source, &target)?;
        File::open(&target)?.sync_all()?;
        files.push(serde_json::json!({"name": name, "size": target.metadata()?.len(), "sha256": crate::updates::hash_file(&target)?}));
    }
    ensure!(
        files.iter().any(|f| f["name"] == "database"),
        "Cannot preserve the pre-migration database"
    );
    let digest = ring::digest::digest(&ring::digest::SHA256, &serde_json::to_vec(&files)?);
    let identity: String = digest.as_ref().iter().map(|b| format!("{b:02x}")).collect();
    let final_path = folder.join(format!("before-open-{identity}"));
    if final_path.exists() {
        // A previously retained identical snapshot is reused only after checking every byte.
        for entry in &files {
            let file = final_path.join(entry["name"].as_str().context("Invalid snapshot entry")?);
            ensure!(
                crate::updates::hash_file(&file)? == entry["sha256"].as_str().unwrap_or_default(),
                "Existing recovery snapshot is damaged; no profile was opened"
            );
        }
        return Ok(final_path);
    }
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as u64;
    let manifest = staging.path().join("manifest.json");
    fs::write(
        &manifest,
        serde_json::to_vec_pretty(
            &serde_json::json!({"schema": 1, "reason": reason, "verified": false, "pinned": false, "created_at": created, "source_name": path.file_name(), "files": files}),
        )?,
    )?;
    File::open(manifest)?.sync_all()?;
    File::open(staging.path())?.sync_all()?;
    let kept = staging.keep();
    fs::rename(&kept, &final_path)?;
    File::open(folder)?.sync_all()?;
    Ok(final_path)
}
/// Only successful routine-open snapshots rotate. Failed checks and migration/WAL
/// recovery snapshots stay pinned; never delete the last known recovery data on failure.
pub(super) fn mark_verified(saved: &Path, pinned: bool) -> Result<()> {
    let file = saved.join("manifest.json");
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&file)?)?;
    value["verified"] = true.into();
    value["pinned"] = (pinned || value["pinned"] == true).into();
    crate::export::write_atomic(&file, &serde_json::to_string_pretty(&value)?)?;
    let parent = saved.parent().context("Missing recovery directory")?;
    let mut routine = Vec::new();
    for entry in fs::read_dir(parent)? {
        let path = entry?.path();
        let owned = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_prefix("before-open-"))
            .is_some_and(|id| id.len() == 64 && id.bytes().all(|b| b.is_ascii_hexdigit()));
        if !owned || !path.is_dir() || fs::symlink_metadata(&path)?.file_type().is_symlink() {
            continue;
        }
        let Ok(bytes) = fs::read(path.join("manifest.json")) else {
            continue;
        };
        let Ok(info) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        if info["verified"] == true && info["pinned"] == false {
            routine.push((info["created_at"].as_u64().unwrap_or(0), path));
        }
    }
    routine.retain(|(_, path)| path != saved);
    routine.sort();
    let keep_others = if value["pinned"] == true { 3 } else { 2 };
    let excess = routine.len().saturating_sub(keep_others);
    for (_, path) in routine.into_iter().take(excess) {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}
/// Full comparison includes history, properties, views, preferences and exact attachment bytes.
pub async fn rehearse(
    notebook: &Notebook,
    vault: Option<(&VaultConfig, &Key)>,
    parent: &Path,
) -> Result<()> {
    notebook.validate()?;
    let staging = tempfile::Builder::new()
        .prefix("restore-check-")
        .tempdir_in(parent)?;
    let path = staging.path().join("check.db");
    let assets = workspace::asset_dir(&path);
    let encryption = || vault.map(|(cfg, key)| cfg.encryption_options(key));
    {
        let target = Store::open_with_encryption(&path, encryption()).await?;
        target.restore_notebook(notebook, &assets).await?;
        target.verify_storage().await?;
        ensure!(
            target.notebook(&assets).await? == *notebook,
            "Restored backup does not match the notebook"
        );
    }
    let reopened = Store::open_with_encryption(&path, encryption()).await?;
    reopened.verify_storage().await?;
    ensure!(
        reopened.notebook(&assets).await? == *notebook,
        "Backup verification failed after reopening Turso"
    );
    Ok(())
}
pub fn read_back(expected: &Notebook, path: &Path, key: Option<&Key>) -> Result<()> {
    ensure!(
        security::read_backup(path, key, None)? == *expected,
        "The backup read-back differs from the notebook"
    );
    Ok(())
}
pub async fn write_backup(
    store: &Store,
    database: &Path,
    path: &Path,
    vault: Option<(&VaultConfig, &Key)>,
    for_update: bool,
) -> Result<()> {
    store.verify_storage().await?;
    let expected = store.notebook(&workspace::asset_dir(database)).await?;
    if let Some((config, key)) = vault {
        security::write_backup(&expected, path, config, key)?;
    } else {
        expected.write(path)?;
    }
    read_back(&expected, path, vault.map(|(_, key)| key))?;
    if for_update {
        certify(&expected, path, vault).await?;
    }
    Ok(())
}
async fn certify(
    expected: &Notebook,
    path: &Path,
    vault: Option<(&VaultConfig, &Key)>,
) -> Result<()> {
    let parent = path.parent().context("Missing backup directory")?;
    let initial_hash = crate::updates::hash_file(path)?;
    rehearse(expected, vault, parent)
        .await
        .context("The pre-update backup could not be restored; update cancelled")?;
    read_back(expected, path, vault.map(|(_, key)| key))?;
    ensure!(
        crate::updates::hash_file(path)? == initial_hash,
        "Backup changed during restoration verification"
    );
    let proof = BackupProof {
        schema: 1,
        sha256: crate::updates::hash_file(path)?,
        size: path.metadata()?.len(),
        storage_contract: crate::updates::config().storage_contract,
        restored_and_reopened: true,
        verified_by_version: crate::updates::VERSION.into(),
        verified_by_revision: crate::updates::REVISION.into(),
        schema_version: super::SCHEMA_VERSION,
    };
    crate::export::write_atomic(
        &BackupProof::path(path),
        &serde_json::to_string_pretty(&proof)?,
    )?;
    BackupProof::verify(path)?;
    Ok(())
}
/// Offline CLI check for synthetic fixtures / a copied plaintext backup. No live profile is opened.
pub async fn certify_plain_backup(path: &Path) -> Result<()> {
    let expected = security::read_backup(path, None, None)?;
    certify(&expected, path, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_snapshot_timestamps_preserve_current_pinned_failed_and_manual_copies() {
        let dir = tempfile::tempdir().unwrap();
        for letter in ['a', 'b', 'c', 'd', 'e', 'f'] {
            let path = dir
                .path()
                .join(format!("before-open-{}", letter.to_string().repeat(64)));
            fs::create_dir(&path).unwrap();
            fs::write(path.join("manifest.json"), serde_json::json!({"schema":1,"verified":letter!='f',"pinned":letter=='e',"created_at":0}).to_string()).unwrap();
        }
        let manual = dir.path().join("manual-recovery");
        fs::create_dir(&manual).unwrap();
        fs::write(
            manual.join("manifest.json"),
            serde_json::json!({"schema":1,"verified":true,"pinned":false,"created_at":0})
                .to_string(),
        )
        .unwrap();
        let current = dir.path().join(format!("before-open-{}", "a".repeat(64)));
        mark_verified(&current, false).unwrap();
        assert!(current.exists() && manual.exists());
        for letter in ['e', 'f'] {
            assert!(
                dir.path()
                    .join(format!("before-open-{}", letter.to_string().repeat(64)))
                    .exists()
            );
        }
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 6);
    }
}
