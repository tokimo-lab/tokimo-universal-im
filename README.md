# tokimo-universal-im

**通用企业 IM Rust SDK — 统一钉钉、企业微信、飞书 API**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

## 概述

`tokimo-universal-im` 是一个 Rust 编写的统一企业即时通讯库，将钉钉（DingTalk）、企业微信（WeCom）和飞书（Lark/Feishu）的 API 抽象为一套统一的 trait 接口。

**设计目标：做通用客户端**。你只需面对一套 API，即可同时对接多个 IM 平台。未来扩展新平台只需实现 `ImProvider` trait。

## 架构

```
┌──────────────────────────────────────────────────┐
│              你的应用 / 通用客户端                   │
├──────────────────────────────────────────────────┤
│              tokimo-core (trait + types)            │
│  ImProvider / MessagingService / ContactService ... │
├────────────┬────────────────┬────────────────────┤
│ tokimo-    │  tokimo-       │  tokimo-           │
│ dingtalk   │  wecom         │  lark              │
│ (钉钉)     │  (企业微信)     │  (飞书)            │
└────────────┴────────────────┴────────────────────┘
```

### Crate 说明

| Crate | 说明 |
|-------|------|
| `tokimo-core` | 核心 trait 定义 + 统一数据类型，**所有客户端只需依赖此 crate** |
| `tokimo-dingtalk` | 钉钉平台实现 |
| `tokimo-wecom` | 企业微信平台实现 |
| `tokimo-lark` | 飞书/Lark 平台实现 |

## 快速开始

### 依赖配置

```toml
[dependencies]
tokimo-core = { path = "crates/tokimo-core" }

# 按需选择平台
tokimo-dingtalk = { path = "crates/tokimo-dingtalk" }
tokimo-wecom = { path = "crates/tokimo-wecom" }
tokimo-lark = { path = "crates/tokimo-lark" }

tokio = { version = "1", features = ["full"] }
```

### 使用示例

```rust
use tokimo_core::*;
use tokimo_lark::{LarkProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建 provider
    let provider = LarkProvider::feishu("app_id", "app_secret");

    // 2. 获取 token
    let token = provider.auth().get_access_token(&Credentials {
        client_id: "app_id".into(),
        client_secret: "app_secret".into(),
    }).await?;

    // 3. 发送文本消息
    let resp = provider.messaging().unwrap().send_message(SendMessageRequest {
        target: ChatTarget::Group("oc_xxxxx".into()),
        content: MessageContent::Text(TextContent {
            text: "Hello from tokimo!".into(),
            mentions: vec![],
        }),
        bot_id: None,
        idempotency_key: None,
    }).await?;
    println!("Sent: {}", resp.message_id);

    // 4. 获取消息列表
    let messages = provider.messaging().unwrap().list_messages(ListMessagesRequest {
        chat_id: "oc_xxxxx".into(),
        start_time: None,
        end_time: None,
        cursor: None,
        limit: Some(20),
    }).await?;
    for msg in &messages.items {
        println!("[{}] {}: {:?}", msg.id, msg.sender.id, msg.content);
    }

    Ok(())
}
```

### 通用客户端模式（面向 trait 编程）

```rust
use tokimo_core::*;

/// 你的通用客户端只依赖 trait，不依赖具体平台
async fn send_notification(
    provider: &dyn ImProvider,
    chat_id: &str,
    text: &str,
) -> ImResult<()> {
    let messaging = provider.messaging().ok_or_else(|| ImError::NotSupported {
        feature: "messaging".into(),
        platform: provider.platform().to_string(),
    })?;

    messaging.send_message(SendMessageRequest {
        target: ChatTarget::Group(chat_id.into()),
        content: MessageContent::Text(TextContent {
            text: text.into(),
            mentions: vec![],
        }),
        bot_id: None,
        idempotency_key: None,
    }).await?;

    Ok(())
}
```

## 统一消息类型

