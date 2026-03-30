//! # tokimo-dingtalk
//!
//! DingTalk (钉钉) provider implementation for the Tokimo Universal IM library.
//!
//! Implements the core traits using DingTalk's Open Platform REST APIs.

mod client;
mod auth;
mod messaging;
mod contact;
mod group;
mod calendar;
mod task;
mod provider;

pub use client::DingTalkClient;
pub use provider::DingTalkProvider;
