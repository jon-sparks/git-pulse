use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub subject: String,
    pub author_name: String,
    pub relative_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentBranch {
    pub name: String,
    pub relative_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoContext {
    pub repo_name: String,
    pub repo_path: String,
    pub current_branch: String,
    pub default_branch: String,
    pub is_default_branch: bool,
    pub ahead: u32,
    pub behind: u32,
    pub has_upstream: bool,
    pub ahead_of_main: u32,
    pub behind_main: u32,
    pub is_rebasing: bool,
    pub is_merging: bool,
    pub last_commit: Option<CommitInfo>,
    pub recent_branches: Vec<RecentBranch>,
    pub detected_github_repo: Option<String>,
    pub uncommitted_changes: u32,
    pub changed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub url: String,
    pub created_at: String,
    pub state: String, // "OPEN", "CLOSED", "MERGED"
    pub head_ref_name: String,
    pub review_decision: Option<String>, // "APPROVED", "CHANGES_REQUESTED", "REVIEW_REQUIRED", or None
    pub ci_status: Option<String>,       // "SUCCESS", "FAILURE", "PENDING", "ERROR", etc., or None
    pub unresolved_threads: usize,
    pub merged_at: Option<String>,
    pub author_login: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubContext {
    pub pull_requests: Vec<PullRequestInfo>,
    pub team_open_prs: Vec<PullRequestInfo>,
    pub team_merged_prs: Vec<PullRequestInfo>,
    pub last_repo_merged_pr: Option<PullRequestInfo>,
}

fn expand_path(p: &str) -> PathBuf {
    let trimmed = p.trim();
    if let Some(stripped) = trimmed.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(stripped);
        }
    }
    PathBuf::from(trimmed)
}

fn parse_github_remote(remote_url: &str) -> Option<String> {
    let trimmed = remote_url.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. SSH scp-like syntax: git@github.com:owner/repo.git or git@github-alias:owner/repo.git
    if let Some(colon_idx) = trimmed.find(':') {
        let prefix = &trimmed[..colon_idx];
        // Ensure colon is not protocol colon (e.g. https://)
        if !prefix.contains("://") && prefix.contains("github") {
            let path = &trimmed[colon_idx + 1..];
            let cleaned = path.trim_end_matches(".git").trim_matches('/');
            let parts: Vec<&str> = cleaned.split('/').collect();
            if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                return Some(format!("{}/{}", parts[0], parts[1]));
            }
        }
    }

    // 2. URL syntax: https://github.com/owner/repo[.git] or ssh://git@github.com/owner/repo[.git]
    if let Some(idx) = trimmed.find("github.") {
        if let Some(slash_idx) = trimmed[idx..].find('/') {
            let path = &trimmed[idx + slash_idx + 1..];
            let cleaned = path.trim_end_matches(".git").trim_matches('/');
            let parts: Vec<&str> = cleaned.split('/').collect();
            if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
                return Some(format!("{}/{}", parts[0], parts[1]));
            }
        }
    }

    None
}

fn detect_default_branch(repo_dir: &Path) -> String {
    let origin_head = Command::new("git")
        .arg("symbolic-ref")
        .arg("refs/remotes/origin/HEAD")
        .current_dir(repo_dir)
        .output();

    if let Ok(out) = origin_head {
        if out.status.success() {
            let ref_name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Some(short) = ref_name.strip_prefix("refs/remotes/origin/") {
                return short.to_string();
            }
        }
    }

    let check_main = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg("refs/heads/main")
        .current_dir(repo_dir)
        .output();
    if let Ok(out) = check_main {
        if out.status.success() {
            return "main".to_string();
        }
    }

    let check_master = Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg("refs/heads/master")
        .current_dir(repo_dir)
        .output();
    if let Ok(out) = check_master {
        if out.status.success() {
            return "master".to_string();
        }
    }

    "main".to_string()
}

