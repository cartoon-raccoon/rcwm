//! This module contains the Icccm trait, which implements methods 
//! that expose ICCCM functionality.
//! Currently implemented by XConn.

use xcb_util::icccm;

use crate::x::{XConn, XWindowID, Atom};
use crate::types::{XWinProperties, WindowState};

/// Exposes ICCCM functionality for an object holding an XCB connection.
/// Mainly used to expose ICCCM functionality for XConn only when needed.
/// For more information on what these methods return, consult the
/// [ICCCM](https://www.x.org/releases/X11R7.6/doc/xorg-docs/specs/ICCCM/icccm.html)
/// reference.
/// 
/// Warning: DO NOT READ THROUGH EVERYTHING. It is incredibly boring and you _will_
/// fall asleep. Consult only the parts you need, as needed.
pub trait Icccm {
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties;
    fn get_wm_name(&self, window: XWindowID) -> String;
    fn get_wm_icon_name(&self, window: XWindowID) -> String;
    fn get_wm_size_hints(&self, window: XWindowID) -> Option<icccm::SizeHints>;
    fn get_wm_hints(&self, window: XWindowID) -> Option<icccm::WmHints>;    
    fn get_wm_class(&self, window: XWindowID) -> Option<(String, String)>;
    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<Atom>>;
    fn get_wm_state(&self, window: XWindowID) -> WindowState;
    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID>;
    fn get_urgency(&self, window: XWindowID) -> bool;
}

impl Icccm for XConn {
    fn get_client_properties(&self, window: XWindowID) -> XWinProperties {
        debug!("Getting client properties for window {}", window);
        //todo: get wm_transient_for
        // fire off all requests at once, then get replies all together
        let wm_name_cookie = icccm::get_wm_name(&self.conn, window);
        let wm_icon_cookie = icccm::get_wm_icon_name(&self.conn, window);
        let wm_size_hints_cookie = icccm::get_wm_normal_hints(&self.conn, window);
        let wm_hints_cookie = icccm::get_wm_hints(&self.conn, window);
        let wm_class_cookie = icccm::get_wm_class(&self.conn, window);
        let wm_protocols_cookie = icccm::get_wm_protocols(&self.conn, window, self.atoms.WM_PROTOCOLS);
        let wm_state_cookie = icccm::get_wm_state(&self.conn, window);

        let wm_name = match wm_name_cookie.get_reply() {
            Ok(reply) => reply.name().into(),
            Err(_) => String::new()
        };
        let wm_icon_name = match wm_icon_cookie.get_reply() {
            Ok(reply) => reply.name().into(),
            Err(_) => String::new()
        };
        let wm_size_hints = wm_size_hints_cookie.get_reply().ok();
        let wm_hints = wm_hints_cookie.get_reply().ok();
        let wm_class = match wm_class_cookie.get_reply() {
            Ok(reply) => (reply.instance().into(), reply.class().into()),
            Err(_) => (String::new(), String::new())
        };
        let wm_protocols = match wm_protocols_cookie.get_reply() {
            Ok(reply) => Some(reply.atoms().into()),
            Err(_) => None
        };
        let wm_state = match wm_state_cookie.get_reply() {
            Ok(reply) => reply.state().into(),
            Err(_) => WindowState::default()
        };

        XWinProperties {
            wm_name: wm_name,
            wm_icon_name: wm_icon_name,
            wm_size_hints: wm_size_hints,
            wm_hints: wm_hints,
            wm_class: wm_class,
            wm_protocols: wm_protocols,
            wm_state: wm_state,
        }
    }

    fn get_wm_name(&self, window: XWindowID) -> String {
        match icccm::get_wm_name(&self.conn, window).get_reply() {
            Ok(reply) => reply.name().into(),
            Err(_) => String::new()
        }
    }

    fn get_wm_icon_name(&self, window: XWindowID) -> String {
        match icccm::get_wm_icon_name(&self.conn, window).get_reply() {
            Ok(reply) => reply.name().into(),
            Err(_) => String::new()
        }
    }

    fn get_wm_size_hints(&self, window: XWindowID) -> Option<icccm::SizeHints> {
        icccm::get_wm_normal_hints(&self.conn, window).get_reply().ok()
    }

    fn get_wm_hints(&self, window: XWindowID) -> Option<icccm::WmHints> {
        icccm::get_wm_hints(&self.conn, window).get_reply().ok()
    }

    fn get_wm_class(&self, window: XWindowID) -> Option<(String, String)> {
        match icccm::get_wm_class(&self.conn, window).get_reply() {
            Ok(reply) => Some((reply.instance().into(), reply.class().into())),
            Err(_) => None
        }
    }

    fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<Atom>> {
        debug!("Getting protocols for window {}", window);
        match icccm::get_wm_protocols(&self.conn, window, self.atoms.WM_PROTOCOLS)
        .get_reply() {
            Ok(reply) => {
                Some(reply.atoms().to_owned())
            }
            Err(_e) => {
                None
            }
        }
    }

    fn get_wm_transient_for(&self, window: XWindowID) -> Option<XWindowID> {
        debug!("Get wm_transient_for for window {}", window);
        let res = xcb::get_property(
            &self.conn, // connection
            false,      // delete?
            window,     // window to query
            xcb::ATOM_WM_TRANSIENT_FOR,
            xcb::ATOM_WINDOW,
            0, 1,
        ).get_reply();
        match res {
            Ok(reply) => {
                debug!("Reply format is {}", reply.format());
                if reply.format() != 32 ||
                reply.type_() != xcb::ATOM_WINDOW ||
                reply.value_len() == 0 {
                    return None
                }
                return Some(reply.value()[0]) 
            }
            Err(e) => {
                warn!("Error: {}", e);
                return None 
            }
        }
    }

    fn get_wm_state(&self, window: XWindowID) -> WindowState {
        match icccm::get_wm_state(&self.conn, window).get_reply() {
            Ok(reply) => reply.state().into(),
            Err(_) => WindowState::default()
        }
    }

    fn get_urgency(&self, window: XWindowID) -> bool {
        if let Some(hints) = self.get_wm_hints(window) {
            return if let Some(u) = hints.is_urgent() {u} else {false}
        }
        false
    }
}