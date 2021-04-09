//! The core components of RaccoonWM's functionality.

pub mod window;
pub mod workspace;
pub mod desktop;
pub mod types;
pub mod ring;

pub use self::ring::{
    Ring, Selector,
};

pub use self::workspace::Workspace;
pub use self::window::{Client, ClientRing};