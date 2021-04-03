use xcb_util::{
    ewmh::Connection,
    cursor,
};

use crate::values;
use crate::x::core::{XConn, XWindowID};
use crate::x::Ewmh;
use crate::desktop::{Desktop, Screen};
use crate::layout::LayoutType;
use crate::keys;

/// Whether the mouse button is pressed.
enum MouseMode {
    None,
    Move,
    Resize,
}

#[allow(dead_code)]
pub struct WM<'a> {
    pub conn: XConn<'a>,
    pub desktop: Desktop,
    pub screen: Screen,
    root: i32,
    mousemode: MouseMode,
    selected: Option<XWindowID>,
    last_mouse_x: i32,
    last_mouse_y: i32,
    to_quit: bool,
}

impl<'a> WM<'a> {
    pub fn register(conn: &'a Connection, screen_idx: i32) -> Self {
        let mut xconn = XConn::new(conn, screen_idx);

        let root_id = xconn.get_root_id();

        debug!("Got root id of {}", root_id);

        // register for substructure redirect and substructure notify on root window 
        xconn.change_window_attributes_checked(root_id, &values::ROOT_ATTRS)
            // we panic here because this is a fatal error
            .unwrap_or_else(|_e| {
                error!("Another window manager is running.");
                std::process::exit(1)
            });

        xconn.set_supported(screen_idx, &[
            xconn.atoms.WM_PROTOCOLS, 
            xconn.atoms.WM_DELETE_WINDOW
        ]);

        xconn.grab_button(root_id, values::ROOT_BUTTON_GRAB_MASK, xcb::BUTTON_INDEX_1, xcb::MOD_MASK_4, true);
        xconn.grab_button(root_id, values::ROOT_BUTTON_GRAB_MASK, xcb::BUTTON_INDEX_3, xcb::MOD_MASK_4, true);

        for (mask, ks, _) in keys::KEYBINDS {
            xconn.grab_key(root_id, *mask, *ks);
        }

        // fatal because this is the WM setup process
        xconn.create_cursor(cursor::LEFT_PTR)
            .unwrap_or_else(|_e| {
                fatal!("Could not create cursor")
        });

        xconn.set_cursor(root_id);

        let mut screen = Screen::new(screen_idx, root_id);

        screen.xwindow.set_geometry_conn(&xconn);

        let mut new = Self {
            conn: xconn,
            desktop: Desktop::new(LayoutType::DTiled),
            screen: screen,
            root: screen_idx,
            mousemode: MouseMode::None,
            selected: None,
            last_mouse_x: 0,
            last_mouse_y: 0,
            to_quit: false,
        };

        // find existing windows and map them
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
    pub fn run(&mut self) {
        info!("Running WM");

        self.desktop.current_mut().activate(&self.conn, &self.screen);

        loop {
            let event = self.conn.next_event();

            unsafe {
                match event.response_type() & !0x80 {
                    xcb::CONFIGURE_NOTIFY => self.on_config_notify(xcb::cast_event(&event)),
                    xcb::CONFIGURE_REQUEST => self.on_config_request(xcb::cast_event(&event)),
                    xcb::MAP_REQUEST => self.on_map_request(xcb::cast_event(&event)),
                    xcb::UNMAP_NOTIFY => self.on_unmap_notify(xcb::cast_event(&event)),
                    xcb::DESTROY_NOTIFY => self.on_destroy_notify(xcb::cast_event(&event)),
                    xcb::ENTER_NOTIFY => self.on_enter_notify(xcb::cast_event(&event)),
                    xcb::LEAVE_NOTIFY => {debug!("Leave notify")}
                    xcb::MOTION_NOTIFY => self.on_motion_notify(xcb::cast_event(&event)),
                    xcb::REPARENT_NOTIFY => {debug!("Reparent notify")}
                    xcb::KEY_PRESS => self.on_key_press(xcb::cast_event(&event)),
                    xcb::BUTTON_PRESS => self.on_button_press(xcb::cast_event(&event)),
                    xcb::BUTTON_RELEASE => self.on_button_release(xcb::cast_event(&event)),
                    xcb::CLIENT_MESSAGE => {debug!("Client message")}
                    unhandled => {
                        debug!("Unhandled event {}", unhandled);
                    }
                }
            }

            if self.to_quit {
                info!("Quitting!");
                break;
            }
        }
    }

    pub fn goto_workspace(&mut self, idx: usize) {
        self.desktop.goto(&self.conn, &self.screen, idx);
    }

    pub fn send_window_to(&mut self, idx: usize) {
        self.desktop.send_window_to(&self.conn, &self.screen, idx);
    }

    pub fn on_config_notify(&mut self, event: &xcb::ConfigureNotifyEvent) {
        if event.window() == self.screen.xwindow.id {
            debug!("On configure notify for root window");

            self.screen.xwindow.geom.x = event.x() as i32;
            self.screen.xwindow.geom.y = event.y() as i32;
            self.screen.xwindow.geom.width = event.width() as i32;
            self.screen.xwindow.geom.height = event.height() as i32;

            self.desktop.current_mut().deactivate(&self.conn);
            self.desktop.current_mut().activate(&self.conn, &self.screen);
        }
    }

    pub fn on_config_request(&mut self, event: &xcb::ConfigureRequestEvent) {
        if let Some((ws, idx)) = self.desktop.contains_mut(event.window()) {
            debug!("On configure request for window {}", event.window());

            let is_tiling = ws.is_tiling();
            let ref mut window = ws[idx];

            // if we are tiling the window
            if is_tiling && window.is_tiled() {
                // reject the request
                debug!("Workspace is tiling, rejecting request");
                // send back unchanged geometry
                window.update_geometry(&self.conn);
                return
            }
            
            let mut values = Vec::new();

            if xcb::CONFIG_WINDOW_Y as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_Y as u16, event.x() as u32));
                window.xwindow.geom.y = event.y() as i32;
            }
            if xcb::CONFIG_WINDOW_X as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_X as u16, event.x() as u32));
                window.xwindow.geom.x = event.x() as i32;
            }
            if xcb::CONFIG_WINDOW_WIDTH as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_WIDTH as u16, event.width() as u32));
                window.xwindow.geom.width = event.width() as i32;
            }
            if xcb::CONFIG_WINDOW_HEIGHT as u16 & event.value_mask() != 0 {
                values.push((xcb::CONFIG_WINDOW_HEIGHT as u16, event.height() as u32));
                window.xwindow.geom.height = event.height() as i32;
            }

            self.conn.configure_window(event.window(), &values);
            dbg!(window);
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

    pub fn on_enter_notify(&mut self, event: &xcb::EnterNotifyEvent) {
        
        if !(event.mode() as u32 == xcb::NOTIFY_MODE_NORMAL ||
             event.mode() as u32 == xcb::NOTIFY_MODE_UNGRAB) {
            return
        }
        
        if self.desktop.current().contains(event.event()).is_some() {
            debug!("On enter notify for {}", event.event());
            self.desktop.current_mut().focus_window(&self.conn, &self.screen, event.event())
        } else {
            warn!("On enter notify for untracked window {}", event.event());
        }
    }

    pub fn on_key_press(&mut self, event: &xcb::KeyPressEvent) {
        debug!("Button press for window {}", event.event());

        let (modm, keysym) = self.conn.lookup_keysym(event);

        if let Some((_, _, cb)) = keys::find_keybind(modm, keysym) {
            debug!("Found keybind");
            cb(self);
            return
        }

        debug!("No keybind found for key press event")
    }

    pub fn on_button_press(&mut self, event: &xcb::ButtonPressEvent) {
        debug!("Button press for window {}", event.event());
        if event.child() == xcb::NONE {
            return
        }

        self.selected = Some(event.child());

        self.conn.grab_pointer(self.screen.xwindow.id, values::ROOT_POINTER_GRAB_MASK);

        self.last_mouse_x = event.root_x() as i32;
        self.last_mouse_y = event.root_y() as i32;

        if !self.desktop.current().windows.is_focused(event.child()) {
            self.desktop.current_mut().focus_window(&self.conn, &self.screen, event.child());
        }

        match event.detail() as u32 {
            xcb::BUTTON_INDEX_1 => {
                self.mousemode = MouseMode::Move;
            }
            xcb::BUTTON_INDEX_3 => {
                self.mousemode = MouseMode::Resize;
            }
            xcb::BUTTON_INDEX_2 => {
                debug!("Middle mouse button selected")
            }
            _ => {
                warn!("Unhandled mouse button event")
            }

        }
    }

    pub fn on_button_release(&mut self, event: &xcb::ButtonReleaseEvent) {
        debug!("Button release for {}", event.event());

        self.selected = None;
        self.mousemode = MouseMode::None;

        self.conn.ungrab_pointer();
    }

    pub fn on_motion_notify(&mut self, event: &xcb::MotionNotifyEvent) {
        if let Some(selected) = self.selected {
            // focus the window
            self.desktop.current_mut().focus_window(&self.conn, &self.screen, selected);
            // stack the window on top
            self.desktop.current_mut().windows.focused().unwrap()
                .configure(&self.conn, &values::stack_above());
            debug!("On motion notify");

            let dx = event.root_x() as i32 - self.last_mouse_x;
            let dy = event.root_y() as i32 - self.last_mouse_y;

            self.last_mouse_x = event.root_x() as i32;
            self.last_mouse_y = event.root_y() as i32;

            if let Some(idx) = self.desktop.current().windows.contains(selected) {
                let selected = self.desktop.current_mut().windows.get_mut(idx).unwrap();
                match self.mousemode {
                    MouseMode::None => {
                        error!("Encountered no button press while grabbing mouse")
                    }
                    MouseMode::Move => {
                        selected.do_move(&self.conn, &self.screen, dx, dy);
                        if selected.is_tiled() {
                            self.desktop.current_mut().toggle_focused_state(&self.conn, &self.screen);
                            self.desktop.current_mut().relayout(&self.conn, &self.screen);
                            assert!(self.desktop.current().is_tiling());
                        }
                    }
                    MouseMode::Resize => {
                        selected.do_resize(&self.conn, &self.screen, dx, dy);
                        if selected.is_tiled() {
                            self.desktop.current_mut().toggle_focused_state(&self.conn, &self.screen);
                            self.desktop.current_mut().relayout(&self.conn, &self.screen);
                            assert!(self.desktop.current().is_tiling());
                        }
                    }
                }
            }

        } else {
            return
        }
    }

    pub fn quit(&mut self) {

        // we use a field to mark a flag for quitting
        // so that instead of exiting on the spot, we can instead
        // break the loop and thereby run cleanup code if we need to
        self.to_quit = true;
    }
}