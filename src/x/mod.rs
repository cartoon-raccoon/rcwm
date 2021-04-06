//! This module provides RaccoonWM's main interface to the X server.
//! Built on top of the XCB crate, it exposes a connection handle
//! and provides the basic methods for retrieving data from and setting data
//! on the X server, that are called by other modules within RaccoonWM.

pub mod xserver;
pub mod ewmh;
pub mod icccm;

pub use xserver as core;

pub use self::core::{XConn, XWindow, XWindowID};
pub use self::icccm::Icccm;
pub use self::ewmh::Ewmh;