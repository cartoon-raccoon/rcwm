pub mod floating;

use crate::xserver::{XConn, XWindowID};

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutType {
    Floating,
    Tiled,
}

fn set_focus_colour(conn: &XConn, window: XWindowID) {
    conn.change_window_attributes(window, &[(xcb::CW_BORDER_PIXEL, 0xff0000)]);
}

fn set_unfocus_colour(conn: &XConn, window: XWindowID) {
    conn.change_window_attributes(window, &[(xcb::CW_BORDER_PIXEL, 0xdddddd)]);
}