<script setup lang="ts">
import { useRepoStore } from "../stores/repoStore";
import { Plus, FolderGit2, GitBranch, ArrowUp, ArrowDown } from "lucide-vue-next";

const store = useRepoStore();

function getSummary(id: string) {
  return store.repoSummaries[id];
}
</script>

<template>
  <div
    v-if="store.settings.repositories.length > 0"
    class="bg-slate-900/60 border-b border-slate-800/60 px-6 py-2 flex items-center gap-2 overflow-x-auto scrollbar-none"
  >
    <div class="flex items-center gap-2 flex-1 min-w-0">
      <button
        v-for="repo in store.settings.repositories"
        :key="repo.id"
        @click="store.setActiveRepo(repo.id)"
        class="group relative flex items-center gap-2 px-3.5 py-1.5 rounded-xl text-xs font-medium transition-all shrink-0 cursor-pointer border"
        :class="[
          repo.id === store.settings.activeRepoId
            ? 'bg-slate-800 text-white border-indigo-500/60 shadow-md shadow-indigo-950/40 ring-1 ring-indigo-500/20'
            : 'bg-slate-950/40 text-slate-400 border-slate-800/80 hover:bg-slate-800/50 hover:text-slate-200 hover:border-slate-700/80',
        ]"
      >
        <!-- Icon -->
        <FolderGit2
          class="w-3.5 h-3.5 transition-colors"
          :class="repo.id === store.settings.activeRepoId ? 'text-indigo-400' : 'text-slate-500 group-hover:text-slate-400'"
        />

        <!-- Repo Name -->
        <span class="font-semibold tracking-tight">{{ repo.name }}</span>

        <!-- Branch preview from summary -->
        <div
          v-if="getSummary(repo.id)?.current_branch"
          class="flex items-center gap-1 text-[11px] font-mono px-1.5 py-0.2 rounded bg-slate-950/60 text-slate-400 border border-slate-800/80"
        >
          <GitBranch class="w-2.5 h-2.5 text-indigo-400" />
          <span class="truncate max-w-[100px]">{{ getSummary(repo.id)?.current_branch }}</span>
        </div>

        <!-- Sync status mini indicator dot / badge -->
        <div v-if="getSummary(repo.id)" class="flex items-center gap-1">
          <!-- Behind -->
          <span
            v-if="(getSummary(repo.id)?.behind || 0) > 0"
            class="flex items-center text-[10px] font-bold text-rose-400 bg-rose-950/80 px-1 rounded border border-rose-900/60"
            title="Needs pull"
          >
            <ArrowDown class="w-2.5 h-2.5 mr-0.5" />
            {{ getSummary(repo.id)?.behind }}
          </span>

          <!-- Ahead -->
          <span
            v-else-if="(getSummary(repo.id)?.ahead || 0) > 0"
            class="flex items-center text-[10px] font-bold text-sky-400 bg-sky-950/80 px-1 rounded border border-sky-900/60"
            title="Needs push"
          >
            <ArrowUp class="w-2.5 h-2.5 mr-0.5" />
            {{ getSummary(repo.id)?.ahead }}
          </span>

          <!-- Up to date dot -->
          <span
            v-else-if="getSummary(repo.id)?.has_upstream"
            class="w-2 h-2 rounded-full bg-emerald-500/80 shadow-sm shadow-emerald-500/40"
            title="Up to date"
          ></span>

          <!-- Error dot -->
          <span
            v-else-if="getSummary(repo.id)?.error"
            class="w-2 h-2 rounded-full bg-amber-500/80"
            title="Error checking repo"
          ></span>
        </div>
      </button>

      <!-- Quick Add Repo Button -->
      <button
        @click="store.pickAndAddRepo()"
        class="flex items-center gap-1 px-2.5 py-1.5 rounded-xl text-xs font-medium text-slate-400 hover:text-slate-200 bg-slate-950/30 hover:bg-slate-800/60 border border-dashed border-slate-800 hover:border-slate-700 transition shrink-0"
        title="Add another local Git repository"
      >
        <Plus class="w-3.5 h-3.5 text-indigo-400" />
        <span>Add Repo</span>
      </button>
    </div>
  </div>
</template>
