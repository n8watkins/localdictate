use std::{fs, path::PathBuf, time::Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    app_state::{AppEvent, AppStateSnapshot, AppStatus},
    audio::{RecordingResult, RecordingResultStatus},
    commands::BackendState,
    error::CommandError,
    incremental::{self, SessionHandle},
    model_manager, output,
    settings::Language,
    transcript::Transcript,
    whisper::{WhisperRequest, WhisperTranscription},
    whisper_server::WarmTranscriber,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationStatus {
    Saved,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictationResult {
    pub session_id: String,
    pub status: DictationStatus,
    pub transcript: Transcript,
    pub model_id: String,
    pub duration_ms: u64,
    pub transcription_latency_ms: u32,
}

pub fn transcribe_recording_for_app(
    app: &AppHandle,
    recording: RecordingResult,
) -> Result<DictationResult, CommandError> {
    // Take ownership of the incremental session (if any) up front so it is
    // consumed exactly once, whatever happens below. `stopped` anchors the
    // stop-to-final-text latency measurement.
    let stopped = Instant::now();
    let incremental = app
        .state::<BackendState>()
        .incremental()
        .take(&recording.session_id);

    validate_recording_result(&recording)?;
    let wav_path = PathBuf::from(recording.wav_path.as_deref().ok_or_else(|| {
        CommandError::new(
            "recording_wav_missing",
            "Recording completed but did not include a WAV path. Record again.",
        )
    })?);
    let cleanup = WavCleanup::new(wav_path.clone());

    let result = transcribe_recording_inner(app, &recording, wav_path, incremental, stopped);
    cleanup.remove();
    result
}

fn transcribe_recording_inner(
    app: &AppHandle,
    recording: &RecordingResult,
    wav_path: PathBuf,
    incremental: Option<SessionHandle>,
    stopped: Instant,
) -> Result<DictationResult, CommandError> {
    let state = app.state::<BackendState>();
    let settings = state.db()?.get_settings()?;
    let (model_id, model_path) = {
        let db = state.db()?;
        model_manager::selected_model_path(app, &db)?
    };
    let language = whisper_language(&settings.language);

    let whisper_result = match incremental
        .and_then(|handle| collect_incremental_transcription(app, recording, handle, stopped))
    {
        Some(assembled) => Ok(assembled),
        None => {
            log::info!(
                "Transcription started for session {} (model {}, recording {} ms)",
                recording.session_id,
                model_id,
                recording.duration_ms
            );
            // The warm whisper-server transcriber falls back to whisper-cli
            // internally when the server path is unavailable.
            app.state::<WarmTranscriber>().transcribe(
                app,
                WhisperRequest {
                    model_path,
                    wav_path,
                    language: language.clone(),
                    vocabulary_prompt: settings.vocabulary_prompt.clone(),
                },
            )
        }
    };

    let whisper_result = match whisper_result {
        Ok(result) => result,
        Err(error) => {
            log::error!(
                "Transcription failed for session {} (model {}): {}",
                recording.session_id,
                model_id,
                error.message
            );
            transition_after_failure(app);
            return Err(error);
        }
    };

    log::info!(
        "Transcription finished for session {} (model {}, latency {} ms)",
        recording.session_id,
        model_id,
        whisper_result.latency_ms
    );

    let Some(mut transcript) = Transcript::new_last_buffer(
        whisper_result.text,
        Some(recording.duration_ms.min(u32::MAX as u64) as u32),
        Some(model_id.clone()),
        Some(language),
    ) else {
        transition_after_failure(app);
        return Err(CommandError::new(
            "empty_transcript",
            "Whisper returned an empty transcript. The previous Last Transcript Buffer was preserved.",
        ));
    };

    transcript.output_mode = Some(settings.output_mode.clone());
    transcript.paste_method = Some(settings.paste_method.clone());
    transcript.transcription_latency_ms = Some(whisper_result.latency_ms);

    state
        .db()?
        .save_last_transcript_with_history(&transcript, settings.history_enabled)?;
    transition_after_success(app);

    let result = DictationResult {
        session_id: recording.session_id.clone(),
        status: DictationStatus::Saved,
        transcript: transcript.clone(),
        model_id,
        duration_ms: recording.duration_ms,
        transcription_latency_ms: whisper_result.latency_ms,
    };

    let _ = app.emit("localdictate:dictation-transcribed", &result);
    if let Err(error) = output::handle_transcription_output(app, &transcript, &settings) {
        output::emit_output_failed(app, transcript.id.clone(), &error);
    }
    Ok(result)
}

/// Waits (bounded) for the incremental coordinator's assembled text and turns
/// it into the dictation transcription, with the latency measured from stop.
/// Returns None whenever the full-clip transcription path should run instead;
/// by the time None is returned the coordinator has definitively finished,
/// failed, or been cancelled, so the fallback never races a segment job for
/// the WarmTranscriber (which is serialized internally anyway).
fn collect_incremental_transcription(
    app: &AppHandle,
    recording: &RecordingResult,
    handle: SessionHandle,
    stopped: Instant,
) -> Option<WhisperTranscription> {
    match handle.wait(incremental::RESULT_TIMEOUT) {
        Ok(assembled) if !assembled.text.is_empty() => {
            let latency_ms = stopped.elapsed().as_millis().min(u32::MAX as u128) as u32;
            log::info!(
                "Incremental transcription assembled {} segment(s) for session {} ({} ms stop-to-text)",
                assembled.segments,
                recording.session_id,
                latency_ms
            );
            incremental::emit_partial_transcript(
                app,
                &recording.session_id,
                &assembled.text,
                assembled.segments,
                true,
            );
            Some(WhisperTranscription {
                text: assembled.text,
                latency_ms,
            })
        }
        Ok(_) => {
            // Zero segments produced any text (e.g. speech never crossed the
            // segmenter threshold): let the full clip decide.
            log::warn!(
                "Incremental transcription produced no text for session {}; falling back to full-clip transcription",
                recording.session_id
            );
            None
        }
        Err(reason) => {
            log::warn!(
                "Incremental transcription unavailable for session {} ({}); falling back to full-clip transcription",
                recording.session_id,
                reason
            );
            // On timeout the coordinator may still be working: tell it to
            // discard everything before the fallback transcription starts.
            handle.cancel();
            None
        }
    }
}

fn validate_recording_result(recording: &RecordingResult) -> Result<(), CommandError> {
    if !matches!(
        recording.status,
        RecordingResultStatus::Completed | RecordingResultStatus::TimedOut
    ) {
        return Err(CommandError::new(
            "recording_not_transcribable",
            "Only completed or timed-out recordings with a WAV file can be transcribed.",
        ));
    }

    if recording
        .wav_path
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        return Err(CommandError::new(
            "recording_wav_missing",
            "Recording completed but did not include a WAV path. Record again.",
        ));
    }

    Ok(())
}

pub(crate) fn whisper_language(language: &Language) -> String {
    match language {
        Language::Auto | Language::En => "en".to_string(),
    }
}

fn transition_after_success(app: &AppHandle) {
    transition_if_transcribing(app, AppEvent::TranscriptionSucceeded);
}

fn transition_after_failure(app: &AppHandle) {
    transition_if_transcribing(app, AppEvent::TranscriptionFailed);
}

fn transition_if_transcribing(app: &AppHandle, event: AppEvent) {
    let state = app.state::<BackendState>();
    let Ok(snapshot) = state.app_state().map(|state| state.snapshot()) else {
        return;
    };

    if snapshot.status != AppStatus::Transcribing {
        return;
    }

    let Ok(snapshot) = state.transition_app_state(event) else {
        return;
    };

    emit_state_snapshot(app, &snapshot);
}

fn emit_state_snapshot(app: &AppHandle, snapshot: &AppStateSnapshot) {
    let _ = app.emit("localdictate:app-state", snapshot);
}

struct WavCleanup {
    path: PathBuf,
}

impl WavCleanup {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn remove(&self) {
        let _ = fs::remove_file(&self.path);
    }
}
