# tokimo-universal-im

**通用企业 IM Rust SDK — 统一钉钉、企业微信、飞书 API**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

## 概述

`tokimo-universal-im` 是一个 Rust 编写的统一企业即时通讯库，将钉钉（DingTalk）、企业微信（WeCom）和飞书（Lark/Feishu）的 API 抽象为一套统一的 trait 接口。

**设计目标：做通用客户端**。你只需面对一套 API，即可同时对接多个 IM 平台。未来扩展新平台只需实现 `ImProvider` trait。

## 架构

```
┌────────────────────────────────────────────────────────┐
│                你的应用 / 通用客户端                       │
├────────────────────────────────────────────────────────┤
│                 tokimo-core (traits + types)              │
│    21 个服务 trait + 统一数据类型 + 统一错误处理              │
├──────────────┬──────────────────┬──────────────────────┤
│  tokimo-     │   tokimo-        │   tokimo-            │
│  dingtalk    │   wecom          │   lark               │
│  (钉钉)      │   (企业微信)       │   (飞书/Lark)         │
│  14 服务      │   14 服务         │   19 服务             │
└──────────────┴──────────────────┴──────────────────────┘
```

### Crate 说明

| Crate | 说明 |
|-------|------|
| `tokimo-core` | 核心 trait 定义 + 统一数据类型，**所有客户端只需依赖此 crate** |
| `tokimo-dingtalk` | 钉钉平台实现 (14 服务: auth + messaging + contact + group + calendar + task + webhook + event + department + meeting_room + approval + attendance + report + data_table) |
| `tokimo-wecom` | 企业微信平台实现 (14 服务: auth + messaging + contact + group + calendar + task + meeting + chat_list + media + document + webhook + event + department + data_table) |
| `tokimo-lark` | 飞书/Lark 平台实现 (19 服务，功能最完整) |

---

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

### 基本用法 — 发送消息

```rust
use tokimo_core::*;
use tokimo_lark::LarkProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = LarkProvider::feishu("app_id", "app_secret");

    // 获取 token
    provider.auth().get_access_token(&Credentials {
        client_id: "app_id".into(),
        client_secret: "app_secret".into(),
    }).await?;

    // 发送文本消息
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
    Ok(())
}
```

### 拉取历史消息

```rust
let page = provider.messaging().unwrap().list_messages(ListMessagesRequest {
    chat_id: "oc_xxxxx".into(),
    chat_type: Some(ChatTypeHint::Group),
    start_time: None, end_time: None,
    cursor: None, limit: Some(50),
}).await?;

for msg in &page.items {
    println!("[{}] {}: {:?}", msg.id, msg.sender.name, msg.content);
}
```

### Webhook 发送消息

```rust
// 通过 Webhook 发送消息（钉钉/企微自定义机器人）
provider.webhook().unwrap().send_webhook(WebhookMessageRequest {
    webhook_url: "https://oapi.dingtalk.com/robot/send?access_token=xxx".into(),
    content: MessageContent::Text(TextContent {
        text: "服务告警：CPU 使用率超过 90%".into(),
        mentions: vec![],
    }),
    secret: Some("SEC_xxx".into()), // 可选签名
    at_user_ids: vec![],
    at_all: true,
}).await?;
```

### 消息回复、转发、表情、置顶（飞书）

```rust
let ext = provider.message_ext().unwrap();

// 回复消息
ext.reply_message(ReplyMessageRequest {
    reply_to_message_id: "om_xxx".into(),
    content: MessageContent::Text(TextContent { text: "收到！".into(), mentions: vec![] }),
}).await?;

// 置顶消息
ext.pin_message("om_xxx").await?;

// 查看已读状态
let status = ext.get_read_status("om_xxx").await?;
println!("已读: {}/{}", status.read_count, status.total_count);
```

### 部门与组织架构

```rust
let dept = provider.department().unwrap();

// 列出根部门
let depts = dept.list_departments(ListDepartmentsRequest {
    parent_id: None, cursor: None, limit: Some(50),
}).await?;

// 获取部门成员
let members = dept.list_department_members(ListDepartmentMembersRequest {
    department_id: "dept_001".into(),
    cursor: None, limit: Some(100),
}).await?;
```

