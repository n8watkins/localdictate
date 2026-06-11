import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  currentMonitor,
  getCurrentWindow,
  PhysicalPosition,
} from "@tauri-apps/api/window";
import { Square } from "lucide-react";
import {
  getAppState,
  getSettings,
  stopRecording,
  transcribeRecording,
  updateSettings,
  type AppSettings,
  type AppStateSnapshot,
  type AppStatus,
} from "./backend";
import "./pill.css";

const VISIBLE_STATUSES: ReadonlySet<AppStatus> = new Set([
  "Recording",
  "Stopping",
  "Transcribing",
  "Pasting",
  "Ready",
  "Error",
]);

const READY_HIDE_DELAY_MS = 5000;
const MOVE_PERSIST_DEBOUNCE_MS = 600;
const BOTTOM_MARGIN_PX = 90;

function pillTone(status: AppStatus) {
  switch (status) {
    case "Recording":
      return "recording";
    case "Stopping":
    case "Transcribing":
    case "Pasting":
      return "pending";
    case "Ready":
      return "ready";
    case "Error":
      return "error";
    default:
      return "idle";
  }
}

function pillLines(appState: AppStateSnapshot): [string, string] {
  switch (appState.status) {
    case "Recording":
      return ["Recording...", "Click to stop · drag to move"];
    case "Stopping":
      return ["Saving audio...", "Preparing transcription"];
    case "Transcribing":
      return ["Transcribing...", "Whisper is running locally"];
    case "Pasting":
      return ["Inserting transcript...", "Clipboard preserved"];
    case "Ready":
      return ["Transcript ready", "Saved to Last Transcript"];
    case "Error":
      return ["Needs attention", appState.error?.message ?? "Check LocalDictate"];
    default:
      return ["Ready for dictation", ""];
  }
}

function PillApp() {
  const [appState, setAppState] = useState<AppStateSnapshot | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [stopping, setStopping] = useState(false);
  const settingsRef = useRef<AppSettings | null>(null);
  const positionedRef = useRef(false);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;

    const loadInitial = async () => {
      try {
        const [state, loadedSettings] = await Promise.all([
          getAppState(),
          getSettings(),
        ]);
        if (!disposed) {
          setAppState(state);
          setSettings(loadedSettings);
        }
      } catch {
        // Backend not ready yet; the window stays hidden until events arrive.
      }
    };

    const setup = async () => {
      const stop = await listen<AppStateSnapshot>(
        "localdictate:app-state",
        (event) => {
          setAppState(event.payload);
          void getSettings()
            .then((latest) => {
              if (!disposed) {
                setSettings(latest);
              }
            })
            .catch(() => {
              // Keep the last known settings.
            });
        },
      );
      unlisten = stop;
      if (disposed) {
        stop();
      }
    };

    void loadInitial();
    void setup();

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  // Persist the pill position after the user drags it.
  useEffect(() => {
    const pillWindow = getCurrentWindow();
    let disposed = false;
    let timer: number | null = null;
    let unlisten: (() => void) | null = null;

    void pillWindow
      .onMoved((event) => {
        if (timer !== null) {
          window.clearTimeout(timer);
        }
        const { x, y } = event.payload;
        timer = window.setTimeout(() => {
          const current = settingsRef.current;
          if (!current || (current.pillX === x && current.pillY === y)) {
            return;
          }
          const next = { ...current, pillX: x, pillY: y };
          settingsRef.current = next;
          setSettings(next);
          void updateSettings(next).catch(() => {
            // Position persistence is best-effort.
          });
        }, MOVE_PERSIST_DEBOUNCE_MS);
      })
      .then((stop) => {
        unlisten = stop;
        if (disposed) {
          stop();
        }
      });

    return () => {
      disposed = true;
      if (timer !== null) {
        window.clearTimeout(timer);
      }
      unlisten?.();
    };
  }, []);

  const status = appState?.status ?? null;
  const showPill = settings?.showFloatingPill ?? false;
  const pillX = settings?.pillX ?? null;
  const pillY = settings?.pillY ?? null;
  const updatedAt = appState?.updatedAt ?? null;

  // Show/hide the native window to match app state.
  useEffect(() => {
    const pillWindow = getCurrentWindow();
    let hideTimer: number | null = null;

    if (!status || !showPill || !VISIBLE_STATUSES.has(status)) {
      void pillWindow.hide().catch(() => {});
      return;
    }

    const show = async () => {
      try {
        if (!positionedRef.current) {
          positionedRef.current = true;
          if (typeof pillX === "number" && typeof pillY === "number") {
            await pillWindow.setPosition(new PhysicalPosition(pillX, pillY));
          } else {
            const monitor = await currentMonitor();
            if (monitor) {
              const size = await pillWindow.outerSize();
              const x = Math.round(
                monitor.position.x + (monitor.size.width - size.width) / 2,
              );
              const y = Math.round(
                monitor.position.y +
                  monitor.size.height -
                  size.height -
                  BOTTOM_MARGIN_PX,
              );
              await pillWindow.setPosition(new PhysicalPosition(x, y));
            }
          }
        }
        await pillWindow.show();
      } catch {
        // Window management is unavailable outside Tauri.
      }
    };

    void show();

    if (status === "Ready") {
      hideTimer = window.setTimeout(() => {
        void pillWindow.hide().catch(() => {});
      }, READY_HIDE_DELAY_MS);
    }

    return () => {
      if (hideTimer !== null) {
        window.clearTimeout(hideTimer);
      }
    };
  }, [status, showPill, pillX, pillY, updatedAt]);

  const handleStop = useCallback(async () => {
    setStopping(true);

    try {
      const recording = await stopRecording();
      if (recording.status === "completed" || recording.status === "timed_out") {
        await transcribeRecording(recording);
      }
    } catch {
      // Errors surface in the main window via audio://recording-error.
    } finally {
      setStopping(false);
    }
  }, []);

  if (!appState) {
    return null;
  }

  const [title, subtitle] = pillLines(appState);
  const isRecording = appState.status === "Recording";

  return (
    <div
      className={`pill-shell ${pillTone(appState.status)}`}
      data-tauri-drag-region
    >
      <span aria-hidden="true" className="pill-pulse" data-tauri-drag-region />
      <div className="pill-text" data-tauri-drag-region>
        <strong data-tauri-drag-region>{title}</strong>
        {subtitle ? <span data-tauri-drag-region>{subtitle}</span> : null}
      </div>
      {isRecording ? (
        <button
          aria-label="Stop recording"
          className="pill-stop"
          disabled={stopping}
          onClick={() => void handleStop()}
          title="Stop recording"
          type="button"
        >
          <Square aria-hidden="true" size={13} />
        </button>
      ) : null}
    </div>
  );
}

export default PillApp;
