use crate::xserver::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::{Screen, Window, Geometry};
use crate::values;

use super::BORDER_WIDTH;

pub fn activate(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
    // we cannot use the base activate function here as it cannot
    // account for when a new window is sent to the workspace from another
    
    
    super::activate(conn, ws, screen)
}

pub fn deactivate(conn: &XConn, ws: &mut Workspace) {
    super::deactivate(conn, ws)
}

pub fn add_window(conn: &XConn, ws: &mut Workspace, screen: &Screen, window_id: XWindowID) {
    let mut window = Window::tiled(window_id);
    window.set_supported(conn);

    let root_geom = conn.get_root_geom().expect("Could not get root geometry");
    let root_geom2 = screen.xwindow.geom;

    if root_geom != root_geom2 {
        warn!("Stored geom and retrieved geom mismatch");
        debug!("gotten: {:?}", root_geom);
        debug!("stored: {:?}", root_geom2);
    }

    let pre_master = ws.master.is_some();

    if let Some(mstr) = ws.master() {
        debug!("Master exists, using tiling algo");

        if ws.is_empty() {
            error!("Workspace is empty but has a master window");
            return
        }
        // the window we are trying to map is a slave window

        // if the master window is the only window
        // it takes up the entire screen
        if ws.windows.len() == 1 {
            debug!("Only master window onscreen");
            // get a mutable reference to the master window
            let master = ws.windows.lookup_mut(mstr).unwrap();

            // cut the master in half
            let mut master_geom = master.xwindow.geom;
            master_geom.width = (master_geom.width / 2) - BORDER_WIDTH as i32;
            
            // update its geometry
            master.update_geometry(conn, master_geom);

            // now for the slave window

            // get its xy coords
            let (slave_x, slave_y) = {
                let left_corner = master_geom.x + master_geom.width;
                // todo: using 0 will not work when incorporating gaps and bars
                (left_corner, 0)
            };

            let slave_height = master_geom.height;
            let slave_width = master_geom.width;

            let slave_geom = Geometry {
                x: slave_x,
                y: slave_y,
                height: slave_height,
                width: slave_width,
            };

            dbg!(slave_geom);

            window.update_geometry(conn, slave_geom);
        } else { 
            // the master window is already chopped in half
            // and we need to update the slave windows instead
            // only update their height and xy coords

            // get a mutable reference to the master window
            let master = ws.windows.lookup_mut(mstr).unwrap();

            // get master's geometry
            let master_geom = master.xwindow.geom;

            // get no of slave windows
            let slave_count = ws.windows.len() - 1 /*master*/ + 1 /*new slave*/;

            // calculate new height of all slave windows
            let slave_height = root_geom.height / slave_count as i32;

            let mut idx = 0;
            let mut win_width = master_geom.width / 2;

            // for each window that is not a master
            for (i, win) in ws.windows.iter_mut().filter(|win| win.id() != mstr).enumerate() {
                // calculate coords
                let y = i as i32 * slave_height;
                let x = master_geom.width;

                let slave_geom = Geometry {
                    x: x,
                    y: y,
                    height: slave_height,
                    width: win.width(),
                };

                dbg!(slave_geom);

                win.update_geometry(conn, slave_geom);
                idx = i;
                win_width = win.width();
            }

            let new_win_geometry = Geometry {
                x: master_geom.width,
                y: (idx as i32 + 1) * slave_height,
                height: slave_height,
                width: win_width,
            };

            window.update_geometry(conn, new_win_geometry);
        }



    } else {
        debug!("No master exists, tiling to full window");
        // if there is no master window, this should mean the workspace is empty
        // and we are mapping the master window
        if !ws.is_empty() {
            warn!("Workspace is not empty but has no master window");
        }
        window.update_geometry(conn, root_geom);
        ws.set_master(window.id());

        //todo: account for windows like status bars, etc.
    }

    conn.configure_window(window.id(), &[(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, BORDER_WIDTH)]);

    conn.map_window(window.id());
    conn.change_window_attributes(window.id(), &values::child_events());

    if pre_master {
        debug!("Pre-existing master, inserting after");
        ws.windows.insert(1, window);
    } else {
        debug!("No pre-existing master, pushing directly");
        ws.windows.push(window);
    }
}

pub fn del_window(
    conn: &XConn,
    ws: &mut Workspace,
    screen: &Screen,
    window_id: XWindowID,
    idx: usize
) -> Window {
    //todo: placeholder
    debug!("Got window with idx {}", idx);
    let window = ws.windows.pop(idx);

    conn.change_window_attributes(window_id, &values::disable_events());
    conn.unmap_window(window_id);

    if ws.is_master(window_id) {
        debug!("Window to destroy is master, doing pre-unmap checks");
        if ws.windows.len() == 0 {
            debug!("Workspace is now empty, unsetting master");
            ws.unset_master(); //workspace is now empty
            ws.windows.unset_focused();
        } else {
            debug!("Workspace has {} windows, setting new master", ws.windows.len());
            let new_master = ws.windows.get(0).unwrap().id();
            debug!("New master is now {}", new_master);
            ws.set_master(new_master);
            debug!("Window at idx 0 is {:?}", ws.windows.get(0));
            window_focus(conn, ws, new_master);
        }
    }

    relayout(conn, ws, screen);

    window

    //super::floating::del_window(conn, ws, screen, window_id, idx)
}

pub fn window_focus(conn: &XConn, ws: &mut Workspace, window: XWindowID) {
    // todo: placeholder
    super::floating::window_focus(conn, ws, window)
}

pub fn toggle_floating() {

}

pub fn relayout(conn: &XConn, ws: &mut Workspace, screen: &Screen) {

}