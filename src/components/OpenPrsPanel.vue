<script setup lang="ts">
import { ref, computed } from "vue";
import { useRepoStore } from "../stores/repoStore";
import {
  GitPullRequest,
  GitMerge,
  GitBranch,
  User,
  Users,
  CheckCircle2,
  XCircle,
  Clock,
  MessageSquareWarning,
  ExternalLink,
  HelpCircle,
  ShieldCheck,
  ShieldAlert,
} from "lucide-vue-next";
import { openExternal } from "../services/opener";
import type { PullRequestInfo } from "../types";

const store = useRepoStore();

// Top-level scope: "me" vs "team"
const scope = ref<"me" | "team">("me");

// Sub-filter: "open" vs "merged"
const statusFilter = ref<"open" | "merged">("open");

const isTeamView = computed(() => scope.value === "team");
const isMergedView = computed(() => statusFilter.value === "merged");

const currentPRList = computed<PullRequestInfo[]>(() => {
  if (scope.value === "me") {
    return statusFilter.value === "open"
      ? store.openPullRequests
      : store.mergedPullRequests;
  } else {
    return statusFilter.value === "open"
      ? store.teamOpenPRs
      : store.teamMergedPRs;
  }
});

const currentOpenCount = computed(() => {
  return scope.value === "me"
    ? store.openPullRequests.length
    : store.teamOpenPRs.length;
});

const currentMergedCount = computed(() => {
  return scope.value === "me"
    ? store.mergedPullRequests.length
    : store.teamMergedPRs.length;
});

function formatPrAge(createdAt?: string | null): string {
  if (!createdAt) return "";
  const created = new Date(createdAt);
  const now = new Date();
  const diffMs = now.getTime() - created.getTime();
  const diffSecs = Math.max(0, Math.floor(diffMs / 1000));
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffDays > 0) {
    return diffDays === 1 ? "1 day ago" : `${diffDays} days ago`;
  }
  if (diffHours > 0) {
    return diffHours === 1 ? "1 hr ago" : `${diffHours} hrs ago`;
  }
  if (diffMins > 0) {
    return diffMins === 1 ? "1 min ago" : `${diffMins} mins ago`;
  }
  return "just now";
}

function getReviewBadge(decision: PullRequestInfo["review_decision"]) {
  if (!decision) {
    return {
      label: "Review Pending",
      badgeClass: "bg-slate-800 text-slate-400 border border-slate-700/60",
      icon: Clock,
    };
  }

  const d = decision.toUpperCase();
  if (d === "APPROVED") {
    return {
      label: "Approved",
      badgeClass: "bg-emerald-950/80 text-emerald-400 border border-emerald-800/70 shadow-sm shadow-emerald-950",
      icon: ShieldCheck,
    };
  }
  if (d === "CHANGES_REQUESTED") {
    return {
      label: "Changes Requested",
      badgeClass: "bg-rose-950/80 text-rose-300 border border-rose-800/70 shadow-sm shadow-rose-950",
      icon: ShieldAlert,
    };
  }
  return {
    label: "Review Required",
    badgeClass: "bg-amber-950/80 text-amber-300 border border-amber-800/70 shadow-sm shadow-amber-950",
    icon: Clock,
  };
}

function getCiBadge(status: PullRequestInfo["ci_status"]) {
  if (!status) {
    return {
      label: "No CI Checks",
      badgeClass: "text-slate-500",
      icon: HelpCircle,
    };
  }

  const s = status.toUpperCase();
  if (s === "SUCCESS") {
    return {
      label: "CI Passing",
      badgeClass: "text-emerald-400",
      icon: CheckCircle2,
    };
  }
  if (s === "FAILURE" || s === "ERROR") {
    return {
      label: "CI Failing",
      badgeClass: "text-rose-400",
      icon: XCircle,
    };
  }
  return {
    label: "CI Pending",
    badgeClass: "text-amber-400 animate-pulse",
    icon: Clock,
  };
}
</script>

