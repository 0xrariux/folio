//! Offline release signing utility. Private key bytes are never printed.
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, KeyPair},
};
use std::{fs, io::Write, path::Path};
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("keygen") if args.len() == 4 => {
            let directory = Path::new(&args[2]);
            ensure!(
                !directory.exists(),
                "Signing directory already exists; refusing to overwrite it"
            );
            fs::create_dir(directory)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
            }
            let document = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
                .map_err(|_| anyhow::anyhow!("Cannot generate signing key"))?;
            let key = Ed25519KeyPair::from_pkcs8(document.as_ref())
                .map_err(|_| anyhow::anyhow!("Cannot load signing key"))?;
            let mut options = fs::File::options();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            options
                .open(directory.join("private.pk8"))?
                .write_all(document.as_ref())?;
            let config_path = Path::new(&args[3]);
            let mut cfg: folio::updates::Config = serde_json::from_slice(&fs::read(config_path)?)?;
            ensure!(
                cfg.public_key.is_empty(),
                "Public key is already configured; use an explicit rotation procedure"
            );
            cfg.public_key = STANDARD.encode(key.public_key().as_ref());
            fs::write(
                config_path,
                format!("{}\n", serde_json::to_string_pretty(&cfg)?),
            )?;
            println!(
                "Signing key created. Back up the private directory securely; commit only release.json."
            );
        }
        Some("sign") if args.len() == 5 => {
            let secret = zeroize::Zeroizing::new(
                fs::read(&args[2]).context("Cannot read private signing key")?,
            );
            let key = Ed25519KeyPair::from_pkcs8(&secret)
                .map_err(|_| anyhow::anyhow!("Invalid private signing key"))?;
            ensure!(
                STANDARD.encode(key.public_key().as_ref()) == folio::updates::config().public_key,
                "Private signing key does not match the application's embedded public key"
            );
            let bytes = fs::read(&args[3])?;
            let sig = STANDARD.encode(key.sign(&bytes).as_ref());
            folio::updates::verify(&bytes, &sig, &folio::updates::config())?;
            fs::write(&args[4], format!("{sig}\n"))?;
            println!("Release manifest signed.");
        }
        Some("verify") if args.len() == 5 => {
            let release = folio::updates::verify(
                &fs::read(&args[2])?,
                &fs::read_to_string(&args[3])?,
                &folio::updates::config(),
            )?;
            folio::updates::validate_release_archive(Path::new(&args[4]), &release)?;
            println!("Signature, archive and application bundle verified.");
        }
        _ => anyhow::bail!(
            "Usage: folio-release keygen PRIVATE_DIRECTORY release.json | sign PRIVATE_KEY MANIFEST SIGNATURE | verify MANIFEST SIGNATURE ARCHIVE"
        ),
    }
    Ok(())
}
