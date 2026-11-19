#!/usr/bin/env python3
"""Exercise a signed update against a temporary app and fresh profile, never your notebook.
Requires a built older app and signed newer release assets using the same trust anchor.
Runs native windows briefly; it never publishes or mutates the supplied app.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time


def wait_for(predicate, seconds=30):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        result = predicate()
        if result:
            return result
        time.sleep(.1)
    raise RuntimeError("Timed out waiting for the temporary app")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--app", type=Path, required=True)
    parser.add_argument("--release", type=Path, required=True)
    parser.add_argument("--damage-backup", action="store_true", help="Verify that a corrupted backup rejects installation and keeps the old bundle")
    args = parser.parse_args()
    base = Path(tempfile.mkdtemp(prefix="folio-install-smoke-")).resolve()
    target = base / "Folio.app"
    subprocess.run(["/usr/bin/ditto", str(args.app.resolve()), str(target)], check=True)
    profile = base / "profile.db"
    health = base / "initial-health.json"
    env = os.environ.copy()
    env.pop("FOLIO_DEMO", None)
    env.pop("FOLIO_UPDATE_ERROR", None)
    env.update(FOLIO_DB_PATH=str(profile), FOLIO_UPDATE_HEALTH=str(health))
    descendants = []
    original_build = json.loads(subprocess.check_output([str(target / "Contents/MacOS/folio"), "--release-info"], text=True))
    try:
        with (base / "initial.log").open("w") as log:
            old = subprocess.Popen([str(target / "Contents/MacOS/folio")], env=env, stdout=log, stderr=log)
            try:
                wait_for(lambda: health.exists() or old.poll() is not None)
                assert health.exists(), "Older app did not acknowledge startup"
                wait_for(lambda: list(profile.with_suffix(".backups").glob("*.folio")))
            finally:
                if old.poll() is None:
                    old.terminate()
                old.wait(timeout=10)
        backups = sorted(profile.with_suffix(".backups").glob("*.folio"))
        previous_notebook = json.loads(backups[-1].read_text())
        stage = profile.with_suffix(".updates") / "release-test"
        stage.mkdir(parents=True)
        manifest = json.loads((args.release / "folio-release.json").read_text())
        for name in ["folio-release.json", "folio-release.sig", manifest["archive"]]:
            shutil.copy2(args.release / name, stage / name)
        backup = stage / "before-update.folio"
        shutil.copy2(backups[-1], backup)
        subprocess.run([str(target / "Contents/MacOS/folio"), "--verify-backup", str(backup)], check=True)
        if args.damage_backup:
            with backup.open("ab") as damaged:
                damaged.write(b"corrupted after verification")
        helper = stage / "folio-installer"
        shutil.copy2(target / "Contents/MacOS/folio", helper)
        plan = stage / "install.json"
        plan.write_text(json.dumps({"target": str(target), "parent_pid": old.pid, "backup": str(backup), "database": str(profile)}))
        with (base / "installer.log").open("w") as log:
            result = subprocess.run([str(helper), "--apply-update", str(plan)], env=env, stdout=log, stderr=log, timeout=120)
        receipts = list(stage.glob("health-*.json"))
        for receipt in receipts:
            descendants.append(json.loads(receipt.read_text())["pid"])
        if args.damage_backup:
            assert result.returncode != 0, "A corrupted backup was accepted"
            failure = json.loads((stage / "failed.json").read_text())
            if failure.get("restarted_pid"):
                descendants.append(failure["restarted_pid"])
            current = json.loads(subprocess.check_output([str(target / "Contents/MacOS/folio"), "--release-info"], text=True))
            assert current == original_build, "Rejected update changed the installed bundle"
            assert not (stage / "installed.json").exists()
            print(f"Verified refusal of a corrupted backup with the original application preserved: {base}")
            return
        assert result.returncode == 0, (base / "installer.log").read_text()
        installed = json.loads((stage / "installed.json").read_text())
        assert installed["version"] == manifest["version"]
        assert Path(installed["previous"]).is_dir(), "Previous app was not preserved"
        build = json.loads(subprocess.check_output([str(target / "Contents/MacOS/folio"), "--release-info"], text=True))
        assert build["version"] == manifest["version"] and build["revision"] == manifest["revision"]
        wait_for(lambda: len(list(profile.with_suffix(".backups").glob("*.folio"))) > len(backups))
        new_backup = sorted(profile.with_suffix(".backups").glob("*.folio"))[-1]
        new_notebook = json.loads(new_backup.read_text())
        assert new_notebook["pages"] == previous_notebook["pages"], "Notebook changed across bundle replacement"
        print(f"Verified signed installation, startup acknowledgement, retained previous app and unchanged notebook: {base}")
    finally:
        # Only descendants acknowledged by this isolated installation, never a global kill.
        for pid in descendants:
            try:
                command = subprocess.check_output(["/bin/ps", "-p", str(pid), "-o", "comm="], text=True).strip()
                if str(target) in command:
                    os.kill(pid, signal.SIGTERM)
            except (ProcessLookupError, subprocess.CalledProcessError):
                pass
        print(f"Test logs retained in {base}")


if __name__ == "__main__":
    main()
