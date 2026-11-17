import { restoreConfigStorage } from '@/desktop/configStorage';

/**
 * Boot gate for the desktop .exe: app data lives in the config folder
 * (Music\BeatFlow\config\app-storage.json), not the WebView2 browser
 * profile. Seeding localStorage from the file must finish before any store
 * hydrates — zustand persist hydrates at import time — so the real entry
 * loads only after the restore. In the browser this resolves immediately.
 */
const boot = async (): Promise<void> => {
  await restoreConfigStorage();
  await import('./bootstrap');
};

void boot();
