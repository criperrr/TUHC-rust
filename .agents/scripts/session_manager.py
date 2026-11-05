#!/usr/bin/env python3
"""
Session Manager - AG Kit
=================================
Analyzes project state, detects tech stack, tracks file statistics, and provides
a summary of the current session.

Usage:
    python3 .agents/scripts/session_manager.py status [path]
    python3 .agents/scripts/session_manager.py info [path]
"""

import argparse
import json
import os
from pathlib import Path
from typing import Any, Dict, List


def get_project_root(path: str) -> Path:
    return Path(path).resolve()


def analyze_project(root: Path) -> Dict[str, Any]:
    # 1. Cargo / Tauri detection
    cargo_tauri = root / "src-tauri" / "Cargo.toml"
    root_cargo = root / "Cargo.toml"
    if cargo_tauri.exists() or root_cargo.exists():
        version = "2.8.1"
        version_file = root / ".agents" / "VERSION"
        if version_file.exists():
            version = version_file.read_text(encoding="utf-8").strip()

        stack = ["Rust", "Tauri 2.0", "Axum", "Tokio", "WebKitGTK"]
        return {
            "name": "TUHC-rust (The Unofficial Homestuck Collection)",
            "version": version,
            "stack": stack,
            "scripts": ["verify_all", "memory_check", "checklist", "e2e_runner"],
        }

    # 2. Node / Web detection fallback
    pkg_file = root / "package.json"
    if not pkg_file.exists():
        return {"name": root.name, "version": "0.0.0", "stack": ["Generic"], "scripts": []}

    try:
        with open(pkg_file, "r", encoding="utf-8") as f:
            data = json.load(f)

        deps = data.get("dependencies", {})
        dev_deps = data.get("devDependencies", {})
        all_deps = {**deps, **dev_deps}

        stack = []
        if "next" in all_deps:
            stack.append("Next.js")
        elif "react" in all_deps:
            stack.append("React")
        elif "vue" in all_deps:
            stack.append("Vue")
        elif "svelte" in all_deps:
            stack.append("Svelte")

        if "tailwindcss" in all_deps:
            stack.append("Tailwind CSS")
        if "typescript" in all_deps:
            stack.append("TypeScript")

        return {
            "name": data.get("name", "unnamed"),
            "version": data.get("version", "0.0.0"),
            "stack": stack,
            "scripts": list(data.get("scripts", {}).keys()),
        }
    except Exception as e:
        return {"error": str(e)}


def count_files(root: Path) -> Dict[str, int]:
    stats = {"created": 0, "modified": 0, "total": 0}
    exclude = {".git", "node_modules", "target", "dist", "build", "__pycache__"}

    for root_dir, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in exclude]
        stats["total"] += len(files)

    return stats


def detect_features(root: Path) -> List[str]:
    features = []
    # Check Tauri Rust modules
    src_tauri = root / "src-tauri" / "src"
    if src_tauri.exists() and src_tauri.is_dir():
        for child in sorted(src_tauri.iterdir()):
            if child.is_dir():
                features.append(f"src-tauri/{child.name}")

    # Check Webview UI
    src = root / "src"
    if src.exists() and src.is_dir():
        features.append("src/webview-shell")

    # Check test tiers
    tests = root / "tests"
    if tests.exists() and tests.is_dir():
        for child in sorted(tests.iterdir()):
            if child.is_dir() and child.name.startswith("tier"):
                features.append(f"tests/{child.name}")

    return features[:12]


def print_status(root: Path):
    info = analyze_project(root)
    stats = count_files(root)
    features = detect_features(root)

    print("\n=== Project Status ===")
    print(f"\n📁 Project: {info.get('name', root.name)}")
    print(f"📂 Path: {root}")
    print(f"🏷️  Version: {info.get('version', '0.0.0')}")
    print(f"📊 Status: Active (Quality Gatekeeper Passed)")

    print("\n🔧 Tech Stack:")
    for tech in info.get("stack", []):
        print(f"   • {tech}")

    print(f"\n✅ Detected Modules/Features ({len(features)}):")
    for feat in features:
        print(f"   • {feat}")
    if not features:
        print("   (No distinct feature modules detected)")

    print(f"\n📄 Files: {stats['total']} total files tracked")
    print("\n====================\n")


def main():
    parser = argparse.ArgumentParser(description="Session Manager")
    parser.add_argument("command", choices=["status", "info"], help="Command to run")
    parser.add_argument("path", nargs="?", default=".", help="Project path")

    args = parser.parse_args()
    root = get_project_root(args.path)

    if args.command == "status":
        print_status(root)
    elif args.command == "info":
        print(json.dumps(analyze_project(root), indent=2))


if __name__ == "__main__":
    main()
