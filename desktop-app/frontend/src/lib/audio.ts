let ctx: AudioContext | null = null;

function getCtx(): AudioContext {
  if (!ctx) ctx = new AudioContext();
  return ctx;
}

export function playTone(freq: number, duration: number, type: OscillatorType = 'sine', volume = 0.15) {
  try {
    const c = getCtx();
    const osc = c.createOscillator();
    const gain = c.createGain();
    osc.type = type;
    osc.frequency.setValueAtTime(freq, c.currentTime);
    gain.gain.setValueAtTime(volume, c.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, c.currentTime + duration);
    osc.connect(gain);
    gain.connect(c.destination);
    osc.start();
    osc.stop(c.currentTime + duration);
  } catch {
    // Audio not available
  }
}

export function playAlertConnect() {
  playTone(880, 0.15, 'sine', 0.1);
}

export function playAlertDisconnect() {
  playTone(220, 0.3, 'sawtooth', 0.12);
  setTimeout(() => playTone(180, 0.4, 'sawtooth', 0.1), 300);
}

export function playAlertEstop() {
  for (let i = 0; i < 3; i++) {
    setTimeout(() => playTone(660, 0.15, 'square', 0.2), i * 200);
  }
}

export function playAlertFault() {
  playTone(440, 0.1, 'triangle', 0.08);
  setTimeout(() => playTone(550, 0.1, 'triangle', 0.08), 150);
}

export function playAlertCommandSent() {
  playTone(1200, 0.05, 'sine', 0.06);
}
