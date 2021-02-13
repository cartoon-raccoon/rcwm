use xcb_util::ewmh;
use anyhow::{Context, Result};

use crate::window::Geometry;

pub const ROOT_ATTRS: [(u32, u32); 1] = [
    (
        xcb::CW_EVENT_MASK, 
        xcb::EVENT_MASK_SUBSTRUCTURE_REDIRECT | xcb::EVENT_MASK_STRUCTURE_NOTIFY
    )
];

pub type XWindowID = xcb::Window;

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

pub struct XConn<'a> {
    pub conn: &'a ewmh::Connection,
}

impl<'a> XConn<'a> {
    pub fn new(xconn: &'a ewmh::Connection) -> Self {
        Self {
            conn: xconn,
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