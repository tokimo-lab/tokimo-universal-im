//! # tokimo-lark
//!
//! Lark/Feishu (飞书) provider implementation for the Tokimo Universal IM library.

mod approval;
mod attendance;
mod auth;
mod calendar;
mod chat_list;
mod client;
mod contact;
mod data_table;
mod department;
mod document;
mod email;
mod event;
mod group;
mod media;
mod meeting;
mod meeting_room;
mod message_ext;
mod messaging;
mod provider;
mod task;
mod wiki;

pub use client::LarkClient;
pub use client::LarkRegion;
pub use provider::LarkProvider;
