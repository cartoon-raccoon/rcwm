use std::ops::Index;

use xcb_util::ewmh;

use crate::types::Geometry;

pub use super::xserver::XCBConnection;
pub use super::event::*;

pub type Atom = xcb::Atom;

/// The list of atoms interned from the X Server by the WM.
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct InternedAtoms {
    pub SUPPORTED: Atom,
    
    pub WM_DELETE_WINDOW: Atom,
    pub WM_TAKE_FOCUS: Atom,

    pub WM_PROTOCOLS: Atom,

    pub WM_WINDOW_TYPE_DESKTOP: Atom,
    pub WM_WINDOW_TYPE_DOCK: Atom,
    pub WM_WINDOW_TYPE_TOOLBAR: Atom,
    pub WM_WINDOW_TYPE_MENU: Atom,
    pub WM_WINDOW_TYPE_UTILITY: Atom,
    pub WM_WINDOW_TYPE_SPLASH: Atom,
    pub WM_WINDOW_TYPE_DIALOG: Atom,
    pub WM_WINDOW_TYPE_DROPDOWN_MENU: Atom,
    pub WM_WINDOW_TYPE_NOTIFICATION: Atom,
    pub WM_WINDOW_TYPE_NORMAL: Atom,
    pub WM_STATE: Atom,
}

/// You really shouldn't be using this.
impl Index<usize> for InternedAtoms {
    type Output = Atom;

    fn index(&self, idx: usize) -> &Atom {
        match idx {
            0 => &self.SUPPORTED,
            1 => &self.WM_DELETE_WINDOW,
            2 => &self.WM_PROTOCOLS,
            3 => &self.WM_WINDOW_TYPE_NORMAL,
            4 => &self.WM_WINDOW_TYPE_DIALOG,
            5 => &self.WM_WINDOW_TYPE_TOOLBAR,
            6 => &self.WM_WINDOW_TYPE_UTILITY,
            7 => &self.WM_WINDOW_TYPE_SPLASH,
            8 => &self.WM_STATE,
            // yeah, if you index anything more than 8, you'll probably get a null pointer.
            _ => &0,
        }
    }
}

impl InternedAtoms {
    pub fn new(conn: &ewmh::Connection) -> Self {
        Self {
            SUPPORTED: conn.SUPPORTED(),

            WM_DELETE_WINDOW: xcb::intern_atom(conn, false, "WM_DELETE_WINDOW")
                .get_reply()
                .unwrap_or_else(|e| panic!("{}", e))
                .atom(),

            WM_TAKE_FOCUS: xcb::intern_atom(conn, false, "WM_TAKE_FOCUS")
                .get_reply()
                .unwrap_or_else(|e| panic!("{}", e))
                .atom(),
            
            WM_PROTOCOLS: conn.WM_PROTOCOLS(),
            
            WM_WINDOW_TYPE_DESKTOP: conn.WM_WINDOW_TYPE_DESKTOP(),
            WM_WINDOW_TYPE_DOCK: conn.WM_WINDOW_TYPE_DOCK(),
            WM_WINDOW_TYPE_TOOLBAR: conn.WM_WINDOW_TYPE_TOOLBAR(),
            WM_WINDOW_TYPE_MENU: conn.WM_WINDOW_TYPE_MENU(),
            WM_WINDOW_TYPE_UTILITY: conn.WM_WINDOW_TYPE_UTILITY(),
            WM_WINDOW_TYPE_SPLASH: conn.WM_WINDOW_TYPE_SPLASH(),
            WM_WINDOW_TYPE_DIALOG: conn.WM_WINDOW_TYPE_DIALOG(),
            WM_WINDOW_TYPE_DROPDOWN_MENU: conn.WM_WINDOW_TYPE_DROPDOWN_MENU(),
            WM_WINDOW_TYPE_NOTIFICATION: conn.WM_WINDOW_TYPE_NOTIFICATION(),
            WM_WINDOW_TYPE_NORMAL: conn.WM_WINDOW_TYPE_NORMAL(),

            WM_STATE: conn.WM_STATE(),
        }
    }
}

pub type XWindowID = xcb::Window;

/// A low-level window type that contains a window ID and its geometry.
#[derive(Debug, Clone, Copy)]
pub struct XWindow {
    pub id: XWindowID,
    pub geom: Geometry,
}

impl PartialEq for XWindow {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl From<XWindowID> for XWindow {
    fn from(from: XWindowID) -> Self {
        Self {
            id: from,
            geom: Geometry::from((0,0,0,0))
        }
    }
}

impl XWindow {
    /// Sets the geometry using a provided XCBConnection.
    pub fn set_geometry_conn(&mut self, conn: &XCBConnection) {
        match conn.get_geometry(self.id) {
            Ok(geom) => {
                debug!(
                    "Updating geometry (conn):\nx: {}, y: {}, h: {}, w: {}", 
                    geom.x, geom.y, geom.height, geom.width
                );
                self.geom = geom;
            }

            Err(e) => {
                error!("{}", e);
                return
            }
        }
    }

    /// Sets the geometry using a provided Geometry.
    pub fn set_geometry(&mut self, geom: Geometry) {
        debug!(
            "Updating geometry for window {}:\nx: {}, y: {}, h: {}, w: {}", 
            self.id, geom.x, geom.y, geom.height, geom.width
        );
        self.geom = geom;
    }
    /// Updates the width by a given difference.
    pub fn update_width(&mut self, dx: i32) {
        self.geom.width += dx;
    }
    /// Updates the height by given difference.
    pub fn update_height(&mut self, dy: i32) {
        self.geom.height += dy;
    }
    /// Updates the x coordinate of the window by a given difference.
    pub fn update_pos_x(&mut self, dx: i32) {
        self.geom.x += dx;
    }
    /// Updates the y coordinate of the window by a given difference.
    pub fn update_pos_y(&mut self, dy: i32) {
        self.geom.y += dy;
    }
}