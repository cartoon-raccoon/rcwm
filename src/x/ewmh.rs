//! This module contains the Ewmh trait, which implements methods 
//! that expose EWMH functionality.
//! Currently implemented by XConn.

use xcb_util::ewmh;

use crate::x::core::{
    Atom,
    XConn, 
    XWindowID, 
    ClientMessageData as CMData
};
use crate::core::Workspace;
use crate::types::{
    NetWindowStates
};

pub trait Ewmh {
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<Atom>>;
    fn get_window_states(&self, window: XWindowID) -> NetWindowStates;
    fn set_supported(&self, screen_idx: i32, atoms: &[Atom]);
    fn set_wm_state(&self, window: XWindowID, atoms: &[Atom]);
}

impl Ewmh for XConn {
    //fn get_wm_name()
    fn get_window_type(&self, window: XWindowID) -> Option<Vec<Atom>> {
        debug!("Getting type for window {}", window);

        ewmh::get_wm_window_type(&self.conn, window)
        .get_reply().ok()
        .map(|ok| ok.atoms().to_owned())
    }

    fn get_window_states(&self, window: XWindowID) -> NetWindowStates {
        debug!("Getting EWMH states for window {}", window);

        if let Some(states) = ewmh::get_wm_state(&self.conn, window)
        .get_reply().ok()
        .map(|ok| ok.atoms().to_owned()) {
            return NetWindowStates::from(states)
        } else {
            return NetWindowStates::new()
        }
    }

    fn set_supported(&self, screen_idx: i32, atoms: &[Atom]) {
        ewmh::set_supported(&self.conn, screen_idx, atoms);
    }

    fn set_wm_state(&self, window: XWindowID, atoms: &[Atom]) {
        ewmh::set_wm_state(&self.conn, window, atoms);
    }
}

// impl dyn Ewmh {
//     fn change_window_state(&)
// }

/// Called from `wm::clientmessage`.
pub(crate) fn handle_wm_state(
    conn: &XConn, 
    ws: &mut Workspace, 
    window: XWindowID,
    data: CMData,
) {
    //todo: implement handling
    assert!(data.is_u32());

    let raw = conn.get_raw();

    if let CMData::U32(data) = data {
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

        let win = ws.windows.lookup_mut(window);

        match action {
            ewmh::STATE_ADD => {
                if let Some(win) = win {
                    if prop1 != 0 {
                        win.add_wm_state(prop1);
                    }
                    if prop2 != 0 {
                        win.add_wm_state(prop2);
                    }
                    win.set_wm_states(conn);
                } else {
                    let mut states = conn.get_window_states(window);
                    if prop1 != 0 {states.add(prop1);}
                    if prop2 != 0 {states.add(prop2);}
                    conn.set_wm_state(window, &states);
                }
            }
            ewmh::STATE_REMOVE => {
                if let Some(win) = win {
                    if prop1 != 0 {
                        win.remove_wm_state(prop1);
                    }
                    if prop2 != 0 {
                        win.remove_wm_state(prop2);
                    }
                    win.set_wm_states(conn);
                } else {
                    let mut states = conn.get_window_states(window);
                    if prop1 != 0 {states.remove(prop1);}
                    if prop2 != 0 {states.remove(prop2);}
                    conn.set_wm_state(window, &states);
                }
            }
            ewmh::STATE_TOGGLE => {
                //todo: we need to remember the previous state of the window
                debug!("Toggle");
            }
            _ => {unreachable!()}
        }
    } else {
        error!("Client message data incorrectly formatted")
    }
}

//todo: make this return shit
fn resolve_state(conn: &ewmh::Connection, state: Atom) {
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