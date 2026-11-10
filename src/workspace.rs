//! Portable notebook files, managed assets and persistent application preferences.
use crate::model::Page;
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    io::Read,
    path::{Path, PathBuf},
};

pub const MAX_FILE: u64 = 50 * 1024 * 1024;
pub const MAX_BACKUP: u64 = 512 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    pub dark: bool,
    pub sidebar_width: u16,
    pub last_page: Option<String>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark: true,
            sidebar_width: 248,
            last_page: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub id: String,
    pub name: String,
    pub mime: String,
    pub size: u64,
    #[serde(default = "full_width")]
    pub width_percent: u16,
}
fn full_width() -> u16 {
    100
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddedView {
    pub base_id: String,
    pub view_id: Option<String>,
    /// A relation column in the source, filtered to the page hosting this block.
    pub relation_filter: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub page_id: String,
    pub created_at: i64,
    pub page: Page,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notebook {
    pub format: u32,
    pub document_version: u32,
    pub pages: Vec<Page>,
    pub preferences: Preferences,
    pub history: Vec<HistoryEntry>,
    pub assets: BTreeMap<String, String>,
}
impl Notebook {
    pub fn capture(
        pages: Vec<Page>,
        preferences: Preferences,
        history: Vec<HistoryEntry>,
        assets: &Path,
    ) -> Result<Self> {
        let mut bundle = Self {
            format: 1,
            document_version: 6,
            pages,
            preferences,
            history,
            assets: BTreeMap::new(),
        };
        let mut total = 0;
        for page in bundle
            .pages
            .iter()
            .chain(bundle.history.iter().map(|h| &h.page))
        {
            for asset in page.blocks.iter().filter_map(|b| b.asset.as_ref()) {
                if bundle.assets.contains_key(&asset.id) {
                    continue;
                }
                let path = asset_path(assets, &asset.id)?;
                let bytes = read_limited(&path, MAX_FILE)?;
                total += bytes.len() as u64;
                ensure!(
                    total <= MAX_BACKUP / 2,
                    "Les pièces jointes dépassent la taille d’une sauvegarde (256 Mio)."
                );
                bundle
                    .assets
                    .insert(asset.id.clone(), STANDARD.encode(bytes));
            }
        }
        bundle.validate()?;
        Ok(bundle)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == 1 && matches!(self.document_version, 1..=6),
            "Cette sauvegarde nécessite une version plus récente de Folio."
        );
        ensure!(
            self.pages.len() <= 100_000 && self.history.len() <= 100_000,
            "Sauvegarde trop volumineuse."
        );
        let mut ids = HashSet::new();
        for page in &self.pages {
            ensure!(
                !page.id.is_empty() && ids.insert(&page.id),
                "Identifiants de pages invalides ou dupliqués."
            );
        }
        for page in self
            .pages
            .iter()
            .chain(self.history.iter().map(|h| &h.page))
        {
            ensure!(!page.blocks.is_empty(), "Une page ne contient aucun bloc.");
            let mut blocks = HashSet::new();
            for b in &page.blocks {
                ensure!(
                    blocks.insert(&b.id) && !b.id.is_empty(),
                    "Identifiants de blocs dupliqués ou vides."
                );
                ensure!(b.indent <= 6, "Profondeur de bloc invalide.");
                if let Some(asset) = &b.asset {
                    ensure!(
                        uuid::Uuid::parse_str(&asset.id).is_ok(),
                        "Identifiant de pièce jointe invalide."
                    );
                    ensure!(
                        self.assets.contains_key(&asset.id),
                        "Pièce jointe manquante : {}",
                        asset.name
                    );
                }
            }
        }
        for page in &self.pages {
            let mut seen = HashSet::new();
            let mut at = Some(page.id.as_str());
            while let Some(id) = at {
                ensure!(seen.insert(id), "Cycle dans l’arborescence des pages.");
                at = self
                    .pages
                    .iter()
                    .find(|p| p.id == id)
                    .and_then(|p| p.parent_id.as_deref());
            }
        }
        let mut total = 0;
        for (id, encoded) in &self.assets {
            ensure!(
                uuid::Uuid::parse_str(id).is_ok(),
                "Identifiant de pièce jointe invalide."
            );
            ensure!(
                encoded.len() as u64 <= MAX_FILE * 4 / 3 + 8,
                "Pièce jointe trop volumineuse."
            );
            let bytes = STANDARD.decode(encoded).context("Pièce jointe corrompue")?;
            ensure!(
                bytes.len() as u64 <= MAX_FILE,
                "Pièce jointe trop volumineuse."
            );
            total += bytes.len() as u64;
            ensure!(total <= MAX_BACKUP / 2, "Sauvegarde trop volumineuse.");
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self> {
        let bundle: Self = serde_json::from_slice(&read_limited(path, MAX_BACKUP)?)
            .context("Lecture de la sauvegarde Folio")?;
        bundle.validate()?;
        Ok(bundle)
    }
    pub fn write(&self, path: &Path) -> Result<()> {
        self.validate()?;
        crate::export::write_atomic(path, &serde_json::to_string(self)?)
    }
    pub fn install_assets(&self, folder: &Path) -> Result<()> {
        self.validate()?;
        std::fs::create_dir_all(folder)?;
        for (id, encoded) in &self.assets {
            let path = asset_path(folder, id)?;
            let bytes = STANDARD.decode(encoded)?;
            if path.exists() {
                ensure!(
                    read_limited(&path, MAX_FILE)? == bytes,
                    "Une pièce jointe existante diffère de la sauvegarde."
                );
            } else {
                use std::io::Write;
                let mut temp = tempfile::NamedTempFile::new_in(folder)?;
                temp.write_all(&bytes)?;
                temp.as_file().sync_all()?;
                temp.persist_noclobber(path).map_err(|e| e.error)?;
            }
        }
        Ok(())
    }
}
pub fn asset_dir(database: &Path) -> PathBuf {
    database.with_extension("assets")
}
pub fn asset_path(folder: &Path, id: &str) -> Result<PathBuf> {
    ensure!(
        uuid::Uuid::parse_str(id).is_ok(),
        "Identifiant de pièce jointe invalide."
    );
    Ok(folder.join(id))
}
pub fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file =
        std::fs::File::open(path).with_context(|| format!("Ouverture de {}", path.display()))?;
    ensure!(
        file.metadata()?.is_file() && file.metadata()?.len() <= limit,
        "Fichier trop volumineux (maximum {} Mio).",
        limit / 1024 / 1024
    );
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "Fichier trop volumineux.");
    Ok(bytes)
}
pub fn import_asset(source: &Path, folder: &Path) -> Result<Asset> {
    let bytes = read_limited(source, MAX_FILE)?;
    std::fs::create_dir_all(folder)?;
    let id = uuid::Uuid::new_v4().to_string();
    let path = asset_path(folder, &id)?;
    use std::io::Write;
    let mut file = tempfile::NamedTempFile::new_in(folder)?;
    file.write_all(&bytes)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path).map_err(|e| e.error)?;
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    };
    Ok(Asset {
        id,
        name: source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into(),
        mime: mime.into(),
        size: bytes.len() as u64,
        width_percent: 100,
    })
}

pub fn asset_export_name(asset: &Asset) -> String {
    let ext = Path::new(&asset.name)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty() && e.len() <= 12 && e.bytes().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("bin");
    format!("{}.{}", asset.id, ext.to_ascii_lowercase())
}
pub fn export_assets(assets: &[Asset], source: &Path, destination: &Path) -> Result<()> {
    use std::io::Write;
    if assets.is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(destination)?;
    for asset in assets {
        let bytes = read_limited(&asset_path(source, &asset.id)?, MAX_FILE)?;
        let mut temp = tempfile::NamedTempFile::new_in(destination)?;
        temp.write_all(&bytes)?;
        temp.as_file().sync_all()?;
        temp.persist(destination.join(asset_export_name(asset)))
            .map_err(|e| e.error)?;
    }
    Ok(())
}
