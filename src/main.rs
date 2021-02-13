use xcb::base::Connection;
use xcb_util::ewmh;

mod wm;
mod xserver;
mod window;

use wm::WM;

/*
 * Step 1: Get root window ID
 * Step 2: Register substructure redirection on root window
 * Step 3: Register atoms
 * Step 4: Register whatever mouse events
*/

fn main() {
    let (conn, screen_idx) = Connection::connect(None)
        .expect("Failed to connect to X server");
    
    let conn = ewmh::Connection::connect(conn)
        .map_err(|(err, _)| { err })
        .expect("Failed to connect via EWMH");

    let wm = WM::register(&conn, screen_idx);
}
