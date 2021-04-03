use crate::x::core::{XConn, XWindowID};
use crate::workspace::Workspace;
use crate::window::Client;
use crate::types::Geometry;
use crate::desktop::Screen;
use crate::values;

use super::BORDER_WIDTH;

pub fn activate(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
    function_ends!("[start] dtiled::activate");
    // we cannot use the base activate function here as it cannot
    // account for when a new window is sent to the workspace from another

    // if the workspace was previously empty but had a window sent to it
    // the len is > 0 and the master is none
    // if !ws.windows.is_empty() && ws.master.is_none() {
    //     // set the first window as the master
    //     debug!("Workspace contains window, but master is none - setting master");
    //     ws.set_master(ws.windows.get(0).unwrap(/*this unwrap should be safe*/).id());
    // }
    
    // reload the window layout
    let root_geom = conn.get_root_geom().expect("Could not get root geom");
    calculate_geoms(ws, screen, root_geom);
    // now that all the geometries are calculated, we can map the normal way
    super::activate(conn, ws, screen);
    function_ends!("[end] dtiled::activate");
}

pub fn deactivate(conn: &XConn, ws: &mut Workspace) {
    function_ends!("[start] dtiled::deactivate");
    // no special treatment needed for this one.
    super::deactivate(conn, ws);
    function_ends!("[end] dtiled::deactivate");
}

pub fn add_window(conn: &XConn, ws: &mut Workspace, screen: &Screen, window_id: XWindowID) {
    function_ends!("[start] dtiled::add_window");
    // Internally create a new window and set its supported protocols
    let mut window = Client::tiled(window_id, conn);
    window.set_supported(conn);

    // Get root geometries
    let root_geom = conn.get_root_geom().expect("Could not get root geometry");
    let root_geom2 = screen.xwindow.geom;

    if root_geom != root_geom2 {
        warn!("Stored geom and retrieved geom mismatch");
        debug!("gotten: {:?}", root_geom);
        debug!("stored: {:?}", root_geom2);
    }

    // Add new windows
    // If there is already a master, insert after it
    // Else, set new window as the master
    if ws.master.is_some() {
        debug!("dtiled::add_window: Pre-existing master, inserting after");
        ws.windows.append(window);
    } else {
        debug!("dtiled::add_window: No pre-existing master, pushing directly");
        ws.windows.push(window);
        ws.set_master(window_id);
    }

    // Calculate the tile sizes
    calculate_geoms(ws, screen, root_geom);

    for win in ws.windows.iter_mut() {
        win.update_geometry(conn);
        if win.id() == window_id {
            win.configure(conn, &[(xcb::CONFIG_WINDOW_BORDER_WIDTH as u16, BORDER_WIDTH)]);

            conn.map_window(win.id());
            win.configure(conn, &values::stack_above());
            win.change_attributes(conn, &values::child_events());
        }
    }
    window_focus(conn, ws, window_id);

    function_ends!("[end] dtiled::add_window");
}

pub fn del_window(
    conn: &XConn,
    ws: &mut Workspace,
    screen: &Screen,
    window_id: XWindowID,
    idx: usize
) -> Client {
    function_ends!("[start] dtiled::del_window");
    debug!("Got window with idx {}", idx);

    // internally remove window from tracking
    let window = ws.windows.pop(idx);

    // disable events and unmap the window
    conn.change_window_attributes(window_id, &values::disable_events());
    conn.unmap_window(window_id);
    ws.windows.unset_focused();

    // set new workspace master or unset it if empty
    if ws.is_master(window_id) {
        debug!("dtiled::del_window: Window to destroy is master, doing unmap checks");
        if ws.tiled_count() == 0 {
            debug!("dtiled::del_window: Workspace is now empty, unsetting master");
            ws.unset_master(); //workspace is now empty
            ws.windows.unset_focused();
        } else {
            debug!("dtiled::del_window: Workspace has {} tiled windows, setting new master", ws.tiled_count());
            let new_master = ws.windows.get(0).unwrap().id();
            debug!("New master is now {}", new_master);
            ws.set_master(new_master);
            debug!("Window at idx 0 is {:#?}", ws.windows.get(0));
            window_focus(conn, ws, new_master);
        }
    } else {
        // only master is left
        if ws.tiled_count() == 1 {
            let master = ws.master.unwrap();
            window_focus(conn, ws, master);        
        } else {
            if !ws.is_empty() {
                assert!(ws.tiled_count() > 1);
                //todo: add last focused so we can focus to that
                // placeholder code to focus to master by default
                let master = ws.master.unwrap();
                window_focus(conn, ws, master);
            } else {
                ws.windows.unset_focused();
            }
        }
    }

    // recalculate layouts
    relayout(conn, ws, screen);

    function_ends!("[end] dtiled::del_window");
    window

    //super::floating::del_window(conn, ws, screen, window_id, idx)
}

