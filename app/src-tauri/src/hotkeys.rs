use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Mutex,
};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

use crate::{
    app_state::AppStatus,
    commands::BackendState,
    error::CommandError,
    settings::{HotkeySettings, RecordingMode},
    tray,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HotkeyAction {
    HoldToTalk,
    ToggleDictation,
    PasteLastTranscript,
    OpenDashboard,
}

impl HotkeyAction {
    pub fn parse(value: &str) -> Result<Self, CommandError> {
        match value {
            "holdToTalk" | "hold_to_talk" | "hold-to-talk" => Ok(Self::HoldToTalk),
            "toggleDictation" | "toggle_dictation" | "toggle-dictation" => {
                Ok(Self::ToggleDictation)
            }
            "pasteLastTranscript" | "paste_last_transcript" | "paste-last-transcript" => {
                Ok(Self::PasteLastTranscript)
            }
            "openDashboard" | "open_dashboard" | "open-dashboard" => Ok(Self::OpenDashboard),
            _ => Err(CommandError::new(
                "invalid_hotkey_action",
                format!("Unknown hotkey action '{}'.", value),
            )),
        }
    }

    pub fn event_name(self) -> &'static str {
        match self {
            Self::HoldToTalk => "hold_to_talk",
            Self::ToggleDictation => "toggle_dictation",
            Self::PasteLastTranscript => "paste_last_transcript",
            Self::OpenDashboard => "open_dashboard",
        }
    }

    pub fn shortcut(self, hotkeys: &HotkeySettings) -> &str {
        match self {
            Self::HoldToTalk => &hotkeys.hold_to_talk,
            Self::ToggleDictation => &hotkeys.toggle_dictation,
            Self::PasteLastTranscript => &hotkeys.paste_last_transcript,
            Self::OpenDashboard => &hotkeys.open_dashboard,
        }
    }

    pub fn set_shortcut(self, hotkeys: &mut HotkeySettings, shortcut: String) {
        match self {
            Self::HoldToTalk => hotkeys.hold_to_talk = shortcut,
            Self::ToggleDictation => hotkeys.toggle_dictation = shortcut,
            Self::PasteLastTranscript => hotkeys.paste_last_transcript = shortcut,
            Self::OpenDashboard => hotkeys.open_dashboard = shortcut,
        }
    }
}

