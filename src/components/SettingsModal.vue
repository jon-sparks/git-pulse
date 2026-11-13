<script setup lang="ts">
import { ref, watch } from "vue";
import { useRepoStore } from "../stores/repoStore";
import { openExternal } from "../services/opener";
import type { RepoConfig } from "../types";
import {
  X,
  Key,
  User,
  GitFork,
  Eye,
  EyeOff,
  AlertTriangle,
  Check,
  Plus,
  Trash2,
  ExternalLink,
  Clock,
  Zap,
  Code2,
  Terminal,
  Sparkles,
  FolderSearch,
  Loader2,
  CheckCircle2,
} from "lucide-vue-next";

const store = useRepoStore();

const repositories = ref<RepoConfig[]>([]);
const activeRepoId = ref("");
const githubToken = ref("");
const githubUsername = ref("");

// Editor & Terminal preferences
const defaultEditor = ref("code");
const customEditorCmd = ref("");
const defaultTerminal = ref("Terminal");
const customTerminalCmd = ref("");

// New repo draft
const newPath = ref("");
const newName = ref("");
const newGithub = ref("");
const newAutoPullDaily = ref(false);
const newDailyPullTime = ref("09:00");
const newAutoPullMode = ref<"all" | "main_only" | "current_only">("all");
const isInspectingNew = ref(false);
const autoDetectedStatus = ref<{ detected: boolean; message: string } | null>(null);
const isDetectingExisting = ref<Record<string, boolean>>({});

const showToken = ref(false);
const isSaving = ref(false);

watch(
  () => store.isSettingsOpen,
  (open) => {
    if (open) {
      repositories.value = JSON.parse(JSON.stringify(store.settings.repositories));
      activeRepoId.value = store.settings.activeRepoId;
      githubToken.value = store.settings.githubToken;
      githubUsername.value = store.settings.githubUsername;
      defaultEditor.value = store.settings.defaultEditor || "code";
      customEditorCmd.value = store.settings.customEditorCmd || "";
      defaultTerminal.value = store.settings.defaultTerminal || "Terminal";
      customTerminalCmd.value = store.settings.customTerminalCmd || "";
      newPath.value = "";
      newName.value = "";
      newGithub.value = "";
      newAutoPullDaily.value = false;
      newDailyPullTime.value = "09:00";
      newAutoPullMode.value = "all";
      autoDetectedStatus.value = null;

      // Auto-detect for any repo that is missing its GitHub repo mapping
      for (const r of repositories.value) {
        if (!r.githubRepo?.trim()) {
          autoDetectExistingRepo(r);
        }
      }
    }
  },
  { immediate: true }
);

async function inspectNewRepo(p: string) {
  const trimmed = p.trim();
  if (!trimmed) {
    autoDetectedStatus.value = null;
    return;
  }
  isInspectingNew.value = true;
  autoDetectedStatus.value = null;
  try {
    const ctx = await store.inspectRepoPath(trimmed);
    if (ctx) {
      if (!newName.value || newName.value === "Repo") {
        newName.value = ctx.repo_name;
      }
      if (ctx.detected_github_repo) {
        newGithub.value = ctx.detected_github_repo;
        autoDetectedStatus.value = {
          detected: true,
          message: `Auto-detected from git remote: ${ctx.detected_github_repo}`,
        };
      } else {
        autoDetectedStatus.value = {
          detected: false,
          message: "Local Git repo found, but remote.origin.url is not GitHub",
        };
      }
    } else {
      if (!newName.value) {
        newName.value = trimmed.split(/[\\/]/).filter(Boolean).pop() || "Repo";
      }
      autoDetectedStatus.value = null;
    }
  } finally {
    isInspectingNew.value = false;
  }
}

async function handleBrowseNew() {
  const chosen = await store.pickRepoFolder();
  if (chosen) {
    newPath.value = chosen;
    await inspectNewRepo(chosen);
  }
}

