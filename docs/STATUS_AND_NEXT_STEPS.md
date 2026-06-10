# LocalDictate - Status and Next Steps

Status: Ready to start Agent 1 - Backend Foundation  
Last updated: 2026-06-10  
Repository: `https://github.com/n8watkins/localdictate`  
Visibility: Private

## Current Progress

### Product and planning

- Captured the full product requirements in [PRD.md](./PRD.md).
- Created the milestone implementation outline in [IMPLEMENTATION_PLAN.md](./IMPLEMENTATION_PLAN.md).
- Created the multi-agent assignment plan in [AGENT_ORCHESTRATION.md](./AGENT_ORCHESTRATION.md).
- Created the frontend-specific refinement plan in [FRONTEND_UX_REFINEMENT_PLAN.md](./FRONTEND_UX_REFINEMENT_PLAN.md).

### App scaffold

- Created a Tauri v2 + React + TypeScript app in `app/`.
- Set app identity to `LocalDictate` with identifier `com.natkins.localdictate`.
- Installed and used `lucide-react` for app/navigation/action icons.
- Confirmed WSL/Linux Tauri prerequisites are installed and detected.

### Frontend state

- Built the actual frontend foundation, not a throwaway reference mock.
- Current frontend includes views for:
  - Dashboard
  - Transcribe
  - History
  - Settings
  - Hotkeys
  - Models
  - Audio
  - About
- Refined the UI based on product feedback:
  - reduced dashboard clutter
  - added clearer component hierarchy
  - added reusable UI primitives
  - added toggles and setting rows
  - added transcript row actions
  - improved model table/list styling
  - restyled selects/dropdowns for the dark theme

Current frontend data is still mock/local component state. The shell is intended to be kept and wired to backend commands.

### GitHub state

Private GitHub repo has been created and pushed:

```text
https://github.com/n8watkins/localdictate
```

Current commits:

```text
68b9592 Refine LocalDictate frontend UX
7d186c1 Initial LocalDictate scaffold and plans
```

## Verified Commands

From repo root:

```bash
cd /home/natkins/personal/tools/localdictate
git status -sb
```

Expected:

```text
## main...origin/main
```

Frontend build:

```bash
cd /home/natkins/personal/tools/localdictate/app
npm run build
```

Expected: `tsc && vite build` passes.

Tauri environment:

```bash
cd /home/natkins/personal/tools/localdictate/app
npm run tauri info
```

Expected: WebKitGTK, RSVG, Rust, Cargo, Node, npm, and Tauri are detected.

Rust check:

```bash
cd /home/natkins/personal/tools/localdictate/app/src-tauri
cargo check
```

Expected: passes.

## Immediate Next Step

Start:

```text
Agent 1 - Backend Foundation
```

This is the correct next step because the frontend now needs real backend contracts before audio, whisper, paste, tray, hotkey, history, or model work can be integrated cleanly.

## Fresh Context Start Prompt

Use this exact prompt in a new context:

```text
Start Agent 1 - Backend Foundation for LocalDictate.

Repo path: /home/natkins/personal/tools/localdictate

Read these first:
- docs/STATUS_AND_NEXT_STEPS.md
- docs/AGENT_ORCHESTRATION.md
- docs/PRD.md sections 5, 8, 15, 16, and 18

Goal:
Implement the Rust backend foundation for the Tauri app. Create an explicit app state machine, Last Transcript Buffer domain model, settings defaults/schema, SQLite migrations, and initial Tauri commands for frontend integration.

Own these areas:
- app/src-tauri/src/**
- app/src-tauri/Cargo.toml
- app/src-tauri/capabilities/**
- app/src-tauri/migrations/** if needed

Do not implement real audio capture, whisper transcription, tray/hotkeys, or paste behavior yet. Stub command responses are acceptable only when they preserve the final contract shape.

Required commands:
- get_app_state
- get_settings
- update_settings
- get_last_transcript
- clear_last_transcript
- list_recent_transcripts
- get_basic_stats

Required checks:
- cd app/src-tauri && cargo check
- cd app/src-tauri && cargo test
- cd app && npm run build if frontend command types or bindings are touched

Commit and push when done.
```

## Agent 1 Required Deliverables

Agent 1 should produce:

- `app_state` module with states:
  - `Idle`
  - `Recording`
  - `Stopping`
  - `Transcribing`
  - `Pasting`
  - `Ready`
  - `Error`
  - `Paused`
- Last Transcript Buffer model:
  - id
  - text
  - created_at
  - duration_ms
  - word_count
  - character_count
  - model_id
  - language
