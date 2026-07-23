# Unsigned IPA Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fork `madsmtm/ruffle-ios`, build an unsigned physical-device IPA in GitHub Actions, verify the artifact, and deliver it to the user's iPhone through Taildrop.

**Architecture:** A manually dispatched workflow will run on a GitHub-hosted macOS runner, prepare Java and the repository-selected Rust nightly toolchain, build the existing Xcode scheme with signing disabled, and package the resulting application bundle as an IPA. GitHub stores the IPA and SHA-256 checksum for seven days; Taildrop delivery occurs from the local Mac after independent artifact verification.

**Tech Stack:** GitHub Actions, Xcode/xcodebuild, Rust/rustup/Cargo, Java 17, ZIP/IPA packaging, GitHub CLI, Tailscale Taildrop.

---

## File Structure

- Create `.github/workflows/build-unsigned-ipa.yml`: owns the complete unsigned device-build, package validation, checksum, and artifact-upload pipeline.
- Existing `build-in-xcode.sh`: invoked by Xcode without modification.
- Existing `rust-toolchain.toml`: selects the project's Rust nightly toolchain without modification.

### Task 1: Create the GitHub Fork and Configure Remotes

**Files:**
- No files created or modified.

- [ ] **Step 1: Confirm the authenticated GitHub account and clean tracked state**

Run:

```bash
gh auth status
git status --short --branch
```

Expected: GitHub account `seva3125` is active; only the audit-generated `.gstack/` directory is untracked.

- [ ] **Step 2: Create the fork and add it as `origin`**

Run:

```bash
gh repo fork --remote
git remote -v
```

Expected: `origin` points to `seva3125/ruffle-ios` and `upstream` points to `madsmtm/ruffle-ios`.

### Task 2: Add the Unsigned IPA Workflow

**Files:**
- Create: `.github/workflows/build-unsigned-ipa.yml`

- [ ] **Step 1: Create the workflow**

Create `.github/workflows/build-unsigned-ipa.yml` with:

```yaml
name: Build unsigned IPA

on:
  workflow_dispatch:

permissions:
  contents: read

concurrency:
  group: unsigned-ipa-${{ github.ref }}
  cancel-in-progress: true

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    name: Build unsigned iPhone IPA
    runs-on: macos-15
    timeout-minutes: 90

    steps:
      - name: Check out repository
        uses: actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4

      - name: Set up Java
        uses: actions/setup-java@c1e323688fd81a25caa38c78aa6df2d33d3e20d9 # v4
        with:
          distribution: temurin
          java-version: "17"

      - name: Prepare Rust toolchain
        shell: bash
        run: |
          set -euo pipefail
          rustup show active-toolchain
          rustup target add aarch64-apple-ios

      - name: Cache Cargo files
        uses: actions/cache@0057852bfaa89a56745cba8c7296529d2fc39830 # v4
        with:
          path: |
            ~/.cargo/git
            ~/.cargo/registry
          key: cargo-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('Cargo.lock', 'rust-toolchain.toml') }}
          restore-keys: |
            cargo-${{ runner.os }}-${{ runner.arch }}-

      - name: Fetch locked dependencies
        shell: bash
        run: cargo fetch --locked

      - name: Build unsigned application
        shell: bash
        env:
          CARGO_NET_OFFLINE: "true"
        run: |
          set -euo pipefail
          xcodebuild \
            -project ruffle-ios.xcodeproj \
            -scheme ruffle-ios \
            -configuration Release \
            -sdk iphoneos \
            -destination 'generic/platform=iOS' \
            -derivedDataPath "$RUNNER_TEMP/DerivedData" \
            CODE_SIGNING_ALLOWED=NO \
            CODE_SIGNING_REQUIRED=NO \
            CODE_SIGN_IDENTITY="" \
            DEVELOPMENT_TEAM="" \
            clean build

      - name: Package and verify IPA
        shell: bash
        run: |
          set -euo pipefail
          app_path="$RUNNER_TEMP/DerivedData/Build/Products/Release-iphoneos/ruffle-ios.app"
          package_root="$RUNNER_TEMP/ipa-package"
          ipa_path="$GITHUB_WORKSPACE/Ruffle-unsigned.ipa"

          test -d "$app_path"
          test -f "$app_path/Info.plist"
          test -f "$app_path/ruffle-ios"
          /usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$app_path/Info.plist"

          mkdir -p "$package_root/Payload"
          ditto "$app_path" "$package_root/Payload/ruffle-ios.app"
          (
            cd "$package_root"
            /usr/bin/zip -qry "$ipa_path" Payload
          )

          /usr/bin/unzip -t "$ipa_path"
          /usr/bin/unzip -l "$ipa_path" | grep -F 'Payload/ruffle-ios.app/Info.plist'
          test ! -d "$package_root/Payload/ruffle-ios.app/_CodeSignature"
          shasum -a 256 "$ipa_path" > "$ipa_path.sha256"

      - name: Upload unsigned IPA
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4
        with:
          name: Ruffle-unsigned-ipa
          path: |
            Ruffle-unsigned.ipa
            Ruffle-unsigned.ipa.sha256
          if-no-files-found: error
          retention-days: 7
          compression-level: 0
```

