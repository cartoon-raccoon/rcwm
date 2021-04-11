//! This module provides RaccoonWM's main interface to the X server.
//! Built on top of the XCB crate, it exposes a connection handle
//! and provides the basic methods for retrieving data from and setting data
//! on the X server, that are called by other modules within RaccoonWM.

pub mod core;
pub mod xserver;
pub mod ewmh;
pub mod icccm;
pub mod event;

pub use self::core::{XConn, XWindow, XWindowID, Atom};
pub use self::event::XEvent;
pub use self::icccm::Icccm;
pub use self::ewmh::Ewmh;