async function autoDetectExistingRepo(r: RepoConfig) {
  isDetectingExisting.value[r.id] = true;
  try {
    const ctx = await store.inspectRepoPath(r.path);
    if (ctx?.detected_github_repo) {
      r.githubRepo = ctx.detected_github_repo;
    }
    if (!r.name && ctx?.repo_name) {
      r.name = ctx.repo_name;
    }
  } finally {
    isDetectingExisting.value[r.id] = false;
  }
}

function addNewRepository() {
  const p = newPath.value.trim();
  if (!p) return;

  const n = newName.value.trim() || p.split(/[\\/]/).filter(Boolean).pop() || "Repo";
  const id = "repo-" + Date.now() + "-" + Math.random().toString(36).slice(2, 6);

  repositories.value.push({
    id,
    name: n,
    path: p,
    githubRepo: newGithub.value.trim(),
    autoPullDaily: newAutoPullDaily.value,
    dailyPullTime: newAutoPullDaily.value ? newDailyPullTime.value || "09:00" : undefined,
    autoPullMode: newAutoPullDaily.value ? newAutoPullMode.value : undefined,
  });

  if (!activeRepoId.value) {
    activeRepoId.value = id;
  }

  newPath.value = "";
  newName.value = "";
  newGithub.value = "";
  newAutoPullDaily.value = false;
  newDailyPullTime.value = "09:00";
  newAutoPullMode.value = "all";
  autoDetectedStatus.value = null;
}

function removeRepo(id: string) {
  repositories.value = repositories.value.filter((r) => r.id !== id);
  if (activeRepoId.value === id) {
    activeRepoId.value = repositories.value[0]?.id || "";
  }
}

