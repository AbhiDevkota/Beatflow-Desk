/** Build-time configuration, from Vite's `VITE_*` environment variables. */
export const config = {
  androidAppUrl: import.meta.env.VITE_ANDROID_APP_URL || 'https://github.com/AbhiDevkota/BeatFlow/releases/latest',
  /** BitChord's Listen Together server. Defaults to beatflow-jam-together on Render if not specified. */
  partyServerUrl: import.meta.env.VITE_PARTY_SERVER_URL || 'https://beatflow-jam-together.onrender.com',
} as const;