use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub launch_at_startup: bool,
    pub minimize_to_tray: bool,
    pub show_floating_pill: bool,
    pub notifications_enabled: bool,
    pub sounds_enabled: bool,
    pub recording_mode: RecordingMode,
    pub min_recording_ms: u32,
    pub max_recording_ms: u32,
    pub silence_trim_enabled: bool,
    pub selected_mic_id: Option<String>,
    pub selected_model_id: Option<String>,
    pub language: Language,
    pub output_mode: OutputMode,
    pub paste_method: PasteMethod,
    pub history_enabled: bool,
    pub save_audio_clips: bool,
    pub history_retention_days: Option<u16>,
    pub hotkeys: HotkeySettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordingMode {
    Hold,
    Toggle,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    Auto,
    En,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    SaveOnly,
    AutoPaste,
    CopyClipboard,
    CopyAndPaste,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasteMethod {
    DirectInsert,
    ClipboardRestore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeySettings {
    pub hold_to_talk: String,
    pub toggle_dictation: String,
    pub paste_last_transcript: String,
    pub open_dashboard: String,
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            hold_to_talk: "Ctrl+Shift".to_string(),
            toggle_dictation: "Backquote".to_string(),
            paste_last_transcript: "Ctrl+Alt+V".to_string(),
            open_dashboard: "Ctrl+Alt+D".to_string(),
        }
    }
}

impl HotkeySettings {
    /// The defaults that shipped before modifier-only chord support. Windows
    /// intercepts Ctrl+Win+Space (layout switcher) and Ctrl+Win+D (new
    /// virtual desktop), so installs still on these exact values are migrated
    /// to the current defaults.
    pub fn matches_legacy_defaults(&self) -> bool {
        self.hold_to_talk == "Ctrl+Win+Space"
            && self.toggle_dictation == "Ctrl+Win+D"
            && self.paste_last_transcript == "Ctrl+Alt+V"
            && self.open_dashboard == "Ctrl+Win+H"
    }

    /// Replaces the stored hotkeys with the current defaults when they still
    /// exactly equal the legacy defaults. Returns true when a migration
    /// happened and the settings should be saved back.
    pub fn migrate_legacy_defaults(&mut self) -> bool {
        if self.matches_legacy_defaults() {
            *self = Self::default();
            true
        } else {
            false
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_at_startup: false,
            minimize_to_tray: true,
            show_floating_pill: true,
            notifications_enabled: true,
            sounds_enabled: true,
            recording_mode: RecordingMode::Both,
            min_recording_ms: 300,
            max_recording_ms: 180_000,
            silence_trim_enabled: true,
            selected_mic_id: None,
            selected_model_id: Some("small.en-q5_1".to_string()),
            language: Language::En,
            output_mode: OutputMode::SaveOnly,
            paste_method: PasteMethod::DirectInsert,
            history_enabled: true,
            save_audio_clips: false,
            history_retention_days: Some(30),
            hotkeys: HotkeySettings::default(),
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), SettingsValidationError> {
        if self.min_recording_ms == 0 {
            return Err(SettingsValidationError::new(
                "minRecordingMs must be greater than zero.",
            ));
        }

        if self.max_recording_ms < self.min_recording_ms {
            return Err(SettingsValidationError::new(
                "maxRecordingMs must be greater than or equal to minRecordingMs.",
            ));
        }

        if !matches!(self.history_retention_days, Some(7 | 30 | 90 | 365) | None) {
            return Err(SettingsValidationError::new(
                "historyRetentionDays must be 7, 30, 90, 365, or null.",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsValidationError {
    message: String,
}

impl SettingsValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SettingsValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SettingsValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_prd_baseline() {
        let settings = AppSettings::default();

        assert_eq!(settings.recording_mode, RecordingMode::Both);
        assert_eq!(settings.min_recording_ms, 300);
        assert_eq!(settings.max_recording_ms, 180_000);
        assert_eq!(settings.output_mode, OutputMode::SaveOnly);
        assert_eq!(settings.paste_method, PasteMethod::DirectInsert);
        assert!(settings.history_enabled);
        assert!(!settings.save_audio_clips);
    }

    #[test]
    fn validates_history_retention_options() {
        let mut settings = AppSettings::default();
        settings.history_retention_days = Some(14);

        assert!(settings.validate().is_err());
    }

    #[test]
    fn default_hotkeys_avoid_windows_reserved_shortcuts() {
        let hotkeys = HotkeySettings::default();

        assert_eq!(hotkeys.hold_to_talk, "Ctrl+Shift");
        assert_eq!(hotkeys.toggle_dictation, "Backquote");
        assert_eq!(hotkeys.paste_last_transcript, "Ctrl+Alt+V");
        assert_eq!(hotkeys.open_dashboard, "Ctrl+Alt+D");
    }

    #[test]
    fn migrates_exact_legacy_default_hotkeys() {
        let mut hotkeys = HotkeySettings {
            hold_to_talk: "Ctrl+Win+Space".to_string(),
            toggle_dictation: "Ctrl+Win+D".to_string(),
            paste_last_transcript: "Ctrl+Alt+V".to_string(),
            open_dashboard: "Ctrl+Win+H".to_string(),
        };

        assert!(hotkeys.migrate_legacy_defaults());
        assert_eq!(hotkeys, HotkeySettings::default());
    }

    #[test]
    fn does_not_migrate_customized_hotkeys() {
        let mut hotkeys = HotkeySettings {
            hold_to_talk: "Ctrl+Win+Space".to_string(),
            toggle_dictation: "Ctrl+Win+D".to_string(),
            paste_last_transcript: "Ctrl+Alt+V".to_string(),
            open_dashboard: "Ctrl+Alt+J".to_string(),
        };
        let before = hotkeys.clone();

        assert!(!hotkeys.migrate_legacy_defaults());
        assert_eq!(hotkeys, before);
    }

    #[test]
    fn does_not_migrate_current_defaults() {
        let mut hotkeys = HotkeySettings::default();

        assert!(!hotkeys.migrate_legacy_defaults());
        assert_eq!(hotkeys, HotkeySettings::default());
    }
}
