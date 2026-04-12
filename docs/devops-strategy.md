# Work Tracker 2 — DevOps Strategy

**Author:** Lando (DevOps Expert)  
**Date:** 2026-04-12  
**Status:** DESIGN APPROVED — IMPLEMENTATION PENDING

---

## Overview

### Pipeline Philosophy

**Fast Feedback, Fail Early, Ship Confidently**

The CI/CD pipeline for Work Tracker 2 is designed with three core principles:

1. **Fast Feedback** — Developers get build/test results in <5 minutes for common workflows
2. **Fail Early** — Lint and test before expensive build steps; catch issues cheaply
3. **Ship Confidently** — Every release is tested across all platforms with consistent artifacts

**Key Constraints**:
- Desktop app with native bundles (Windows .msi/.exe, macOS .dmg/.app, Linux .AppImage/.deb)
- Dual-stack build (Rust + Node.js) with Tauri orchestration
- Small team (solo dev + AI) — optimize for simplicity over sophistication
- Open-source tooling only — no paid CI services or proprietary dependencies

---

## Current State Assessment

**Stack**:
- **Frontend**: Svelte 5 + TypeScript + Vite 6
- **Backend**: Rust (edition 2021) + Tauri 2
- **Database**: SQLite (bundled via rusqlite)
- **Test frameworks**: Vitest (frontend), Cargo test (Rust integration tests)

**Existing Workflows**: 
- `.github/workflows/squad-*.yml` — Squad automation (heartbeat, triage, labels)
- **NO** build/test/release workflows yet

**Test Coverage Baseline**:
- Rust: 16 integration tests (services/crud + services/session)
- Frontend: 2 Vitest tests (component smoke tests)
- **Current coverage**: ~10% (estimated)

**Build Commands**:
- `npm run build` — Frontend (Vite → `build/`)
- `npm run tauri:build` — Full Tauri bundle (frontend + backend + packaging)
- `cargo build --release` — Rust backend only

**Version Strategy**:
- Current: `0.1.0` (both package.json and Cargo.toml)
- Single source of truth: `package.json` (Tauri reads from here)

---

## Workflow Inventory

### 1. `ci.yml` — Continuous Integration (Primary Feedback Loop)

**Trigger**: 
- Push to `main` branch
- Pull requests targeting `main`

**Purpose**: Fast feedback on code quality and correctness

**Jobs**:

#### Job 1: `lint`
**Runner**: `ubuntu-latest`  
**Steps**:
1. Checkout code
2. Setup Node.js 22.x
3. Setup Rust stable toolchain
4. Cache npm dependencies (key: `package-lock.json` hash)
5. Cache Cargo registry (key: `Cargo.lock` hash)
6. `npm ci`
7. `npm run check` (TypeScript type checking via Svelte compiler)
8. `cargo fmt -- --check` (Rust formatting)
9. `cargo clippy -- -D warnings` (Rust linting)

**Expected runtime**: 90-120 seconds  
**Caching benefit**: ~60% reduction (180s → 90s)

#### Job 2: `test-frontend`
**Runner**: `ubuntu-latest`  
**Steps**:
1. Checkout code
2. Setup Node.js 22.x
3. Cache npm dependencies
4. `npm ci`
5. `npm test` (Vitest run mode)

**Expected runtime**: 30-45 seconds  
**Exit on**: Test failure (no retry)

