import { LazyStore } from "@tauri-apps/plugin-store";
import type { AppSettings } from "../types";

const STORE_KEY = "git-pulse-settings.json";
let storeInstance: LazyStore | null = null;

function getTauriStore(): LazyStore | null {
  try {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) {
      if (!storeInstance) {
        storeInstance = new LazyStore(STORE_KEY);
      }
      return storeInstance;
    }
  } catch {
    // Non-tauri fallback
  }
  return null;
}

const DEFAULT_SETTINGS: AppSettings = {
  repositories: [],
  activeRepoId: "",
  githubToken: "",
  githubUsername: "",
  repoPath: "",
  githubRepo: "",
};

function migrateSettings(raw: any): AppSettings {
  const merged: AppSettings = { ...DEFAULT_SETTINGS, ...raw };

  // Migrate legacy single repoPath to repositories array
  if ((!merged.repositories || merged.repositories.length === 0) && merged.repoPath) {
    const defaultName = merged.repoPath.split(/[\\/]/).filter(Boolean).pop() || "Repository";
    const migratedId = "repo-" + Date.now();
    merged.repositories = [
      {
        id: migratedId,
        name: defaultName,
        path: merged.repoPath,
        githubRepo: merged.githubRepo || "",
      },
    ];
    merged.activeRepoId = migratedId;
  }

  // Ensure activeRepoId is valid if repositories exist
  if (merged.repositories.length > 0) {
    const exists = merged.repositories.some((r) => r.id === merged.activeRepoId);
    if (!exists) {
      merged.activeRepoId = merged.repositories[0].id;
    }
  }

  return merged;
}

export async function loadSettings(): Promise<AppSettings> {
  const store = getTauriStore();
  if (store) {
    try {
      const saved = await store.get<AppSettings>("app_settings");
      if (saved) {
        return migrateSettings(saved);
      }
    } catch (err) {
      console.warn("Failed to read settings from Tauri store:", err);
    }
  }

  // Fallback to localStorage
  try {
    const raw = localStorage.getItem("git_pulse_settings");
    if (raw) {
      return migrateSettings(JSON.parse(raw));
    }
  } catch (err) {
    console.warn("Failed to read settings from localStorage:", err);
  }

  return { ...DEFAULT_SETTINGS };
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  const plain = JSON.parse(JSON.stringify(settings));
  const store = getTauriStore();
  if (store) {
    try {
      await store.set("app_settings", plain);
      await store.save();
      // Remove any legacy localStorage copy so tokens aren't duplicated in WebView storage
      try {
        localStorage.removeItem("git_pulse_settings");
      } catch {
        // Ignore storage cleanup errors
      }
      return;
    } catch (err) {
      console.warn("Failed to save settings to Tauri store:", err);
    }
  }

  // Non-Tauri browser fallback
  try {
    localStorage.setItem("git_pulse_settings", JSON.stringify(plain));
  } catch (err) {
    console.warn("Failed to save settings to localStorage:", err);
  }
}