fn check_rebase_or_merge_status(repo_dir: &Path) -> (bool, bool) {
    let git_dir_out = Command::new("git")
        .arg("rev-parse")
        .arg("--git-path")
        .arg("rebase-merge")
        .current_dir(repo_dir)
        .output();

    let is_rebase_merge = git_dir_out.ok().map(|o| {
        let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
        let path = if Path::new(&p).is_absolute() { PathBuf::from(&p) } else { repo_dir.join(p) };
        path.exists()
    }).unwrap_or(false);

    let git_apply_out = Command::new("git")
        .arg("rev-parse")
        .arg("--git-path")
        .arg("rebase-apply")
        .current_dir(repo_dir)
        .output();
    let is_rebase_apply = git_apply_out.ok().map(|o| {
        let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
        let path = if Path::new(&p).is_absolute() { PathBuf::from(&p) } else { repo_dir.join(p) };
        path.exists()
    }).unwrap_or(false);

    let git_merge_out = Command::new("git")
        .arg("rev-parse")
        .arg("--git-path")
        .arg("MERGE_HEAD")
        .current_dir(repo_dir)
        .output();
    let is_merging = git_merge_out.ok().map(|o| {
        let p = String::from_utf8_lossy(&o.stdout).trim().to_string();
        let path = if Path::new(&p).is_absolute() { PathBuf::from(&p) } else { repo_dir.join(p) };
        path.exists()
    }).unwrap_or(false);

    (is_rebase_merge || is_rebase_apply, is_merging)
}

#[tauri::command]
async fn get_repo_context(path: String) -> Result<RepoContext, String> {
    let repo_dir = expand_path(&path);

    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    // Check if inside a git work tree
    let is_git = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !is_git.status.success() {
        return Err(format!("Not a git repository: {}", repo_dir.display()));
    }

    // Get repo root path and repo name
    let toplevel_out = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| e.to_string())?;

    let root_path_str = if toplevel_out.status.success() {
        String::from_utf8_lossy(&toplevel_out.stdout).trim().to_string()
    } else {
        repo_dir.to_string_lossy().to_string()
    };

    let repo_name = Path::new(&root_path_str)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Repository".to_string());

    // Action 1: Background fetch: `git fetch --quiet` (best effort; ignore failure if offline)
    let _ = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("fetch")
        .arg("--quiet")
        .current_dir(&repo_dir)
        .output();

    // Action 2: Current Branch: `git rev-parse --abbrev-ref HEAD`
    let branch_out = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to get branch: {}", e))?;

    let current_branch = if branch_out.status.success() {
        let b = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();
        if b == "HEAD" {
            // Detached HEAD
            let commit_short = Command::new("git")
                .arg("rev-parse")
                .arg("--short")
                .arg("HEAD")
                .current_dir(&repo_dir)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| "detached".to_string());
            format!("HEAD ({})", commit_short)
        } else {
            b
        }
    } else {
        "HEAD".to_string()
    };

    let default_branch = detect_default_branch(&repo_dir);

    // If on a different branch, quietly fast-forward local default_branch in the background
    if current_branch != default_branch && !current_branch.starts_with("HEAD") {
        let _ = Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .arg("fetch")
            .arg("origin")
            .arg(format!("{}:{}", default_branch, default_branch))
            .arg("--quiet")
            .current_dir(&repo_dir)
            .output();
    }

    // Action 3: Sync Status: `git rev-list --left-right --count HEAD...@{upstream}`
    let sync_out = Command::new("git")
        .arg("rev-list")
        .arg("--left-right")
        .arg("--count")
        .arg("HEAD...@{upstream}")
        .current_dir(&repo_dir)
        .output();

    let (ahead, behind, has_upstream) = match sync_out {
        Ok(output) if output.status.success() => {
            let s = String::from_utf8_lossy(&output.stdout);
            let mut parts = s.split_whitespace();
            let ah: u32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            let bh: u32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
            (ah, bh, true)
        }
        _ => (0, 0, false),
    };

    // Action 4: Last Commit: `git log -1 --pretty=format:"%h|%s|%an|%ar"`
    let commit_out = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--pretty=format:%h|%s|%an|%ar")
        .current_dir(&repo_dir)
        .output();

    let last_commit = match commit_out {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            let raw = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = raw.split('|').collect();
            if parts.len() >= 4 {
                // If subject contained '|', reconstruct properly:
                // first is hash, last two are author_name and relative_time
                let hash = parts[0].to_string();
                let relative_time = parts[parts.len() - 1].to_string();
                let author_name = parts[parts.len() - 2].to_string();
                let subject = parts[1..parts.len() - 2].join("|");
                Some(CommitInfo {
                    hash,
                    subject,
                    author_name,
                    relative_time,
                })
            } else if !parts.is_empty() {
                Some(CommitInfo {
                    hash: parts.get(0).unwrap_or(&"").to_string(),
                    subject: parts.get(1).unwrap_or(&"").to_string(),
                    author_name: parts.get(2).unwrap_or(&"").to_string(),
                    relative_time: parts.get(3).unwrap_or(&"").to_string(),
                })
            } else {
                None
            }
        }
        _ => None,
    };

    // Action 5: Recent Local Branches: `git for-each-ref --sort=-committerdate refs/heads/ --count=5 --format="%(refname:short)|%(committerdate:relative)"`
    let branches_out = Command::new("git")
        .arg("for-each-ref")
        .arg("--sort=-committerdate")
        .arg("refs/heads/")
        .arg("--count=5")
        .arg("--format=%(refname:short)|%(committerdate:relative)")
        .current_dir(&repo_dir)
        .output();

    let mut recent_branches = Vec::new();
    if let Ok(output) = branches_out {
        if output.status.success() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            for line in out_str.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some((bname, rel_time)) = line.split_once('|') {
                    recent_branches.push(RecentBranch {
                        name: bname.trim().to_string(),
                        relative_time: rel_time.trim().to_string(),
                    });
                } else {
                    recent_branches.push(RecentBranch {
                        name: line.to_string(),
                        relative_time: "".to_string(),
                    });
                }
            }
        }
    }

    // Detected GitHub repository from remote.origin.url
    let remote_out = Command::new("git")
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .current_dir(&repo_dir)
        .output();

    let detected_github_repo = remote_out
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            let url_str = String::from_utf8_lossy(&o.stdout);
            parse_github_remote(&url_str)
        });

    // Action 6: Working tree status: `git status --porcelain`
    let status_out = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(&repo_dir)
        .output();

    let mut changed_files = Vec::new();
    if let Ok(output) = status_out {
        if output.status.success() {
            let out_str = String::from_utf8_lossy(&output.stdout);
            for line in out_str.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    changed_files.push(trimmed.to_string());
                }
            }
        }
    }
    let uncommitted_changes = changed_files.len() as u32;

    let is_default_branch = current_branch == default_branch;
    let (ahead_of_main, behind_main) = if is_default_branch {
        (0, 0)
    } else {
        let target_ref = format!("origin/{}", default_branch);
        let rev_list = Command::new("git")
            .arg("rev-list")
            .arg("--left-right")
            .arg("--count")
            .arg(format!("HEAD...{}", target_ref))
            .current_dir(&repo_dir)
            .output();

        match rev_list {
            Ok(output) if output.status.success() => {
                let s = String::from_utf8_lossy(&output.stdout);
                let mut parts = s.split_whitespace();
                let ah: u32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let bh: u32 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                (ah, bh)
            }
            _ => (0, 0),
        }
    };

    let (is_rebasing, is_merging) = check_rebase_or_merge_status(&repo_dir);

    Ok(RepoContext {
        repo_name,
        repo_path: root_path_str,
        current_branch,
        default_branch,
        is_default_branch,
        ahead,
        behind,
        has_upstream,
        ahead_of_main,
        behind_main,
        is_rebasing,
        is_merging,
        last_commit,
        recent_branches,
        detected_github_repo,
        uncommitted_changes,
        changed_files,
    })
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<GraphQLData>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Deserialize)]
struct GraphQLData {
    search: Option<GraphQLSearch>,
    repository: Option<GraphQLRepository>,
}

