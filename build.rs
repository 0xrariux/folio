use std::process::Command;
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=release.json");
    println!("cargo:rerun-if-changed=module-library.json");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=rust-toolchain.toml");
    let lock = std::fs::read_to_string("Cargo.lock").expect("committed Cargo.lock");
    let versions: Vec<_> = lock
        .split("[[package]]")
        .filter(|p| p.lines().any(|l| l.trim() == "name = \"turso\""))
        .filter_map(|p| {
            p.lines().find_map(|l| {
                l.trim()
                    .strip_prefix("version = \"")
                    .and_then(|v| v.strip_suffix('"'))
            })
        })
        .collect();
    assert_eq!(versions.len(), 1, "one resolved Turso engine is required");
    println!("cargo:rustc-env=FOLIO_TURSO_VERSION={}", versions[0]);
    // A worktree has a .git file, and its branch need not be main.
    let branch = Command::new("git")
        .args(["symbolic-ref", "--quiet", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
    for name in [
        Some("HEAD"),
        Some("index"),
        Some("packed-refs"),
        branch.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if let Ok(output) = Command::new("git")
            .args(["rev-parse", "--git-path", name])
            .output()
            && output.status.success()
        {
            let path = String::from_utf8_lossy(&output.stdout);
            let path = path.trim();
            if std::path::Path::new(path).is_file() {
                println!("cargo:rerun-if-changed={path}");
            }
        }
    }
    let rust = Command::new("rustc")
        .arg("--version")
        .output()
        .expect("Rust compiler available");
    println!(
        "cargo:rustc-env=FOLIO_RUST_VERSION={}",
        String::from_utf8_lossy(&rust.stdout).trim()
    );
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=FOLIO_BUILD_REVISION={revision}");
    let dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .is_none_or(|o| !o.status.success() || !o.stdout.is_empty());
    println!("cargo:rustc-env=FOLIO_BUILD_DIRTY={dirty}");
}
