import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  RepoConfig,
  RepoContext,
  GitHubContext,
  PullRequestInfo,
  RepoSummary,
  CustomCommand,
  CommandExecutionResult,
} from "../types";
import { loadSettings, saveSettings } from "../services/storage";

export const useRepoStore = defineStore("repo", () => {
  const settings = ref<AppSettings>({
    repositories: [],
    activeRepoId: "",
    githubToken: "",
    githubUsername: "",
  });

  const repoContext = ref<RepoContext | null>(null);
  const githubContext = ref<GitHubContext | null>(null);
  const repoSummaries = ref<Record<string, RepoSummary>>({});

  const isLoading = ref(false);
  const isRefreshing = ref(false);
  const isSettingsOpen = ref(false);

  const gitError = ref<string | null>(null);
  const githubError = ref<string | null>(null);
  const isPulling = ref(false);
  const isPushing = ref(false);
  const isSyncingWithMain = ref(false);
  const isCheckingOut = ref<string | null>(null);
  const gitActionNotification = ref<{ type: "success" | "error"; text: string } | null>(null);
  const runningCommandId = ref<string | null>(null);
  const lastCommandExecution = ref<{
    command: CustomCommand;
    result: CommandExecutionResult;
    repoId: string;
  } | null>(null);
  const commandError = ref<string | null>(null);
  const lastUpdated = ref<Date | null>(null);
  const refreshSecondsAgo = ref(0);

  let timerId: any = null;
  let secondsTimerId: any = null;

  const activeRepo = computed<RepoConfig | null>(() => {
    if (!settings.value.repositories.length) return null;
    return (
      settings.value.repositories.find(
        (r) => r.id === settings.value.activeRepoId
      ) || settings.value.repositories[0]
    );
  });

  const activeGitHubRepo = computed<string>(() => {
    return (
      activeRepo.value?.githubRepo?.trim() ||
      repoContext.value?.detected_github_repo?.trim() ||
      settings.value.githubRepo?.trim() ||
      ""
    );
  });

  const isGitHubConfigured = computed<boolean>(() => {
    return Boolean(
      settings.value.githubToken.trim() &&
      settings.value.githubUsername.trim() &&
      activeGitHubRepo.value
    );
  });

  const openPullRequests = computed<PullRequestInfo[]>(() => {
    if (!githubContext.value?.pull_requests) return [];
    return githubContext.value.pull_requests.filter(
      (pr) => pr.state.toUpperCase() === "OPEN"
    );
  });

  const mergedPullRequests = computed<PullRequestInfo[]>(() => {
    if (!githubContext.value?.pull_requests) return [];
    return githubContext.value.pull_requests
      .filter((pr) => pr.state.toUpperCase() === "MERGED")
      .sort((a, b) => {
        const timeA = new Date(a.merged_at || a.created_at).getTime();
        const timeB = new Date(b.merged_at || b.created_at).getTime();
        return timeB - timeA;
      });
  });

  const teamOpenPRs = computed<PullRequestInfo[]>(() => {
    return githubContext.value?.team_open_prs || [];
  });

  const teamMergedPRs = computed<PullRequestInfo[]>(() => {
    return githubContext.value?.team_merged_prs || [];
  });

  const lastRepoMergedPR = computed<PullRequestInfo | null>(() => {
    return githubContext.value?.last_repo_merged_pr || null;
  });

  const lastMergedPR = computed<PullRequestInfo | null>(() => {
    return githubContext.value?.last_repo_merged_pr || mergedPullRequests.value[0] || null;
  });

  const lastUpdatedText = computed(() => {
    if (!lastUpdated.value) return "Never";
    if (refreshSecondsAgo.value < 5) return "Just now";
    if (refreshSecondsAgo.value < 60) return `${refreshSecondsAgo.value}s ago`;
    const mins = Math.floor(refreshSecondsAgo.value / 60);
    return `${mins}m ago`;
  });

  function getBranchPrStatus(branchName: string): {
    status: "none" | "open" | "merged" | "closed";
    label: string;
    badgeClass: string;
    pr?: PullRequestInfo;
  } {
    if (!githubContext.value?.pull_requests?.length) {
      return {
        status: "none",
        label: "Local only",
        badgeClass: "bg-slate-800 text-slate-400 border border-slate-700/60",
      };
    }

    const pr = githubContext.value.pull_requests.find(
      (p) => p.head_ref_name.toLowerCase() === branchName.toLowerCase()
    );

    if (!pr) {
      return {
        status: "none",
        label: "Local only",
        badgeClass: "bg-slate-800 text-slate-400 border border-slate-700/60",
      };
    }

    const state = pr.state.toUpperCase();
    if (state === "OPEN") {
      return {
        status: "open",
        label: `Active PR #${pr.number}`,
        badgeClass:
          "bg-blue-950/80 text-blue-400 border border-blue-800/60 shadow-sm shadow-blue-950",
        pr,
      };
    }
    if (state === "MERGED") {
      return {
        status: "merged",
        label: `Merged PR #${pr.number}`,
        badgeClass:
          "bg-purple-950/80 text-purple-300 border border-purple-800/60 shadow-sm shadow-purple-950",
        pr,
      };
    }
    return {
      status: "closed",
      label: `Closed PR #${pr.number}`,
      badgeClass:
        "bg-rose-950/80 text-rose-300 border border-rose-800/60 shadow-sm shadow-rose-950",
      pr,
    };
  }

  async function pickRepoFolder(): Promise<string | null> {
    try {
      const selected = await invoke<string | null>("pick_folder");
      return selected;
    } catch (e: any) {
      console.warn("Folder picker failed:", e);
      return null;
    }
  }

  async function addRepository(path: string, name?: string, githubRepo?: string): Promise<RepoConfig | null> {
    const trimmedPath = path.trim();
    if (!trimmedPath) return null;

    // Check if already in list
    const existing = settings.value.repositories.find((r) => r.path === trimmedPath);
    if (existing) {
      await setActiveRepo(existing.id);
      gitActionNotification.value = {
        type: "success",
        text: `Switched to existing repo "${existing.name}"`,
      };
      setTimeout(() => {
        gitActionNotification.value = null;
      }, 4000);
      return existing;
    }

    // Try inspecting to get actual repo name & remote
    let detectedName = name?.trim();
    let detectedGh = githubRepo?.trim();

    try {
      const testCtx = await invoke<RepoContext>("get_repo_context", { path: trimmedPath });
      if (!detectedName) detectedName = testCtx.repo_name;
      if (!detectedGh && testCtx.detected_github_repo) detectedGh = testCtx.detected_github_repo;
    } catch {
      if (!detectedName) {
        detectedName = trimmedPath.split(/[\\/]/).filter(Boolean).pop() || "Repo";
      }
    }

    const newRepo: RepoConfig = {
      id: "repo-" + Date.now() + "-" + Math.random().toString(36).slice(2, 6),
      name: detectedName || "Repo",
      path: trimmedPath,
      githubRepo: detectedGh || "",
    };

    settings.value.repositories.push(newRepo);
    settings.value.activeRepoId = newRepo.id;

    await saveCurrentSettings();
    await refreshAll();

    gitActionNotification.value = {
      type: "success",
      text: `Added "${newRepo.name}"${detectedGh ? ` (GitHub: ${detectedGh})` : ""}`,
    };
    setTimeout(() => {
      gitActionNotification.value = null;
    }, 5000);

    return newRepo;
  }

  async function inspectRepoPath(path: string): Promise<RepoContext | null> {
    const trimmed = path.trim();
    if (!trimmed) return null;
    try {
      return await invoke<RepoContext>("get_repo_context", { path: trimmed });
    } catch {
      return null;
    }
  }

  async function pickAndAddRepo(): Promise<RepoConfig | null> {
    const chosen = await pickRepoFolder();
    if (chosen) {
      return await addRepository(chosen);
    }
    return null;
  }

  async function removeRepository(id: string) {
    const idx = settings.value.repositories.findIndex((r) => r.id === id);
    if (idx !== -1) {
      settings.value.repositories.splice(idx, 1);
      delete repoSummaries.value[id];

      if (settings.value.activeRepoId === id) {
        settings.value.activeRepoId =
          settings.value.repositories.length > 0
            ? settings.value.repositories[0].id
            : "";
      }
      await saveCurrentSettings();
      await refreshAll();
    }
  }

  async function setActiveRepo(id: string) {
    if (settings.value.activeRepoId === id) return;
    settings.value.activeRepoId = id;
    await saveCurrentSettings();
    await refreshAll();
  }

  async function updateRepository(id: string, updates: Partial<RepoConfig>) {
    const target = settings.value.repositories.find((r) => r.id === id);
    if (target) {
      Object.assign(target, updates);
      await saveCurrentSettings();
      if (settings.value.activeRepoId === id) {
        await refreshAll();
      }
    }
  }

  async function pullRepo(repo: RepoConfig, isDaily = false, customMode?: "all" | "main_only" | "current_only") {
    if (!repo?.path) return;
    const today = new Date().toLocaleDateString("en-CA");

    // Safety check for daily auto-pull: verify working tree has no tracked modifications
    if (isDaily) {
      try {
        const ctx = await invoke<RepoContext>("get_repo_context", { path: repo.path });
        // Only block if there are tracked modifications (not untracked '??' files)
        const trackedChanges = (ctx.changed_files || []).filter((f) => !f.startsWith("??"));
        if (trackedChanges.length > 0) {
          const sampleFiles = trackedChanges
            .slice(0, 2)
            .map((f) => f.slice(2).trim())
            .join(", ");
          const suffix = trackedChanges.length > 2 ? ` (+${trackedChanges.length - 2} more)` : "";

          gitActionNotification.value = {
            type: "error",
            text: `Skipped daily auto-pull for ${repo.name}: uncommitted changes in ${sampleFiles}${suffix}`,
          };

          // Record attempt date so it doesn't loop every 30 seconds
          repo.lastDailyPullAttemptDate = today;
          await saveCurrentSettings();

          setTimeout(() => {
            if (gitActionNotification.value?.text.includes(repo.name)) {
              gitActionNotification.value = null;
            }
          }, 8000);
          return;
        }
      } catch (err) {
        console.warn(`Could not verify working tree for ${repo.name}:`, err);
      }
    }

    if (repo.id === activeRepo.value?.id) {
      isPulling.value = true;
    }
    gitActionNotification.value = null;

    try {
      const mode = customMode || repo.autoPullMode || "all";
      const res = await invoke<{ success: boolean; message: string }>("git_pull", {
        path: repo.path,
        mode,
      });

      if (isDaily) {
        repo.lastDailyPullDate = today;
        repo.lastDailyPullAttemptDate = undefined;
        await saveCurrentSettings();
      }

      gitActionNotification.value = {
        type: "success",
        text: isDaily
          ? `⚡ Daily auto-pull complete for ${repo.name}`
          : (res.message || `Git pull successful for ${repo.name}`),
      };
      await refreshAll();

      // Check for auto-run commands configured for this repo
      const autoCmds = repo.commands?.filter((c) => c.autoRunOnPull) || [];
      if (autoCmds.length > 0) {
        gitActionNotification.value = {
          type: "success",
          text: `Pulled ${repo.name}! Auto-running ${autoCmds.map((c) => c.name).join(", ")}...`,
        };
        for (const cmd of autoCmds) {
          await executeRepoCommand(cmd, repo);
        }
      }

      setTimeout(() => {
        gitActionNotification.value = null;
      }, 5000);
      return res;
    } catch (e: any) {
      if (isDaily) {
        repo.lastDailyPullAttemptDate = today;
        await saveCurrentSettings();
      }
      const msg = typeof e === "string" ? e : (e?.message || `Git pull failed for ${repo.name}`);
      gitActionNotification.value = {
        type: "error",
        text: msg,
      };
    } finally {
      if (repo.id === activeRepo.value?.id) {
        isPulling.value = false;
      }
    }
  }

  async function pullCurrentRepo(mode?: "all" | "main_only" | "current_only") {
    if (!activeRepo.value) return;
    return await pullRepo(activeRepo.value, false, mode);
  }

  async function syncWithMain(strategy: "rebase" | "merge" = "rebase") {
    const current = activeRepo.value;
    if (!current?.path) return;

    isSyncingWithMain.value = true;
    gitActionNotification.value = null;

    try {
      const res = await invoke<{ success: boolean; message: string }>("git_sync_with_main", {
        path: current.path,
        strategy,
      });

      gitActionNotification.value = {
        type: "success",
        text: res.message || `Synced with ${repoContext.value?.default_branch || "main"}`,
      };
      await refreshAll();
      setTimeout(() => {
        gitActionNotification.value = null;
      }, 5000);
      return res;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Sync with main failed");
      gitActionNotification.value = {
        type: "error",
        text: msg,
      };
      await refreshAll();
    } finally {
      isSyncingWithMain.value = false;
    }
  }

  async function fastForwardMain(repo?: RepoConfig) {
    const target = repo || activeRepo.value;
    if (!target?.path) return;

    try {
      const res = await invoke<{ success: boolean; message: string }>("fast_forward_main", {
        path: target.path,
      });
      gitActionNotification.value = {
        type: "success",
        text: res.message || "Fast-forwarded main",
      };
      await refreshAll();
      setTimeout(() => {
        gitActionNotification.value = null;
      }, 5000);
      return res;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Failed to fast-forward main");
      gitActionNotification.value = {
        type: "error",
        text: msg,
      };
    }
  }

  async function abortRebase() {
    const current = activeRepo.value;
    if (!current?.path) return;

    try {
      const res = await invoke<{ success: boolean; message: string }>("git_rebase_abort", {
        path: current.path,
      });
      gitActionNotification.value = {
        type: "success",
        text: res.message || "Rebase aborted successfully",
      };
      await refreshAll();
      setTimeout(() => {
        gitActionNotification.value = null;
      }, 5000);
      return res;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Failed to abort rebase");
      gitActionNotification.value = {
        type: "error",
        text: msg,
      };
    }
  }

  async function continueRebase() {
    const current = activeRepo.value;
    if (!current?.path) return;

    try {
      const res = await invoke<{ success: boolean; message: string }>("git_rebase_continue", {
        path: current.path,
      });
      gitActionNotification.value = {
        type: "success",
        text: res.message || "Rebase continued successfully",
      };
      await refreshAll();
      setTimeout(() => {
        gitActionNotification.value = null;
      }, 5000);
      return res;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Failed to continue rebase");
      gitActionNotification.value = {
        type: "error",
        text: msg,
      };
    }
  }

  async function checkoutBranch(branch: string) {
    const current = activeRepo.value;
    if (!current || !current.path) return;

    isCheckingOut.value = branch;
    try {
      const res = await invoke<{ success: boolean; message: string }>("git_checkout", {
        path: current.path,
        branch,
      });

      gitActionNotification.value = {
        type: "success",
        text: res.message || `Switched to branch '${branch}'`,
      };
      await refreshAll();
      setTimeout(() => {
        gitActionNotification.value = null;
      }, 5000);
      return res;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Failed to switch branch");
      gitActionNotification.value = {
        type: "error",
        text: msg,
      };
    } finally {
      isCheckingOut.value = null;
    }
  }

  async function openInEditor() {
    const current = activeRepo.value;
    if (!current?.path) return;
    try {
      const editorSetting =
        settings.value.defaultEditor === "custom"
          ? (settings.value.customEditorCmd ? `cmd:${settings.value.customEditorCmd}` : "code")
          : (settings.value.defaultEditor || "code");
      await invoke("open_in_editor", { path: current.path, editor: editorSetting });
    } catch (e: any) {
      console.warn("Failed to open editor:", e);
    }
  }

  async function openInTerminal() {
    const current = activeRepo.value;
    if (!current?.path) return;
    try {
      const terminalSetting =
        settings.value.defaultTerminal === "custom"
          ? (settings.value.customTerminalCmd ? `cmd:${settings.value.customTerminalCmd}` : "Terminal")
          : (settings.value.defaultTerminal || "Terminal");
      await invoke("open_in_terminal", { path: current.path, terminal: terminalSetting });
    } catch (e: any) {
      console.warn("Failed to open terminal:", e);
    }
  }

  async function pushCurrentRepo() {
    const current = activeRepo.value;
    if (!current || !current.path) return;

    isPushing.value = true;
    gitActionNotification.value = null;

    try {
      const res = await invoke<{ success: boolean; message: string }>("git_push", {
        path: current.path,
      });

      gitActionNotification.value = {
        type: "success",
        text: res.message || "Git push successful",
      };
      await refreshAll();
      setTimeout(() => {
        gitActionNotification.value = null;
      }, 5000);
      return res;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Git push failed");
      gitActionNotification.value = {
        type: "error",
        text: msg,
      };
    } finally {
      isPushing.value = false;
    }
  }

  async function executeRepoCommand(cmd: CustomCommand, targetRepo?: RepoConfig) {
    const current = targetRepo || activeRepo.value;
    if (!current || !current.path) return;

    runningCommandId.value = cmd.id;
    commandError.value = null;

    try {
      const res = await invoke<CommandExecutionResult>("run_repo_command", {
        repoPath: current.path,
        command: cmd.command,
      });

      lastCommandExecution.value = {
        command: cmd,
        result: res,
        repoId: current.id,
      };

      // Also trigger a refresh to reflect any branch or commit changes
      await refreshAll();
      return res;
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Command failed");
      commandError.value = msg;
    } finally {
      runningCommandId.value = null;
    }
  }

  async function addCommandToRepo(repoId: string, name: string, command: string, autoRunOnPull = false) {
    const rawCmd = command.trim() || name.trim();
    if (!rawCmd) return;

    const finalName = name.trim() || rawCmd;

    let repo = settings.value.repositories.find((r) => r.id === repoId);
    if (!repo && activeRepo.value) {
      repo = activeRepo.value;
    }
    if (!repo) return;

    const newCmd: CustomCommand = {
      id: "cmd-" + Date.now() + "-" + Math.random().toString(36).slice(2, 6),
      name: finalName,
      command: rawCmd,
      autoRunOnPull,
    };

    repo.commands = [...(repo.commands || []), newCmd];
    await saveCurrentSettings();
    return newCmd;
  }

  async function removeCommandFromRepo(repoId: string, cmdId: string) {
    let repo = settings.value.repositories.find((r) => r.id === repoId);
    if (!repo && activeRepo.value) {
      repo = activeRepo.value;
    }
    if (!repo || !repo.commands) return;

    repo.commands = repo.commands.filter((c) => c.id !== cmdId);
    await saveCurrentSettings();
  }

  async function updateCommandInRepo(
    repoId: string,
    cmdId: string,
    name: string,
    command: string,
    autoRunOnPull = false
  ) {
    let repo = settings.value.repositories.find((r) => r.id === repoId);
    if (!repo && activeRepo.value) {
      repo = activeRepo.value;
    }
    if (!repo || !repo.commands) return;

    const target = repo.commands.find((c) => c.id === cmdId);
    if (target) {
      target.name = name.trim() || command.trim();
      target.command = command.trim();
      target.autoRunOnPull = autoRunOnPull;
      await saveCurrentSettings();
    }
  }

  async function updateSettings(newSettings: Partial<AppSettings>) {
    settings.value = { ...settings.value, ...newSettings };
    await saveCurrentSettings();
    await refreshAll();
  }

  async function saveCurrentSettings() {
    if (activeRepo.value) {
      settings.value.repoPath = activeRepo.value.path;
      settings.value.githubRepo = activeGitHubRepo.value;
    }
    await saveSettings(settings.value);
  }

  async function refreshActiveRepoContext() {
    const current = activeRepo.value;
    if (!current || !current.path.trim()) {
      repoContext.value = null;
      gitError.value = null;
      return;
    }

    try {
      gitError.value = null;
      const ctx = await invoke<RepoContext>("get_repo_context", {
        path: current.path.trim(),
      });
      repoContext.value = ctx;

      // Update summary for active repo
      repoSummaries.value[current.id] = {
        id: current.id,
        name: current.name || ctx.repo_name,
        current_branch: ctx.current_branch,
        ahead: ctx.ahead,
        behind: ctx.behind,
        has_upstream: ctx.has_upstream,
        error: null,
      };

      // Auto-detect & save GitHub repo name if not set
      if (!current.githubRepo?.trim() && ctx.detected_github_repo) {
        current.githubRepo = ctx.detected_github_repo;
        await saveCurrentSettings();
      }
    } catch (e: any) {
      const msg = typeof e === "string" ? e : (e?.message || "Failed to inspect local Git repo");
      gitError.value = msg;
      repoSummaries.value[current.id] = {
        id: current.id,
        name: current.name,
        error: msg,
      };
    }
  }

  async function refreshActiveGitHubContext() {
    const current = activeRepo.value;
    const { githubToken, githubUsername } = settings.value;
    const ghRepo = activeGitHubRepo.value;

    if (!githubToken.trim() || !githubUsername.trim() || !ghRepo) {
      githubContext.value = null;
      githubError.value = null;
      return;
    }

    try {
      githubError.value = null;
      const res = await invoke<GitHubContext>("fetch_github_context", {
        token: githubToken.trim(),
        username: githubUsername.trim(),
        repo: ghRepo,
      });
      githubContext.value = res;

      if (current) {
        const openCount = res.pull_requests.filter(
          (p) => p.state.toUpperCase() === "OPEN"
        ).length;
        if (repoSummaries.value[current.id]) {
          repoSummaries.value[current.id].openPrCount = openCount;
        }
      }
    } catch (e: any) {
      githubError.value = typeof e === "string" ? e : (e?.message || "Failed to fetch GitHub PRs");
    }
  }

  // Background summary refresh for other configured repos
  async function refreshBackgroundSummaries() {
    const others = settings.value.repositories.filter(
      (r) => r.id !== settings.value.activeRepoId
    );

    await Promise.allSettled(
      others.map(async (r) => {
        try {
          const ctx = await invoke<RepoContext>("get_repo_context", {
            path: r.path.trim(),
          });
          repoSummaries.value[r.id] = {
            id: r.id,
            name: r.name || ctx.repo_name,
            current_branch: ctx.current_branch,
            ahead: ctx.ahead,
            behind: ctx.behind,
            has_upstream: ctx.has_upstream,
            error: null,
          };
        } catch (e: any) {
          repoSummaries.value[r.id] = {
            id: r.id,
            name: r.name,
            error: typeof e === "string" ? e : "Error",
          };
        }
      })
    );
  }

  async function checkDailyAutoPulls() {
    const today = new Date().toLocaleDateString("en-CA");
    const now = new Date();
    const currentHours = String(now.getHours()).padStart(2, "0");
    const currentMinutes = String(now.getMinutes()).padStart(2, "0");
    const currentTimeStr = `${currentHours}:${currentMinutes}`;

    for (const repo of settings.value.repositories) {
      if (!repo.autoPullDaily) continue;
      if (repo.lastDailyPullDate === today || repo.lastDailyPullAttemptDate === today) continue;

      const targetTime = repo.dailyPullTime || "00:00";
      if (currentTimeStr >= targetTime) {
        console.log(`[GitPulse] Triggering daily auto-pull for repo: ${repo.name}`);
        await pullRepo(repo, true);
      }
    }
  }

  async function refreshAll() {
    isRefreshing.value = true;
    try {
      await Promise.allSettled([
        refreshActiveRepoContext(),
        refreshActiveGitHubContext(),
        refreshBackgroundSummaries(),
      ]);
      lastUpdated.value = new Date();
      refreshSecondsAgo.value = 0;
      await checkDailyAutoPulls();
    } finally {
      isRefreshing.value = false;
    }
  }

  function startPolling() {
    stopPolling();
    timerId = setInterval(() => {
      refreshAll();
    }, 30000);

    secondsTimerId = setInterval(() => {
      if (lastUpdated.value) {
        refreshSecondsAgo.value = Math.floor(
          (Date.now() - lastUpdated.value.getTime()) / 1000
        );
      }
    }, 1000);
  }

  function stopPolling() {
    if (timerId) {
      clearInterval(timerId);
      timerId = null;
    }
    if (secondsTimerId) {
      clearInterval(secondsTimerId);
      secondsTimerId = null;
    }
  }

  async function init() {
    isLoading.value = true;
    try {
      const saved = await loadSettings();
      settings.value = saved;
      if (saved.repositories.length > 0) {
        await refreshAll();
      } else {
        isSettingsOpen.value = true;
      }
      startPolling();
    } finally {
      isLoading.value = false;
    }
  }

  return {
    settings,
    activeRepo,
    activeGitHubRepo,
    isGitHubConfigured,
    repoContext,
    githubContext,
    repoSummaries,
    openPullRequests,
    mergedPullRequests,
    teamOpenPRs,
    teamMergedPRs,
    lastMergedPR,
    lastRepoMergedPR,
    isLoading,
    isRefreshing,
    isSettingsOpen,
    gitError,
    githubError,
    isPulling,
    isPushing,
    isSyncingWithMain,
    isCheckingOut,
    gitActionNotification,
    runningCommandId,
    lastCommandExecution,
    commandError,
    lastUpdated,
    lastUpdatedText,
    refreshSecondsAgo,
    init,
    refreshAll,
    setActiveRepo,
    addRepository,
    removeRepository,
    updateRepository,
    pickRepoFolder,
    pickAndAddRepo,
    inspectRepoPath,
    executeRepoCommand,
    addCommandToRepo,
    removeCommandFromRepo,
    updateCommandInRepo,
    checkoutBranch,
    openInEditor,
    openInTerminal,
    pullCurrentRepo,
    pullRepo,
    syncWithMain,
    fastForwardMain,
    abortRebase,
    continueRebase,
    checkDailyAutoPulls,
    pushCurrentRepo,
    updateSettings,
    saveCurrentSettings,
    getBranchPrStatus,
    startPolling,
    stopPolling,
  };
});
