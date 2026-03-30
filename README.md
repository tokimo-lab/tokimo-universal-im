# tokimo-universal-im

**通用企业 IM Rust SDK — 统一钉钉、企业微信、飞书 API**

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

## 概述

`tokimo-universal-im` 是一个 Rust 编写的统一企业即时通讯库，将钉钉（DingTalk）、企业微信（WeCom）和飞书（Lark/Feishu）的 API 抽象为一套统一的 trait 接口。

**设计目标：做通用客户端**。你只需面对一套 API，即可同时对接多个 IM 平台。未来扩展新平台只需实现 `ImProvider` trait。

## 架构

```
┌───────────────────────────────────────────────────────┐
│               你的应用 / 通用客户端                       │
├───────────────────────────────────────────────────────┤
│                tokimo-core (traits + types)              │
│   ImProvider / MessagingService / MeetingService / ...   │
│   11 个服务 trait + 统一数据类型 + 统一错误处理              │
├─────────────┬─────────────────┬───────────────────────┤
│  tokimo-    │   tokimo-       │   tokimo-             │
│  dingtalk   │   wecom         │   lark                │
│  (钉钉)     │   (企业微信)      │   (飞书/Lark)          │
│  6 服务      │   10 服务        │   11 服务              │
└─────────────┴─────────────────┴───────────────────────┘
```

### Crate 说明

