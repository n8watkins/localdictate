import { useState, type ReactNode } from "react";
import {
  Archive,
  CheckCircle2,
  Clipboard,
  ClipboardPaste,
  Copy,
  Database,
  Download,
  Eraser,
  FolderOpen,
  Gauge,
  History as HistoryIcon,
  Info,
  Keyboard,
  Mic,
  MonitorCog,
  Pencil,
  Play,
  Radio,
  Search,
  Settings as SettingsIcon,
  ShieldCheck,
  SlidersHorizontal,
  Square,
  Trash2,
  type LucideIcon,
} from "lucide-react";
import "./App.css";

type ViewName =
  | "Dashboard"
  | "Transcribe"
  | "History"
  | "Settings"
  | "Hotkeys"
  | "Models"
  | "Audio"
  | "About";

const navItems: { label: ViewName; Icon: LucideIcon }[] = [
  { label: "Dashboard", Icon: Gauge },
  { label: "Transcribe", Icon: Mic },
  { label: "History", Icon: HistoryIcon },
  { label: "Settings", Icon: SettingsIcon },
  { label: "Hotkeys", Icon: Keyboard },
  { label: "Models", Icon: Database },
  { label: "Audio", Icon: Radio },
  { label: "About", Icon: Info },
];

const viewTitles: Record<ViewName, { eyebrow: string; title: string }> = {
  Dashboard: {
    eyebrow: "Dashboard",
    title: "Local speech-to-text control center",
  },
  Transcribe: {
    eyebrow: "Transcribe",
    title: "Record, review, and route the next transcript",
  },
  History: {
    eyebrow: "History",
    title: "Search and reuse local transcripts",
  },
  Settings: {
    eyebrow: "Settings",
    title: "Privacy, output, and app behavior",
  },
  Hotkeys: {
    eyebrow: "Hotkeys",
    title: "Global shortcuts and recording controls",
  },
  Models: {
    eyebrow: "Models",
    title: "Local Whisper model manager",
  },
  Audio: {
    eyebrow: "Audio",
    title: "Microphone input and recording quality",
  },
  About: {
    eyebrow: "About",
    title: "Private local dictation for Windows",
  },
};

const hotkeys = [
  { label: "Hold-to-Talk", value: "Ctrl + Win + Space", status: "Ready" },
  { label: "Toggle Dictation", value: "Ctrl + Win + D", status: "Ready" },
  { label: "Paste Last", value: "Ctrl + Alt + V", status: "Ready" },
  { label: "Open Dashboard", value: "Ctrl + Win + H", status: "Ready" },
];

const recentTranscripts = [
  {
    title: "Project status note",
    text: "The core product promise is clipboard-safe local dictation with a reusable last transcript buffer.",
    meta: "142 words | small.en-q5_1 | 10:42 AM",
    output: "Save Only",
  },
  {
    title: "Email draft",
    text: "Can you review the implementation plan and confirm which Windows paste path we should validate first?",
    meta: "31 words | small.en-q5_1 | 9:18 AM",
    output: "Auto Paste",
  },
  {
    title: "Meeting capture",
    text: "Prioritize hotkey reliability, audio normalization, and the first recording to transcription slice.",
    meta: "24 words | base.en | Yesterday",
    output: "Save Only",
  },
];

const stats = [
  { label: "Words today", value: "1,284" },
  { label: "Dictations today", value: "18" },
  { label: "Average WPM", value: "132" },
  { label: "Latency avg", value: "1.8s" },
];

const models = [
  {
    name: "small.en quantized",
    id: "small.en-q5_1",
    size: "181 MB",
    status: "Selected",
    progress: 100,
  },
  {
    name: "base.en",
    id: "base.en",
    size: "142 MB",
    status: "Downloaded",
    progress: 100,
  },
  {
    name: "medium.en quantized",
    id: "medium.en-q5_0",
    size: "514 MB",
    status: "Downloading",
    progress: 42,
  },
  {
    name: "large-v3-turbo quantized",
    id: "large-v3-turbo-q5_0",
    size: "1.6 GB",
    status: "Not Downloaded",
    progress: 0,
  },
];