#### Job 3: `test-backend`
**Runner**: `ubuntu-latest`  
**Steps**:
1. Checkout code
2. Setup Rust stable toolchain
3. Cache Cargo registry + build artifacts
4. Install system deps: `libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
5. `cd src-tauri && cargo test --all-features`

**Expected runtime**: 120-180 seconds (includes build)  
**Caching benefit**: ~50% reduction on cached runs

#### Job 4: `build-check`
**Runner**: `ubuntu-latest`  
**Dependencies**: `[lint, test-frontend, test-backend]` (runs only if all pass)  
**Steps**:
1. Checkout code
2. Setup Node.js 22.x
3. Setup Rust stable toolchain
4. Cache dependencies
5. Install system deps (same as test-backend)
6. `npm ci`
7. `npm run tauri build -- --debug` (fast debug build, skip code signing)

**Expected runtime**: 180-240 seconds  
**Purpose**: Verify full build pipeline without producing release artifacts

**Total CI Runtime** (parallel): ~3-4 minutes (lint + tests in parallel, then build-check)

---

### 2. `coverage.yml` — Code Coverage Reporting

**Trigger**:
- Pull requests targeting `main`
- Weekly schedule (Mondays 9 AM UTC)

**Purpose**: Track test coverage and prevent regressions

**Jobs**:

#### Job 1: `coverage-rust`
**Runner**: `ubuntu-latest`  
**Steps**:
1. Checkout code
2. Setup Rust stable toolchain
3. Cache Cargo registry
4. Install system deps (same as CI)
5. Install `cargo-tarpaulin`: `cargo install cargo-tarpaulin`
6. Run coverage: `cd src-tauri && cargo tarpaulin --out Xml --output-dir ../coverage`
7. Upload `coverage/cobertura.xml` as artifact

**Expected runtime**: 180-240 seconds

#### Job 2: `coverage-frontend`
**Runner**: `ubuntu-latest`  
**Steps**:
1. Checkout code
2. Setup Node.js 22.x
3. Cache npm dependencies
4. `npm ci`
5. `npm test -- --coverage --reporter=json --reporter=html`
6. Upload `coverage/` as artifact

**Expected runtime**: 45-60 seconds

#### Job 3: `coverage-report`
**Dependencies**: `[coverage-rust, coverage-frontend]`  
**Steps**:
1. Download all coverage artifacts
2. Combine reports (optional: use `codecov` or manual merge)
3. Post PR comment with coverage summary:
   - Rust coverage: X% (target: 40% by Phase 2, 70% long-term)
   - Frontend coverage: Y% (target: 40% by Phase 2, 70% long-term)
   - Overall: Z%
   - Delta from main: +/- N%

**Expected runtime**: 20-30 seconds

**Coverage Thresholds**:
- **Phase 1 (MVP)**: No blocking threshold (informational only)
- **Phase 2**: Enforce 40% minimum on new code
- **Long-term**: 70% overall target

**Reporting Format**: 
- PR comment with markdown table
- Optional: Upload to GitHub Pages for historical tracking

---

### 3. `release.yml` — Release Pipeline

**Trigger**:
- Tag push matching `v*` (e.g., `v0.1.0`, `v1.2.3-beta.1`)

**Purpose**: Build production artifacts for all platforms and publish GitHub Release

**Jobs**:

#### Job 1: `build-matrix`
**Strategy**: Matrix across platforms  
**Platforms**:
- `windows-latest` (Windows 10+)
- `macos-latest` (macOS 11+, universal binary for Intel + Apple Silicon)
- `ubuntu-22.04` (Linux with AppImage + .deb)

**Steps** (per platform):
1. Checkout code
2. Setup Node.js 22.x
3. Setup Rust stable toolchain
4. Cache dependencies
5. Platform-specific system deps:
   - **Windows**: None (MSVC bundled with runner)
   - **macOS**: None (Xcode bundled with runner)
   - **Linux**: `libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf`
6. `npm ci`
7. `npm run tauri build`
8. Collect artifacts from `src-tauri/target/release/bundle/`:
   - **Windows**: `*.msi`, `*.exe` (NSIS installer)
   - **macOS**: `*.dmg`, `*.app` (application bundle)
   - **Linux**: `*.AppImage`, `*.deb`
9. Upload artifacts with naming convention: `work-tracker-2-{version}-{platform}-{arch}.{ext}`

**Expected runtime per platform**: 8-12 minutes

**Artifact Naming**:
```
work-tracker-2-0.1.0-windows-x64.msi
work-tracker-2-0.1.0-windows-x64.exe
work-tracker-2-0.1.0-macos-universal.dmg
work-tracker-2-0.1.0-linux-x64.AppImage
work-tracker-2-0.1.0-linux-x64.deb
```

#### Job 2: `create-release`
**Dependencies**: `build-matrix`  
**Steps**:
1. Download all artifacts from matrix builds
2. Extract version from tag (strip `v` prefix)
3. Generate changelog:
   - Parse git log since last tag
   - Group by conventional commit type (feat, fix, chore, etc.)
   - Format as markdown
4. Create GitHub Release:
   - Tag: `vX.Y.Z`
   - Title: `Work Tracker 2 v{version}`
   - Body: Generated changelog
   - Attach all platform artifacts
   - Mark as pre-release if tag contains `-alpha`, `-beta`, `-rc`

**Expected runtime**: 60-90 seconds

**Total Release Runtime**: ~10-15 minutes (matrix builds in parallel)

---

### 4. `audit.yml` — Security & Dependency Audit

**Trigger**:
- Pull requests targeting `main`
- Weekly schedule (Sundays 9 AM UTC)
- Manual workflow dispatch

**Purpose**: Catch known vulnerabilities in dependencies

**Jobs**:

#### Job 1: `audit-rust`
**Runner**: `ubuntu-latest`  
**Steps**:
1. Checkout code
2. Setup Rust stable toolchain
3. Install `cargo-audit`: `cargo install cargo-audit`
4. Run audit: `cd src-tauri && cargo audit`
5. Fail on: High/Critical vulnerabilities
6. Warn on: Medium vulnerabilities (allow PR to pass but post comment)

**Expected runtime**: 45-60 seconds

#### Job 2: `audit-npm`
**Runner**: `ubuntu-latest`  
**Steps**:
1. Checkout code
2. Setup Node.js 22.x
3. `npm audit --audit-level=high`
4. Fail on: High/Critical vulnerabilities
5. Warn on: Moderate vulnerabilities

**Expected runtime**: 30-45 seconds

**Failure Behavior**:
- **On PR**: Block merge if High/Critical vulnerabilities found
- **On schedule**: Create issue with label `security` and assign to maintainer
- **Exceptions**: Allow ignoring specific advisories via `.cargo/audit.toml` and `npm audit --ignore`

---

## Build Matrix

### Platform Targets

| Platform | Architectures | Bundle Formats | Minimum OS Version |
|----------|---------------|----------------|-------------------|
| **Windows** | x64 | .msi (Windows Installer), .exe (NSIS) | Windows 10 (1903+) |
| **macOS** | Universal (x64 + arm64) | .dmg (disk image), .app (bundle) | macOS 11 (Big Sur+) |
| **Linux** | x64 | .AppImage (portable), .deb (Debian/Ubuntu) | Ubuntu 22.04+ |

**Why these targets?**:
- **Windows x64**: 95%+ of Windows users (x86 discontinued)
- **macOS Universal**: Single binary for Intel + Apple Silicon (required for modern macOS)
- **Linux x64**: Covers Debian/Ubuntu family (most popular for desktop Linux)

**Future expansion** (Phase 3+):
- Windows ARM64 (for Surface Pro X, etc.)
- Linux ARM64 (for Raspberry Pi, etc.)
- Flatpak / Snap packages

---

### Toolchain Version Strategy

#### Rust Toolchain
**Version**: `stable` (latest)  
**Pinning strategy**: No explicit pin in CI (always use latest stable)  
**Rationale**: 
- Rust has strong backwards compatibility guarantees
- Latest stable includes security fixes and optimizations
- Cargo.lock pins exact dependency versions (consistency across builds)

**Fallback**: If breaking change detected, pin to specific version in `rust-toolchain.toml`:
```toml
[toolchain]
channel = "1.75.0"
```

#### Node.js
**Version**: `22.x` (LTS)  
**Pinning strategy**: Use `setup-node@v4` with `node-version: 22`  
**Rationale**:
- Node 22 is the active LTS as of project start (2026)
- `package-lock.json` pins exact dependency versions
- Minor version updates (22.x → 22.y) are safe

**Fallback**: If Node 22 breaks compatibility, pin to specific version:
```yaml
node-version: '22.11.0'
```

#### Tauri CLI
**Version**: `^2.0.0` (from package.json)  
**Pinning strategy**: Let npm resolve within semver range  
**Rationale**:
- Tauri 2.x is stable (released 2024-12)
- Patch updates (2.0.x) are safe
- Major version bumps (3.x) would be explicit migration

**Lock file**: `package-lock.json` ensures exact version consistency across CI runs

---

### System Dependencies

**Ubuntu** (for Linux builds and test runners):
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

**Windows**: No extra deps (MSVC bundled with runner)

**macOS**: No extra deps (Xcode bundled with runner)

---

## Caching Strategy

### Goals
1. Reduce CI runtime by 50-60%
2. Minimize network I/O (registry downloads)
3. Maintain cache freshness (invalidate on dependency changes)

### Cache Layers

#### 1. Cargo Registry Cache
**Path**: `~/.cargo/registry`  
**Key**: `${{ runner.os }}-cargo-registry-${{ hashFiles('src-tauri/Cargo.lock') }}`  
**Restore keys**: 
  - `${{ runner.os }}-cargo-registry-`

**Invalidation**: When `Cargo.lock` changes (new/updated dependencies)

**Benefit**: Avoids re-downloading crates from crates.io (~60-90 seconds saved)

#### 2. Cargo Build Cache
**Path**: `src-tauri/target`  
**Key**: `${{ runner.os }}-cargo-build-${{ hashFiles('src-tauri/Cargo.lock') }}-${{ hashFiles('src-tauri/src/**/*.rs') }}`  
**Restore keys**:
  - `${{ runner.os }}-cargo-build-${{ hashFiles('src-tauri/Cargo.lock') }}-`
  - `${{ runner.os }}-cargo-build-`

**Invalidation**: When Rust source files change

**Benefit**: Incremental compilation (~90-120 seconds saved on cache hit)

**Note**: Cache size can grow large (500MB-1GB). Consider using `cargo-cache` to prune old artifacts.

#### 3. npm Dependencies Cache
**Path**: `~/.npm`  
**Key**: `${{ runner.os }}-npm-${{ hashFiles('package-lock.json') }}`  
**Restore keys**:
  - `${{ runner.os }}-npm-`

**Invalidation**: When `package-lock.json` changes

**Benefit**: Avoids re-downloading packages from npmjs.org (~30-45 seconds saved)

**Action**: Use `actions/setup-node@v4` with `cache: 'npm'` (handles caching automatically)

#### 4. Rust Toolchain Cache
**Built-in**: `actions-rust-lang/setup-rust-toolchain@v1` caches toolchain automatically  
**No explicit config needed**

---

### Cache Warming Strategy

**Cold start** (no cache):
- CI runtime: ~8-10 minutes (full build)

**Warm cache** (dependencies unchanged):
- CI runtime: ~3-4 minutes (incremental build)

**Cache hit rate target**: 70%+ (most PRs don't change dependencies)

---

## Version & Release Strategy

### Semantic Versioning (SemVer)

**Format**: `MAJOR.MINOR.PATCH[-PRERELEASE]`

**Increment rules**:
- **MAJOR** (1.0.0): Breaking changes (incompatible database schema, API changes)
- **MINOR** (0.1.0): New features (backward-compatible)
- **PATCH** (0.0.1): Bug fixes (backward-compatible)
- **PRERELEASE** (0.1.0-beta.1): Alpha/Beta releases

**Current version**: `0.1.0` (Phase 1 MVP)

**Roadmap**:
- `0.1.0` — Phase 1 MVP (basic time tracking)
- `0.2.0` — Phase 2 (multi-customer workflows, favorites)
- `0.3.0` — Phase 3 (reporting & analytics)
- `1.0.0` — Stable release (after Phase 3 + production validation)

---

### Version Bumping Workflow

**Manual approach** (Phase 1):
1. Developer updates `package.json` version
2. Commit: `chore: bump version to 0.1.0`
3. Tag: `git tag v0.1.0`
4. Push: `git push origin main --tags`
5. GitHub Actions builds and releases

**Automated approach** (Phase 2+):
- Use `release-please` GitHub Action (Google's release automation)
- Parses conventional commits to determine version bump
- Creates release PR with changelog
- Merging PR triggers tag + release

**Conventional Commits** (required for automation):
```
feat: add quick-add overlay (Cmd+N)       → MINOR bump
fix: prevent timer drift on sleep/wake    → PATCH bump
feat!: migrate to new database schema     → MAJOR bump
chore: update dependencies                → no bump
```

---

### Changelog Generation

**Phase 1 (manual)**:
- Developer writes `CHANGELOG.md` entry
- Include in release commit

**Phase 2+ (automated)**:
- Parse git log since last tag
- Group by type (feat, fix, chore, docs, etc.)
- Generate markdown list
- Inject into GitHub Release body

**Format**:
```markdown
## v0.1.0 (2026-04-15)

