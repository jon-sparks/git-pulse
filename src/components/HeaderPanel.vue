<script setup lang="ts">
import { ref, computed } from "vue";
import { useRepoStore } from "../stores/repoStore";
import {
  availableUpdate,
  isInstallingUpdate,
  installAppUpdate,
} from "../services/updater";
import {
  GitBranch,
  RefreshCw,
  Settings as SettingsIcon,
  Activity,
  Download,
  Upload,
  Check,
  Loader2,
  Code2,
  Terminal,
  CircleDot,
  Plus,
  X,
  GitMerge,
  ChevronDown,
} from "lucide-vue-next";

const store = useRepoStore();
const showChangesPopover = ref(false);
const showSyncMenu = ref(false);

const trackedChangesCount = computed(() => {
  if (!store.repoContext?.changed_files) return 0;
  return store.repoContext.changed_files.filter((f) => !f.startsWith("??")).length;
});

const untrackedCount = computed(() => {
  if (!store.repoContext?.changed_files) return 0;
  return store.repoContext.changed_files.filter((f) => f.startsWith("??")).length;
});

const syncStatus = computed(() => {
  if (!store.repoContext) return null;
  const { has_upstream, ahead, behind } = store.repoContext;

  if (!has_upstream) {
    return {
      type: "no_upstream",
      label: "No Upstream",
      color: "bg-slate-800/80 text-slate-400 border-slate-700/60",
      actionable: false,
    };
  }

  if (behind > 0) {
    return {
      type: "behind",
      label: `Pull (${behind})`,
      color: "bg-rose-600 hover:bg-rose-500 text-white shadow-sm shadow-rose-950 font-semibold cursor-pointer animate-pulse",
      actionable: true,
      action: () => store.pullCurrentRepo(),
    };
  }

  if (ahead > 0) {
    return {
      type: "ahead",
      label: `Push (${ahead})`,
      color: "bg-sky-600 hover:bg-sky-500 text-white shadow-sm shadow-sky-950 font-semibold cursor-pointer",
      actionable: true,
      action: () => store.pushCurrentRepo(),
    };
  }

  return {
    type: "synced",
    label: "Up to date",
    color: "bg-emerald-950/60 text-emerald-400 border-emerald-800/60",
    actionable: false,
  };
});
</script>

