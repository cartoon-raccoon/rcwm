//! This module exports Client and ClientRing, which encapsulate
//! data about windows and manage them internally respectively.

use std::collections::HashSet;

use crate::x::core::{
    XConn,
    XWindow, 
    XWindowID
};
use crate::x::Icccm;
use crate::utils;
use crate::types::{WinLayoutState, Geometry, Ring, Selector};
use crate::desktop::Screen;

pub const WIN_HEIGHT_MIN: i32 = 100;
pub const WIN_WIDTH_MIN: i32 = 100;
pub const MIN_ONSCREEN: i32 = 20;

#[allow(dead_code)]
fn ensure_in_bounds(val: &mut i32, min: i32, max: i32) {
    if *val < min {
        *val = min;
    } else if *val > max {
        *val = max;
    }
}

/// A Ring of type Client.
///
/// Contains additional methods more specific to window management.
pub type ClientRing = Ring<Client>;

impl ClientRing {

    pub fn remove_by_id(&mut self, id: XWindowID) -> Option<Client> {
        if let Some(i) = self.get_idx(id) {
            self.remove(i)
        } else {
            None
        }
    }

    pub fn get_idx(&self, id: XWindowID) -> Option<usize> {
        self.index(Selector::Condition(&|win| win.id() == id))
    }

    pub fn lookup(&self, id: XWindowID) -> Option<&Client> {
        if let Some(i) = self.get_idx(id) {
            self.get(i)
        } else {
            None
        }
    }

    pub fn lookup_mut(&mut self, id: XWindowID) -> Option<&mut Client> {
        if let Some(i) = self.get_idx(id) {
            self.get_mut(i)
        } else {
            None
        }
    }

    pub fn contains(&mut self, id: XWindowID) -> bool {
        for win in self.items.iter() {
            if win.id() == id {
                return true
            }
        }
        false
    }

    pub fn set_focused_by_winid(&mut self, id: XWindowID) {
        if let Some(i) = self.get_idx(id) {
            self.focused = Some(i)
        } else {
            error!("Tried to focus a client not in the workspace")
        }
    }

    pub fn set_focused_by_idx(&mut self, idx: usize) {
        self.set_focused(idx);
    }

    pub fn is_focused(&self, id: XWindowID) -> bool {
        if let Some(window) = self.focused() {
            return window.id() == id
        } else {
            false
        }
    }
}

/// Various metadata about a X window.
/// 
/// Contains ICCCM and EWMH properties.
#[derive(Debug, Clone)]
pub struct Client {
    pub xwindow: XWindow,
    pub name: String,
    pub icon_name: String,
    
    state: WinLayoutState,
    protocols: HashSet<xcb::Atom>,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.xwindow.id == other.xwindow.id
    }
}

//todo: fix your calculations, they are deeply broken.
impl Client {
    pub fn tiled(from: XWindowID, conn: &XConn) -> Self {
        Self::new(from, conn, WinLayoutState::Tiled)
    }

    pub fn floating(from: XWindowID, conn: &XConn) -> Self {
        Self::new(from, conn, WinLayoutState::Floating)
    }

    fn new(from: XWindowID, conn: &XConn, layout: WinLayoutState) -> Self {
        let properties = conn.get_client_properties(from);
        Self {
            xwindow: XWindow::from(from),
            state: layout,
            name: properties.wm_name,
            icon_name: properties.wm_icon_name,
            protocols: HashSet::new(),
        }
    }

    #[inline]
    pub fn is_tiled(&self) -> bool {
        if let WinLayoutState::Tiled = self.state {
            return true
        }
        false
    }

    #[inline]
    pub fn is_floating(&self) -> bool {
        if let WinLayoutState::Floating = self.state {
            return true
        }
        false
    }

    #[inline]
    pub fn set_tiled(&mut self) {
        self.state = WinLayoutState::Tiled
    }

    #[inline]
    pub fn set_floating(&mut self) {
        self.state = WinLayoutState::Floating
    }

    #[inline]
    pub fn toggle_state(&mut self) {
        if let WinLayoutState::Floating = self.state {
            self.state = WinLayoutState::Tiled
        } else if let WinLayoutState::Tiled = self.state {
            self.state = WinLayoutState::Floating
        }
    }

    #[inline(always)]
    pub fn id(&self) -> XWindowID {
        self.xwindow.id
    }

    #[inline(always)]
    pub fn x(&self) -> i32 {
        self.xwindow.geom.x
    }

