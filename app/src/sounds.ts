// Lightweight audio cues generated with the Web Audio API (no asset files).

let sharedContext: AudioContext | null = null;

function getAudioContext(): AudioContext | null {
  try {
    if (!sharedContext) {
      sharedContext = new AudioContext();
    }

    if (sharedContext.state === "suspended") {
      void sharedContext.resume();
    }

    return sharedContext;
  } catch {
    return null;
  }
}

const MAX_GAIN = 0.15;

function playTone(
  context: AudioContext,
  frequency: number,
  startAt: number,
  duration: number,
) {
  const oscillator = context.createOscillator();
  const gain = context.createGain();

  oscillator.type = "sine";
  oscillator.frequency.setValueAtTime(frequency, startAt);

  // Gentle attack/release envelope to avoid clicks.
  gain.gain.setValueAtTime(0.0001, startAt);
  gain.gain.exponentialRampToValueAtTime(MAX_GAIN, startAt + 0.012);
  gain.gain.exponentialRampToValueAtTime(0.0001, startAt + duration);

  oscillator.connect(gain);
  gain.connect(context.destination);
  oscillator.start(startAt);
  oscillator.stop(startAt + duration + 0.02);
}

/** Short two-tone rising blip played when recording starts. */
export function playStartCue() {
  const context = getAudioContext();
  if (!context) {
    return;
  }

  const now = context.currentTime;
  playTone(context, 660, now, 0.055);
  playTone(context, 880, now + 0.055, 0.065);
}

/** Falling blip played when recording stops. */
export function playStopCue() {
  const context = getAudioContext();
  if (!context) {
    return;
  }

  const now = context.currentTime;
  playTone(context, 880, now, 0.055);
  playTone(context, 440, now + 0.055, 0.075);
}
