//! # tokimo-core
//!
//! Core traits and unified types for the Tokimo Universal IM library.
//!
//! This crate defines the abstract interfaces that each IM platform
//! (DingTalk, WeCom, Lark/Feishu, etc.) must implement.

pub mod error;
pub mod types;
pub mod traits;

pub use error::*;
pub use types::*;

// Re-export traits explicitly to avoid ambiguous glob conflicts
pub use traits::auth::AuthService;
pub use traits::messaging::MessagingService;
pub use traits::contact::ContactService;
pub use traits::group::GroupService;
pub use traits::calendar::CalendarService;
pub use traits::task::TaskService;
pub use traits::meeting::MeetingService;
pub use traits::chat_list::ChatListService;
pub use traits::media::MediaService;
pub use traits::message_ext::MessageExtService;
pub use traits::document::DocumentService;
pub use traits::webhook::WebhookService;
pub use traits::event::EventService;
pub use traits::department::DepartmentService;
pub use traits::meeting_room::MeetingRoomService;
pub use traits::approval::ApprovalService;
pub use traits::attendance::AttendanceService;
pub use traits::report::ReportService;
pub use traits::data_table::DataTableService;
pub use traits::wiki::WikiService;
pub use traits::email::EmailService;
pub use traits::provider::ImProvider;
