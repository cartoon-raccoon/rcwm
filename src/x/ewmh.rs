//! This module contains the Ewmh trait, which implements methods 
//! that expose EWMH functionality.
//! Currently implemented by XConn.

use xcb_util::ewmh;

use crate::x::{XConn, XWindowID};
use crate::core::{Workspace, Client};
use crate::types::ClientMessageData as CMData;

pub trait Ewmh {
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<xcb::Atom>>;
    fn get_window_states(&self, window: XWindowID) -> Option<Vec<xcb::Atom>>;
    fn set_supported(&self, screen_idx: i32, atoms: &[xcb::Atom]);
}

impl Ewmh for XConn {
    //fn get_wm_name()
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<xcb::Atom>> {
        debug!("Getting type for window {}", window);

        ewmh::get_wm_window_type(&self.conn, window)
        .get_reply().ok()
        .map(|ok| ok.atoms().to_owned())
    }

    fn get_window_states(&self, window: XWindowID) -> Option<Vec<xcb::Atom>> {
        debug!("Getting EWMH states for window {}", window);

        ewmh::get_wm_state(&self.conn, window)
        .get_reply().ok()
        .map(|ok| ok.atoms().to_owned())
    }

    fn set_supported(&self, screen_idx: i32, atoms: &[xcb::Atom]) {
        ewmh::set_supported(&self.conn, screen_idx, atoms);
    }
}

pub fn handle_wm_state(
    conn: &XConn, 
    ws: &mut Workspace, 
    window: XWindowID,
    data: CMData,
) {
    // Why did xcb_util have to implement it this way...
    assert!(data.is_32());

    let raw = conn.get_raw();

    if let CMData::DWord(data) = data {
        let action = data[0];
        let prop1 = data[1];
        let prop2 = data[2];

        if prop1 != 0 {
            debug!("Prop1 is not empty: {}", prop1);
            resolve_state(raw, prop1);
        }
        if prop2 != 0 {
            debug!("Prop2 is not empty: {}", prop2);
            resolve_state(raw, prop2);
        }

        match action {
            ewmh::STATE_ADD => {
                debug!("Add");
            }
            ewmh::STATE_REMOVE => {
                debug!("Remove");
            }
            ewmh::STATE_TOGGLE => {
                debug!("Toggle");
            }
            _ => {unreachable!()}
        }
    } else {
        error!("Client message data incorrectly formatted")
    }
}

fn resolve_state(conn: &ewmh::Connection, state: xcb::Atom) {
    if state == conn.WM_STATE_MODAL() {
        debug!("State is modal");
    } else if state == conn.WM_STATE_STICKY() {
        debug!("State is sticky");
    } else if state == conn.WM_STATE_MAXIMIZED_VERT() {
        debug!("State is max-vert");
    } else if state == conn.WM_STATE_MAXIMIZED_HORZ() {
        debug!("State is max-horz");
    } else if state == conn.WM_STATE_SHADED() {
        debug!("State is shaded");
    } else if state == conn.WM_STATE_SKIP_TASKBAR() {
        debug!("State is skip taskbar");
    } else if state == conn.WM_STATE_SKIP_PAGER() {
        debug!("State is skip pager");
    } else if state == conn.WM_STATE_HIDDEN() {
        debug!("State is hidden");
    } else if state == conn.WM_STATE_FULLSCREEN() {
        debug!("State is fullscreen");
    } else if state == conn.WM_STATE_ABOVE() {
        debug!("State is above");
    } else if state == conn.WM_STATE_BELOW() {
        debug!("State is below");
    } else if state == conn.WM_STATE_DEMANDS_ATTENTION() {
        debug!("State is urgent");
    } else {
        error!("Unknown WM_STATE value {}", state);
    }
}