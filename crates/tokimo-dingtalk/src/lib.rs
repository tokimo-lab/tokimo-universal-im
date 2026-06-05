//! # tokimo-dingtalk
//!
//! DingTalk (钉钉) provider implementation for the Tokimo Universal IM library.
//!
//! Implements the core traits using DingTalk's Open Platform REST APIs.

mod approval;
mod attendance;
mod auth;
mod calendar;
mod client;
mod contact;
mod data_table;
mod department;
mod event;
mod group;
mod meeting_room;
mod messaging;
mod provider;
mod report;
mod task;
mod webhook;

pub use client::DingTalkClient;
pub use provider::DingTalkProvider;
