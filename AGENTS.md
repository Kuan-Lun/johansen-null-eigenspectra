# Agents

This document contains information about AI agents and automated tools used in this project.

## Purpose

Document AI agent interactions, configurations, and best practices for maintaining and developing the Johansen Null Eigenspectra project.

## Agent Guidelines Index

本文件已分割為多個專門化的指導文件，以減少上下文閱讀量並便於添加詳細要求。AI Agent 應根據具體任務類型閱讀相應的指導文件：

### 按任務類型分類的指導文件

- **[程式碼生成與實作](./.agents/code-generation.md)** - 用於程式碼編寫、重構、新功能實作
- **[測試相關](./.agents/testing.md)** - 用於編寫測試、驗證功能、測試驅動開發
- **[Git 工作流程](./.agents/git-workflow.md)** - 用於版本控制、分支管理、提交訊息
- **[文件編寫](./.agents/documentation.md)** - 用於文件更新、API 文件、註解規範
- **[效能最佳化](./.agents/performance.md)** - 用於效能分析、最佳化建議、基準測試

### 使用指南

1. **識別任務類型** - 確定當前任務屬於哪個類別
2. **閱讀對應指導** - 只閱讀相關的指導文件，減少不必要的上下文
3. **遵循具體規範** - 按照專門指導文件中的詳細要求執行
4. **組合使用** - 複雜任務可能需要參考多個指導文件

## 快速參考

### 專案概要

- Johansen 協整檢驗的統計計算專案
- 使用 Rust 程式語言，重視數值精度和計算效能

### 語言使用規範

- **使用者介面和錯誤訊息**: 英文
- **內部程式碼註解**: 繁體中文  
- **API 文件註解**: 英文
- **Git 提交訊息**: 英文

### 程式碼品質

- 執行 `cargo fmt` 和 `cargo clippy`
- 確保通過 `cargo check` 無警告

## 更新指南

當需要增加新的指導內容時：

1. 確定內容歸屬的類別
2. 更新對應的專門指導文件
3. 如需要新類別，建立新的指導文件
4. 在本文件中新增索引鏈接

## Notes

This modular approach allows AI agents to focus on specific task-related guidelines, improving efficiency and reducing context overhead while maintaining comprehensive coverage of project requirements.