pub fn window_focus(conn: &XConn, ws: &mut Workspace, window: XWindowID) {
    // todo: placeholder
    super::floating::window_focus(conn, ws, window)
}

pub fn relayout(conn: &XConn, ws: &mut Workspace, screen: &Screen) {
    let root_geom = conn.get_root_geom().expect("Could not get root geom");
    calculate_geoms(ws, screen, root_geom);
    for win in ws.windows.iter_mut() {
        win.update_geometry(conn);
    }
}

fn calculate_geoms(ws: &mut Workspace, _screen: &Screen, root_geom: Geometry) {
    function_ends!("[start] dtiled::calculate_geoms");
    assert!(ws.is_tiling(), "calculate_geoms called while workspace is tiling");

    // Calculate the tile sizes
    if let Some(mstr) = ws.master() {
        if ws.tiled_count() == 0 {
            warn!("Workspace is empty but has a master window");
            function_ends!("[end] dtiled::calculate_geoms");
            ws.unset_master();
            return
        }
        // the window we are trying to map is a slave window

        // if the master window is the only window
        // it takes up the entire screen
        if ws.tiled_count() == 1 {
            debug!("dtiled::calculate_geoms: Only master exists, tiling to full window");
            // if there is no master window, this should mean the workspace is empty
            // and we are mapping the master window
            let master = ws.windows.lookup_mut(mstr).unwrap();

            //account for window border
            let master_geom = Geometry {
                x: root_geom.x,
                y: root_geom.y,
                height: root_geom.height - (BORDER_WIDTH as i32 * 2),
                width: root_geom.width - (BORDER_WIDTH as i32 * 2),
            };

            master.set_geometry(master_geom);

        } else if ws.tiled_count() == 2 {
            debug!("dtiled::calculate_geoms: 1 master + 1 slave, mapping half-half");

            // move master window to the front
            let master_idx = ws.windows.get_idx(mstr).unwrap();
            ws.windows.move_front(master_idx);
            
            // get a mutable reference to the master window
            let master = ws.windows.lookup_mut(mstr).unwrap();

            // cut the master in half
            let mut master_geom = root_geom;
            master_geom.height = root_geom.height - (BORDER_WIDTH as i32 * 2);
            master_geom.width = (
                (root_geom.width - BORDER_WIDTH as i32 * 2) / 2
            ) - BORDER_WIDTH as i32;
            
            // set its geometry
            master.set_geometry(master_geom);

            // now for the slave window

            // get its xy coords
            let (slave_x, slave_y) = {
                let left_corner = master_geom.x + master_geom.width + (BORDER_WIDTH as i32 *2);
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

            let slave = ws.windows.get_mut(1).unwrap();

            slave.set_geometry(slave_geom);
        } else { 
            debug!("dtiled::calculate_geoms: Multiple windows mapped, calculating");
            // the master window is already chopped in half
            // and we need to update the slave windows instead
            // only update their height and xy coords

            let master_idx = ws.windows.get_idx(mstr).unwrap();
            ws.windows.move_front(master_idx);

            // get a mutable reference to the master window
            let master = ws.windows.lookup_mut(mstr).unwrap();

            // get master's geometry
            let mut master_geom = master.xwindow.geom;

            if master_geom.x != 0 || master_geom.y != 0 {
                debug!("Master is not in position");

                master_geom.x = root_geom.x;
                master_geom.y = root_geom.y;
                master_geom.height = root_geom.height - (BORDER_WIDTH as i32 * 2);
                master_geom.width = (
                    (root_geom.width - BORDER_WIDTH as i32 * 2) / 2
                ) - BORDER_WIDTH as i32;

                master.set_geometry(master_geom);
            }

            // get no of slave windows
            let slave_count = if ws.tiled_count() == 0 { 0 } else { ws.tiled_count() - 1 };

            // calculate new height of all slave windows
            let slave_height = (root_geom.height / slave_count as i32) - (BORDER_WIDTH as i32 * 2);
            let slave_width = master_geom.width;

            // for each window that is not a master
            for (i, win) in ws.windows.iter_mut()
                .filter(|win| win.id() != mstr && win.is_tiled())
                .enumerate() {
                // calculate coords
                let y = i as i32 * slave_height + if i == 0 { 0 } else {
                    (BORDER_WIDTH as i32 * 2) * i as i32
                };
                let x = master_geom.width + (BORDER_WIDTH as i32 * 2);

                let slave_geom = Geometry {
                    x: x,
                    y: y,
                    height: slave_height,
                    width: slave_width,
                };

                dbg!(slave_geom);

                win.set_geometry(slave_geom);
            }

            
        }
    } else {
        if !ws.is_empty() {
            for window in ws.windows.iter() {
                assert!(window.is_floating(), "No master but window is tiled");
            }
        }
    }
    function_ends!("[end] dtiled::calculate_geoms");
}
