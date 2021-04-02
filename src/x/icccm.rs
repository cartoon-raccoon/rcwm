//! This module doesn't export anything.
//! It just contains methods for XConn that involve certain
//! aspects of ICCCM defined functionality.

use xcb_util::icccm;
use anyhow::Result;

use crate::x::{XConn, XWindowID};
use crate::types::XWinProperties;

impl<'a> XConn<'a> {
    pub fn get_client_properties(&self, window: XWindowID) -> Result<XWinProperties> {
        //todo: get wm_transient_for
        // fire off all requests at once, then get replies all together
        let wm_name_cookie = icccm::get_wm_name(&self.conn, window);
        let wm_icon_cookie = icccm::get_wm_icon_name(&self.conn, window);
        let wm_size_hints_cookie = icccm::get_wm_normal_hints(&self.conn, window);
        let wm_hints_cookie = icccm::get_wm_hints(&self.conn, window);
        let wm_class_cookie = icccm::get_wm_class(&self.conn, window);
        let wm_protocols_cookie = icccm::get_wm_protocols(&self.conn, window, self.atoms.WM_PROTOCOLS);
        let wm_state_cookie = icccm::get_wm_state(&self.conn, window);

        let wm_name = wm_name_cookie.get_reply()?.name().into();
        let wm_icon_name = wm_icon_cookie.get_reply()?.name().into();
        let wm_size_hints = wm_size_hints_cookie.get_reply()?;
        let wm_hints = wm_hints_cookie.get_reply()?;
        let wm_class = {
            let reply = wm_class_cookie.get_reply()?;
            (reply.instance().into(), reply.class().into())
        };
        let wm_protocols = wm_protocols_cookie.get_reply()?.atoms().to_owned();
        let wm_state = wm_state_cookie.get_reply()?.state().into();

        Ok(XWinProperties {
                wm_name: wm_name,
                wm_icon_name: wm_icon_name,
                wm_size_hints: wm_size_hints,
                wm_hints: wm_hints,
                wm_class: wm_class,
                wm_protocols: wm_protocols,
                wm_state: wm_state,
        })
    }

    pub fn get_wm_protocols(&self, window: XWindowID) -> Option<Vec<xcb::Atom>> {
        debug!("Getting protocols for window {}", window);
        match icccm::get_wm_protocols(self.conn, window, self.atoms.WM_PROTOCOLS)
        .get_reply() {
            Ok(reply) => {
                Some(reply.atoms().to_owned())
            }
            Err(_e) => {
                None
            }
        }
    }
}