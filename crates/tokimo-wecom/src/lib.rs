//! # tokimo-wecom
//!
//! WeCom (企业微信) provider implementation for the Tokimo Universal IM library.

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
mod document;
mod webhook;
mod department;
mod data_table;
mod event;
mod provider;

pub use client::WeComClient;
pub use provider::WeComProvider;
