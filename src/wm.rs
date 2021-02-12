use xcb_util::ewmh::Connection;

use crate::xserver::XConn;

pub struct WM<'a> {
    conn: XConn<'a>,
}

impl<'a> WM<'a> {
    pub fn register(conn: &'a Connection, screen_idx: i32) -> Self {
        let mut xconn = XConn::new(conn);

        Self {
            conn: xconn,
        }
    }
}