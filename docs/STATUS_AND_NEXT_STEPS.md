# Scribe - Status and Next Steps

> **Current handoff: see [`docs/HANDOFF.md`](HANDOFF.md)** — it carries the
> 2026-06-11 evening session (timeout-freeze fix, defaults v4, saved audio
> clips + playback, pill display modes, paste focus guard, tail-clipping fix,
> file transcription) and the up-to-date next steps. This doc keeps the longer
> project history.

Status: V1 shipped — working Windows installer, hotkey dictation verified on real hardware by the owner  
Last updated: 2026-06-11 (see HANDOFF.md for the latest session)  
Repository: `https://github.com/n8watkins/scribe` (private; ready to flip public)  
Release: [v0.1.0](https://github.com/n8watkins/scribe/releases/tag/v0.1.0) with the NSIS installer attached

## Where things stand

Scribe is a working product: hold `Ctrl+Shift` (or tap `~`), talk, and text is typed at your cursor by a locally running whisper.cpp model. The owner uses it daily on Windows 11. 54 backend tests pass on both Linux and Windows; the frontend builds clean.

## What was done (2026-06-10 → 06-11)

### Made it compile and ship
- Fixed the Windows build (170 errors from three roots: cpal's `!Send` stream wrapper in Tauri managed state, a missing `windows`-crate feature, stray `Debug` derives). Established the rule that matters: **Windows-gated code must be verified with the Windows toolchain** — `cargo check` in WSL never compiles it. Workflow: WSL repo is source of truth; the clone at `C:\Users\natha\Projects\Tools\localdictate` builds/tests via `cmd.exe` interop.
- Produced the first NSIS/MSI installers; pruned the whisper.cpp resource drop to exactly the needed binaries (everything in `resources/` gets bundled); verified the MSI payload.

### Made hotkeys real
- Replaced the unusable defaults (`Ctrl+Win+D` is "new virtual desktop"…) with `Ctrl+Shift` hold-to-talk, `~` toggle, `Ctrl+Alt+V` paste, `Ctrl+Alt+D` dashboard — with one-time migrations for existing installs.
- Modifier-only chords (e.g. bare `Ctrl+Shift`) are unsupported by the global-shortcut plugin, so there's a native Windows watcher: `GetAsyncKeyState` polling with a 150 ms arming delay that suppresses ordinary `Ctrl+Shift+<key>` shortcuts.
- Real rebind UI (press-to-capture, inline conflict errors, reset to defaults). Registration is per-binding best-effort with failures surfaced as toasts — and the recording-mode gate that silently discarded hold-to-talk presses in toggle mode is gone.

### Made it pleasant
- Pill overlay is a real always-on-top frameless window (label `pill`): visible while the main window is hidden, draggable, position persisted (`pillX`/`pillY`), click-to-stop.
- UI restructured and densified twice: Stats and Data & Privacy views, History owns recents, icon-only actions with tooltips, friendly mic names (never endpoint GUIDs), audio start/stop cues, stop controls in topbar/pill, test-clip playback, open data/models folder commands, 940×600 default window.

### Made it fast and smart (waves 1–2)
- **Auto-paste is the default output mode** (versioned migration via `defaultsVersion`).
- **Warm transcriber** (`src/whisper_server.rs`): resident `whisper-server.exe` holds the model in RAM across dictations; per-request vocabulary prompt (verified empirically); 10-minute idle shutdown; auto-fallback to `whisper-cli.exe`; killed on exit. `transcribe()` is a stateless serialized primitive, deliberately segment-shaped.
- **Auto-stop on silence** for toggle/manual recordings (arms after speech ≥ 0.03 RMS, fires after `silenceAutoStopMs` below 0.015 RMS); real silence trimming replaced the placeholder.
- **Custom vocabulary** setting → whisper `--prompt`.
- Single-instance plugin; file logging via `tauri-plugin-log` (LogDir) — release builds are no longer silent.

### Made it open-sourceable
- Root README (user install + build-from-source incl. the required whisper.cpp binaries), MIT LICENSE, v0.1.0 GitHub release with installer.

## What to do next (priority order)

Done since this list was written: incremental transcription (shipped),
tag-triggered CI release workflow + manual dispatch (shipped), launch at
startup (wired). Current priorities live in [`docs/HANDOFF.md`](HANDOFF.md) —
short version: visual QA of the 06-11 evening session, the Notes feature
(needs owner sign-off on details), removing the OpenWhispr cache fallback.

Still-open carry-overs:
1. **Flip the repo public** (owner action — Settings → Change visibility). Everything is in place.
2. **Auto-updater** — `tauri-plugin-updater` (needs updater signing keys; unrelated to code signing).
3. **Code signing** — kills the SmartScreen warning; costs money; matters once strangers install it.
4. Smaller: GPU whisper builds as optional download, tray icon state variants, FTS5 search if histories grow, pill shutdown when main window closes with tray-minimize off (known edge case, currently moot).

## Working notes for the next session

- Verify Rust changes on Windows: `cd /mnt/c && cmd.exe /c "cd /d C:\Users\natha\Projects\Tools\localdictate\app\src-tauri && cargo check 2>&1"` (likewise `cargo test`, `npm run tauri build` from `app\`).
- The Windows clone's `resources/bin/windows/` binaries are untracked — `git reset --hard` keeps them. `whisper-extras-unbundled/` at the clone root holds the unused whisper.cpp extras.
- Installed-app data (settings DB, models, logs): `%APPDATA%\com.natkins.localdictate\` — readable from WSL for debugging; reading the settings JSON there found the toggle-mode bug.
