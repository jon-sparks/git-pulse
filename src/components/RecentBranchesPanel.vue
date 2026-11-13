<script setup lang="ts">
import { useRepoStore } from "../stores/repoStore";
import { GitBranch, Clock, ExternalLink, Loader2, Plus, GitMerge } from "lucide-vue-next";
import { openExternal } from "../services/opener";

const store = useRepoStore();

function handlePrClick(url?: string) {
  if (url) {
    openExternal(url);
  }
}

function handleCreatePr(branchName: string) {
  if (!store.activeGitHubRepo) return;
  const url = `https://github.com/${store.activeGitHubRepo}/compare/${encodeURIComponent(branchName)}?expand=1`;
  openExternal(url);
}

function handleCheckout(branchName: string) {
  store.checkoutBranch(branchName);
}
</script>

<template>
  <div class="bg-slate-900/70 border border-slate-800/80 rounded-2xl p-5 flex flex-col h-full min-h-0">
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center gap-2">
        <GitBranch class="w-4 h-4 text-indigo-400" />
        <h3 class="text-sm font-semibold text-slate-200 uppercase tracking-wider">
          Recent Local Branches
        </h3>
      </div>
      <span class="text-xs text-slate-500 font-mono">Top 5</span>
    </div>

    <!-- Branch List -->
    <div
      v-if="store.repoContext?.recent_branches?.length"
      class="flex flex-col gap-2.5 flex-1 min-h-0 overflow-y-auto pr-1 scrollbar-thin"
    >
      <div
        v-for="branch in store.repoContext.recent_branches"
        :key="branch.name"
        class="group p-3 rounded-xl border transition-all duration-150 flex items-center justify-between gap-3"
        :class="[
          branch.name === store.repoContext.current_branch
            ? 'bg-slate-800/80 border-indigo-500/50 shadow-sm shadow-indigo-950/40 ring-1 ring-indigo-500/20'
            : 'bg-slate-950/40 border-slate-800/80 hover:bg-slate-800/40 hover:border-slate-700/80',
        ]"
      >
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <span
              class="font-mono text-sm font-medium text-slate-200 truncate group-hover:text-indigo-300 transition-colors"
              :title="branch.name"
            >
              {{ branch.name }}
            </span>

            <span
              v-if="branch.name === store.repoContext.current_branch"
              class="px-1.5 py-0.2 rounded text-[10px] font-semibold tracking-wide uppercase bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 shrink-0"
            >
              Current
            </span>

            <button
              v-if="branch.name === store.repoContext.current_branch && !store.repoContext.is_default_branch && store.repoContext.behind_main > 0"
              @click="store.syncWithMain('rebase')"
              :disabled="store.isSyncingWithMain"
              class="px-2 py-0.5 rounded text-[10px] font-semibold bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/40 transition flex items-center gap-1 cursor-pointer"
              :title="`Rebase ${branch.name} on top of ${store.repoContext.default_branch}`"
            >
              <GitMerge class="w-3 h-3 text-amber-400" />
              <span>Rebase on {{ store.repoContext.default_branch }} ({{ store.repoContext.behind_main }})</span>
            </button>
          </div>

          <div
            v-if="branch.relative_time"
            class="flex items-center gap-1.5 text-xs text-slate-500 mt-1"
          >
            <Clock class="w-3 h-3 text-slate-600" />
            <span>{{ branch.relative_time }}</span>
          </div>
        </div>

        <!-- Actions: Checkout switch button, Create PR, and PR status badge -->
        <div class="shrink-0 flex items-center gap-2">
          <!-- 1-Click Checkout button (for non-current branches) -->
          <button
            v-if="branch.name !== store.repoContext.current_branch"
            @click="handleCheckout(branch.name)"
            :disabled="store.isCheckingOut !== null"
            class="opacity-0 group-hover:opacity-100 focus:opacity-100 flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-300 border border-slate-700 transition cursor-pointer disabled:opacity-50"
            title="Checkout this branch"
          >
            <Loader2
              v-if="store.isCheckingOut === branch.name"
              class="w-3 h-3 animate-spin text-indigo-400"
            />
            <GitBranch v-else class="w-3 h-3 text-indigo-400" />
            <span>{{ store.isCheckingOut === branch.name ? "Switching..." : "Switch" }}</span>
          </button>

          <!-- 1-Click Create PR button (if local only and github repo configured) -->
          <button
            v-if="store.getBranchPrStatus(branch.name).status === 'none' && store.activeGitHubRepo"
            @click="handleCreatePr(branch.name)"
            class="opacity-0 group-hover:opacity-100 focus:opacity-100 flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-indigo-950/80 hover:bg-indigo-900 text-indigo-300 border border-indigo-800/80 transition cursor-pointer"
            title="Create a Pull Request on GitHub for this branch"
          >
            <Plus class="w-3 h-3 text-indigo-400" />
            <span>PR ↗</span>
          </button>

          <!-- Cross-referenced PR Status Badge -->
          <component
            :is="store.getBranchPrStatus(branch.name).pr?.url ? 'button' : 'div'"
            @click="handlePrClick(store.getBranchPrStatus(branch.name).pr?.url)"
            class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-semibold tracking-tight transition"
            :class="[
              store.getBranchPrStatus(branch.name).badgeClass,
              store.getBranchPrStatus(branch.name).pr?.url ? 'hover:scale-105 cursor-pointer' : '',
            ]"
            :title="store.getBranchPrStatus(branch.name).pr?.title || 'Branch Status'"
          >
            <span>{{ store.getBranchPrStatus(branch.name).label }}</span>
            <ExternalLink
              v-if="store.getBranchPrStatus(branch.name).pr?.url"
              class="w-3 h-3 opacity-70"
            />
          </component>
        </div>
      </div>
    </div>

    <!-- Empty branches fallback -->
    <div
      v-else
      class="flex flex-col items-center justify-center p-8 text-center text-slate-500 text-xs flex-1"
    >
      <GitBranch class="w-8 h-8 text-slate-700 mb-2" />
      <p>No recent local branches found.</p>
    </div>
  </div>
</template>
