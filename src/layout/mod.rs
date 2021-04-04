//! Layout types for defining the layout style used by RaccoonWM.

pub(crate) mod floating;
pub(crate) mod dtiled;

use crate::x::core::{XConn, XWindowID};
use crate::types::{Direction, BorderStyle};
use crate::workspace::Workspace;
use crate::desktop::Screen;
use crate::utils;

pub(crate) use crate::config::BORDER_WIDTH;

/// The layouts that RaccoonWM can tile windows in.
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

fn window_stack_and_focus(ws: &mut Workspace, conn: &XConn, window: XWindowID) {
    use BorderStyle::*;
    // disable events
    conn.change_window_attributes(window, &utils::disable_events());

    let win = ws.windows.lookup_mut(window).unwrap();

    // if there is a focused window, stack it above
    // if let Some(win) = ws.windows.focused() {
    //     debug!("Focusing window {}", win.id());
    //     conn.configure_window(window, &utils::stack_above(win.id()));
    // }

    
    // focus to current window
    win.set_border(conn, Focused);
    conn.set_input_focus(window);

    // re-enable events
    conn.change_window_attributes(window, &utils::child_events());
}

/// The base activate function.
/// 
/// Sequentially maps every window to the screen.
pub(crate) fn activate(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
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
        window.change_attributes(conn, &utils::disable_events());
        // update window geometry in the x server
        window.update_geometry(conn);
        // map window
        conn.map_window(window.id());
        // re-enable events
        window.change_attributes(conn, &utils::child_events());
    }
}

/// The base deactivate function.
/// 
/// Sequentially unmaps every window in reverse.
pub(crate) fn deactivate(conn: &XConn, ws: &mut Workspace) {
    for window in ws.windows.iter() {
        conn.change_window_attributes(window.id(), &utils::disable_events());

        conn.unmap_window(window.id());

        conn.change_window_attributes(window.id(), &utils::child_events());
    }
}

pub(crate) fn cycle_focus(conn: &XConn, ws: &mut Workspace, direction: Direction) {
    use BorderStyle::*;

    //change currently focused border colour to unfocused
    if let Some(win) = ws.windows.focused_mut() {
        win.set_border(conn, Unfocused);
    }
    
    //internally, cycle focus
    ws.windows.cycle_focus(direction);

    // change focus colours
    if ws.windows.focused().is_some() {
        let focused = ws.windows.focused().unwrap().id();

        window_stack_and_focus(ws, conn, focused);
    }
}