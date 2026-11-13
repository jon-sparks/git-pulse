import { ref, shallowRef } from "vue";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export const availableUpdate = shallowRef<Update | null>(null);
export const isCheckingUpdate = ref(false);
export const isInstallingUpdate = ref(false);
export const updateError = ref<string | null>(null);

export async function checkForAppUpdate(): Promise<void> {
  if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) {
    return;
  }
  if (isCheckingUpdate.value || isInstallingUpdate.value) {
    return;
  }

  isCheckingUpdate.value = true;
  updateError.value = null;
  try {
    const update = await check();
    availableUpdate.value = update ?? null;
  } catch (err) {
    // Fail silently on launch if offline or no release exists yet
    console.warn("Update check failed:", err);
  } finally {
    isCheckingUpdate.value = false;
  }
}

export async function installAppUpdate(): Promise<void> {
  const update = availableUpdate.value;
  if (!update || isInstallingUpdate.value) return;

  isInstallingUpdate.value = true;
  updateError.value = null;
  try {
    await update.downloadAndInstall();
    await relaunch();
  } catch (err) {
    console.error("Failed to install update:", err);
    updateError.value = err instanceof Error ? err.message : String(err);
    isInstallingUpdate.value = false;
  }
}