function App() {
  const [activeView, setActiveView] = useState<ViewName>("Dashboard");
  const heading = viewTitles[activeView];

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">LD</div>
          <div>
            <div className="brand-name">LocalDictate</div>
            <div className="brand-subtitle">Private local dictation</div>
          </div>
        </div>

        <nav className="nav-list" aria-label="Primary">
          {navItems.map((item) => {
            const Icon = item.Icon;
            return (
              <button
                className={
                  item.label === activeView ? "nav-item active" : "nav-item"
                }
                key={item.label}
                onClick={() => setActiveView(item.label)}
                type="button"
              >
                <Icon aria-hidden="true" className="nav-icon" size={17} />
                {item.label}
              </button>
            );
          })}
        </nav>

        <div className="privacy-panel">
          <div className="privacy-status">
            <ShieldCheck aria-hidden="true" size={16} />
            Offline ready
          </div>
          <p>Audio and transcripts stay on this device after model download.</p>
        </div>
      </aside>

      <main className="dashboard">
        <header className="topbar">
          <div>
            <p className="eyebrow">{heading.eyebrow}</p>
            <h1>{heading.title}</h1>
          </div>
          <div className="topbar-actions">
            <button
              className="secondary-button"
              onClick={() => setActiveView("History")}
              type="button"
            >
              <HistoryIcon aria-hidden="true" size={16} />
              Open history
            </button>
            <button
              className="primary-button"
              onClick={() => setActiveView("Transcribe")}
              type="button"
            >
              <Mic aria-hidden="true" size={16} />
              Start dictation
            </button>
          </div>
        </header>

        {renderView(activeView, setActiveView)}
      </main>
    </div>
  );
}

function renderView(
  activeView: ViewName,
  setActiveView: (view: ViewName) => void,
) {
  switch (activeView) {
    case "Transcribe":
      return <TranscribeView />;
    case "History":
      return <HistoryView />;
    case "Settings":
      return <SettingsView />;
    case "Hotkeys":
      return <HotkeysView />;
    case "Models":
      return <ModelsView />;
    case "Audio":
      return <AudioView />;
    case "About":
      return <AboutView />;
    case "Dashboard":
    default:
      return <DashboardView setActiveView={setActiveView} />;
  }
}

function DashboardView({
  setActiveView,
}: {
  setActiveView: (view: ViewName) => void;
}) {
  return (
    <>
      <section className="status-grid" aria-label="Current setup">
        <StatusCard
          action="Record"
          Icon={Gauge}
          label="Current status"
          onAction={() => setActiveView("Transcribe")}
          status={<span className="pill ready">Ready</span>}
          value="Idle"
        />
        <StatusCard
          action="Choose"
          Icon={Mic}
          label="Active microphone"
          onAction={() => setActiveView("Audio")}
          status={<span className="status-dot success" />}
          value="Default communications device"
        />
        <StatusCard
          action="Manage"
          Icon={Database}
          label="Active model"
          onAction={() => setActiveView("Models")}
          status={<span className="pill selected">Selected</span>}
          value="small.en quantized"
        />
        <StatusCard
          action="Change"
          Icon={Clipboard}
          label="Output mode"
          onAction={() => setActiveView("Settings")}
          status={<span className="pill preserve">Clipboard Untouched</span>}
          value="Save Only"
        />
      </section>

      <section className="main-grid">
        <LastTranscriptCard />

        <article className="panel-card">
          <div className="section-heading compact">
            <h2>Hotkeys</h2>
            <button
              className="ghost-button"
              onClick={() => setActiveView("Hotkeys")}
              type="button"
            >
              <Keyboard aria-hidden="true" size={15} />
              Rebind
            </button>
          </div>
          <HotkeyList compact />
        </article>

        <RecentTranscriptsCard setActiveView={setActiveView} />

        <StatsCard />
      </section>
    </>
  );
}

