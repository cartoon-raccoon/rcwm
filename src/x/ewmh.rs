//! This module doesn't export anything.
//! It just contains methods for XConn that involve certain
//! aspects of EWMH defined functionality.

use xcb_util::ewmh;

use crate::x::{XConn, XWindowID};

impl<'a> XConn<'a> {
    pub fn get_window_type(&self, window: XWindowID) -> Option<Vec<xcb::Atom>> {
        debug!("Getting type for window {}", window);

        ewmh::get_wm_window_type(self.conn, window)
        .get_reply().ok()
        .map(|ok| ok.atoms().to_owned())
    }

    pub fn set_supported(&self, screen_idx: i32, atoms: &[xcb::Atom]) {
        ewmh::set_supported(self.conn, screen_idx, atoms);
    }
}