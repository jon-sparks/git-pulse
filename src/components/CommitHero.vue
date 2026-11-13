<script setup lang="ts">
import { ref } from "vue";
import { useRepoStore } from "../stores/repoStore";
import { GitCommit, User, Clock, Copy, Check } from "lucide-vue-next";

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
</script>

<template>
  <section class="bg-gradient-to-br from-slate-900 to-slate-900/60 border border-slate-800/90 rounded-2xl p-5 shadow-lg relative overflow-hidden">
    <!-- Subtle background accent glow -->
    <div class="absolute -right-10 -top-10 w-44 h-44 bg-indigo-500/10 rounded-full blur-3xl pointer-events-none"></div>

    <div class="flex items-center justify-between mb-3">
      <div class="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-indigo-400">
        <GitCommit class="w-4 h-4" />
        <span>Latest Commit</span>
      </div>

      <div v-if="store.repoContext?.last_commit" class="flex items-center gap-1.5">
        <button
          @click="copyHash(store.repoContext.last_commit.hash)"
          class="inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-mono bg-slate-800/90 hover:bg-slate-700/90 text-slate-300 border border-slate-700/70 transition"
          title="Click to copy commit hash"
        >
          <component :is="copied ? Check : Copy" class="w-3 h-3 text-indigo-400" />
          <span>{{ store.repoContext.last_commit.hash }}</span>
        </button>
      </div>
    </div>

    <!-- Commit Content -->
    <div v-if="store.repoContext?.last_commit" class="space-y-3">
      <h2 class="text-base sm:text-lg font-medium text-slate-100 tracking-tight leading-snug line-clamp-2">
        {{ store.repoContext.last_commit.subject }}
      </h2>

      <div class="flex flex-wrap items-center gap-y-2 gap-x-4 text-xs text-slate-400 pt-1 border-t border-slate-800/60">
        <div class="flex items-center gap-1.5">
          <User class="w-3.5 h-3.5 text-slate-500" />
          <span class="text-slate-300 font-medium">{{ store.repoContext.last_commit.author_name }}</span>
        </div>

        <div class="flex items-center gap-1.5">
          <Clock class="w-3.5 h-3.5 text-slate-500" />
          <span>{{ store.repoContext.last_commit.relative_time }}</span>
        </div>
      </div>
    </div>

    <!-- No commit fallback -->
    <div v-else class="text-xs text-slate-500 py-3 italic">
      No commit history found in this repository branch.
    </div>
  </section>
</template>
