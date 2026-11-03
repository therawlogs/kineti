# Temporary local Kineti uninstall

Date: 2026-11-03. The owner approved uninstalling this project integration and the globally installed command until the new version is separately approved.

Completed changes:

1. Removed the global npm `kineti` 0.3.9 package from the existing Homebrew prefix, using offline npm with lifecycle scripts, audit and funding requests disabled. The global command is no longer on PATH, and its package root and symlink are absent. Other global npm entries remain present. Node, npm, Bun, Rust and other products were not uninstalled.
2. Removed only the legacy `kineti` entry from the project's Cursor MCP file, preserving the file and all unrelated settings. Retired the obsolete always-on Cursor rule. Updated development instructions so they do not depend on a global installation or automatically register Kineti. The passive example now uses explicit placeholder native paths instead of the removed Bun runner; Cursor remains unverified.
3. Left the main Codex configuration byte-identical: it had no Kineti registration to remove. Locked project state is byte-identical. No Cursor host or legacy Kineti runner process was running during verification, so no app with potentially unsaved work was restarted. Configuration removal and absent processes are verified; a new Cursor launch was not observed.
4. Retained source, work, signatures, accounting, current source-built development tools and both required test VMs. These explicit test binaries do not constitute reinstalling or re-registering the product. No provider login, administrator authentication, model acceptance turn, new paid spending, merge, publication or deployment was performed for the uninstall.

A verified exact backup of the former installed package is 23,780,797 compressed bytes. It and exact configuration/rule backups, before/after hashes, native undo proposals, offline npm command/result and restoration instructions are private under `.kineti/owner-completion-20261103/`. Restoration is a separate reinstallation action; neither a backup nor an undo entry grants permission to execute it. Guarded restoration must refuse later changed files or a newer global installation. Do not automatically restore the old version.

Owner-service implementation continues from retained source. The live APFS snapshot approach hit an additional Apple entitlement requirement in the retained guest; see `AUTONOMOUS-BUILD-CHECKPOINT-DECISION-20261103.md`. The owner approved stop/export checkpoints; the installed protected-worker component passed offline. See `AUTONOMOUS-BUILD-STOP-EXPORT-20261103.md`. The macOS backend remains fail-closed and no full contract acceptance is claimed.
