import { invoke } from "@tauri-apps/api/core";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function invokeSafe<T>(cmd: string, args?: Record<string, unknown>, fallback?: T): Promise<T> {
  if (isTauriRuntime()) {
    return await invoke<T>(cmd, args);
  }
  if (fallback !== undefined) {
    return fallback;
  }
  throw new Error(`Tauri runtime not active for command: ${cmd}`);
}
