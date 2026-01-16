# Feishu Bot

> Sentry 告警自动转发到飞书群的轻量级服务

[![Rust](https://img.shields.io/badge/Rust-1.92+-orange?logo=rust)](https://www.rust-lang.org/)
[![Node](https://img.shields.io/badge/Node-20+-green?logo=node.js)](https://nodejs.org/)
[![React](https://img.shields.io/badge/React-19-blue?logo=react)](https://react.dev/)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue?logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

## 功能特性

- **Webhook 接收** - 接收 Sentry Internal Integration 告警
- **智能路由** - 按项目、环境、告警级别匹配规则
- **项目映射** - 自定义 Sentry 项目更名和描述
- **多种消息格式** - 纯文本 / 富文本 / 交互式卡片
- **Web 管理界面** - 现代化的可视化配置，支持认证登录
- **告警历史** - 记录所有告警，支持重试发送
- **日志管理** - 日志自动按天轮转，保留最近 7 天
- **安全验证** - 支持 Sentry Webhook 签名验证

---

## 快速开始

### 安装

#### Windows

Windows 仅支持 **Docker** 方式运行，请参考下方的 Docker 安装说明。

#### macOS / Linux

```bash
# 使用安装脚本（推荐）
curl -sSL https://raw.githubusercontent.com/lynn1286/feishu-bot/main/install.sh | bash

# 或手动安装
git clone https://github.com/lynn1286/feishu-bot.git
cd feishu-bot
cd web && pnpm install && pnpm run build && cd ..
cargo build --release
cargo install --path .
```

#### Docker

```bash
git clone https://github.com/lynn1286/feishu-bot.git
cd feishu-bot
docker build -t feishu-bot .
docker run -d \
  --name feishu-bot \
  -p 3000:3000 \
  -v feishu-bot-data:/app/data \
  -e DATABASE_URL=sqlite:/app/data/data.db?mode=rwc \
  feishu-bot
```

**环境变量**：

| 变量                   | 默认值                              | 说明                                      |
| ---------------------- | ----------------------------------- | ----------------------------------------- |
| `DATABASE_URL`         | `sqlite:/app/data/data.db?mode=rwc` | 数据库连接字符串                          |
| `FEISHU_BOT_DATA_DIR`  | `/app/data`                         | 数据目录                                  |
| `SENTRY_CLIENT_SECRET` | -                                   | Sentry Client Secret (可选，用于签名验证) |
| `RUST_LOG`             | `info`                              | 日志级别 (error/warn/info/debug/trace)    |

启动后访问：http://localhost:3000/admin

首次启动会自动创建 admin 用户，随机密码会显示在日志中。

### 配置

1. **添加飞书群** - 在飞书群中添加「自定义机器人」，获取 Webhook URL，在管理界面添加
2. **配置项目映射** - (可选) 为 Sentry 项目 ID 设置自定义名称和描述
3. **配置路由规则** - 设置项目、环境、级别的匹配条件
4. **配置 Sentry** - 创建 Internal Integration，Webhook URL 填写 `http://your-server:3000/webhook/sentry`

---

## 开发

```bash
git clone https://github.com/lynn1286/feishu-bot.git
cd feishu-bot

# 前端开发
cd web && pnpm install && pnpm run dev

# 后端开发（另开终端）
cargo run -- serve

# 构建发布版本
cd web && pnpm run build && cd ..
cargo build --release
```

**环境变量配置**（开发环境）：

```bash
# .env
DATABASE_URL=sqlite:data.db?mode=rwc
RUST_LOG=debug
```

**代码检查与格式化**：

```bash
# Rust 代码检查
cargo clippy

# Rust 代码格式化
cargo fmt

# 运行测试
cargo test
```

**技术栈**：Rust + Axum + SQLite / React + TypeScript + Tailwind CSS

---

## License

[MIT](LICENSE)

<p align="center">
  <a href="https://github.com/lynn1286/feishu-bot">GitHub</a>
</p>
