# GitPulse

A native macOS desktop dashboard for tracking your local Git repositories and GitHub Pull Requests, built with **Tauri 2**, **Vue 3**, **TypeScript**, and **Tailwind CSS**.

## Features

- **Multi-Repo Tabs:** Add multiple local Git repositories and switch between them quickly from the top tab bar, with ahead/behind sync indicators per repo.
- **Live Branch & Commit Context:** Displays your current branch, working tree changes, behind-`main` count, and latest commit details with automatic background polling.
- **One-Click Git Actions:** Pull, push, switch branches, fast-forward `main`, and sync your current branch with `main` (rebase or merge).
- **Rebase Conflict Helper:** Detects paused rebases with merge conflicts and provides quick actions to open your editor, **Continue Rebase**, or **Abort Rebase**.
- **Recent Branches:** Quick-switch between recent local branches, cross-referenced with PR badges and one-click compare/create-PR links.
- **Open PRs Panel:** Lists your open GitHub pull requests with CI check and review status (configured with a GitHub personal access token in Settings).
- **Quick Launchers & Custom Commands:** Open the active repo directly in your preferred editor (VS Code, Cursor, Windsurf, Zed, Sublime, WebStorm, IntelliJ, Xcode, or custom `cmd:`) or terminal (Terminal, Ghostty, iTerm2, Warp, Alacritty, kitty, or custom `cmd:`), and run custom per-repo commands from the actions bar.
- **Daily Auto-Pull (Per Repo):** Optional scheduled daily auto-pull per repository.
- **Built-in Auto-Updater:** Checks GitHub Releases on launch and shows an **Update** button in the header to download, install, and relaunch when a new version is live.

## Development

### Prerequisites
- Node.js & npm
- Rust & Cargo (installed via `rustup`)

### Run locally
```bash
npm install
npm run tauri dev
```

### Build `.dmg` locally
```bash
npm run tauri build
```

## Releasing Updates

Releases are built and signed automatically by [release.yml](.github/workflows/release.yml) when a `v*` tag is pushed:

1. Bump `"version"` in `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`.
2. Commit, tag, and push:
   ```bash
   git tag v0.1.1
   git push origin main --tags
   ```
