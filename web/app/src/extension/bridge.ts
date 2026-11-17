import { isDesktopApp, desktopBridge, type Bridge } from '@bitchord/shared/bridge';

/** The desktop host's bridge, for code outside React; null outside the .exe. */
export function getBridge(): Bridge | null {
  return isDesktopApp() ? desktopBridge() : null;
}
