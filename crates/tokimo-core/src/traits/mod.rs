//! Abstract trait definitions for IM platform providers.
//!
//! Each trait represents a capability domain. Platform implementations
//! only need to implement the traits they support. The [`ImProvider`]
//! super-trait ties everything together.

pub mod auth;
pub mod messaging;
pub mod contact;
pub mod group;
pub mod calendar;
pub mod task;
pub mod provider;

pub use auth::*;
pub use messaging::*;
pub use contact::*;
pub use group::*;
pub use calendar::*;
pub use task::*;
pub use provider::*;
