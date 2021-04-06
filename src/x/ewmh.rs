//! This module contains the Ewmh trait, which implements methods 
//! that expose EWMH functionality.
//! Currently implemented by XConn.

use xcb_util::ewmh;

use crate::x::{XConn, XWindowID};

pub trait Ewmh {
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<xcb::Atom>>;
    fn set_supported(&self, screen_idx: i32, atoms: &[xcb::Atom]);
}

impl Ewmh for XConn {
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<xcb::Atom>> {
        debug!("Getting type for window {}", window);

        ewmh::get_wm_window_type(&self.conn, window)
        .get_reply().ok()
        .map(|ok| ok.atoms().to_owned())
    }

    fn set_supported(&self, screen_idx: i32, atoms: &[xcb::Atom]) {
        ewmh::set_supported(&self.conn, screen_idx, atoms);
    }
}