| MessageContent 枚举 | 说明 | 钉钉 | 企微 | 飞书 |
|---------------------|------|:----:|:----:|:----:|
| `Text` | 纯文本消息 | ✅ 发送 | ✅ 收发 | ✅ 收发 |
| `Markdown` | 富文本 / Markdown | ✅ 发送 | ⚠️ 转为 text 发送 | ✅ 转为 post 发送 |
| `Image` | 图片消息 | ✅ 发送(URL) | ✅ 接收 | ✅ 收发(image_key) |
| `File` | 文件消息 | ❌ | ✅ 接收 | ✅ 收发(file_key) |
| `Audio` | 语音消息 | ❌ | ✅ 接收 | ✅ 收发(file_key) |
| `Video` | 视频消息 | ❌ | ✅ 接收 | ✅ 收发(file_key+cover) |
| `Card` | 交互卡片 | ❌ | ❌ | ✅ 发送(interactive JSON) |
| `Unknown` | 未知/平台特定类型 | — | — | — |

## 功能实现状态

### ✅ 已实现功能

#### 1. 认证 (AuthService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 获取 access_token | ✅ OAuth2 UserAccessToken | ✅ CorpToken | ✅ TenantAccessToken |
| 刷新 token | ✅ refresh_token 机制 | ✅ 重新获取 | ✅ 重新获取 |

#### 2. 消息收发 (MessagingService) — 核心功能

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 发送文本消息（单聊） | ✅ Bot 批量单聊 | ✅ send_message | ✅ open_id 发送 |
| 发送文本消息（群聊） | ✅ Bot 群聊 | ✅ send_message | ✅ chat_id 发送 |
| 发送 Markdown 消息 | ✅ sampleMarkdown | ⚠️ 转为纯文本 | ✅ post 格式 |
| 发送图片消息 | ✅ sampleImageMsg | ❌ 不支持发送 | ✅ image_key |
| 发送文件消息 | ❌ | ❌ 不支持发送 | ✅ file_key |
| 发送音频消息 | ❌ | ❌ 不支持发送 | ✅ file_key |
| 发送视频消息 | ❌ | ❌ 不支持发送 | ✅ file_key + image_key |
| 发送卡片消息 | ❌ | ❌ | ✅ interactive JSON |
| 获取消息列表 | ⚠️ 不支持(需webhook) | ✅ get_message (7天) | ✅ 分页列出 |
| 撤回消息 | ✅ processQueryKey | ❌ 不支持 | ✅ DELETE message_id |

#### 3. 通讯录 (ContactService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 获取当前用户 | ✅ /contact/users/me | ❌ 不支持 | ✅ /authen/v1/user_info |
| 搜索用户 | ✅ keyword 搜索 | ✅ 列出可见联系人 | ✅ query 搜索 |
| 批量获取用户详情 | ✅ userIds 批量 | ❌ 不支持 | ✅ 逐个获取 |

#### 4. 群组管理 (GroupService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建群 | ✅ | ❌ | ✅ |
| 搜索群 | ✅ | ❌ | ✅ |
| 获取群信息 | ✅ | ❌ | ✅ |
| 获取群成员 | ✅ | ❌ | ✅ |
| 添加群成员 | ✅ | ❌ | ✅ |
| 移除群成员 | ✅ | ❌ | ✅ |

#### 5. 日历/日程 (CalendarService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建日程 | ✅ | ✅ | ✅ |
| 列出日程 | ✅ | ✅ | ✅ |
| 获取日程详情 | ✅ | ✅ | ✅ |
| 更新日程 | ✅ | ✅ | ✅ |
| 删除日程 | ✅ | ✅ | ✅ |
| 查询忙闲状态 | ✅ | ✅ | ✅ |

#### 6. 待办任务 (TaskService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建任务 | ✅ | ✅ | ✅ |
| 列出任务 | ✅ | ✅ | ✅ |
| 获取任务详情 | ✅ | ✅ | ✅ |
| 更新任务 | ✅ | ✅ | ✅ |
| 删除任务 | ✅ | ✅ | ✅ |

### 🔲 尚未实现的功能

以下功能在部分平台的 API 中存在，但尚未在统一接口中实现：

