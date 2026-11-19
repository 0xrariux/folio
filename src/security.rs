//! Versioned, authenticated encryption. No passwords or derived keys are serialized.
use anyhow::{Context, Result, ensure};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use zeroize::Zeroizing;

pub const FORMAT_VERSION: u32 = 1;

pub type Key = Arc<Zeroizing<[u8; 32]>>;
#[derive(Clone)]
pub struct PageKey {
    pub key: Key,
    pub salt: [u8; 16],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sealed {
    pub version: u32,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VaultConfig {
    pub version: u32,
    pub verifier: Sealed,
}
const VAULT_AAD: &[u8] = b"Folio vault key v1";
const BACKUP_AAD: &[u8] = b"Folio encrypted backup v1";
pub fn config_path(database: &Path) -> PathBuf {
    database.with_extension("security.json")
}
pub fn encrypted_path(database: &Path) -> PathBuf {
    database.with_extension("encrypted.db")
}
pub fn random_salt() -> [u8; 16] {
    let mut salt = [0; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}
pub fn derive(password: &str, salt: &[u8; 16]) -> Result<Key> {
    let params = Params::new(65536, 3, 1, Some(32)).map_err(|e| anyhow::anyhow!("Argon2: {e}"))?;
    let mut key = Zeroizing::new([0; 32]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut *key)
        .map_err(|e| anyhow::anyhow!("Key derivation: {e}"))?;
    Ok(Arc::new(key))
}
pub fn seal(bytes: &[u8], key: &Key, salt: &[u8; 16], aad: &[u8]) -> Result<Sealed> {
    let mut nonce = [0; 24];
    OsRng.fill_bytes(&mut nonce);
    let ciphertext = XChaCha20Poly1305::new_from_slice(&***key)
        .unwrap()
        .encrypt(XNonce::from_slice(&nonce), Payload { msg: bytes, aad })
        .map_err(|_| anyhow::anyhow!("Encryption failed"))?;
    Ok(Sealed {
        version: FORMAT_VERSION,
        salt: B64.encode(salt),
        nonce: B64.encode(nonce),
        ciphertext: B64.encode(ciphertext),
    })
}
pub fn open(sealed: &Sealed, key: &Key, aad: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    ensure!(
        sealed.version == FORMAT_VERSION,
        "Unsupported encryption format"
    );
    let nonce: [u8; 24] = B64
        .decode(&sealed.nonce)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid nonce"))?;
    let ciphertext = B64.decode(&sealed.ciphertext)?;
    XChaCha20Poly1305::new_from_slice(&***key)
        .unwrap()
        .decrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: &ciphertext,
                aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| anyhow::anyhow!("Incorrect code or damaged data"))
}
pub fn salt(sealed: &Sealed) -> Result<[u8; 16]> {
    B64.decode(&sealed.salt)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid encryption salt"))
}
impl VaultConfig {
    pub fn create(password: &str) -> Result<(Self, Key)> {
        ensure!(
            password.chars().count() >= 12,
            "Choose a master password of at least 12 characters."
        );
        let salt = random_salt();
        let key = derive(password, &salt)?;
        let verifier = seal(b"Folio vault v1", &key, &salt, VAULT_AAD)?;
        Ok((
            Self {
                version: FORMAT_VERSION,
                verifier,
            },
            key,
        ))
    }
    pub fn unlock(&self, password: &str) -> Result<Key> {
        ensure!(self.version == FORMAT_VERSION, "Unsupported vault version");
        let key = derive(password, &salt(&self.verifier)?)?;
        ensure!(
            &*open(&self.verifier, &key, VAULT_AAD)? == b"Folio vault v1",
            "Incorrect password"
        );
        Ok(key)
    }
    pub fn read(database: &Path) -> Result<Option<Self>> {
        match std::fs::read(config_path(database)) {
            Ok(bytes) => Ok(Some(
                serde_json::from_slice(&bytes).context("Invalid vault configuration")?,
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    pub fn write(&self, database: &Path) -> Result<()> {
        crate::export::write_atomic(&config_path(database), &serde_json::to_string(self)?)
    }
    pub fn encryption_options(&self, key: &Key) -> turso::EncryptionOpts {
        turso::EncryptionOpts {
            cipher: "aes256gcm".into(),
            hexkey: key.iter().map(|v| format!("{v:02x}")).collect(),
        }
    }
}
pub fn new_page_key(pin: &str) -> Result<PageKey> {
    ensure!(
        pin.len() >= 6 && pin.len() <= 128 && pin.chars().all(|c| c.is_ascii_digit()),
        "The PIN must contain 6 to 128 digits."
    );
    let salt = random_salt();
    Ok(PageKey {
        key: derive(pin, &salt)?,
        salt,
    })
}
pub fn seal_page(page: &crate::model::Page, keys: &PageKey) -> Result<crate::model::Page> {
    ensure!(page.protected.is_none(), "The page is already locked");
    ensure!(
        page.blocks.iter().all(|b| b.asset.is_none()),
        "Move attachments out of this page before protecting it."
    );
    let bytes = Zeroizing::new(serde_json::to_vec(page)?);
    let aad = format!("Folio protected page v1:{}", page.id);
    let sealed = seal(&bytes, &keys.key, &keys.salt, aad.as_bytes())?;
    let mut placeholder = crate::model::Page::new(page.parent_id.clone());
    placeholder.id = page.id.clone();
    placeholder.title = "Protected page".into();
    placeholder.archived = page.archived;
    placeholder.order = page.order;
    placeholder.favorite = page.favorite;
    placeholder.updated_at = page.updated_at;
    placeholder.protected = Some(sealed);
    Ok(placeholder)
}
pub fn unlock_page(page: &crate::model::Page, pin: &str) -> Result<(crate::model::Page, PageKey)> {
    let sealed = page
        .protected
        .as_ref()
        .context("This page is not protected")?;
    let salt = salt(sealed)?;
    let key = derive(pin, &salt)?;
    let keys = PageKey { key, salt };
    let plain = open_page(page, &keys)?;
    Ok((plain, keys))
}
pub fn open_page(page: &crate::model::Page, keys: &PageKey) -> Result<crate::model::Page> {
    let aad = format!("Folio protected page v1:{}", page.id);
    let bytes = open(
        page.protected.as_ref().context("Page is not encrypted")?,
        &keys.key,
        aad.as_bytes(),
    )?;
    let mut plain: crate::model::Page = serde_json::from_slice(&bytes)?;
    ensure!(
        plain.id == page.id && plain.protected.is_none(),
        "Invalid encrypted page identity"
    );
    plain.archived = page.archived;
    plain.order = page.order;
    plain.parent_id = page.parent_id.clone();
    plain.favorite = page.favorite;
    Ok(plain)
}
pub fn write_backup(
    notebook: &crate::workspace::Notebook,
    path: &Path,
    config: &VaultConfig,
    key: &Key,
) -> Result<()> {
    notebook.validate()?;
    let bytes = Zeroizing::new(serde_json::to_vec(notebook)?);
    let sealed = seal(&bytes, key, &salt(&config.verifier)?, BACKUP_AAD)?;
    let encoded = serde_json::to_string(&sealed)?;
    ensure!(
        encoded.len() as u64 <= crate::workspace::MAX_BACKUP,
        "The encrypted backup exceeds 512 MiB."
    );
    crate::export::write_atomic(path, &encoded)
}
pub fn read_backup(
    path: &Path,
    key: Option<&Key>,
    password: Option<&str>,
) -> Result<crate::workspace::Notebook> {
    let bytes = crate::workspace::read_limited(path, crate::workspace::MAX_BACKUP)?;
    if let Ok(sealed) = serde_json::from_slice::<Sealed>(&bytes) {
        let derived;
        let key = if let Some(password) = password {
            derived = derive(password, &salt(&sealed)?)?;
            &derived
        } else {
            key.context("This backup is encrypted: its master password is required.")?
        };
        let plain = open(&sealed, key, BACKUP_AAD)?;
        let notebook: crate::workspace::Notebook = serde_json::from_slice(&plain)?;
        notebook.validate()?;
        Ok(notebook)
    } else {
        let notebook: crate::workspace::Notebook = serde_json::from_slice(&bytes)?;
        notebook.validate()?;
        Ok(notebook)
    }
}

/// Include both the plain and encrypted files, configuration and lock in write guards.
pub fn storage_files(database: &Path) -> Vec<PathBuf> {
    let mut files = vec![config_path(database), database.with_extension("lock")];
    for path in [database.to_path_buf(), encrypted_path(database)] {
        files.push(PathBuf::from(format!("{}-wal", path.display())));
        files.push(PathBuf::from(format!("{}-shm", path.display())));
        files.push(path);
    }
    files
}
