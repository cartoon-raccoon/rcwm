use xcb_util::{
    ewmh::Connection,
    cursor,
};

use crate::values::ROOT_ATTRS;
use crate::xserver::{XConn, XWindowID};
use crate::window::{Screen};
use crate::desktop::Desktop;
use crate::layout::LayoutType;

#[derive(Clone)]
#[allow(dead_code)]
pub struct WM<'a> {
    conn: XConn<'a>,
    desktop: Desktop,
    screen: Screen,
    root: i32,
    layout: LayoutType,
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

        let mut screen = Screen::new(screen_idx, root_id);

        screen.xwindow.update_geometry_conn(&xconn);

        let mut new = Self {
            conn: xconn,
            desktop: Desktop::new(),
            screen: Screen::new(screen_idx, root_id),
            root: screen_idx,
            layout: LayoutType::Floating,
        };

        for &existing in &new.conn.query_tree(root_id).unwrap() {
            let attr = if let Some(attr) = new.conn.get_window_attributes(existing) {
                attr
            } else {
                continue
            };

            if attr.override_redirect() || attr.map_state() as u32 != xcb::MAP_STATE_VIEWABLE {
                continue
            }

            debug!("Mapping window {}", existing);

            new.map_window(existing);
        }

        new

    }

    //using a mutable reference statically ensures there is only one instance running
    pub fn run(&mut self) -> ! {
        info!("Running WM");

        loop {
            let event = self.conn.next_event();

            unsafe {
                match event.response_type() & !0x80 {
                    xcb::CONFIGURE_NOTIFY => self.on_config_notify(xcb::cast_event(&event)),
                    xcb::CONFIGURE_REQUEST => self.on_config_request(xcb::cast_event(&event)),
                    xcb::MAP_REQUEST => self.on_map_request(xcb::cast_event(&event)),
                    xcb::UNMAP_NOTIFY => self.on_unmap_notify(xcb::cast_event(&event)),
                    xcb::DESTROY_NOTIFY => self.on_destroy_notify(xcb::cast_event(&event)),
                    unhandled => {
                        debug!("Unhandled event {}", unhandled);
                    }
                }
            }
        }
    }

    pub fn on_config_notify(&mut self, event: &xcb::ConfigureNotifyEvent) {
        if event.window() == self.screen.xwindow.id {
            debug!("On configure notify for root window");

            self.screen.xwindow.update_pos_x(event.x() as i32);
            self.screen.xwindow.update_pos_y(event.y() as i32);
            self.screen.xwindow.update_width(event.width() as i32);
            self.screen.xwindow.update_height(event.height() as i32);

            self.desktop.current_mut().deactivate(&self.conn);
            self.desktop.current_mut().activate(&self.conn, &self.screen);
        }
    }

    pub fn on_config_request(&mut self, event: &xcb::ConfigureRequestEvent) {
        if let Some((ws, idx)) = self.desktop.contains_mut(event.window()) {
            debug!("On configure request for window {}", event.window());

            let ref mut window = ws[idx];

            let mut values = Vec::new();

            if xcb::CONFIG_WINDOW_Y as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_Y as u16, event.x() as u32));
            }
            if xcb::CONFIG_WINDOW_X as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_X as u16, event.x() as u32));
            }
            if xcb::CONFIG_WINDOW_WIDTH as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32));
                window.xwindow.update_width(event.width() as i32);
            }
            if xcb::CONFIG_WINDOW_HEIGHT as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32));
                window.xwindow.update_height(event.height() as i32);
            }

            self.conn.configure_window(event.window(), &values);
        }
    }

    pub fn on_map_request(&mut self, event: &xcb::MapRequestEvent) {
        if self.desktop.contains(event.window()).is_none() {
            debug!("On map request for window {}", event.window());

            self.map_window(event.window());
        } else {
            debug!("Map request for existing window");
        }
    }

    fn map_window(&mut self, window: XWindowID) {
        if let Some(window_type) = self.conn.get_window_type(window) {
            if !(window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_NORMAL)||
                 window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_DIALOG)||
                 window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_SPLASH)||
                window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_UTILITY)||
                window_type.contains(&self.conn.atoms.WM_WINDOW_TYPE_TOOLBAR)
                ) {
                debug!("Mapping but not tracking window {}", window);

                self.conn.map_window(window);
                return
            }
        }

        self.desktop.current_mut().add_window(&self.conn, &self.screen, window);
    }

    pub fn on_unmap_notify(&mut self, event: &xcb::UnmapNotifyEvent) {
        debug!("On unmap notify");
        self.unmap_window(event.window());
    }

    pub fn on_destroy_notify(&mut self, event: &xcb::DestroyNotifyEvent) {
        debug!("On destroy notify");
        self.unmap_window(event.window());
    }

    fn unmap_window(&mut self, window: XWindowID) {
        if let Some((ws, idx)) = self.desktop.contains_mut(window) {
            debug!("Unmap notify for window {}", window);
            ws.del_window(&self.conn, &self.screen, window, idx);
        } else {
            debug!("Unmap notify for untracked window {}", window)
        }
    }
}