<template>
  <div class="bg-slate-900/70 border border-slate-800/80 rounded-2xl p-5 flex flex-col h-full min-h-0 gap-3.5">
    <!-- Header: Title -->
    <div class="flex items-center justify-between pb-1">
      <div class="flex items-center gap-2">
        <GitPullRequest class="w-4 h-4 text-indigo-400 shrink-0" />
        <h3 class="text-sm font-semibold text-slate-200 uppercase tracking-wider">
          Pull Requests
        </h3>
      </div>
      <span class="text-xs text-slate-500 font-mono">
        {{ currentPRList.length }} shown
      </span>
    </div>

    <!-- 2 Compact Joined Segmented Controls (Row 1: Me / Team, Row 2: Open / Merged) -->
    <div class="flex flex-col gap-1.5 text-xs">
      <!-- Top Row: Joined Me / Team -->
      <div class="bg-slate-950/80 p-0.5 rounded-lg border border-slate-800/80 flex items-center">
        <button
          @click="scope = 'me'"
          class="flex-1 py-1 px-3 rounded-md transition cursor-pointer font-medium flex items-center justify-center gap-1.5"
          :class="[
            scope === 'me'
              ? 'bg-indigo-600 text-white font-semibold shadow-sm shadow-indigo-950/40'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          <User class="w-3.5 h-3.5 shrink-0" :class="scope === 'me' ? 'text-indigo-200' : 'text-slate-400'" />
          <span>Me</span>
          <span
            class="px-1.5 py-0.2 rounded-full text-[10px] font-mono font-bold"
            :class="scope === 'me' ? 'bg-indigo-800 text-indigo-100' : 'bg-slate-800 text-slate-400'"
          >
            {{ store.openPullRequests.length }}
          </span>
        </button>

        <button
          @click="scope = 'team'"
          class="flex-1 py-1 px-3 rounded-md transition cursor-pointer font-medium flex items-center justify-center gap-1.5"
          :class="[
            scope === 'team'
              ? 'bg-indigo-600 text-white font-semibold shadow-sm shadow-indigo-950/40'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          <Users class="w-3.5 h-3.5 shrink-0" :class="scope === 'team' ? 'text-indigo-200' : 'text-slate-400'" />
          <span>Team</span>
          <span
            class="px-1.5 py-0.2 rounded-full text-[10px] font-mono font-bold"
            :class="scope === 'team' ? 'bg-indigo-800 text-indigo-100' : 'bg-slate-800 text-slate-400'"
          >
            {{ store.teamOpenPRs.length }}
          </span>
        </button>
      </div>

      <!-- Bottom Row: Joined Open / Merged -->
      <div class="bg-slate-950/80 p-0.5 rounded-lg border border-slate-800/80 flex items-center">
        <button
          @click="statusFilter = 'open'"
          class="flex-1 py-1 px-3 rounded-md transition cursor-pointer font-medium flex items-center justify-center gap-1.5"
          :class="[
            statusFilter === 'open'
              ? 'bg-emerald-950/80 text-emerald-300 font-semibold shadow-sm border border-emerald-800/60'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          <GitPullRequest class="w-3.5 h-3.5 shrink-0" :class="statusFilter === 'open' ? 'text-emerald-400' : 'text-slate-400'" />
          <span>Open</span>
          <span
            class="px-1.5 py-0.2 rounded-full text-[10px] font-mono font-bold"
            :class="statusFilter === 'open' ? 'bg-emerald-500/20 text-emerald-300' : 'bg-slate-800 text-slate-400'"
          >
            {{ currentOpenCount }}
          </span>
        </button>

        <button
          @click="statusFilter = 'merged'"
          class="flex-1 py-1 px-3 rounded-md transition cursor-pointer font-medium flex items-center justify-center gap-1.5"
          :class="[
            statusFilter === 'merged'
              ? 'bg-purple-950/80 text-purple-300 font-semibold shadow-sm border border-purple-800/60'
              : 'text-slate-400 hover:text-slate-200'
          ]"
        >
          <GitMerge class="w-3.5 h-3.5 shrink-0" :class="statusFilter === 'merged' ? 'text-purple-400' : 'text-slate-400'" />
          <span>Merged</span>
          <span
            class="px-1.5 py-0.2 rounded-full text-[10px] font-mono font-bold"
            :class="statusFilter === 'merged' ? 'bg-purple-500/20 text-purple-300' : 'bg-slate-800 text-slate-400'"
          >
            {{ currentMergedCount }}
          </span>
        </button>
      </div>
    </div>

    <!-- GitHub Not Configured Notice -->
    <div
      v-if="!store.isGitHubConfigured"
      class="flex flex-col items-center justify-center p-8 text-center bg-slate-950/40 rounded-xl border border-slate-800/80 text-xs text-slate-400 flex-1"
    >
      <GitPullRequest class="w-8 h-8 text-slate-600 mb-2" />
      <p class="font-medium text-slate-300 mb-1">GitHub integration not fully configured</p>
      <p class="text-slate-500 mb-3 text-[11px]">Configure your GitHub Token and Repo Mapping to view pull requests.</p>
      <button
        @click="store.isSettingsOpen = true"
        class="px-3.5 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white font-medium text-xs transition shadow-sm cursor-pointer"
      >
        Open Settings to Fix
      </button>
    </div>

    <!-- PR List Content Area -->
    <div v-else class="flex flex-col flex-1 min-h-0">
      <!-- 1. PR Cards List (If items exist in active scope & filter) -->
      <div
        v-if="currentPRList.length"
        class="flex flex-col gap-2.5 flex-1 min-h-0 overflow-y-auto pr-1 scrollbar-thin"
      >
        <div
          v-for="pr in currentPRList"
          :key="pr.number"
          @click="openExternal(pr.url)"
          class="group p-3 rounded-xl bg-slate-950/40 hover:bg-slate-800/40 border border-slate-800/70 hover:border-slate-700 transition cursor-pointer shadow-sm flex flex-col gap-2 min-w-0"
        >
          <!-- Line 1: #PR + Title + External Link -->
          <div class="flex items-center justify-between gap-2.5 min-w-0">
            <div class="flex items-center gap-2 min-w-0 flex-1">
              <span
                class="font-mono text-xs font-bold shrink-0"
                :class="!isMergedView ? 'text-emerald-400' : 'text-purple-400'"
              >
                #{{ pr.number }}
              </span>

              <h4
                class="text-xs font-medium text-slate-100 group-hover:text-indigo-200 transition truncate"
                :title="pr.title"
              >
                {{ pr.title }}
              </h4>
            </div>

            <ExternalLink class="w-3.5 h-3.5 text-slate-600 group-hover:text-slate-300 transition shrink-0" />
          </div>

          <!-- Line 2: Dedicated Branch Context (Full Width) -->
          <div class="flex items-center gap-1.5 text-[11px] font-mono text-slate-300 min-w-0">
            <GitBranch class="w-3 h-3 text-indigo-400 shrink-0" />
            <span class="truncate" :title="pr.head_ref_name">{{ pr.head_ref_name }}</span>
          </div>

          <!-- Line 3: Footer Metadata (Author & Time on Left, Status Badges on Right) -->
          <div class="flex items-center justify-between gap-2 pt-2 border-t border-slate-800/60 min-w-0 text-[11px]">
            <!-- Left: Author & Time -->
            <div class="flex items-center gap-1.5 text-slate-400 shrink-0 min-w-0">
              <span v-if="isTeamView && pr.author_login" class="text-indigo-300 font-mono font-medium truncate max-w-[120px]">
                @{{ pr.author_login }}
              </span>
              <span v-if="isTeamView && pr.author_login" class="text-slate-600">•</span>
              <span class="text-slate-500 text-[10px] shrink-0">
                {{ !isMergedView ? `opened ${formatPrAge(pr.created_at)}` : `merged ${formatPrAge(pr.merged_at || pr.created_at)}` }}
              </span>
            </div>

            <!-- Right: Status Badges (Review, CI, Comments) -->
            <div class="flex items-center gap-1.5 shrink-0">
              <!-- Review Badge (Open) -->
              <div
                v-if="!isMergedView"
                class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium"
                :class="getReviewBadge(pr.review_decision).badgeClass"
              >
                <component :is="getReviewBadge(pr.review_decision).icon" class="w-3 h-3" />
                <span>{{ getReviewBadge(pr.review_decision).label }}</span>
              </div>

              <!-- Merged Badge (Merged) -->
              <span
                v-else
                class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-purple-500/10 text-purple-300 border border-purple-500/20"
              >
                <GitMerge class="w-3 h-3 text-purple-400" />
                <span>Merged</span>
              </span>

              <!-- CI Status -->
              <div
                v-if="!isMergedView && pr.ci_status"
                class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-slate-900 border border-slate-800"
                :class="getCiBadge(pr.ci_status).badgeClass"
                :title="getCiBadge(pr.ci_status).label"
              >
                <component :is="getCiBadge(pr.ci_status).icon" class="w-3 h-3 shrink-0" />
                <span class="text-[10px]">{{ getCiBadge(pr.ci_status).label }}</span>
              </div>

              <!-- Unresolved Threads -->
              <span
                v-if="!isMergedView && pr.unresolved_threads > 0"
                class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] bg-amber-950/70 text-amber-300 border border-amber-800/60 font-medium"
                :title="`${pr.unresolved_threads} unresolved conversation threads`"
              >
                <MessageSquareWarning class="w-3 h-3 text-amber-400" />
                <span>{{ pr.unresolved_threads }}</span>
              </span>
            </div>
          </div>
        </div>
      </div>

      <!-- 2. Empty State: When Current List is Empty -->
      <div v-else class="flex flex-col gap-3 flex-1 justify-center">
        <!-- If in Me > Open: Highlight Celebratory Card with Last Merged PR -->
        <template v-if="scope === 'me' && statusFilter === 'open'">
          <div class="p-4 rounded-xl bg-slate-950/40 border border-slate-800 text-center text-xs">
            <div class="w-8 h-8 rounded-full bg-emerald-500/10 text-emerald-400 flex items-center justify-center mx-auto mb-2">
              <CheckCircle2 class="w-4 h-4" />
            </div>
            <p class="font-medium text-slate-200 mb-0.5">All Your PRs Are Merged!</p>
            <p class="text-slate-500 text-[11px]">No active open PRs authored by you right now.</p>
          </div>

          <!-- Featured Last Merged PR Card -->
          <div
            v-if="store.lastMergedPR"
            @click="openExternal(store.lastMergedPR.url)"
            class="p-3.5 rounded-xl bg-purple-950/20 hover:bg-purple-950/35 border border-purple-900/50 hover:border-purple-800/80 transition cursor-pointer shadow-sm flex flex-col gap-2 min-w-0"
          >
            <div class="flex items-center justify-between gap-2 min-w-0">
              <div class="flex items-center gap-1.5 text-purple-400 text-xs font-semibold shrink-0">
                <GitMerge class="w-3.5 h-3.5" />
                <span>Your Last Merged PR #{{ store.lastMergedPR.number }}</span>
              </div>
              <span class="text-[11px] text-purple-300/80 font-sans shrink-0 font-medium">
                Merged {{ formatPrAge(store.lastMergedPR.merged_at || store.lastMergedPR.created_at) }}
              </span>
            </div>

            <p class="text-xs font-medium text-slate-200 line-clamp-1 min-w-0" :title="store.lastMergedPR.title">
              {{ store.lastMergedPR.title }}
            </p>

            <div class="flex items-center justify-between gap-2 pt-1.5 border-t border-purple-900/30 min-w-0 text-[11px]">
              <div class="flex items-center gap-1.5 bg-slate-900/90 px-2 py-0.5 rounded border border-slate-800 text-slate-300 font-mono text-[10px] min-w-0 max-w-[65%]">
                <GitBranch class="w-3 h-3 text-purple-400 shrink-0" />
                <span class="truncate" :title="store.lastMergedPR.head_ref_name">
                  {{ store.lastMergedPR.head_ref_name }}
                </span>
              </div>

              <span class="text-purple-400 hover:text-purple-300 flex items-center gap-1 font-sans font-medium shrink-0">
                View on GitHub <ExternalLink class="w-3 h-3" />
              </span>
            </div>
          </div>
        </template>

        <!-- General Empty Fallback -->
        <div v-else class="flex flex-col items-center justify-center p-8 text-center text-slate-500 text-xs flex-1">
          <component
            :is="!isMergedView ? GitPullRequest : GitMerge"
            class="w-8 h-8 text-slate-700 mb-2"
          />
          <p class="font-medium text-slate-300 mb-1">
            {{
              isTeamView
                ? (isMergedView ? 'No merged team PRs found' : 'No open team PRs in this repo')
                : (isMergedView ? 'No merged PRs authored by you' : 'No open PRs authored by you')
            }}
          </p>
          <p class="text-slate-600 text-[11px]">
            {{ isTeamView ? 'Click another status card above to view repository activity.' : 'Click Team Open above to see what colleagues are shipping.' }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
