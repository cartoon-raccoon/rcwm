pub mod floating;
pub mod dtiled;

use crate::x::core::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::desktop::Screen;
use crate::values;

pub const BORDER_WIDTH: u32 = 2;

#[non_exhaustive]
#[allow(dead_code)]
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

fn window_stack_and_focus(_ws: &mut Workspace, conn: &XConn, window: XWindowID) {
    // disable events
    conn.change_window_attributes(window, &values::disable_events());

    // if there is a focused window, stack it above
    // if let Some(win) = ws.windows.focused() {
    //     debug!("Focusing window {}", win.id());
    //     conn.configure_window(window, &values::stack_above(win.id()));
    // }

    // focus to current window
    conn.set_input_focus(window);
    set_focus_colour(conn, window);

    // re-enable events
    conn.change_window_attributes(window, &values::child_events());
}

/// The base activate function.
/// 
/// Sequentially maps every window to the screen.
pub fn activate(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
    if ws.windows.is_empty() {
        return
    }

    // focus the main window in the workspace
    // if floating, focus the first window
    // else (should be tiled), focus the master window
    if let LayoutType::Floating = ws.layout {
        assert!(ws.master.is_none());
        if !ws.is_empty() {
            ws.focus_window(conn, screen, ws.windows.get(0).unwrap().id());
        }
    } else {
        debug!("Master is {:?}", ws.master);
        if let Some(win) = ws.master {
            ws.focus_window(conn, screen, win);
        }
    }

    for window in ws.windows.iter_rev() {
        // disable events
        window.change_attributes(conn, &values::disable_events());
        // update window geometry in the x server
        window.update_geometry(conn);
        // map window
        conn.map_window(window.id());
        // re-enable events
        window.change_attributes(conn, &values::child_events());
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