### 审批流程

```rust
let approval_svc = provider.approval().unwrap();

// 发起审批
let instance = approval_svc.create_approval(CreateApprovalRequest {
    process_code: "PROC-xxx".into(),
    initiator_id: "user_001".into(),
    form_data: serde_json::json!({"reason": "出差申请", "days": 3}),
    approvers: vec!["manager_001".into()],
    cc_users: vec![],
}).await?;

// 审批通过
approval_svc.action_approval(ApprovalActionRequest {
    instance_id: instance.id.clone(),
    action: ApprovalAction::Approve,
    comment: Some("同意".into()),
}).await?;
```

### 数据表 (AITable / Bitable / Smartsheet)

```rust
let dt = provider.data_table().unwrap();

// 查询记录
let records = dt.list_records(ListRecordsRequest {
    base_id: "base_xxx".into(),
    table_id: "tbl_xxx".into(),
    view_id: None, filter: None, sort: None,
    field_names: vec![], cursor: None, limit: Some(100),
}).await?;

// 写入记录
dt.write_records(WriteRecordsRequest {
    base_id: "base_xxx".into(),
    table_id: "tbl_xxx".into(),
    records: vec![DataRecordWrite {
        id: None, // None = 新建, Some = 更新
        fields: serde_json::json!({"名称": "新任务", "状态": "进行中"}),
    }],
}).await?;
```

### 会议管理

```rust
let meeting_svc = provider.meeting().unwrap();
let meeting = meeting_svc.create_meeting(CreateMeetingRequest {
    title: "周会".into(),
    start_time: chrono::Utc::now(),
    end_time: chrono::Utc::now() + chrono::Duration::hours(1),
    attendees: vec!["user_1".into(), "user_2".into()],
    settings: None, description: Some("每周例会".into()),
}).await?;
```

### Wiki 知识库（飞书）

```rust
let wiki = provider.wiki().unwrap();

let spaces = wiki.list_spaces(ListWikiSpacesRequest {
    cursor: None, limit: Some(20),
}).await?;

let nodes = wiki.list_nodes(ListWikiNodesRequest {
    space_id: "space_xxx".into(),
    parent_node_id: None, cursor: None, limit: Some(50),
}).await?;

wiki.create_node(CreateWikiNodeRequest {
    space_id: "space_xxx".into(),
    parent_node_id: None,
    node_type: "doc".into(),
    title: "技术方案".into(),
    content: Some("# 方案概述\n...".into()),
}).await?;
```

### 邮件（飞书）

```rust
let email_svc = provider.email().unwrap();

email_svc.send_email(SendEmailRequest {
    subject: "项目进展".into(),
    to: vec![EmailAddress { address: "user@example.com".into(), name: Some("张三".into()) }],
    cc: vec![], bcc: vec![],
    body: EmailBody { content_type: "text/html".into(), content: "<h1>本周进展</h1>...".into() },
}).await?;
```

### 通用客户端模式（面向 trait 编程）

```rust
use tokimo_core::*;

/// 你的通用客户端只依赖 trait，不依赖具体平台
async fn send_notification(provider: &dyn ImProvider, chat_id: &str, text: &str) -> ImResult<()> {
    let messaging = provider.messaging().ok_or_else(|| ImError::NotSupported {
        feature: "messaging".into(),
        platform: provider.platform().to_string(),
    })?;
    messaging.send_message(SendMessageRequest {
        target: ChatTarget::Group(chat_id.into()),
        content: MessageContent::Text(TextContent { text: text.into(), mentions: vec![] }),
        bot_id: None, idempotency_key: None,
    }).await?;
    Ok(())
}

/// 优雅降级：优先回复，不支持则普通发送
async fn try_reply(provider: &dyn ImProvider, msg_id: &str, chat_id: &str, text: &str) -> ImResult<()> {
    let content = MessageContent::Text(TextContent { text: text.into(), mentions: vec![] });
    if let Some(ext) = provider.message_ext() {
        ext.reply_message(ReplyMessageRequest {
            reply_to_message_id: msg_id.into(), content,
        }).await?;
    } else if let Some(m) = provider.messaging() {
        m.send_message(SendMessageRequest {
            target: ChatTarget::Group(chat_id.into()), content,
            bot_id: None, idempotency_key: None,
        }).await?;
    }
    Ok(())
}
```