function TranscribeView() {
  return (
    <section className="split-grid">
      <article className="buffer-card">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Recording</p>
            <h2>Push-to-talk capture</h2>
          </div>
          <span className="pill ready">Idle</span>
        </div>

        <div className="recording-stage">
          <Waveform />
          <div>
            <strong>Ready for dictation</strong>
            <p className="muted">
              Hold Ctrl + Win + Space or use toggle mode. Backend recording will
              stream input level here once the audio service is wired.
            </p>
          </div>
        </div>

        <div className="button-row">
          <button className="primary-button" type="button">
            <Mic aria-hidden="true" size={16} />
            Start recording
          </button>
          <button className="secondary-button" type="button">
            <Square aria-hidden="true" size={15} />
            Stop and transcribe
          </button>
          <button className="ghost-button" type="button">
            <Eraser aria-hidden="true" size={15} />
            Cancel
          </button>
        </div>
      </article>

      <div className="stack">
        <article className="panel-card">
          <div className="section-heading compact">
            <h2>Output behavior</h2>
            <span className="pill preserve">Clipboard Untouched</span>
          </div>
          <SegmentedControl
            options={["Save Only", "Auto Paste", "Copy", "Copy + Paste"]}
            selected="Save Only"
          />
        </article>

        <article className="panel-card">
          <div className="section-heading compact">
            <h2>Paste method</h2>
          </div>
          <SegmentedControl
            options={["Direct Insert", "Compatibility Paste"]}
            selected="Direct Insert"
          />
        </article>

        <LastTranscriptCard compact />
      </div>
    </section>
  );
}

function HistoryView() {
  return (
    <section className="view-grid">
      <article className="panel-card span-2">
        <div className="toolbar-row">
          <div className="search-field">
            <Search aria-hidden="true" size={16} />
            <input aria-label="Search transcripts" placeholder="Search transcripts" />
          </div>
          <select aria-label="Retention">
            <option>30 day retention</option>
            <option>7 day retention</option>
            <option>90 day retention</option>
            <option>Forever</option>
          </select>
          <button className="secondary-button" type="button">
            <Trash2 aria-hidden="true" size={15} />
            Clear all
          </button>
        </div>
      </article>

      <article className="panel-card span-2">
        <div className="section-heading compact">
          <h2>Transcript archive</h2>
          <Archive aria-hidden="true" size={16} />
          <span className="muted">3 local records</span>
        </div>
        <div className="transcript-list">
          {recentTranscripts.map((item) => (
            <TranscriptRow item={item} key={item.title} variant="full" />
          ))}
        </div>
      </article>
    </section>
  );
}