| 功能域 | 具体功能 | 可用平台 |
|--------|----------|----------|
| **消息** | Webhook 发送 | 钉钉 |
| **消息** | 消息转发 / 合并转发 | 飞书 |
| **消息** | 消息表情回应 (Reactions) | 飞书 |
| **消息** | 消息已读状态 | 飞书 |
| **消息** | 消息置顶 (Pin) | 飞书 |
| **消息** | 媒体文件下载 | 企微、飞书 |
| **消息** | 图片/文件上传 | 飞书 |
| **消息** | DING 紧急提醒 | 钉钉 |
| **群组** | 群公告 | 飞书 |
| **群组** | 添加/移除机器人 | 钉钉 |
| **通讯录** | 部门搜索/列表 | 钉钉 |
| **通讯录** | 批量获取部门成员 | 钉钉 |
| **日历** | 会议室管理 | 钉钉 |
| **日历** | 参会人管理 | 钉钉 |
| **会议** | 创建/取消/列出会议 | 企微、飞书 |
| **会议** | 会议成员管理 | 企微 |
| **文档** | 创建/读取/编辑文档 | 企微、飞书 |
| **文档** | 智能表格 CRUD | 企微、飞书 |
| **审批** | OA 审批流程 | 钉钉 |
| **考勤** | 打卡记录/班次查询 | 钉钉 |
| **日报** | 日志/日报提交与查询 | 钉钉 |
| **应用** | 工作台应用管理 | 钉钉 |
| **Wiki** | 知识库管理 | 飞书 |
| **邮件** | 邮件收发 | 飞书 |
| **事件** | WebSocket 事件订阅 | 飞书 |
| **事件** | Webhook 回调 | 钉钉、企微 |
| **数据表** | AITable / Base CRUD | 钉钉、飞书 |

## 扩展新平台

实现 `ImProvider` trait 即可接入新平台：

```rust
use tokimo_core::*;

pub struct MyPlatformProvider { /* ... */ }

impl ImProvider for MyPlatformProvider {
    fn platform(&self) -> Platform {
        // 未来可以扩展 Platform 枚举，或使用自定义标识
        Platform::DingTalk // placeholder
    }

    fn auth(&self) -> &dyn AuthService { /* ... */ }
    fn messaging(&self) -> Option<&dyn MessagingService> { Some(/* ... */) }
    fn contact(&self) -> Option<&dyn ContactService> { None } // 不支持则返回 None
    fn group(&self) -> Option<&dyn GroupService> { None }
    fn calendar(&self) -> Option<&dyn CalendarService> { None }
    fn task(&self) -> Option<&dyn TaskService> { None }
}
```

## 核心数据类型一览

### 消息相关

- `Message` — 统一消息结构 (id, chat_id, sender, content, timestamp)
- `MessageContent` — 枚举: Text / Markdown / Image / File / Audio / Video / Card / Unknown
- `SendMessageRequest` — 发送请求 (target, content, bot_id, idempotency_key)
- `SendMessageResponse` — 发送响应 (message_id, extra)
- `ListMessagesRequest` — 列出消息请求 (chat_id, time range, pagination)
- `RecallMessageRequest` — 撤回请求 (message_id, bot_id, chat_id)

### 联系人相关

- `User` — 用户 (id, name, email, phone, avatar, departments)
- `Department` — 部门 (id, name, parent_id)
- `SearchUserRequest` — 搜索请求 (keyword, pagination)

### 群组相关

- `GroupChat` — 群组 (id, name, owner_id, member_count, description)
- `GroupMember` — 群成员 (user_id, name, role)
- `CreateGroupRequest` — 建群请求
- `ModifyMembersRequest` — 增删成员请求
- `SearchGroupRequest` — 搜索群请求

### 日历相关

- `CalendarEvent` — 日程 (id, title, time range, location, attendees)
- `EventAttendee` — 参与者 (user_id, name, status)
- `BusySlot` — 忙闲时段
- `FreeBusyRequest` — 忙闲查询请求

### 任务相关

- `Task` — 任务 (id, title, status, priority, due_time, assignees)
- `TaskStatus` — Pending / InProgress / Done / Deleted
- `TaskPriority` — Low / Normal / High / Urgent

### 通用类型

- `Platform` — DingTalk / WeCom / Lark
- `ChatTarget` — User(id) / Group(id)
- `Page<T>` — 分页包装 (items, has_more, next_cursor)
- `Credentials` — 认证凭证 (client_id, client_secret)
- `AccessToken` — 访问令牌 (token, expires_at, refresh_token)
- `ImError` — 统一错误类型 (Auth / NotFound / RateLimited / Platform / ...)