---

## 功能实现状态 — 完整矩阵 (21 服务)

### 服务总览

| # | 服务 (Trait) | 说明 | 钉钉 | 企微 | 飞书 |
|:-:|-------------|------|:----:|:----:|:----:|
| 1 | `AuthService` | 认证 | ✅ | ✅ | ✅ |
| 2 | `MessagingService` | 消息收发 | ✅ | ✅ | ✅ |
| 3 | `MessageExtService` | 回复/转发/表情/已读/置顶 | ❌ | ❌ | ✅ |
| 4 | `ContactService` | 通讯录 | ✅ | ✅ | ✅ |
| 5 | `GroupService` | 群组管理+公告+Bot | ✅ | ⚠️ | ✅ |
| 6 | `ChatListService` | 会话列表 | ❌ | ✅ | ✅ |
| 7 | `CalendarService` | 日历/日程 | ✅ | ✅ | ✅ |
| 8 | `TaskService` | 待办任务 | ✅ | ✅ | ✅ |
| 9 | `MeetingService` | 会议管理 | ❌ | ✅ | ✅ |
| 10 | `MediaService` | 文件上传/下载 | ❌ | ✅ | ✅ |
| 11 | `DocumentService` | 文档管理 | ❌ | ✅ | ✅ |
| 12 | `WebhookService` | Webhook 发送 | ✅ | ✅ | ❌ |
| 13 | `EventService` | 事件订阅/回调 | ✅ | ✅ | ✅ |
| 14 | `DepartmentService` | 部门/组织架构 | ✅ | ✅ | ✅ |
| 15 | `MeetingRoomService` | 会议室预定 | ✅ | ❌ | ✅ |
| 16 | `ApprovalService` | 审批/OA流程 | ✅ | ❌ | ✅ |
| 17 | `AttendanceService` | 考勤打卡 | ✅ | ❌ | ✅ |
| 18 | `ReportService` | 日报/周报 | ✅ | ❌ | ❌ |
| 19 | `DataTableService` | 数据表 CRUD | ✅ | ✅ | ✅ |
| 20 | `WikiService` | 知识库 | ❌ | ❌ | ✅ |
| 21 | `EmailService` | 邮件 | ❌ | ❌ | ✅ |

> ⚠️ = 部分支持 (trait 已实现但核心方法返回 NotSupported)

---

### 详细功能矩阵

#### 1. 认证 (AuthService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 获取 access_token | ✅ OAuth2 | ✅ CorpToken | ✅ TenantAccessToken |
| 刷新 token | ✅ refresh | ✅ 重新获取 | ✅ 重新获取 |

#### 2. 消息收发 (MessagingService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 发送文本消息 | ✅ Bot | ✅ | ✅ |
| 发送 Markdown | ✅ | ⚠️ 转text | ✅ post |
| 发送图片/文件/音视频 | ⚠️ 仅图片 | ❌ | ✅ 全部 |
| 发送卡片 | ❌ | ❌ | ✅ |
| 获取历史消息 | ❌ | ✅ 7天 | ✅ 分页 |
| 撤回消息 | ✅ | ❌ | ✅ |

#### 3. 消息扩展 (MessageExtService) — 仅飞书

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 回复消息 | ❌ | ❌ | ✅ |
| 转发消息 | ❌ | ❌ | ✅ |
| 表情回应 | ❌ | ❌ | ✅ |
| 已读状态 | ❌ | ❌ | ✅ |
| 批量获取 | ❌ | ❌ | ✅ |
| **消息置顶** | ❌ | ❌ | ✅ |

