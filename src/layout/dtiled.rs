//todo: remove this once everything is done
#![allow(unused_variables)]

use crate::xserver::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::{Screen, Window, Geometry};
use crate::values;

pub fn activate(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
    
}

pub fn deactivate(conn: &XConn, ws: &mut Workspace) {

}

pub fn add_window(conn: &XConn, ws: &mut Workspace, _screen: &Screen, window_id: XWindowID) {
    let mut window = Window::from(window_id);
    let root_geom = conn.get_root_geom().expect("Could not get root geometry");

    if let Some(win) = ws.master() {

        if ws.is_empty() {
            fatal!("Workspace is empty but has a master window")
        }
        // the window we are trying to map is a slave window

        // if the master window is the only window
        // it takes up the entire screen
        if ws.windows.len() == 1 {
            // get a mutable reference to the master window
            let master = ws.windows.lookup_mut(win).unwrap();

            // cut the master in half
            let mut master_geom = master.xwindow.geom;
            master_geom.width = master_geom.width / 2;
            
            // update its geometry
            master.update_geometry(conn, master_geom);

            // now for the slave window

            // get its xy coords
            let (slave_x, slave_y) = {
                let left_corner = master_geom.x + master_geom.width;
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

            window.update_geometry(conn, slave_geom);
        } else { 
            // the master window is already chopped in half
            // and we need to update the slave windows instead
            // only update their height and xy coords

            // get no of slave windows
            let slave_count = ws.windows.len() - 1;

            // calculate new height of all slave windows
            let slave_height = root_geom.height / slave_count as i32;
        }



    } else {
        // if there is no master window, this should mean the workspace is empty
        // and we are mapping the master window
        if !ws.is_empty() {
            warn!("Workspace is not empty but has no master window");
        }
        window.update_geometry(conn, root_geom);

        //todo: account for windows like status bars, etc.
    }

    conn.configure_window(window.id(), &[(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, 5)]);

    conn.map_window(window.id());
    conn.change_window_attributes(window.id(), &values::child_events());

    ws.windows.push(window);
}

pub fn del_window() {

}

pub fn window_focus() {

}

pub fn toggle_floating() {

}