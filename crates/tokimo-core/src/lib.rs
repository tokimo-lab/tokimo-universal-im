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
pub use traits::provider::ImProvider;
