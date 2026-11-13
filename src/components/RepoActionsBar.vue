<script setup lang="ts">
import { ref, computed } from "vue";
import { useRepoStore } from "../stores/repoStore";
import type { CustomCommand } from "../types";
import {
  Play,
  Plus,
  Loader2,
  Trash2,
  Pencil,
  Terminal,
  X,
  CheckCircle,
  XCircle,
  Zap,
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

  await store.updateCommandInRepo(
    store.activeRepo.id,
    editingId.value,
    name,
    cmd,
    editAutoRun.value
  );
  editingId.value = null;
}

function handleRemove(cmdId: string) {
  if (!store.activeRepo) return;
  store.removeCommandFromRepo(store.activeRepo.id, cmdId);
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <!-- Small Actions Row (Only buttons with display names) -->
    <div
      class="bg-slate-900/40 border border-slate-800/60 rounded-xl px-3.5 py-2 flex flex-wrap items-center justify-between gap-2.5 text-xs shadow-sm"
    >
      <!-- Left: Actions Header & Buttons -->
      <div class="flex items-center gap-2 flex-wrap min-w-0">
        <div class="flex items-center gap-1.5 text-slate-400 font-semibold text-[11px] uppercase tracking-wider shrink-0 mr-1">
          <Zap class="w-3.5 h-3.5 text-amber-400" />
          <span>Actions</span>
        </div>

        <!-- Empty State if no actions -->
        <span
          v-if="activeCommands.length === 0 && !isAdding"
          class="text-slate-500 text-xs italic"
        >
          No repo actions configured
        </span>

        <!-- Action Buttons (Clean display names only) -->
        <div
          v-for="cmd in activeCommands"
          :key="cmd.id"
          class="group relative inline-flex items-center gap-1.5 pl-3 pr-2 py-1 rounded-lg bg-slate-950/80 border border-slate-800 hover:border-slate-700 hover:bg-slate-900 transition shadow-sm"
        >
          <!-- Display Name Button that runs command -->
          <button
            @click="handleRun(cmd)"
            :disabled="store.runningCommandId !== null"
            class="flex items-center gap-1.5 font-medium text-slate-200 hover:text-emerald-300 transition cursor-pointer disabled:opacity-50"
            :title="`Click to run\nCommand: ${cmd.command}`"
          >
            <Loader2
              v-if="store.runningCommandId === cmd.id"
              class="w-3 h-3 text-emerald-400 animate-spin"
            />
            <Play
              v-else
              class="w-3 h-3 text-emerald-400 fill-emerald-400/20 group-hover:fill-emerald-400 transition"
            />
            <span>{{ cmd.name }}</span>
          </button>

          <!-- Auto-pull tag -->
          <span
            v-if="cmd.autoRunOnPull"
            class="px-1 py-0.2 rounded text-[9px] uppercase font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30"
            title="Runs automatically whenever you pull"
          >
            Auto
          </span>

          <!-- Quick Edit & Delete icons on hover -->
          <div class="opacity-0 group-hover:opacity-100 flex items-center gap-0.5 ml-1 transition">
            <button
              @click.stop="startEdit(cmd)"
              class="p-0.5 text-slate-500 hover:text-amber-300 transition cursor-pointer"
              title="Edit action"
            >
              <Pencil class="w-3 h-3" />
            </button>
            <button
              @click.stop="handleRemove(cmd.id)"
              class="p-0.5 text-slate-500 hover:text-rose-400 transition cursor-pointer"
              title="Delete action"
            >
              <Trash2 class="w-3 h-3" />
            </button>
          </div>
        </div>

        <!-- Add Action Button -->
        <button
          v-if="!isAdding"
          @click="isAdding = true"
          class="inline-flex items-center gap-1 px-2.5 py-1 rounded-lg text-slate-400 hover:text-slate-200 bg-slate-950/40 border border-dashed border-slate-800 hover:border-slate-700 transition cursor-pointer font-medium text-xs"
          title="Add a custom command for this repo"
        >
          <Plus class="w-3 h-3" />
          <span>Add</span>
        </button>
      </div>

      <!-- Right: Latest Execution Indicator (Click to toggle output) -->
      <div v-if="lastRun" class="flex items-center gap-2 shrink-0">
        <button
          @click="showOutput = !showOutput"
          class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs transition cursor-pointer border"
          :class="[
            lastRun.result.success
              ? 'bg-emerald-950/40 border-emerald-900/60 text-emerald-300 hover:bg-emerald-950/70'
              : 'bg-rose-950/40 border-rose-900/60 text-rose-300 hover:bg-rose-950/70',
          ]"
          :title="showOutput ? 'Hide terminal output' : 'View terminal output'"
        >
          <component
            :is="lastRun.result.success ? CheckCircle : XCircle"
            class="w-3.5 h-3.5 shrink-0"
          />
          <span class="font-medium truncate max-w-[140px]">{{ lastRun.command.name }}</span>
          <span class="text-[10px] opacity-75 font-mono">({{ lastRun.result.duration_ms }}ms)</span>
          <Terminal class="w-3 h-3 ml-1 text-slate-400" />
        </button>
      </div>
    </div>

    <!-- Add Command Inline Form (Expands only when 'Add' is clicked) -->
    <div
      v-if="isAdding"
      class="p-3 bg-slate-900/90 border border-indigo-500/40 rounded-xl flex flex-col gap-2.5 text-xs shadow-lg animate-in fade-in duration-150"
    >
      <div class="flex items-center justify-between">
        <span class="font-semibold text-slate-200">Add Repo Action</span>
        <button
          @click="isAdding = false"
          class="text-slate-400 hover:text-white cursor-pointer"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
        <div class="flex flex-col gap-1">
          <label class="text-[11px] text-slate-400 font-medium">Display Name (Button Text)</label>
          <input
            v-model="newName"
            placeholder="e.g. Run Migration, Seed DB, Clean Cache"
            class="px-2.5 py-1.5 rounded-lg bg-slate-950 border border-slate-800 focus:border-indigo-500 focus:outline-none text-slate-200 placeholder-slate-600 text-xs"
            @keydown.enter="handleAdd"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] text-slate-400 font-medium">Terminal Command</label>
          <input
            v-model="newCommand"
            placeholder="e.g. ttdb deploy / npm run db:migrate"
            class="px-2.5 py-1.5 rounded-lg bg-slate-950 border border-slate-800 focus:border-indigo-500 focus:outline-none text-slate-200 placeholder-slate-600 text-xs font-mono"
            @keydown.enter="handleAdd"
          />
        </div>
      </div>

      <div class="flex items-center justify-between pt-1">
        <label class="flex items-center gap-2 cursor-pointer text-slate-300">
          <input
            type="checkbox"
            v-model="newAutoRun"
            class="rounded border-slate-700 bg-slate-950 text-indigo-500 focus:ring-0 focus:ring-offset-0"
          />
          <span class="text-xs">⚡ Auto-run automatically after Pulling</span>
        </label>

        <div class="flex items-center gap-2">
          <button
            @click="isAdding = false"
            class="px-2.5 py-1 rounded-lg text-slate-400 hover:text-white transition cursor-pointer"
          >
            Cancel
          </button>
          <button
            @click="handleAdd"
            :disabled="!newCommand.trim() && !newName.trim()"
            class="px-3 py-1 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium transition cursor-pointer disabled:opacity-50 shadow-sm"
          >
            Save Action
          </button>
        </div>
      </div>
    </div>

    <!-- Edit Command Inline Form -->
    <div
      v-if="editingId"
      class="p-3 bg-slate-900/90 border border-amber-500/40 rounded-xl flex flex-col gap-2.5 text-xs shadow-lg animate-in fade-in duration-150"
    >
      <div class="flex items-center justify-between">
        <span class="font-semibold text-amber-300">Edit Repo Action</span>
        <button
          @click="editingId = null"
          class="text-slate-400 hover:text-white cursor-pointer"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
        <div class="flex flex-col gap-1">
          <label class="text-[11px] text-slate-400 font-medium">Display Name (Button Text)</label>
          <input
            v-model="editName"
            class="px-2.5 py-1.5 rounded-lg bg-slate-950 border border-slate-800 focus:border-amber-500 focus:outline-none text-slate-200 text-xs"
            @keydown.enter="handleSaveEdit"
          />
        </div>
        <div class="flex flex-col gap-1">
          <label class="text-[11px] text-slate-400 font-medium">Terminal Command</label>
          <input
            v-model="editCommand"
            class="px-2.5 py-1.5 rounded-lg bg-slate-950 border border-slate-800 focus:border-amber-500 focus:outline-none text-slate-200 text-xs font-mono"
            @keydown.enter="handleSaveEdit"
          />
        </div>
      </div>

      <div class="flex items-center justify-between pt-1">
        <label class="flex items-center gap-2 cursor-pointer text-slate-300">
          <input
            type="checkbox"
            v-model="editAutoRun"
            class="rounded border-slate-700 bg-slate-950 text-amber-500 focus:ring-0 focus:ring-offset-0"
          />
          <span class="text-xs">⚡ Auto-run automatically after Pulling</span>
        </label>

        <div class="flex items-center gap-2">
          <button
            @click="editingId = null"
            class="px-2.5 py-1 rounded-lg text-slate-400 hover:text-white transition cursor-pointer"
          >
            Cancel
          </button>
          <button
            @click="handleSaveEdit"
            :disabled="!editCommand.trim()"
            class="px-3 py-1 rounded-lg bg-amber-600 hover:bg-amber-500 text-white font-medium transition cursor-pointer disabled:opacity-50 shadow-sm"
          >
            Update Action
          </button>
        </div>
      </div>
    </div>

    <!-- Output Drawer: Shows terminal stdout/stderr only when running or toggled -->
    <div
      v-if="showOutput && lastRun"
      class="bg-slate-950 border border-slate-800 rounded-xl p-3 flex flex-col gap-2 font-mono text-xs shadow-inner"
    >
      <div class="flex items-center justify-between text-slate-400 text-[11px]">
        <div class="flex items-center gap-2">
          <Terminal class="w-3.5 h-3.5 text-indigo-400" />
          <span class="text-slate-200 font-semibold">{{ lastRun.command.name }}</span>
          <span class="text-slate-500 text-[10px]">`{{ lastRun.command.command }}`</span>
        </div>
        <div class="flex items-center gap-3">
          <span
            :class="lastRun.result.success ? 'text-emerald-400' : 'text-rose-400'"
            class="font-semibold"
          >
            Exit {{ lastRun.result.exit_code }} ({{ lastRun.result.duration_ms }}ms)
          </span>
          <button
            @click="showOutput = false"
            class="text-slate-500 hover:text-slate-300 transition cursor-pointer"
            title="Close output"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <pre
        class="bg-slate-900/90 rounded-lg p-2.5 text-[11px] leading-relaxed overflow-x-auto max-h-48 text-slate-300 scrollbar-thin whitespace-pre-wrap select-text"
      >{{ lastRun.result.stdout || lastRun.result.stderr || "(No output produced)" }}</pre>
    </div>
  </div>
</template>
