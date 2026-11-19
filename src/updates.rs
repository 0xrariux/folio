//! Signed, prebuilt Apple Silicon releases. Source builds belong to the developer workflow.
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use ring::{digest, signature};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::JoinHandle,
    time::Duration,
};
mod installer;
pub use installer::{acknowledge_startup, apply, handoff, installed_bundle};
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const REVISION: &str = env!("FOLIO_BUILD_REVISION");
pub const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    pub repository: String,
    pub bundle_id: String,
    pub target: String,
    pub minimum_macos: String,
    pub storage_contract: String,
    pub public_key: String,
}
pub fn config() -> Config {
    let cfg: Config = serde_json::from_str(include_str!("../release.json"))
        .expect("valid embedded release configuration");
    assert_eq!(
        cfg.storage_contract,
        crate::storage::storage_contract(),
        "release.json must match the resolved Turso engine and implemented SQL/document/encryption formats"
    );
    cfg
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub version: String,
    pub revision: String,
    pub target: String,
    pub bundle_id: String,
    pub minimum_macos: String,
    pub storage_contract: String,
    pub archive: String,
    pub size: u64,
    pub sha256: String,
}
#[derive(Clone, Debug)]
pub struct SignedRelease {
    pub manifest: Manifest,
    pub bytes: Vec<u8>,
    pub signature: String,
}
#[derive(Clone, Debug)]
pub struct RemoteBuild {
    pub revision: String,
    pub version: String,
    pub ahead: bool,
    pub release: SignedRelease,
}
pub fn valid_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
pub fn verify(bytes: &[u8], signed: &str, cfg: &Config) -> Result<SignedRelease> {
    ensure!(
        bytes.len() <= 65536 && signed.len() <= 256,
        "Release metadata is too large"
    );
    let key = STANDARD
        .decode(&cfg.public_key)
        .context("Invalid update public key")?;
    let sig = STANDARD
        .decode(signed.trim())
        .context("Invalid release signature")?;
    signature::UnparsedPublicKey::new(&signature::ED25519, key)
        .verify(bytes, &sig)
        .map_err(|_| anyhow::anyhow!("Release signature verification failed"))?;
    let m: Manifest = serde_json::from_slice(bytes)?;
    ensure!(
        cfg.schema == 1 && m.schema == 1,
        "Unsupported release format"
    );
    let v = semver::Version::parse(&m.version)?;
    ensure!(
        v.pre.is_empty() && v.build.is_empty(),
        "Only stable releases can be installed"
    );
    ensure!(valid_revision(&m.revision), "Invalid release revision");
    ensure!(
        m.target == cfg.target && m.bundle_id == cfg.bundle_id,
        "Release does not match this application"
    );
    ensure!(
        m.storage_contract == cfg.storage_contract,
        "This release requires a database migration. Install it manually after reviewing its release notes."
    );
    ensure!(
        m.minimum_macos == cfg.minimum_macos,
        "Release requires a different macOS compatibility level"
    );
    ensure!(
        m.archive == format!("Folio-{}-macos-arm64.zip", m.version),
        "Invalid archive name"
    );
    ensure!(m.size > 0 && m.size <= MAX_ARCHIVE, "Invalid archive size");
    ensure!(
        m.sha256.len() == 64 && m.sha256.bytes().all(|c| c.is_ascii_hexdigit()),
        "Invalid archive checksum"
    );
    Ok(SignedRelease {
        manifest: m,
        bytes: bytes.to_vec(),
        signature: signed.trim().into(),
    })
}
fn client() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .https_only(true)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(180))
        .user_agent(concat!("Folio/", env!("CARGO_PKG_VERSION")))
        .build()?)
}
fn limited(mut response: reqwest::blocking::Response, max: u64) -> Result<Vec<u8>> {
    response.error_for_status_ref()?;
    let mut out = Vec::new();
    Read::by_ref(&mut response)
        .take(max + 1)
        .read_to_end(&mut out)?;
    ensure!(out.len() as u64 <= max, "Download exceeds its size limit");
    Ok(out)
}
pub fn asset_url(cfg: &Config, version: &str, name: &str) -> String {
    format!(
        "https://github.com/{}/releases/download/v{version}/{name}",
        cfg.repository
    )
}
pub fn check() -> Result<RemoteBuild> {
    let cfg = config();
    ensure!(
        !cfg.public_key.is_empty(),
        "Update signing is not configured"
    );
    let client = client()?;
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
        draft: bool,
        prerelease: bool,
    }
    let response = client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            cfg.repository
        ))
        .send()?;
    ensure!(
        response.status().as_u16() != 404,
        "No public release available yet"
    );
    let published: Release = serde_json::from_slice(&limited(response, 1024 * 1024)?)?;
    ensure!(
        !published.draft && !published.prerelease,
        "No stable release available"
    );
    let version = published
        .tag_name
        .strip_prefix('v')
        .context("Invalid release tag")?;
    let parsed = semver::Version::parse(version)?;
    ensure!(parsed.to_string() == version, "Invalid release version");
    let bytes = limited(
        client
            .get(asset_url(&cfg, version, "folio-release.json"))
            .send()?,
        65536,
    )?;
    let sig = String::from_utf8(limited(
        client
            .get(asset_url(&cfg, version, "folio-release.sig"))
            .send()?,
        256,
    )?)?;
    let release = verify(&bytes, &sig, &cfg)?;
    ensure!(
        release.manifest.version == version,
        "Release tag and manifest disagree"
    );
    let ahead = parsed > semver::Version::parse(VERSION)?;
    Ok(RemoteBuild {
        revision: release.manifest.revision.clone(),
        version: version.into(),
        ahead,
        release,
    })
}
pub fn hash_file(path: &Path) -> Result<String> {
    let mut hash = digest::Context::new(&digest::SHA256);
    let mut file = File::open(path)?;
    let mut buf = [0; 65536];
    loop {
        let count = file.read(&mut buf)?;
        if count == 0 {
            break;
        }
        hash.update(&buf[..count]);
    }
    Ok(hash
        .finish()
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}
pub fn verify_archive(path: &Path, m: &Manifest) -> Result<()> {
    ensure!(
        path.metadata()?.len() == m.size,
        "Archive size does not match the signed release"
    );
    ensure!(
        hash_file(path)? == m.sha256.to_lowercase(),
        "Archive checksum verification failed"
    );
    Ok(())
}
pub fn validate_release_archive(archive: &Path, release: &SignedRelease) -> Result<()> {
    verify_archive(archive, &release.manifest)?;
    let directory = tempfile::tempdir()?;
    installer::unpack_and_validate(directory.path(), archive, &release.manifest)?;
    Ok(())
}
pub struct Prepared {
    pub(crate) directory: tempfile::TempDir,
    pub(crate) release: SignedRelease,
}
pub enum Event {
    Stage(String),
    Ready(Box<Prepared>),
    Failed(String),
}
pub struct Task {
    pub events: Receiver<Event>,
    cancel: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Task {
    pub fn start(remote: RemoteBuild, wake: rune::Waker) -> Self {
        let (tx, events) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancellation = cancel.clone();
        let thread = std::thread::spawn(move || {
            let _ = tx.send(Event::Stage("Downloading update…".into()));
            wake();
            let result = prepare(&remote, &cancellation);
            let _ = tx.send(match result {
                Ok(p) => Event::Ready(Box::new(p)),
                Err(e) => Event::Failed(format!("{e:#}")),
            });
            wake();
        });
        Self {
            events,
            cancel,
            thread: Some(thread),
        }
    }
}
impl Drop for Task {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        // Network reads have a timeout. Do not block the UI while cancelling a download.
        self.thread.take();
    }
}
fn prepare(remote: &RemoteBuild, cancel: &AtomicBool) -> Result<Prepared> {
    installed_bundle(&std::env::current_exe()?)?;
    let cfg = config();
    let release = verify(&remote.release.bytes, &remote.release.signature, &cfg)?;
    ensure!(
        semver::Version::parse(&release.manifest.version)? > semver::Version::parse(VERSION)?,
        "A newer version is required"
    );
    let cache = crate::storage::database_path().with_extension("updates");
    fs::create_dir_all(&cache)?;
    let directory = tempfile::Builder::new()
        .prefix("release-")
        .tempdir_in(cache)?;
    let archive = directory.path().join(&release.manifest.archive);
    let mut response = client()?
        .get(asset_url(
            &cfg,
            &release.manifest.version,
            &release.manifest.archive,
        ))
        .send()?
        .error_for_status()?;
    let mut file = File::create(&archive)?;
    let mut bytes = 0u64;
    let mut buf = [0; 65536];
    loop {
        ensure!(!cancel.load(Ordering::Relaxed), "Update cancelled");
        let n = response.read(&mut buf)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        ensure!(
            bytes <= release.manifest.size,
            "Archive exceeds signed size"
        );
        file.write_all(&buf[..n])?;
    }
    file.sync_all()?;
    verify_archive(&archive, &release.manifest)?;
    installer::unpack_and_validate(directory.path(), &archive, &release.manifest)?;
    Ok(Prepared { directory, release })
}
#[cfg(test)]
mod tests {
    use super::*;
    use ring::{
        rand::SystemRandom,
        signature::{Ed25519KeyPair, KeyPair},
    };
    fn fixture() -> (Config, Manifest, Ed25519KeyPair) {
        let doc = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new()).unwrap();
        let key = Ed25519KeyPair::from_pkcs8(doc.as_ref()).unwrap();
        let mut cfg = config();
        cfg.public_key = STANDARD.encode(key.public_key().as_ref());
        let m = Manifest {
            schema: 1,
            version: "0.13.0".into(),
            revision: "a".repeat(40),
            target: cfg.target.clone(),
            bundle_id: cfg.bundle_id.clone(),
            minimum_macos: cfg.minimum_macos.clone(),
            storage_contract: cfg.storage_contract.clone(),
            archive: "Folio-0.13.0-macos-arm64.zip".into(),
            size: 3,
            sha256: "a".repeat(64),
        };
        (cfg, m, key)
    }
    #[test]
    fn signed_metadata_rejects_tampering_and_wrong_key() {
        let (cfg, m, key) = fixture();
        let bytes = serde_json::to_vec(&m).unwrap();
        let sig = STANDARD.encode(key.sign(&bytes).as_ref());
        assert!(verify(&bytes, &sig, &cfg).is_ok());
        let mut changed = bytes.clone();
        changed[10] ^= 1;
        assert!(verify(&changed, &sig, &cfg).is_err());
        assert!(verify(&bytes, &sig, &fixture().0).is_err());
        assert!(verify(&bytes, "", &cfg).is_err());
    }
    #[test]
    fn authenticated_release_still_requires_compatibility() {
        let (cfg, m, key) = fixture();
        for field in [
            "schema",
            "target",
            "bundle_id",
            "minimum_macos",
            "storage_contract",
            "archive",
            "version",
            "size",
            "revision",
        ] {
            let mut value = serde_json::to_value(&m).unwrap();
            value[field] = match field {
                "schema" => 2.into(),
                "size" => (MAX_ARCHIVE + 1).into(),
                "version" => "0.13.0-beta.1".into(),
                _ => "wrong".into(),
            };
            let bytes = serde_json::to_vec(&value).unwrap();
            assert!(
                verify(&bytes, &STANDARD.encode(key.sign(&bytes).as_ref()), &cfg).is_err(),
                "{field}"
            );
        }
    }
    #[test]
    fn signed_size_and_hash_are_both_enforced() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("archive");
        fs::write(&p, b"abc").unwrap();
        let (_, mut m, _) = fixture();
        m.sha256 = hash_file(&p).unwrap();
        assert!(verify_archive(&p, &m).is_ok());
        fs::write(&p, b"abd").unwrap();
        assert!(verify_archive(&p, &m).is_err());
        fs::write(&p, b"abcd").unwrap();
        assert!(verify_archive(&p, &m).is_err());
    }
}
