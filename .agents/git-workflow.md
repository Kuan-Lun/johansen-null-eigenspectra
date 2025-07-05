# Git Workflow Guidelines

本文件為 Git 相關操作的 AI Agent 提供詳細指導。

## 分支管理

### 分支命名規範

- 使用英文名稱命名所有分支（不使用中文字元）
- 遵循傳統的分支命名模式：
  - `feature/feature-name` - 新功能開發
  - `fix/bug-description` - 錯誤修復
  - `refactor/module-name` - 程式碼重構
  - `docs/topic` - 文件更新
  - `test/test-description` - 測試相關更改
  - `chore/task-description` - 維護任務

### 分支建立和管理

- 使用描述性但簡潔的分支名稱
- 從最新的主分支建立新分支
- 定期同步主分支的更改
- 完成工作後及時刪除功能分支

## 提交訊息規範

### 提交前檢查流程

在編寫提交訊息之前，按順序執行以下命令：

1. `git log --oneline -30` - 了解整體提交訊息格式和風格模式
2. `git log -15` - 學習詳細的提交訊息編寫技巧和慣例
3. `git status` - 確認已更改文件的清單及其狀態
4. `git diff <file>` - 檢查每個要提交文件的具體更改

### 提交訊息格式

使用以下格式編寫提交訊息：

```text
<type>(scope): <description>

[optional body]

[optional footer]
```

#### 類型（Type）

- `feat` - 新功能
- `fix` - 錯誤修復
- `docs` - 文件更改
- `style` - 程式碼格式更改（不影響程式碼邏輯）
- `refactor` - 程式碼重構（既不修復錯誤也不添加功能）
- `test` - 添加或修改測試
- `chore` - 建構流程或輔助工具的更改
- `perf` - 效能改進

#### 範圍（Scope）

使用有意義的範圍來指示受影響的模組或元件：

- `matrix` - 矩陣操作相關
- `johansen` - Johansen 統計相關
- `simulation` - 模擬計算相關
- `data` - 資料處理相關
- `cli` - 命令列介面相關
- `test` - 測試相關
- `docs` - 文件相關

#### 描述（Description）

- 使用英文編寫簡潔的描述
- 使用祈使語氣（如 "add" 而不是 "adds" 或 "added"）
- 不要以大寫字母開頭
- 不要以句號結尾
- 限制在 50 個字元以內

### 提交策略

#### 原子性提交

- 根據文件差異和修改內容編寫提交訊息
- 當修改多個文件時，按類型（feat, fix, docs, style, refactor, test, chore）分組進行原子性提交
- 每個提交應該代表一個邏輯上的更改單元
- 避免在單個提交中混合不同類型的更改

#### 多類型修改處理

當涉及多種類型的修改時：

1. 首先建立臨時分支
2. 完成所有程式碼更改
3. 按類型分別提交
4. 合併回主分支
5. 清理 AI 建立的分支（僅刪除 AI 建立的分支，不刪除使用者建立的分支）

**格式說明**: 使用圓括號 `(scope)` 而非方括號 `[scope]`，例如：

- 正確：`feat(matrix): add eigenvalue calculation`
- 正確：`fix(johansen): correct statistical test logic`
- 錯誤：`feat[matrix]: add eigenvalue calculation`

## 分支操作規範

### 分支建立工作流程

```bash
# 建立臨時工作分支（使用時間戳確保唯一性）
BRANCH_NAME="temp/ai-changes-$(date +%s)"
git checkout -b "$BRANCH_NAME"

# 實作所有更改
# ...

# 按類型分別提交
git add <files-for-feature>
git commit -m "feat(scope): description"

git add <files-for-docs>
git commit -m "docs(scope): description"

# 合併回主分支
git checkout main
git merge --no-ff "$BRANCH_NAME"

# 刪除臨時分支
git branch -d "$BRANCH_NAME"
```

**注意事項**：在 AI 代理環境中，建議使用以下簡化流程：

```bash
# 直接在主分支上按類型分別提交（適用於 AI 環境）
git add <files-for-feature>
git commit -m "feat(scope): description"

git add <files-for-docs>  
git commit -m "docs(scope): description"

# 或者使用預定義的分支名稱
git checkout -b temp/ai-refactor
# ... 進行更改和提交
git checkout main
git merge --no-ff temp/ai-refactor
git branch -d temp/ai-refactor
```

### 分支安全規則

- **僅刪除 AI 代理建立的分支**
- **永遠不刪除使用者建立的分支**
- 刪除分支前確認分支名稱以 `temp/ai-` 或類似 AI 識別開頭
- 合併前檢查分支狀態和內容

## 程式碼審查準備

### 審查前檢查

- 驗證統計實作的正確性
- 檢查更改對效能的影響
- 確保程式碼品質和一致性
- 執行所有相關測試

### 審查重點

- 不強制執行向後相容性（除非明確要求）
- 專注於程式碼品質和效能，而不是傳統支援
- 驗證數值計算的準確性
- 檢查並行安全性

## 協作規範

### 衝突解決

- 優先使用 rebase 而不是 merge 來保持歷史清潔
- 解決衝突時保持程式碼功能完整性
- 測試衝突解決後的程式碼
- 記錄重要的衝突解決決策

### 遠端儲存庫同步

- 定期取得遠端更改
- 推送前先拉取最新更改
- 使用 force push 時要格外小心
- 確保團隊協作的順暢

## 特殊注意事項

### 大文件處理

- 避免提交大型二進位文件
- 使用 Git LFS 處理必要的大文件
- 定期清理不必要的文件

### 敏感資訊保護

- 不要提交密碼、金鑰或其他敏感資訊
- 使用 .gitignore 排除臨時文件和建構產物
- 檢查提交內容中的敏感資料

### 版本更新

- 更新 `Cargo.toml` 中的 `version` 時，必須在 `.github/releases` 目錄新增對應版本的說明檔
