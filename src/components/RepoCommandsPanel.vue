<script setup lang="ts">
import { ref, computed } from "vue";
import { useRepoStore } from "../stores/repoStore";
import type { CustomCommand } from "../types";
import {
  Terminal,
  Play,
  Plus,
  Trash2,
  CheckCircle,
  XCircle,
  Loader2,
  X,
  Sparkles,
  Pencil,
  Check,
} from "lucide-vue-next";

const store = useRepoStore();

const isAdding = ref(false);
const newName = ref("");
const newCommand = ref("");
const newAutoRun = ref(false);
const showOutput = ref(false);

const editingId = ref<string | null>(null);
const editName = ref("");
const editCommand = ref("");
const editAutoRun = ref(false);

const activeCommands = computed<CustomCommand[]>(() => {
  if (!store.activeRepo) return [];
  const r = store.settings.repositories.find((x) => x.id === store.activeRepo?.id);
  return r?.commands || store.activeRepo.commands || [];
});

const lastRun = computed(() => {
  if (
    store.lastCommandExecution &&
    store.lastCommandExecution.repoId === store.activeRepo?.id
  ) {
    return store.lastCommandExecution;
  }
  return null;
});

async function handleRun(cmd: CustomCommand) {
  showOutput.value = true;
  await store.executeRepoCommand(cmd);
}

async function handleAdd() {
  const cmd = newCommand.value.trim() || newName.value.trim();
  if (!cmd || !store.activeRepo) return;
  const label = newName.value.trim() || cmd;
  await store.addCommandToRepo(store.activeRepo.id, label, cmd, newAutoRun.value);
  newName.value = "";
  newCommand.value = "";
  newAutoRun.value = false;
  isAdding.value = false;
}

function startEdit(cmd: CustomCommand) {
  editingId.value = cmd.id;
  editName.value = cmd.name;
  editCommand.value = cmd.command;
  editAutoRun.value = !!cmd.autoRunOnPull;
}

async function handleSaveEdit() {
  if (!editingId.value || !store.activeRepo) return;
  const cmd = editCommand.value.trim();
  const name = editName.value.trim() || cmd;
  if (!cmd) return;

  await store.updateCommandInRepo(store.activeRepo.id, editingId.value, name, cmd, editAutoRun.value);
  editingId.value = null;
}

function cancelEdit() {
  editingId.value = null;
}

function handleRemove(cmdId: string) {
  if (!store.activeRepo) return;
  store.removeCommandFromRepo(store.activeRepo.id, cmdId);
}
</script>

