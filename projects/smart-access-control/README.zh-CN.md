# Smart Access Control（智能门禁系统）

[English](README.md) | **简体中文**

一个用 Rust（Axum + Tokio）编写的**纯软件模拟**的物理门禁后端。这是一个学习项目，
灵感来自企业级门禁平台。它不与任何真实硬件通信，也不兼容任何专有门禁协议。

> 状态：**已完成（第 1–8 阶段）**。后端与 React 管理面板
> （[`dashboard/`](dashboard/README.md)）共同构成一个纯软件的门禁模拟系统，
> 并包含安全评审（[SECURITY.md](SECURITY.md)）、可观测性和容器化部署。
>
> **系统如何构建以及为何这样设计：**[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
> （分层、请求流程、数据模型、设计决策，英文）。

## 通行判定是如何进行的

1. `AccessDecisionService` 接收卡号和门 ID。判定时间始终取自服务器时钟，
   从不采用设备提供的时间。
2. `PgAccessDataSource` 在一个只读的 `REPEATABLE READ` 事务中加载卡、持卡人、门、
   所属分组、权限和时间表，因此判定看到的是同一个一致的数据状态。
3. 纯函数 `domain::decide` 按顺序检查：卡存在、卡处于有效状态、未过期、持卡人有效、
   门存在 / 未停用 / 在线、存在适用的权限、且其时间表允许当前本地时间。
   第一个未通过的检查即为拒绝原因。任何缺失的信息都会导致拒绝。
4. 无论允许还是拒绝，这次尝试都会追加写入 `access_events`。如果无法记录，
   服务返回错误，调用方必须将其视为拒绝。数据库触发器会拒绝对该日志的任何
   `UPDATE`、`DELETE` 或 `TRUNCATE`。

拒绝原因：`unknown_card`、`card_revoked`、`card_suspended`、`card_expired`、
`user_suspended`、`user_archived`、`unknown_door`、`door_disabled`、
`door_offline`、`permission_denied`、`outside_schedule`。

## 环境要求

- Rust stable（edition 2024，已在 1.98 上测试）
- Docker 及 Docker Compose（用于 PostgreSQL）
- 管理面板需要 Node.js 20.19+ 或 22.12+（已在 24 上测试）
- 可选：`cargo install cargo-audit`，用于依赖安全审计

## 快速开始

所有命令都在本目录（`projects/smart-access-control/`）下执行。

```sh
# 1. 配置：复制示例文件并修改密码。
cp .env.example .env

# 2. 启动 PostgreSQL。应用依赖它，并会在启动时执行数据库迁移。
docker compose up -d
docker compose ps            # 等待状态变为 "healthy"

# 3. 运行 API。
cargo run

# 4. 检查。
curl http://127.0.0.1:8080/api/v1/health
# {"status":"ok","version":"0.1.0"}
curl http://127.0.0.1:8080/api/v1/health/ready
# {"status":"ready"}
```

仅执行 `cp .env.example .env` 还不够：请把 `JWT_SECRET` 替换为
`openssl rand -hex 32` 的输出。密钥缺失、过短或仍为示例值时，服务器会拒绝启动。

然后创建管理员并启动管理面板：

```sh
# 5. 创建用于登录的账号（密码从标准输入读取）。
cargo run -- create-admin ops admin

# 6. 在第二个终端中启动管理面板。
cd dashboard && npm install && npm run dev   # http://localhost:5173
```

当 `SIMULATOR_ENABLED=true`（`.env.example` 中的默认值）时，管理面板的
**Monitor**（实时监控）页面可以启动模拟控制器并模拟刷卡，无需其他工具即可
实时观看判定结果。

## 管理员与登录

系统没有默认账号。使用 `create-admin` 命令创建第一个管理员；它从标准输入读取
密码（至少 12 个字符）：

```sh
cargo run -- create-admin alice admin     # 角色：admin（读写）或 viewer（只读）
```

在提示符下输入密码时，密码会显示在屏幕上；若要避免，可以通过管道传入，
例如来自密码管理器的命令行工具。

```sh
# 登录：返回 15 分钟有效的访问令牌和 7 天有效的刷新令牌。
curl -s http://127.0.0.1:8080/api/v1/auth/login \
  -H 'content-type: application/json' \
  -d '{"username":"alice","password":"<password>"}'

# 使用访问令牌调用受保护的接口。
curl -s http://127.0.0.1:8080/api/v1/auth/me -H "authorization: Bearer <access_token>"
```

会话机制：

- 密码以 Argon2id 哈希形式存储。哈希计算在 Tokio 的阻塞线程池中运行，
  不会拖慢其他请求。
- 每次登录失败都返回相同的 `401 invalid credentials`，并且未知用户名与错误密码
  消耗相同的哈希时间，因此无论是响应内容还是响应耗时都无法暴露哪些账号存在。
- 访问令牌是 HS256 JWT（15 分钟），不做存储，因此被停用的管理员在当前令牌过期前
  仍可访问。
- 刷新令牌是随机值，只以 SHA-256 哈希形式存储，并且只能使用一次：
  `POST /auth/refresh` 会返回一对新令牌。再次提交已被轮换的刷新令牌会被视为
  令牌被盗，并吊销该管理员的所有会话。
- 使用 `"session": "cookie"` 登录（管理面板就是这样做的）时，刷新令牌只通过
  作用于 `/api/v1/auth` 的 `HttpOnly; Secure; SameSite=Strict` Cookie 下发；
  之后的刷新和登出都读取这个 Cookie。不使用该选项时，令牌在响应体中返回，
  供 API 客户端使用。
- 登录失败会被限流：15 分钟内同一账号名失败 5 次、或同一客户端地址失败 20 次后，
  返回 `429` 并附带 `Retry-After`。
- 登录、登录失败、登出、检测到的令牌重用以及账号创建，都会记录在只追加的
  `admin_audit_log` 表中。

尚未实现：多因素认证，以及通过 API 停用或删除管理员。安全评审、现有防护措施和
已知风险见 [SECURITY.md](SECURITY.md)。

使用 `Ctrl+C`（或 SIGTERM）停止服务器，它会先处理完正在进行的请求再退出。
使用 `docker compose down` 停止数据库，或使用 `docker compose down -v`
同时删除其数据卷。

## 配置

配置来自环境变量。如果存在 `.env` 文件会自动加载，真实的环境变量优先于它。

| 变量 | 默认值 | 用途 |
| ---- | ------ | ---- |
| `APP_HOST`          | `127.0.0.1`                                    | API 监听地址 |
| `APP_PORT`          | `8080`                                         | API 端口 |
| `RUST_LOG`          | `smart_access_control=debug,tower_http=debug`  | 日志过滤 |
| `LOG_FORMAT`        | `pretty`                                       | 日志格式：`pretty` 或 `json` |
| `METRICS_TOKEN`     | 未设置（无 `/metrics`）                        | `/metrics` 的 Bearer 令牌，≥ 16 字节（从不记录到日志） |
| `POSTGRES_USER`     | compose 必填                                   | 数据库用户 |
| `POSTGRES_PASSWORD` | compose 必填                                   | 数据库密码 |
| `POSTGRES_DB`       | compose 必填                                   | 数据库名 |
| `POSTGRES_PORT`     | `55432`                                        | compose 数据库在宿主机上的端口 |
| `DATABASE_URL`      | **必填**                                       | 应用的数据库连接（从不记录到日志） |
| `DATABASE_MAX_CONNECTIONS` | `10`                                    | 连接池大小 |
| `JWT_SECRET`        | **必填**，≥ 32 字节                            | 签发访问令牌的密钥（从不记录到日志） |
| `ACCESS_TOKEN_TTL_SECONDS` | `900`                                   | 访问令牌有效期 |
| `REFRESH_TOKEN_TTL_SECONDS` | `604800`                               | 刷新令牌有效期 |
| `CORS_ALLOWED_ORIGINS` | 空                                          | 允许的浏览器来源（CORS） |
| `CONTROLLER_TIMEOUT_SECONDS` | `30`                                  | 控制器静默多久后被标记为离线 |
| `SIMULATOR_ENABLED` | `false`                                        | 挂载 `/simulator/*`（仅限开发/演示） |
| `WS_MAX_CONNECTIONS` | `256`                                         | 实时事件 WebSocket 的最大并发连接数 |
| `COOKIE_SECURE`     | `true`                                         | 刷新令牌 Cookie 的 `Secure` 标志 |
| `TRUSTED_PROXIES`   | 空                                             | 可信代理的 IP/CIDR，只信任它们的 `X-Forwarded-For` |
| `STATIC_DIR`        | 未设置                                         | 从该目录在 `/` 提供构建好的管理面板 |
| `TEST_DATABASE_URL` | 无（仅测试使用）                               | 用于创建测试数据库的本地服务器 |

compose 数据库默认监听 `127.0.0.1:55432`，因此不会与本机 5432 端口上的
PostgreSQL 冲突。如果配置无效，应用会记录错误并以非零状态码退出。

## API

基础路径为 `/api/v1`。所有请求/响应体均为 JSON；时间戳为 RFC 3339（UTC）。

**访问规则。** 健康检查和 `auth/login|refresh|logout` 是公开的。其他所有路由都需要
`Authorization: Bearer <access token>`。两种角色（`admin`、`viewer`）都可以读取；
所有修改数据的路由都需要 `admin`（viewer 会得到 `403`）。

| 方法 | 路径 | 说明 |
| ---- | ---- | ---- |
| GET    | `/health` | 存活检查：`{status, version}` |
| GET    | `/health/ready` | 就绪检查：数据库可用返回 200，否则 503 |
| POST   | `/auth/login` | `{username, password}` → 访问令牌 + 刷新令牌 |
| POST   | `/auth/refresh` | `{refresh_token}` → 新的一对令牌（旧令牌随即失效） |
| POST   | `/auth/logout` | `{refresh_token}` → 204 |
| GET    | `/auth/me` | 令牌所属管理员的 `{id, role}` |
| GET    | `/users` | 列表（分页） |
| POST   | `/users` | `{name, email}` → 201 |
| GET    | `/users/{id}` | 单个用户 |
| PATCH  | `/users/{id}` | `{name, email, status}` 中任意字段；status 为 `active`/`suspended`/`archived` |
| DELETE | `/users/{id}` | 归档（软删除）→ 204 |
| GET    | `/cards` | 列表；`?user_id=` 查询某个用户的卡 |
| POST   | `/cards` | `{user_id, card_number, expires_at?}` → 201 |
| GET    | `/cards/{id}` | 单张卡，包含 `effective_status`（例如 `expired`） |
| PATCH  | `/cards/{id}` | `{status: "active" \| "suspended"}` |
| POST   | `/cards/{id}/revoke` | 永久吊销 |
| GET    | `/doors` | 列表 |
| POST   | `/doors` | `{name, location, controller_id}` → 201（初始为 `offline`） |
| GET    | `/doors/{id}` | 单个门 |
| PATCH  | `/doors/{id}` | `{name, location, controller_id}` 中任意字段 |
| PATCH  | `/doors/{id}/status` | `{status: "online" \| "offline" \| "disabled"}`（设为 `offline` 可重新启用已停用的门） |
| GET    | `/access-groups` | 列表 |
| POST   | `/access-groups` | `{name, description?}` → 201 |
| GET    | `/access-groups/{id}` | 单个分组 |
| PATCH  | `/access-groups/{id}` | `{name?, description?}`；`"description": null` 表示清空 |
| DELETE | `/access-groups/{id}` | 同时删除其成员关系和权限 |
| GET    | `/access-groups/{id}/members` | `{user_ids: [...]}` |
| POST   | `/access-groups/{id}/members` | `{user_id}` → 204 |
| DELETE | `/access-groups/{id}/members/{user_id}` | → 204 |
| GET    | `/schedules` | 列表 |
| POST   | `/schedules` | `{name, timezone, rules: [{days, start, end}], effective_from?, effective_until?}` → 201 |
| GET    | `/schedules/{id}` | 单个时间表 |
| GET    | `/permissions` | 列表 |
| POST   | `/permissions` | `{group_id, door_id, schedule_id?}` → 201（无时间表 = 任何时间） |
| DELETE | `/permissions/{id}` | → 204 |
| POST   | `/access/decisions` | `{card_number, door_id}` → 已记录的事件（允许和拒绝都返回 200） |
| GET    | `/events` | 通行事件，最新的在前；过滤条件见下文 |
| GET    | `/events/stream` | WebSocket：实时通行事件（见下文） |
| GET    | `/events/recent` | 最新事件，`?limit=`（默认 20） |
| GET    | `/events/{id}` | 单个事件 |
| GET    | `/audit-log` | 管理操作记录，最新的在前（分页） |
| GET    | `/controllers` | 已注册的控制器及其状态和最后通信时间 |
| POST   | `/controllers` | `{controller_id}` → 201，返回控制器的密钥 `key`（仅显示一次） |
| GET    | `/controllers/{id}` | 单个控制器 |
| POST   | `/controllers/{id}/rotate-key` | 生成新密钥；旧密钥随即失效 |
| POST   | `/device/heartbeat` | 需要控制器密钥 → `{controller_id, doors_online}` |
| POST   | `/device/access-requests` | 控制器密钥 → `{card_number, door_id}` → 已记录的事件 |
| POST   | `/device/disconnect` | 控制器密钥 → 204；其所有门变为离线 |

时间表规则示例：`{"days": ["mon","tue","wed","thu","fri"], "start":
"09:00", "end": "17:00"}`，时间按时间表的 IANA `timezone` 计算；`end <= start`
表示时间窗口在次日结束。

**事件过滤条件**（`GET /events` 和 WebSocket 订阅）：`door_id`、`user_id`、
`card_id`、`decision`（`granted`/`denied`）、`reason`（例如 `door_offline`）、
`from`（RFC 3339，包含）、`until`（不包含）。设置的多个条件以 AND 组合。

**列表**接口接受 `?limit=`（默认 50，最大 100）和 `?offset=`，返回
`{"items": [...], "limit": n, "offset": m}`。

**错误**格式始终为 `{"error": {"code": "...", "message": "..."}}`：
`400 invalid_request/invalid_path/invalid_query`、`401 unauthorized`、
`403 forbidden`、`404 not_found`、`409 conflict/invalid_state`、
`413 payload_too_large`、`415 unsupported_media_type`、
`422 validation_failed`（未知或类型错误的 JSON 字段也返回此错误）、
`500 internal_error`（详细信息只记录在日志中，从不返回）。请求体上限为 64 KiB。

**审计。** 通过 API 做出的每一项修改都会连同执行操作的管理员一起记录到
`admin_audit_log`。审计记录在修改成功之后写入，而不是在同一个事务中：
如果写入失败，修改依然生效，并以错误级别记录该失败。

**CORS。** 设置 `CORS_ALLOWED_ORIGINS`（逗号分隔，例如 `http://localhost:5173`）
可以让其他来源的浏览器管理面板调用 API。为空（默认）时只允许同源请求。

## 门控制器（模拟）

这里的控制器是**软件模拟**。本项目中没有任何部分与真实门禁硬件或任何厂商协议通信，
也不与之兼容。

1. 管理员注册控制器：`POST /controllers {"controller_id": "ctrl-001"}`。
   响应中包含密钥 `key`，只显示一次（只存储其 SHA-256 哈希）。
2. 门通过 `controller_id`（创建门时设置）与控制器关联。
3. 控制器调用 `/device/*` 接口时携带请求头 `X-Controller-Key: <key>`。
   该凭证与管理员令牌相互独立：两者不能互相替代。
4. 每次设备调用都算作一次心跳，会使控制器及其已启用的门变为在线。
   静默超过 `CONTROLLER_TIMEOUT_SECONDS`（默认 30 秒）的控制器会被后台检查
   连同其门一起标记为离线；该检查在每个超时周期内运行三次。
   `POST /device/disconnect` 会立即产生同样的效果。
5. `POST /device/access-requests` 只接受控制器自己的门；其他门会得到 `403`，
   且不会记录任何内容。控制器必须把任何错误都视为"拒绝"。

`POST /access/decisions`（管理员令牌）仍可作为绕过控制器的手动测试工具使用。

### 进程内模拟器

当 `SIMULATOR_ENABLED=true` 时，服务器可以自己运行模拟控制器（每个都是一个
Tokio 任务），并通过仅限管理员的接口进行控制。它们使用与设备相同的协议：
控制器密钥、每 `CONTROLLER_TIMEOUT_SECONDS` 的三分之一发送一次心跳，以及相同的
通行检查。

| 方法 | 路径 | 说明 |
| ---- | ---- | ---- |
| POST   | `/simulator/controllers` | `{controller_id}` → 启动（未注册则注册，已注册则轮换其密钥） |
| GET    | `/simulator/controllers` | 运行中的模拟：网络通/断、心跳次数、最后一次错误 |
| GET    | `/simulator/controllers/{id}` | 单个模拟 |
| POST   | `/simulator/controllers/{id}/swipe` | `{card_number, door_id}` → 判定结果（以事件形式返回） |
| POST   | `/simulator/controllers/{id}/outage` | 网络静默中断；后端通过超时发现 |
| POST   | `/simulator/controllers/{id}/disconnect` | 正常断开；门立即变为离线 |
| POST   | `/simulator/controllers/{id}/reconnect` | 网络恢复；立即发送心跳 |
| DELETE | `/simulator/controllers/{id}` | 停止（正常断开） |

网络中断期间刷卡返回 `503 controller_offline`：模拟读卡器在本地拒绝，
请求不会到达后端，也不会写入事件日志。服务器关闭时，所有模拟控制器会先正常断开。
模拟器被禁用时（默认），这些路由不存在（404）。

快速演示（以管理员身份登录后，`$T` 为访问令牌）：

```sh
api() { curl -s -X "$1" "http://127.0.0.1:8080/api/v1$2" -H "authorization: Bearer $T" \
          -H 'content-type: application/json' ${3:+-d "$3"}; }
DOOR=$(api POST /doors '{"name":"Lab","location":"B1","controller_id":"sim-1"}' | jq -r .id)
api POST /simulator/controllers '{"controller_id":"sim-1"}'    # 门变为在线
api POST /simulator/controllers/sim-1/swipe "{\"card_number\":\"CARD-1\",\"door_id\":\"$DOOR\"}"
```

## 实时事件（WebSocket）

连接到 `ws://<host>/api/v1/events/stream`，并在 5 秒内发送：

```json
{"type": "subscribe", "token": "<access token>", "filter": {"door_id": "...", "decision": "denied"}}
```

`filter` 是可选的。服务器回复 `{"type": "subscribed", "expires_at": "..."}`，
之后在发生通行判定时推送 `{"type": "access_event", "event": {...}}`。两种角色都可以
订阅。令牌放在第一条消息中而不是 URL 中，因为浏览器无法为 WebSocket 设置请求头，
而 URL 会出现在日志里。

关闭码：`4400` 消息或过滤条件格式错误，`4401` 令牌无效，`4408` 5 秒内未收到订阅消息，
`4409` 访问令牌已过期（使用新令牌重新连接），`1001` 服务器正在关闭。

**投递保证。** 实时投递是尽力而为的：

- 事件只有在存储之后才会发布，因此管理面板显示的所有内容都已在数据库中。
- 发布从不等待订阅者；缓慢的管理面板不会拖慢通行判定。落后超过 1024 个事件的
  订阅者会丢失最旧的事件，并收到 `{"type": "lagged", "missed": n}`。
  10 秒内无法接收消息的客户端会被断开。
- 事件按顺序投递，最多一次，且只投递给当时已连接的客户端。重新连接时不会重放：
  请使用 `GET /events?from=<最后看到的 occurred_at>` 获取错过的事件。
- 该通道位于进程内。运行多个服务器实例时，每个客户端只能看到由其所连接实例
  判定的事件。

## 可观测性

**请求 ID。** 每个响应都带有 `X-Request-Id` 响应头，处理该请求期间写入的每一行日志
都带有相同的 `request_id`。传入的 `X-Request-Id`（例如来自反向代理）如果由 1–64 个
`A–Z a–z 0–9 - _ . :` 字符组成则会被保留；否则替换为新的 UUID，因此客户端提供的
文本无法伪造或破坏日志行。

**日志。** `LOG_FORMAT=json` 每行输出一个 JSON 对象，便于日志收集系统（Loki、
Elasticsearch、CloudWatch 等）处理；默认的 `pretty` 适合在终端中阅读。
`RUST_LOG` 设置日志级别。密钥和卡号从不写入日志。

```bash
LOG_FORMAT=json cargo run | jq 'select(.span.request_id == "…")'
```

**指标。** 设置 `METRICS_TOKEN` 后，`GET /metrics`（位于 `/api/v1` 之外）会向发送
`Authorization: Bearer <token>` 的调用方提供 Prometheus 文本格式的指标；
未设置时该路由不存在。

| 指标 | 类型 | 标签 |
| ---- | ---- | ---- |
| `http_requests_total` | counter | `method`、`path`（路由模式，例如 `/api/v1/users/{id}`）、`status` |
| `http_request_duration_seconds` | histogram | 同上 |
| `access_decisions_total` | counter | `decision`（`granted`/`denied`）、`reason` |
| `auth_login_attempts_total` | counter | `outcome`（`success`/`failure`/`throttled`） |
| `auth_refresh_token_reuse_total` | counter | —（刷新令牌被重放：可能被盗） |
| `websocket_connections` | gauge | — |
| `controllers_expired_total` | counter | —（被存活监控标记为离线的控制器） |

路径按路由模式打标签，而不是原始 URL，因此 ID 不会产生无限增长的时间序列。
Prometheus 任务示例：

```yaml
scrape_configs:
  - job_name: smart-access-control
    authorization: { credentials: "<METRICS_TOKEN>" }
    static_configs: [{ targets: ["127.0.0.1:8080"] }]
```

## 部署

使用 Docker Compose 运行一个接近生产环境的部署：

```text
internet ──► proxy (Caddy) ──────────► app (API + 管理面板) ──► db (PostgreSQL)
             HTTPS、HSTS、gzip/zstd     单个容器，               不对外发布端口，
             屏蔽 /metrics              非 root、只读文件系统     仅在内部网络中
```

```bash
cp .env.production.example .env.production   # 填写 SITE_ADDRESS 和各项密钥
docker compose -f docker-compose.prod.yml --env-file .env.production up -d --build

# 第一个管理员（密码从标准输入读取）：
docker compose -f docker-compose.prod.yml --env-file .env.production \
  exec -T app smart-access-control create-admin ops admin < password.txt
```

如果 `SITE_ADDRESS` 是公网域名且 80/443 端口可以访问到主机，Caddy 会自动获取
Let's Encrypt 证书。若要在本机试用，请使用 `SITE_ADDRESS=localhost`、
`HSTS_MAX_AGE=0`，如果 80/443 已被占用，再加上 `HTTP_PORT=8088 HTTPS_PORT=8443`；
此时 Caddy 使用它自己的本地 CA（用 `curl -k`，或在浏览器中接受警告）。

各组成部分的作用：

- **镜像**（[`Dockerfile`](Dockerfile)）：三个构建阶段。Node 构建管理面板，Rust 构建
  release 二进制文件（依赖下载和构建产物会在多次构建之间缓存），最终结果复制到
  distroless `cc-debian13` 中：只有 glibc 和 CA 证书，没有 shell，以 uid 65532 运行，
  约 66 MB。镜像的健康检查运行 `smart-access-control healthcheck`，它会请求
  `/api/v1/health/ready`（镜像中没有 curl）。
- **管理面板**：`STATIC_DIR` 让 API 直接提供管理面板，因此管理面板和 API 同源
  （无需 CORS，刷新令牌 Cookie 为第一方 Cookie）。`/assets` 下带哈希的文件缓存一年；
  `index.html` 每次加载都会重新验证；未知路径返回 `index.html` 以支持客户端路由，
  但 `/api/` 下的路径除外，它们返回 JSON 格式的 404。页面有自己的
  Content-Security-Policy（`default-src 'self'`，禁止内联脚本，禁止被嵌入框架）；
  API 响应保持 `default-src 'none'`。
- **客户端地址**：在代理之后，所有连接都来自代理。应用只信任来自
  `TRUSTED_PROXIES`（代理的固定地址）的 `X-Forwarded-For`，从右向左读取，
  遇到第一个不是可信代理的地址即停止，因此客户端无法通过自己发送该请求头来伪造地址。
  这样，按地址的登录限流就会作用于每一个真实客户端。
- **网络**：数据库只与应用位于一个 `internal` 网络中；代理无法访问数据库，
  应用也无法访问外部网络。
- **指标**：Prometheus 在 Docker 网络内部使用 `METRICS_TOKEN` 抓取
  `http://app:8080/metrics`；代理对 `/metrics` 返回 404。
- **数据库迁移**在应用启动时运行。如果有多个副本，请改为单独的发布步骤执行。

未包含：`db-data` 数据卷的备份、日志转发，以及 Prometheus 服务器。与 PostgreSQL
的连接未加密，这只因为连接从不离开内部 Docker 网络才可以接受。

## 项目结构

```text
src/
├── main.rs              # 入口：`serve`（默认）、`create-admin`、`healthcheck`
├── lib.rs               # `run()`：绑定监听、提供服务、优雅关闭
├── config.rs            # 类型化、经过校验的环境配置
├── domain/              # 实体、时间表、判定引擎（`access.rs`）、事件
├── application/         # 用例服务（包括通行判定）、端口（trait）、错误类型
├── infrastructure/
│   ├── postgres/        # 连接池、迁移、Pg*Repository、PgAccessDataSource
│   ├── auth/            # Argon2id 哈希、JWT 签发
│   ├── event_hub.rs     # 进程内实时事件广播
│   ├── metrics.rs       # Prometheus 记录器和指标说明
│   └── ...              # SystemClock、日志（pretty/JSON）、关闭信号
├── interfaces/http/     # 路由、状态、提取器，每种资源一个模块，
│                        # observability.rs（请求 ID、HTTP 指标、/metrics），
│                        # client_ip.rs（X-Forwarded-For）、dashboard.rs（静态文件）
├── interfaces/background.rs # 存活监控（由时间驱动的入口）
└── simulator/           # 模拟控制器（位于分层之外：扮演设备）
migrations/              # 带版本的 SQL 迁移（启动时执行）
deploy/Caddyfile         # 反向代理：HTTPS、HSTS、压缩
Dockerfile               # 生产镜像（管理面板 + API，distroless）
docker-compose.yml       # 开发用数据库
docker-compose.prod.yml  # 接近生产的部署：代理、应用、数据库
tests/
├── common/mod.rs        # 测试数据库工具
├── health.rs            # 健康检查接口测试（进程内路由）
├── postgres_*.rs        # 基于真实 PostgreSQL 的仓储测试
├── use_cases.rs         # 连接 PostgreSQL 和系统时钟的服务测试
├── access_decisions.rs  # 端到端：配置策略 -> 判定 -> 审计日志
├── postgres_auth.rs     # 管理员、刷新令牌（含并发竞争）、审计日志
├── http_auth.rs         # 通过 HTTP 测试登录/刷新/登出/角色
├── http_management.rs   # 管理 API：通行规则、工作流、错误
├── websocket.rs         # 基于真实 socket 的实时事件流
├── postgres_controllers.rs # 控制器存储、过期边界
├── http_device.rs       # 设备 API、凭证、断开/重连
├── http_simulator.rs    # 模拟器：刷卡、故障检测、重连
├── observability.rs     # 请求 ID、/metrics 保护和标签
└── deployment.rs        # 管理面板静态服务、代理后的客户端地址
```

依赖方向向内：`interfaces → application → domain`，`infrastructure` 实现
`application` 中定义的 trait。

## 测试

`cargo test` 运行单元测试和集成测试。仓储测试和就绪检查测试需要运行 compose 中的
PostgreSQL（`docker compose up -d`）并设置 `TEST_DATABASE_URL`（`.env.example`
中已包含）。

每个数据库测试都会创建自己的临时数据库 `sac_test_<uuid>`，执行迁移，并在测试通过后
删除该数据库，因此测试可以并行运行，也永远不会触碰应用数据库。作为安全保护，
测试工具只通过 `TEST_DATABASE_URL` 连接，并拒绝 `localhost`、`127.0.0.1` 或 `::1`
以外的任何主机。

失败的测试会保留其数据库以便检查。清理残留数据库：

```sh
docker compose exec -T postgres psql -U sac -d postgres -Atc \
  "SELECT 'DROP DATABASE \"' || datname || '\";' FROM pg_database WHERE datname LIKE 'sac_test_%'" \
  | docker compose exec -T postgres psql -U sac -d postgres
```

## 质量检查

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit

cd dashboard
npm run lint && npm run typecheck && npm test && npm run build
npm audit
```

## 故障排查

| 现象 | 原因与解决方法 |
| ---- | -------------- |
| `JWT_SECRET must be at least 32 bytes` / `is still the example placeholder` | 把 `openssl rand -hex 32` 的输出写入 `.env`。 |
| `docker compose up` 失败：port is already allocated | 另一个 PostgreSQL 占用了该端口。在 `.env` 中设置 `POSTGRES_PORT`，并在 `DATABASE_URL` 和 `TEST_DATABASE_URL` 中使用同一端口。 |
| macOS：`cargo build` 失败，提示 "You have not agreed to the Xcode license agreements" | 运行 `sudo xcodebuild -license accept`，或改用 Command Line Tools 构建：`DEVELOPER_DIR=/Library/Developer/CommandLineTools cargo build`。 |
| 数据库测试失败，提示 "TEST_DATABASE_URL must be set" 或主机错误 | 将 `TEST_DATABASE_URL` 设置为**本地**服务器（见 `.env.example`）；测试工具有意拒绝其他主机。 |
| 生产部署：`Pool overlaps with other one on this address space` | 另一个 Docker 网络正在使用 `10.83.83.0/24`。在 `.env.production` 中设置 `PROXY_SUBNET` 和 `PROXY_IP`。 |
| 登录返回 `429` | 登录限流：15 分钟内同一账号名失败 5 次或同一客户端地址失败 20 次。等待 `Retry-After` 指定的时间，或重启服务器（限流数据保存在内存中）。 |
| Monitor 页面显示 "Reconnecting…" | 后端未运行或已重启；事件流会自动按退避策略重新连接。 |
