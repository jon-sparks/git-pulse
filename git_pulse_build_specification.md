# GitPulse: Native Desktop PR & Git Dashboard

## 1. Project Overview

GitPulse is a lean, native desktop application designed to give developers immediate, auto-refreshing visibility into a specific local Git repository and its associated GitHub Pull Requests. It merges local filesystem Git states with remote GitHub API data.

## 2. Tech Stack

* **Framework:** Tauri v2 (Lightweight, low memory footprint)

* **Backend:** Rust (Handles filesystem scanning, spawning Git subprocesses, and GitHub API requests)

* **Frontend:** Vue 3, Vite, TypeScript, Tailwind CSS

* **State Management:** Pinia (Vue)

* **API Communication:** GitHub GraphQL API via `reqwest` in Rust.

## 3. Backend Specifications (Rust / src-tauri)

The Rust backend will expose functions to the Vue frontend via Tauri's IPC `invoke` system.

### Command 1: `get_repo_context(path: String)`

Executes native `git` commands in the specified directory to return the current local state.

* **Action 1:** Background fetch: `git fetch --quiet`

* **Action 2:** Current Branch: `git rev-parse --abbrev-ref HEAD`

* **Action 3:** Sync Status: `git rev-list --left-right --count HEAD...@{upstream}`

* **Action 4:** Last Commit: `git log -1 --pretty=format:"%h|%s|%an|%ar"`

  * *Output parsing:* Hash, Subject, Author Name, Relative Time.

* **Action 5:** Recent Local Branches: `git for-each-ref --sort=-committerdate refs/heads/ --count=5 --format="%(refname:short)|%(committerdate:relative)"`

### Command 2: `fetch_github_context(token: String, username: String, repo: String)`

Calls the GitHub GraphQL API to fetch PRs authored by the user for the specific repository, plus data to cross-reference recent local branches.

**GraphQL Query Requirements:**

* Search constraint: `is:pr author:{username} repo:{repo}`

* Nodes to extract:

  * `title`, `url`, `number`, `createdAt`, `state` (Open/Closed/Merged)

  * `headRefName` (To cross-reference with recent local branches)

  * `reviewDecision` (APPROVED, CHANGES_REQUESTED, REVIEW_REQUIRED)

  * `commits(last: 1)` -> `statusCheckRollup` (CI/CD pass/fail state)

  * `reviewThreads(first: 50)` -> filter for `isResolved: false` to get the unresolved comment count.

## 4. Frontend Specifications (Vue 3)

The UI is a dark-mode, single-repository focused dashboard divided into logical panels.

### A. Header Panel (Current State)

* **Repo Name & Branch:** E.g., `guide-seo` — `feat/cron-audits`.

* **Sync Badge:** Dynamically styled based on ahead/behind count (e.g., Green "Up to date", Red "Behind 2 (Needs Pull)").

* **Auto-refresh indicator:** "Last updated: Just now" (Polls every 30-60 seconds).

### B. Last Commit Hero Widget

* Displays the parsed output from Action 4.

* Shows the commit message prominently, with the author, relative time, and short hash below it.

### C. Context Switch Panel (Recent Branches)

* Displays the 5 most recently touched local branches (from Action 5).

* **Cross-referencing Logic:** The frontend will check if the branch name matches the `headRefName` of any PRs returned from the GraphQL API.

* **Status Indicators:**

  * If no PR: "Local only" (Gray)

  * If PR matches and is Open: "Active PR #12" (Blue)

  * If PR matches and is Merged: "Merged PR #10" (Purple)

  * If PR matches and is Closed: "Closed PR #9" (Red)

### D. Active Work Panel (My Open PRs)

* Lists open Pull Requests authored by the user in this repo.

* **Metrics per PR card:**

  * Age (calculated from `createdAt`).

  * Review Status Badge (Red for Changes Requested, Green for Approved).

  * CI/CD Status (Pass/Fail icon).

  * Unresolved Comments (e.g., "⚠️ 2 unresolved comments").

## 5. Security & Authentication

* **Local Git:** Uses the system's existing Git auth context.

* **GitHub API:** Requires a Personal Access Token (PAT).

  * *Note for User:* If checking work repos, the PAT must have SAML SSO authorized for the organization, or if it is a Fine-Grained PAT, it must be granted access to the specific organization repos.

* **Storage:** Store the PAT locally using `@tauri-apps/plugin-store` for persistence across app restarts.

## 6. Implementation Steps for AI Agent

1. **Scaffold:** Run `npm create tauri-app@latest` (Choose Vue, TS, Vite). Install Tailwind CSS.

2. **Rust Setup:** Add `reqwest`, `serde`, `serde_json`, and `tokio` to `src-tauri/Cargo.toml`.

3. **Backend Logic:** Implement `get_repo_context` and `fetch_github_context` in `lib.rs` / `main.rs`. Expose them via `tauri::generate_handler!`.

4. **Frontend State:** Set up Pinia store to manage the settings (Repo Path, GitHub Token, Username).

5. **UI Development:** Build the main dashboard using Vue components mapped to the layout specs in Section 4.

6. **Polling Mechanism:** Implement `setInterval` in the Vue `onMounted` lifecycle to ping the Tauri backend commands every 30 seconds.
