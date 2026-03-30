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
mod provider;

pub use client::LarkClient;
pub use provider::LarkProvider;
