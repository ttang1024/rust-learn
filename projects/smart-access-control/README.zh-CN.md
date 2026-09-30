# Smart Access Control（智能门禁系统）

[English](README.md) | **简体中文**

一个用 Rust（Axum + Tokio + PostgreSQL）编写的**纯软件模拟**的物理门禁后端，
附带 React 管理面板（[`dashboard/`](dashboard/README.md)）。这是一个学习项目，
不与任何真实硬件或厂商协议通信。

- 设计与请求流程：[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)（英文）
- 安全评审与已知风险：[SECURITY.md](SECURITY.md)

## 功能

- **通行判定**：刷卡时依次检查卡、持卡人、门状态、分组权限和时间表；每次尝试都写入
  只追加的事件日志。任何缺失或检查失败都判为拒绝。
- **管理 API**（`/api/v1`）：用户、卡、门、门禁分组、时间表、权限、事件和审计日志。
  角色：`admin`（读写）和 `viewer`（只读）。
- **认证**：Argon2id 密码哈希、短时效 JWT 访问令牌、可轮换并检测重用的刷新令牌、
  登录限流。
- **模拟门控制器**：每个控制器独立密钥、心跳与离线检测，另有用于演示的进程内模拟器。
- **实时事件**（WebSocket）、请求 ID、JSON 日志和 Prometheus 指标。

## 环境要求

- Rust stable（edition 2024）
- Docker 与 Docker Compose
- Node.js 20.19+ 或 22.12+（管理面板）

## 快速开始

在本目录下运行：

```sh
cp .env.example .env          # 然后将 JWT_SECRET 设为 `openssl rand -hex 32` 的输出
docker compose up -d          # PostgreSQL；应用启动时自动执行迁移
cargo run                     # API 地址 http://127.0.0.1:8080

curl http://127.0.0.1:8080/api/v1/health

cargo run -- create-admin ops admin           # 首个管理员；密码从标准输入读取
cd dashboard && npm install && npm run dev    # http://localhost:5173
```

当 `SIMULATOR_ENABLED=true`（`.env.example` 的默认值）时，管理面板的
**Monitor** 页面可以启动模拟控制器并模拟刷卡。

## 配置

配置来自环境变量（或 `.env`）。主要项：

| 变量 | 用途 |
| ---- | ---- |
| `DATABASE_URL` | **必填。** PostgreSQL 连接串 |
| `JWT_SECRET` | **必填**，≥ 32 字节 |
| `APP_HOST` / `APP_PORT` | 监听地址（默认 `127.0.0.1:8080`） |
| `SIMULATOR_ENABLED` | 启用 `/simulator/*`（仅限开发/演示） |
| `CORS_ALLOWED_ORIGINS` | 允许调用 API 的浏览器来源 |
| `LOG_FORMAT` | `pretty` 或 `json` |
| `METRICS_TOKEN` | 启用 `/metrics`，需携带此 Bearer 令牌 |

完整列表见 [`.env.example`](.env.example) 和 [`src/config.rs`](src/config.rs)。

## 部署

```sh
cp .env.production.example .env.production   # 填写 SITE_ADDRESS 和各项密钥
docker compose -f docker-compose.prod.yml --env-file .env.production up -d --build
```

架构：Caddy（HTTPS）→ 同时提供 API 和管理面板的 distroless 应用容器 →
位于内部网络的 PostgreSQL。

## 测试

数据库测试需要运行 compose 中的 PostgreSQL 并设置 `TEST_DATABASE_URL`
（已在 `.env.example` 中）。每个测试使用独立的临时数据库。

```sh
cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
cd dashboard && npm run lint && npm run typecheck && npm test
```
