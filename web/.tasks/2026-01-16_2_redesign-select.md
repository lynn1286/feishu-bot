# 背景

文件名：2026-01-16_2_redesign-select.md
创建于：2026-01-16_13:45:00
创建者：Antigravity
主分支：main
任务分支：task/redesign-select_2026-01-16_2
Yolo 模式：Off

# 任务描述

当前 select 组件的交互和设计都不好，重新设计下。
追加任务：右上角的用户菜单不应该使用 Select 组件，应改为 DropdownMenu，避免触发器显示选中值。

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

- 当前 Select 组件使用 Radix UI + Tailwind CSS。
- 视觉上比较单调，边框较淡，缺乏质感。
- 下拉菜单样式简单，缺乏动效。
- 触发器在不同场景下的样式不统一（Layout 里的透明触发器 vs Form 里的带边框触发器）。
- Header 中的用户菜单错误地使用了 Select 组件，理应使用 DropdownMenu。

# 提议的解决方案

- [x] 增强视觉效果：引入玻璃拟态 (Glassmorphism) 效果，通过 `backdrop-blur` 和透明度提升下拉菜单质感。
- [x] 优化交互反馈：添加更平滑的微动效（Scale, Opacity），通过 Tailwind 的 `data-state` 属性精确控制动画。
- [x] 提升设计细节：优化阴影 (Shadows)、圆角 (Border Radius) 和间距。
- [x] 统一样式系统：为 SelectTrigger 提供变体支持（如 outline, ghost, premium 等）。
- [x] 引入 DropdownMenu：安装 `@radix-ui/react-dropdown-menu` 并实现相应组件，用于 Layout 中的用户菜单。

# 当前执行步骤："已完成"

- [x] 安装依赖 `@radix-ui/react-dropdown-menu`
- [x] 实现 `dropdown-menu.tsx` 核心组件
- [x] 修改 `select.tsx` 核心逻辑（完成重构）
- [x] 更新 `layout.tsx`：将 Select 替换为 DropdownMenu
- [x] 验证 Header 表现

实施清单：

1. [x] 安装依赖：`pnpm add @radix-ui/react-dropdown-menu`
2. [x] 创建 `dropdown-menu.tsx`，采用高级感设计（玻璃拟态/动效）
3. [x] 重定义 `SelectTrigger` 样式，增加 `variant` 属性支持
4. [x] 优化 `SelectContent` 样式
5. [x] 优化 `SelectItem` 的选中和悬停状态
6. [x] 更新 `layout.tsx` 中的用户菜单实现
7. [x] 验证所有页面的整体表现

# 任务进度

[2026-01-16_13:45:00]

- 已修改：N/A
- 更改：初始化任务文件
- 原因：开始重新设计 Select 组件
- 状态：成功

[2026-01-16_13:55:00]

- 已修改：.tasks/2026-01-16_2_redesign-select.md
- 更改：更新任务描述，追加 DropdownMenu 需求
- 原因：用户反馈 Header 不应使用 Select
- 状态：成功

[2026-01-16_14:20:00]

- 已修改：dropdown-menu.tsx, select.tsx, layout.tsx
- 更改：完成重构与替换
- 原因：任务执行完毕
- 状态：成功

# 最终审查

实施完全符合用户要求，解决了右上角菜单的语义与交互问题，并提升了 Select 组件的视觉表现。

实施与计划完全匹配。
