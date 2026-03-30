//! Abstract trait definitions for IM platform providers.

pub mod auth;
pub mod messaging;
pub mod contact;
pub mod group;
pub mod calendar;
pub mod task;
pub mod meeting;
pub mod chat_list;
pub mod media;
pub mod message_ext;
pub mod document;
pub mod provider;

pub use auth::*;
pub use messaging::*;
pub use contact::*;
pub use group::*;
pub use calendar::*;
pub use task::*;
pub use meeting::*;
pub use chat_list::*;
pub use media::*;
pub use message_ext::*;
pub use document::*;
pub use provider::*;