#### 4-5. 通讯录 + 群组

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 获取当前用户 | ✅ | ❌ | ✅ |
| 搜索用户 | ✅ | ✅ | ✅ |
| 创建/搜索/管理群 | ✅ | ❌ | ✅ |
| 群公告 | 默认NotSupported | 默认NotSupported | 默认NotSupported |
| 群Bot管理 | 默认NotSupported | 默认NotSupported | 默认NotSupported |

#### 6-8. 会话列表 + 日历 + 任务

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 列出会话 | ❌ | ✅ | ✅ |
| 日程 CRUD + 忙闲 | ✅ | ✅ | ✅ |
| 任务 CRUD | ✅ | ✅ | ✅ |

#### 9-11. 会议 + 媒体 + 文档

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 会议 CRUD | ❌ | ✅ | ✅ VC API |
| 图片/文件上传 | ❌ | ❌ | ✅ multipart |
| 媒体下载 | ❌ | ✅ | ✅ |
| 文档 CRUD | ❌ | ✅ | ✅ |

#### 12. Webhook 发送 (WebhookService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| Webhook 发送 | ✅ HMAC签名 | ✅ URL鉴权 | ❌ |
| 支持文本 | ✅ | ✅ | — |
| 支持 Markdown | ✅ | ✅ | — |
| @提及 | ✅ atUserIds | ✅ mentioned_list | — |

#### 13. 事件订阅 (EventService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 注册回调 | ✅ callback register | ✅ callback create | ⚠️ 需在控制台配置 |
| 列出订阅 | ✅ | ✅ | ⚠️ |
| 删除订阅 | ✅ | ✅ | ⚠️ |
| 事件类型列表 | ✅ 静态列表 | ✅ 静态列表 | ✅ 静态列表 |

> **飞书事件**: 飞书的事件订阅需在开发者控制台配置回调 URL 或使用 WebSocket 长连接

#### 14. 部门管理 (DepartmentService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 列出子部门 | ✅ | ✅ | ✅ |
| 获取部门详情 | ✅ | ✅ | ✅ |
| 列出部门成员 | ✅ | ✅ | ✅ |

#### 15. 会议室预定 (MeetingRoomService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 搜索会议室 | ✅ | ❌ | ✅ |
| 获取会议室 | ✅ | ❌ | ✅ |
| 预定会议室 | ✅ | ❌ | ⚠️ 通过日历 |
| 取消预定 | ✅ | ❌ | ⚠️ 通过日历 |

#### 16. 审批流程 (ApprovalService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 发起审批 | ✅ | ❌ | ✅ |
| 列出审批 | ✅ | ❌ | ✅ |
| 获取审批详情 | ✅ | ❌ | ✅ |
| 审批/拒绝 | ✅ | ❌ | ✅ |

#### 17. 考勤打卡 (AttendanceService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 查询打卡记录 | ✅ | ❌ | ✅ |
| 查询班次 | ✅ | ❌ | ✅ |
| 考勤汇总 | ✅ | ❌ | ✅ |

#### 18. 日报/周报 (ReportService) — 仅钉钉

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 列出模板 | ✅ | ❌ | ❌ |
| 创建日报 | ✅ | ❌ | ❌ |
| 列出/获取日报 | ✅ | ❌ | ❌ |
| 日报统计 | ✅ | ❌ | ❌ |

#### 19. 数据表 (DataTableService)

| 功能 | 钉钉 AITable | 企微 Smartsheet | 飞书 Bitable |
|------|:----:|:----:|:----:|
| Base CRUD | ✅ | ❌ | ✅ |
| Table CRUD | ✅ | ❌ | ✅ |
| Field/Column | ✅ | ✅ | ✅ |
| Record 查询 | ✅ | ✅ | ✅ |
| Record 写入 | ✅ | ✅ 500/批 | ✅ 自动分批 |
| Record 删除 | ✅ | ✅ | ✅ |

#### 20. 知识库 (WikiService) — 仅飞书

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 列出空间 | ❌ | ❌ | ✅ |
| 列出/获取节点 | ❌ | ❌ | ✅ |
| 创建节点 | ❌ | ❌ | ✅ |
| 移动节点 | ❌ | ❌ | ✅ |
| 搜索 | ❌ | ❌ | ✅ |