- [ ] **Step 2: Install the local workflow linter**

Run:

```bash
brew install actionlint
```

Expected: `actionlint` is installed or already current.

- [ ] **Step 3: Validate the workflow before committing**

Run:

```bash
actionlint .github/workflows/build-unsigned-ipa.yml
git diff --check
git diff -- .github/workflows/build-unsigned-ipa.yml
```

Expected: `actionlint` and `git diff --check` exit successfully, and the diff contains only the intended workflow.

- [ ] **Step 4: Commit the workflow**

Run:

```bash
git add .github/workflows/build-unsigned-ipa.yml
git commit -m "Add unsigned IPA build workflow"
```

Expected: one commit containing the workflow.

### Task 3: Publish and Execute the Workflow

**Files:**
- No additional files.

- [ ] **Step 1: Push the implementation branch**

Run:

```bash
git push -u origin ci/unsigned-ipa
```

Expected: branch `ci/unsigned-ipa` exists on `seva3125/ruffle-ios`.

- [ ] **Step 2: Register the workflow on the fork's default branch**

Run:

```bash
git push origin ci/unsigned-ipa:main
```

Expected: the fork's default branch is fast-forwarded to the reviewed workflow
commit, allowing GitHub to register the manual workflow.

- [ ] **Step 3: Dispatch the workflow**

Run:

```bash
gh workflow run build-unsigned-ipa.yml --repo seva3125/ruffle-ios --ref main
```

Expected: GitHub accepts the dispatch.

- [ ] **Step 4: Watch the run to completion**

Run:

```bash
run_id="$(gh run list --repo seva3125/ruffle-ios --workflow build-unsigned-ipa.yml --branch main --limit 1 --json databaseId --jq '.[0].databaseId')"
gh run watch "$run_id" --repo seva3125/ruffle-ios --exit-status
```

Expected: the run concludes successfully. If it fails, inspect it with:

```bash
gh run view "$run_id" --repo seva3125/ruffle-ios --log-failed
```

and correct only the demonstrated failure before rerunning the static checks and workflow.

### Task 4: Verify the Produced Artifact

**Files:**
- Download to: `work/artifacts/Ruffle-unsigned.ipa`
- Download to: `work/artifacts/Ruffle-unsigned.ipa.sha256`

- [ ] **Step 1: Download the workflow artifact**

Run:

```bash
mkdir -p ../artifacts
gh run download "$run_id" \
  --repo seva3125/ruffle-ios \
  --name Ruffle-unsigned-ipa \
  --dir ../artifacts
```

Expected: both artifact files exist under `../artifacts`.

- [ ] **Step 2: Verify the checksum, archive structure, and absence of a signature**

Run:

```bash
(
  cd ../artifacts
  shasum -a 256 -c Ruffle-unsigned.ipa.sha256
  unzip -t Ruffle-unsigned.ipa
  unzip -l Ruffle-unsigned.ipa | grep -F 'Payload/ruffle-ios.app/Info.plist'
  if unzip -l Ruffle-unsigned.ipa | grep -Fq 'Payload/ruffle-ios.app/_CodeSignature/'; then
    echo "Unexpected code signature" >&2
    exit 1
  fi
)
```

Expected: checksum reports `OK`, ZIP validation succeeds, `Info.plist` exists, and no `_CodeSignature` directory is found.

### Task 5: Audit and Deliver Through Taildrop

**Files:**
- No source files modified.

- [ ] **Step 1: Audit the completed repository change**

Run:

```bash
git diff upstream/main...HEAD --check
git diff upstream/main...HEAD
actionlint .github/workflows/build-unsigned-ipa.yml
git status --short --branch
```

Expected: the diff contains only the design, plan, and workflow; no lint errors or whitespace errors appear; `.gstack/` remains the only unrelated untracked path.

- [ ] **Step 2: List Taildrop targets**

Run:

```bash
tailscale file cp --targets
```

Expected: `iphone-13-pro` appears as target `100.65.190.63`.

- [ ] **Step 3: Transfer the verified IPA**

Run:

```bash
tailscale file cp ../artifacts/Ruffle-unsigned.ipa 100.65.190.63:
```

Expected: Taildrop reports a successful transfer. The user can retrieve the file from the Files app and open it with SideStore.

- [ ] **Step 4: Report the exact commit, workflow run, artifact checksum, and transfer result**

Run:

```bash
git rev-parse HEAD
gh run view "$run_id" --repo seva3125/ruffle-ios --json url,conclusion,headSha
cat ../artifacts/Ruffle-unsigned.ipa.sha256
```

Expected: the report identifies the pushed commit, successful run URL and conclusion, matching SHA-256 checksum, and confirmed Taildrop result.
