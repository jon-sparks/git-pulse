import { openUrl } from "@tauri-apps/plugin-opener";

export async function openExternal(url: string) {
  if (!url) return;
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    console.warn("Blocked invalid external URL:", url);
    return;
  }
  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    console.warn("Blocked non-HTTP(S) external URL:", parsed.protocol);
    return;
  }

  try {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      await openUrl(parsed.toString());
      return;
    }
  } catch (e) {
    console.warn("Tauri openUrl failed, falling back to window.open", e);
  }
  window.open(parsed.toString(), "_blank", "noopener,noreferrer");
}