#### 21. 邮件 (EmailService) — 仅飞书

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 发送邮件 | ❌ | ❌ | ✅ |
| 列出/获取邮件 | ❌ | ❌ | ✅ |
| 列出文件夹 | ❌ | ❌ | ✅ |
| 标记已读 | ❌ | ❌ | ✅ |
| 删除邮件 | ❌ | ❌ | ✅ |

---

## 统一消息类型

| MessageContent 枚举 | 说明 | 钉钉 | 企微 | 飞书 |
|---------------------|------|:----:|:----:|:----:|
| `Text` | 纯文本 | ✅ 发送 | ✅ 收发 | ✅ 收发 |
| `Markdown` | 富文本 | ✅ 发送 | ⚠️ 转text | ✅ post |
| `Image` | 图片 | ✅ URL | ✅ 接收 | ✅ 收发 |
| `File` | 文件 | ❌ | ✅ 接收 | ✅ 收发 |
| `Audio` | 语音 | ❌ | ✅ 接收 | ✅ 收发 |
| `Video` | 视频 | ❌ | ✅ 接收 | ✅ 收发 |
| `Card` | 卡片 | ❌ | ❌ | ✅ 发送 |
| `Unknown` | 平台特定 | — | — | — |

---

## 核心数据类型一览

### 消息 & 扩展
`Message`, `MessageContent`, `SendMessageRequest`, `ListMessagesRequest`, `RecallMessageRequest`, `ReplyMessageRequest`, `ForwardMessageRequest`, `MessageReaction`, `AddReactionRequest`, `MessageReadStatus`, `BatchGetMessagesRequest`, `MessagePin`

### Webhook & 事件
`WebhookMessageRequest`, `WebhookMessageResponse`, `ImEvent`, `RegisterCallbackRequest`, `EventSubscription`

### 组织架构
`User`, `Department`, `DepartmentDetail`, `ListDepartmentsRequest`, `ListDepartmentMembersRequest`

### 群组 & 会话
`GroupChat`, `GroupMember`, `GroupAnnouncement`, `Conversation`, `ConversationType`

### 日历 & 会议
`CalendarEvent`, `EventAttendee`, `BusySlot`, `Meeting`, `MeetingRoom`, `SearchRoomRequest`, `BookRoomRequest`

### 审批 & 考勤 & 日报
`ApprovalInstance`, `ApprovalStatus`, `ApprovalAction`, `AttendanceRecord`, `AttendanceShift`, `AttendanceSummary`, `Report`, `ReportTemplate`, `ReportStatistics`

### 数据表
`DataBase`, `DataTable`, `DataField`, `DataRecord`, `DataRecordWrite`, `ListRecordsRequest`, `WriteRecordsRequest`

### 知识库 & 邮件 & 文档
`WikiSpace`, `WikiNode`, `Email`, `EmailAddress`, `EmailBody`, `Mailbox`, `Document`, `DocumentType`

### 通用
`Platform` (DingTalk/WeCom/Lark), `ChatTarget` (User/Group), `Page<T>`, `Credentials`, `AccessToken`, `ImError`, `MediaInfo`, `Task`

---

## 各平台认证方式

### 钉钉 (DingTalk)
```rust
use tokimo_dingtalk::DingTalkProvider;
let provider = DingTalkProvider::new("client_id", "client_secret");
```

### 企业微信 (WeCom)
```rust
use tokimo_wecom::WeComProvider;
let provider = WeComProvider::new("corp_id", "corp_secret");
```

### 飞书 (Lark/Feishu)
```rust
use tokimo_lark::LarkProvider;
let provider = LarkProvider::feishu("app_id", "app_secret"); // 中国
let provider = LarkProvider::lark("app_id", "app_secret");   // 国际
```

---

## 扩展新平台

实现 `ImProvider` trait 即可接入新平台。21 个服务中除 `auth()` 必需外，其余全部返回 `Option`——不支持的返回 `None` 即可：

