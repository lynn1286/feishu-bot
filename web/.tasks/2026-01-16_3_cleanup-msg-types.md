# 背景

文件名：2026-01-16_3_cleanup-msg-types.md
创建于：2026-01-16_14:25:00
创建者：Antigravity
主分支：main
任务分支：task/cleanup-msg-types_2026-01-16_3
Yolo 模式：Off

# 任务描述

当前飞书机器人配置只支持交互式卡片，其他的（纯文本、富文本）都不支持，但是 UI 依旧显示，清理它。

# 项目概览

项目是一个飞书机器人管理后台，使用 React + TypeScript + Vite + Tailwind CSS (v4) + Radix UI。

⚠️ 警告：永远不要修改此部分 ⚠️

- 系统思维：从整体架构到具体实现进行分析
- 辩证思维：评估多种解决方案及其利弊
- 创新思维：打破常规模式，寻求创造性解决方案
- 批判性思维：从多个角度验证和优化解决方案
- 模式声明：[MODE: MODE_NAME]
- 严格遵循 RESEARCH -> INNOVATE -> PLAN -> EXECUTE -> REVIEW 流程
  ⚠️ 警告：永远不要修改此部分 ⚠️

# 分析

- `groups.tsx` 定义了 `MSG_TYPE_OPTIONS` 包含 `interactive`, `post`, `text`。
- 表单根据 `msgType` 动态渲染不同的模板编辑器。
- 需要将逻辑简化为仅支持 `interactive`。

# 提议的解决方案

- [x] 移除 `groups.tsx` 中的 `post` 和 `text` 选项。
- [x] 简化表单逻辑，默认并强制使用 `interactive`。
- [x] 移除冗余的条件渲染代码块。
- [x] 优化列表页的 Badge 显示。

# 当前执行步骤："已完成"

- [x] 审批实现计划
- [x] 修改 `groups.tsx`

实施清单：

1. [x] 修改 `groups.tsx`：移除 `MSG_TYPE_OPTIONS` 冗余项。
2. [x] 修改 `groups.tsx`：移除表单中的 Radio 选择器。
3. [x] 修改 `groups.tsx`：移除 `msgType === 'text'` 和 `msgType === 'post'` 的渲染逻辑。
4. [x] 修改 `groups.tsx`：将卡片配置项改为直接渲染。
5. [x] 验证功能。

# 任务进度

[2026-01-16_13:50:00]

- 已修改：N/A
- 更改：初始化任务文件
- 原因：开始清理不支持的消息类型
- 状态：成功

[2026-01-16_14:30:00]

- 已修改：groups.tsx
- 更改：移除不支持的消息类型 UI 和逻辑
- 原因：由于后端仅支持交互式卡片，清理前端多余配置项
- 状态：成功

# 最终审查

UI 已成功简化，仅展示受支持的交互式卡片配置。
实施与计划完全匹配。
