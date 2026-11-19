//! Versioned module lifecycle and private, opaque module state. V1 loads metadata only.
//! No executable, shell command, URL or native library is run by this host.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};

pub const API_VERSION: u32 = 1;
pub const MAX_MANIFEST: u64 = 32 * 1024;
pub const MAX_CATALOG: u64 = 1024 * 1024;
pub const MAX_MODULE_DATA: usize = 8 * 1024 * 1024;
pub const MAX_REGISTRY: usize = 64 * 1024 * 1024;
fn one() -> u32 {
    1
}
fn empty_data() -> Value {
    serde_json::json!({})
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub format: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub description: String,
    pub folio: String,
    pub api: u32,
    pub runtime: String,
    pub data_version: u32,
    #[serde(default)]
    pub permissions: Vec<String>,
}
pub fn valid_id(id: &str) -> bool {
    (3..=128).contains(&id.len())
        && id.contains('.')
        && id.split('.').all(|part| {
            !part.is_empty()
                && part.bytes().next().is_some_and(|c| c.is_ascii_lowercase())
                && part.bytes().all(|c| {
                    c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'-' | b'_')
                })
        })
}
pub fn digest(value: &Value) -> String {
    ring::digest::digest(&ring::digest::SHA256, value.to_string().as_bytes())
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
impl Manifest {
    pub fn parse(value: &Value) -> Result<Self> {
        ensure!(
            value.to_string().len() as u64 <= MAX_MANIFEST,
            "Module definition is too large"
        );
        let m: Self =
            serde_json::from_value(value.clone()).context("Unsupported module definition")?;
        ensure!(
            valid_id(&m.id),
            "Use a reverse-domain module ID, such as org.example.name"
        );
        ensure!(
            !m.name.trim().is_empty()
                && m.name.len() <= 100
                && !m.author.trim().is_empty()
                && m.author.len() <= 120
                && m.description.len() <= 2000,
            "Invalid module name, author or description"
        );
        semver::Version::parse(&m.version).context("Invalid module version")?;
        semver::VersionReq::parse(&m.folio).context("Invalid Folio version requirement")?;
        ensure!(
            m.data_version > 0 && m.permissions.len() <= 16,
            "Invalid module data version or permissions"
        );
        let mut seen = HashSet::new();
        ensure!(
            m.permissions
                .iter()
                .all(|p| p.len() <= 80 && seen.insert(p)),
            "Duplicate or invalid module permissions"
        );
        Ok(m)
    }
    pub fn compatibility(&self) -> Result<()> {
        ensure!(
            self.format == 1 && self.api == API_VERSION,
            "Requires a different module API"
        );
        ensure!(
            self.runtime == "manifest",
            "This runtime is not available in Folio yet"
        );
        ensure!(
            self.permissions.iter().all(|p| p == "storage"),
            "This module requires capabilities that are not available yet"
        );
        ensure!(
            semver::VersionReq::parse(&self.folio)?
                .matches(&semver::Version::parse(crate::updates::VERSION)?),
            "Requires Folio {}",
            self.folio
        );
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Candidate {
    pub manifest: Value,
    pub metadata: Manifest,
    pub fingerprint: String,
}
impl Candidate {
    pub fn new(manifest: Value) -> Result<Self> {
        let metadata = Manifest::parse(&manifest)?;
        metadata.compatibility()?;
        let fingerprint = digest(&manifest);
        Ok(Self {
            manifest,
            metadata,
            fingerprint,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub format: u32,
    pub id: String,
    pub name: String,
    pub modules: Vec<Value>,
}
impl Catalog {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == 1
                && valid_id(&self.id)
                && !self.name.trim().is_empty()
                && self.name.len() <= 120
                && self.modules.len() <= 512,
            "Invalid module catalog"
        );
        ensure!(
            serde_json::to_vec(self)?.len() as u64 <= MAX_CATALOG,
            "Module catalog is too large"
        );
        let mut ids = HashSet::new();
        for value in &self.modules {
            let id = value
                .get("id")
                .and_then(Value::as_str)
                .context("Missing module ID in catalog")?;
            ensure!(
                valid_id(id) && ids.insert(id) && value.to_string().len() as u64 <= MAX_MANIFEST,
                "Duplicate, invalid or oversized module in catalog"
            );
        }
        Ok(())
    }
    pub fn built_in() -> Self {
        serde_json::from_str(include_str!("../module-library.json"))
            .expect("valid built-in module catalog")
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    // Keep the manifest opaque at rest: an unavailable/newer module must not block the notebook.
    pub manifest: Value,
    pub fingerprint: String,
    pub installed: bool,
    pub enabled: bool,
    pub grants: Vec<String>,
    pub data_version: u32,
    pub data: Value,
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    pub instance: uuid::Uuid,
    pub installed_at: i64,
    pub updated_at: i64,
}
impl Record {
    fn invalidate_sessions(&mut self) -> Result<()> {
        self.generation = self
            .generation
            .checked_add(1)
            .context("Module session counter exhausted")?;
        Ok(())
    }
    pub fn metadata(&self) -> Result<Manifest> {
        Manifest::parse(&self.manifest)
    }
    pub fn active(&self) -> bool {
        self.installed
            && self.enabled
            && self.metadata().is_ok_and(|m| {
                m.compatibility().is_ok()
                    && m.permissions == self.grants
                    && m.data_version == self.data_version
            })
    }
    pub fn name(&self) -> &str {
        self.manifest
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("Unavailable module")
    }
    pub fn has_data(&self) -> bool {
        self.data != Value::Null && self.data != empty_data()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    #[serde(default = "one")]
    pub format: u32,
    #[serde(default)]
    pub records: BTreeMap<String, Record>,
    #[serde(default)]
    pub catalogs: BTreeMap<String, Catalog>,
}
impl Default for Registry {
    fn default() -> Self {
        Self {
            format: 1,
            records: BTreeMap::new(),
            catalogs: BTreeMap::new(),
        }
    }
}
impl Registry {
    pub fn is_empty(&self) -> bool {
        self.records.is_empty() && self.catalogs.is_empty()
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == 1 && self.records.len() <= 512 && self.catalogs.len() <= 16,
            "Unsupported module registry"
        );
        ensure!(
            serde_json::to_vec(self)?.len() <= MAX_REGISTRY,
            "Module storage exceeds 64 MiB"
        );
        for (id, record) in &self.records {
            ensure!(
                valid_id(id)
                    && record.manifest.get("id").and_then(Value::as_str) == Some(id.as_str())
                    && record.fingerprint == digest(&record.manifest),
                "Module identity or fingerprint is damaged"
            );
            ensure!(
                record.data_version > 0 && record.data.to_string().len() <= MAX_MODULE_DATA,
                "Invalid module data"
            );
            ensure!(
                record.installed || !record.enabled,
                "An uninstalled module cannot be enabled"
            );
            ensure!(
                record.grants.len() <= 16
                    && record.grants.iter().all(|p| !p.is_empty() && p.len() <= 80),
                "Invalid stored module permissions"
            );
        }
        for (id, catalog) in &self.catalogs {
            catalog.validate()?;
            ensure!(
                id == &catalog.id && id != &Catalog::built_in().id,
                "Invalid catalog identity"
            );
        }
        Ok(())
    }
    pub fn apply(&mut self, action: Action) -> Result<()> {
        // Mutate a private candidate and publish only after all validation succeeds.
        let mut next = self.clone();
        next.apply_inner(action)?;
        next.validate()?;
        *self = next;
        Ok(())
    }
    fn apply_inner(&mut self, action: Action) -> Result<()> {
        let now = crate::model::now();
        match action {
            Action::Install {
                manifest,
                fingerprint,
            } => {
                let candidate = Candidate::new(manifest)?;
                ensure!(
                    candidate.fingerprint == fingerprint,
                    "The reviewed module definition changed"
                );
                let m = &candidate.metadata;
                if let Some(old) = self.records.get_mut(&m.id) {
                    let previous = old.metadata().context("Keep or export this module's data before replacing an unsupported definition")?;
                    ensure!(
                        semver::Version::parse(&m.version)?
                            >= semver::Version::parse(&previous.version)?,
                        "Module downgrades are blocked"
                    );
                    ensure!(
                        old.data_version == m.data_version,
                        "This update needs a data migration, which is not available yet"
                    );
                    if old.installed && m.version == previous.version {
                        ensure!(
                            fingerprint == old.fingerprint,
                            "A module version cannot be replaced with different contents"
                        );
                        if old.grants != m.permissions {
                            old.invalidate_sessions()?;
                            old.grants = m.permissions.clone();
                            old.enabled = false;
                            old.updated_at = now;
                        }
                        return Ok(());
                    }
                    if m.version == previous.version {
                        ensure!(
                            fingerprint == old.fingerprint,
                            "This version differs from the retained definition"
                        );
                    }
                    old.invalidate_sessions()?;
                    old.manifest = candidate.manifest;
                    old.fingerprint = fingerprint;
                    old.installed = true;
                    old.enabled = false;
                    old.grants = m.permissions.clone();
                    old.updated_at = now;
                } else {
                    self.records.insert(
                        m.id.clone(),
                        Record {
                            manifest: candidate.manifest,
                            fingerprint,
                            installed: true,
                            enabled: false,
                            grants: m.permissions.clone(),
                            data_version: m.data_version,
                            data: empty_data(),
                            generation: 0,
                            instance: uuid::Uuid::new_v4(),
                            installed_at: now,
                            updated_at: now,
                        },
                    );
                }
            }
            Action::Enable(id) => {
                let r = self.records.get_mut(&id).context("Module not found")?;
                ensure!(r.installed, "Install this module first");
                let m = r.metadata()?;
                m.compatibility()?;
                ensure!(
                    m.permissions == r.grants && m.data_version == r.data_version,
                    "Review this module's permissions and data version first"
                );
                if !r.enabled {
                    r.invalidate_sessions()?;
                }
                r.enabled = true;
                r.updated_at = now;
            }
            Action::Disable(id) => {
                let r = self.records.get_mut(&id).context("Module not found")?;
                if r.enabled {
                    r.invalidate_sessions()?;
                }
                r.enabled = false;
                r.updated_at = now;
            }
            Action::Uninstall(id) => {
                let r = self.records.get_mut(&id).context("Module not found")?;
                r.invalidate_sessions()?;
                r.installed = false;
                r.enabled = false;
                r.grants.clear();
                r.updated_at = now;
            }
            Action::Purge(id) => {
                ensure!(
                    !self.records.get(&id).context("Module not found")?.installed,
                    "Uninstall the module before deleting its data"
                );
                self.records.remove(&id);
            }
            Action::Catalog(catalog) => {
                catalog.validate()?;
                ensure!(
                    catalog.id != Catalog::built_in().id,
                    "The built-in catalog cannot be replaced"
                );
                self.catalogs.insert(catalog.id.clone(), catalog);
            }
            Action::RemoveCatalog(id) => {
                ensure!(self.catalogs.remove(&id).is_some(), "Catalog not found");
            }
        }
        Ok(())
    }
    pub fn session(&self, id: &str) -> Result<Session> {
        let record = self.records.get(id).context("Module not found")?;
        ensure!(
            record.active() && record.grants.iter().any(|p| p == "storage"),
            "Module storage access is not granted"
        );
        Ok(Session {
            id: id.into(),
            fingerprint: record.fingerprint.clone(),
            generation: record.generation,
            instance: record.instance,
            context: None,
        })
    }
    fn check_session(&self, session: &Session) -> Result<&Record> {
        let record = self.records.get(&session.id).context("Module not found")?;
        ensure!(
            record.active()
                && record.fingerprint == session.fingerprint
                && record.generation == session.generation
                && record.instance == session.instance
                && record.grants.iter().any(|p| p == "storage"),
            "Module access was revoked or this session is stale"
        );
        Ok(record)
    }
    pub fn read_data(&self, session: &Session) -> Result<Value> {
        Ok(self.check_session(session)?.data.clone())
    }
    pub fn write_data(&mut self, session: &Session, data: Value) -> Result<()> {
        self.check_session(session)?;
        ensure!(
            data.to_string().len() <= MAX_MODULE_DATA,
            "Module data exceeds 8 MiB"
        );
        let mut next = self.clone();
        let record = next
            .records
            .get_mut(&session.id)
            .context("Module not found")?;
        record.data = data;
        record.updated_at = crate::model::now();
        next.validate()?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone)]
pub enum Action {
    Install {
        manifest: Value,
        fingerprint: String,
    },
    Enable(String),
    Disable(String),
    Uninstall(String),
    Purge(String),
    Catalog(Catalog),
    RemoveCatalog(String),
}

/// Core-issued access to exactly one module namespace; revoked on lifecycle transitions.
#[derive(Clone, Debug)]
pub struct Session {
    id: String,
    fingerprint: String,
    generation: u64,
    instance: uuid::Uuid,
    pub(crate) context: Option<(uuid::Uuid, u64)>,
}
