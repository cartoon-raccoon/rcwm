use crate::xserver::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::{Window, Screen};
use crate::values;

pub fn activate(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
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
    unimplemented!()
}

pub fn add_window(conn: &XConn, ws: &mut Workspace, screen: &Screen, window: XWindowID) {
    let mut window = Window::from(window);

    window.set_supported(conn);

    conn.map_window(window.id());


}

pub fn del_window(
    conn: &XConn, 
    ws: &mut Workspace, 
    screen: &Screen, 
    window: XWindowID,
    idx: usize
) ->  Window {
    unimplemented!()

}

pub fn window_focus(conn: &XConn, ws: &mut Workspace, window: XWindowID) {

    if let Some(idx) = ws.windows.contains(window) {
        // internally focus
        ws.windows.move_front(idx);

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

    // re-enable events
    conn.change_window_attributes(window, &values::child_events());
}