# 背景

文件名：2026-01-16_4_enhance-card-info.md
创建于：2026-01-16_14:40:00
创建者：Antigravity
主分支：main
任务分支：task/enhance-card-info_2026-01-16_4
Yolo 模式：Off

# 任务描述

当前交互式卡片好像不能将关键信息（如具体的错误堆栈、Tags 等）整理发送到飞书机器人，需要增强卡片的信息含量和排版。

# 项目概览

项目是一个飞书机器人管理后台，后端使用 Rust (Axum + SQLx)，前端使用 React。

⚠️ 警告：永远不要修改此部分 ⚠️

- 系统思维：从整体架构到具体实现进行分析
- 辩证思维：评估多种解决方案及其利弊
- 创新思维：打破常规模式，寻求创造性解决方案
- 批判性思维：从多个角度验证和优化解决方案
- 模式声明：[MODE: MODE_NAME]
- 严格遵循 RESEARCH -> INNOVATE -> PLAN -> EXECUTE -> REVIEW 流程
  ⚠️ 警告：永远不要修改此部分 ⚠️

# 分析

- `message_builder.rs` 负责构建卡片，目前仅包含了 title, project, environment, level, timestamp, rule。
- `vars.message`（即 Sentry 的详细描述）在代码中已被提取但未放入卡片元素中。
- Payload 中还含有 `platform`, `tags`, `event_id`, `issue_id` 等非常有用的调试信息，目前被忽略。

# 提议的解决方案

- [x] 在 `TemplateVars` 中加入更多字段。
- [x] 在 `build_card_message` 中引入 `vars.message` 展示。
- [x] 增加 `platform` 展示。
- [x] 尝试展示告警的 `tags`。
- [x] 优化卡片布局，使用分块显示提升阅读效率。

# 当前执行步骤："已完成"

- [x] 审批实现计划
- [x] 修改 `message_builder.rs`
- [x] 验证后端逻辑（编译测试通过）

实施清单：

1. [x] 修改 `message_builder.rs`：扩展 `TemplateVars` 结构及 `from_payload` 逻辑。
2. [x] 修改 `message_builder.rs`：更新 `build_card_message` UI 布局。
3. [x] 验证后端逻辑（编译测试）。
4. [x] 验证效果。

# 任务进度

[2026-01-16_13:56:00]

- 已修改：N/A
- 更改：初始化任务文件
- 原因：开始增强卡片信息显示
- 状态：成功

[2026-01-16_14:45:00]

- 已修改：message_builder.rs
- 更改：扩展卡片信息展示，加入详细错误信息、Tags、平台及 ID
- 原因：提升告警卡片的调试价值
- 状态：成功

# 最终审查

卡片信息密度大幅提升，包含了调试所需的所有关键上下文。
实施与计划完全匹配。
