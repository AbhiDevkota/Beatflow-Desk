import { desktopBridge, isDesktopApp } from '@bitchord/shared/bridge';

/**
 * The desktop app's own state (settings, theme, lyrics prefs, …) lives in the
 * config folder — `Music\BeatFlow\config\app-storage.json` — not in the
 * WebView2 browser profile, so clearing the browser container keeps every
 * setting and the file sits beside the other config files (addons.json,
 * yt_auth.json).
 *
 * localStorage stays the runtime store: zustand persist and the direct
 * `localStorage.getItem` callers keep their sync semantics. Every write is
 * mirrored into the config file, and on boot the file fills whatever
 * localStorage is missing (first run, cleared profile, another machine), with
 * the merged snapshot written back — which migrates existing profiles on the
 * first boot after this ships. The browser build keeps plain localStorage.
 */

const SAVE_DEBOUNCE_MS = 500;

/** The untouched originals, kept so boot seeding doesn't feed the mirror. */
let rawSetItem: ((key: string, value: string) => void) | null = null;
let rawRemoveItem: ((key: string) => void) | null = null;
let rawClear: (() => void) | null = null;

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let saveInFlight = false;
let saveDirty = false;
let installed = false;

/** The current localStorage, as the config file should hold it. */
function snapshot(): Record<string, string> {
  const data: Record<string, string> = {};
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key !== null) {
      const value = localStorage.getItem(key);
      if (value !== null) data[key] = value;
    }
  }
  return data;
}

async function saveNow(): Promise<void> {
  saveTimer = null;
  if (saveInFlight) {
    saveDirty = true; // a newer change landed mid-save; run again after it
    return;
  }
  saveInFlight = true;
  try {
    await desktopBridge().call('storage.save', { data: snapshot() });
  } catch (err) {
    console.warn('[ConfigStorage] Failed to save app-storage.json:', err);
  } finally {
    saveInFlight = false;
    if (saveDirty) {
      saveDirty = false;
      scheduleSave(0);
    }
  }
}

function scheduleSave(delay: number = SAVE_DEBOUNCE_MS): void {
  if (saveTimer !== null) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    void saveNow();
  }, delay);
}

/** A pending debounce is lost when the window closes — flush on hide. */
function flushOnHide(): void {
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'hidden' && saveTimer !== null) void saveNow();
  });
  window.addEventListener('pagehide', () => {
    if (saveTimer !== null) void saveNow();
  });
}

/**
 * Route every localStorage write through the mirror. The prototype is the
 * only seam — `window.localStorage` itself is unforgeable — and the
 * `this` check leaves sessionStorage untouched.
 */
function patchStorage(): void {
  if (rawSetItem !== null) return;
  const proto = Storage.prototype;
  rawSetItem = proto.setItem;
  rawRemoveItem = proto.removeItem;
  rawClear = proto.clear;

  proto.setItem = function (key, value) {
    rawSetItem!.call(this, key, value);
    if (this === window.localStorage) scheduleSave();
  };
  proto.removeItem = function (key) {
    rawRemoveItem!.call(this, key);
    if (this === window.localStorage) scheduleSave();
  };
  proto.clear = function () {
    rawClear!.call(this);
    if (this === window.localStorage) scheduleSave();
  };
}

/**
 * Boot gate for the desktop app: seed localStorage from the config file and
 * mirror every further write into it. Must run before any store hydrates,
 * so the entry point awaits this before importing the app bundle.
 */
export async function restoreConfigStorage(): Promise<void> {
  if (installed || !isDesktopApp()) return;
  installed = true;

  patchStorage();
  flushOnHide();

  let disk: Record<string, string> = {};
  try {
    disk = (await desktopBridge().call('storage.load', {})) as Record<string, string>;
  } catch (err) {
    console.warn('[ConfigStorage] Failed to load app-storage.json:', err);
  }
  if (typeof disk !== 'object' || disk === null) disk = {};

  let seeded = false;
  for (const [key, value] of Object.entries(disk)) {
    if (window.localStorage.getItem(key) === null) {
      rawSetItem!.call(window.localStorage, key, value);
      seeded = true;
    }
  }

  // localStorage wins where both sides hold a key (the browser profile had
  // the data until now); the merged snapshot becomes the file's content,
  // seeding it on first run and migrating older profiles.
  scheduleSave(seeded ? 0 : SAVE_DEBOUNCE_MS);
}