<template>
  <div
    v-if="store.activeRepo"
    class="bg-slate-900/70 border border-slate-800/80 rounded-2xl p-5 shadow-lg flex flex-col gap-4"
  >
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <Terminal class="w-4 h-4 text-emerald-400" />
        <h3 class="text-sm font-semibold text-slate-200 uppercase tracking-wider">
          Custom Actions & Commands
        </h3>
        <span class="text-xs text-slate-500 font-mono">({{ activeCommands.length }})</span>
      </div>

      <button
        v-if="!isAdding"
        @click="isAdding = true"
        class="flex items-center gap-1 px-2.5 py-1 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-white text-xs font-medium border border-slate-700/80 transition cursor-pointer"
        title="Add a custom command for this repository"
      >
        <Plus class="w-3.5 h-3.5 text-emerald-400" />
        <span>Add Command</span>
      </button>
    </div>

    <!-- Inline Add Command Form -->
    <form
      v-if="isAdding"
      @submit.prevent="handleAdd"
      class="p-4 bg-slate-950/90 border border-indigo-500/40 rounded-xl space-y-3 transition shadow-md shadow-indigo-950/20"
    >
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-slate-200 flex items-center gap-1.5">
          <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
          Add Command for <strong class="text-indigo-300 font-mono">{{ store.activeRepo.name }}</strong>
        </span>
        <button
          type="button"
          @click="isAdding = false"
          class="text-slate-500 hover:text-slate-300 p-0.5 rounded cursor-pointer"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <!-- Display Label field -->
        <div class="space-y-1">
          <label class="block text-[11px] font-semibold text-slate-300 uppercase tracking-wide">
            Display Label
          </label>
          <input
            v-model="newName"
            type="text"
            placeholder="e.g. Run Migration, Seed DB"
            class="w-full bg-slate-900 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-500"
            autofocus
          />
        </div>

        <!-- Shell Command field -->
        <div class="space-y-1">
          <label class="block text-[11px] font-semibold text-slate-300 uppercase tracking-wide">
            Shell Command To Run <span class="text-rose-400">*</span>
          </label>
          <input
            v-model="newCommand"
            type="text"
            placeholder="e.g. ttdb deploy ..., npm run migrate"
            class="w-full bg-slate-900 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-indigo-300 placeholder-slate-600 focus:outline-none focus:border-indigo-500 font-mono"
          />
        </div>
      </div>

      <!-- Auto-run on Pull checkbox -->
      <label class="flex items-center gap-2 cursor-pointer pt-1 text-xs text-slate-300 select-none">
        <input
          type="checkbox"
          v-model="newAutoRun"
          class="rounded text-indigo-600 focus:ring-indigo-500 bg-slate-900 border-slate-700 cursor-pointer"
        />
        <span>⚡ Automatically run this command whenever I click <strong>Pull</strong> for this repository</span>
      </label>

      <div class="flex items-center justify-end gap-2 pt-1 border-t border-slate-800/60">
        <button
          type="button"
          @click="isAdding = false"
          class="px-3 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition cursor-pointer"
        >
          Cancel
        </button>
        <button
          type="submit"
          :disabled="!newCommand.trim() && !newName.trim()"
          class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white transition disabled:opacity-40 cursor-pointer shadow-md shadow-indigo-950 flex items-center gap-1.5"
        >
          <Plus class="w-3.5 h-3.5" />
          <span>Save Command</span>
        </button>
      </div>
    </form>

    <!-- Inline Edit Form (when editing an existing command) -->
    <form
      v-if="editingId"
      @submit.prevent="handleSaveEdit"
      class="p-4 bg-slate-950/90 border border-amber-500/40 rounded-xl space-y-3 transition shadow-md shadow-amber-950/20"
    >
      <div class="flex items-center justify-between">
        <span class="text-xs font-semibold text-amber-300 flex items-center gap-1.5">
          <Pencil class="w-3.5 h-3.5" />
          Edit Action
        </span>
        <button
          type="button"
          @click="cancelEdit"
          class="text-slate-500 hover:text-slate-300 p-0.5 rounded cursor-pointer"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>

      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div class="space-y-1">
          <label class="block text-[11px] font-semibold text-slate-300 uppercase tracking-wide">
            Display Label
          </label>
          <input
            v-model="editName"
            type="text"
            class="w-full bg-slate-900 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-slate-200 focus:outline-none focus:border-amber-500"
          />
        </div>
        <div class="space-y-1">
          <label class="block text-[11px] font-semibold text-slate-300 uppercase tracking-wide">
            Shell Command To Run
          </label>
          <input
            v-model="editCommand"
            type="text"
            class="w-full bg-slate-900 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-amber-300 focus:outline-none focus:border-amber-500 font-mono"
          />
        </div>
      </div>

      <!-- Auto-run on Pull checkbox in edit -->
      <label class="flex items-center gap-2 cursor-pointer pt-1 text-xs text-slate-300 select-none">
        <input
          type="checkbox"
          v-model="editAutoRun"
          class="rounded text-amber-500 focus:ring-amber-500 bg-slate-900 border-slate-700 cursor-pointer"
        />
        <span>⚡ Automatically run this command whenever I click <strong>Pull</strong> for this repository</span>
      </label>

      <div class="flex items-center justify-end gap-2 pt-1 border-t border-slate-800/60">
        <button
          type="button"
          @click="cancelEdit"
          class="px-3 py-1.5 rounded-lg text-xs font-medium text-slate-400 hover:text-slate-200 hover:bg-slate-800 transition cursor-pointer"
        >
          Cancel
        </button>
        <button
          type="submit"
          :disabled="!editCommand.trim()"
          class="px-4 py-1.5 rounded-lg text-xs font-semibold bg-amber-600 hover:bg-amber-500 text-white transition disabled:opacity-40 cursor-pointer shadow-md shadow-amber-950 flex items-center gap-1.5"
        >
          <Check class="w-3.5 h-3.5" />
          <span>Update Action</span>
        </button>
      </div>
    </form>

    <!-- Commands List -->
    <div v-if="activeCommands.length > 0" class="flex flex-wrap gap-2.5 items-center">
      <div
        v-for="cmd in activeCommands"
        :key="cmd.id"
        class="group flex items-center gap-2 p-1.5 pl-3 rounded-xl bg-slate-950/60 border border-slate-800 hover:border-slate-700 transition"
      >
        <div class="min-w-0 pr-1">
          <div class="flex items-center gap-1.5">
            <span class="font-medium text-xs text-slate-200 truncate">{{ cmd.name }}</span>
            <span
              v-if="cmd.autoRunOnPull"
              class="px-1.5 py-0.2 rounded text-[9px] font-bold tracking-wider uppercase bg-amber-500/20 text-amber-300 border border-amber-500/30 shrink-0"
              title="Runs automatically whenever you click Pull"
            >
              ⚡ Auto-Pull
            </span>
          </div>
          <div class="font-mono text-[11px] text-slate-500 truncate max-w-[220px]" :title="cmd.command">
            {{ cmd.command }}
          </div>
        </div>

        <!-- Run button -->
        <button
          @click="handleRun(cmd)"
          :disabled="store.runningCommandId !== null"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-emerald-950/80 hover:bg-emerald-900 text-emerald-300 border border-emerald-800/80 transition cursor-pointer disabled:opacity-50 shrink-0 shadow-sm"
        >
          <Loader2
            v-if="store.runningCommandId === cmd.id"
            class="w-3.5 h-3.5 animate-spin text-emerald-400"
          />
          <Play v-else class="w-3.5 h-3.5 text-emerald-400 fill-emerald-400" />
          <span>{{ store.runningCommandId === cmd.id ? "Running..." : "Run" }}</span>
        </button>

        <!-- Edit button -->
        <button
          @click="startEdit(cmd)"
          class="p-1.5 text-slate-600 hover:text-amber-300 rounded-lg hover:bg-slate-800/80 opacity-40 group-hover:opacity-100 transition cursor-pointer"
          title="Edit command"
        >
          <Pencil class="w-3.5 h-3.5" />
        </button>

        <!-- Delete button -->
        <button
          @click="handleRemove(cmd.id)"
          class="p-1.5 text-slate-600 hover:text-rose-400 rounded-lg hover:bg-slate-800/80 opacity-40 group-hover:opacity-100 transition cursor-pointer"
          title="Delete command"
        >
          <Trash2 class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Empty state for commands -->
    <div
      v-else-if="!isAdding"
      class="flex items-center justify-between p-3.5 bg-slate-950/30 rounded-xl border border-dashed border-slate-800/80 text-xs text-slate-400"
    >
      <div class="flex items-center gap-2">
        <Terminal class="w-4 h-4 text-slate-600" />
        <span>No custom actions defined for <strong>{{ store.activeRepo?.name || 'this repo' }}</strong> yet.</span>
      </div>
      <button
        @click="isAdding = true"
        class="text-indigo-400 hover:text-indigo-300 text-xs font-medium underline cursor-pointer"
      >
        Add your first command (e.g. post-pull migrations)
      </button>
    </div>

    <!-- Terminal Output Panel (Expandable) -->
    <div
      v-if="lastRun && showOutput"
      class="mt-1 bg-slate-950 border border-slate-800 rounded-xl overflow-hidden text-xs font-mono shadow-inner"
    >
      <!-- Output Header -->
      <div class="bg-slate-900/90 px-3.5 py-2 border-b border-slate-800/80 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <component
            :is="lastRun.result.success ? CheckCircle : XCircle"
            class="w-3.5 h-3.5"
            :class="lastRun.result.success ? 'text-emerald-400' : 'text-rose-400'"
          />
          <span class="font-semibold text-slate-200">{{ lastRun.command.name }}</span>
          <span class="text-slate-500">({{ lastRun.result.duration_ms }}ms)</span>
          <span
            class="px-1.5 py-0.2 rounded text-[10px] uppercase font-bold"
            :class="lastRun.result.success ? 'bg-emerald-950 text-emerald-400 border border-emerald-800' : 'bg-rose-950 text-rose-400 border border-rose-800'"
          >
            {{ lastRun.result.success ? 'Exit 0' : `Exit ${lastRun.result.exit_code ?? 'Err'}` }}
          </span>
        </div>

        <button
          @click="showOutput = false"
          class="p-1 text-slate-500 hover:text-slate-300 rounded cursor-pointer"
          title="Close output"
        >
          <X class="w-3 h-3" />
        </button>
      </div>

      <!-- Log Content -->
      <div class="p-3 max-h-52 overflow-y-auto space-y-1 select-text">
        <div v-if="lastRun.result.stdout" class="text-slate-300 whitespace-pre-wrap leading-relaxed">
          {{ lastRun.result.stdout }}
        </div>
        <div v-if="lastRun.result.stderr" class="text-rose-400 whitespace-pre-wrap leading-relaxed">
          {{ lastRun.result.stderr }}
        </div>
        <div
          v-if="!lastRun.result.stdout && !lastRun.result.stderr"
          class="text-slate-500 italic"
        >
          (Command completed with no terminal output)
        </div>
      </div>
    </div>
  </div>
</template>
