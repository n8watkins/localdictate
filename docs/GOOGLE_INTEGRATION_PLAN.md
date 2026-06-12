# LocalDictate — Google Integration & Cloud Analysis Plan

Status: PLANNED (not started). Written 2026-06-12.
Owner vision captured from a planning conversation; this is the implementation
blueprint, not yet code. Read `docs/HANDOFF.md` for current project state.

## Goal / product intent

Make LocalDictate an "awesome free tool" that works whether or not the user can
run a local LLM:

1. **Cloud analysis option (Gemini, bring-your-own-key).** People who can't
   afford / don't want to run a local model can paste their *own* free Google
   AI Studio API key and have Gemini analyze their notes. We do NOT resell
   tokens or get into billing — the user's key, the user's quota. They pick the
   Gemini model; we surface their token usage and link out to Google's usage
   page.
2. **Google Drive sync of notes.** Push notes to a Drive folder
   ("LocalDictate Voice Notes") so the owner can reference them from his other
   tools — organized by daily timestamps.
3. **Minimal background footprint.** The local LLM should not have to run all
   day; LocalDictate itself is the only required always-on process (for
   hotkeys). Cloud users run nothing locally.

Non-goals: reselling LLM access; a hosted backend; the owner's separate
"one hotkey launches my apps" idea (unrelated — track elsewhere).

## Background footprint — answers to the owner's questions

- **Local LLM does NOT need to run constantly.** LM Studio's
  **JIT load + Max idle TTL** (already configured; set TTL ~5 min) loads the
  model on demand and unloads it from VRAM after idle. Good for gaming —
  VRAM frees itself.
- **Headless is the right way to keep it always-available.** LM Studio
  Settings → Developer → **"Enable Local LLM Service (headless)"** runs the
  server as a background service WITHOUT the GUI app open. Turn it on and never
  open the LM Studio window.
- **Gemini users run zero local processes** — the biggest footprint win.
- **LocalDictate stays always-on** (tray) because the global hotkeys require a
  live process. Already minimizes to tray.

## Feature 1 — Pluggable analysis provider (Local | Gemini BYO-key)

Today `app/src-tauri/src/note_analysis.rs::analyze_text` is hardcoded to an
OpenAI-compatible `/chat/completions` call (LM Studio / Ollama). Generalize it
to a provider dimension.

### Settings (settings.rs / backend.ts / App.tsx)
New fields (all `#[serde(default)]`, bump `CURRENT_DEFAULTS_VERSION` only if a
shipped default changes):
- `notesAnalysisProvider`: `"local"` | `"gemini"` (default `"local"`).
- `notesAnalysisGeminiModel`: e.g. `"gemini-2.5-flash"` (default empty → pick).
- Gemini **API key is a SECRET** → store in the OS keychain (Windows Credential
  Manager via the `keyring` crate), NOT in the SQLite settings JSON. Settings
  holds only a boolean "key is set".

### Backend
- Refactor `note_analysis.rs` into a small provider enum:
  - `local`: existing OpenAI-compatible path (unchanged).
  - `gemini`: POST
    `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={KEY}`
    body `{ "system_instruction": {parts:[{text: prompt}]},
    "contents":[{parts:[{text: note}]}] }`; parse
    `candidates[0].content.parts[0].text`. reqwest is `default-features=false`,
    so use `.text()` + serde_json (same as the existing code).
  - Capture `usageMetadata.{promptTokenCount,candidatesTokenCount,totalTokenCount}`.
- New command `list_gemini_models(key)` → `GET /v1beta/models?key=` filtered to
  models supporting `generateContent`, for the UI dropdown.
- New command `test_analysis_connection` (works for both providers).
- Persist usage: add columns/table for cumulative tokens + request count
  (extend `app_stats_daily` or a new `analysis_usage` table). Return to UI.

### Frontend (App.tsx Settings → Notes analysis)
- Provider selector (Local / Gemini). Conditionally render:
  - Local → existing endpoint + model fields.
  - Gemini → API-key field (write-only, shows "set/not set"), model dropdown
    (populated by `list_gemini_models`), "Get a free key" link to AI Studio.
- "Test connection" button.
- Usage readout (tokens this session/total) + link to Google's usage page.
  We track tokens locally; we canNOT read dollar billing without the heavy
  Cloud Billing API — link out for cost.

## Feature 2 — Google Drive notes sync

New **Integrations** tab in Settings (decided long ago: plain Drive REST +
OAuth, NOT MCP).

### OAuth (the fiddly part)
- Desktop OAuth 2.0 with **loopback redirect + PKCE** (no real client secret;
  desktop secrets aren't secret). Embed a client ID shipped with the app.
- Scope **`drive.file` ONLY** — app-created files. Least privilege AND it keeps
  us out of Google's restricted-scope security assessment, which matters for
  distributing free to others.
- Store the **refresh token in the OS keychain** (keyring crate), never SQLite.
- Flow: open system browser → user consents → loopback catches the code →
  exchange for tokens → refresh as needed.

### Sync behavior
- Find/create a Drive folder **"LocalDictate Voice Notes"** (remember its id).
- On note save and/or analyze, upload. **Default layout: one Markdown file per
  day** (`YYYY-MM-DD.md`), each note appended as a timestamped section
  (download-modify-upload, or keep a daily file id). Configurable alternative:
  one file per note (`YYYY-MM-DD_HHMMSS_<slug>.md`) — simpler, no
  read-modify-write.
- Include note text + analysis (if present) + timestamp.
- A manual "Sync now / backfill" button plus optional auto-on-save.
- Enable/disable toggle; clear "signed in as <email> / Sign out".

### Files touched
- New `app/src-tauri/src/google_oauth.rs`, `google_drive.rs`.
- `lib.rs`: register new commands.
- `backend.ts`: wrappers/types.
- `App.tsx`: new Integrations tab.

## Phasing

1. **Provider abstraction + Gemini BYO-key** — self-contained, immediately
   useful, no OAuth. Ship behind the existing notes-analysis UI.
2. **Drive sync** — own branch; OAuth + Drive client + Integrations tab.
3. **Polish** — usage display, headless docs, per-integration disable toggles,
   "sync now / backfill".

## Decisions (owner may override)

- Secrets (Gemini key, Google refresh token) → **OS keychain** (`keyring`
  crate), not plaintext SQLite. Matters because the app is distributed.
- Drive scope = **`drive.file`** (avoids Google verification).
- Drive layout = **daily Markdown rollup** files (referenceable), configurable.

## Open questions to confirm before building

- Drive: daily-rollup file vs one-file-per-note as the default?
- Auto-sync every note on save, or manual "sync now" only?
- Sync just notes, or full transcripts too?
- Distribute one shared Google OAuth client ID with the app, or document how
  each user creates their own? (Shared is friendlier; needs an unverified-app
  consent screen or going through Google verification for the brand.)

## Cross-cutting reminders (from HANDOFF.md)

- Windows verification is mandatory for Rust changes; test on the **Dev flavor**
  first, ship to stable only on owner request.
- New settings fields need `#[serde(default)]`; changed shipped defaults go
  through `CURRENT_DEFAULTS_VERSION` + `migrate_defaults`.
- reqwest is `default-features = false`: no `.json()`, use `.text()` +
  serde_json.
- Commit per logical change with the Co-Authored-By trailer.
