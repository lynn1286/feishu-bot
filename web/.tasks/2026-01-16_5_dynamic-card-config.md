# 背景

文件名：2026-01-16_5_dynamic-card-config.md
创建于：2026-01-16_14:45:00
创建者：Antigravity
主分支：main
任务分支：task/dynamic-card-config_2026-01-16_5
Yolo 模式：Off

# 任务描述

实现飞书卡片的动态配置化。用户可以在前端界面选择要在卡片中展示的字段，后端根据配置动态构建飞书交互式卡片。

# 项目概览

项目是一个飞书机器人管理后台，后端使用 Rust (Axum + SQLx + SQLite)，前端使用 React。

⚠️ 警告：永远不要修改此部分 ⚠️

- 系统思维：从整体架构到具体实现进行分析
- 辩证思维：评估多种解决方案及其利弊
- 创新思维：打破常规模式，寻求创造性解决方案
- 批判性思维：从多个角度验证和优化解决方案
- 模式声明：[MODE: MODE_NAME]
- 严格遵循 RESEARCH -> INNOVATE -> PLAN -> EXECUTE -> REVIEW 流程
  ⚠️ 警告：永远不要修改此部分 ⚠️

# 分析

- 数据库 `feishu_groups` 需要存储配置 JSON。
- 后端 `message_builder.rs` 需要从硬编码逻辑转向驱动逻辑。
- 前端需要交互友好的字段选择器和预览。

# 提议的解决方案

- [x] 数据库 Schema 更新。
- [x] 后端模型与数据库层更新。
- [x] 前端 API 定义更新。
- [x] 前端配置 UI 实现。
- [x] 后端动态卡片构建逻辑实现。

# 当前执行步骤："已经完成"

- [x] 审批实现计划

实施清单：

1. [x] 修改 `src/db.rs`：添加 `card_config_json` 字段。
2. [x] 修改 `src/db/groups.rs`：更新模型和 SQL。
3. [x] 修改 `web/src/api/client.ts`：更新前端接口。
4. [x] 修改 `web/src/pages/groups.tsx`：实现配置 UI 和预览。
5. [x] 修改 `src/services/message_builder.rs`：实现动态渲染逻辑。
6. [x] 验证全流程。

# 任务进度

[2026-01-16_14:00:00]

- 已修改：N/A
- 更改：初始化任务文件
- 原因：开始实施动态卡片配置
- 状态：成功

[2026-01-16_14:15:00]

- 已修改：src/db.rs, src/db/groups.rs, web/src/api/client.ts, src/services/message_builder.rs, web/src/pages/groups.tsx
- 更改：实现全套动态配置功能
- 原因：完成用户需求
- 状态：成功

# 最终审查

N/A