| Crate | 说明 |
|-------|------|
| `tokimo-core` | 核心 trait 定义 + 统一数据类型，**所有客户端只需依赖此 crate** |
| `tokimo-dingtalk` | 钉钉平台实现 (auth, messaging, contact, group, calendar, task) |
| `tokimo-wecom` | 企业微信平台实现 (auth, messaging, contact, group, calendar, task, meeting, chat_list, media, document) |
| `tokimo-lark` | 飞书/Lark 平台实现 (auth + 全部 10 个服务，功能最完整) |

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
    // 1. 创建 provider
    let provider = LarkProvider::feishu("app_id", "app_secret");

    // 2. 获取 token
    let _token = provider.auth().get_access_token(&Credentials {
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

    Ok(())
}
```

### 拉取历史消息

```rust
use tokimo_core::*;

async fn pull_history(provider: &dyn ImProvider) -> ImResult<()> {
    let messaging = provider.messaging().unwrap();

    // 拉取群聊历史消息（支持分页）
    let page = messaging.list_messages(ListMessagesRequest {
        chat_id: "oc_xxxxx".into(),
        chat_type: Some(ChatTypeHint::Group),
        start_time: None,
        end_time: None,
        cursor: None,
        limit: Some(50),
    }).await?;

    for msg in &page.items {
        println!("[{}] {}: {:?}", msg.id, msg.sender.name, msg.content);
    }

    // 如果还有更多消息，继续分页
    if page.has_more {
        let _next = messaging.list_messages(ListMessagesRequest {
            chat_id: "oc_xxxxx".into(),
            chat_type: Some(ChatTypeHint::Group),
            start_time: None,
            end_time: None,
            cursor: page.next_cursor.clone(),
            limit: Some(50),
        }).await?;
    }

    Ok(())
}
```

### 消息回复、转发、表情回应（飞书）

```rust
use tokimo_core::*;

async fn message_operations(provider: &dyn ImProvider) -> ImResult<()> {
    let ext = provider.message_ext().ok_or(ImError::NotSupported {
        feature: "message_ext".into(),
        platform: provider.platform().to_string(),
    })?;

    // 回复消息
    ext.reply_message(ReplyMessageRequest {
        reply_to_message_id: "om_xxxxx".into(),
        content: MessageContent::Text(TextContent {
            text: "收到，已处理！".into(),
            mentions: vec![],
        }),
    }).await?;

    // 转发消息到另一个群
    ext.forward_message(ForwardMessageRequest {
        message_id: "om_xxxxx".into(),
        target: ChatTarget::Group("oc_yyyyy".into()),
    }).await?;

    // 给消息添加表情回应
    ext.add_reaction(AddReactionRequest {
        message_id: "om_xxxxx".into(),
        emoji_type: "THUMBSUP".into(),
    }).await?;

    // 查看消息已读状态
    let status = ext.get_read_status("om_xxxxx").await?;
    println!("已读: {}/{}", status.read_count, status.total_count);

    // 批量获取消息
    let _messages = ext.batch_get_messages(BatchGetMessagesRequest {
        message_ids: vec!["om_aaa".into(), "om_bbb".into()],
    }).await?;

    Ok(())
}
```

### 会议管理

```rust
use tokimo_core::*;

async fn meeting_operations(provider: &dyn ImProvider) -> ImResult<()> {
    let meeting_svc = provider.meeting().ok_or(ImError::NotSupported {
        feature: "meeting".into(),
        platform: provider.platform().to_string(),
    })?;

    // 创建会议
    let meeting = meeting_svc.create_meeting(CreateMeetingRequest {
        title: "周会".into(),
        start_time: chrono::Utc::now(),
        end_time: chrono::Utc::now() + chrono::Duration::hours(1),
        attendees: vec!["user_id_1".into(), "user_id_2".into()],
        settings: None,
        description: Some("每周例会".into()),
    }).await?;
    println!("会议ID: {}", meeting.id);

    // 列出会议
    let _meetings = meeting_svc.list_meetings(ListMeetingsRequest {
        start_time: None,
        end_time: None,
        cursor: None,
        limit: Some(20),
    }).await?;

    // 取消会议
    meeting_svc.cancel_meeting(&meeting.id).await?;

    Ok(())
}
```

### 会话列表

```rust
use tokimo_core::*;

async fn list_chats(provider: &dyn ImProvider) -> ImResult<()> {
    let chat_list = provider.chat_list().ok_or(ImError::NotSupported {
        feature: "chat_list".into(),
        platform: provider.platform().to_string(),
    })?;

    let conversations = chat_list.list_conversations(ListConversationsRequest {
        cursor: None,
        limit: Some(50),
    }).await?;

    for conv in &conversations.items {
        println!("[{:?}] {} ({})", conv.conversation_type, conv.name, conv.id);
    }

    Ok(())
}
```

### 媒体文件上传/下载（飞书）

```rust
use tokimo_core::*;

async fn media_operations(provider: &dyn ImProvider) -> ImResult<()> {
    let media = provider.media().ok_or(ImError::NotSupported {
        feature: "media".into(),
        platform: provider.platform().to_string(),
    })?;

    // 上传图片 → 获取 image_key
    let image_data = std::fs::read("photo.jpg").unwrap();
    let image_info = media.upload_image(image_data, "photo.jpg").await?;
    println!("image_key: {}", image_info.media_id);

    // 上传文件 → 获取 file_key
    let file_data = std::fs::read("report.pdf").unwrap();
    let file_info = media.upload_file(file_data, "report.pdf").await?;
    println!("file_key: {}", file_info.media_id);

    // 下载媒体
    let content = media.download_media(&image_info.media_id).await?;
    std::fs::write("downloaded.jpg", content).unwrap();

    Ok(())
}
```

### 文档管理

```rust
use tokimo_core::*;

async fn document_operations(provider: &dyn ImProvider) -> ImResult<()> {
    let doc_svc = provider.document().ok_or(ImError::NotSupported {
        feature: "document".into(),
        platform: provider.platform().to_string(),
    })?;

    // 创建文档
    let doc = doc_svc.create_document(CreateDocumentRequest {
        title: "项目方案".into(),
        doc_type: DocumentType::Doc,
        content: Some("# 项目方案\n\n## 背景\n...".into()),
        folder_id: None,
    }).await?;

    // 获取文档
    let _doc_info = doc_svc.get_document(&doc.id).await?;

    // 搜索文档
    let _results = doc_svc.search_documents(SearchDocumentRequest {
        query: "项目方案".into(),
        doc_type: None,
        cursor: None,
        limit: Some(10),
    }).await?;

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

/// 优雅处理不同平台的功能差异（降级策略）
async fn try_reply_or_send(
    provider: &dyn ImProvider,
    message_id: &str,
    chat_id: &str,
    text: &str,
) -> ImResult<()> {
    let content = MessageContent::Text(TextContent {
        text: text.into(),
        mentions: vec![],
    });

    // 优先尝试回复，不支持则降级为普通发送
    if let Some(ext) = provider.message_ext() {
        ext.reply_message(ReplyMessageRequest {
            reply_to_message_id: message_id.into(),
            content,
        }).await?;
    } else if let Some(messaging) = provider.messaging() {
        messaging.send_message(SendMessageRequest {
            target: ChatTarget::Group(chat_id.into()),
            content,
            bot_id: None,
            idempotency_key: None,
        }).await?;
    }

    Ok(())
}
```

---

## 统一消息类型

| MessageContent 枚举 | 说明 | 钉钉 | 企微 | 飞书 |
|---------------------|------|:----:|:----:|:----:|
| `Text` | 纯文本消息 | ✅ 发送 | ✅ 收发 | ✅ 收发 |
| `Markdown` | 富文本 / Markdown | ✅ 发送 | ⚠️ 转为 text | ✅ 转为 post |
| `Image` | 图片消息 | ✅ 发送(URL) | ✅ 接收 | ✅ 收发(image_key) |
| `File` | 文件消息 | ❌ | ✅ 接收 | ✅ 收发(file_key) |
| `Audio` | 语音消息 | ❌ | ✅ 接收 | ✅ 收发(file_key) |
| `Video` | 视频消息 | ❌ | ✅ 接收 | ✅ 收发(file_key+cover) |
| `Card` | 交互卡片 | ❌ | ❌ | ✅ 发送(interactive JSON) |
| `Unknown` | 未知/平台特定类型 | — | — | — |

---

## 功能实现状态 — 完整矩阵

### 服务总览

| # | 服务 (Trait) | 说明 | 钉钉 | 企微 | 飞书 |
|:-:|-------------|------|:----:|:----:|:----:|
| 1 | `AuthService` | 认证 | ✅ | ✅ | ✅ |
| 2 | `MessagingService` | 消息收发 | ✅ | ✅ | ✅ |
| 3 | `MessageExtService` | 回复/转发/表情/已读 | ❌ | ❌ | ✅ |
| 4 | `ContactService` | 通讯录 | ✅ | ✅ | ✅ |
| 5 | `GroupService` | 群组管理 | ✅ | ⚠️ | ✅ |
| 6 | `ChatListService` | 会话列表 | ❌ | ✅ | ✅ |
| 7 | `CalendarService` | 日历/日程 | ✅ | ✅ | ✅ |
| 8 | `TaskService` | 待办任务 | ✅ | ✅ | ✅ |
| 9 | `MeetingService` | 会议管理 | ❌ | ✅ | ✅ |
| 10 | `MediaService` | 文件上传/下载 | ❌ | ✅ | ✅ |
| 11 | `DocumentService` | 文档管理 | ❌ | ✅ | ✅ |

> ⚠️ = 部分支持（trait 已实现但方法返回 NotSupported）

---

### 详细功能矩阵

#### 1. 认证 (AuthService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 获取 access_token | ✅ OAuth2 UserAccessToken | ✅ CorpToken | ✅ TenantAccessToken |
| 刷新 token | ✅ refresh_token | ✅ 重新获取 | ✅ 重新获取 |

#### 2. 消息收发 (MessagingService) — 核心功能

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 发送文本消息（单聊） | ✅ Bot 批量单聊 | ✅ send_message | ✅ open_id |
| 发送文本消息（群聊） | ✅ Bot 群聊 | ✅ send_message | ✅ chat_id |
| 发送 Markdown 消息 | ✅ sampleMarkdown | ⚠️ 转为纯文本 | ✅ post 格式 |
| 发送图片消息 | ✅ sampleImageMsg | ❌ | ✅ image_key |
| 发送文件消息 | ❌ | ❌ | ✅ file_key |
| 发送音频/视频消息 | ❌ | ❌ | ✅ |
| 发送卡片消息 | ❌ | ❌ | ✅ interactive JSON |
| **获取历史消息** | ⚠️ 无REST API | ✅ 7天 单/群 | ✅ 分页拉取 |
| 撤回消息 | ✅ processQueryKey | ❌ | ✅ DELETE |

> **关于拉取历史消息：**
> - **企业微信**：`list_messages` 通过 `chat_type` 指定单聊(1)/群聊(2)，最多拉取7天内消息
> - **飞书**：`list_messages` 通过 `container_id_type=chat` + `container_id` 分页拉取
> - **钉钉**：REST API 不支持拉取历史消息，需通过 webhook/回调接收实时消息

#### 3. 消息扩展 (MessageExtService) — 仅飞书

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 回复消息 | ❌ | ❌ | ✅ reply to message_id |
| 转发消息 | ❌ | ❌ | ✅ forward to user/group |
| 添加表情回应 | ❌ | ❌ | ✅ emoji reactions |
| 删除表情回应 | ❌ | ❌ | ✅ |
| 获取消息已读状态 | ❌ | ❌ | ✅ read_users + count |
| 批量获取消息 | ❌ | ❌ | ✅ batch get by IDs |

#### 4. 通讯录 (ContactService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 获取当前用户 | ✅ | ❌ | ✅ |
| 搜索用户 | ✅ keyword | ✅ 列出可见联系人 | ✅ query |
| 批量获取用户 | ✅ userIds | ❌ | ✅ 逐个获取 |

#### 5. 群组管理 (GroupService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建群 | ✅ | ❌ | ✅ |
| 搜索群 | ✅ | ❌ | ✅ |
| 获取群信息 | ✅ | ❌ | ✅ |
| 获取群成员 | ✅ | ❌ | ✅ |
| 添加/移除成员 | ✅ | ❌ | ✅ |

#### 6. 会话列表 (ChatListService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 列出会话 | ❌ | ✅ get_msg_chat_list | ✅ list chats |

> 返回 `Conversation` 列表，包含 id、name、类型(单聊/群聊)

#### 7. 日历/日程 (CalendarService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建日程 | ✅ | ✅ | ✅ |
| 列出日程 | ✅ | ✅ | ✅ |
| 获取日程详情 | ✅ | ✅ | ✅ |
| 更新日程 | ✅ | ✅ | ✅ |
| 删除日程 | ✅ | ✅ | ✅ |
| 查询忙闲 | ✅ | ✅ | ✅ |

#### 8. 待办任务 (TaskService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建/列出/获取/更新/删除 | ✅ | ✅ | ✅ |

#### 9. 会议管理 (MeetingService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建会议 | ❌ | ✅ | ✅ VC API |
| 列出会议 | ❌ | ✅ | ✅ |
| 获取会议详情 | ❌ | ✅ | ✅ |
| 取消会议 | ❌ | ✅ | ❌ |
| 更新会议成员 | ❌ | ✅ 添加/删除/静音 | ❌ |

> **钉钉**：conference 标记为"即将支持"，暂无 REST API
> **飞书**：通过 Video Conference (VC) API 实现

#### 10. 媒体管理 (MediaService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 上传图片 | ❌ | ❌ | ✅ multipart form |
| 上传文件 | ❌ | ❌ | ✅ multipart form |
| 下载媒体 | ❌ | ✅ get_msg_media | ✅ resource download |

> **企业微信**：`download_media` 返回媒体元信息（非二进制），需额外处理
> **飞书**：上传返回 `image_key` / `file_key`，可直接用于消息发送

#### 11. 文档管理 (DocumentService)

| 功能 | 钉钉 | 企微 | 飞书 |
|------|:----:|:----:|:----:|
| 创建文档 | ❌ | ✅ create_doc | ✅ create docx |
| 获取文档 | ❌ | ✅ async轮询 | ✅ get |
| 更新文档 | ❌ | ✅ edit_doc | ✅ raw content |
| 搜索文档 | ❌ | ❌ | ✅ suite search |

> **企业微信**：`get_document` 是异步操作——先提交任务，再用 `task_id` 轮询结果
> **钉钉**：文档功能（doc/drive）标记为"即将支持"

---

## 🔲 尚未实现的功能

### 高优先级（推荐后续实现）

| 功能域 | 具体功能 | 可用平台 | 备注 |
|--------|----------|----------|------|
| **事件** | WebSocket 实时事件订阅 | 飞书 | 实时接收消息的核心能力 |
| **事件** | Webhook 回调 | 钉钉、企微 | 接收推送消息 |
| **消息** | Webhook 发送 | 钉钉 | 自定义机器人 outgoing |
| **消息** | 消息置顶 (Pin) | 飞书 | IM 常用功能 |
| **消息** | DING 紧急提醒 | 钉钉 | 钉钉特色功能 |

### 中优先级

| 功能域 | 具体功能 | 可用平台 | 备注 |
|--------|----------|----------|------|
| **群组** | 群公告 | 飞书 | 群管理常用 |
| **群组** | 添加/移除机器人 | 钉钉 | 自动化 |
| **通讯录** | 部门搜索/列表 | 钉钉 | 组织架构 |
| **通讯录** | 批量获取部门成员 | 钉钉 | 批量操作 |
| **日历** | 会议室管理 | 钉钉 | 资源预定 |

### 低优先级（平台特有）

| 功能域 | 具体功能 | 可用平台 | 备注 |
|--------|----------|----------|------|
| **数据表** | AITable / Base CRUD | 钉钉、飞书 | 结构化数据 |
| **审批** | OA 审批流程 | 钉钉 | 企业流程 |
| **考勤** | 打卡/班次 | 钉钉 | HR |
| **日报** | 日志/日报 | 钉钉 | 工作汇报 |
| **Wiki** | 知识库管理 | 飞书 | 文档协作 |
| **邮件** | 邮件收发 | 飞书 | 邮箱集成 |

---

## 各平台认证方式

### 钉钉 (DingTalk)

```rust
use tokimo_dingtalk::DingTalkProvider;

let provider = DingTalkProvider::new("client_id", "client_secret");
// OAuth2 device flow 获取用户级 token
// 或企业内部应用 AppKey/AppSecret
```

### 企业微信 (WeCom)

```rust
use tokimo_wecom::WeComProvider;

let provider = WeComProvider::new("corp_id", "corp_secret");
// corpid + corpsecret 获取 access_token
```

### 飞书 (Lark/Feishu)

```rust
use tokimo_lark::LarkProvider;

// 中国大陆 - Feishu
let provider = LarkProvider::feishu("app_id", "app_secret");

// 国际版 - Lark
let provider = LarkProvider::lark("app_id", "app_secret");
```

---

## 核心数据类型一览

### 消息相关

| 类型 | 说明 |
|------|------|
| `Message` | 统一消息 (id, chat_id, sender, content, timestamp, thread_id) |
| `MessageContent` | 枚举: Text / Markdown / Image / File / Audio / Video / Card / Unknown |
| `SendMessageRequest` | 发送请求 (target, content, bot_id, idempotency_key) |
| `ListMessagesRequest` | 列出请求 (chat_id, chat_type, time range, cursor, limit) |
| `RecallMessageRequest` | 撤回请求 (message_id, bot_id, chat_id) |
| `ReplyMessageRequest` | 回复请求 (reply_to_message_id, content) |
| `ForwardMessageRequest` | 转发请求 (message_id, target) |
| `MessageReaction` | 表情回应 (reaction_id, emoji_type, user_id) |
| `AddReactionRequest` | 添加回应 (message_id, emoji_type) |
| `MessageReadStatus` | 已读状态 (read_users, total/read_count) |
| `BatchGetMessagesRequest` | 批量获取 (message_ids) |

### 会议相关

| 类型 | 说明 |
|------|------|
| `Meeting` | 会议 (id, title, status, time range, host, attendees) |
| `MeetingStatus` | Scheduled / InProgress / Ended / Cancelled |
| `CreateMeetingRequest` | 创建请求 (title, time, attendees, settings) |
| `ListMeetingsRequest` | 列出请求 (time range, cursor, limit) |

### 会话/文档/媒体

| 类型 | 说明 |
|------|------|
| `Conversation` | 会话 (id, name, type, last_message_time) |
| `Document` | 文档 (id, title, doc_type, url, content) |
| `MediaInfo` | 媒体信息 (media_id, media_type, size, name) |

### 联系人/群组/日历/任务

| 类型 | 说明 |
|------|------|
| `User` | 用户 (id, name, email, phone, avatar, departments) |
| `Department` | 部门 (id, name, parent_id) |
| `GroupChat` | 群组 (id, name, owner_id, member_count) |
| `GroupMember` | 群成员 (user_id, name, role) |
| `CalendarEvent` | 日程 (id, title, time, location, attendees) |
| `Task` | 任务 (id, title, status, priority, due_time) |

### 通用类型

| 类型 | 说明 |
|------|------|
| `Platform` | DingTalk / WeCom / Lark |
| `ChatTarget` | User(id) / Group(id) |
| `ChatTypeHint` | Single / Group (用于 list_messages) |
| `Page<T>` | 分页包装 (items, has_more, next_cursor) |
| `Credentials` | 认证凭证 (client_id, client_secret) |
| `AccessToken` | 令牌 (token, expires_at, refresh_token) |
| `ImError` | 统一错误 (Auth / NotFound / RateLimited / Platform / NotSupported / ...) |

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

## 项目结构

```
tokimo-universal-im/
├── Cargo.toml                     # Workspace 配置
├── README.md                      # 本文档
└── crates/
    ├── tokimo-core/               # 核心 trait + 类型
    │   └── src/
    │       ├── lib.rs
    │       ├── error.rs           # ImError 统一错误
    │       ├── types/
    │       │   ├── common.rs      # Platform, Page, ChatTarget, Credentials
    │       │   ├── message.rs     # Message, MessageContent, Reply/Forward/Reaction
    │       │   ├── contact.rs     # User, Department
    │       │   ├── group.rs       # GroupChat, GroupMember
    │       │   ├── calendar.rs    # CalendarEvent, BusySlot
    │       │   ├── task.rs        # Task, TaskStatus
    │       │   ├── meeting.rs     # Meeting, MeetingStatus
    │       │   ├── conversation.rs # Conversation, ConversationType
    │       │   ├── document.rs    # Document, DocumentType
    │       │   └── media.rs       # MediaInfo, MediaType
    │       └── traits/
    │           ├── provider.rs    # ImProvider (11 个服务入口)
    │           ├── auth.rs        # AuthService
    │           ├── messaging.rs   # MessagingService
    │           ├── message_ext.rs # MessageExtService
    │           ├── contact.rs     # ContactService
    │           ├── group.rs       # GroupService
    │           ├── chat_list.rs   # ChatListService
    │           ├── calendar.rs    # CalendarService
    │           ├── task.rs        # TaskService
    │           ├── meeting.rs     # MeetingService
    │           ├── media.rs       # MediaService
    │           └── document.rs    # DocumentService
    ├── tokimo-dingtalk/           # 钉钉 (6 服务)
    │   └── src/
    │       ├── client.rs          # HTTP 客户端
    │       ├── auth.rs            # OAuth2 token
    │       ├── messaging.rs       # Bot 消息
    │       ├── contact.rs         # 联系人
    │       ├── group.rs           # 群组
    │       ├── calendar.rs        # 日历
    │       ├── task.rs            # 任务
    │       └── provider.rs        # DingTalkProvider
    ├── tokimo-wecom/              # 企业微信 (10 服务)
    │   └── src/
    │       ├── client.rs          # HTTP 客户端
    │       ├── auth.rs            # corpid/corpsecret
    │       ├── messaging.rs       # 消息 + 历史记录 (7天)
    │       ├── contact.rs         # 联系人
    │       ├── group.rs           # 群组 (stub)
    │       ├── calendar.rs        # 日程 + 忙闲
    │       ├── task.rs            # 待办
    │       ├── meeting.rs         # 会议 CRUD + 成员管理
    │       ├── chat_list.rs       # 会话列表
    │       ├── media.rs           # 媒体下载
    │       ├── document.rs        # 文档 CRUD
    │       └── provider.rs        # WeComProvider
    └── tokimo-lark/               # 飞书 (11 服务, 最完整)
        └── src/
            ├── client.rs          # HTTP 客户端 (Feishu/Lark)
            ├── auth.rs            # Tenant/User access token
            ├── messaging.rs       # 全格式消息 + 撤回
            ├── message_ext.rs     # 回复/转发/表情/已读/批量获取
            ├── contact.rs         # 用户搜索
            ├── group.rs           # 群组 CRUD + 成员
            ├── chat_list.rs       # 会话列表
            ├── calendar.rs        # 日历 + 忙闲
            ├── task.rs            # 任务
            ├── meeting.rs         # VC 会议
            ├── media.rs           # 图片/文件上传 + 下载
            ├── document.rs        # 文档 CRUD + 搜索
            └── provider.rs        # LarkProvider
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
