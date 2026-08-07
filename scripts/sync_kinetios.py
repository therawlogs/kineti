#!/usr/bin/env python3
"""
Kineti OS Sync Engine (v0.1)
Synchronizes changes from your central Kineti_OS development project in Documents
into your global AI configuration directory (~/.gemini/config/).

Usage:
  python3 /Users/praveen/Documents/Kineti_OS/scripts/sync_kinetios.py
"""

import os
import shutil
import sys
from pathlib import Path

def main():
    kineti_dir = Path("/Users/praveen/Documents/Kineti_OS")
    target_config_dir = Path("/Users/praveen/.gemini/config")

    if not kineti_dir.exists():
        print(f"Error: Central Kineti_OS directory not found at {kineti_dir}")
        sys.exit(1)

    target_config_dir.mkdir(parents=True, exist_ok=True)
    (target_config_dir / "scripts").mkdir(parents=True, exist_ok=True)
    (target_config_dir / "skills").mkdir(parents=True, exist_ok=True)

    # 1. Sync Root Master Documents
    master_files = [
        "ETHOS.md",
        "guide_to_kinetios.md",
        "master_fde_founder_specification.md",
        "founder_sprint_framework.md",
        "sdlc_hybrid_guide.md",
        "README.md"
    ]
    for f_name in master_files:
        src = kineti_dir / f_name
        if src.exists():
            shutil.copy2(src, target_config_dir / f_name)
            print(f"Synced document: {f_name} -> {target_config_dir / f_name}")

    # 2. Sync Scripts
    scripts_src = kineti_dir / "scripts"
    if scripts_src.exists():
        for s_file in scripts_src.glob("*.py"):
            shutil.copy2(s_file, target_config_dir / "scripts" / s_file.name)
            print(f"Synced script: {s_file.name} -> {target_config_dir / 'scripts' / s_file.name}")

    # 3. Sync Skills
    skills_src = kineti_dir / "skills"
    if skills_src.exists():
        for item in skills_src.iterdir():
            if item.is_dir():
                target_skill_dir = target_config_dir / "skills" / item.name
                if target_skill_dir.exists():
                    shutil.rmtree(target_skill_dir)
                shutil.copytree(item, target_skill_dir)
                print(f"Synced skill folder: /{item.name} -> {target_skill_dir}")

    print("\n=== Kineti OS Synchronization Complete ===")
    print("All master files, scripts, and skills are updated in global config.")

if __name__ == "__main__":
    main()