<template>
  <header
    class="bg-slate-900/90 backdrop-blur-md border-b border-slate-800/80 px-5 py-2.5 sticky top-0 z-20 flex items-center justify-between gap-4"
  >
    <!-- Left: Logo & Repository Tabs -->
    <div class="flex items-center gap-3 min-w-0">
      <!-- App Logo -->
      <div
        class="w-8 h-8 rounded-lg bg-gradient-to-tr from-indigo-600 to-violet-500 flex items-center justify-center text-white shadow-md shadow-indigo-950/60 shrink-0"
        title="GitPulse"
      >
        <Activity class="w-4 h-4" />
      </div>

      <!-- Repo Tabs Strip -->
      <div class="flex items-center gap-1.5 overflow-x-auto scrollbar-none py-0.5">
        <button
          v-for="repo in store.settings.repositories"
          :key="repo.id"
          @click="store.setActiveRepo(repo.id)"
          class="flex items-center gap-1.5 px-3 py-1 rounded-lg text-xs font-medium transition cursor-pointer shrink-0 border"
          :class="[
            repo.id === store.settings.activeRepoId
              ? 'bg-slate-800 text-white border-indigo-500/60 shadow-sm shadow-indigo-950/40 ring-1 ring-indigo-500/30'
              : 'bg-transparent text-slate-400 border-transparent hover:bg-slate-800/60 hover:text-slate-200',
          ]"
          :title="repo.autoPullDaily
            ? `${repo.name} • Daily auto-pull at ${repo.dailyPullTime || '09:00'}${repo.lastDailyPullDate === new Date().toLocaleDateString('en-CA') ? ' (pulled today)' : ''}`
            : repo.name"
        >
          <span class="font-semibold">{{ repo.name }}</span>

          <!-- Tiny sync indicator dot if behind/ahead -->
          <span
            v-if="(store.repoSummaries[repo.id]?.behind || 0) > 0"
            class="w-1.5 h-1.5 rounded-full bg-rose-400"
            title="Needs pull"
          ></span>
          <span
            v-else-if="(store.repoSummaries[repo.id]?.ahead || 0) > 0"
            class="w-1.5 h-1.5 rounded-full bg-sky-400"
            title="Unpushed commits"
          ></span>
        </button>

        <!-- Quick Add Repo Button -->
        <button
          @click="store.pickAndAddRepo()"
          class="p-1 rounded-lg text-slate-500 hover:text-slate-300 hover:bg-slate-800/60 transition cursor-pointer"
          title="Add another repository"
        >
          <Plus class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Right: Context Info & Quick Actions -->
    <div class="flex items-center gap-2.5 shrink-0">
      <!-- Current branch & working tree pill -->
      <div
        v-if="store.repoContext?.current_branch"
        class="hidden sm:flex items-center gap-2 px-2.5 py-1 rounded-lg bg-slate-950/70 border border-slate-800 text-xs text-slate-300 font-mono"
      >
        <div class="flex items-center gap-1.5">
          <GitBranch class="w-3 h-3 text-indigo-400" />
          <span class="max-w-[130px] truncate">{{ store.repoContext.current_branch }}</span>
          <span
            v-if="!store.repoContext.is_default_branch && store.repoContext.behind_main > 0"
            class="px-1.5 py-0.2 rounded text-[10px] font-semibold bg-amber-500/20 text-amber-300 border border-amber-500/30 font-sans"
            :title="`${store.repoContext.behind_main} commit(s) behind ${store.repoContext.default_branch}`"
          >
            {{ store.repoContext.behind_main }} behind {{ store.repoContext.default_branch }}
          </span>
        </div>

        <span class="text-slate-700">•</span>

        <!-- Clean/Dirty status dot with interactive popover -->
        <div class="relative">
          <button
            @click="showChangesPopover = !showChangesPopover"
            class="flex items-center gap-1 text-[11px] hover:text-white transition cursor-pointer"
            :class="[
              trackedChangesCount > 0
                ? 'text-amber-400 font-semibold'
                : untrackedCount > 0
                ? 'text-sky-400'
                : 'text-slate-500'
            ]"
            title="Click to view changed / untracked files"
          >
            <CircleDot class="w-2.5 h-2.5" />
            <span>
              {{
                trackedChangesCount > 0
                  ? `${trackedChangesCount} mod${untrackedCount > 0 ? ` + ${untrackedCount} untracked` : ''}`
                  : untrackedCount > 0
                  ? `${untrackedCount} untracked`
                  : 'Clean'
              }}
            </span>
          </button>

          <!-- Dropdown Popover listing files -->
          <div
            v-if="showChangesPopover"
            class="absolute right-0 top-full mt-2 w-80 bg-slate-900 border border-slate-700/80 rounded-xl p-3 shadow-2xl z-50 flex flex-col gap-2 font-sans"
          >
            <div class="flex items-center justify-between border-b border-slate-800 pb-2">
              <span class="text-xs font-semibold text-slate-200">
                Working Tree Files ({{ store.repoContext?.uncommitted_changes || 0 }})
              </span>
              <button
                @click="showChangesPopover = false"
                class="text-slate-400 hover:text-white cursor-pointer"
              >
                <X class="w-3.5 h-3.5" />
              </button>
            </div>

            <div
              v-if="!store.repoContext?.changed_files || store.repoContext.changed_files.length === 0"
              class="text-xs text-slate-400 italic py-2 text-center"
            >
              Working tree is completely clean
            </div>

            <div v-else class="max-h-60 overflow-y-auto space-y-1.5 scrollbar-thin">
              <div
                v-for="(f, i) in store.repoContext.changed_files"
                :key="i"
                class="flex items-start gap-2 p-1.5 rounded-lg bg-slate-950/60 border border-slate-800/80 text-xs font-mono"
              >
                <span
                  v-if="f.startsWith('??')"
                  class="px-1.5 py-0.2 rounded text-[10px] uppercase font-bold bg-sky-500/20 text-sky-300 border border-sky-500/30 shrink-0 font-sans"
                >
                  Untracked
                </span>
                <span
                  v-else-if="f.trimStart().startsWith('D')"
                  class="px-1.5 py-0.2 rounded text-[10px] uppercase font-bold bg-rose-500/20 text-rose-300 border border-rose-500/30 shrink-0 font-sans"
                >
                  Deleted
                </span>
                <span
                  v-else-if="f.trimStart().startsWith('A')"
                  class="px-1.5 py-0.2 rounded text-[10px] uppercase font-bold bg-emerald-500/20 text-emerald-300 border border-emerald-500/30 shrink-0 font-sans"
                >
                  Added
                </span>
                <span
                  v-else
                  class="px-1.5 py-0.2 rounded text-[10px] uppercase font-bold bg-amber-500/20 text-amber-300 border border-amber-500/30 shrink-0 font-sans"
                >
                  Modified
                </span>

                <span class="text-slate-300 truncate flex-1" :title="f.slice(2).trim()">
                  {{ f.slice(2).trim() }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Combined Sync / Pull / Push Button -->
      <button
        v-if="syncStatus"
        @click="syncStatus.actionable && syncStatus.action ? syncStatus.action() : store.pullCurrentRepo()"
        :disabled="store.isPulling || store.isPushing"
        class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-medium border transition disabled:opacity-50 cursor-pointer"
        :class="syncStatus.color"
      >
        <Loader2 v-if="store.isPulling || store.isPushing" class="w-3 h-3 animate-spin" />
        <Download v-else-if="syncStatus.type === 'behind'" class="w-3 h-3" />
        <Upload v-else-if="syncStatus.type === 'ahead'" class="w-3 h-3" />
        <Check v-else class="w-3 h-3" />
        <span>{{ store.isPulling ? "Pulling..." : store.isPushing ? "Pushing..." : syncStatus.label }}</span>
      </button>

      <!-- Sync with Main Action (when on a feature branch) -->
      <div v-if="store.repoContext && !store.repoContext.is_default_branch" class="relative">
        <button
          @click="showSyncMenu = !showSyncMenu"
          :disabled="store.isSyncingWithMain || store.isPulling"
          class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-medium border transition disabled:opacity-50 cursor-pointer"
          :class="[
            store.repoContext.behind_main > 0
              ? 'bg-amber-950/70 hover:bg-amber-900/90 text-amber-300 border-amber-700/80 shadow-sm'
              : 'bg-slate-900/60 hover:bg-slate-800 text-slate-300 border-slate-800'
          ]"
          :title="`Sync with ${store.repoContext.default_branch}`"
        >
          <Loader2 v-if="store.isSyncingWithMain" class="w-3 h-3 animate-spin text-amber-400" />
          <GitMerge v-else class="w-3 h-3 text-amber-400" />
          <span>
            {{ store.isSyncingWithMain ? "Syncing..." : `Sync ${store.repoContext.default_branch}${store.repoContext.behind_main > 0 ? ` (${store.repoContext.behind_main})` : ''}` }}
          </span>
          <ChevronDown class="w-3 h-3 text-slate-400" />
        </button>

        <!-- Dropdown Menu -->
        <div
          v-if="showSyncMenu"
          class="absolute right-0 top-full mt-1.5 w-64 bg-slate-900 border border-slate-700 rounded-xl p-1.5 shadow-2xl z-50 flex flex-col gap-1 text-xs"
        >
          <button
            @click="showSyncMenu = false; store.syncWithMain('rebase')"
            class="w-full text-left px-2.5 py-2 rounded-lg hover:bg-slate-800 text-slate-200 transition cursor-pointer"
          >
            <div class="font-semibold text-slate-100 flex items-center justify-between">
              <span>Rebase on {{ store.repoContext.default_branch }}</span>
              <span class="text-[9px] bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 px-1 rounded uppercase font-bold">Recommended</span>
            </div>
            <p class="text-[10px] text-slate-400 mt-0.5">Replays your commits on top of fresh {{ store.repoContext.default_branch }}</p>
          </button>

          <button
            @click="showSyncMenu = false; store.syncWithMain('merge')"
            class="w-full text-left px-2.5 py-2 rounded-lg hover:bg-slate-800 text-slate-200 transition cursor-pointer"
          >
            <div class="font-semibold text-slate-100">Merge {{ store.repoContext.default_branch }}</div>
            <p class="text-[10px] text-slate-400 mt-0.5">Creates a merge commit from {{ store.repoContext.default_branch }}</p>
          </button>

          <div class="border-t border-slate-800 my-0.5"></div>

          <button
            @click="showSyncMenu = false; store.fastForwardMain()"
            class="w-full text-left px-2.5 py-1.5 rounded-lg hover:bg-slate-800 text-slate-300 transition cursor-pointer"
          >
            <div class="font-medium text-slate-200">Fast-forward {{ store.repoContext.default_branch }} only</div>
            <p class="text-[10px] text-slate-400">Updates local {{ store.repoContext.default_branch }} without altering this branch</p>
          </button>
        </div>
      </div>

      <!-- Utility Group: Update, Editor, Terminal, Settings -->
      <div class="flex items-center gap-1 border-l border-slate-800 pl-2.5">
        <button
          v-if="availableUpdate"
          @click="installAppUpdate()"
          :disabled="isInstallingUpdate"
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white shadow-sm shadow-emerald-950 transition cursor-pointer disabled:opacity-60 mr-1"
          :title="`Update available: v${availableUpdate.version}. Click to download and relaunch.`"
        >
          <Loader2 v-if="isInstallingUpdate" class="w-3.5 h-3.5 animate-spin" />
          <Download v-else class="w-3.5 h-3.5" />
          <span>{{ isInstallingUpdate ? "Updating..." : `Update v${availableUpdate.version}` }}</span>
        </button>

        <button
          v-if="store.activeRepo"
          @click="store.openInEditor()"
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-md transition cursor-pointer"
          :title="`Open in ${store.settings.defaultEditor === 'custom' ? (store.settings.customEditorCmd || 'Editor') : (store.settings.defaultEditor || 'Editor')}`"
        >
          <Code2 class="w-3.5 h-3.5" />
        </button>

        <button
          v-if="store.activeRepo"
          @click="store.openInTerminal()"
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-md transition cursor-pointer"
          :title="`Open in ${store.settings.defaultTerminal === 'custom' ? (store.settings.customTerminalCmd || 'Terminal') : (store.settings.defaultTerminal || 'Terminal')}`"
        >
          <Terminal class="w-3.5 h-3.5" />
        </button>

        <button
          @click="store.refreshAll()"
          :disabled="store.isRefreshing"
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-md transition cursor-pointer disabled:opacity-50"
          :title="`Refresh (last: ${store.lastUpdatedText})`"
        >
          <RefreshCw
            class="w-3.5 h-3.5"
            :class="{ 'animate-spin text-indigo-400': store.isRefreshing }"
          />
        </button>

        <button
          @click="store.isSettingsOpen = true"
          class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-md transition cursor-pointer"
          title="Settings"
        >
          <SettingsIcon class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  </header>
</template>