- Settings model/defaults matching PRD section 16.
- SQLite persistence foundation for:
  - `transcripts`
  - `settings`
  - `models`
  - `app_stats_daily`
- Tauri command registration in `lib.rs`.
- Unit tests for state transitions and metadata helpers.
- A short handoff note for Agent 2 frontend command integration.

## Agent 1 Non-Goals

Do not implement:

- Real audio recording.
- Whisper model downloads.
- whisper.cpp transcription.
- Direct insert paste.
- Clipboard restore paste.
- Windows tray behavior.
- Global hotkey registration.
- Real transcript history search UI wiring.
- Any cloud or telemetry feature.

## Suggested Agent 1 Internal Plan

1. Inspect current Tauri backend:

   ```bash
   cd /home/natkins/personal/tools/localdictate/app/src-tauri
   sed -n '1,220p' src/lib.rs
   sed -n '1,220p' Cargo.toml
   ```

2. Choose a conservative Rust module layout:

   ```text
   src/
     lib.rs
     app_state.rs
     settings.rs
     transcript.rs
     stats.rs
     db.rs
     commands.rs
   ```

3. Add dependencies only if needed. Preferred likely dependencies:

   - `serde`
   - `serde_json`
   - `chrono`
   - `uuid`
   - SQLite layer if implementing persistence now, such as `rusqlite` or Tauri SQL plugin.

4. Implement type models and pure helpers first.

5. Implement simple in-memory `tauri::State` service if SQLite setup would otherwise block command contracts.

6. Add migrations/persistence foundation before finishing if feasible in the same pass.

7. Register commands and test command payload types compile.

8. Run:

   ```bash
   cargo fmt
   cargo test
   cargo check
   ```

9. Commit and push:

   ```bash
   git status -sb
   git add app/src-tauri docs/STATUS_AND_NEXT_STEPS.md
   git commit -m "Add backend foundation"
   git push
   ```

## Sub-Agent Use Instructions

Sub-agents are allowed and encouraged for Agent 1, but keep write scopes disjoint.

Before spawning sub-agents, the lead agent should:

1. Read the required docs.
2. Inspect the existing backend files.
3. Decide the immediate local critical-path task.
4. Delegate only parallel side tasks that do not block the next local step.

Recommended sub-agent split:

### Backend Worker A - State and Domain Models

Ownership:

- `app/src-tauri/src/app_state.rs`
- `app/src-tauri/src/transcript.rs`
- unit tests in those files

Mission:

- Implement app states and transition helpers.
- Implement Last Transcript Buffer and transcript metadata helpers.
- Ensure empty transcript text cannot update the buffer.

### Backend Worker B - Settings and Stats Models

Ownership:

- `app/src-tauri/src/settings.rs`
- `app/src-tauri/src/stats.rs`
- unit tests in those files

Mission:

- Implement settings schema/defaults from PRD section 16.
- Implement basic stats payload types and placeholder aggregation helpers.

### Backend Worker C - Persistence Spike

Ownership:

- `app/src-tauri/src/db.rs`
- `app/src-tauri/migrations/**`
- `app/src-tauri/Cargo.toml` only if dependency changes are required

Mission:

- Determine the simplest SQLite foundation for V1.
- Add the required schema/migrations.
- Expose repository functions or a clear persistence interface.

### Lead Agent - Integration

Ownership:

- `app/src-tauri/src/lib.rs`
- `app/src-tauri/src/commands.rs`
- any final module exports

Mission:

- Integrate workers' modules.
- Register Tauri commands.
- Resolve compile/test failures.
- Run final checks.
- Commit and push.

Important:

- Workers must not edit each other's files.
- Workers must not revert existing frontend work.
- If persistence becomes too large, preserve command contract shapes with in-memory state and document persistence follow-up clearly.

## Agent 1 Completion Criteria

Agent 1 is done when:

- The backend compiles.
- Tests pass.
- Initial Tauri commands exist and return stable payload shapes.
- Last Transcript Buffer is modeled separately from clipboard/history.
- Settings defaults exist.
- SQLite schema/migration path is either implemented or explicitly stubbed with a clear next task.
- A commit is pushed to the private GitHub repo.

## After Agent 1

Next recommended work:

1. Agent 2 - Frontend Command Integration.
2. Agent 3 - Tray and Global Hotkeys.
3. Agent 4 - Audio Capture.

Agent 2 should replace mock data in `App.tsx` with real command calls. Agent 3 and Agent 4 can run in parallel once Agent 1 contracts are stable.