function SettingsView() {
  return (
    <section className="view-grid">
      <SectionPanel icon={<ShieldCheck aria-hidden="true" size={16} />} title="Privacy defaults">
        <SettingRow
          description="Keep searchable local transcript records."
          label="History enabled"
        >
          <Toggle defaultOn label="History enabled" />
        </SettingRow>
        <SettingRow
          description="Store source clips beside transcript metadata."
          label="Save raw audio clips"
        >
          <Toggle label="Save raw audio clips" />
        </SettingRow>
        <SettingRow description="Automatically delete old history." label="Retention">
          <select defaultValue="30">
            <option value="7">7 days</option>
            <option value="30">30 days</option>
            <option value="90">90 days</option>
            <option value="forever">Forever</option>
          </select>
        </SettingRow>
        <SettingRow description="Speech recognition language preference." label="Language">
          <select defaultValue="en">
            <option value="auto">Auto detect</option>
            <option value="en">English</option>
          </select>
        </SettingRow>
      </SectionPanel>

      <SectionPanel icon={<MonitorCog aria-hidden="true" size={16} />} title="App behavior">
        <SettingRow description="Start LocalDictate when Windows starts." label="Launch at startup">
          <Toggle defaultOn label="Launch at startup" />
        </SettingRow>
        <SettingRow description="Keep the app available from the system tray." label="Minimize to tray">
          <Toggle defaultOn label="Minimize to tray" />
        </SettingRow>
        <SettingRow description="Show capture state near the cursor." label="Show floating pill">
          <Toggle defaultOn label="Show floating pill" />
        </SettingRow>
        <SettingRow description="Display completion and failure notices." label="Notifications">
          <Toggle defaultOn label="Notifications" />
        </SettingRow>
        <SettingRow description="Play start and stop capture tones." label="Sounds">
          <Toggle label="Sounds" />
        </SettingRow>
      </SectionPanel>

      <SectionPanel icon={<SlidersHorizontal aria-hidden="true" size={16} />} title="Recording rules">
        <SettingRow description="Choose which global capture modes are active." label="Recording mode">
          <SegmentedControl
            options={["Hold", "Toggle", "Both"]}
            selected="Both"
          />
        </SettingRow>
        <SettingRow description="Trim leading and trailing quiet segments." label="Silence trim">
          <Toggle defaultOn label="Silence trim" />
        </SettingRow>
        <SettingRow description="Ignore accidental taps shorter than this." label="Minimum duration">
          <input defaultValue="300 ms" />
        </SettingRow>
        <SettingRow description="Stop long recordings automatically." label="Maximum duration">
          <input defaultValue="3 minutes" />
        </SettingRow>
      </SectionPanel>

      <SectionPanel icon={<MonitorCog aria-hidden="true" size={16} />} title="Data controls">
        <div className="button-column">
          <button className="secondary-button" type="button">
            <FolderOpen aria-hidden="true" size={15} />
            Open local data folder
          </button>
          <button className="secondary-button" type="button">
            <Eraser aria-hidden="true" size={15} />
            Clear Last Transcript Buffer
          </button>
          <button className="ghost-button danger" type="button">
            <Trash2 aria-hidden="true" size={15} />
            Reset all settings
          </button>
        </div>
      </SectionPanel>
    </section>
  );
}

function HotkeysView() {
  return (
    <section className="view-grid">
      <article className="panel-card span-2">
        <div className="section-heading compact">
          <h2>Registered global hotkeys</h2>
          <CheckCircle2 aria-hidden="true" size={16} />
          <span className="pill ready">All valid</span>
        </div>
        <div className="hotkey-editor-list">
          {hotkeys.map((hotkey) => (
            <div className="hotkey-editor-row" key={hotkey.label}>
              <div>
                <strong>{hotkey.label}</strong>
                <span>Registered globally</span>
              </div>
              <kbd>{hotkey.value}</kbd>
              <span className="pill ready">{hotkey.status}</span>
              <button className="secondary-button" type="button">
                <Keyboard aria-hidden="true" size={15} />
                Rebind
              </button>
            </div>
          ))}
        </div>
      </article>

      <article className="panel-card">
        <div className="section-heading compact">
          <h2>Capture behavior</h2>
        </div>
        <SegmentedControl
          options={["Hold-to-talk", "Toggle", "Both enabled"]}
          selected="Both enabled"
        />
      </article>

      <article className="panel-card">
        <div className="section-heading compact">
          <h2>Conflict handling</h2>
        </div>
        <div className="conflict-panel">
          <CheckCircle2 aria-hidden="true" size={16} />
          <span>No shortcut conflicts detected.</span>
        </div>
      </article>
    </section>
  );
}

