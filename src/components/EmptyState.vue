<script setup lang="ts">
import { useRepoStore } from "../stores/repoStore";
import { FolderGit2, Settings, ArrowRight } from "lucide-vue-next";

const store = useRepoStore();
</script>

<template>
  <div class="flex-1 flex flex-col items-center justify-center p-8 text-center max-w-lg mx-auto">
    <div
      class="w-16 h-16 rounded-2xl bg-indigo-600/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 mb-6 shadow-inner"
    >
      <FolderGit2 class="w-8 h-8" />
    </div>

    <h2 class="text-xl font-bold text-slate-100 tracking-tight mb-2">
      Welcome to GitPulse
    </h2>

    <p class="text-sm text-slate-400 mb-6 leading-relaxed">
      A native desktop PR & Git dashboard for developers. Select a local Git repository and connect your GitHub credentials to get started.
    </p>

    <div class="flex flex-col sm:flex-row items-center gap-3 w-full sm:w-auto">
      <button
        @click="store.pickAndAddRepo()"
        class="w-full sm:w-auto px-5 py-2.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs shadow-lg shadow-indigo-950 flex items-center justify-center gap-2 transition cursor-pointer"
      >
        <span>Select Local Repository</span>
        <ArrowRight class="w-4 h-4" />
      </button>

      <button
        @click="store.isSettingsOpen = true"
        class="w-full sm:w-auto px-5 py-2.5 rounded-xl bg-slate-800 hover:bg-slate-700 text-slate-200 font-medium text-xs border border-slate-700/80 flex items-center justify-center gap-2 transition"
      >
        <Settings class="w-4 h-4 text-slate-400" />
        <span>Open Settings</span>
      </button>
    </div>
  </div>
</template>