#[derive(Deserialize)]
struct GraphQLRepository {
    #[serde(rename = "teamOpen")]
    team_open: Option<GraphQLSearch>,
    #[serde(rename = "teamMerged")]
    team_merged: Option<GraphQLSearch>,
}

#[derive(Deserialize)]
struct GraphQLAuthor {
    login: Option<String>,
}

#[derive(Deserialize)]
struct GraphQLSearch {
    nodes: Option<Vec<Option<GraphQLPRNode>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLPRNode {
    number: Option<u64>,
    title: Option<String>,
    url: Option<String>,
    created_at: Option<String>,
    state: Option<String>,
    head_ref_name: Option<String>,
    review_decision: Option<String>,
    merged_at: Option<String>,
    author: Option<GraphQLAuthor>,
    commits: Option<GraphQLCommits>,
    review_threads: Option<GraphQLReviewThreads>,
}

#[derive(Deserialize)]
struct GraphQLCommits {
    nodes: Option<Vec<Option<GraphQLCommitWrapper>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLCommitWrapper {
    commit: Option<GraphQLCommitInner>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLCommitInner {
    status_check_rollup: Option<GraphQLStatusCheckRollup>,
}

#[derive(Deserialize)]
struct GraphQLStatusCheckRollup {
    state: Option<String>,
}

#[derive(Deserialize)]
struct GraphQLReviewThreads {
    nodes: Option<Vec<Option<GraphQLReviewThreadNode>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLReviewThreadNode {
    is_resolved: Option<bool>,
}

fn parse_pr_node(node: GraphQLPRNode) -> PullRequestInfo {
    let number = node.number.unwrap_or(0);
    let title = node.title.unwrap_or_default();
    let url = node.url.unwrap_or_default();
    let created_at = node.created_at.unwrap_or_default();
    let state = node.state.unwrap_or_else(|| "OPEN".to_string());
    let head_ref_name = node.head_ref_name.unwrap_or_default();
    let review_decision = node.review_decision;
    let author_login = node.author.and_then(|a| a.login);
    let merged_at = node.merged_at;

    let ci_status = node
        .commits
        .and_then(|c| c.nodes)
        .and_then(|mut list| list.pop())
        .flatten()
        .and_then(|w| w.commit)
        .and_then(|i| i.status_check_rollup)
        .and_then(|r| r.state);

    let unresolved_threads = node
        .review_threads
        .and_then(|rt| rt.nodes)
        .map(|threads| {
            threads
                .into_iter()
                .flatten()
                .filter(|t| !t.is_resolved.unwrap_or(true))
                .count()
        })
        .unwrap_or(0);

    PullRequestInfo {
        number,
        title,
        url,
        created_at,
        state,
        head_ref_name,
        review_decision,
        ci_status,
        unresolved_threads,
        merged_at,
        author_login,
    }
}

#[tauri::command]
async fn fetch_github_context(
    token: String,
    username: String,
    repo: String,
) -> Result<GitHubContext, String> {
    let clean_token = token.trim();
    if clean_token.is_empty() {
        return Err("GitHub Personal Access Token is required.".to_string());
    }

    let clean_repo = repo.trim();
    if clean_repo.is_empty() {
        return Err("Repository name (owner/repo) is required.".to_string());
    }

    let clean_user = username.trim();
    if clean_user.is_empty() {
        return Err("GitHub username is required.".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("GitPulse-App")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let (owner, repo_name) = match clean_repo.split_once('/') {
        Some((o, r)) => (o.trim(), r.trim()),
        None => ("", ""),
    };
    let has_repo_split = !owner.is_empty() && !repo_name.is_empty();

    let search_query = format!("is:pr author:{} repo:{}", clean_user, clean_repo);

    let query = if has_repo_split {
        r#"
query($searchQuery: String!, $owner: String!, $repoName: String!) {
  search(query: $searchQuery, type: ISSUE, first: 50) {
    nodes {
      ... on PullRequest {
        number
        title
        url
        createdAt
        state
        headRefName
        reviewDecision
        mergedAt
        author {
          login
        }
        commits(last: 1) {
          nodes {
            commit {
              statusCheckRollup {
                state
              }
            }
          }
        }
        reviewThreads(first: 50) {
          nodes {
            isResolved
          }
        }
      }
    }
  }
  repository(owner: $owner, name: $repoName) {
    teamOpen: pullRequests(states: [OPEN], orderBy: { field: UPDATED_AT, direction: DESC }, first: 25) {
      nodes {
        number
        title
        url
        createdAt
        state
        headRefName
        reviewDecision
        author {
          login
        }
        commits(last: 1) {
          nodes {
            commit {
              statusCheckRollup {
                state
              }
            }
          }
        }
        reviewThreads(first: 25) {
          nodes {
            isResolved
          }
        }
      }
    }
    teamMerged: pullRequests(states: [MERGED], orderBy: { field: UPDATED_AT, direction: DESC }, first: 25) {
      nodes {
        number
        title
        url
        createdAt
        state
        headRefName
        mergedAt
        author {
          login
        }
      }
    }
  }
}
"#
    } else {
        r#"
query($searchQuery: String!) {
  search(query: $searchQuery, type: ISSUE, first: 50) {
    nodes {
      ... on PullRequest {
        number
        title
        url
        createdAt
        state
        headRefName
        reviewDecision
        mergedAt
        author {
          login
        }
        commits(last: 1) {
          nodes {
            commit {
              statusCheckRollup {
                state
              }
            }
          }
        }
        reviewThreads(first: 50) {
          nodes {
            isResolved
          }
        }
      }
    }
  }
}
"#
    };

    let request_body = if has_repo_split {
        serde_json::json!({
            "query": query,
            "variables": {
                "searchQuery": search_query,
                "owner": owner,
                "repoName": repo_name,
            }
        })
    } else {
        serde_json::json!({
            "query": query,
            "variables": {
                "searchQuery": search_query,
            }
        })
    };

    let res = client
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", clean_token))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if res.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err("GitHub authentication failed (401 Unauthorized). Please check your Personal Access Token.".to_string());
    }

    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        return Err(format!("GitHub API error ({}): {}", status, body));
    }

    let gql_resp: GraphQLResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {}", e))?;

    if let Some(errors) = gql_resp.errors {
        if !errors.is_empty() {
            let err_msgs: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            return Err(format!("GitHub GraphQL error: {}", err_msgs.join(", ")));
        }
    }

    let mut pull_requests = Vec::new();
    let mut team_open_prs = Vec::new();
    let mut team_merged_prs = Vec::new();

    if let Some(data) = gql_resp.data {
        if let Some(search) = data.search {
            if let Some(nodes) = search.nodes {
                for node_opt in nodes {
                    if let Some(node) = node_opt {
                        pull_requests.push(parse_pr_node(node));
                    }
                }
            }
        }

        if let Some(repo_data) = data.repository {
            if let Some(open_search) = repo_data.team_open {
                if let Some(nodes) = open_search.nodes {
                    for node_opt in nodes {
                        if let Some(node) = node_opt {
                            team_open_prs.push(parse_pr_node(node));
                        }
                    }
                }
            }

            if let Some(merged_search) = repo_data.team_merged {
                if let Some(nodes) = merged_search.nodes {
                    for node_opt in nodes {
                        if let Some(node) = node_opt {
                            team_merged_prs.push(parse_pr_node(node));
                        }
                    }
                }
            }
        }
    }

    let last_repo_merged_pr = team_merged_prs.first().cloned();

    Ok(GitHubContext {
        pull_requests,
        team_open_prs,
        team_merged_prs,
        last_repo_merged_pr,
    })
}

#[tauri::command]
async fn pick_folder() -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        let script = r#"tell application "System Events"
            activate
            set folderPath to POSIX path of (choose folder with prompt "Select Local Git Repository")
            return folderPath
        end tell"#;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("Failed to open folder picker: {}", e))?;

        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                // remove trailing slash if any
                let clean = path.trim_end_matches('/').to_string();
                return Ok(Some(clean));
            }
        }
        return Ok(None);
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
}

#[tauri::command]
async fn run_repo_command(repo_path: String, command: String) -> Result<CommandResult, String> {
    let repo_dir = expand_path(&repo_path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let start = std::time::Instant::now();

    #[cfg(unix)]
    let output = {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
        Command::new(shell)
            .arg("-l")
            .arg("-c")
            .arg(&command)
            .current_dir(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?
    };

    #[cfg(not(unix))]
    let output = {
        Command::new("cmd")
            .args(["/C", &command])
            .current_dir(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to execute command: {}", e))?
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(CommandResult {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout,
        stderr,
        duration_ms,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitActionResult {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
async fn git_pull(path: String, mode: Option<String>) -> Result<GitActionResult, String> {
    let repo_dir = expand_path(&path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let pull_mode = mode.unwrap_or_else(|| "all".to_string());
    let default_branch = detect_default_branch(&repo_dir);

    // Get current branch
    let branch_out = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| e.to_string())?;
    let current_branch = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();

    let mut parts: Vec<String> = Vec::new();

    // 1. If mode is "all" or "main_only", and current branch is NOT default_branch, fast-forward default branch
    if (pull_mode == "all" || pull_mode == "main_only")
        && current_branch != default_branch
        && !current_branch.starts_with("HEAD")
    {
        let ff_out = Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .arg("fetch")
            .arg("origin")
            .arg(format!("{}:{}", default_branch, default_branch))
            .current_dir(&repo_dir)
            .output();

        if let Ok(o) = ff_out {
            if o.status.success() {
                parts.push(format!("Fast-forwarded local {}", default_branch));
            }
        }
    }

    // If mode is "main_only" and we're not on main, we're done
    if pull_mode == "main_only" && current_branch != default_branch {
        let msg = if parts.is_empty() {
            format!("{} is up to date", default_branch)
        } else {
            parts.join(" · ")
        };
        return Ok(GitActionResult {
            success: true,
            message: msg,
        });
    }

    // 2. Pull current branch
    let output = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("pull")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git pull: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        let msg = if !stdout.is_empty() {
            stdout
        } else {
            format!("Pulled {}", current_branch)
        };
        parts.push(msg);
        Ok(GitActionResult {
            success: true,
            message: parts.join(" · "),
        })
    } else {
        let err_msg = if !stderr.is_empty() { stderr } else { stdout };
        Err(format!("Git pull failed: {}", err_msg))
    }
}

#[tauri::command]
async fn fast_forward_main(path: String) -> Result<GitActionResult, String> {
    let repo_dir = expand_path(&path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let default_branch = detect_default_branch(&repo_dir);

    let branch_out = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| e.to_string())?;

    let current = String::from_utf8_lossy(&branch_out.stdout).trim().to_string();

    if current == default_branch {
        let pull_out = Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .arg("pull")
            .arg("--ff-only")
            .current_dir(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to pull {}: {}", default_branch, e))?;

        if pull_out.status.success() {
            let stdout = String::from_utf8_lossy(&pull_out.stdout).trim().to_string();
            let msg = if stdout.is_empty() {
                format!("{} is up to date", default_branch)
            } else {
                stdout
            };
            Ok(GitActionResult {
                success: true,
                message: msg,
            })
        } else {
            let stderr = String::from_utf8_lossy(&pull_out.stderr).trim().to_string();
            Err(format!("Failed to fast-forward {}: {}", default_branch, stderr))
        }
    } else {
        let fetch_out = Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .arg("fetch")
            .arg("origin")
            .arg(format!("{}:{}", default_branch, default_branch))
            .current_dir(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to fetch: {}", e))?;

        if fetch_out.status.success() {
            Ok(GitActionResult {
                success: true,
                message: format!(
                    "Fast-forwarded local {} to match origin/{}",
                    default_branch, default_branch
                ),
            })
        } else {
            let stderr = String::from_utf8_lossy(&fetch_out.stderr).trim().to_string();
            if stderr.contains("non-fast-forward") {
                Err(format!(
                    "Cannot fast-forward {}: local branch has diverged commits",
                    default_branch
                ))
            } else {
                Ok(GitActionResult {
                    success: true,
                    message: format!("{} is already up to date with remote", default_branch),
                })
            }
        }
    }
}

#[tauri::command]
async fn git_sync_with_main(
    path: String,
    strategy: Option<String>,
) -> Result<GitActionResult, String> {
    let repo_dir = expand_path(&path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let strat = strategy.unwrap_or_else(|| "rebase".to_string());
    let default_branch = detect_default_branch(&repo_dir);

    // 1. Safety check: make sure no uncommitted tracked changes
    let status_out = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to inspect status: {}", e))?;

    if status_out.status.success() {
        let raw = String::from_utf8_lossy(&status_out.stdout);
        let tracked_modified = raw.lines().any(|l| !l.starts_with("??"));
        if tracked_modified {
            return Err(
                "Cannot sync with main: you have uncommitted changes. Please commit or stash them first."
                    .to_string(),
            );
        }
    }

    // 2. Fetch latest origin/default_branch
    let _ = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("fetch")
        .arg("origin")
        .arg(&default_branch)
        .current_dir(&repo_dir)
        .output();

    // Also fast-forward local default branch ref if possible
    let _ = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("fetch")
        .arg("origin")
        .arg(format!("{}:{}", default_branch, default_branch))
        .current_dir(&repo_dir)
        .output();

    let target_ref = format!("origin/{}", default_branch);

    if strat == "merge" {
        let merge_out = Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .arg("merge")
            .arg(&target_ref)
            .current_dir(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to run git merge: {}", e))?;

        let stdout = String::from_utf8_lossy(&merge_out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&merge_out.stderr).trim().to_string();

        if merge_out.status.success() {
            let msg = if !stdout.is_empty() {
                stdout
            } else {
                format!("Merged {} successfully", target_ref)
            };
            Ok(GitActionResult {
                success: true,
                message: msg,
            })
        } else {
            let err_msg = if !stderr.is_empty() { stderr } else { stdout };
            Err(format!("Merge conflict: {}", err_msg))
        }
    } else {
        // Default: rebase
        let rebase_out = Command::new("git")
            .env("GIT_TERMINAL_PROMPT", "0")
            .arg("rebase")
            .arg(&target_ref)
            .current_dir(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to run git rebase: {}", e))?;

        let stdout = String::from_utf8_lossy(&rebase_out.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&rebase_out.stderr).trim().to_string();

        if rebase_out.status.success() {
            let msg = if !stdout.is_empty() {
                stdout
            } else {
                format!("Successfully rebased on {}", target_ref)
            };
            Ok(GitActionResult {
                success: true,
                message: msg,
            })
        } else {
            let err_msg = if !stderr.is_empty() { stderr } else { stdout };
            Err(format!(
                "Rebase paused due to merge conflicts: {}. Use 'Abort Rebase' or resolve in your editor.",
                err_msg
            ))
        }
    }
}

#[tauri::command]
async fn git_rebase_abort(path: String) -> Result<GitActionResult, String> {
    let repo_dir = expand_path(&path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let out = Command::new("git")
        .arg("rebase")
        .arg("--abort")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to abort rebase: {}", e))?;

    if out.status.success() {
        Ok(GitActionResult {
            success: true,
            message: "Rebase aborted successfully".to_string(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(format!("Failed to abort rebase: {}", stderr))
    }
}

#[tauri::command]
async fn git_rebase_continue(path: String) -> Result<GitActionResult, String> {
    let repo_dir = expand_path(&path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let out = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_EDITOR", "true")
        .arg("rebase")
        .arg("--continue")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to continue rebase: {}", e))?;

    if out.status.success() {
        Ok(GitActionResult {
            success: true,
            message: "Rebase continued successfully".to_string(),
        })
    } else {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        Err(format!(
            "Cannot continue rebase (unresolved conflicts remain): {}",
            stderr
        ))
    }
}

#[tauri::command]
async fn git_push(path: String) -> Result<GitActionResult, String> {
    let repo_dir = expand_path(&path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let output = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("push")
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git push: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        let msg = if !stdout.is_empty() { stdout } else { "Pushed successfully".to_string() };
        Ok(GitActionResult { success: true, message: msg })
    } else {
        let err_msg = if !stderr.is_empty() { stderr } else { stdout };
        Err(format!("Git push failed: {}", err_msg))
    }
}

#[tauri::command]
async fn git_checkout(path: String, branch: String) -> Result<GitActionResult, String> {
    if branch.is_empty() || branch.starts_with('-') {
        return Err("Invalid branch name".to_string());
    }
    let repo_dir = expand_path(&path);
    if !repo_dir.exists() {
        return Err(format!("Directory does not exist: {}", repo_dir.display()));
    }

    let output = Command::new("git")
        .arg("checkout")
        .arg(&branch)
        .current_dir(&repo_dir)
        .output()
        .map_err(|e| format!("Failed to run git checkout: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        let msg = if !stdout.is_empty() {
            stdout
        } else if !stderr.is_empty() {
            stderr
        } else {
            format!("Switched to branch '{}'", branch)
        };
        Ok(GitActionResult { success: true, message: msg })
    } else {
        let err_msg = if !stderr.is_empty() { stderr } else { stdout };
        Err(format!("Git checkout failed: {}", err_msg))
    }
}

#[tauri::command]
async fn open_in_terminal(path: String, terminal: Option<String>) -> Result<(), String> {
    let repo_dir = expand_path(&path);
    #[cfg(target_os = "macos")]
    {
        let term = terminal.unwrap_or_else(|| "Terminal".to_string());
        let term_trim = term.trim();
        let term_lower = term_trim.to_lowercase();

        if term_lower.starts_with("cmd:") {
            let cmd_str = term_trim.trim_start_matches("cmd:").trim();
            let shell_script = format!("{} \"$1\"", cmd_str);
            Command::new("zsh")
                .arg("-l")
                .arg("-c")
                .arg(&shell_script)
                .arg("gitpulse-terminal")
                .arg(&repo_dir)
                .spawn()
                .map_err(|e| format!("Failed to run custom terminal command: {}", e))?;
            return Ok(());
        }

        match term_lower.as_str() {
            "ghostty" => {
                if let Ok(st) = Command::new("open").arg("-a").arg("Ghostty").arg(&repo_dir).status() {
                    if st.success() {
                        return Ok(());
                    }
                }
                if let Ok(st) = Command::new("open")
                    .arg("-na")
                    .arg("Ghostty")
                    .arg("--args")
                    .arg(format!("--working-directory={}", repo_dir.display()))
                    .status()
                {
                    if st.success() {
                        return Ok(());
                    }
                }
            }
            "iterm" | "iterm2" => {
                if let Ok(st) = Command::new("open").arg("-a").arg("iTerm").arg(&repo_dir).status() {
                    if st.success() {
                        return Ok(());
                    }
                }
                if let Ok(st) = Command::new("open").arg("-a").arg("iTerm2").arg(&repo_dir).status() {
                    if st.success() {
                        return Ok(());
                    }
                }
            }
            "warp" => {
                if let Ok(st) = Command::new("open").arg("-a").arg("Warp").arg(&repo_dir).status() {
                    if st.success() {
                        return Ok(());
                    }
                }
            }
            "alacritty" => {
                if let Ok(st) = Command::new("open").arg("-a").arg("Alacritty").arg(&repo_dir).status() {
                    if st.success() {
                        return Ok(());
                    }
                }
            }
            "kitty" => {
                if let Ok(st) = Command::new("open").arg("-a").arg("kitty").arg(&repo_dir).status() {
                    if st.success() {
                        return Ok(());
                    }
                }
            }
            _ => {
                if let Ok(st) = Command::new("open").arg("-a").arg(term_trim).arg(&repo_dir).status() {
                    if st.success() {
                        return Ok(());
                    }
                }
            }
        }

        // Default fallback to Terminal.app
        Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(&repo_dir)
            .status()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Open terminal is only supported on macOS".to_string())
    }
}

#[tauri::command]
async fn open_in_editor(path: String, editor: Option<String>) -> Result<(), String> {
    let repo_dir = expand_path(&path);
    let ed = editor.unwrap_or_else(|| "code".to_string());
    let ed_trim = ed.trim();
    let ed_lower = ed_trim.to_lowercase();

    if ed_lower.starts_with("cmd:") {
        let cmd_str = ed_trim.trim_start_matches("cmd:").trim();
        let shell_script = format!("{} \"$1\"", cmd_str);
        Command::new("zsh")
            .arg("-l")
            .arg("-c")
            .arg(&shell_script)
            .arg("gitpulse-editor")
            .arg(&repo_dir)
            .spawn()
            .map_err(|e| format!("Failed to run custom editor command: {}", e))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let (cli_cmd, app_name) = match ed_lower.as_str() {
            "code" | "vscode" | "visual studio code" => (Some("code"), Some("Visual Studio Code")),
            "cursor" => (Some("cursor"), Some("Cursor")),
            "windsurf" => (Some("windsurf"), Some("Windsurf")),
            "zed" => (Some("zed"), Some("Zed")),
            "sublime" | "sublime text" | "subl" => (Some("subl"), Some("Sublime Text")),
            "webstorm" => (Some("webstorm"), Some("WebStorm")),
            "intellij" | "idea" => (Some("idea"), Some("IntelliJ IDEA")),
            "xcode" => (None, Some("Xcode")),
            _ => (None, Some(ed_trim)),
        };

        // Try CLI command first if defined
        if let Some(cmd) = cli_cmd {
            if let Ok(status) = Command::new(cmd).arg(&repo_dir).status() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // Try macOS open -a <app_name>
        if let Some(app) = app_name {
            if let Ok(status) = Command::new("open").arg("-a").arg(app).arg(&repo_dir).status() {
                if status.success() {
                    return Ok(());
                }
            }
        }

        // Fallbacks in priority order
        if let Ok(status) = Command::new("open").arg("-a").arg("Visual Studio Code").arg(&repo_dir).status() {
            if status.success() {
                return Ok(());
            }
        }
        if let Ok(status) = Command::new("open").arg("-a").arg("Cursor").arg(&repo_dir).status() {
            if status.success() {
                return Ok(());
            }
        }

        // Fallback: open directory in Finder
        Command::new("open")
            .arg(&repo_dir)
            .output()
            .map_err(|e| format!("Failed to open directory: {}", e))?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(status) = Command::new(&ed).arg(&repo_dir).status() {
            if status.success() {
                return Ok(());
            }
        }
        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            get_repo_context,
            fetch_github_context,
            pick_folder,
            run_repo_command,
            git_pull,
            git_push,
            git_checkout,
            open_in_terminal,
            open_in_editor,
            fast_forward_main,
            git_sync_with_main,
            git_rebase_abort,
            git_rebase_continue
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_remote_ssh() {
        assert_eq!(
            parse_github_remote("git@github.com:owner/repo.git"),
            Some("owner/repo".to_string())
        );
        assert_eq!(
            parse_github_remote("git@github.com:octocat/Hello-World.git"),
            Some("octocat/Hello-World".to_string())
        );
        assert_eq!(
            parse_github_remote("git@github-work:acme/example-app.git"),
            Some("acme/example-app".to_string())
        );
        assert_eq!(
            parse_github_remote("ssh://git@github.com/owner/repo.git"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_parse_github_remote_https() {
        assert_eq!(
            parse_github_remote("https://github.com/owner/repo.git"),
            Some("owner/repo".to_string())
        );
        assert_eq!(
            parse_github_remote("https://github.com/owner/repo"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_expand_path() {
        let p = expand_path("~/some/path");
        assert!(!p.to_string_lossy().starts_with("~"));
    }
}
