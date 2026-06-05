//! # tokimo-wecom
//!
//! WeCom (企业微信) provider implementation for the Tokimo Universal IM library.

mod auth;
mod calendar;
mod chat_list;
mod client;
mod contact;
mod data_table;
mod department;
mod document;
mod event;
mod group;
mod media;
mod meeting;
mod messaging;
mod provider;
mod task;
mod webhook;

pub use client::WeComClient;
pub use provider::WeComProvider;
