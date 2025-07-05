# 版本更新

## 版本更新前的檢查

在更新 `Cargo.toml` 中的 `version` 欄位之前，請先執行以下命令，確認自上一版本以來的所有實際變更：

1. 使用 `git log --oneline v{上一版本}..HEAD` 查看提交摘要。
2. 使用 `git tag --list | grep v{上一版本}` 確認版本標籤是否存在。
3. 使用 `git show --stat v{上一版本}..{版本更新前的最後提交}` 查看詳細變更統計。

## 版本更新提交順序

版本更新應包含以下兩個獨立且原子性的提交：

1. **第一次提交** - 版本號更新：

    ```bash
    git add Cargo.toml
    git commit -m "chore(version): bump version from {舊版本} to {新版本}"
    ```

2. **第二次提交** - 發布說明：

    - **文件命名**：請在 `.github/releases` 目錄下新增名為 `RELEASE_NOTES_v{版本}.md` 的詳細說明檔案。
    - **文件內容**：內容必須根據實際的 git 提交記錄和程式碼變更，包含以下分類：
    - 主要功能變更 (Breaking Changes)
    - 新增功能 (New Features)
    - 錯誤修復 (Bug Fixes)
    - 重構和改進 (Refactoring and Improvements)
    - 文件更新 (Documentation Updates)

    - **命名示例**：
    - 正確：`RELEASE_NOTES_v0.8.0.md`
    - 正確：`RELEASE_NOTES_v1.2.3.md`
    - 錯誤：`v0.8.0.md`（缺少前綴）
    - 錯誤：`release_notes_v0.8.0.md`（格式不統一）

    ```bash
    git add .github/releases/RELEASE_NOTES_v{版本}.md
    git commit -m "docs(release): add comprehensive release notes for version {版本}"
    ```
