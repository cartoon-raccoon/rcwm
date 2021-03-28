pub mod floating;
pub mod dtiled;

use crate::xserver::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::Screen;
use crate::values;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutType {
    /// All-floating layout.
    Floating,
    /// Dynamic tiled, XMonad style.
    DTiled,
    /// Manual tiled, i3 style.
    MTiled,
}

fn set_focus_colour(conn: &XConn, window: XWindowID) {
    conn.change_window_attributes(window, &[(xcb::CW_BORDER_PIXEL, 0xff0000)]);
}

fn set_unfocus_colour(conn: &XConn, window: XWindowID) {
    conn.change_window_attributes(window, &[(xcb::CW_BORDER_PIXEL, 0xdddddd)]);
}

pub fn activate(conn: &XConn, ws: &mut Workspace, _screen: &Screen) {
    if ws.windows.is_empty() {
        return
    }

    for window in ws.windows.iter_rev() {
        // disable events
        conn.change_window_attributes(window.id(), &values::disable_events());
        // map window
        conn.map_window(window.id());
        // re-enable events
        conn.change_window_attributes(window.id(), &values::child_events());
    }
}

pub fn deactivate(conn: &XConn, ws: &mut Workspace) {
    for window in ws.windows.iter() {
        conn.change_window_attributes(window.id(), &values::disable_events());

        conn.unmap_window(window.id());

        conn.change_window_attributes(window.id(), &values::child_events());
    }
}