### Features
- ✨ Add quick-add overlay (Cmd+N) (#42)
- ✨ Implement taskbar quick-switch (#43)

### Bug Fixes
- 🐛 Fix timer drift on sleep/wake (#44)
- 🐛 Prevent duplicate sessions on race condition (#45)

### Chores
- 🔧 Update Tauri to 2.0.5 (#46)
```

---

### Release Artifact Hosting

**Primary**: GitHub Releases (attached binaries)  
**Benefits**:
- Free for open-source projects
- Integrated with git tags
- Download analytics
- Persistent URLs

**Artifact retention**: Indefinite (GitHub doesn't expire release assets)

**Auto-update** (Phase 3+):
- Use Tauri's built-in updater plugin
- Point to GitHub Releases API
- Users get in-app update notifications

---

### Pre-release Strategy

**Alpha releases** (`v0.1.0-alpha.1`):
- Internal testing only
- Breaking changes allowed
- No stability guarantees

**Beta releases** (`v0.1.0-beta.1`):
- Public testing (opt-in via GitHub Releases)
- Feature-complete but may have bugs
- No breaking changes within beta series

**Release candidates** (`v0.1.0-rc.1`):
- Final testing before stable release
- Only critical bug fixes allowed
- Promoted to stable if no issues found

**GitHub Release flags**:
- Alphas/Betas: Mark as "Pre-release" (orange badge)
- RCs: Mark as "Pre-release" until promoted
- Stable: Default release (green badge)

---

## Coverage Targets

### Current Baseline
- **Rust**: 16 integration tests (services layer)
- **Frontend**: 2 Vitest tests (smoke tests)
- **Estimated coverage**: ~10%

### Coverage Goals

| Phase | Rust Coverage | Frontend Coverage | Overall Target |
|-------|---------------|-------------------|----------------|
| **Phase 1** (current) | 10-20% | 5-10% | ~10% |
| **Phase 2** (multi-customer) | 40-50% | 30-40% | **40%** |
| **Phase 3** (reporting) | 60-70% | 50-60% | 60% |
| **Long-term** (stable) | **70-80%** | **70-80%** | **70%+** |

### What Gets Measured

**Rust**:
- ✅ Service layer (`src-tauri/src/services/*.rs`) — Primary focus
- ✅ Database layer (`src-tauri/src/db/*.rs`) — Critical for data integrity
- ✅ Command handlers (`src-tauri/src/commands/*.rs`) — Integration tests
- ❌ Models/DTOs (`src-tauri/src/models/*.rs`) — Excluded (no logic)

**Frontend**:
- ✅ Component logic (user interactions, state management)
- ✅ API client wrappers (`src/lib/api/*.ts`)
- ✅ Store modules (`src/lib/stores/*.svelte.ts`)
- ❌ Pure presentational components with no logic — Low priority

### Coverage Tools

**Rust**: `cargo-tarpaulin`
- Output format: Cobertura XML (for reporting), HTML (for local review)
- Command: `cargo tarpaulin --out Xml --out Html --output-dir coverage`
- Run time: ~2-3 minutes

**Frontend**: Vitest built-in coverage (via `@vitest/coverage-v8`)
- Already configured in `vitest.config.ts`
- Command: `npm test -- --coverage`
- Output: HTML + JSON

### Reporting Strategy

**Phase 1** (informational only):
- Post coverage report as PR comment
- Show delta from `main` branch
- No blocking thresholds (too early)

**Phase 2** (enforcement):
- Block PR if new code drops below 40% coverage
- Existing code grandfathered (incremental improvement)
- Configure in `ci.yml`:
  ```yaml
  - name: Check coverage threshold
    run: |
      coverage=$(cat coverage.json | jq '.total.lines.pct')
      if (( $(echo "$coverage < 40" | bc -l) )); then
        echo "❌ Coverage $coverage% is below 40% threshold"
        exit 1
      fi
  ```

**Phase 3** (tracking):
- Upload coverage history to GitHub Pages
- Track trends over time (coverage increasing?)
- Celebrate when hitting 70% overall target 🎉

---

## Dependency Management

### Dependabot Configuration

**Location**: `.github/dependabot.yml`

**Ecosystems**:
1. **Cargo** (Rust dependencies)
2. **npm** (JavaScript dependencies)
3. **GitHub Actions** (workflow actions)

**Update schedule**:
- **Cargo**: Weekly (Mondays)
- **npm**: Weekly (Mondays)
- **GitHub Actions**: Monthly (1st of month)

**Configuration**:
```yaml
version: 2
updates:
  # Rust dependencies
  - package-ecosystem: "cargo"
    directory: "/src-tauri"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 5
    labels:
      - "dependencies"
      - "rust"
    reviewers:
      - "frewikes"

  # npm dependencies
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 5
    labels:
      - "dependencies"
      - "javascript"
    reviewers:
      - "frewikes"

  # GitHub Actions
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "monthly"
    labels:
      - "dependencies"
      - "ci"
```

**PR limits**: Max 5 open PRs per ecosystem (prevents spam)

---

### Auto-Merge Policy

**Patch updates** (e.g., `1.2.3` → `1.2.4`):
- ✅ **Auto-merge** if:
  - CI passes (all tests green)
  - No security vulnerabilities introduced
  - Dependabot confidence rating: High

**Minor updates** (e.g., `1.2.0` → `1.3.0`):
- ⚠️ **Manual review required**
- Check changelog for breaking changes
- Run local tests before merging

**Major updates** (e.g., `1.0.0` → `2.0.0`):
- 🚨 **Always manual review**
- Significant changes expected
- May require code changes
- Test thoroughly before merging

**Implementation**:
- Use GitHub's auto-merge feature (enable for Dependabot PRs with specific labels)
- Alternatively: `dependabot-auto-merge` GitHub Action

---

### Security Audit Schedule

**Automated audits**: Weekly (Sundays via `audit.yml`)

**Manual review**: Quarterly (every 3 months)
- Review all dependencies for obsolescence
- Check for better alternatives
- Remove unused dependencies

**Security advisory response**:
1. Dependabot creates PR to fix vulnerability
2. High/Critical: Merge within 24 hours
3. Medium: Merge within 1 week
4. Low: Merge in next dependency update cycle

---

## Implementation Order

### Phase 1: Fast Feedback CI (Week 1)
**Goal**: Catch bugs before they reach `main`

**Tasks**:
1. ✅ Create `.github/workflows/ci.yml`
   - Lint job (Rust + TypeScript)
   - Test job (Cargo + Vitest)
   - Build check job (debug mode)
2. ✅ Configure caching (Cargo + npm)
3. ✅ Test on sample PR (verify all jobs pass)
4. ✅ Document in `README.md` (add CI badge)

**Success criteria**: CI runs on PRs, completes in <5 minutes

---

### Phase 2: Security & Auditing (Week 1)
**Goal**: Catch vulnerabilities early

**Tasks**:
1. ✅ Create `.github/workflows/audit.yml`
   - Cargo audit job
   - npm audit job
2. ✅ Create `.github/dependabot.yml`
   - Configure Cargo + npm + GitHub Actions
3. ✅ Run initial audit (fix any existing issues)
4. ✅ Test on schedule trigger (verify weekly runs)

**Success criteria**: Audits run weekly, Dependabot creates PRs

---

### Phase 3: Coverage Reporting (Week 2)
**Goal**: Track test coverage trends

**Tasks**:
1. ✅ Create `.github/workflows/coverage.yml`
   - Rust coverage (cargo-tarpaulin)
   - Frontend coverage (Vitest)
2. ✅ Configure PR comment bot (post coverage delta)
3. ✅ Test on sample PR (verify report appears)
4. ✅ Set up GitHub Pages for coverage history (optional)

**Success criteria**: Coverage reports on PRs, baseline established

---

### Phase 4: Release Pipeline (Week 3)
**Goal**: Automated multi-platform releases

**Tasks**:
1. ✅ Create `.github/workflows/release.yml`
   - Build matrix (Windows, macOS, Linux)
   - Artifact naming convention
   - GitHub Release creation
2. ✅ Test with pre-release tag (`v0.1.0-alpha.1`)
3. ✅ Verify artifacts download and install correctly
4. ✅ Document release process in `CONTRIBUTING.md`

**Success criteria**: Tagging `v0.1.0` produces working binaries for all platforms

---

### Phase 5: Optimizations (Week 4+)
**Goal**: Faster CI, better DX

**Tasks**:
1. ⏳ Implement advanced caching (`sccache` for Rust)
2. ⏳ Parallelize jobs further (split tests by module)
3. ⏳ Add manual workflow triggers (e.g., "Rebuild all platforms")
4. ⏳ Set up auto-merge for Dependabot patch updates
5. ⏳ Integrate coverage into GitHub status checks (block PR if drops)

**Success criteria**: CI runtime <3 minutes (from current 5 minutes)

---

## Success Metrics

**CI Pipeline Health**:
- ✅ <5 minute CI feedback loop (lint + test)
- ✅ >95% CI success rate (no flaky tests)
- ✅ <1% false positive rate (tests fail incorrectly)

**Release Cadence**:
- ✅ Weekly pre-releases during active development
- ✅ Bi-weekly stable releases after Phase 1
- ✅ Zero manual steps (fully automated via tags)

**Coverage Growth**:
- ✅ 10% baseline (Phase 1 start)
- ✅ 40% by Phase 2 completion
- ✅ 70% by stable 1.0 release

**Dependency Freshness**:
- ✅ <30 days behind latest patch versions
- ✅ <90 days behind latest minor versions
- ✅ Zero known High/Critical vulnerabilities

---

## Appendix: GitHub Actions Configuration Examples

### CI Workflow Skeleton
```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: 'npm'
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: npm ci
      - run: npm run check
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
```

### Release Workflow Skeleton
```yaml
name: Release
on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [windows-latest, macos-latest, ubuntu-22.04]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: 'npm'
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Install Linux deps
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev ...
      - run: npm ci
      - run: npm run tauri build
      - uses: actions/upload-artifact@v4
        with:
          name: artifacts-${{ matrix.platform }}
          path: src-tauri/target/release/bundle/
```

---

## Next Steps

1. **Review this document** with Han (Lead) and Fredrik (User)
2. **Approve strategy** before implementation
3. **Create workflows** in sequential order (Phase 1 → Phase 4)
4. **Test each phase** before moving to next
5. **Document learnings** in `.squad/agents/lando/history.md`

---

**Document Status**: APPROVED  
**Implementation**: READY TO START  
**Owner**: Lando (DevOps Expert)  
**Last Updated**: 2026-04-12
