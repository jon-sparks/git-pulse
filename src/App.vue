<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { useRepoStore } from "./stores/repoStore";
import { checkForAppUpdate } from "./services/updater";
import HeaderPanel from "./components/HeaderPanel.vue";
import CommitInfoBar from "./components/CommitInfoBar.vue";
import RepoActionsBar from "./components/RepoActionsBar.vue";
import RecentBranchesPanel from "./components/RecentBranchesPanel.vue";
import OpenPrsPanel from "./components/OpenPrsPanel.vue";
import SettingsModal from "./components/SettingsModal.vue";
import EmptyState from "./components/EmptyState.vue";
import {
  AlertTriangle,
  AlertCircle,
  CheckCircle2,
  Code2,
  RotateCcw,
  Check,
} from "lucide-vue-next";

const store = useRepoStore();

onMounted(() => {
  store.init();
  checkForAppUpdate();
});

onUnmounted(() => {
  store.stopPolling();
});
</script>

<template>
  <div class="min-h-screen lg:h-screen lg:overflow-hidden bg-slate-950 text-slate-100 flex flex-col font-sans select-none antialiased">
    <!-- Top Lean Header (includes Brand, Repo Tabs, Branch, Pull/Sync, and Utilities) -->
    <HeaderPanel />

    <!-- Error / Notification Banners (Slim & Dismissible) -->
    <div v-if="store.gitError" class="px-5 pt-3">
      <div class="p-2.5 bg-rose-950/40 border border-rose-900/60 rounded-xl text-xs text-rose-300 flex items-center justify-between gap-3 shadow-sm">
        <div class="flex items-center gap-2 min-w-0">
          <AlertCircle class="w-4 h-4 text-rose-400 shrink-0" />
          <span class="truncate"><strong>Local Git Error:</strong> {{ store.gitError }}</span>
        </div>
        <button
          @click="store.isSettingsOpen = true"
          class="underline hover:text-white shrink-0 font-medium cursor-pointer"
        >
          Check path
        </button>
      </div>
    </div>

    <div v-if="store.githubError" class="px-5 pt-3">
      <div class="p-2.5 bg-amber-950/40 border border-amber-900/60 rounded-xl text-xs text-amber-300 flex items-center justify-between gap-3 shadow-sm">
        <div class="flex items-center gap-2 min-w-0">
          <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0" />
          <span class="truncate"><strong>GitHub API Notice:</strong> {{ store.githubError }}</span>
        </div>
        <button
          @click="store.isSettingsOpen = true"
          class="underline hover:text-white shrink-0 font-medium cursor-pointer"
        >
          Check token
        </button>
      </div>
    </div>

    <!-- Git Action Toast (Pull / Push / Checkout Output) -->
    <div v-if="store.gitActionNotification" class="px-5 pt-3">
      <div
        class="p-2.5 rounded-xl text-xs flex items-center justify-between gap-3 shadow-md font-mono"
        :class="[
          store.gitActionNotification.type === 'success'
            ? 'bg-emerald-950/70 border border-emerald-800 text-emerald-300'
            : 'bg-rose-950/70 border border-rose-800 text-rose-300',
        ]"
      >
        <div class="flex items-center gap-2 min-w-0">
          <component
            :is="store.gitActionNotification.type === 'success' ? CheckCircle2 : AlertCircle"
            class="w-4 h-4 shrink-0"
            :class="store.gitActionNotification.type === 'success' ? 'text-emerald-400' : 'text-rose-400'"
          />
          <span class="truncate">{{ store.gitActionNotification.text }}</span>
        </div>
        <button
          @click="store.gitActionNotification = null"
          class="text-slate-400 hover:text-white text-xs underline cursor-pointer shrink-0"
        >
          Dismiss
        </button>
      </div>
    </div>

    <!-- Rebase in Progress Banner (Conflict resolution assistance) -->
    <div v-if="store.repoContext?.is_rebasing" class="px-5 pt-3">
      <div class="p-3 bg-amber-950/70 border border-amber-600/80 rounded-xl text-xs text-amber-200 flex flex-wrap items-center justify-between gap-3 shadow-lg shadow-amber-950/40">
        <div class="flex items-center gap-2.5 min-w-0">
          <AlertTriangle class="w-4 h-4 text-amber-400 shrink-0 animate-bounce" />
          <div>
            <span class="font-bold text-amber-300">Rebase Paused (Merge Conflicts):</span>
            <span class="ml-1.5 text-amber-200/90 font-mono">Resolve conflicted files in your editor, then continue or abort.</span>
          </div>
        </div>

        <div class="flex items-center gap-2 shrink-0">
          <button
            @click="store.openInEditor()"
            class="px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 text-xs font-medium transition cursor-pointer flex items-center gap-1.5"
          >
            <Code2 class="w-3.5 h-3.5 text-indigo-400" />
            <span>Open in Editor</span>
          </button>

          <button
            @click="store.continueRebase()"
            class="px-2.5 py-1 rounded-lg bg-amber-600 hover:bg-amber-500 text-white text-xs font-semibold shadow-sm transition cursor-pointer flex items-center gap-1.5"
          >
            <Check class="w-3.5 h-3.5" />
            <span>Continue Rebase</span>
          </button>

          <button
            @click="store.abortRebase()"
            class="px-2.5 py-1 rounded-lg bg-rose-900/60 hover:bg-rose-800/80 text-rose-200 border border-rose-700/60 text-xs font-medium transition cursor-pointer flex items-center gap-1.5"
          >
            <RotateCcw class="w-3.5 h-3.5" />
            <span>Abort Rebase</span>
          </button>
        </div>
      </div>
    </div>

    <!-- Main Content Area -->
    <main class="flex-1 flex flex-col min-h-0 p-5 max-w-7xl w-full mx-auto gap-4">
      <div v-if="!store.activeRepo && !store.repoContext" class="flex-1 flex items-center justify-center">
        <EmptyState />
      </div>

      <div v-else class="flex flex-col gap-3 flex-1 min-h-0">
        <!-- Dedicated Git Commit Context Strip -->
        <CommitInfoBar />

        <!-- Small Repo Actions Row (Buttons with display names) -->
        <RepoActionsBar />

        <!-- 2-Column Dashboard Panels: Recent Branches & My Open PRs -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 flex-1 min-h-0">
          <!-- Left: Context Switch Panel (Recent Branches with Cross-Referenced PR Badges) -->
          <RecentBranchesPanel />

          <!-- Right: Active Work Panel (My Open PRs with CI & Review Status) -->
          <OpenPrsPanel />
        </div>
      </div>
    </main>

    <!-- Settings Modal -->
    <SettingsModal />
  </div>
</template>