## 各平台认证方式

### 钉钉 (DingTalk)

```rust
use tokimo_dingtalk::DingTalkProvider;

let provider = DingTalkProvider::new("client_id", "client_secret");
// 需要 OAuth2 device flow 获取用户级 token
// 或使用企业内部应用的 AppKey/AppSecret
```

### 企业微信 (WeCom)

```rust
use tokimo_wecom::WeComProvider;

let provider = WeComProvider::new("corp_id", "corp_secret");
// 使用 corpid + corpsecret 获取 access_token
```

### 飞书 (Lark/Feishu)

```rust
use tokimo_lark::LarkProvider;

// 中国大陆 - Feishu
let provider = LarkProvider::feishu("app_id", "app_secret");

// 国际版 - Lark
let provider = LarkProvider::lark("app_id", "app_secret");
// 使用 app_id + app_secret 获取 tenant_access_token
```

## 技术栈

- **Rust 2021 Edition**
- **tokio** — 异步运行时
- **reqwest** — HTTP 客户端
- **serde / serde_json** — 序列化
- **async-trait** — 异步 trait 支持
- **chrono** — 时间处理
- **thiserror** — 错误类型

## 项目结构

```
tokimo-universal-im/
├── Cargo.toml                    # Workspace 配置
├── README.md                     # 本文档
└── crates/
    ├── tokimo-core/              # 核心 trait + 类型
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs          # ImError 统一错误
    │       ├── types/            # 统一数据类型
    │       │   ├── common.rs     # Platform, Page, ChatTarget, Credentials
    │       │   ├── message.rs    # Message, MessageContent, SendMessageRequest
    │       │   ├── contact.rs    # User, Department, SearchUserRequest
    │       │   ├── group.rs      # GroupChat, GroupMember, CreateGroupRequest
    │       │   ├── calendar.rs   # CalendarEvent, BusySlot, FreeBusyRequest
    │       │   ├── task.rs       # Task, TaskStatus, TaskPriority
    │       │   └── media.rs      # MediaInfo, MediaType
    │       └── traits/           # 抽象 trait 接口
    │           ├── provider.rs   # ImProvider (顶层 trait)
    │           ├── auth.rs       # AuthService
    │           ├── messaging.rs  # MessagingService
    │           ├── contact.rs    # ContactService
    │           ├── group.rs      # GroupService
    │           ├── calendar.rs   # CalendarService
    │           └── task.rs       # TaskService
    ├── tokimo-dingtalk/          # 钉钉实现
    │   └── src/
    │       ├── client.rs         # HTTP 客户端
    │       ├── auth.rs           # OAuth2 token 管理
    │       ├── messaging.rs      # Bot 消息发送/撤回
    │       ├── contact.rs        # 联系人搜索
    │       ├── group.rs          # 群组 CRUD
    │       ├── calendar.rs       # 日历事件
    │       ├── task.rs           # 待办任务
    │       └── provider.rs       # DingTalkProvider
    ├── tokimo-wecom/             # 企业微信实现
    │   └── src/
    │       ├── client.rs         # HTTP 客户端 (access_token in query)
    │       ├── auth.rs           # corpid/corpsecret token
    │       ├── messaging.rs      # 文本消息收发 + 历史记录
    │       ├── contact.rs        # 联系人列表
    │       ├── group.rs          # (有限支持)
    │       ├── calendar.rs       # 日程 CRUD + 忙闲查询
    │       ├── task.rs           # 待办 CRUD
    │       └── provider.rs       # WeComProvider
    └── tokimo-lark/              # 飞书实现
        └── src/
            ├── client.rs         # HTTP 客户端 (Bearer token)
            ├── auth.rs           # Tenant/User access token
            ├── messaging.rs      # 全格式消息收发 + 撤回
            ├── contact.rs        # 用户搜索 + 详情
            ├── group.rs          # 群组 CRUD + 成员管理
            ├── calendar.rs       # 日历事件 + 忙闲
            ├── task.rs           # 任务 CRUD
            └── provider.rs       # LarkProvider
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