```rust
use tokimo_core::*;

pub struct MyPlatformProvider { /* ... */ }

impl ImProvider for MyPlatformProvider {
    fn platform(&self) -> Platform { Platform::DingTalk /* placeholder */ }
    fn auth(&self) -> &dyn AuthService { /* required */ }
    fn messaging(&self) -> Option<&dyn MessagingService> { Some(/* ... */) }
    fn contact(&self) -> Option<&dyn ContactService> { None }
    // ... 其他 19 个服务默认返回 None
}
```

---

## 技术栈

| 依赖 | 用途 |
|------|------|
| **tokio 1** | 异步运行时 |
| **reqwest 0.12** | HTTP 客户端 (json + multipart) |
| **serde / serde_json** | 序列化 |
| **async-trait** | 异步 trait 支持 |
| **chrono** | 时间处理 |
| **thiserror** | 错误类型 |
| **tracing** | 日志 |
| **hmac + sha2 + base64** | Webhook 签名 (钉钉) |

## 项目结构

```
tokimo-universal-im/
├── Cargo.toml                       # Workspace 配置
├── README.md                        # 本文档
└── crates/
    ├── tokimo-core/                 # 核心 trait + 类型 (21 服务)
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs             # ImError 统一错误
    │       ├── types/               # 19 个类型模块
    │       │   ├── common.rs        # Platform, Page, ChatTarget
    │       │   ├── message.rs       # Message, Reply, Forward, Reaction, Pin
    │       │   ├── contact.rs       # User, Department
    │       │   ├── group.rs         # GroupChat, GroupMember
    │       │   ├── calendar.rs      # CalendarEvent, BusySlot
    │       │   ├── task.rs          # Task, TaskStatus
    │       │   ├── meeting.ms       # Meeting
    │       │   ├── meeting_room.rs  # MeetingRoom, SearchRoomRequest
    │       │   ├── conversation.rs  # Conversation
    │       │   ├── document.rs      # Document
    │       │   ├── media.rs         # MediaInfo
    │       │   ├── webhook.rs       # WebhookMessageRequest
    │       │   ├── event.rs         # ImEvent, EventSubscription
    │       │   ├── approval.rs      # ApprovalInstance, ApprovalAction
    │       │   ├── attendance.rs    # AttendanceRecord, Shift, Summary
    │       │   ├── report.rs        # Report, ReportTemplate
    │       │   ├── data_table.rs    # DataBase, DataTable, DataRecord
    │       │   ├── wiki.rs          # WikiSpace, WikiNode
    │       │   ├── email.rs         # Email, Mailbox
    │       │   └── extra.rs         # Pin, Announcement, Department extras
    │       └── traits/              # 21 个 trait 模块
    │           ├── provider.rs      # ImProvider (21 服务入口)
    │           ├── auth.rs          # AuthService
    │           ├── messaging.rs     # MessagingService
    │           ├── message_ext.rs   # MessageExtService (+pin)
    │           ├── contact.rs       # ContactService
    │           ├── group.rs         # GroupService (+announcement, +bot)
    │           ├── chat_list.rs     # ChatListService
    │           ├── calendar.rs      # CalendarService
    │           ├── task.rs          # TaskService
    │           ├── meeting.rs       # MeetingService
    │           ├── media.rs         # MediaService
    │           ├── document.rs      # DocumentService
    │           ├── webhook.rs       # WebhookService
    │           ├── event.rs         # EventService
    │           ├── department.rs    # DepartmentService
    │           ├── meeting_room.rs  # MeetingRoomService
    │           ├── approval.rs      # ApprovalService
    │           ├── attendance.rs    # AttendanceService
    │           ├── report.rs        # ReportService
    │           ├── data_table.rs    # DataTableService
    │           ├── wiki.rs          # WikiService
    │           └── email.rs         # EmailService
    ├── tokimo-dingtalk/             # 钉钉 (14 服务)
    ├── tokimo-wecom/                # 企业微信 (14 服务)
    └── tokimo-lark/                 # 飞书 (19 服务, 最完整)
```

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
