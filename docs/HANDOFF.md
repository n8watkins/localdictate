# LocalDictate — Session Handoff

Last updated: 2026-06-11 (late evening session)
Read this first, then `docs/STATUS_AND_NEXT_STEPS.md` for deeper project history.

## Project summary

LocalDictate is a Windows-only Tauri app (Rust backend + React/Vite frontend) for
local push-to-talk dictation via whisper.cpp. Hold `Ctrl+Win` (or tap `` ` ``),
talk, and text is typed at the cursor by a locally running model. The owner
(Nathan) uses it daily on Windows 11; it is his personal tool and he is the only
stakeholder. Source of truth is the WSL repo
(`/home/natkins/personal/tools/localdictate`); a Windows clone at
`C:\Users\natha\Projects\Tools\localdictate` exists solely to build/test with the
Windows toolchain. GitHub: `https://github.com/n8watkins/localdictate`.

## State after this session

Eight commits, all verified (Windows `cargo check` clean, **88/88 tests** on
both WSL and Windows toolchains, `npm run build` clean). **Local only — not
pushed.**

| Commit | What |
|---|---|
| `15052af` | Max-duration timeout now transcribes instead of stranding the app in "Transcribing" forever (the freeze bug). `audio.rs::timeout_recording_for_app` mirrors `tray::stop_dictation`. |
| `9e3c903` | Defaults v4 + one-time migration: max recording 600 000 ms, silence auto-stop ON at 60 s (validation up to 300 s), save raw audio ON, paste `Ctrl+Alt+V`, dashboard `Ctrl+Alt+F`. Duration fields labeled "(ms)" with live human-readable readout. |
| `1b53183` | Paste focus guard (`output.rs`): if our own window is foreground, refocus the next eligible Z-order window before SendInput; with no candidate, fail with `paste_target_unavailable` rather than typing into LocalDictate. Also routed paste errors through the `output-failed` event (previously escaped). |
| `c40ccc6` | Saved audio clips for real (`save_audio_clips` was a no-op): WAV moves to `{app_data}/clips/{transcript_id}.wav`, `audioPath` on Transcript (SQLite migration `002_audio_clips.sql`), `get_transcript_audio(id)` command returns base64 WAV, clips deleted with transcript / clear-history / retention sweep / buffer replacement. |
| `a712ba2` | Pill redesign: `pillDisplayMode` setting (`dot` / `visualizer` / `visualizer_with_text`, default text mode, ~5 lines live transcript, window resizes per mode and grows upward); bars driven by `sqrt(rms/0.12)`; 5 s post-transcription confirmation with Copy→Copied button. |
| `98ef6a7` | Archive cleanup: rows are Play (when clip exists) / Copy / Insert / Delete; transcript editing and the output-mode chip removed; horizontal scroll fixed (`overflow-x: hidden`, `overflow-wrap: anywhere`). |
| `8f5a957` | Tail-clipping fix: capture runs 400 ms past a user-initiated stop (`STOP_GRACE_MS` in `audio.rs`); 300 ms silence appended to every incremental segment WAV (`SEGMENT_TAIL_PAD_MS`); trim padding 150→300 ms. Whisper was dropping the last words on abrupt-ending audio. |
| `41bbacd` | Local file transcription: `transcribe_file` command (new `file_transcribe.rs`) runs whisper-cli directly on WAV/MP3/FLAC/OGG, extracts other formats (video etc.) via ffmpeg from PATH (`ffmpeg_required` error if absent), `save_text_file` writes `<source>.txt`. "Transcribe file" card on the Transcribe view (plain path input — dialog plugin deliberately not added). |

**Verified by code/tests only — NOT yet visually QA'd on real hardware:** pill
resize/anchoring per mode, archive Play button end-to-end, the focus guard,
the timeout-transcribe path, the 400 ms stop-grace feel, and the
Transcribe-file card (incl. the ffmpeg path with a real video). First launch
after these changes runs migration 002 and the defaults-v4 migration; both are
tested but watch the logs (`%APPDATA%\com.natkins.localdictate\logs`) on first
real run.

## Next steps (priority order)

1. **Visual QA of this session's work** on the Windows machine: build
   (`npm run tauri build` from `app\` in the Windows clone, or dev run), then:
   dictate past a short max duration to confirm transcription fires; check all
   three pill modes incl. resize; play a clip from the archive; dictate while
   the dashboard is focused and confirm the text lands in the previous app.
2. **Notes feature** — designed but NOT approved in detail; confirm specifics
   with the owner before building big pieces. Decided so far: a dedicated
   note-taking hotkey (owner suggested a `~`+Q-style chord; exact bind TBD),
   pill turns **blue** while taking a note (normal dictation stays
   yellow/orange), notes save as normal transcripts but flagged as notes, a
   Notes section in the dashboard, optional local-LLM analysis with a
   user-editable prompt stored in settings, Google Drive sync via a new
   Integrations tab. Architecture decision already made: **plain Google Drive
   REST + OAuth, not MCP**.
3. **Remove the OpenWhispr model-cache fallback** (`model_manager.rs::external_model_dirs`,
   the `~/.cache/openwhispr/whisper-models` entry) once the owner confirms
   OpenWhispr is uninstalled. It caused "model can't be deleted" confusion
   (delete only touches the app's own dir, then the resolver finds the cache
   copy again). Keep `LOCALDICTATE_MODEL_DIR`.
4. Carry-overs from `docs/STATUS_AND_NEXT_STEPS.md`: flip repo public,
   auto-updater, code signing.

## Conventions & gotchas

- **Windows verification is mandatory** for Rust changes: almost everything is
  `#[cfg(windows)]`; green WSL `cargo check` proves nothing. Sync the clone
  without pushing: from `/mnt/c/Users/natha/Projects/Tools/localdictate`,
  `git fetch /home/natkins/personal/tools/localdictate main && git merge --ff-only FETCH_HEAD`,
  then `cd /mnt/c && cmd.exe /c "cd /d C:\Users\natha\Projects\Tools\localdictate\app\src-tauri && cargo check 2>&1"`
  (likewise `cargo test`).
- Commit per logical change with a `Co-Authored-By: Claude ...` trailer. Push
  only when the owner asks (this session did not push).
- Shipped-default changes go through `CURRENT_DEFAULTS_VERSION` +
  `migrate_defaults` in `settings.rs` (only migrate values still on the old
  default). Brand-new settings fields just need `#[serde(default = ...)]`.
- DB schema changes: numbered SQL files in `app/src-tauri/migrations/`, applied
  in `db.rs::apply_migrations`; "duplicate column name" is treated as
  already-migrated.
- Frontend check: `npx tsc --noEmit -p tsconfig.json` then `npm run build` in `app/`.
- The owner dictates long, stream-of-consciousness requests and prefers
  multi-part batches fanned out to **parallel subagents partitioned by file
  ownership** (no shared files, agents never run git; orchestrator commits and
  verifies).
- Owner deleted/is deleting OpenWhispr; its cache at
  `C:\Users\natha\.cache\openwhispr` (~10 GB) may still exist.

## File map

- `app/src-tauri/src/audio.rs` — capture, silence auto-stop, timeout thread (freeze fix in `timeout_recording_for_app`), stop-grace capture
- `app/src-tauri/src/incremental.rs` — live phrase segmentation; segment tail padding
- `app/src-tauri/src/file_transcribe.rs` — transcribe-a-file backend (whisper-cli + ffmpeg fallback)
- `app/src-tauri/src/dictation.rs` — transcribe pipeline; `save_audio_clip` moves the WAV when clips are on
- `app/src-tauri/src/output.rs` — paste/insert + the focus guard (`ensure_foreign_focus`)
- `app/src-tauri/src/settings.rs` — AppSettings, defaults v4, `PillDisplayMode`
- `app/src-tauri/src/db.rs` — SQLite, migrations, clip-file cleanup
- `app/src-tauri/src/model_manager.rs` — model catalog/download; OpenWhispr fallback to remove (step 4)
- `app/src-tauri/src/commands.rs`, `lib.rs` — Tauri command surface (`get_transcript_audio` etc.)
- `app/src/App.tsx` — dashboard (settings rows, transcript archive with playback)
- `app/src/PillApp.tsx`, `app/src/pill.css` — pill display modes, confirmation state
- `app/src/backend.ts` — TS command wrappers and types (`audioPath`, `pillDisplayMode`)
