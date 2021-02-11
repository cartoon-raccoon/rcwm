use xcb::base::Connection;
use xcb_util::ewmh;

fn main() {
    let (conn, screen_idx) = Connection::connect(None)
        .expect("Failed to connect to X server");
    
    let xconn = ewmh::Connection::connect(conn)
        .map_err(|(err, _)| { err })
        .expect("Failed to connect via EWMH");

    
    
}
