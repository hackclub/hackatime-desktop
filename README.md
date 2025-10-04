# Hackatime Desktop

[![Release](https://github.com/hackclub/hackatime-desktop/actions/workflows/release.yaml/badge.svg)](https://github.com/hackclub/hackatime-desktop/actions/workflows/release.yaml)

Desktop app for [Hackatime](https://hackatime.hackclub.com). Built with Tauri, Vue 3, TypeScript, and Rust.

## 🛠️ Tech Stack

- **Frontend**: Vue 3, TypeScript, Tailwind CSS, Chart.js
- **Backend**: Rust (Tauri v2)
- **Build Tools**: Vite, pnpm
- **CI/CD**: GitHub Actions with Release Please

## 📦 Installation

### Build from Source

#### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or higher)
- [pnpm](https://pnpm.io/) (v9.15.0 or higher)
- [Rust](https://rustup.rs/) (latest stable)
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: See [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)
  - **Windows**: See [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

#### Build Steps

```bash
# Clone the repository
git clone https://github.com/hackclub/hackatime-desktop.git
cd hackatime-desktop

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build binaries
pnpm tauri build
```

## 🤝 Contributing

We welcome contributions! Please follow the guidelines below to ensure smooth collaboration.

### Commit Message Convention

This project uses [**Conventional Commits**](https://www.conventionalcommits.org/) for automated versioning and changelog generation via [Release Please](https://github.com/googleapis/release-please).

#### Commit Format

```
<type>: <description>

[optional body]

[optional footer(s)]
```

#### Types

Use these commit types for Release Please to automatically detect changes:

- **`feat:`** - A new feature 
  ```bash
  git commit -m "feat: add Discord RPC integration"
  ```

- **`fix:`** - A bug fix 
  ```bash
  git commit -m "fix: resolve authentication timeout issue"
  ```

- **`chore:`** - Maintenance tasks, CI/CD, dependencies 
  ```bash
  git commit -m "chore: update dependencies"
  git commit -m "chore(ci): update release workflow"
  ```

#### Breaking Changes

For breaking changes that require a major version bump (e.g., 1.0.0 → 2.0.0), add `BREAKING CHANGE:` in the commit body or use `!` after the type:

```bash
git commit -m "feat!: migrate to new API v2" -m "BREAKING CHANGE: requires new authentication flow"
```

### Workflow

1. **Fork the repository**
2. **Create a feature branch**:
   ```bash
   git checkout -b feat/my-new-feature
   # or
   git checkout -b fix/bug-description
   ```

3. **Make your changes** and commit using conventional commits:
   ```bash
   git add .
   git commit -m "feat: add new statistics chart"
   ```

4. **Push to your fork**:
   ```bash
   git push origin feat/my-new-feature
   ```

5. **Create a Pull Request** to the `main` branch

## 🔄 How Release Please Works

This project uses [Release Please](https://github.com/googleapis/release-please) for automated releases. Here's how it works:

### Automated Release Process

1. **Commit with Conventional Commits** - When you merge commits to `main` using the conventional commit format (`feat:`, `fix:`, `chore:`)

2. **Release PR Creation** - Release Please automatically:
   - Analyzes commit messages since the last release
   - Determines the next version number based on semantic versioning:
     - `feat:` → minor version bump (1.0.0 → 1.1.0)
     - `fix:` → patch version bump (1.0.0 → 1.0.1)
     - Breaking changes → major version bump (1.0.0 → 2.0.0)
   - Creates/updates a Release PR with:
     - Updated `CHANGELOG.md`
     - Bumped version in `package.json`
     - Bumped version in `src-tauri/tauri.conf.json`

3. **Release PR Review** - The automatically created PR will show:
   - All changes since the last release
   - New version number
   - Updated changelog
   
4. **Merge to Release** - When the Release PR is merged to `main`:
   - A new GitHub Release is created with the version tag (e.g., `app-v1.2.0`)
   - The release workflow builds binaries for all platforms
   - Binaries are automatically uploaded to the S3 release bucket
   - The auto-updater manifest is updated for in-app updates

## 🐛 Issues & Support

Found a bug or have a feature request? Please open an issue on the [GitHub Issues](https://github.com/hackclub/hackatime-desktop/issues) page.

---

Made with ❤️ by Hack Club
