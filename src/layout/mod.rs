pub mod floating;
pub mod dtiled;

use crate::xserver::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::Screen;
use crate::values;

pub const BORDER_WIDTH: u32 = 2;

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

fn window_stack_and_focus(ws: &mut Workspace, conn: &XConn, window: XWindowID) {
    // disable events
    conn.change_window_attributes(window, &values::disable_events());

    // if there is a focused window, stack it above
    if let Some(win) = ws.windows.focused() {
        conn.configure_window(window, &values::stack_above_sibling(win.id()));
    }

    // focus to current window
    conn.set_input_focus(window);
    set_focus_colour(conn, window);

    // re-enable events
    conn.change_window_attributes(window, &values::child_events());
}

/// The base activate function.
/// 
/// Sequentially maps every window to the screen.
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

/// The base deactivate function.
/// 
/// Sequentially unmaps every window in reverse.
pub fn deactivate(conn: &XConn, ws: &mut Workspace) {
    for window in ws.windows.iter() {
        conn.change_window_attributes(window.id(), &values::disable_events());

        conn.unmap_window(window.id());

        conn.change_window_attributes(window.id(), &values::child_events());
    }
}