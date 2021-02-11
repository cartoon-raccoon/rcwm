use xcb::base::Connection;

fn main() {
    let xconn = Connection::connect(None)
        .expect("Failed to connect to X server");

    
}
