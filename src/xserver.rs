use xcb_util::ewmh;
use anyhow::{Context, Result};

use std::ops::Index;

use crate::window::Geometry;

pub const ROOT_ATTRS: [(u32, u32); 1] = [
    (
        xcb::CW_EVENT_MASK, 
        xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_STRUCTURE_NOTIFY
    )
];

/// The list of atoms interned from the X Server by the WM.
#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct InternedAtoms {
    pub SUPPORTED: xcb::Atom,
    
    pub WM_DELETE_WINDOW: xcb::Atom,

    pub WM_PROTOCOLS: xcb::Atom,

    pub WM_WINDOW_TYPE_NORMAL: xcb::Atom,
    pub WM_WINDOW_TYPE_DIALOG: xcb::Atom,
    pub WM_WINDOW_TYPE_TOOLBAR: xcb::Atom,
    pub WM_WINDOW_TYPE_UTILITY: xcb::Atom,
    pub WM_WINDOW_TYPE_SPLASH: xcb::Atom,
    pub WM_STATE: xcb::Atom,
}

/// You really shouldn't be using this.
impl Index<usize> for InternedAtoms {
    type Output = xcb::Atom;

    fn index(&self, idx: usize) -> &xcb::Atom {
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
            
            WM_PROTOCOLS: conn.WM_PROTOCOLS(),

            WM_WINDOW_TYPE_NORMAL: conn.WM_WINDOW_TYPE_NORMAL(),
            WM_WINDOW_TYPE_DIALOG: conn.WM_WINDOW_TYPE_DIALOG(),
            WM_WINDOW_TYPE_TOOLBAR: conn.WM_WINDOW_TYPE_TOOLBAR(),
            WM_WINDOW_TYPE_UTILITY: conn.WM_WINDOW_TYPE_UTILITY(),
            WM_WINDOW_TYPE_SPLASH: conn.WM_WINDOW_TYPE_SPLASH(),

            WM_STATE: conn.WM_STATE(),
        }
    }
}

pub type XWindowID = xcb::Window;

#[derive(Debug, Clone, Copy)]
pub struct XWindow {
    pub id: XWindowID,
    pub geom: Geometry,
}

impl PartialEq for XWindow {
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

#[derive(Clone, Copy)]
pub struct XConn<'a> {
    pub conn: &'a ewmh::Connection,
    pub atoms: InternedAtoms,
}

impl<'a> XConn<'a> {
    pub fn new(xconn: &'a ewmh::Connection) -> Self {
        Self {
            conn: xconn,
            atoms: InternedAtoms::new(xconn),
        }
    }

    pub fn get_setup(&self) -> xcb::Setup {
        self.conn.get_setup()
    }

    pub fn change_window_attributes(&self, window: XWindowID, attrs: &[(u32, u32)]) {
        xcb::change_window_attributes(self.conn, window, attrs);
    }

    pub fn change_window_attributes_checked(&self, window: XWindowID, attrs: &[(u32, u32)]) -> Result<()> {
        xcb::change_window_attributes_checked(self.conn, window, attrs).request_check()
            .with_context(|| String::from("Could not change window attributes"))
    }

    pub fn set_supported(&self, screen_idx: i32, atoms: &[xcb::Atom]) {
        ewmh::set_supported(self.conn, screen_idx, atoms);
    }

    pub fn get_geometry(&self, window_id: XWindowID) -> Result<Geometry> {
        xcb::get_geometry(self.conn, window_id).get_reply()
            .with_context(|| {
                String::from("Failed to get window geometry")
            })
            .map(|ok| Geometry::from(
                (
                    ok.x() as i32, 
                    ok.y() as i32, 
                    ok.width() as i32, 
                    ok.height() as i32,
                )
            ))
    }
}