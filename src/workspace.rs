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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SidebarShortcut {
    Overview,
    AllPages,
    Favorites,
    Archives,
}
impl SidebarShortcut {
    pub const ALL: [Self; 4] = [
        Self::Overview,
        Self::AllPages,
        Self::Favorites,
        Self::Archives,
    ];

    pub fn defaults() -> Vec<Self> {
        Self::ALL.to_vec()
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|shortcut| shortcut.key() == key)
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::AllPages => "all_pages",
            Self::Favorites => "favorites",
            Self::Archives => "archives",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::AllPages => "All pages",
            Self::Favorites => "Favorites",
            Self::Archives => "Archives",
        }
    }
}

fn deserialize_shortcuts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<SidebarShortcut>, D::Error> {
    let names = Vec::<String>::deserialize(deserializer)?;
    let mut shortcuts = Vec::new();
    for name in names {
        let shortcut = SidebarShortcut::from_key(&name).ok_or_else(|| {
            serde::de::Error::custom("This shortcut requires a newer version of Folio")
        })?;
        if !shortcuts.contains(&shortcut) {
            shortcuts.push(shortcut);
        }
    }
    Ok(shortcuts)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DarkPalette {
    #[default]
    Aurum,
    Sage,
    Nord,
    Mocha,
    RosePine,
    Gruvbox,
}
impl DarkPalette {
    pub const ALL: [Self; 6] = [
        Self::Aurum,
        Self::Sage,
        Self::Nord,
        Self::Mocha,
        Self::RosePine,
        Self::Gruvbox,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::Aurum => "aurum",
            Self::Sage => "sage",
            Self::Nord => "nord",
            Self::Mocha => "mocha",
            Self::RosePine => "rose_pine",
            Self::Gruvbox => "gruvbox",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Aurum => "Aurum",
            Self::Sage => "Sage",
            Self::Nord => "Nord",
            Self::Mocha => "Catppuccin Mocha",
            Self::RosePine => "Rosé Pine",
            Self::Gruvbox => "Gruvbox Material",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Aurum => "Burnt orange, terracotta & ivory",
            Self::Sage => "Soft greens & charcoal",
            Self::Nord => "Arctic slate & frost blue",
            Self::Mocha => "Deep navy & pastel lavender",
            Self::RosePine => "Midnight plum & dusty rose",
            Self::Gruvbox => "Earthy charcoal, amber & olive",
        }
    }
    fn is_default(&self) -> bool {
        *self == Self::Aurum
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LightPalette {
    #[default]
    Original,
    Vanilla,
}
impl LightPalette {
    pub const ALL: [Self; 2] = [Self::Original, Self::Vanilla];
    pub fn key(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Vanilla => "vanilla",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Vanilla => "Vanille",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Original => "Original · Linen & soft greens",
            Self::Vanilla => "Pastel yellow, warm cream & honey",
        }
    }
    fn is_default(&self) -> bool {
        *self == Self::Original
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub dark: bool,
    #[serde(default, skip_serializing_if = "DarkPalette::is_default")]
    pub dark_palette: DarkPalette,
    #[serde(default, skip_serializing_if = "LightPalette::is_default")]
    pub light_palette: LightPalette,
    pub sidebar_width: u16,
    pub last_page: Option<String>,
    #[serde(
        default = "SidebarShortcut::defaults",
        deserialize_with = "deserialize_shortcuts"
    )]
    pub sidebar_shortcuts: Vec<SidebarShortcut>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark: true,
            dark_palette: DarkPalette::default(),
            light_palette: LightPalette::default(),
            sidebar_width: 248,
            last_page: None,
            sidebar_shortcuts: SidebarShortcut::defaults(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct EmbeddedView {
    pub base_id: String,
    pub view_id: Option<String>,
    /// A relation column in the source, filtered to the page hosting this block.
    pub relation_filter: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryEntry {
    pub id: String,
    pub page_id: String,
    pub created_at: i64,
    pub page: Page,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Notebook {
    pub format: u32,
    pub document_version: u32,
    pub pages: Vec<Page>,
    pub preferences: Preferences,
    pub history: Vec<HistoryEntry>,
    pub assets: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "crate::modules::Registry::is_empty")]
    pub modules: crate::modules::Registry,
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
            document_version: crate::document::CURRENT_VERSION,
            pages,
            preferences,
            history,
            assets: BTreeMap::new(),
            modules: Default::default(),
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
                    "Attachments exceed the backup size limit (256 MiB)."
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
            (self.format == 1 || self.format == 2)
                && (self.format == 2 || self.modules.is_empty())
                && matches!(self.document_version, 1..=crate::document::CURRENT_VERSION),
            "This backup requires a newer version of Folio."
        );
        ensure!(
            self.pages.len() <= 100_000 && self.history.len() <= 100_000,
            "Backup too large."
        );
        self.modules.validate()?;
        validate_structure(&self.pages, &self.history)?;
        for page in self
            .pages
            .iter()
            .chain(self.history.iter().map(|h| &h.page))
        {
            for asset in page.blocks.iter().filter_map(|b| b.asset.as_ref()) {
                let encoded = self
                    .assets
                    .get(&asset.id)
                    .context("Missing attachment data")?;
                ensure!(
                    STANDARD.decode(encoded)?.len() as u64 == asset.size,
                    "Attachment size does not match its metadata"
                );
            }
        }
        let mut total = 0;
        for (id, encoded) in &self.assets {
            ensure!(uuid::Uuid::parse_str(id).is_ok(), "Invalid attachment ID.");
            ensure!(
                encoded.len() as u64 <= MAX_FILE * 4 / 3 + 8,
                "Attachment too large."
            );
            let bytes = STANDARD.decode(encoded).context("Corrupted attachment")?;
            ensure!(bytes.len() as u64 <= MAX_FILE, "Attachment too large.");
            total += bytes.len() as u64;
            ensure!(total <= MAX_BACKUP / 2, "Backup too large.");
        }
        Ok(())
    }
    pub fn read(path: &Path) -> Result<Self> {
        let bundle: Self = serde_json::from_slice(&read_limited(path, MAX_BACKUP)?)
            .context("Reading the Folio backup")?;
        bundle.validate()?;
        Ok(bundle)
    }
    pub fn write(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let encoded = serde_json::to_string(self)?;
        ensure!(
            encoded.len() as u64 <= MAX_BACKUP,
            "The backup exceeds the supported size"
        );
        crate::export::write_atomic(path, &encoded)
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
                    "An existing attachment differs from the backup."
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
    ensure!(uuid::Uuid::parse_str(id).is_ok(), "Invalid attachment ID.");
    Ok(folder.join(id))
}
pub fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).with_context(|| format!("Opening {}", path.display()))?;
    ensure!(
        file.metadata()?.is_file() && file.metadata()?.len() <= limit,
        "File too large (maximum {} MiB).",
        limit / 1024 / 1024
    );
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "File too large.");
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

