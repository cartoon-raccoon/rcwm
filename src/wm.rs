use xcb_util::{
    ewmh::Connection,
    cursor,
};

use crate::values::ROOT_ATTRS;
use crate::xserver::{XConn};

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct WM<'a> {
    conn: XConn<'a>,
    root: i32,
}

impl<'a> WM<'a> {
    pub fn register(conn: &'a Connection, screen_idx: i32) -> Self {
        let mut xconn = XConn::new(conn);

        let root_id = xconn.get_setup()
            .roots()
            .nth(screen_idx as usize)
            .expect("Could not get root")
            .root();

        // register for substructure redirect and substructure notify on root window 
        xconn.change_window_attributes_checked(root_id, &ROOT_ATTRS)
            // we panic here because this is a fatal error
            .unwrap_or_else(|_e| {
                error!("Another window manager is running.");
                std::process::exit(1)
            });

        xconn.set_supported(screen_idx, &[
            xconn.atoms.WM_PROTOCOLS, 
            xconn.atoms.WM_DELETE_WINDOW
        ]);

        // fatal because this is the WM setup process
        xconn.create_cursor(cursor::LEFT_PTR)
            .unwrap_or_else(|_e| {
                fatal!("Could not create cursor")
            });

        xconn.set_cursor(root_id);

        let new = Self {
            conn: xconn,
            root: screen_idx,
        };

        // for existing in &new.conn.query_tree(root_id).unwrap() {

        // }

        new

    }

    //using a mutable reference statically ensures there is only one instance running
    pub fn run(&mut self) -> ! {
        info!("Running WM");
        
        loop {
            let event = self.conn.next_event();

            unsafe {
                match event.response_type() & !0x80 {
                    xcb::MAP_REQUEST => self.on_map_request(xcb::cast_event(&event)),
                    unhandled => {
                        debug!("Unhandled event {}", unhandled);
                    }
                }
            }
        }
    }

    pub fn on_map_request(&mut self, event: &xcb::MapRequestEvent) {

    }

    fn map_window(&mut self) {

    }
}