async function handleSave() {
  isSaving.value = true;
  try {
    // If user filled in the new repo inputs without clicking "Add", add it automatically
    if (newPath.value.trim()) {
      addNewRepository();
    }

    await store.updateSettings({
      repositories: repositories.value,
      activeRepoId: activeRepoId.value,
      githubToken: githubToken.value.trim(),
      githubUsername: githubUsername.value.trim(),
      defaultEditor: defaultEditor.value,
      customEditorCmd: customEditorCmd.value.trim(),
      defaultTerminal: defaultTerminal.value,
      customTerminalCmd: customTerminalCmd.value.trim(),
    });
    store.isSettingsOpen = false;
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <div
    v-if="store.isSettingsOpen"
    class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm"
  >
    <div
      class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
    >
      <!-- Header -->
      <div class="px-6 py-4 border-b border-slate-800/80 flex items-center justify-between">
        <div>
          <h2 class="text-base font-semibold text-slate-100">Settings & Repositories</h2>
          <p class="text-xs text-slate-400">Manage multiple local repositories and GitHub credentials</p>
        </div>

        <button
          @click="store.isSettingsOpen = false"
          class="p-1.5 rounded-lg text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition cursor-pointer"
        >
          <X class="w-4 h-4" />
        </button>
      </div>

      <!-- Form Body -->
      <form @submit.prevent="handleSave" class="p-6 space-y-6 overflow-y-auto flex-1">
        <!-- 1. Configured Repositories List -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <label class="block text-xs font-semibold uppercase tracking-wider text-slate-300">
              Configured Repositories ({{ repositories.length }})
            </label>
            <span class="text-[11px] text-slate-500">
              Select default active repository
            </span>
          </div>

          <!-- Existing repos list -->
          <div v-if="repositories.length" class="space-y-2.5">
            <div
              v-for="r in repositories"
              :key="r.id"
              class="p-3 bg-slate-950/70 border rounded-xl flex items-center justify-between gap-3 transition"
              :class="r.id === activeRepoId ? 'border-indigo-500/60 shadow-sm shadow-indigo-950/40 ring-1 ring-indigo-500/20' : 'border-slate-800'"
            >
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <input
                  type="radio"
                  :value="r.id"
                  v-model="activeRepoId"
                  name="activeRepo"
                  class="text-indigo-600 focus:ring-indigo-500 cursor-pointer"
                  title="Set as active repository"
                />

                <div class="min-w-0 flex-1 space-y-1.5">
                  <div class="flex items-center gap-2">
                    <span class="font-semibold text-sm text-slate-100 truncate">{{ r.name }}</span>
                    <span
                      v-if="r.id === activeRepoId"
                      class="px-1.5 py-0.2 rounded text-[10px] font-semibold uppercase bg-indigo-500/20 text-indigo-300 border border-indigo-500/30"
                    >
                      Active
                    </span>
                  </div>
                  <p class="text-xs text-slate-400 font-mono truncate" :title="r.path">{{ r.path }}</p>
                  
                  <!-- GitHub Repo Mapping Input -->
                  <div class="flex items-center gap-1.5 pt-1">
                    <GitFork class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                    <input
                      v-model="r.githubRepo"
                      type="text"
                      placeholder="GitHub repo: owner/repo (e.g. octocat/hello-world)"
                      class="bg-slate-900 border border-slate-800 rounded px-2 py-0.5 text-xs text-indigo-300 placeholder-slate-600 focus:outline-none focus:border-indigo-500 font-mono flex-1"
                    />
                    <button
                      type="button"
                      @click="autoDetectExistingRepo(r)"
                      :disabled="isDetectingExisting[r.id]"
                      class="px-2 py-0.5 text-[11px] font-medium text-indigo-400 hover:text-indigo-300 bg-indigo-500/10 hover:bg-indigo-500/20 rounded border border-indigo-500/20 transition cursor-pointer flex items-center gap-1 shrink-0 disabled:opacity-50"
                      title="Auto-detect GitHub owner/repo from local .git/config"
                    >
                      <Loader2 v-if="isDetectingExisting[r.id]" class="w-3 h-3 animate-spin" />
                      <Sparkles v-else class="w-3 h-3" />
                      <span>Auto-detect</span>
                    </button>
                  </div>

                  <!-- Daily Auto-Pull Option -->
                  <div class="flex flex-col gap-2 pt-2 border-t border-slate-900/90 text-xs">
                    <div class="flex flex-wrap items-center justify-between gap-2">
                      <label class="flex items-center gap-2 cursor-pointer text-slate-300 hover:text-white transition">
                        <input
                          type="checkbox"
                          v-model="r.autoPullDaily"
                          class="rounded border-slate-700 bg-slate-900 text-indigo-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
                        />
                        <span class="font-medium flex items-center gap-1.5">
                          <Zap class="w-3.5 h-3.5 text-amber-400" />
                          <span>Auto-pull every day</span>
                        </span>
                      </label>

                      <div v-if="r.autoPullDaily" class="flex items-center gap-2">
                        <Clock class="w-3.5 h-3.5 text-slate-500 shrink-0" />
                        <span class="text-[11px] text-slate-400">at</span>
                        <input
                          v-model="r.dailyPullTime"
                          type="time"
                          class="bg-slate-900 border border-slate-800 rounded px-2 py-0.5 text-xs text-indigo-300 font-mono focus:outline-none focus:border-indigo-500 cursor-pointer"
                          title="Scheduled pull time"
                        />
                        <span
                          v-if="r.lastDailyPullDate"
                          class="text-[10px] text-slate-500 font-mono"
                          :title="`Last pulled: ${r.lastDailyPullDate}`"
                        >
                          {{ r.lastDailyPullDate === new Date().toLocaleDateString('en-CA') ? '✓ Pulled today' : `Last: ${r.lastDailyPullDate}` }}
                        </span>
                      </div>
                    </div>

                    <!-- Auto-Pull Mode Selector -->
                    <div v-if="r.autoPullDaily" class="flex items-center gap-2 pl-5 pt-0.5 text-[11px] text-slate-400">
                      <span class="shrink-0 text-slate-400">Mode:</span>
                      <select
                        v-model="r.autoPullMode"
                        class="bg-slate-900 border border-slate-800 rounded px-2 py-0.5 text-xs text-indigo-300 focus:outline-none focus:border-indigo-500 font-sans cursor-pointer"
                      >
                        <option :value="undefined">Fast-forward main & pull current branch (Recommended)</option>
                        <option value="all">Fast-forward main & pull current branch (Recommended)</option>
                        <option value="main_only">Fast-forward main only (Never touches feature branch)</option>
                        <option value="current_only">Pull current branch only</option>
                      </select>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Delete repo button -->
              <button
                type="button"
                @click="removeRepo(r.id)"
                class="p-1.5 text-slate-500 hover:text-rose-400 rounded-lg hover:bg-slate-800 transition cursor-pointer"
                title="Remove from GitPulse"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </div>
          </div>

          <div v-else class="text-xs text-slate-500 italic p-4 text-center bg-slate-950/40 rounded-xl border border-dashed border-slate-800">
            No repositories configured yet. Add your first repository below.
          </div>

          <!-- Add another repository drawer -->
          <div class="p-4 bg-slate-950/60 border border-dashed border-indigo-500/30 rounded-xl space-y-3">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-slate-200 flex items-center gap-1.5">
                <Plus class="w-3.5 h-3.5 text-indigo-400" />
                Add Local Repository
              </span>
              <span class="text-[11px] text-slate-400 flex items-center gap-1">
                <Sparkles class="w-3 h-3 text-indigo-400" />
                Auto-detects from <code class="font-mono text-slate-300">.git/config</code>
              </span>
            </div>

            <!-- Path Picker (Primary Step) -->
            <div class="space-y-1.5">
              <label class="block text-[10px] text-slate-400 uppercase tracking-wider font-semibold">
                1. Select Local Repo Folder
              </label>
              <div class="flex items-center gap-2">
                <input
                  v-model="newPath"
                  @change="inspectNewRepo(newPath)"
                  type="text"
                  placeholder="/path/to/local/git/repo"
                  class="flex-1 bg-slate-900 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-500 font-mono"
                />
                <button
                  type="button"
                  @click="handleBrowseNew"
                  class="px-3.5 py-1.5 bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 text-xs font-medium rounded-lg border border-indigo-500/30 transition cursor-pointer shrink-0 flex items-center gap-1.5"
                >
                  <FolderSearch class="w-3.5 h-3.5" />
                  <span>Browse...</span>
                </button>
              </div>

              <!-- Status indicator -->
              <div v-if="isInspectingNew" class="flex items-center gap-1.5 text-[11px] text-slate-400 font-mono">
                <Loader2 class="w-3 h-3 animate-spin text-indigo-400" />
                <span>Reading Git configuration...</span>
              </div>
              <div
                v-else-if="autoDetectedStatus"
                class="flex items-center gap-1.5 text-[11px]"
                :class="autoDetectedStatus.detected ? 'text-emerald-400' : 'text-slate-400'"
              >
                <CheckCircle2 v-if="autoDetectedStatus.detected" class="w-3 h-3 text-emerald-400 shrink-0" />
                <Sparkles v-else class="w-3 h-3 text-slate-500 shrink-0" />
                <span>{{ autoDetectedStatus.message }}</span>
              </div>
            </div>

            <!-- Auto-filled Name & GitHub repo -->
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2.5 pt-1">
              <div>
                <label class="block text-[10px] text-slate-400 uppercase tracking-wider mb-1 font-semibold">
                  Repo Display Name
                </label>
                <input
                  v-model="newName"
                  type="text"
                  placeholder="Auto-detected from git folder"
                  class="w-full bg-slate-900 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-[10px] text-slate-400 uppercase tracking-wider mb-1 font-semibold">
                  GitHub Repo (owner/repo)
                </label>
                <input
                  v-model="newGithub"
                  type="text"
                  placeholder="Auto-detected from git remote"
                  class="w-full bg-slate-900 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-indigo-300 placeholder-slate-600 focus:outline-none focus:border-indigo-500 font-mono"
                />
              </div>
            </div>

            <!-- Auto-pull toggle & Add Button -->
            <div class="flex flex-col gap-2 pt-2 border-t border-slate-900 text-xs">
              <div class="flex flex-wrap items-center justify-between gap-3">
                <label class="flex items-center gap-2 cursor-pointer text-slate-300 hover:text-white transition">
                  <input
                    type="checkbox"
                    v-model="newAutoPullDaily"
                    class="rounded border-slate-700 bg-slate-900 text-indigo-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
                  />
                  <span class="flex items-center gap-1.5">
                    <Zap class="w-3.5 h-3.5 text-amber-400" />
                    <span>Auto-pull every day</span>
                  </span>
                </label>

                <div class="flex items-center gap-2">
                  <div v-if="newAutoPullDaily" class="flex items-center gap-1.5 mr-2">
                    <span class="text-slate-400">at</span>
                    <input
                      v-model="newDailyPullTime"
                      type="time"
                      class="bg-slate-900 border border-slate-800 rounded px-1.5 py-0.5 text-xs text-indigo-300 font-mono focus:outline-none focus:border-indigo-500"
                    />
                  </div>

                  <button
                    type="button"
                    @click="addNewRepository"
                    :disabled="!newPath.trim()"
                    class="px-4 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 text-white text-xs font-semibold rounded-lg transition cursor-pointer flex items-center gap-1.5 shadow-sm shadow-indigo-950/40"
                  >
                    <Plus class="w-3.5 h-3.5" />
                    <span>Add Repository</span>
                  </button>
                </div>
              </div>

              <!-- New Auto-Pull Mode Selector -->
              <div v-if="newAutoPullDaily" class="flex items-center gap-2 pl-5 pt-0.5 text-[11px] text-slate-400">
                <span class="shrink-0 text-slate-400">Mode:</span>
                <select
                  v-model="newAutoPullMode"
                  class="bg-slate-900 border border-slate-800 rounded px-2 py-0.5 text-xs text-indigo-300 focus:outline-none focus:border-indigo-500 font-sans cursor-pointer"
                >
                  <option value="all">Fast-forward main & pull current branch (Recommended)</option>
                  <option value="main_only">Fast-forward main only (Never touches feature branch)</option>
                  <option value="current_only">Pull current branch only</option>
                </select>
              </div>
            </div>
          </div>
        </div>

        <!-- 2. Editor & Terminal Shortcuts -->
        <div class="border-t border-slate-800/80 pt-4 space-y-3">
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-300">
            Editor & Terminal Shortcuts
          </label>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <!-- Default Code Editor -->
            <div class="space-y-1.5">
              <label class="text-xs text-slate-300 font-medium flex items-center gap-1.5">
                <Code2 class="w-3.5 h-3.5 text-indigo-400" />
                <span>Code Editor</span>
              </label>

              <select
                v-model="defaultEditor"
                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-slate-200 focus:outline-none focus:border-indigo-500 cursor-pointer"
              >
                <option value="code">Visual Studio Code</option>
                <option value="cursor">Cursor</option>
                <option value="windsurf">Windsurf</option>
                <option value="zed">Zed</option>
                <option value="sublime">Sublime Text</option>
                <option value="webstorm">WebStorm</option>
                <option value="finder">Finder (Reveal in folder)</option>
                <option value="custom">Custom Command / Path...</option>
              </select>

              <input
                v-if="defaultEditor === 'custom'"
                v-model="customEditorCmd"
                placeholder="Command, e.g. nvim or subl"
                class="w-full bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs text-indigo-300 font-mono placeholder-slate-600 focus:outline-none focus:border-indigo-500 mt-1"
              />
            </div>

            <!-- Default Terminal -->
            <div class="space-y-1.5">
              <label class="text-xs text-slate-300 font-medium flex items-center gap-1.5">
                <Terminal class="w-3.5 h-3.5 text-emerald-400" />
                <span>Terminal</span>
              </label>

              <select
                v-model="defaultTerminal"
                class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-slate-200 focus:outline-none focus:border-indigo-500 cursor-pointer"
              >
                <option value="Ghostty">Ghostty</option>
                <option value="Terminal">Terminal (macOS Built-in)</option>
                <option value="iTerm">iTerm2</option>
                <option value="Warp">Warp</option>
                <option value="Alacritty">Alacritty</option>
                <option value="kitty">kitty</option>
                <option value="custom">Custom Command / Path...</option>
              </select>

              <input
                v-if="defaultTerminal === 'custom'"
                v-model="customTerminalCmd"
                placeholder="Command, e.g. wezterm"
                class="w-full bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-xs text-emerald-300 font-mono placeholder-slate-600 focus:outline-none focus:border-indigo-500 mt-1"
              />
            </div>
          </div>
        </div>

        <div class="border-t border-slate-800/80 pt-4 space-y-4">
          <!-- 3. GitHub Username -->
          <div class="space-y-1.5">
            <label class="block text-xs font-semibold uppercase tracking-wider text-slate-300">
              GitHub Username
            </label>
            <div class="relative">
              <User class="w-4 h-4 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                v-model="githubUsername"
                type="text"
                placeholder="e.g. octocat"
                class="w-full bg-slate-950 border border-slate-800 rounded-xl pl-9 pr-3 py-2 text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 font-mono"
              />
            </div>
          </div>

          <!-- 3. GitHub Personal Access Token (PAT) -->
          <div class="space-y-1.5">
            <div class="flex items-center justify-between">
              <label class="block text-xs font-semibold uppercase tracking-wider text-slate-300">
                GitHub Personal Access Token (PAT)
              </label>
              <button
                type="button"
                @click="openExternal('https://github.com/settings/tokens/new?description=GitPulse&scopes=repo')"
                class="text-[11px] text-indigo-400 hover:text-indigo-300 flex items-center gap-1 font-medium transition cursor-pointer"
              >
                <span>Generate token on GitHub</span>
                <ExternalLink class="w-3 h-3" />
              </button>
            </div>
            <div class="relative">
              <Key class="w-4 h-4 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" />
              <input
                v-model="githubToken"
                :type="showToken ? 'text' : 'password'"
                placeholder="ghp_xxxxxxxxxxxxxxxxxxxx or github_pat_xxxx"
                class="w-full bg-slate-950 border border-slate-800 rounded-xl pl-9 pr-10 py-2 text-sm text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 font-mono"
              />
              <button
                type="button"
                @click="showToken = !showToken"
                class="absolute right-3 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-300 transition cursor-pointer"
                title="Toggle token visibility"
              >
                <component :is="showToken ? EyeOff : Eye" class="w-4 h-4" />
              </button>
            </div>

            <!-- Token security note -->
            <div class="p-3 bg-amber-950/20 border border-amber-900/40 rounded-xl text-xs text-amber-300/90 flex gap-2.5 items-start mt-2">
              <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
              <div class="space-y-1 leading-relaxed">
                <span class="font-semibold block text-amber-200">Organization & SAML SSO Note</span>
                <span>
                  For work repositories, ensure the Personal Access Token is authorized for <strong>SAML SSO</strong> in your organization's security settings.
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Footer Actions -->
        <div class="flex items-center justify-end gap-3 pt-3 border-t border-slate-800/80">
          <button
            type="button"
            @click="store.isSettingsOpen = false"
            class="px-4 py-2 rounded-xl text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition cursor-pointer"
          >
            Cancel
          </button>
          <button
            type="submit"
            :disabled="isSaving"
            class="px-5 py-2 rounded-xl text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white shadow-md shadow-indigo-950 flex items-center gap-1.5 transition disabled:opacity-50 cursor-pointer"
          >
            <Check class="w-3.5 h-3.5" />
            <span>{{ isSaving ? "Saving..." : "Save & Sync" }}</span>
          </button>
        </div>
      </form>
    </div>
  </div>
</template>
