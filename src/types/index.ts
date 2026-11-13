export interface CommitInfo {
  hash: string;
  subject: string;
  author_name: string;
  relative_time: string;
}

export interface RecentBranch {
  name: string;
  relative_time: string;
}

export interface RepoContext {
  repo_name: string;
  repo_path: string;
  current_branch: string;
  default_branch: string;
  is_default_branch: boolean;
  ahead: number;
  behind: number;
  has_upstream: boolean;
  ahead_of_main: number;
  behind_main: number;
  is_rebasing: boolean;
  is_merging: boolean;
  last_commit: CommitInfo | null;
  recent_branches: RecentBranch[];
  detected_github_repo: string | null;
  uncommitted_changes: number;
  changed_files: string[];
}

export interface PullRequestInfo {
  number: number;
  title: string;
  url: string;
  created_at: string;
  state: "OPEN" | "CLOSED" | "MERGED" | string;
  head_ref_name: string;
  review_decision: "APPROVED" | "CHANGES_REQUESTED" | "REVIEW_REQUIRED" | null;
  ci_status: "SUCCESS" | "FAILURE" | "PENDING" | "ERROR" | "EXPECTED" | null;
  unresolved_threads: number;
  merged_at?: string | null;
  author_login?: string | null;
}

export interface GitHubContext {
  pull_requests: PullRequestInfo[];
  team_open_prs?: PullRequestInfo[];
  team_merged_prs?: PullRequestInfo[];
  last_repo_merged_pr?: PullRequestInfo | null;
}

export interface GitActionResult {
  success: boolean;
  message: string;
}

export interface CustomCommand {
  id: string;
  name: string;
  command: string;
  autoRunOnPull?: boolean;
}

export interface CommandExecutionResult {
  success: boolean;
  exit_code: number | null;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

export interface RepoConfig {
  id: string;
  name: string;
  path: string;
  githubRepo?: string; // e.g. "owner/repo"
  commands?: CustomCommand[];
  autoPullDaily?: boolean; // Enable automatic daily pull
  dailyPullTime?: string; // e.g. "09:00" (24-hour format)
  lastDailyPullDate?: string; // "YYYY-MM-DD" of last successful daily pull
  lastDailyPullAttemptDate?: string; // "YYYY-MM-DD" of last attempted pull if skipped/failed
  autoPullMode?: "all" | "main_only" | "current_only"; // "all" = ff main & pull branch, "main_only" = ff main, "current_only" = pull branch
}

export interface RepoSummary {
  id: string;
  name: string;
  current_branch?: string;
  ahead?: number;
  behind?: number;
  has_upstream?: boolean;
  openPrCount?: number;
  error?: string | null;
}

export interface AppSettings {
  repositories: RepoConfig[];
  activeRepoId: string;
  githubToken: string;
  githubUsername: string;
  defaultTerminal?: string; // "Terminal", "Ghostty", "iTerm", "Warp", "Alacritty", "kitty", "custom"
  customTerminalCmd?: string;
  defaultEditor?: string; // "code", "cursor", "windsurf", "zed", "sublime", "webstorm", "custom"
  customEditorCmd?: string;
  // Fallbacks for backwards compatibility
  repoPath?: string;
  githubRepo?: string;
}
