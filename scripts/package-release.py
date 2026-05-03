#!/usr/bin/env python3
from __future__ import annotations

import json
import shutil
import sys
import tarfile
import zipfile
import subprocess
from pathlib import Path


def find_first(base: Path, patterns: list[str]) -> Path:
    for pattern in patterns:
        matches = sorted(base.glob(pattern))
        if matches:
            return matches[0]
    raise FileNotFoundError(f"no file matching {patterns} in {base}")

def get_resolved_version(package_name: str) -> str:
   
    try:
        cmd = ["cargo", "metadata", "--format-version", "1", "--no-deps"]
        result = subprocess.run(cmd, capture_output=True, text=True, check=True)
        data = json.loads(result.stdout)
        
        for pkg in data["packages"]:
            if pkg["name"] == package_name:
                return pkg["version"]
        
        raise ValueError(f"Package {package_name} not found in metadata")
    except Exception as e:
        print(f"Error resolving version for {package_name}: {e}", file=sys.stderr)
        sys.exit(1)


def main() -> int:
    if len(sys.argv) != 4:
        print("usage: package-release.py <target> <profile> <out-dir>", file=sys.stderr)
        return 1

    target, profile, out_dir_raw = sys.argv[1:]
    root_dir = Path(__file__).resolve().parent.parent
    out_dir = Path(out_dir_raw).resolve()
    
    # --- PROSES WASM ---
    if target == "wasm32-unknown-unknown":
        wasm_pkg_dir = root_dir / "crates" / "axhash-wasm"
        wasm_out_dir = root_dir / "target" / "wasm-package"
        
        print(f"--- Building WASM for {profile} profile ---")
        
        wasm_pack_cmd = [
            "wasm-pack", "build", str(wasm_pkg_dir),
            "--target", "bundler",
            "--out-dir", str(wasm_out_dir),
            "--release" if profile == "release" else "--dev"
        ]
        
        try:
            subprocess.run(wasm_pack_cmd, check=True)
        except FileNotFoundError:
            print("Error: wasm-pack not found.", file=sys.stderr)
            return 1

        # Menggunakan resolver versi otomatis
        version = get_resolved_version("axhash-wasm")
        archive_name = out_dir / f"axhash-wasm-{version}.tar.gz"

        out_dir.mkdir(parents=True, exist_ok=True)
        with tarfile.open(archive_name, "w:gz") as tf:
            tf.add(wasm_out_dir, arcname=".")
        
        print(f"WASM package created at: {archive_name}")
        return 0

    # --- PROSES NATIVE (FFI) ---
    lib_dir = root_dir / "target" / target / profile
    stage_dir = root_dir / "target" / "package" / target
    include_dir = root_dir / "crates" / "axhash-ffi" / "include"
    
    # Menggunakan resolver versi otomatis
    version = get_resolved_version("axhash-ffi")
    archive_base = f"axhash-ffi-{version}-{target}"

    if stage_dir.exists():
        shutil.rmtree(stage_dir)
    (stage_dir / "include").mkdir(parents=True)
    out_dir.mkdir(parents=True, exist_ok=True)

    shutil.copy2(root_dir / "LICENSE-MIT", stage_dir / "LICENSE-MIT")
    shutil.copy2(root_dir / "crates" / "axhash-ffi" / "README.md", stage_dir / "README.md")
    shutil.copy2(include_dir / "axhash.h", stage_dir / "include" / "axhash.h")

    if "windows-msvc" in target:
        bins = [
            find_first(lib_dir, ["axhash_ffi.dll", "axhash-ffi.dll"]),
            find_first(lib_dir, ["axhash_ffi.lib", "axhash-ffi.lib"]),
        ]
        for item in bins:
            shutil.copy2(item, stage_dir / item.name)

        archive = out_dir / f"{archive_base}.zip"
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as zf:
            for path in stage_dir.rglob("*"):
                if path.is_file():
                    zf.write(path, path.relative_to(stage_dir))
        return 0

    shared_name = "libaxhash_ffi.dylib" if "apple-darwin" in target else "libaxhash_ffi.so"
    files = [find_first(lib_dir, ["libaxhash_ffi.a"]), find_first(lib_dir, [shared_name])]
    for item in files:
        shutil.copy2(item, stage_dir / item.name)

    archive = out_dir / f"{archive_base}.tar.gz"
    with tarfile.open(archive, "w:gz") as tf:
        for path in stage_dir.rglob("*"):
            tf.add(path, arcname=path.relative_to(stage_dir))
    
    print(f"Native package created at: {archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())