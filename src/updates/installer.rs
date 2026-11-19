use super::*;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Serialize, Deserialize)]
struct Plan {
    target: PathBuf,
    parent_pid: u32,
    backup: PathBuf,
    database: PathBuf,
}
fn run(command: &mut Command) -> Result<()> {
    let output = command.output()?;
    ensure!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
}
fn info(bundle: &Path) -> Result<serde_json::Value> {
    let output = Command::new("/usr/bin/plutil")
        .args(["-convert", "json", "-o", "-"])
        .arg(bundle.join("Contents/Info.plist"))
        .output()?;
    ensure!(output.status.success(), "Invalid application bundle");
    Ok(serde_json::from_slice(&output.stdout)?)
}
pub fn installed_bundle(executable: &Path) -> Result<PathBuf> {
    ensure!(
        cfg!(all(target_os = "macos", target_arch = "aarch64")),
        "Updates require Apple Silicon macOS"
    );
    let executable = executable.canonicalize()?;
    let macos = executable.parent().context("Missing executable folder")?;
    let contents = macos.parent().context("Missing bundle contents")?;
    let bundle = contents.parent().context("Missing application bundle")?;
    ensure!(
        macos.file_name().is_some_and(|n| n == "MacOS")
            && contents.file_name().is_some_and(|n| n == "Contents")
            && bundle.extension().is_some_and(|e| e == "app"),
        "Open Folio.app to install updates"
    );
    let plist = info(bundle)?;
    ensure!(
        plist["CFBundleIdentifier"].as_str() == Some(config().bundle_id.as_str())
            && plist["CFBundleExecutable"] == "folio",
        "Unexpected application bundle identity"
    );
    // Finder translocation and read-only volumes cannot receive an in-place update.
    let parent = bundle.parent().context("Missing installation folder")?;
    ensure!(
        !bundle.to_string_lossy().contains("/AppTranslocation/"),
        "Move Folio.app to Applications before updating"
    );
    tempfile::Builder::new()
        .prefix(".folio-write-check-")
        .tempdir_in(parent)
        .context("Move Folio.app to a writable Applications folder before updating")?;
    Ok(bundle.to_owned())
}
fn reject_symlinks(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    ensure!(
        !m.file_type().is_symlink(),
        "Symlinks are not allowed in a Folio release"
    );
    if m.is_dir() {
        for entry in fs::read_dir(path)? {
            reject_symlinks(&entry?.path())?;
        }
    } else {
        ensure!(m.is_file(), "Unexpected release file type");
    }
    Ok(())
}
fn validate_bundle(bundle: &Path, m: &Manifest) -> Result<()> {
    reject_symlinks(bundle)?;
    let plist = info(bundle)?;
    ensure!(
        plist["CFBundleIdentifier"] == m.bundle_id
            && plist["CFBundleExecutable"] == "folio"
            && plist["CFBundleShortVersionString"] == m.version
            && plist["CFBundleVersion"] == m.version,
        "Bundle identity/version does not match its signed manifest"
    );
    ensure!(
        plist["LSMinimumSystemVersion"] == m.minimum_macos,
        "Bundle compatibility does not match its signed manifest"
    );
    let provenance: serde_json::Value = serde_json::from_slice(&fs::read(
        bundle.join("Contents/Resources/build-info.json"),
    )?)?;
    ensure!(
        provenance["version"] == m.version
            && provenance["revision"] == m.revision
            && provenance["storage_contract"] == m.storage_contract
            && provenance["dirty"] == false,
        "Bundle build provenance does not match the release"
    );
    let binary = bundle.join("Contents/MacOS/folio");
    let mut header = [0; 8];
    File::open(&binary)?.read_exact(&mut header)?;
    ensure!(
        header == [0xcf, 0xfa, 0xed, 0xfe, 0x0c, 0, 0, 1],
        "Release executable must be a native arm64 Mach-O binary"
    );
    run(Command::new("/usr/bin/codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(bundle))?;
    Ok(())
}
pub(super) fn unpack_and_validate(folder: &Path, archive: &Path, m: &Manifest) -> Result<PathBuf> {
    // Archive bytes have already passed Ed25519 + size + SHA256 verification.
    // Still reject absolute/traversing ZIP entries before extraction.
    let listing = Command::new("/usr/bin/unzip")
        .arg("-Z1")
        .arg(archive)
        .output()?;
    ensure!(listing.status.success(), "Invalid ZIP archive");
    let listing = String::from_utf8(listing.stdout)?;
    ensure!(
        listing.lines().all(|line| {
            let path = Path::new(line);
            !path.is_absolute()
                && path
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_)))
                && (line.starts_with("Folio.app/") || line.starts_with("__MACOSX/"))
        }),
        "Unsafe archive path"
    );
    let attributes = Command::new("/usr/bin/unzip")
        .args(["-Z", "-l"])
        .arg(archive)
        .output()?;
    ensure!(
        attributes.status.success(),
        "Cannot inspect archive attributes"
    );
    let attributes = String::from_utf8(attributes.stdout)?;
    ensure!(
        !attributes
            .lines()
            .any(|line| line.starts_with(['l', 'p', 'b', 'c', 's'])),
        "Special files and symlinks are not allowed in a release archive"
    );
    let extracted = folder.join("unpacked");
    if extracted.exists() {
        fs::remove_dir_all(&extracted)?;
    }
    fs::create_dir(&extracted)?;
    run(Command::new("/usr/bin/ditto")
        .args(["-x", "-k"])
        .arg(archive)
        .arg(&extracted))?;
    let bundle = extracted.join("Folio.app");
    validate_bundle(&bundle, m)?;
    Ok(bundle)
}
/// Prepare a separate installer process. It waits for the UI and storage worker to exit.
/// The private staging directory is retained as an installation log/recovery record.
pub fn handoff(prepared: Prepared, backup: &Path) -> Result<()> {
    crate::storage::recovery::BackupProof::verify(backup)?;
    let current = std::env::current_exe()?;
    let target = installed_bundle(&current)?;
    let folder = prepared.directory.path();
    fs::write(folder.join("folio-release.json"), &prepared.release.bytes)?;
    fs::write(
        folder.join("folio-release.sig"),
        &prepared.release.signature,
    )?;
    let helper = folder.join("folio-installer");
    fs::copy(&current, &helper)?;
    let plan = Plan {
        target,
        parent_pid: std::process::id(),
        backup: backup.canonicalize()?,
        database: crate::storage::database_path(),
    };
    let plan_path = folder.join("install.json");
    fs::write(&plan_path, serde_json::to_vec(&plan)?)?;
    let log = File::create(folder.join("install.log"))?;
    Command::new(helper)
        .arg("--apply-update")
        .arg(&plan_path)
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()?;
    let _ = prepared.directory.keep();
    Ok(())
}
fn alive(pid: u32) -> bool {
    // kill(pid, 0) only probes existence; this does not signal another process.
    unsafe { libc::kill(pid as libc::pid_t, 0) == 0 }
}
fn launch(
    target: &Path,
    plan: &Plan,
    receipt: Option<&Path>,
    error: Option<&str>,
) -> Result<std::process::Child> {
    let mut cmd = Command::new(target.join("Contents/MacOS/folio"));
    cmd.env("FOLIO_DB_PATH", &plan.database)
        .env_remove("FOLIO_DEMO")
        .env_remove("FOLIO_UPDATE_HEALTH")
        .env_remove("FOLIO_UPDATE_ERROR")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(path) = receipt {
        cmd.env("FOLIO_UPDATE_HEALTH", path);
    }
    if let Some(message) = error {
        cmd.env("FOLIO_UPDATE_ERROR", message);
    }
    Ok(cmd.spawn()?)
}
/// Two renames with an explicit rollback on replacement or launch failure.
fn replace(
    target: &Path,
    staged: &Path,
    previous: &Path,
    start: impl FnOnce() -> Result<()>,
) -> Result<()> {
    ensure!(!previous.exists(), "Rollback bundle already exists");
    fs::rename(target, previous).context("Cannot preserve the previous application")?;
    if let Err(error) = fs::rename(staged, target) {
        fs::rename(previous, target).context("Cannot restore previous application")?;
        return Err(error.into());
    }
    if let Err(error) = start() {
        fs::rename(target, staged).context("Cannot remove the failed update")?;
        fs::rename(previous, target).context("Cannot restore previous application")?;
        return Err(error);
    }
    Ok(())
}
pub fn apply(plan_path: &Path) -> Result<()> {
    let plan_path = plan_path.canonicalize()?;
    let folder = plan_path
        .parent()
        .context("Missing installation directory")?;
    let plan: Plan = serde_json::from_slice(&fs::read(&plan_path)?)?;
    let result = apply_inner(folder, &plan);
    if let Err(error) = &result {
        eprintln!("Installation failed: {error:#}");
        let restarted = if !alive(plan.parent_pid) && plan.target.exists() {
            launch(
                &plan.target,
                &plan,
                None,
                Some("Update failed. See the installation log in your data folder."),
            )
            .ok()
            .map(|child| child.id())
        } else {
            None
        };
        let _ = fs::write(
            folder.join("failed.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "error": format!("{error:#}"), "restarted_pid": restarted, "target": plan.target, "backup": plan.backup,
            }))?,
        );
    }
    result
}
fn apply_inner(folder: &Path, plan: &Plan) -> Result<()> {
    ensure!(plan.parent_pid > 1, "Invalid parent process");
    let deadline = Instant::now() + Duration::from_secs(60);
    while alive(plan.parent_pid) {
        ensure!(
            Instant::now() < deadline,
            "Folio did not close; update cancelled"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    // Serialize installation against other installers. The application worker itself
    // serializes database access. No database is opened or rewritten by this helper.
    let lock = File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(
            plan.target
                .parent()
                .context("Missing installation folder")?
                .join(".folio-install.lock"),
        )?;
    lock.try_lock().context("Another installation is running")?;
    let release = verify(
        &fs::read(folder.join("folio-release.json"))?,
        &fs::read_to_string(folder.join("folio-release.sig"))?,
        &config(),
    )?;
    let m = &release.manifest;
    ensure!(
        semver::Version::parse(&m.version)? > semver::Version::parse(VERSION)?,
        "Refusing a downgrade or same-version update"
    );
    crate::storage::recovery::BackupProof::verify(&plan.backup)?;
    ensure!(
        installed_bundle(&plan.target.join("Contents/MacOS/folio"))? == plan.target,
        "Installation target changed"
    );
    let archive = folder.join(&m.archive);
    verify_archive(&archive, m)?;
    let unpacked = unpack_and_validate(folder, &archive, m)?;
    let parent = plan
        .target
        .parent()
        .context("Missing installation folder")?;
    let swap = tempfile::Builder::new()
        .prefix(".folio-install-")
        .tempdir_in(parent)?;
    let staged = swap.path().join("Folio.app");
    run(Command::new("/usr/bin/ditto").arg(&unpacked).arg(&staged))?;
    validate_bundle(&staged, m)?;
    let previous = parent.join(format!(".Folio.previous-{}.app", uuid::Uuid::new_v4()));
    let receipt = folder.join(format!("health-{}.json", uuid::Uuid::new_v4()));
    fs::write(
        folder.join("recovery.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"target": plan.target, "previous": previous, "backup": plan.backup}),
        )?,
    )?;
    crate::storage::recovery::BackupProof::verify(&plan.backup)?;
    replace(&plan.target, &staged, &previous, || {
        let mut child = launch(&plan.target, plan, Some(&receipt), None)?;
        let health = (|| -> Result<()> {
            let deadline = Instant::now() + Duration::from_secs(90);
            loop {
                ensure!(
                    child.try_wait()?.is_none(),
                    "New application exited before startup completed"
                );
                if receipt.exists() {
                    let ack: serde_json::Value = serde_json::from_slice(&fs::read(&receipt)?)?;
                    ensure!(
                        ack["pid"] == child.id()
                            && ack["revision"] == m.revision
                            && ack["version"] == m.version,
                        "Invalid startup acknowledgement"
                    );
                    return Ok(());
                }
                ensure!(
                    Instant::now() < deadline,
                    "New application did not finish startup"
                );
                std::thread::sleep(Duration::from_millis(200));
            }
        })();
        if health.is_err() {
            let _ = child.kill();
            let _ = child.wait();
        }
        health
    })?;
    if let Err(error) = fs::write(
        folder.join("installed.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"version": m.version, "revision": m.revision, "previous": previous, "backup": plan.backup}),
        )?,
    ) {
        eprintln!("Installed successfully, but could not write completion receipt: {error}");
    }
    // Keep the rollback bundle and backup, but discard duplicate downloads and helper.
    for name in [&m.archive, "folio-installer"] {
        let _ = fs::remove_file(folder.join(name));
    }
    let _ = fs::remove_dir_all(folder.join("unpacked"));
    println!(
        "Installed Folio {}. Previous application: {}",
        m.version,
        previous.display()
    );
    Ok(())
}
/// Called only after a successful data load (or the vault lock screen) in the UI poll.
pub fn acknowledge_startup() {
    use std::sync::atomic::AtomicBool;
    static ACK: AtomicBool = AtomicBool::new(false);
    if ACK.swap(true, Ordering::Relaxed) {
        return;
    }
    let Some(path) = std::env::var_os("FOLIO_UPDATE_HEALTH") else {
        return;
    };
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent()
        && let Ok(mut file) = tempfile::NamedTempFile::new_in(parent)
        && serde_json::to_writer(&mut file, &serde_json::json!({"pid": std::process::id(), "version": VERSION, "revision": REVISION})).is_ok()
        && file.as_file().sync_all().is_ok()
    {
        let _ = file.persist_noclobber(path);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launch_failure_restores_whole_previous_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("Folio.app");
        let new = dir.path().join("staged.app");
        let backup = dir.path().join("previous.app");
        fs::create_dir(&old).unwrap();
        fs::write(old.join("version"), "old").unwrap();
        fs::create_dir(&new).unwrap();
        fs::write(new.join("version"), "new").unwrap();
        assert!(replace(&old, &new, &backup, || anyhow::bail!("launch failed")).is_err());
        assert_eq!(fs::read_to_string(old.join("version")).unwrap(), "old");
        assert_eq!(fs::read_to_string(new.join("version")).unwrap(), "new");
        assert!(!backup.exists());
    }
    #[test]
    fn successful_install_retains_previous_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("Folio.app");
        let new = dir.path().join("staged.app");
        let backup = dir.path().join("previous.app");
        fs::create_dir(&old).unwrap();
        fs::create_dir(&new).unwrap();
        fs::write(new.join("new"), "ok").unwrap();
        replace(&old, &new, &backup, || Ok(())).unwrap();
        assert!(old.join("new").is_file());
        assert!(backup.is_dir());
        assert!(!new.exists());
    }
    #[test]
    fn missing_staged_bundle_does_not_lose_installed_app() {
        let dir = tempfile::tempdir().unwrap();
        let old = dir.path().join("Folio.app");
        fs::create_dir(&old).unwrap();
        assert!(
            replace(
                &old,
                &dir.path().join("missing"),
                &dir.path().join("previous"),
                || Ok(())
            )
            .is_err()
        );
        assert!(old.is_dir());
    }
}
