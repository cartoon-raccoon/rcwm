use crate::xserver::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::{Window, Screen};
use crate::values;

use super::{set_focus_colour, set_unfocus_colour};

pub fn add_window(conn: &XConn, ws: &mut Workspace, screen: &Screen, window_id: XWindowID) {
    let mut window = Window::from(window_id);

    window.set_supported(conn);

    conn.map_window(window.id());

    if let Some(focused) = ws.windows.focused() {
        conn.configure_window(window_id, &values::stack_above_sibling(focused.id()));
    }
    conn.configure_window(window.id(), &[(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 5)]);

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
            window_stack_and_focus(ws, conn, next.id());
        }
    }

    window
}

pub fn window_focus(conn: &XConn, ws: &mut Workspace, window: XWindowID) {

    if let Some(idx) = ws.windows.contains(window) {
        if let Some(focused) = ws.windows.focused() {
            set_unfocus_colour(conn, focused.id());
        }
        // internally focus
        ws.windows.move_front(idx);
        
        debug!("Moving window {} to the front",  window);
        debug!("{}", ws.windows[0].id());

        // tell x to focus
        window_stack_and_focus(ws, conn, window)
    }
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