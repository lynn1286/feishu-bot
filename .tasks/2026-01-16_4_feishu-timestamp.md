# 背景

文件名：2026-01-16_4
创建于：2026-01-16
创建者：Lynn
主分支：main
任务分支：task/feishu-timestamp_2026-01-16_4
Yolo 模式：Off

# 任务描述

飞书机器人中，时间不应该是拿 timestamp，应该是跟列表的时间一致才对。

# 分析

飞书消息构建时使用的 `timestamp` 字段目前取自 Sentry Payload 中的事件时间。但用户指出应使用 `datetime` 字段，以确保时间准确性。
我们需要从 Payload 中提取 `datetime`（UTC 格式），并将其转换为服务器本地时间显示。

# 提议的解决方案

1.  在 `src/models/sentry.rs` 中为 `SentryWebhookPayload` 添加 `get_datetime` 方法。
2.  在 `src/services/message_builder.rs` 中，优先使用 `get_datetime` 获取时间，并使用 `chrono` 库将其转换为本地时间格式 (`%Y-%m-%d %H:%M:%S`)。如果获取失败，则回退到 `Local::now()`。

# 任务进度

2026-01-16 - 任务初始化，开始调查。
2026-01-16 - [EXECUTE] 修改 `src/services/message_builder.rs`，将飞书消息时间戳更改为服务器当前时间。
2026-01-16 - [EXECUTE] 修改 `src/models/sentry.rs` 添加 `get_datetime` 方法，并在 `src/services/message_builder.rs` 中使用该字段并处理时区转换。

# 最终审查

实施与计划完全匹配。消息构建器现在优先使用 payload 中的 `datetime` 字段（UTC），并将其转换为服务器本地时间显示，确保了与事件发生时间的准确一致。
