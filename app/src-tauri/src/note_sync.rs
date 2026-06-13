//! Phase 2 auto-sync: when a note is saved and Drive sync is enabled, push the
//! day's notes to Google Drive off the dictation thread. A background worker
//! debounces a burst of saves into a single sync.

use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender};
use tauri::{AppHandle, Manager};

use crate::commands::BackendState;
use crate::error::CommandError;
use crate::google_drive::{Drive, SyncReport};

/// Quiet period after the last note save before a sync fires, so rapid notes
/// collapse into one upload.
const DEBOUNCE: Duration = Duration::from_secs(3);

/// Gathers the notes from the DB and syncs them to Google Drive. Shared by the
/// manual `drive_sync_now` command and the auto-sync worker. Blocking (DB +
/// network), so callers run it off the main thread. The DB lock is dropped
/// before any network call.
pub fn collect_and_sync(app: &AppHandle, service: &str) -> Result<SyncReport, CommandError> {
    let state = app.state::<BackendState>();
    let notes = {
        let db = state.db()?;
        let settings = db.get_settings()?;
        if settings.drive_account_email.is_empty()
            && !crate::google_oauth::has_stored_token(service)
        {
            return Err(CommandError::new(
                "google_not_signed_in",
                "Sign in to Google in Settings → Integrations first.",
            ));
        }
        // The daily Drive file is a clean notes-only log (Phase 1/2).
        db.search_transcripts(None, true, 100_000, 0)?.transcripts
    };

    if notes.is_empty() {
        return Ok(SyncReport {
            synced_notes: 0,
            files_written: 0,
        });
    }

    let token = crate::google_oauth::access_token(service)?;
    Drive::new(token)?.sync_notes(&notes)
}

/// Owns the channel that note-save events are pushed onto. Held in Tauri's
/// managed state; dropping it (on app exit) ends the worker thread.
pub struct DriveSyncWorker {
    tx: Sender<()>,
}

impl DriveSyncWorker {
    /// Spawns the background debounce-and-sync thread.
    pub fn spawn(app: AppHandle) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded::<()>();
        let service = app.config().identifier.clone();
        let _ = thread::Builder::new()
            .name("scribe-drive-sync".into())
            .spawn(move || worker_loop(app, service, rx));
        Self { tx }
    }

    /// Signals that a note was saved. The worker debounces, then syncs. Cheap
    /// and non-blocking; safe to call from the dictation path.
    pub fn notify(&self) {
        let _ = self.tx.send(());
    }
}

fn worker_loop(app: AppHandle, service: String, rx: Receiver<()>) {
    loop {
        // Block until at least one note-saved signal arrives.
        if rx.recv().is_err() {
            return; // sender dropped — app is shutting down
        }
        // Debounce: keep waiting while more saves keep coming in.
        loop {
            match rx.recv_timeout(DEBOUNCE) {
                Ok(()) => continue,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        match collect_and_sync(&app, &service) {
            Ok(report) => log::info!(
                "Auto-synced {} note(s) into {} Drive file(s)",
                report.synced_notes,
                report.files_written
            ),
            Err(error) => {
                log::warn!("Auto-sync to Google Drive failed: {}", error.message)
            }
        }
    }
}
