//! Unified data types shared across all IM platforms.

pub mod message;
pub mod contact;
pub mod group;
pub mod calendar;
pub mod task;
pub mod media;
pub mod common;

pub use message::*;
pub use contact::*;
pub use group::*;
pub use calendar::*;
pub use task::*;
pub use media::*;
pub use common::*;
