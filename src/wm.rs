use xcb_util::ewmh::Connection;

use crate::xserver::{XConn, ROOT_ATTRS};

pub struct WM<'a> {
    conn: XConn<'a>,
}

impl<'a> WM<'a> {
    pub fn register(conn: &'a Connection, screen_idx: i32) -> Self {
        let mut xconn = XConn::new(conn);

        let root_id = xconn.get_setup()
            .roots()
            .nth(screen_idx as usize)
            .expect("Could not get root")
            .root();

        xconn.change_window_attributes_checked(root_id, &ROOT_ATTRS);

        //xconn.set_supported(screen_idx, atoms: &[xcb::Atom]);

        Self {
            conn: xconn,
        }
    }
}