function ModelsView() {
  return (
    <section className="view-grid">
      <article className="panel-card span-2">
        <div className="section-heading compact">
          <h2>Whisper models</h2>
          <button className="secondary-button" type="button">
            <FolderOpen aria-hidden="true" size={15} />
            Open model folder
          </button>
        </div>
        <div className="model-table">
          <div className="model-table-header" aria-hidden="true">
            <span>Model</span>
            <span>Size</span>
            <span>Status</span>
            <span>Action</span>
          </div>
          {models.map((model) => (
            <div className="model-row" key={model.id}>
              <div>
                <strong>{model.name}</strong>
                <span>{model.id}</span>
                <div className="progress-track">
                  <div style={{ width: `${model.progress}%` }} />
                </div>
              </div>
              <span>{model.size}</span>
              <span
                className={
                  model.status === "Selected" ? "pill selected" : "pill preserve"
                }
              >
                {model.status}
              </span>
              <div className="row-actions">
                {model.progress === 0 ? (
                  <button className="secondary-button" type="button">
                    <Download aria-hidden="true" size={15} />
                    Download
                  </button>
                ) : null}
                {model.progress > 0 && model.progress < 100 ? (
                  <button className="secondary-button" type="button">
                    <Square aria-hidden="true" size={15} />
                    Cancel
                  </button>
                ) : null}
                {model.progress === 100 ? (
                  <button className="secondary-button" type="button">
                    <CheckCircle2 aria-hidden="true" size={15} />
                    Select
                  </button>
                ) : null}
                {model.progress === 100 ? (
                  <IconButton danger label="Delete">
                    <Trash2 aria-hidden="true" size={15} />
                  </IconButton>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      </article>

      <article className="panel-card">
        <div className="section-heading compact">
          <h2>Default model</h2>
        </div>
        <strong className="standout">small.en quantized</strong>
        <p className="muted">
          Balanced quality and speed for daily English dictation.
        </p>
      </article>

      <article className="panel-card">
        <div className="section-heading compact">
          <h2>Storage</h2>
        </div>
        <code>%APPDATA%/LocalDictate/models/</code>
        <div className="button-row">
          <button className="secondary-button" type="button">
            <FolderOpen aria-hidden="true" size={15} />
            Open folder
          </button>
        </div>
      </article>
    </section>
  );
}

function AudioView() {
  return (
    <section className="split-grid">
      <article className="buffer-card">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Input</p>
            <h2>Default communications device</h2>
          </div>
          <span className="status-dot success" />
        </div>

        <Waveform />
        <div className="meter">
          <div />
        </div>

        <div className="control-grid">
          <label>
            Microphone
            <select defaultValue="default">
              <option value="default">Default communications device</option>
              <option value="usb">USB microphone</option>
              <option value="array">Microphone array</option>
            </select>
          </label>
          <label>
            Target format
            <input readOnly value="16 kHz mono PCM WAV" />
          </label>
        </div>

        <div className="button-row">
          <button className="primary-button" type="button">
            <Mic aria-hidden="true" size={16} />
            Test recording
          </button>
          <button className="secondary-button" type="button">
            <Play aria-hidden="true" size={15} />
            Play test
          </button>
        </div>
      </article>

      <div className="stack">
        <SectionPanel title="Audio processing">
          <SettingRow description="Remove quiet space around speech." label="Silence trim">
            <Toggle defaultOn label="Silence trim" />
          </SettingRow>
          <SettingRow description="Ignore captures below this length." label="Minimum duration">
            <input defaultValue="300 ms" />
          </SettingRow>
          <SettingRow description="Cap single dictation sessions." label="Maximum duration">
            <input defaultValue="3 minutes" />
          </SettingRow>
          <SettingRow description="Preferred file shape for transcription." label="Target format">
            <select defaultValue="wav">
              <option value="wav">16 kHz mono PCM WAV</option>
              <option value="flac">16 kHz mono FLAC</option>
            </select>
          </SettingRow>
          <SettingRow description="Keep original clips for review." label="Save raw audio">
            <Toggle label="Save raw audio" />
          </SettingRow>
        </SectionPanel>
        <article className="panel-card">
          <div className="section-heading compact">
            <h2>Device health</h2>
            <span className="pill ready">Available</span>
          </div>
          <p className="muted">
            Permission, unavailable device, and recording failure states will
            surface here from the Rust audio service.
          </p>
        </article>
      </div>
    </section>
  );
}

function AboutView() {
  return (
    <section className="view-grid">
      <article className="buffer-card span-2">
        <div className="section-heading">
          <div>
            <p className="eyebrow">LocalDictate</p>
            <h2>Dictate locally without consuming your clipboard</h2>
          </div>
          <span className="pill preserve">Local-first</span>
        </div>
        <p className="transcript-text">
          LocalDictate is a Windows tray utility for private speech-to-text. It
          records when you press a global hotkey, transcribes locally with
          Whisper, stores the result in a Last Transcript Buffer, and lets you
          insert it later without permanently overwriting the system clipboard.
        </p>
      </article>

      <SectionPanel title="App details">
        <SettingRow description="Current packaged application version." label="Version">
          <strong>0.1.0</strong>
        </SettingRow>
        <SettingRow description="Transcription runs locally after model download." label="Privacy">
          <span className="pill preserve">Local-first</span>
        </SettingRow>
        <SettingRow description="Default location for app data and models." label="Local data path">
          <code>%APPDATA%/LocalDictate/</code>
        </SettingRow>
      </SectionPanel>
      <SectionPanel title="Resources">
        <div className="button-column">
          <button className="secondary-button" type="button">
            <FolderOpen aria-hidden="true" size={15} />
            Open docs
          </button>
          <button className="secondary-button" type="button">
            <Archive aria-hidden="true" size={15} />
            View licenses
          </button>
        </div>
      </SectionPanel>
    </section>
  );
}

function LastTranscriptCard({ compact = false }: { compact?: boolean }) {
  return (
    <article className={compact ? "panel-card" : "buffer-card"}>
      <div className="section-heading">
        <div>
          <p className="eyebrow">Last Transcript Buffer</p>
          <h2>Ready to insert later</h2>
        </div>
        <span className="pill preserve">Clipboard Preserved</span>
      </div>

      <p className={compact ? "transcript-text compact-text" : "transcript-text"}>
        This is the most recent dictated text. It is stored inside LocalDictate,
        separate from the system clipboard, and can be inserted with the
        paste-last hotkey.
      </p>

      <div className="metadata-row">
        <span>42 words</span>
        <span>231 chars</span>
        <span>8.4s audio</span>
        <span>small.en-q5_1</span>
      </div>

      <div className="button-row">
        <button className="primary-button" type="button">
          <ClipboardPaste aria-hidden="true" size={16} />
          Insert
        </button>
        <button className="secondary-button" type="button">
          <Pencil aria-hidden="true" size={15} />
          Edit
        </button>
        <button className="secondary-button" type="button">
          <Copy aria-hidden="true" size={15} />
          Copy
        </button>
        <button className="ghost-button" type="button">
          <Eraser aria-hidden="true" size={15} />
          Clear
        </button>
      </div>
    </article>
  );
}

function RecentTranscriptsCard({
  setActiveView,
}: {
  setActiveView: (view: ViewName) => void;
}) {
  return (
    <article className="panel-card recent-card">
      <div className="section-heading compact">
        <h2>Recent Transcripts</h2>
        <button
          className="ghost-button"
          onClick={() => setActiveView("History")}
          type="button"
        >
          <Search aria-hidden="true" size={15} />
          Search
        </button>
      </div>
      <div className="transcript-list">
        {recentTranscripts.slice(0, 3).map((item) => (
          <TranscriptRow item={item} key={item.title} variant="compact" />
        ))}
      </div>
    </article>
  );
}

function StatsCard() {
  return (
    <article className="panel-card">
      <div className="section-heading compact">
        <h2>Basic Stats</h2>
        <span className="muted">Today</span>
      </div>
      <div className="stats-grid">
        {stats.map((stat) => (
          <div className="stat-tile" key={stat.label}>
            <span>{stat.label}</span>
            <strong>{stat.value}</strong>
          </div>
        ))}
      </div>
    </article>
  );
}

function HotkeyList({ compact = false }: { compact?: boolean }) {
  return (
    <div className={compact ? "hotkey-list compact-list" : "hotkey-list"}>
      {hotkeys.map((hotkey) => (
        <div className="hotkey-row" key={hotkey.label}>
          <span>{hotkey.label}</span>
          <kbd>{hotkey.value}</kbd>
        </div>
      ))}
    </div>
  );
}

function StatusCard({
  action,
  Icon,
  label,
  onAction,
  status,
  value,
}: {
  action: string;
  Icon: LucideIcon;
  label: string;
  onAction: () => void;
  status: ReactNode;
  value: string;
}) {
  return (
    <article className="metric-card status-card">
      <div className="card-header">
        <span>
          <Icon aria-hidden="true" size={15} />
          {label}
        </span>
        {status}
      </div>
      <strong>{value}</strong>
      <button className="ghost-button" onClick={onAction} type="button">
        {action}
      </button>
    </article>
  );
}

function SectionPanel({
  children,
  icon,
  title,
}: {
  children: ReactNode;
  icon?: ReactNode;
  title: string;
}) {
  return (
    <article className="panel-card">
      <div className="section-heading compact">
        <h2>{title}</h2>
        {icon}
      </div>
      <div className="settings-list">{children}</div>
    </article>
  );
}

function SettingRow({
  children,
  description,
  label,
}: {
  children: ReactNode;
  description: string;
  label: string;
}) {
  return (
    <div className="settings-row">
      <span>
        <strong>{label}</strong>
        <small>{description}</small>
      </span>
      <div className="setting-control">{children}</div>
    </div>
  );
}

function TranscriptRow({
  item,
  variant,
}: {
  item: (typeof recentTranscripts)[number];
  variant: "compact" | "full";
}) {
  const isFull = variant === "full";

  return (
    <div className={isFull ? "history-row" : "transcript-row"}>
      <div>
        <strong>{item.title}</strong>
        <p>{item.text}</p>
        <span>{item.meta}</span>
      </div>
      <div className="row-actions">
        {isFull ? <span className="pill preserve">{item.output}</span> : null}
        <button className={isFull ? "ghost-button" : "compact-action"} type="button">
          <ClipboardPaste aria-hidden="true" size={15} />
          Insert
        </button>
        <button className={isFull ? "ghost-button" : "compact-action"} type="button">
          <Copy aria-hidden="true" size={15} />
          Copy
        </button>
        {isFull ? (
          <>
            <button className="ghost-button" type="button">
              <Pencil aria-hidden="true" size={15} />
              Edit
            </button>
            <button className="ghost-button danger" type="button">
              <Trash2 aria-hidden="true" size={15} />
              Delete
            </button>
          </>
        ) : null}
      </div>
    </div>
  );
}

function Toggle({
  defaultOn = false,
  disabled = false,
  label,
}: {
  defaultOn?: boolean;
  disabled?: boolean;
  label: string;
}) {
  const [enabled, setEnabled] = useState(defaultOn);

  return (
    <button
      aria-label={label}
      aria-pressed={enabled}
      className={enabled ? "toggle is-on" : "toggle"}
      disabled={disabled}
      onClick={() => setEnabled((current) => !current)}
      type="button"
    >
      <span />
    </button>
  );
}

function IconButton({
  children,
  danger = false,
  label,
}: {
  children: ReactNode;
  danger?: boolean;
  label: string;
}) {
  return (
    <button
      aria-label={label}
      className={danger ? "icon-button danger" : "icon-button"}
      title={label}
      type="button"
    >
      {children}
    </button>
  );
}

function SegmentedControl({
  options,
  selected,
}: {
  options: string[];
  selected: string;
}) {
  const [active, setActive] = useState(selected);

  return (
    <div className="segmented-control">
      {options.map((option) => (
        <button
          aria-pressed={option === active}
          className={option === active ? "active-segment" : ""}
          key={option}
          onClick={() => setActive(option)}
          type="button"
        >
          {option}
        </button>
      ))}
    </div>
  );
}

function Waveform() {
  return (
    <div className="recording-visual" aria-hidden="true">
      <span />
      <span />
      <span />
      <span />
      <span />
      <span />
      <span />
    </div>
  );
}

export default App;