    #[inline(always)]
    pub fn y(&self) -> i32 {
        self.xwindow.geom.y
    }

    #[inline(always)]
    pub fn height(&self) -> i32 {
        self.xwindow.geom.height
    }

    #[inline(always)]
    pub fn width(&self) -> i32 {
        self.xwindow.geom.width
    }
    
    /// Configure the `Client` using a provided connection
    /// 
    /// Use `Client::set_geometry` and `Client::update_geometry`
    /// to change client geometry instead of this method.
    pub fn configure(&self, conn: &XConn, attrs: &[(u16, u32)]) {
        conn.configure_window(self.id(), attrs);
    }

    /// Change client attributes.
    pub fn change_attributes(&self, conn: &XConn, attrs: &[(u32, u32)]) {
        conn.change_window_attributes(self.id(), attrs)
    }

    /// Resize the window using _changes_ in height and width.
    /// 
    /// Does not do bounds checking.
    pub fn do_resize(&mut self, conn: &XConn, _scr: &Screen, dx: i32, dy: i32) {
        self.xwindow.update_height(dy);
        self.xwindow.update_width(dx);

        // let scrx = scr.xwindow.geom.x;
        // let scry = scr.xwindow.geom.y;
        // let scrh = scr.xwindow.geom.height;
        // let scrw = scr.xwindow.geom.width;

        // ensure_in_bounds(
        //     &mut self.xwindow.geom.height, 
        //     WIN_HEIGHT_MIN, scry + scrh - self.xwindow.geom.y);
        // ensure_in_bounds(&mut self.xwindow.geom.width, 
        //     WIN_WIDTH_MIN, scrx + scrw - self.xwindow.geom.x);

        conn.configure_window(self.xwindow.id, &utils::configure_resize(
            self.width() as u32, 
            self.height() as u32
        ));

        // debug!(
        //     "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
        //     self.x(), self.y(), self.height(), self.width()
        // );
    }

    /// Move the window using _changes_ in window coordinates.
    /// 
    /// Does not do bounds checking.
    pub fn do_move(&mut self, conn: &XConn, _scr: &Screen, dx: i32, dy: i32) {
        self.xwindow.update_pos_y(dy);
        self.xwindow.update_pos_x(dx);

        // let scrx = scr.xwindow.geom.x;
        // let scry = scr.xwindow.geom.y;
        // let scrh = scr.xwindow.geom.height;
        // let scrw = scr.xwindow.geom.width;

        // ensure_in_bounds(&mut self.xwindow.geom.x, 
        //     scrx - self.xwindow.geom.width + MIN_ONSCREEN, 
        //     scrx + scrw - MIN_ONSCREEN);
        // ensure_in_bounds(&mut self.xwindow.geom.y, 
        //     scry - self.xwindow.geom.height + MIN_ONSCREEN, 
        //     scry + scrh - MIN_ONSCREEN);

        conn.configure_window(self.xwindow.id, &utils::configure_move(
            self.x() as u32, 
            self.y() as u32
        ));

        // debug!(
        //     "Updated geometry:\nx: {}, y: {}, h: {}, w: {}", 
        //     self.x(), self.y(), self.height(), self.width()
        // );
    }

    /// Sets the geometry of the window, but does not update it to the X server.
    pub fn set_geometry(&mut self, geom: Geometry) {
        self.xwindow.set_geometry(geom);
    }

    /// Updates its geometry on the X server.
    /// 
    /// Normally called after `Client::set_geometry`.
    pub fn update_geometry(&self, conn: &XConn) {
        conn.configure_window(self.xwindow.id, &utils::configure_resize(
            self.width() as u32,
            self.height() as u32,
        ));

        conn.configure_window(self.xwindow.id, &utils::configure_move(
            self.x() as u32,
            self.y() as u32,
        ))
    }

    /// Updates and sets the Client geometry with a given Geometry.
    pub fn set_and_update_geometry(&mut self, conn: &XConn, geom: Geometry) {
        self.set_geometry(geom);
        self.update_geometry(conn);
    }

    /// Sets the supported protocols for the client.
    pub fn set_supported(&mut self, conn: &XConn) {
        if let Some(protocols) = conn.get_wm_protocols(self.id()) {
            for protocol in protocols {
                self.protocols.insert(protocol);
            }
        }
    }

    /// Does the client support this protocol?
    pub fn supports(&self, prtcl: xcb::Atom) -> bool {
        self.protocols.contains(&prtcl)
    }
}
