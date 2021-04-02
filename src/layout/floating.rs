use crate::x::core::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::{Window};
use crate::desktop::Screen;
use crate::values;

use super::{set_unfocus_colour, BORDER_WIDTH};

pub fn activate(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
    super::activate(conn, ws, screen)
}

pub fn deactivate(conn: &XConn, ws: &mut Workspace) {
    super::deactivate(conn, ws)
}

pub fn add_window(conn: &XConn, ws: &mut Workspace, screen: &Screen, window_id: XWindowID) {
    let mut window = Window::floating(window_id, conn);

    window.set_supported(conn);

    conn.map_window(window.id());

    if let Some(_focused) = ws.windows.focused() {
        conn.configure_window(window_id, &values::stack_above());
    }
    conn.configure_window(window.id(), &[(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, BORDER_WIDTH)]);

    window.xwindow.set_geometry_conn(conn);

    match conn.query_pointer(screen.xwindow.id) {
        Ok(pointer) => {
            if pointer.child() == screen.xwindow.id || pointer.child() == window_id {
                window_focus(conn, ws, window_id);
            } else {
                if let Some(focused) = ws.windows.focused() {
                    set_unfocus_colour(conn, focused.id());
                    window_focus(conn, ws, window_id);
                } else {
                    set_unfocus_colour(conn, window_id);
                }
            }
        }
        Err(e) => {
            error!("{}", e)
        }
    }
    
    conn.change_window_attributes(window.id(), &values::child_events());

    ws.windows.push(window);
}

#[allow(mutable_borrow_reservation_conflict)]
//* this ^^^ should be fixed
pub fn del_window(
    conn: &XConn, 
    ws: &mut Workspace, 
    _screen: &Screen, 
    window_id: XWindowID,
    idx: usize
) ->  Window {
    let window = ws.windows.pop(idx);

    conn.change_window_attributes(window_id, &values::disable_events());

    conn.unmap_window(window_id);

    if idx == 0 {
        if let Some(next) = ws.windows.get(0) {
            //todo: fix immutable borrow thing
            super::window_stack_and_focus(ws, conn, next.id());
        }
    }

    if ws.is_empty() {
        ws.windows.unset_focused();
    }

    window
}

pub fn window_focus(conn: &XConn, ws: &mut Workspace, window: XWindowID) {

    if let Some(idx) = ws.windows.contains(window) {
        if let Some(focused) = ws.windows.focused() {
            set_unfocus_colour(conn, focused.id());
        }
        // internally focus
        ws.windows.set_focused_by_idx(idx);
        
        // tell x to focus
        super::window_stack_and_focus(ws, conn, window)
    }
}

pub fn relayout(_conn: &XConn, _ws: &mut Workspace, _screen: &Screen) {
    // we do nothing because relayout is not needed
}