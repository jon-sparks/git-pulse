<script setup lang="ts">
import { ref } from "vue";
import { useRepoStore } from "../stores/repoStore";
import { openExternal } from "../services/opener";
import { GitCommit, GitMerge, Copy, Check, ExternalLink } from "lucide-vue-next";

const store = useRepoStore();
const copied = ref(false);

async function copyHash(hash: string) {
  try {
    await navigator.clipboard.writeText(hash);
    copied.value = true;
    setTimeout(() => {
      copied.value = false;
    }, 2000);
  } catch (err) {
    console.warn("Failed to copy commit hash:", err);
  }
}

function formatRelativeTime(dateStr?: string | null): string {
  if (!dateStr) return "";
  const d = new Date(dateStr);
  const now = new Date();
  const diffSec = Math.max(0, Math.floor((now.getTime() - d.getTime()) / 1000));
  if (diffSec < 60) return "just now";
  const diffMin = Math.floor(diffSec / 60);
  if (diffMin < 60) return `${diffMin}m ago`;
  const diffHr = Math.floor(diffMin / 60);
  if (diffHr < 24) return `${diffHr}h ago`;
  const diffDays = Math.floor(diffHr / 24);
  return `${diffDays}d ago`;
}
</script>

<template>
  <div class="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs">
    <!-- 1. Dedicated Git Commit Strip -->
    <div
      v-if="store.repoContext?.last_commit"
      class="bg-slate-900/50 border border-slate-800/70 rounded-xl px-3.5 py-2 flex items-center justify-between gap-2.5 shadow-sm min-w-0"
    >
      <div class="flex items-center gap-2 min-w-0 flex-1">
        <GitCommit class="w-4 h-4 text-indigo-400 shrink-0" title="Latest local commit on HEAD" />

        <div class="flex items-baseline gap-1.5 min-w-0">
          <span class="text-slate-400 text-[10px] uppercase font-bold tracking-wider shrink-0">Commit</span>
          <span
            class="font-medium text-slate-200 truncate"
            :title="store.repoContext.last_commit.subject"
          >
            {{ store.repoContext.last_commit.subject }}
          </span>

          <span class="text-slate-500 text-[11px] shrink-0 font-mono hidden sm:inline">
            · {{ store.repoContext.last_commit.relative_time }}
          </span>
        </div>
      </div>

      <!-- Commit Hash & Copy -->
      <button
        @click="copyHash(store.repoContext.last_commit.hash)"
        class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded font-mono text-[11px] bg-slate-800 hover:bg-slate-700/80 text-slate-300 hover:text-white transition cursor-pointer shrink-0 border border-slate-700/60"
        title="Copy full commit SHA"
      >
        <component :is="copied ? Check : Copy" class="w-2.5 h-2.5 text-indigo-400" />
        <span>{{ store.repoContext.last_commit.hash }}</span>
        <span v-if="copied" class="text-[10px] text-emerald-400 font-sans">Copied!</span>
      </button>
    </div>

    <!-- 2. Last Merged PR Strip -->
    <div
      v-if="store.lastMergedPR"
      class="bg-slate-900/50 border border-purple-900/40 rounded-xl px-3.5 py-2 flex items-center justify-between gap-2.5 shadow-sm min-w-0"
    >
      <div class="flex items-center gap-2 min-w-0 flex-1">
        <GitMerge class="w-4 h-4 text-purple-400 shrink-0" title="Last merged pull request" />

        <div class="flex items-baseline gap-1.5 min-w-0">
          <span class="text-purple-400 text-[10px] uppercase font-bold tracking-wider shrink-0">Last Merged</span>
          <span class="text-purple-300 font-mono text-xs shrink-0 font-semibold">#{{ store.lastMergedPR.number }}</span>
          <span
            class="font-medium text-slate-200 truncate"
            :title="`${store.lastMergedPR.title}${store.lastMergedPR.author_login ? ` (by @${store.lastMergedPR.author_login})` : ''}`"
          >
            {{ store.lastMergedPR.title }}
          </span>

          <span class="text-slate-500 text-[11px] shrink-0 font-mono hidden sm:inline">
            · {{ store.lastMergedPR.author_login ? `@${store.lastMergedPR.author_login} · ` : '' }}{{ formatRelativeTime(store.lastMergedPR.merged_at || store.lastMergedPR.created_at) }}
          </span>
        </div>
      </div>

      <button
        @click="openExternal(store.lastMergedPR.url)"
        class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] bg-purple-950/70 hover:bg-purple-900 text-purple-300 hover:text-white transition cursor-pointer shrink-0 border border-purple-800/60 font-medium"
        title="View merged PR on GitHub"
      >
        <span>View PR</span>
        <ExternalLink class="w-2.5 h-2.5" />
      </button>
    </div>
  </div>
</template>
