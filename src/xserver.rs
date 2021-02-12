use xcb_util::ewmh;

pub struct XConn<'a> {
    pub conn: &'a ewmh::Connection,
}

impl<'a> XConn<'a> {
    pub fn new(xconn: &'a ewmh::Connection) -> Self {
        Self {
            conn: xconn,
        }
    }
}