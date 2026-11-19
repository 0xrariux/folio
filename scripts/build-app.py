#!/usr/bin/env python3
"""Build the native arm64 bundle; optionally prepare signed GitHub release assets."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def run(*args, **kwargs):
    return subprocess.run(args, cwd=ROOT, check=True, **kwargs)


def output(*args):
    return run(*args, stdout=subprocess.PIPE, text=True).stdout.strip()


def rune_checkout(metadata, manifest, pin):
    """Resolve the exact Cargo source, including vendor overrides, before packaging."""
    if not re.fullmatch(r"[0-9a-f]{40}", pin):
        raise ValueError("Rune requires a full reviewed Git revision")
    repository = "https://github.com/anomalyco/rune.git"
    specs = {name: manifest["dependencies"][name] for name in ["rune", "rune-kit"]}
    specs.update(manifest["patch"]["crates-io"])
    expected = f"git+{repository}?rev={pin}#{pin}"
    for name, spec in specs.items():
        if not isinstance(spec, dict) or spec.get("git") != repository or spec.get("rev") != pin:
            raise ValueError(f"{name} differs from the reviewed Rune pin")
        packages = [package for package in metadata["packages"] if package["name"] == name]
        if len(packages) != 1 or packages[0].get("source") != expected:
            raise ValueError(f"{name} did not resolve from the reviewed Rune source")
    for package in metadata["packages"]:
        if package["name"].startswith("rune-") and package.get("source") != expected:
            raise ValueError(f"{package['name']} did not resolve from the reviewed Rune source")
    package = next(p for p in metadata["packages"] if p["name"] == "rune")
    return Path(package["manifest_path"]).parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--development", action="store_true", help="Allow unpublished local changes; cannot produce release assets")
    parser.add_argument("--verify-storage", action="store_true", help="Run the frozen storage corpus in the production release profile (required for publication)")
    parser.add_argument("--release", action="store_true", help="Create ZIP and signed release manifest")
    parser.add_argument("--key", type=Path, help="Private PKCS#8 signing key (required for --release)")
    args = parser.parse_args()
    if platform.system() != "Darwin" or platform.machine() != "arm64":
        parser.error("Build on an Apple Silicon Mac")
    if args.release and (args.development or args.key is None):
        parser.error("--release requires --key and a clean, committed checkout")
    cfg = json.loads((ROOT / "release.json").read_text())
    pin = json.loads((ROOT / "folio-update.json").read_text())["rune_revision"]
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text())
    metadata = json.loads(output("cargo", "metadata", "--locked", "--format-version", "1"))
    try:
        rune = rune_checkout(metadata, manifest, pin)
    except ValueError as error:
        parser.error(str(error))
    if output("git", "-C", str(rune), "rev-parse", "HEAD") != pin:
        parser.error("Cargo's Rune checkout differs from folio-update.json")
    if output("git", "-C", str(rune), "status", "--porcelain", "--untracked-files=no"):
        parser.error("Cargo's Rune source has local changes; restore a clean dependency cache")
    if not args.development and output("git", "status", "--porcelain"):
        parser.error("Commit the reviewed changes first, or use --development for a local app")
    if not cfg["public_key"]:
        parser.error("Configure release.json's public key before building")
    env = os.environ.copy()
    env["MACOSX_DEPLOYMENT_TARGET"] = cfg["minimum_macos"]
    flags = env.get("CARGO_ENCODED_RUSTFLAGS", "").split("\x1f") if env.get("CARGO_ENCODED_RUSTFLAGS") else env.get("RUSTFLAGS", "").split()
    flags.extend([f"--remap-path-prefix={ROOT}=/build/folio", f"--remap-path-prefix={rune}=/build/rune", f"--remap-path-prefix={Path.home()}=/build/host"])
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    env.pop("RUSTFLAGS", None)
    if args.release or args.verify_storage:
        run("cargo", "test", "--release", "--locked", "--test", "data_safety", "--test", "modules", env=env)
    run("cargo", "build", "--release", "--locked", "--bin", "folio", env=env)
    # Respect CARGO_TARGET_DIR (CI and isolated worktrees may relocate it).
    target = Path(metadata["target_directory"]) / "release"
    binary = target / "folio"
    build = json.loads(output(str(binary), "--release-info"))
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    if build["version"] != version or build["revision"] != output("git", "rev-parse", "HEAD"):
        parser.error("Stale executable provenance")
    if not args.development and build["dirty"]:
        parser.error("Executable was built with local changes; rebuild after committing")
    build["bundle_id"] = cfg["bundle_id"]
    dist = ROOT / "dist"
    dist.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".folio-package-", dir=dist) as temporary:
        app = Path(temporary) / "Folio.app"
        contents = app / "Contents"
        resources = contents / "Resources"
        (contents / "MacOS").mkdir(parents=True)
        resources.mkdir()
        shutil.copy2(binary, contents / "MacOS/folio")
        info = {
            "CFBundleIdentifier": cfg["bundle_id"], "CFBundleName": "Folio",
            "CFBundleDisplayName": "Folio", "CFBundleExecutable": "folio",
            "CFBundlePackageType": "APPL", "CFBundleShortVersionString": version,
            "CFBundleVersion": version, "LSMinimumSystemVersion": cfg["minimum_macos"],
            "LSApplicationCategoryType": "public.app-category.productivity",
            "NSHighResolutionCapable": True, "CFBundleIconFile": "Folio.icns",
            "NSHumanReadableCopyright": "Folio contributors. MIT License.",
        }
        (contents / "Info.plist").write_bytes(plistlib.dumps(info))
        (resources / "build-info.json").write_text(json.dumps(build, indent=2) + "\n")
        shutil.copy2(ROOT / "LICENSE", resources / "LICENSE")
        # Use the existing brand artwork at native macOS icon sizes.
        iconset = Path(temporary) / "Folio.iconset"
        iconset.mkdir()
        for size in [16, 32, 128, 256, 512]:
            for scale in [1, 2]:
                suffix = "@2x" if scale == 2 else ""
                run("/usr/bin/sips", "-z", str(size * scale), str(size * scale),
                    str(ROOT / "assets/brand/folio-logo.png"), "--out",
                    str(iconset / f"icon_{size}x{size}{suffix}.png"), stdout=subprocess.DEVNULL)
        run("/usr/bin/iconutil", "-c", "icns", str(iconset), "-o", str(resources / "Folio.icns"))
        run("/usr/bin/codesign", "--force", "--sign", "-", "--timestamp=none", str(app))
        run("/usr/bin/codesign", "--verify", "--deep", "--strict", str(app))
        destination = dist / "Folio.app"
        # Preserve an existing bundle until packaging/signing has succeeded.
        previous = dist / ".Folio.previous.app"
        if previous.exists():
            shutil.rmtree(previous)
        if destination.exists():
            destination.rename(previous)
        try:
            app.rename(destination)
        except OSError:
            if previous.exists():
                previous.rename(destination)
            raise
        print(f"Application ready: {destination}")
    if not args.release:
        return
    run("cargo", "build", "--locked", "--release", "--bin", "folio-release", env=env)
    artifacts = ROOT / "release-artifacts"
    artifacts.mkdir(exist_ok=True)
    archive = artifacts / f"Folio-{version}-macos-arm64.zip"
    if archive.exists():
        archive.unlink()
    run("/usr/bin/ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(dist / "Folio.app"), str(archive))
    manifest = {
        "schema": 1, "version": version, "revision": build["revision"],
        "target": cfg["target"], "bundle_id": cfg["bundle_id"],
        "minimum_macos": cfg["minimum_macos"], "storage_contract": cfg["storage_contract"],
        "archive": archive.name, "size": archive.stat().st_size,
        "sha256": hashlib.file_digest(archive.open("rb"), "sha256").hexdigest(),
    }
    manifest_path = artifacts / "folio-release.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")
    run(str(target / "folio-release"), "sign", str(args.key.resolve()), str(manifest_path), str(artifacts / "folio-release.sig"))
    run(str(target / "folio-release"), "verify", str(manifest_path), str(artifacts / "folio-release.sig"), str(archive))
    print("Signed release assets prepared. Nothing has been uploaded or published.")


if __name__ == "__main__":
    main()
