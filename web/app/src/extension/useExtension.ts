import { desktopBridge, isDesktopApp, type DetectedExtension } from '@bitchord/shared/bridge';
import { queryClient } from '@/lib/queryClient';

export type ExtensionStatus =
  | { state: 'checking' }
  | { state: 'missing' }
  | { state: 'present'; extension: DetectedExtension };

export const extensionQueryKey = ['extension', 'detect'] as const;

/** The native Rust engine stands in for the extension on desktop. */
const DESKTOP_EXTENSION: DetectedExtension = {
  bridge: desktopBridge(),
  info: {
    version: '1.9.2',
    protocol: 1,
    browser: 'chromium',
    capabilities: ['yt', 'yt.auth', 'yt.stats', 'addon', 'lyrics', 'spotify', 'discord'],
  },
};

if (isDesktopApp()) {
  queryClient.setQueryData(extensionQueryKey, DESKTOP_EXTENSION);
}

/**
 * The browser extension is gone; the desktop .exe is the only client with a
 * bridge (the Rust engine answers every call). On desktop the stand-in is
 * always present; elsewhere there is nothing to reach.
 */
export function useExtension(): ExtensionStatus {
  if (!isDesktopApp()) return { state: 'missing' };
  if (!queryClient.getQueryData(extensionQueryKey)) {
    queryClient.setQueryData(extensionQueryKey, DESKTOP_EXTENSION);
  }
  return { state: 'present', extension: DESKTOP_EXTENSION };
}
