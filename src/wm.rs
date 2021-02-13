use xcb_util::ewmh::Connection;

use crate::xserver::{XConn, ROOT_ATTRS};

#[derive(Clone, Copy)]
pub struct WM<'a> {
    conn: XConn<'a>,
}

impl<'a> WM<'a> {
    pub fn register(conn: &'a Connection, screen_idx: i32) -> Self {
        let xconn = XConn::new(conn);

        let root_id = xconn.get_setup()
            .roots()
            .nth(screen_idx as usize)
            .expect("Could not get root")
            .root();

        // we panic here because this is a fatal error
        // register for substructure redirect and substructure notify on root window 
        xconn.change_window_attributes_checked(root_id, &ROOT_ATTRS)
            .unwrap_or_else(|e| panic!("{}", e));

        xconn.set_supported(screen_idx, &[
            xconn.atoms.WM_PROTOCOLS, 
            xconn.atoms.WM_DELETE_WINDOW
        ]);

        Self {
            conn: xconn,
        }
    }

    //using a mutable reference statically ensures there is only one instance running
    pub fn run(&mut self) -> ! {
        loop {
            let event = self.conn.next_event();

            match event.response_type() & !0x80 {
                _ => {
                    //handle event
                }
            }
        }
    }
}