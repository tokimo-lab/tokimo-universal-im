//! # tokimo-lark
//!
//! Lark/Feishu (飞书) provider implementation for the Tokimo Universal IM library.

mod client;
mod auth;
mod messaging;
mod contact;
mod group;
mod calendar;
mod task;
mod meeting;
mod chat_list;
mod media;
mod message_ext;
mod document;
mod department;
mod meeting_room;
mod approval;
mod attendance;
mod data_table;
mod wiki;
mod email;
mod event;
mod provider;

pub use client::LarkClient;
pub use client::LarkRegion;
pub use provider::LarkProvider;
