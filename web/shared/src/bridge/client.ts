import {
  type BridgeError,
  type BridgeEvent,
  type BridgeMethod,
  type ParamsOf,
  type ResultOf,
} from './protocol';

/**
 * Page-side bridge. The product ships as the desktop .exe only, so there is
 * a single transport: the native Rust host answers every call.
 */
export interface Bridge {
  readonly transport: 'desktop';
  call<M extends BridgeMethod>(method: M, params: ParamsOf<M>, options?: CallOptions): Promise<ResultOf<M>>;
  /**
   * The host never pushes events (every state is re-read on demand), so this
   * is a no-op kept for the interface's shape.
   */
  subscribe(_listener: (event: BridgeEvent) => void): () => void;
}

export interface CallOptions {
  timeoutMs?: number;
  signal?: AbortSignal;
}

export class BridgeCallError extends Error {
  readonly code: BridgeError['code'] | 'timeout' | 'unavailable';

  constructor(code: BridgeCallError['code'], message: string) {
    super(message);
    this.name = 'BridgeCallError';
    this.code = code;
  }
}

const DEFAULT_TIMEOUT_MS = 30_000;

/**
 * Runs `start`, settling with its result unless the timeout or the abort
 * signal fires first. `start` receives `settle` and returns a cleanup.
 */
function withDeadline<T>(
  options: CallOptions | undefined,
  start: (settle: (outcome: { value: T } | { error: unknown }) => void) => () => void,
): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    let done = false;
    let cleanup = () => {};
    const finish = (outcome: { value: T } | { error: unknown }) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      options?.signal?.removeEventListener('abort', onAbort);
      cleanup();
      if ('value' in outcome) resolve(outcome.value);
      else reject(outcome.error);
    };
    const timeoutMs = options?.timeoutMs ?? DEFAULT_TIMEOUT_MS;
    const timer = setTimeout(
      () => finish({ error: new BridgeCallError('timeout', `No answer from the host within ${timeoutMs} ms`) }),
      timeoutMs,
    );
    const onAbort = () => finish({ error: options?.signal?.reason ?? new DOMException('Aborted', 'AbortError') });
    if (options?.signal?.aborted) return onAbort();
    options?.signal?.addEventListener('abort', onAbort, { once: true });
    cleanup = start(finish);
  });
}

// ---- Desktop / Tauri Host Bridge ------------------------------------------

export function isDesktopApp(): boolean {
  if (typeof window === 'undefined') return false;
  const w = window as unknown as {
    __BEATFLOW_DESKTOP__?: boolean;
    __TAURI_INTERNALS__?: unknown;
    __TAURI__?: unknown;
  };
  return Boolean(
    w.__BEATFLOW_DESKTOP__ ||
      w.__TAURI_INTERNALS__ ||
      w.__TAURI__ ||
      (typeof window.location !== 'undefined' && window.location.protocol === 'tauri:')
  );
}

export function desktopBridge(): Bridge {
  return {
    transport: 'desktop',
    call(method, params, options) {
      return withDeadline(options, (settle) => {
        void (async () => {
          try {
            const w = window as unknown as {
              __BEATFLOW_CALL__?: (method: string, params: unknown) => Promise<unknown>;
              __TAURI_INTERNALS__?: { invoke: (cmd: string, args: unknown) => Promise<unknown> };
              __TAURI__?: { core?: { invoke: (cmd: string, args: unknown) => Promise<unknown> } };
            };
            let result: unknown;
            if (typeof w?.__BEATFLOW_CALL__ === 'function') {
              result = await w.__BEATFLOW_CALL__(method, params ?? {});
            } else {
              const invoke = w?.__TAURI_INTERNALS__?.invoke ?? w?.__TAURI__?.core?.invoke;
              if (typeof invoke !== 'function') {
                throw new BridgeCallError('unavailable', 'The desktop bridge is not reachable');
              }
              result = await invoke('bridge_call', { method, params: params ?? {} });
            }
            settle({ value: result as ResultOf<typeof method> });
          } catch (error) {
            settle({ error });
          }
        })();
        return () => {};
      });
    },
    subscribe(_listener) {
      return () => {};
    },
  };
}

// ---- Detected extension (desktop stand-in) --------------------------------

export interface DetectedExtension {
  bridge: Bridge;
  info: ResultOf<'ping'>;
}