/// Shared structural validation for live storage and portable backups. Never repairs data.
pub fn validate_structure(pages: &[Page], history: &[HistoryEntry]) -> Result<()> {
    let mut ids = HashSet::new();
    for page in pages {
        ensure!(
            !page.id.is_empty() && ids.insert(page.id.as_str()),
            "Invalid or duplicate page IDs"
        );
    }
    let mut revisions = HashSet::new();
    for entry in history {
        ensure!(
            !entry.id.is_empty() && revisions.insert(&entry.id),
            "Invalid or duplicate history IDs"
        );
        ensure!(
            entry.page_id == entry.page.id && ids.contains(entry.page_id.as_str()),
            "Orphaned or mismatched history entry"
        );
    }
    for page in pages.iter().chain(history.iter().map(|h| &h.page)) {
        validate_page(page)?;
    }
    let parents: std::collections::HashMap<_, _> = pages
        .iter()
        .map(|p| (p.id.as_str(), p.parent_id.as_deref()))
        .collect();
    // Memoize completed paths: deep notebooks are checked in linear time.
    let mut complete = HashSet::new();
    for page in pages {
        let mut chain = HashSet::new();
        let mut at = Some(page.id.as_str());
        while let Some(id) = at {
            if complete.contains(id) {
                break;
            }
            ensure!(chain.insert(id), "Cycle in the page hierarchy");
            at = *parents.get(id).context("Missing parent page")?;
        }
        complete.extend(chain);
    }
    Ok(())
}

pub fn validate_page(page: &Page) -> Result<()> {
    ensure!(!page.id.is_empty(), "Empty page identity");
    ensure!(!page.blocks.is_empty(), "A page contains no blocks");
    let mut blocks = HashSet::new();
    for block in &page.blocks {
        ensure!(
            !block.id.is_empty() && blocks.insert(&block.id),
            "Duplicate or empty block IDs"
        );
        ensure!(block.indent <= 6, "Invalid block depth");
        if let Some(asset) = &block.asset {
            ensure!(
                uuid::Uuid::parse_str(&asset.id).is_ok(),
                "Invalid attachment ID"
            );
            ensure!(
                asset.size <= MAX_FILE,
                "Attachment exceeds the supported size"
            );
        }
    }
    Ok(())
}
