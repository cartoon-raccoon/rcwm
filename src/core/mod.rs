//! The core components of RaccoonWM's functionality.

pub mod window;
pub mod workspace;
pub mod desktop;
pub mod types;
pub mod ring;

pub use self::ring::{
    Ring, Selector,
};