const HOTKEY_ACTIONS: [HotkeyAction; 4] = [
    HotkeyAction::HoldToTalk,
    HotkeyAction::ToggleDictation,
    HotkeyAction::PasteLastTranscript,
    HotkeyAction::OpenDashboard,
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyStatus {
    pub bindings: Vec<HotkeyBindingStatus>,
    pub hold_release_verification_required: bool,
    pub windows_fallback_note: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyBindingStatus {
    pub action: HotkeyAction,
    pub shortcut: String,
    pub normalized_shortcut: Option<String>,
    pub registered: bool,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct HotkeyRuntimeState {
    actions_by_id: Mutex<HashMap<u32, HotkeyAction>>,
    pressed_actions: Mutex<HashSet<HotkeyAction>>,
}

impl HotkeyRuntimeState {
    fn replace_bindings(&self, actions_by_id: HashMap<u32, HotkeyAction>) {
        if let Ok(mut bindings) = self.actions_by_id.lock() {
            *bindings = actions_by_id;
        }

        if let Ok(mut pressed) = self.pressed_actions.lock() {
            pressed.clear();
        }
    }

    fn action_for(&self, shortcut: &Shortcut) -> Option<HotkeyAction> {
        self.actions_by_id
            .lock()
            .ok()
            .and_then(|bindings| bindings.get(&shortcut.id()).copied())
    }

    fn mark_pressed_once(&self, action: HotkeyAction) -> bool {
        self.pressed_actions
            .lock()
            .map(|mut pressed| pressed.insert(action))
            .unwrap_or(false)
    }

    fn mark_released_once(&self, action: HotkeyAction) -> bool {
        self.pressed_actions
            .lock()
            .map(|mut pressed| pressed.remove(&action))
            .unwrap_or(false)
    }
}

pub fn setup(app: &AppHandle, hotkeys: &HotkeySettings) -> Result<(), CommandError> {
    app.manage(HotkeyRuntimeState::default());
    validate_hotkeys(hotkeys)?;

    if let Err(error) = register_initial_hotkeys(app, hotkeys) {
        eprintln!("{}", error);
    }

    Ok(())
}

pub fn handle_shortcut(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    let Some(runtime) = app.try_state::<HotkeyRuntimeState>() else {
        return;
    };
    let Some(action) = runtime.action_for(shortcut) else {
        return;
    };

    match event.state {
        ShortcutState::Pressed => {
            if !runtime.mark_pressed_once(action) {
                return;
            }
            handle_pressed(app, action);
        }
        ShortcutState::Released => {
            if !runtime.mark_released_once(action) {
                return;
            }
            handle_released(app, action);
        }
    }
}

pub fn validate_hotkeys(hotkeys: &HotkeySettings) -> Result<(), CommandError> {
    let mut seen = HashMap::<u32, HotkeyAction>::new();

    for action in HOTKEY_ACTIONS {
        let shortcut = action.shortcut(hotkeys);
        let parsed = parse_shortcut(shortcut)?;

        if let Some(previous_action) = seen.insert(parsed.id(), action) {
            return Err(CommandError::new(
                "duplicate_hotkey",
                format!(
                    "{} is already assigned to {:?}. Choose a different hotkey.",
                    shortcut, previous_action
                ),
            ));
        }
    }

    Ok(())
}

pub fn replace_hotkeys(
    app: &AppHandle,
    previous: &HotkeySettings,
    next: &HotkeySettings,
) -> Result<(), CommandError> {
    validate_hotkeys(next)?;

    unregister_hotkey_set(app, previous);

    let mut registered = Vec::<Shortcut>::new();
    let mut actions_by_id = HashMap::<u32, HotkeyAction>::new();

    for action in HOTKEY_ACTIONS {
        let shortcut_text = action.shortcut(next);
        let shortcut = parse_shortcut(shortcut_text)?;

        if let Err(error) = app.global_shortcut().register(shortcut) {
            unregister_shortcuts(app, &registered);
            let _ = restore_previous_hotkeys(app, previous);
            return Err(CommandError::hotkey_registration_failed(
                shortcut_text,
                error,
            ));
        }

        actions_by_id.insert(shortcut.id(), action);
        registered.push(shortcut);
    }

    if let Some(runtime) = app.try_state::<HotkeyRuntimeState>() {
        runtime.replace_bindings(actions_by_id);
    }

    Ok(())
}

fn register_initial_hotkeys(app: &AppHandle, hotkeys: &HotkeySettings) -> Result<(), CommandError> {
    validate_hotkeys(hotkeys)?;

    let mut registered = Vec::<Shortcut>::new();
    let mut actions_by_id = HashMap::<u32, HotkeyAction>::new();

    for action in HOTKEY_ACTIONS {
        let shortcut_text = action.shortcut(hotkeys);
        let shortcut = parse_shortcut(shortcut_text)?;

        if let Err(error) = app.global_shortcut().register(shortcut) {
            unregister_shortcuts(app, &registered);
            return Err(CommandError::hotkey_registration_failed(
                shortcut_text,
                error,
            ));
        }

        actions_by_id.insert(shortcut.id(), action);
        registered.push(shortcut);
    }

    if let Some(runtime) = app.try_state::<HotkeyRuntimeState>() {
        runtime.replace_bindings(actions_by_id);
    }

    Ok(())
}

pub fn status(app: &AppHandle, hotkeys: &HotkeySettings) -> Result<HotkeyStatus, CommandError> {
    let mut bindings = Vec::new();

    for action in HOTKEY_ACTIONS {
        let shortcut = action.shortcut(hotkeys);
        let (normalized_shortcut, registered, error) = match parse_shortcut(shortcut) {
            Ok(parsed) => (
                Some(parsed.to_string()),
                app.global_shortcut().is_registered(parsed),
                None,
            ),
            Err(error) => (None, false, Some(error.message)),
        };

        bindings.push(HotkeyBindingStatus {
            action,
            shortcut: shortcut.to_string(),
            normalized_shortcut,
            registered,
            error,
        });
    }

    Ok(HotkeyStatus {
        bindings,
        hold_release_verification_required: true,
        windows_fallback_note:
            "Manual Windows verification is still required for hold-to-talk release events. If releases are unreliable, use a Windows-only SetWindowsHookExW(WH_KEYBOARD_LL) fallback."
                .to_string(),
    })
}

fn handle_pressed(app: &AppHandle, action: HotkeyAction) {
    let _ = app.emit("localdictate:hotkey-action", action.event_name());

    match action {
        HotkeyAction::HoldToTalk => {
            if recording_mode_allows(app, |mode| {
                matches!(mode, RecordingMode::Hold | RecordingMode::Both)
            }) {
                let _ = tray::start_dictation(app);
            }
        }
        HotkeyAction::ToggleDictation => {
            if recording_mode_allows(app, |mode| {
                matches!(mode, RecordingMode::Toggle | RecordingMode::Both)
            }) {
                let _ = toggle_dictation(app);
            }
        }
        HotkeyAction::PasteLastTranscript => {
            let _ = tray::paste_last_transcript(app);
        }
        HotkeyAction::OpenDashboard => {
            let _ = tray::open_dashboard(app, None);
        }
    }
}

fn handle_released(app: &AppHandle, action: HotkeyAction) {
    if action == HotkeyAction::HoldToTalk
        && recording_mode_allows(app, |mode| {
            matches!(mode, RecordingMode::Hold | RecordingMode::Both)
        })
    {
        let _ = tray::stop_dictation(app);
    }
}

fn toggle_dictation(app: &AppHandle) -> Result<(), CommandError> {
    let state = app.state::<BackendState>();
    let status = state.app_state()?.status().clone();

    match status {
        AppStatus::Idle | AppStatus::Ready => tray::start_dictation(app),
        AppStatus::Recording => tray::stop_dictation(app),
        _ => Ok(()),
    }
}

fn recording_mode_allows(app: &AppHandle, predicate: impl FnOnce(RecordingMode) -> bool) -> bool {
    app.try_state::<BackendState>()
        .and_then(|state| state.db().ok().and_then(|db| db.get_settings().ok()))
        .map(|settings| predicate(settings.recording_mode))
        .unwrap_or(false)
}

fn restore_previous_hotkeys(
    app: &AppHandle,
    previous: &HotkeySettings,
) -> Result<(), CommandError> {
    let mut actions_by_id = HashMap::<u32, HotkeyAction>::new();

    for action in HOTKEY_ACTIONS {
        let shortcut_text = action.shortcut(previous);
        let shortcut = parse_shortcut(shortcut_text)?;
        app.global_shortcut()
            .register(shortcut)
            .map_err(|error| CommandError::hotkey_registration_failed(shortcut_text, error))?;
        actions_by_id.insert(shortcut.id(), action);
    }

    if let Some(runtime) = app.try_state::<HotkeyRuntimeState>() {
        runtime.replace_bindings(actions_by_id);
    }

    Ok(())
}

fn unregister_hotkey_set(app: &AppHandle, hotkeys: &HotkeySettings) {
    let shortcuts = HOTKEY_ACTIONS
        .into_iter()
        .filter_map(|action| parse_shortcut(action.shortcut(hotkeys)).ok())
        .collect::<Vec<_>>();

    unregister_shortcuts(app, &shortcuts);
}

fn unregister_shortcuts(app: &AppHandle, shortcuts: &[Shortcut]) {
    for shortcut in shortcuts {
        if app.global_shortcut().is_registered(*shortcut) {
            let _ = app.global_shortcut().unregister(*shortcut);
        }
    }
}

fn parse_shortcut(shortcut: &str) -> Result<Shortcut, CommandError> {
    let normalized = normalize_shortcut(shortcut);
    Shortcut::from_str(&normalized).map_err(|error| CommandError::invalid_hotkey(shortcut, error))
}

fn normalize_shortcut(shortcut: &str) -> String {
    shortcut
        .split('+')
        .map(|token| match token.trim().to_ascii_lowercase().as_str() {
            "win" | "windows" | "meta" => "Super".to_string(),
            "control" => "Ctrl".to_string(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_windows_modifier_for_global_hotkey_parser() {
        let shortcut = parse_shortcut("Ctrl+Win+Space").unwrap();

        assert_eq!(shortcut.to_string(), "control+super+Space");
    }

    #[test]
    fn detects_duplicate_shortcuts() {
        let hotkeys = HotkeySettings {
            hold_to_talk: "Ctrl+Win+Space".to_string(),
            toggle_dictation: "Ctrl+Win+Space".to_string(),
            paste_last_transcript: "Ctrl+Alt+V".to_string(),
            open_dashboard: "Ctrl+Win+H".to_string(),
        };

        assert!(validate_hotkeys(&hotkeys).is_err());
    }
}
