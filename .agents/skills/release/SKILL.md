---
name: release
description: Prepare a new release of this project. Use when asked to release, bump the version, or generate release notes. Analyzes commits since the last tag, bumps the version in Cargo.toml, writes release notes from the actual git history, and creates the release commits; tagging, builds, and the draft GitHub Release then run automatically in CI after push.
---

# Release

發布本專案新版本的完整流程。本 skill 為工具中立的通用標準格式，
適用於任何能執行 shell 指令的 AI 開發工具（Claude Code、Codex 等）。

## 前置檢查

依序確認，任一項失敗即中止並回報：

1. 工作樹乾淨且位於 master：`git status --short` 無輸出
2. 品質關卡全數通過（一律從專案根目錄執行，見 `AGENTS.md` 建構環境一節）：
   - `cargo fmt --check`
   - `cargo clippy --all-targets`（零警告）
   - `cargo test`
3. 取得上一版本：`git describe --tags --abbrev=0`（格式為 `v{版本}`）

## 決定新版本號

1. 檢視自上一版本以來的全部變更：
   - `git log v{上一版本}..HEAD --oneline`
   - 必要時以 `git show --stat` 檢視關鍵提交
2. 依 semver 判斷（0.x 時期慣例）：
   - 破壞性變更（檔案格式、公開 API、行為不相容）→ 提升 minor
   - 僅新增功能 → 提升 minor
   - 僅錯誤修復與內部改進 → 提升 patch
3. 使用者有明確指定版本號時，以使用者指定為準

## 執行步驟

1. 更新 `Cargo.toml` 的 `version` 欄位，執行 `cargo check` 同步 `Cargo.lock`
2. 撰寫 `.github/releases/RELEASE_NOTES_v{新版本}.md`：
   - 語言為英文；命名必須是 `RELEASE_NOTES_v{版本}.md`
   - 分類結構參考既有檔案（依實際變更取捨）：Overview、Breaking Changes、
     New Features、Bug Fixes、Refactoring and Improvements、
     Documentation Updates、Testing
   - 內容必須根據實際的 git 提交記錄與程式碼變更撰寫，不得杜撰
3. 依 `.agents/git-workflow/version-update.md` 完成兩個原子提交：
   - `chore(version): bump version from {舊版本} to {新版本}`
     （包含 `Cargo.toml` 與 `Cargo.lock`）
   - `docs(release): add comprehensive release notes for version {新版本}`
4. **請使用者審閱** release notes 與提交內容；經使用者同意後才
   `git push` 到 master

## Push 之後的自動化（無需手動操作）

- `.github/workflows/auto-tag.yml`：偵測 `Cargo.toml` 版本變更後，
  驗證 release notes 存在、執行測試、建立並推送 `v{版本}` tag、
  派發 release workflow
- `.github/workflows/release.yml`：建置三平台 binary，
  以 release notes 內容建立 **draft** GitHub Release
- 最後一步為人工把關：請使用者到 GitHub Releases 頁面將 draft 正式發布
