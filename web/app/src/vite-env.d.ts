/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_ANDROID_APP_URL?: string;
  readonly VITE_PARTY_SERVER_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}