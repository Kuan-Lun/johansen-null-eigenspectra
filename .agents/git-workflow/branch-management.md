# 分支管理

## 分支安全規則

- **僅刪除 AI 代理建立的分支**
- **永遠不刪除使用者建立的分支**
- 刪除分支前，請確認分支名稱以 `temp/ai-` 或類似 AI 識別開頭
- 合併前，務必檢查分支狀態和內容是否正確

## 分支命名規範

- 所有分支名稱請使用英文，不得使用中文字元
- 遵循以下傳統分支命名模式：
  - `feature/feature-name` - 新功能開發
  - `fix/bug-description` - 錯誤修復
  - `refactor/module-name` - 程式碼重構
  - `docs/topic` - 文件更新
  - `test/test-description` - 測試相關更改
  - `chore/task-description` - 維護任務

## 分支建立和管理

- 使用描述性且簡潔的分支名稱
- 從最新的主分支建立新分支
- 定期同步主分支的更改，保持分支更新
- 完成工作後，請及時刪除功能分支以保持倉庫整潔

## 工作流程決策樹

```text
是否為複雜的多文件重構？
├─ 是 → 建立臨時分支 → 完成後原子性提交 → 合併
└─ 否 → 直接在主分支進行原子性提交
```

## 範例

```bash
# 建立臨時工作分支（使用時間戳確保分支名稱唯一）
BRANCH_NAME="temp/ai-changes-$(date +%s)"
git checkout -b "$BRANCH_NAME"

# 在臨時分支上實作所有更改
# ...

# 按類型分別提交更改
# 例如功能相關檔案
git add <files-for-feature>
git commit -m "feat(scope): description"

# 例如文件相關檔案
git add <files-for-docs>
git commit -m "docs(scope): description"

# 切換回主分支並合併臨時分支
git checkout main
git merge --no-ff "$BRANCH_NAME"

# 合併完成後刪除臨時分支
git branch -d "$BRANCH_NAME"
```
